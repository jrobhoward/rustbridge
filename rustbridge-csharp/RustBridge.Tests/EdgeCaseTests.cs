using RustBridge.Native;

namespace RustBridge.Tests;

/// <summary>
/// Edge case and error handling tests.
/// <para>
/// Tests for error conditions, race conditions, and edge cases that are
/// critical for production reliability.
/// </para>
/// </summary>
[Trait("Category", "EdgeCase")]
public class EdgeCaseTests : IDisposable
{
    private readonly IPlugin? _plugin;
    private readonly string? _skipReason;
    private readonly string? _libraryPath;

    public EdgeCaseTests()
    {
        _libraryPath = FindHelloPlugin();
        if (_libraryPath == null)
        {
            _skipReason = "hello-plugin not found. Run: cargo build --release -p hello-plugin";
            return;
        }

        try
        {
            _plugin = NativePluginLoader.Load(_libraryPath);
        }
        catch (Exception ex)
        {
            _skipReason = $"Failed to load plugin: {ex.Message}";
        }
    }

    public void Dispose()
    {
        _plugin?.Dispose();
    }

    private static string? FindHelloPlugin()
    {
        var libraryName = GetLibraryFileName("hello_plugin");
        var assemblyLocation = typeof(EdgeCaseTests).Assembly.Location;
        var assemblyDir = Path.GetDirectoryName(assemblyLocation) ?? ".";

        var searchBases = new[]
        {
            Environment.CurrentDirectory,
            assemblyDir,
            Path.Combine(assemblyDir, "..", "..", "..", ".."),
            Path.Combine(assemblyDir, "..", "..", "..", "..", ".."),
        };

        foreach (var baseDir in searchBases)
        {
            var releasePath = Path.Combine(baseDir, "target", "release", libraryName);
            if (File.Exists(releasePath))
            {
                return Path.GetFullPath(releasePath);
            }

            var debugPath = Path.Combine(baseDir, "target", "debug", libraryName);
            if (File.Exists(debugPath))
            {
                return Path.GetFullPath(debugPath);
            }
        }

        return null;
    }

    private static string GetLibraryFileName(string name)
    {
        if (OperatingSystem.IsWindows()) return $"{name}.dll";
        if (OperatingSystem.IsMacOS()) return $"lib{name}.dylib";
        return $"lib{name}.so";
    }

    private void SkipIfPluginNotAvailable()
    {
        Skip.If(_skipReason != null, _skipReason);
    }

    // ==================== Missing/Invalid Library Tests ====================

    [Fact]
    public void Load___NonexistentPath___ThrowsPluginException()
    {
        var nonexistentPath = "/nonexistent/path/to/libfake_plugin.so";

        var exception = Assert.Throws<PluginException>(() =>
        {
            NativePluginLoader.Load(nonexistentPath);
        });

        Assert.Contains("Failed to load", exception.Message);
    }

    [Fact]
    public void Load___InvalidFilePath___ThrowsPluginException()
    {
        // Create a temp file that is NOT a valid shared library
        var tempFile = Path.GetTempFileName();
        try
        {
            File.WriteAllText(tempFile, "This is not a valid library");

            var exception = Assert.Throws<PluginException>(() =>
            {
                NativePluginLoader.Load(tempFile);
            });

            Assert.Contains("Failed to load", exception.Message);
        }
        finally
        {
            File.Delete(tempFile);
        }
    }

    [Fact]
    public void Load___EmptyPath___ThrowsPluginException()
    {
        var exception = Assert.Throws<PluginException>(() =>
        {
            NativePluginLoader.Load("");
        });

        Assert.Contains("Failed to load", exception.Message);
    }

    // ==================== Concurrent Dispose Tests ====================

