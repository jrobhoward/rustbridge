using BenchmarkDotNet.Attributes;
using RustBridge.Native;

namespace RustBridge.Benchmarks;

/// <summary>
/// Throughput benchmarks measuring operations per second.
/// <para>
/// Run with: dotnet run -c Release -- --filter "*ThroughputBenchmark*"
/// </para>
/// </summary>
[MemoryDiagnoser]
[RankColumn]
public class ThroughputBenchmark : IDisposable
{
    private const int MsgBenchSmall = 1;
    private const string JsonRequest = """{"message": "throughput test"}""";
    private const int OperationsPerInvoke = 1000;

    private IPlugin _plugin = null!;

    [GlobalSetup]
    public void Setup()
    {
        var libraryPath = BenchmarkHelper.GetHelloPluginOrThrow();
        var config = PluginConfig.Defaults().WithWorkerThreads(4);
        _plugin = NativePluginLoader.Load(libraryPath, config);
    }

    [GlobalCleanup]
    public void Cleanup()
    {
        _plugin?.Dispose();
    }

    [Benchmark(Baseline = true, OperationsPerInvoke = OperationsPerInvoke, Description = "JSON throughput")]
    public void JsonThroughput()
    {
        for (int i = 0; i < OperationsPerInvoke; i++)
        {
            _plugin.Call("echo", JsonRequest);
        }
    }

    [Benchmark(OperationsPerInvoke = OperationsPerInvoke, Description = "Binary throughput")]
    public void BinaryThroughput()
    {
        for (int i = 0; i < OperationsPerInvoke; i++)
        {
            var request = SmallRequestRaw.Create("bench_key", 0x01);
            _plugin.CallRaw<SmallRequestRaw, SmallResponseRaw>(MsgBenchSmall, request);
        }
    }
}
