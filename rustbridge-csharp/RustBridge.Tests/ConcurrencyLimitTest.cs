using RustBridge.Native;

namespace RustBridge.Tests;

/// <summary>
/// Tests for concurrency limiting and backpressure.
/// <para>
/// Reference: Java ConcurrencyLimitTest.java
/// </para>
/// </summary>
[Trait("Category", "Integration")]
public class ConcurrencyLimitTest : IDisposable
{
    private readonly string? _pluginPath;
    private readonly string? _skipReason;

    public ConcurrencyLimitTest()
    {
        _pluginPath = FindHelloPlugin();
        if (_pluginPath == null)
        {
            _skipReason = "hello-plugin not found. Run: cargo build --release -p hello-plugin";
        }
    }

    public void Dispose()
    {
        // Nothing to dispose at test class level
    }

    private static string? FindHelloPlugin()
    {
        var libraryName = GetLibraryFileName("hello_plugin");
        var assemblyLocation = typeof(ConcurrencyLimitTest).Assembly.Location;
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

    [SkippableFact]
    public async Task ConcurrencyLimit___Exceeded___ReturnsError()
    {
        SkipIfPluginNotAvailable();

        const int concurrencyLimit = 2;
        const int additionalRequests = 5;
        var config = PluginConfig.Defaults().MaxConcurrentOps(concurrencyLimit);

        using var plugin = NativePluginLoader.Load(_pluginPath!, config);

        var blockingSuccessCount = 0;
        var additionalErrorCount = 0;
        var lockObj = new object();

        // Phase 1: Start blocking calls that will hold all permits
        // These sleep for 3 seconds, holding permits the entire time
        var blockingTasks = new List<Task>();
        for (int i = 0; i < concurrencyLimit; i++)
        {
            var task = Task.Run(() =>
            {
                plugin.Call("test.sleep", """{"duration_ms": 3000}""");
                lock (lockObj) { blockingSuccessCount++; }
            });
            blockingTasks.Add(task);
        }

        // Phase 2: Wait for blocking calls to acquire permits
        // Give them time to enter the sleep (500ms is plenty for FFI overhead)
        await Task.Delay(500);

        // Phase 3: Try additional requests - these should all be rejected
        // since all permits are held by the blocking calls
        var additionalTasks = new List<Task>();
        for (int i = 0; i < additionalRequests; i++)
        {
            var task = Task.Run(() =>
            {
                try
                {
                    plugin.Call("greet", """{"name": "ShouldFail"}""");
                }
                catch (PluginException)
                {
                    lock (lockObj) { additionalErrorCount++; }
                }
            });
            additionalTasks.Add(task);
        }

        // Wait for additional requests to complete (they should fail fast)
        await Task.WhenAll(additionalTasks);

        // Verify all additional requests were rejected
        Assert.Equal(additionalRequests, additionalErrorCount);
        Assert.Equal(additionalRequests, plugin.RejectedRequestCount);

        Console.WriteLine($"Rejected count: {plugin.RejectedRequestCount}");

        // Wait for blocking tasks to complete
        await Task.WhenAll(blockingTasks);
        Assert.Equal(concurrencyLimit, blockingSuccessCount);
    }

    [SkippableFact]
    public async Task ConcurrencyLimit___Unlimited___AllSucceed()
    {
        SkipIfPluginNotAvailable();

        var config = PluginConfig.Defaults().MaxConcurrentOps(0); // Unlimited

        using var plugin = NativePluginLoader.Load(_pluginPath!, config);

        var successCount = 0;
        var errorCount = 0;
        var lockObj = new object();

        // Submit many concurrent requests
        var tasks = new List<Task>();
        for (int i = 0; i < 20; i++)
        {
            var id = i;
            var task = Task.Run(() =>
            {
                try
                {
                    plugin.Call("greet", $$$"""{"name": "User{{{id}}}"}""");
                    lock (lockObj) { successCount++; }
                }
                catch (PluginException)
                {
                    lock (lockObj) { errorCount++; }
                    throw;
                }
            });
            tasks.Add(task);
        }

        await Task.WhenAll(tasks);

        // All should succeed
        Assert.Equal(20, successCount);
        Assert.Equal(0, errorCount);
        Assert.Equal(0, plugin.RejectedRequestCount);
    }

    [SkippableFact]
    public void ConcurrencyLimit___PermitReleased___AfterCompletion()
    {
        SkipIfPluginNotAvailable();

        var config = PluginConfig.Defaults().MaxConcurrentOps(1);

        using var plugin = NativePluginLoader.Load(_pluginPath!, config);

        // Make several sequential calls
        for (int i = 0; i < 10; i++)
        {
            var result = plugin.Call("greet", $$$"""{"name": "User{{{i}}}"}""");
            Assert.NotNull(result);
        }

        // All should succeed since they're sequential
        Assert.Equal(0, plugin.RejectedRequestCount);
    }

    [SkippableFact]
    public async Task RejectedRequestCount___TracksRejectedRequests()
    {
        SkipIfPluginNotAvailable();

        const int concurrencyLimit = 2;
        const int additionalRequests = 10;
        var config = PluginConfig.Defaults().MaxConcurrentOps(concurrencyLimit);

        using var plugin = NativePluginLoader.Load(_pluginPath!, config);

        // Phase 1: Start blocking calls that will hold all permits
        var blockingTasks = new List<Task>();
        for (int i = 0; i < concurrencyLimit; i++)
        {
            var task = Task.Run(() =>
            {
                try
                {
                    plugin.Call("test.sleep", """{"duration_ms": 2000}""");
                }
                catch (PluginException)
                {
                    // Unexpected, but don't fail the test
                }
            });
            blockingTasks.Add(task);
        }

        // Phase 2: Wait for blocking calls to acquire permits
        await Task.Delay(500);

        // Phase 3: Try additional requests - these should all be rejected
        var additionalTasks = new List<Task>();
        for (int i = 0; i < additionalRequests; i++)
        {
            var task = Task.Run(() =>
            {
                try
                {
                    plugin.Call("greet", """{"name": "ShouldFail"}""");
                }
                catch (PluginException)
                {
                    // Expected
                }
            });
            additionalTasks.Add(task);
        }

        await Task.WhenAll(additionalTasks);

        // Verify rejected count is tracked correctly
        var rejectedCount = plugin.RejectedRequestCount;
        Assert.Equal(additionalRequests, rejectedCount);

        Console.WriteLine($"Rejected count: {rejectedCount}");

        // Wait for blocking tasks to complete
        await Task.WhenAll(blockingTasks);
    }
}
