package io.github.jrobhoward.rustbridge.benchmarks;

import io.github.jrobhoward.rustbridge.*;
import io.github.jrobhoward.rustbridge.ffm.BinaryStruct;
import io.github.jrobhoward.rustbridge.ffm.FfmPlugin;
import io.github.jrobhoward.rustbridge.ffm.FfmPluginLoader;
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
@Fork(value = 2, jvmArgs = {"--enable-native-access=ALL-UNNAMED"})
public class ThroughputBenchmark {

    private static final int MSG_BENCH_SMALL = 1;
    private static final String JSON_REQUEST = "{\"message\": \"throughput test\"}";

    private FfmPlugin ffmPlugin;

    @Setup(Level.Trial)
    public void setup() throws Exception {
        Path pluginPath = BenchmarkHelper.findHelloPluginLibrary();
        PluginConfig config = PluginConfig.defaults().workerThreads(4);

        ffmPlugin = (FfmPlugin) FfmPluginLoader.load(pluginPath, config, null);
    }

    @TearDown(Level.Trial)
    public void teardown() {
        if (ffmPlugin != null) ffmPlugin.close();
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

    // ==================== Helpers ====================

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
