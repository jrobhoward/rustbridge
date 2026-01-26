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
import java.lang.foreign.MemorySegment;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

/**
 * JMH throughput benchmarks measuring operations per second.
 * <p>
 * Run with: ./gradlew :rustbridge-benchmarks:jmh -Pjmh.includes=".*ThroughputBenchmark.*"
 */
@BenchmarkMode(Mode.Throughput)
@OutputTimeUnit(TimeUnit.SECONDS)
@State(Scope.Benchmark)
@Warmup(iterations = 3, time = 1)
@Measurement(iterations = 5, time = 2)
@Fork(value = 2, jvmArgs = {"--enable-preview", "--enable-native-access=ALL-UNNAMED"})
public class ThroughputBenchmark {

    private static final int MSG_BENCH_SMALL = 1;
    private static final String JSON_REQUEST = "{\"message\": \"throughput test\"}";

    private FfmPlugin ffmPlugin;
    private JniPlugin jniPlugin;
    private byte[] jniBinaryRequest;

    @Setup(Level.Trial)
    public void setup() throws Exception {
        Path pluginPath = BenchmarkHelper.findHelloPluginLibrary();
        PluginConfig config = PluginConfig.defaults().workerThreads(4);

        ffmPlugin = (FfmPlugin) FfmPluginLoader.load(pluginPath, config, null);

        try {
            System.loadLibrary("rustbridge_jni");
            jniPlugin = (JniPlugin) JniPluginLoader.load(pluginPath.toString(), config);
        } catch (UnsatisfiedLinkError e) {
            jniPlugin = null;
        }

        jniBinaryRequest = createBinaryRequest("bench_key", 0x01);
    }

    @TearDown(Level.Trial)
    public void teardown() {
        if (ffmPlugin != null) ffmPlugin.close();
        if (jniPlugin != null) jniPlugin.close();
    }

    // ==================== FFM Throughput ====================

    @Benchmark
    public String ffmJsonThroughput(Blackhole bh) throws PluginException {
        String response = ffmPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    public byte[] ffmBinaryThroughput(Blackhole bh) throws PluginException {
        try (Arena arena = Arena.ofConfined()) {
            SmallRequestRaw request = new SmallRequestRaw(arena, "bench_key", 0x01);
            byte[] response = ffmPlugin.callRawBytes(MSG_BENCH_SMALL, request);
            bh.consume(response);
            return response;
        }
    }

    // ==================== JNI Throughput ====================

    @Benchmark
    public String jniJsonThroughput(Blackhole bh) throws PluginException {
        if (jniPlugin == null) return null;
        String response = jniPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    public byte[] jniBinaryThroughput(Blackhole bh) throws PluginException {
        if (jniPlugin == null) return null;
        byte[] response = jniPlugin.callRaw(MSG_BENCH_SMALL, jniBinaryRequest);
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
