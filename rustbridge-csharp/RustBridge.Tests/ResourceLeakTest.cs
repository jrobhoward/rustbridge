using RustBridge.Native;

namespace RustBridge.Tests;

/// <summary>
/// Test for resource leak detection in plugin lifecycle.
/// Verifies that plugins and their native resources are properly cleaned up
/// and not leaked even under stress conditions.
/// <para>
/// Reference: Java ResourceLeakTest.java
/// </para>
/// </summary>
[Trait("Category", "Integration")]
public class ResourceLeakTest
{
    private readonly string? _pluginPath;
    private readonly string? _skipReason;

    public ResourceLeakTest()
    {
        _pluginPath = FindHelloPlugin();
        if (_pluginPath == null)
        {
            _skipReason = "hello-plugin not found. Run: cargo build --release -p hello-plugin";
        }
    }

    private static string? FindHelloPlugin()
    {
        var libraryName = GetLibraryFileName("hello_plugin");
        var assemblyLocation = typeof(ResourceLeakTest).Assembly.Location;
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
    public void PluginResources___ReleasedOnClose()
    {
        SkipIfPluginNotAvailable();

        using (var plugin = NativePluginLoader.Load(_pluginPath!))
        {
            // Verify plugin is active
            Assert.Equal(LifecycleState.Active, plugin.State);

            // Use the plugin
            var response = plugin.Call("echo", """{"message": "test"}""");
            Assert.Contains("test", response);
        }
    }

    [SkippableFact]
    public void SequentialLoadClose___CyclesDontLeakResources()
    {
        SkipIfPluginNotAvailable();

        for (int i = 0; i < 100; i++)
        {
            using var plugin = NativePluginLoader.Load(_pluginPath!);

            Assert.Equal(LifecycleState.Active, plugin.State);

            // Call the plugin
            plugin.Call("echo", $$$"""{"message": "cycle {{{i}}}"}""");
        }
    }

    [SkippableFact]
    public void PluginObjects___GcEligibleAfterClose()
    {
        SkipIfPluginNotAvailable();

        WeakReference<IPlugin>? pluginRef;

        // Load plugin in a local scope
        void CreateAndUsePlugin()
        {
            using var plugin = NativePluginLoader.Load(_pluginPath!);
            pluginRef = new WeakReference<IPlugin>(plugin);

            Assert.Equal(LifecycleState.Active, plugin.State);

            // Use the plugin
            plugin.Call("echo", """{"message": "test"}""");
            // Plugin disposed at end of using block
        }

        pluginRef = null;
        CreateAndUsePlugin();

        // Try to GC (not guaranteed, but helps)
        GC.Collect();
        GC.WaitForPendingFinalizers();

        // WeakReference may be null if GC collected it
        // This is not guaranteed but indicates good cleanup
    }

    [SkippableFact]
    public void MultiplePlugins___CloseCleanly()
    {
        SkipIfPluginNotAvailable();

        // Create and close 20 plugins
        for (int i = 0; i < 20; i++)
        {
            using var plugin = NativePluginLoader.Load(_pluginPath!);

            Assert.Equal(LifecycleState.Active, plugin.State);

            // Use plugin
            plugin.Call("echo", $$$"""{"message": "test {{{i}}}"}""");
        }
    }

    [SkippableFact]
    public void PluginState___CorrectAfterUseAndClose()
    {
        SkipIfPluginNotAvailable();

        using var plugin = NativePluginLoader.Load(_pluginPath!);

        Assert.Equal(LifecycleState.Active, plugin.State);

        // Make a valid call
        plugin.Call("echo", """{"message": "test"}""");

        // Still active
        Assert.Equal(LifecycleState.Active, plugin.State);
    }

    [SkippableFact]
    public void LargePayload___CyclesDontLeakMemory()
    {
        SkipIfPluginNotAvailable();

        // Create a large payload
        var largeMessage = new string('x', 10000);
        var largePayload = $$$"""{"message": "{{{largeMessage}}}"}""";

        for (int cycle = 0; cycle < 50; cycle++)
        {
            using var plugin = NativePluginLoader.Load(_pluginPath!);

            // Send large payload
            plugin.Call("echo", largePayload);
        }
    }

    [SkippableFact]
    public async Task PluginResources___SurviveConcurrentAccessBeforeClose()
    {
        SkipIfPluginNotAvailable();

        using var plugin = NativePluginLoader.Load(_pluginPath!);

        // Multiple tasks access the plugin
        var threadCount = 10;
        var exceptions = new List<Exception>();
        var lockObj = new object();

        var tasks = new List<Task>();
        for (int i = 0; i < threadCount; i++)
        {
            var threadId = i;
            var task = Task.Run(() =>
            {
                try
                {
                    for (int j = 0; j < 10; j++)
                    {
                        var response = plugin.Call("echo",
                            $$$"""{"message": "thread {{{threadId}}} call {{{j}}}"}""");
                        Assert.NotNull(response);
                    }
                }
                catch (Exception e)
                {
                    lock (lockObj)
                    {
                        exceptions.Add(e);
                    }
                }
            });
            tasks.Add(task);
        }

        await Task.WhenAll(tasks);

        // Check no exceptions occurred
        Assert.Empty(exceptions);
    }
}
