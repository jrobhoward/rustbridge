using BenchmarkDotNet.Attributes;
using RustBridge.Native;

namespace RustBridge.Benchmarks;

/// <summary>
/// Concurrent benchmarks measuring multi-threaded scalability.
/// <para>
/// Run with: dotnet run -c Release -- --filter "*ConcurrentBenchmark*"
/// </para>
/// </summary>
[MemoryDiagnoser]
[RankColumn]
public class ConcurrentBenchmark : IDisposable
{
    private const int MsgBenchSmall = 1;
    private const string JsonRequest = """{"message": "concurrent test"}""";
    private const int ConcurrentTasks = 100;

    private IPlugin _plugin = null!;

    [GlobalSetup]
    public void Setup()
    {
        var libraryPath = BenchmarkHelper.GetHelloPluginOrThrow();
        // Use more worker threads for concurrent tests
        var config = PluginConfig.Defaults().WorkerThreads(8);
        _plugin = NativePluginLoader.Load(libraryPath, config);
    }

    [GlobalCleanup]
    public void Cleanup()
    {
        _plugin?.Dispose();
    }

    public void Dispose()
    {
        Cleanup();
    }

    [Benchmark(Baseline = true, Description = "JSON concurrent (100 tasks)")]
    public async Task JsonConcurrent()
    {
        var tasks = new Task<string>[ConcurrentTasks];
        for (int i = 0; i < ConcurrentTasks; i++)
        {
            tasks[i] = Task.Run(() => _plugin.Call("echo", JsonRequest));
        }
        await Task.WhenAll(tasks);
    }

    [Benchmark(Description = "Binary concurrent (100 tasks)")]
    public async Task BinaryConcurrent()
    {
        var tasks = new Task<SmallResponseRaw>[ConcurrentTasks];
        for (int i = 0; i < ConcurrentTasks; i++)
        {
            tasks[i] = Task.Run(() =>
            {
                var request = SmallRequestRaw.Create("bench_key", 0x01);
                return _plugin.CallRaw<SmallRequestRaw, SmallResponseRaw>(MsgBenchSmall, request);
            });
        }
        await Task.WhenAll(tasks);
    }
}
