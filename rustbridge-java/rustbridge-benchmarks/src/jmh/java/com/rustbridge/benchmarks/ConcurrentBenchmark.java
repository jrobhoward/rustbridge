package com.rustbridge.benchmarks;

import com.rustbridge.*;
import com.rustbridge.ffm.BinaryStruct;
import com.rustbridge.ffm.FfmPlugin;
import com.rustbridge.ffm.FfmPluginLoader;
import com.rustbridge.jni.JniPlugin;
import com.rustbridge.jni.JniPluginLoader;
import org.openjdk.jmh.annotations.*;
import org.openjdk.jmh.infra.Blackhole;

import java.lang.foreign.Arena;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

/**
 * JMH benchmarks for concurrent/multi-threaded plugin calls.
 * <p>
 * These benchmarks measure how well the transport scales under concurrent load.
 * Each benchmark is run with varying thread counts to show scalability.
 * <p>
 * Run with: ./gradlew :rustbridge-benchmarks:jmh -Pjmh.includes=".*ConcurrentBenchmark.*"
 */
@BenchmarkMode(Mode.Throughput)
@OutputTimeUnit(TimeUnit.SECONDS)
@State(Scope.Benchmark)
@Warmup(iterations = 2, time = 1)
@Measurement(iterations = 3, time = 2)
@Fork(value = 1, jvmArgs = {"--enable-preview", "--enable-native-access=ALL-UNNAMED"})
public class ConcurrentBenchmark {

    private static final int MSG_BENCH_SMALL = 1;
    private static final String JSON_REQUEST = "{\"message\": \"concurrent test\"}";

    private FfmPlugin ffmPlugin;
    private JniPlugin jniPlugin;

    @Setup(Level.Trial)
    public void setup() throws Exception {
        Path pluginPath = BenchmarkHelper.findHelloPluginLibrary();
        // Use more worker threads for concurrent tests
        PluginConfig config = PluginConfig.defaults().workerThreads(8);

        ffmPlugin = (FfmPlugin) FfmPluginLoader.load(pluginPath, config, null);

        try {
            System.loadLibrary("rustbridge_jni");
            jniPlugin = (JniPlugin) JniPluginLoader.load(pluginPath.toString(), config);
        } catch (UnsatisfiedLinkError e) {
            jniPlugin = null;
        }
    }

    @TearDown(Level.Trial)
    public void teardown() {
        if (ffmPlugin != null) ffmPlugin.close();
        if (jniPlugin != null) jniPlugin.close();
    }

    // ==================== Single-threaded baseline ====================

    @Benchmark
    @Threads(1)
    public String ffmJson_1thread(Blackhole bh) throws PluginException {
        String response = ffmPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    @Threads(1)
    public byte[] ffmBinary_1thread(Blackhole bh) throws PluginException {
        try (Arena arena = Arena.ofConfined()) {
            SmallRequestRaw request = new SmallRequestRaw(arena, "bench_key", 0x01);
            byte[] response = ffmPlugin.callRawBytes(MSG_BENCH_SMALL, request);
            bh.consume(response);
            return response;
        }
    }

    // ==================== 4 threads ====================

    @Benchmark
    @Threads(4)
    public String ffmJson_4threads(Blackhole bh) throws PluginException {
        String response = ffmPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    @Threads(4)
    public byte[] ffmBinary_4threads(Blackhole bh) throws PluginException {
        try (Arena arena = Arena.ofConfined()) {
            SmallRequestRaw request = new SmallRequestRaw(arena, "bench_key", 0x01);
            byte[] response = ffmPlugin.callRawBytes(MSG_BENCH_SMALL, request);
            bh.consume(response);
            return response;
        }
    }

    // ==================== 8 threads ====================

    @Benchmark
    @Threads(8)
    public String ffmJson_8threads(Blackhole bh) throws PluginException {
        String response = ffmPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    @Threads(8)
    public byte[] ffmBinary_8threads(Blackhole bh) throws PluginException {
        try (Arena arena = Arena.ofConfined()) {
            SmallRequestRaw request = new SmallRequestRaw(arena, "bench_key", 0x01);
            byte[] response = ffmPlugin.callRawBytes(MSG_BENCH_SMALL, request);
            bh.consume(response);
            return response;
        }
    }

    // ==================== JNI Concurrent ====================

    @Benchmark
    @Threads(1)
    public String jniJson_1thread(Blackhole bh) throws PluginException {
        if (jniPlugin == null) return null;
        String response = jniPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    @Threads(4)
    public String jniJson_4threads(Blackhole bh) throws PluginException {
        if (jniPlugin == null) return null;
        String response = jniPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    @Threads(8)
    public String jniJson_8threads(Blackhole bh) throws PluginException {
        if (jniPlugin == null) return null;
        String response = jniPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    @Threads(1)
    public byte[] jniBinary_1thread(Blackhole bh) throws PluginException {
        if (jniPlugin == null) return null;
        byte[] request = createBinaryRequest("bench_key", 0x01);
        byte[] response = jniPlugin.callRaw(MSG_BENCH_SMALL, request);
        bh.consume(response);
        return response;
    }

    @Benchmark
    @Threads(4)
    public byte[] jniBinary_4threads(Blackhole bh) throws PluginException {
        if (jniPlugin == null) return null;
        byte[] request = createBinaryRequest("bench_key", 0x01);
        byte[] response = jniPlugin.callRaw(MSG_BENCH_SMALL, request);
        bh.consume(response);
        return response;
    }

    @Benchmark
    @Threads(8)
    public byte[] jniBinary_8threads(Blackhole bh) throws PluginException {
        if (jniPlugin == null) return null;
        byte[] request = createBinaryRequest("bench_key", 0x01);
        byte[] response = jniPlugin.callRaw(MSG_BENCH_SMALL, request);
        bh.consume(response);
        return response;
    }

    // ==================== Helpers ====================

    private static byte[] createBinaryRequest(String key, int flags) {
        ByteBuffer buffer = ByteBuffer.allocate(76);
        buffer.order(ByteOrder.LITTLE_ENDIAN);
        buffer.put((byte) 1);
        buffer.put((byte) 0);
        buffer.put((byte) 0);
        buffer.put((byte) 0);
        byte[] keyBytes = key.getBytes(StandardCharsets.UTF_8);
        int keyLen = Math.min(keyBytes.length, 64);
        buffer.put(keyBytes, 0, keyLen);
        for (int i = keyLen; i < 64; i++) buffer.put((byte) 0);
        buffer.putInt(keyLen);
        buffer.putInt(flags);
        return buffer.array();
    }

    static class SmallRequestRaw extends BinaryStruct {
        static final long BYTE_SIZE = 76;

        SmallRequestRaw(Arena arena, String key, int flags) {
            super(arena.allocate(BYTE_SIZE));
            segment.fill((byte) 0);
            setByte(0, (byte) 1);
            setFixedString(key, 4, 64, 68);
            setInt(72, flags);
        }

        @Override
        public long byteSize() {
            return BYTE_SIZE;
        }
    }
}
