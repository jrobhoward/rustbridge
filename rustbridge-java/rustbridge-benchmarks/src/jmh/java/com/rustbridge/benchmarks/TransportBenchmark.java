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
import java.lang.foreign.ValueLayout;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

/**
 * JMH benchmarks comparing transport methods across FFM and JNI.
 * <p>
 * Run with: ./gradlew :rustbridge-benchmarks:jmh
 * Quick run: ./gradlew :rustbridge-benchmarks:jmhQuick
 */
@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MICROSECONDS)
@State(Scope.Benchmark)
@Warmup(iterations = 3, time = 1)
@Measurement(iterations = 5, time = 1)
@Fork(value = 2, jvmArgs = {"--enable-preview", "--enable-native-access=ALL-UNNAMED"})
public class TransportBenchmark {

    private static final int MSG_BENCH_SMALL = 1;
    private static final String JSON_REQUEST = "{\"message\": \"benchmark test\"}";

    private FfmPlugin ffmPlugin;
    private JniPlugin jniPlugin;

    // Pre-allocated request for binary benchmarks
    private byte[] jniBinaryRequest;

    @Setup(Level.Trial)
    public void setup() throws Exception {
        Path pluginPath = BenchmarkHelper.findHelloPluginLibrary();
        PluginConfig config = PluginConfig.defaults().workerThreads(4);

        // Load FFM plugin
        ffmPlugin = (FfmPlugin) FfmPluginLoader.load(pluginPath, config, null);

        // Load JNI plugin
        try {
            System.loadLibrary("rustbridge_jni");
            jniPlugin = (JniPlugin) JniPluginLoader.load(pluginPath.toString(), config);
        } catch (UnsatisfiedLinkError e) {
            System.err.println("JNI library not available: " + e.getMessage());
            jniPlugin = null;
        }

        // Pre-allocate binary request
        jniBinaryRequest = createJniBinaryRequest("bench_key", 0x01);
    }

    @TearDown(Level.Trial)
    public void teardown() {
        if (ffmPlugin != null) {
            ffmPlugin.close();
        }
        if (jniPlugin != null) {
            jniPlugin.close();
        }
    }

    // ==================== FFM Benchmarks ====================

    @Benchmark
    public String ffmJson(Blackhole bh) throws PluginException {
        String response = ffmPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    public byte[] ffmBinary(Blackhole bh) throws PluginException {
        try (Arena arena = Arena.ofConfined()) {
            SmallRequestRaw request = new SmallRequestRaw(arena, "bench_key", 0x01);
            byte[] response = ffmPlugin.callRawBytes(MSG_BENCH_SMALL, request);
            bh.consume(response);
            return response;
        }
    }

    // ==================== JNI Benchmarks ====================

    @Benchmark
    public String jniJson(Blackhole bh) throws PluginException {
        if (jniPlugin == null) {
            return null;
        }
        String response = jniPlugin.call("echo", JSON_REQUEST);
        bh.consume(response);
        return response;
    }

    @Benchmark
    public byte[] jniBinary(Blackhole bh) throws PluginException {
        if (jniPlugin == null) {
            return null;
        }
        byte[] response = jniPlugin.callRaw(MSG_BENCH_SMALL, jniBinaryRequest);
        bh.consume(response);
        return response;
    }

    // ==================== Helper Methods ====================

    private static byte[] createJniBinaryRequest(String key, int flags) {
        ByteBuffer buffer = ByteBuffer.allocate(76);
        buffer.order(ByteOrder.LITTLE_ENDIAN);

        // version (u8)
        buffer.put((byte) 1);
        // _reserved (3 bytes)
        buffer.put((byte) 0);
        buffer.put((byte) 0);
        buffer.put((byte) 0);

        // key (64 bytes)
        byte[] keyBytes = key.getBytes(StandardCharsets.UTF_8);
        int keyLen = Math.min(keyBytes.length, 64);
        buffer.put(keyBytes, 0, keyLen);
        for (int i = keyLen; i < 64; i++) {
            buffer.put((byte) 0);
        }

        // key_len (u32)
        buffer.putInt(keyLen);
        // flags (u32)
        buffer.putInt(flags);

        return buffer.array();
    }

    // ==================== FFM Binary Struct Types ====================

    static class SmallRequestRaw extends BinaryStruct {
        static final byte CURRENT_VERSION = 1;
        static final int KEY_BUFFER_SIZE = 64;
        static final long BYTE_SIZE = 76;

        private static final long VERSION_OFFSET = 0;
        private static final long KEY_OFFSET = 4;
        private static final long KEY_LEN_OFFSET = 68;
        private static final long FLAGS_OFFSET = 72;

        SmallRequestRaw(Arena arena, String key, int flags) {
            super(arena.allocate(BYTE_SIZE));
            segment.fill((byte) 0);
            setByte(VERSION_OFFSET, CURRENT_VERSION);
            setFixedString(key, KEY_OFFSET, KEY_BUFFER_SIZE, KEY_LEN_OFFSET);
            setInt(FLAGS_OFFSET, flags);
        }

        @Override
        public long byteSize() {
            return BYTE_SIZE;
        }
    }

    static class SmallResponseRaw extends BinaryStruct {
        static final long BYTE_SIZE = 80;

        SmallResponseRaw(MemorySegment segment) {
            super(segment);
        }

        @Override
        public long byteSize() {
            return BYTE_SIZE;
        }
    }
}
