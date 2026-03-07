package io.github.jrobhoward.rustbridge.ffm;

import io.github.jrobhoward.rustbridge.*;
import org.junit.jupiter.api.*;

import java.lang.foreign.*;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Benchmark test for FFM transport methods.
 * <p>
 * Compares latency of:
 * - FFM JSON transport
 * - FFM Binary transport
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
    private FfmPlugin ffmPlugin;

    @BeforeAll
    static void setupPaths() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @BeforeEach
    void loadPlugins() throws PluginException {
        PluginConfig config = PluginConfig.defaults().workerThreads(4);
        ffmPlugin = (FfmPlugin) FfmPluginLoader.load(PLUGIN_PATH, config, null);
    }

    @AfterEach
    void closePlugins() {
        if (ffmPlugin != null) {
            ffmPlugin.close();
            ffmPlugin = null;
        }
    }

    @Test
    @Order(1)
    @DisplayName("FFM Transport Benchmark")
    void benchmark___ffm_transports___compare_latency() throws PluginException {
        System.out.println("\n========== FFM Transport Benchmark ==========");
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

        System.out.println();
        System.out.println("Speedup ratios:");
        System.out.printf("  FFM Binary vs FFM JSON:       %.2fx faster%n", ffmJsonLatency / ffmBinaryLatency);

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
