package com.rustbridge.ffm;

import com.rustbridge.*;
import com.rustbridge.jni.JniPlugin;
import com.rustbridge.jni.JniPluginLoader;
import org.junit.jupiter.api.*;

import java.lang.foreign.*;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Benchmark comparison test for transport methods.
 * <p>
 * Compares latency of:
 * - FFM JSON transport
 * - FFM Binary transport
 * - JNI JSON transport
 * - JNI Binary transport
 * <p>
 * Run with: ./gradlew :rustbridge-ffm:test --tests "*TransportBenchmarkTest*" -i
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
@Timeout(value = 120, unit = TimeUnit.SECONDS)
class TransportBenchmarkTest {

    private static final int MSG_BENCH_SMALL = 1;
    private static final int WARMUP_ITERATIONS = 1000;
    private static final int BENCHMARK_ITERATIONS = 10000;

    private static Path PLUGIN_PATH;
    private static Path JNI_LIBRARY_PATH;
    private static boolean JNI_AVAILABLE = false;

    private FfmPlugin ffmPlugin;
    private JniPlugin jniPlugin;

    @BeforeAll
    static void setupPaths() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);

        // Check if JNI is available
        try {
            System.loadLibrary("rustbridge_jni");
            JNI_AVAILABLE = true;
            System.out.println("JNI library available");
        } catch (UnsatisfiedLinkError e) {
            System.out.println("JNI library not available: " + e.getMessage());
            JNI_AVAILABLE = false;
        }
    }

    @BeforeEach
    void loadPlugins() throws PluginException {
        PluginConfig config = PluginConfig.defaults().workerThreads(4);

        // Load FFM plugin
        ffmPlugin = (FfmPlugin) FfmPluginLoader.load(PLUGIN_PATH, config, null);

        // Load JNI plugin if available
        if (JNI_AVAILABLE) {
            try {
                jniPlugin = (JniPlugin) JniPluginLoader.load(PLUGIN_PATH.toString(), config);
            } catch (Exception e) {
                System.out.println("Failed to load JNI plugin: " + e.getMessage());
                jniPlugin = null;
            }
        }
    }

    @AfterEach
    void closePlugins() {
        if (ffmPlugin != null) {
            ffmPlugin.close();
            ffmPlugin = null;
        }
        if (jniPlugin != null) {
            jniPlugin.close();
            jniPlugin = null;
        }
    }

    @Test
    @Order(1)
    @DisplayName("Transport Benchmark Comparison")
    void benchmark___all_transports___compare_latency() throws PluginException {
        System.out.println("\n========== Transport Benchmark ==========");
        System.out.println("Warmup iterations: " + WARMUP_ITERATIONS);
        System.out.println("Benchmark iterations: " + BENCHMARK_ITERATIONS);
        System.out.println();

        // FFM JSON
        double ffmJsonLatency = benchmarkFfmJson();
        System.out.printf("FFM JSON:           %6.2f μs/call (%,9.0f calls/sec)%n",
                ffmJsonLatency, 1_000_000.0 / ffmJsonLatency);

        // FFM Binary (callRawBytes)
        double ffmBinaryLatency = benchmarkFfmBinary();
        System.out.printf("FFM Binary:         %6.2f μs/call (%,9.0f calls/sec)%n",
                ffmBinaryLatency, 1_000_000.0 / ffmBinaryLatency);

        // JNI JSON (if available)
        if (jniPlugin != null) {
            double jniJsonLatency = benchmarkJniJson();
            System.out.printf("JNI JSON:           %6.2f μs/call (%,9.0f calls/sec)%n",
                    jniJsonLatency, 1_000_000.0 / jniJsonLatency);

            // JNI Binary
            double jniBinaryLatency = benchmarkJniBinary();
            System.out.printf("JNI Binary:         %6.2f μs/call (%,9.0f calls/sec)%n",
                    jniBinaryLatency, 1_000_000.0 / jniBinaryLatency);

            System.out.println();
            System.out.println("Speedup ratios:");
            System.out.printf("  FFM Binary vs FFM JSON:        %.2fx faster%n", ffmJsonLatency / ffmBinaryLatency);
            System.out.printf("  JNI Binary vs JNI JSON:        %.2fx faster%n", jniJsonLatency / jniBinaryLatency);
            System.out.printf("  FFM Binary vs JNI Binary:      %.2fx %s%n",
                    Math.max(ffmBinaryLatency, jniBinaryLatency) / Math.min(ffmBinaryLatency, jniBinaryLatency),
                    ffmBinaryLatency < jniBinaryLatency ? "(FFM faster)" : "(JNI faster)");
        } else {
            System.out.println();
            System.out.println("JNI not available - skipping JNI benchmarks");
            System.out.println();
            System.out.println("Speedup ratios:");
            System.out.printf("  FFM Binary vs FFM JSON:       %.2fx faster%n", ffmJsonLatency / ffmBinaryLatency);
        }

        System.out.println("==========================================\n");

        // Basic assertions
        assertTrue(ffmBinaryLatency < ffmJsonLatency, "Binary transport should be faster than JSON");
    }

    private double benchmarkFfmJson() throws PluginException {
        String request = "{\"message\": \"benchmark test\"}";

        // Warmup
        for (int i = 0; i < WARMUP_ITERATIONS; i++) {
            ffmPlugin.call("echo", request);
        }

        // Benchmark
        long start = System.nanoTime();
        for (int i = 0; i < BENCHMARK_ITERATIONS; i++) {
            ffmPlugin.call("echo", request);
        }
        long elapsed = System.nanoTime() - start;

        return (double) elapsed / BENCHMARK_ITERATIONS / 1000.0; // Convert to microseconds
    }

    private double benchmarkFfmBinary() throws PluginException {
        // Warmup
        try (Arena arena = Arena.ofConfined()) {
            for (int i = 0; i < WARMUP_ITERATIONS; i++) {
                SmallRequestRaw request = new SmallRequestRaw(arena, "bench_key", 0x01);
                ffmPlugin.callRawBytes(MSG_BENCH_SMALL, request);
            }
        }

        // Benchmark
        long start = System.nanoTime();
        try (Arena arena = Arena.ofConfined()) {
            for (int i = 0; i < BENCHMARK_ITERATIONS; i++) {
                SmallRequestRaw request = new SmallRequestRaw(arena, "bench_key", 0x01);
                ffmPlugin.callRawBytes(MSG_BENCH_SMALL, request);
            }
        }
        long elapsed = System.nanoTime() - start;

        return (double) elapsed / BENCHMARK_ITERATIONS / 1000.0;
    }

    private double benchmarkJniJson() throws PluginException {
        String request = "{\"message\": \"benchmark test\"}";

        // Warmup
        for (int i = 0; i < WARMUP_ITERATIONS; i++) {
            jniPlugin.call("echo", request);
        }

        // Benchmark
        long start = System.nanoTime();
        for (int i = 0; i < BENCHMARK_ITERATIONS; i++) {
            jniPlugin.call("echo", request);
        }
        long elapsed = System.nanoTime() - start;

        return (double) elapsed / BENCHMARK_ITERATIONS / 1000.0;
    }

    private double benchmarkJniBinary() throws PluginException {
        // Warmup
        for (int i = 0; i < WARMUP_ITERATIONS; i++) {
            byte[] request = JniSmallRequest.create("bench_key", 0x01);
            jniPlugin.callRaw(MSG_BENCH_SMALL, request);
        }

        // Benchmark
        long start = System.nanoTime();
        for (int i = 0; i < BENCHMARK_ITERATIONS; i++) {
            byte[] request = JniSmallRequest.create("bench_key", 0x01);
            jniPlugin.callRaw(MSG_BENCH_SMALL, request);
        }
        long elapsed = System.nanoTime() - start;

        return (double) elapsed / BENCHMARK_ITERATIONS / 1000.0;
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

    // ==================== JNI Binary Struct Types ====================

    static class JniSmallRequest {
        static final byte CURRENT_VERSION = 1;
        static final int KEY_BUFFER_SIZE = 64;
        static final int BYTE_SIZE = 76;

        static byte[] create(String key, int flags) {
            ByteBuffer buffer = ByteBuffer.allocate(BYTE_SIZE);
            buffer.order(ByteOrder.LITTLE_ENDIAN);

            buffer.put(CURRENT_VERSION);
            buffer.put((byte) 0);
            buffer.put((byte) 0);
            buffer.put((byte) 0);

            byte[] keyBytes = key.getBytes(StandardCharsets.UTF_8);
            int keyLen = Math.min(keyBytes.length, KEY_BUFFER_SIZE);
            buffer.put(keyBytes, 0, keyLen);
            for (int i = keyLen; i < KEY_BUFFER_SIZE; i++) {
                buffer.put((byte) 0);
            }

            buffer.putInt(keyLen);
            buffer.putInt(flags);

            return buffer.array();
        }
    }
}
