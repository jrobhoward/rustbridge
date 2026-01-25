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

        var config = PluginConfig.Defaults().WithMaxConcurrentOps(2);

        using var plugin = NativePluginLoader.Load(_pluginPath!, config);

        var successCount = 0;
        var errorCount = 0;
        var lockObj = new object();

        // Submit 15 requests, staggered to ensure we hit the limit
        var tasks = new List<Task>();
        for (int i = 0; i < 15; i++)
        {
            var task = Task.Run(async () =>
            {
                try
                {
                    // Use sleep handler to hold permits longer (300ms)
                    plugin.Call("test.sleep", """{"duration_ms": 300}""");
                    lock (lockObj) { successCount++; }
                }
                catch (PluginException)
                {
                    lock (lockObj) { errorCount++; }
                }
            });
            tasks.Add(task);

            // Small delay to stagger requests
            await Task.Delay(10);
        }

        await Task.WhenAll(tasks);

        Console.WriteLine($"Success: {successCount}, Errors: {errorCount}");

        // With limit of 2 and 15 requests staggered by 10ms with 300ms sleep each:
        // Expected: 2-6 succeed (first batch), 9+ rejected
        var total = successCount + errorCount;
        Assert.Equal(15, total);
        Assert.InRange(successCount, 2, 6);
        Assert.True(errorCount >= 9, $"Expected at least 9 rejected requests, got {errorCount}");

        // Check rejected count
        var rejectedCount = plugin.RejectedRequestCount;
        Assert.True(rejectedCount >= 9, $"Rejected count should be at least 9, got {rejectedCount}");
        Assert.Equal(errorCount, rejectedCount);

        Console.WriteLine($"Rejected count: {rejectedCount}");
    }

    [SkippableFact]
    public async Task ConcurrencyLimit___Unlimited___AllSucceed()
    {
        SkipIfPluginNotAvailable();

        var config = PluginConfig.Defaults().WithMaxConcurrentOps(0); // Unlimited

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

        var config = PluginConfig.Defaults().WithMaxConcurrentOps(1);

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

        var config = PluginConfig.Defaults().WithMaxConcurrentOps(2);

        using var plugin = NativePluginLoader.Load(_pluginPath!, config);

        // Use a barrier to release all threads at once to create contention
        using var barrier = new Barrier(20);

        var tasks = new List<Task>();
        for (int i = 0; i < 20; i++)
        {
            var id = i;
            var task = Task.Run(() =>
            {
                try
                {
                    // Wait for all threads to be ready
                    barrier.SignalAndWait(TimeSpan.FromSeconds(5));
                    // Use sleep handler to hold permits longer (100ms)
                    plugin.Call("test.sleep", """{"duration_ms": 100}""");
                }
                catch (PluginException)
                {
                    // Ignore errors for this test
                }
                catch (BarrierPostPhaseException)
                {
                    // Ignore
                }
            });
            tasks.Add(task);
        }

        await Task.WhenAll(tasks);

        // Check that some requests were rejected
        var rejectedCount = plugin.RejectedRequestCount;
        Assert.True(rejectedCount > 0,
            $"Some requests should have been rejected with limit of 2 and 20 concurrent requests, got {rejectedCount}");

        Console.WriteLine($"Rejected count: {rejectedCount}");
    }
}
