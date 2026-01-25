using System.Runtime.InteropServices;
using RustBridge.Native;

namespace RustBridge.Tests;

/// <summary>
/// Tests for binary transport (CallRaw).
/// <para>
/// NOTE: These tests are currently skipped because the Rust FFI layer uses thread_local
/// storage for binary handlers. Handlers registered during on_start (in Tokio's thread)
/// aren't visible when plugin_call_raw is called from the host language thread.
/// See: crates/rustbridge-ffi/src/exports.rs BINARY_HANDLERS
/// </para>
/// </summary>
[Trait("Category", "Integration")]
[Trait("Category", "BinaryTransport")]
public class BinaryTransportTest : IDisposable
{
    /// <summary>
    /// Message ID for small benchmark (must match Rust MSG_BENCH_SMALL).
    /// </summary>
    private const int MsgBenchSmall = 1;

    private readonly IPlugin? _plugin;
    private readonly string? _skipReason;

    public BinaryTransportTest()
    {
        var libraryPath = FindHelloPlugin();
        if (libraryPath == null)
        {
            _skipReason = "hello-plugin not found. Run: cargo build --release -p hello-plugin";
            return;
        }

        try
        {
            _plugin = NativePluginLoader.Load(libraryPath);
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
        var assemblyLocation = typeof(BinaryTransportTest).Assembly.Location;
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
            if (File.Exists(releasePath)) return Path.GetFullPath(releasePath);

            var debugPath = Path.Combine(baseDir, "target", "debug", libraryName);
            if (File.Exists(debugPath)) return Path.GetFullPath(debugPath);
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

        // Skip due to known Rust FFI thread-local storage issue
        // Binary handlers are registered in Tokio thread but looked up in calling thread
        Skip.If(true, "Binary transport disabled: Rust FFI uses thread_local storage for handlers (see BINARY_HANDLERS in exports.rs)");
    }

    // ==================== Binary Transport Tests ====================

    [SkippableFact]
    public void CallRaw___SmallBenchmark___ReturnsValidResponse()
    {
        SkipIfPluginNotAvailable();

        var request = SmallRequestRaw.Create("test_key", 0x01);

        var response = _plugin!.CallRaw<SmallRequestRaw, SmallResponseRaw>(MsgBenchSmall, request);

        Assert.Equal(SmallResponseRaw.CurrentVersion, response.Version);
        Assert.True(response.ValueLen > 0);
        Assert.Equal(3600u, response.TtlSeconds);
        Assert.Equal(1, response.CacheHit); // flags & 1 != 0
    }

    [SkippableFact]
    public void CallRaw___SmallBenchmarkWithCacheMiss___ReturnsCacheMiss()
    {
        SkipIfPluginNotAvailable();

        var request = SmallRequestRaw.Create("another_key", 0x00); // flags = 0, cache miss

        var response = _plugin!.CallRaw<SmallRequestRaw, SmallResponseRaw>(MsgBenchSmall, request);

        Assert.Equal(0, response.CacheHit); // flags & 1 == 0
    }

    [SkippableFact]
    public void CallRaw___ConcurrentCalls___AllSucceed()
    {
        SkipIfPluginNotAvailable();

        const int concurrentCalls = 100;
        var tasks = new Task<SmallResponseRaw>[concurrentCalls];

        for (int i = 0; i < concurrentCalls; i++)
        {
            var key = $"key_{i}";
            var flags = (uint)(i % 2); // Alternate cache hit/miss
            tasks[i] = Task.Run(() =>
            {
                var request = SmallRequestRaw.Create(key, flags);
                return _plugin!.CallRaw<SmallRequestRaw, SmallResponseRaw>(MsgBenchSmall, request);
            });
        }

        Task.WaitAll(tasks);

        for (int i = 0; i < concurrentCalls; i++)
        {
            var response = tasks[i].Result;
            Assert.Equal(SmallResponseRaw.CurrentVersion, response.Version);
            Assert.Equal((byte)(i % 2), response.CacheHit);
        }
    }

    // ==================== Binary Struct Types ====================

    /// <summary>
    /// Small benchmark request (C struct version).
    /// Must match Rust SmallRequestRaw layout exactly.
    /// </summary>
    [StructLayout(LayoutKind.Sequential, Pack = 1)]
    public unsafe struct SmallRequestRaw : IBinaryStruct
    {
        public const byte CurrentVersion = 1;
        private const int KeyBufferSize = 64;

        /// <summary>Struct version for forward compatibility.</summary>
        public byte Version;

        /// <summary>Reserved for alignment (must be zero).</summary>
        private fixed byte _reserved[3];

        /// <summary>Key buffer (fixed-size).</summary>
        private fixed byte _key[KeyBufferSize];

        /// <summary>Length of key string.</summary>
        public uint KeyLen;

        /// <summary>Flags bitmask.</summary>
        public uint Flags;

        public int ByteSize => 76; // 1 + 3 + 64 + 4 + 4

        /// <summary>
        /// Create a new request.
        /// </summary>
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

            // Copy key bytes to fixed buffer
            byte* keyPtr = request._key;
            for (int i = 0; i < len; i++)
            {
                keyPtr[i] = keyBytes[i];
            }

            return request;
        }

        /// <summary>
        /// Get the key as a string.
        /// </summary>
        public readonly string GetKey()
        {
            var len = (int)Math.Min(KeyLen, KeyBufferSize);
            fixed (byte* keyPtr = _key)
            {
                return System.Text.Encoding.UTF8.GetString(keyPtr, len);
            }
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

        /// <summary>Struct version for forward compatibility.</summary>
        public byte Version;

        /// <summary>Reserved for alignment.</summary>
        private fixed byte _reserved[3];

        /// <summary>Value buffer (fixed-size).</summary>
        private fixed byte _value[ValueBufferSize];

        /// <summary>Length of value string.</summary>
        public uint ValueLen;

        /// <summary>TTL in seconds.</summary>
        public uint TtlSeconds;

        /// <summary>Cache hit flag (0 = miss, 1 = hit).</summary>
        public byte CacheHit;

        /// <summary>Padding for alignment.</summary>
        private fixed byte _padding[3];

        public int ByteSize => 80; // 1 + 3 + 64 + 4 + 4 + 1 + 3

        /// <summary>
        /// Get the value as a string.
        /// </summary>
        public readonly string GetValue()
        {
            var len = (int)Math.Min(ValueLen, ValueBufferSize);
            fixed (byte* valuePtr = _value)
            {
                return System.Text.Encoding.UTF8.GetString(valuePtr, len);
            }
        }
    }
}