    [SkippableFact]
    public async Task Dispose___DuringActiveCall___CompletesGracefully()
    {
        SkipIfPluginNotAvailable();
        Skip.If(_libraryPath == null);

        // Create a fresh plugin for this test
        using var plugin = NativePluginLoader.Load(_libraryPath);

        // Start a slow call in a background task
        var callTask = Task.Run(() =>
        {
            try
            {
                // This call should either complete or fail gracefully when disposed
                plugin.Call("test.sleep", """{"duration_ms": 500}""");
                return true;
            }
            catch (ObjectDisposedException)
            {
                // Expected if dispose happens during call
                return false;
            }
            catch (PluginException)
            {
                // Also acceptable - plugin may reject call during shutdown
                return false;
            }
        });

        // Give the call time to start
        await Task.Delay(50);

        // Dispose while call is in-flight - should not deadlock or crash
        plugin.Dispose();

        // Wait for the call task with timeout
        var completed = await Task.WhenAny(callTask, Task.Delay(5000)) == callTask;

        Assert.True(completed, "Call task should complete (success or failure) within timeout");
    }

    [SkippableFact]
    public async Task Dispose___ConcurrentFromMultipleThreads___IsThreadSafe()
    {
        SkipIfPluginNotAvailable();
        Skip.If(_libraryPath == null);

        using var plugin = NativePluginLoader.Load(_libraryPath);

        // Try to dispose from multiple threads simultaneously
        var tasks = Enumerable.Range(0, 5).Select(_ => Task.Run(() =>
        {
            // Should not throw or crash even if called concurrently
            plugin.Dispose();
        })).ToArray();

        await Task.WhenAll(tasks);

        // Verify plugin is in disposed state
        Assert.Equal(LifecycleState.Stopped, plugin.State);
    }

    [SkippableFact]
    public void Call___AfterDispose___ThrowsObjectDisposedException()
    {
        SkipIfPluginNotAvailable();
        Skip.If(_libraryPath == null);

        var plugin = NativePluginLoader.Load(_libraryPath);
        plugin.Dispose();

        Assert.Throws<ObjectDisposedException>(() =>
        {
            plugin.Call("echo", """{"message": "test"}""");
        });
    }

    [SkippableFact]
    public void SetLogLevel___AfterDispose___ThrowsObjectDisposedException()
    {
        SkipIfPluginNotAvailable();
        Skip.If(_libraryPath == null);

        var plugin = NativePluginLoader.Load(_libraryPath);
        plugin.Dispose();

        Assert.Throws<ObjectDisposedException>(() =>
        {
            plugin.SetLogLevel(LogLevel.Debug);
        });
    }

    [SkippableFact]
    public void State___AfterDispose___ReturnsStopped()
    {
        SkipIfPluginNotAvailable();
        Skip.If(_libraryPath == null);

        var plugin = NativePluginLoader.Load(_libraryPath);
        plugin.Dispose();

        // State should be accessible and return Stopped
        Assert.Equal(LifecycleState.Stopped, plugin.State);
    }

    // ==================== Double Dispose Tests ====================

    [SkippableFact]
    public void Dispose___CalledTwice___IsIdempotent()
    {
        SkipIfPluginNotAvailable();
        Skip.If(_libraryPath == null);

        var plugin = NativePluginLoader.Load(_libraryPath);

        // First dispose
        plugin.Dispose();

        // Second dispose should not throw
        plugin.Dispose();

        Assert.Equal(LifecycleState.Stopped, plugin.State);
    }

    // ==================== Using Statement Exception Safety ====================

    [SkippableFact]
    public void UsingStatement___ExceptionInBlock___PluginStillDisposed()
    {
        SkipIfPluginNotAvailable();
        Skip.If(_libraryPath == null);

        IPlugin? capturedPlugin = null;

        try
        {
            using var plugin = NativePluginLoader.Load(_libraryPath);
            capturedPlugin = plugin;

            // Verify plugin is active
            Assert.Equal(LifecycleState.Active, plugin.State);

            // Throw exception inside using block
            throw new InvalidOperationException("Intentional test exception");
        }
        catch (InvalidOperationException)
        {
            // Expected
        }

        // Plugin should be disposed even after exception
        Assert.NotNull(capturedPlugin);
        Assert.Equal(LifecycleState.Stopped, capturedPlugin.State);
    }
}
