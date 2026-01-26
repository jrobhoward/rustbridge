using System.Runtime.InteropServices;
using BenchmarkDotNet.Attributes;
using RustBridge.Native;

namespace RustBridge.Benchmarks;

/// <summary>
/// Latency benchmarks comparing JSON and binary transport.
/// <para>
/// Run with: dotnet run -c Release -- --filter "*TransportBenchmark*"
/// </para>
/// </summary>
[MemoryDiagnoser]
[RankColumn]
public class TransportBenchmark : IDisposable
{
    private const int MsgBenchSmall = 1;
    private const string JsonRequest = """{"message": "benchmark test"}""";

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

    [Benchmark(Baseline = true, Description = "JSON transport")]
    public string Json()
    {
        return _plugin.Call("echo", JsonRequest);
    }

    [Benchmark(Description = "Binary transport")]
    public SmallResponseRaw Binary()
    {
        var request = SmallRequestRaw.Create("bench_key", 0x01);
        return _plugin.CallRaw<SmallRequestRaw, SmallResponseRaw>(MsgBenchSmall, request);
    }
}

/// <summary>
/// Small benchmark request (C struct version).
/// Must match Rust SmallRequestRaw layout exactly.
/// </summary>
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public unsafe struct SmallRequestRaw : IBinaryStruct
{
    public const byte CurrentVersion = 1;
    private const int KeyBufferSize = 64;

    public byte Version;
    private fixed byte _reserved[3];
    private fixed byte _key[KeyBufferSize];
    public uint KeyLen;
    public uint Flags;

    public int ByteSize => 76; // 1 + 3 + 64 + 4 + 4

    public static SmallRequestRaw Create(string key, uint flags)
    {
        var request = new SmallRequestRaw
        {
            Version = CurrentVersion,
            Flags = flags
        };

        var keyBytes = System.Text.Encoding.UTF8.GetBytes(key);
        var len = Math.Min(keyBytes.Length, KeyBufferSize);
        request.KeyLen = (uint)len;

        byte* keyPtr = request._key;
        for (int i = 0; i < len; i++)
        {
            keyPtr[i] = keyBytes[i];
        }

        return request;
    }
}

/// <summary>
/// Small benchmark response (C struct version).
/// Must match Rust SmallResponseRaw layout exactly.
/// </summary>
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public unsafe struct SmallResponseRaw : IBinaryStruct
{
    public const byte CurrentVersion = 1;
    private const int ValueBufferSize = 64;

    public byte Version;
    private fixed byte _reserved[3];
    private fixed byte _value[ValueBufferSize];
    public uint ValueLen;
    public uint TtlSeconds;
    public byte CacheHit;
    private fixed byte _padding[3];

    public int ByteSize => 80; // 1 + 3 + 64 + 4 + 4 + 1 + 3
}
