package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.lang.foreign.*;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for binary transport (callRawBytes).
 * <p>
 * These tests verify the binary transport functionality which bypasses JSON
 * serialization for high-performance scenarios using fixed-size C structs.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class BinaryTransportTest {

    /**
     * Message ID for small benchmark (must match Rust MSG_BENCH_SMALL).
     */
    private static final int MSG_BENCH_SMALL = 1;

    private static Path PLUGIN_PATH;
    private FfmPlugin plugin;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @BeforeEach
    void loadPlugin() throws PluginException {
        PluginConfig config = PluginConfig.defaults()
                .workerThreads(2);
        plugin = (FfmPlugin) FfmPluginLoader.load(PLUGIN_PATH, config, null);
    }

    @AfterEach
    void closePlugin() {
        if (plugin != null) {
            plugin.close();
            plugin = null;
        }
    }

    // ==================== Binary Transport Tests ====================

    @Test
    @Order(1)
    @DisplayName("hasBinaryTransport___returns_true")
    void hasBinaryTransport___returns_true() {
        assertTrue(plugin.hasBinaryTransport(), "Plugin should support binary transport");
    }

    @Test
    @Order(2)
    @DisplayName("callRawBytes___SmallBenchmark___ReturnsValidResponse")
    void callRawBytes___SmallBenchmark___ReturnsValidResponse() throws PluginException {
        // Arrange
        try (Arena arena = Arena.ofConfined()) {
            SmallRequestRaw request = new SmallRequestRaw(arena, "test_key", 0x01);

            // Act
            byte[] responseBytes = plugin.callRawBytes(MSG_BENCH_SMALL, request);
            SmallResponseRaw response = new SmallResponseRaw(MemorySegment.ofArray(responseBytes));

            // Assert
            assertEquals(SmallResponseRaw.CURRENT_VERSION, response.getVersion());
            assertTrue(response.getValueLen() > 0);
            assertEquals(3600, response.getTtlSeconds());
            assertEquals(1, response.getCacheHit()); // flags & 1 != 0
        }
    }

    @Test
    @Order(3)
    @DisplayName("callRawBytes___SmallBenchmarkWithCacheMiss___ReturnsCacheMiss")
    void callRawBytes___SmallBenchmarkWithCacheMiss___ReturnsCacheMiss() throws PluginException {
        // Arrange
        try (Arena arena = Arena.ofConfined()) {
            SmallRequestRaw request = new SmallRequestRaw(arena, "another_key", 0x00); // flags = 0, cache miss

            // Act
            byte[] responseBytes = plugin.callRawBytes(MSG_BENCH_SMALL, request);
            SmallResponseRaw response = new SmallResponseRaw(MemorySegment.ofArray(responseBytes));

            // Assert
            assertEquals(0, response.getCacheHit()); // flags & 1 == 0
        }
    }

    @Test
    @Order(4)
    @DisplayName("callRawBytes___ConcurrentCalls___AllSucceed")
    void callRawBytes___ConcurrentCalls___AllSucceed() throws InterruptedException {
        // Arrange
        int concurrentCalls = 100;
        AtomicInteger successCount = new AtomicInteger(0);
        AtomicInteger errorCount = new AtomicInteger(0);

        Thread[] threads = new Thread[concurrentCalls];

        // Act
        for (int i = 0; i < concurrentCalls; i++) {
            final int index = i;
            threads[i] = new Thread(() -> {
                try (Arena arena = Arena.ofConfined()) {
                    String key = "key_" + index;
                    int flags = index % 2; // Alternate cache hit/miss
                    SmallRequestRaw request = new SmallRequestRaw(arena, key, flags);

                    byte[] responseBytes = plugin.callRawBytes(MSG_BENCH_SMALL, request);
                    SmallResponseRaw response = new SmallResponseRaw(MemorySegment.ofArray(responseBytes));

                    if (response.getVersion() == SmallResponseRaw.CURRENT_VERSION) {
                        successCount.incrementAndGet();
                    }
                } catch (Exception e) {
                    errorCount.incrementAndGet();
                    System.err.println("Error in thread " + index + ": " + e.getMessage());
                }
            });
            threads[i].start();
        }

        // Wait for all threads
        for (Thread thread : threads) {
            thread.join();
        }

        // Assert
        System.out.println("Concurrent test: " + successCount.get() + " successes, " + errorCount.get() + " errors");
        assertEquals(concurrentCalls, successCount.get(),
                "Expected all calls to succeed, but " + errorCount.get() + " failed");
        assertEquals(0, errorCount.get(), "No errors should occur");
    }

    @Test
    @Order(5)
    @DisplayName("callRawBytes___ResponseValueContainsKey___ValueMatchesExpected")
    void callRawBytes___ResponseValueContainsKey___ValueMatchesExpected() throws PluginException {
        // Arrange
        try (Arena arena = Arena.ofConfined()) {
            SmallRequestRaw request = new SmallRequestRaw(arena, "my_key", 0x01);

            // Act
            byte[] responseBytes = plugin.callRawBytes(MSG_BENCH_SMALL, request);
            SmallResponseRaw response = new SmallResponseRaw(MemorySegment.ofArray(responseBytes));

            // Assert - the handler returns "value_for_{key}"
            String value = response.getValue();
            assertTrue(value.contains("my_key"), "Value should contain the key, got: " + value);
        }
    }

    // ==================== Binary Struct Types ====================

    /**
     * Small benchmark request (C struct version).
     * Must match Rust SmallRequestRaw layout exactly.
     * <p>
     * Layout (76 bytes):
     * - version: u8 (1 byte)
     * - _reserved: [u8; 3] (3 bytes)
     * - key: [u8; 64] (64 bytes)
     * - key_len: u32 (4 bytes)
     * - flags: u32 (4 bytes)
     */
    static class SmallRequestRaw extends BinaryStruct {
        static final byte CURRENT_VERSION = 1;
        static final int KEY_BUFFER_SIZE = 64;
        static final long BYTE_SIZE = 76; // 1 + 3 + 64 + 4 + 4

        // Offsets
        private static final long VERSION_OFFSET = 0;
        private static final long RESERVED_OFFSET = 1;
        private static final long KEY_OFFSET = 4;
        private static final long KEY_LEN_OFFSET = 68;
        private static final long FLAGS_OFFSET = 72;

        SmallRequestRaw(Arena arena, String key, int flags) {
            super(arena.allocate(BYTE_SIZE));

            // Zero the segment
            segment.fill((byte) 0);

            // Set version
            setByte(VERSION_OFFSET, CURRENT_VERSION);

            // Set key
            setFixedString(key, KEY_OFFSET, KEY_BUFFER_SIZE, KEY_LEN_OFFSET);

            // Set flags
            setInt(FLAGS_OFFSET, flags);
        }

        @Override
        public long byteSize() {
            return BYTE_SIZE;
        }

        public String getKey() {
            return getFixedString(KEY_OFFSET, KEY_BUFFER_SIZE, KEY_LEN_OFFSET);
        }
    }

    /**
     * Small benchmark response (C struct version).
     * Must match Rust SmallResponseRaw layout exactly.
     * <p>
     * Layout (80 bytes):
     * - version: u8 (1 byte)
     * - _reserved: [u8; 3] (3 bytes)
     * - value: [u8; 64] (64 bytes)
     * - value_len: u32 (4 bytes)
     * - ttl_seconds: u32 (4 bytes)
     * - cache_hit: u8 (1 byte)
     * - _padding: [u8; 3] (3 bytes)
     */
    static class SmallResponseRaw extends BinaryStruct {
        static final byte CURRENT_VERSION = 1;
        static final int VALUE_BUFFER_SIZE = 64;
        static final long BYTE_SIZE = 80; // 1 + 3 + 64 + 4 + 4 + 1 + 3

        // Offsets
        private static final long VERSION_OFFSET = 0;
        private static final long RESERVED_OFFSET = 1;
        private static final long VALUE_OFFSET = 4;
        private static final long VALUE_LEN_OFFSET = 68;
        private static final long TTL_SECONDS_OFFSET = 72;
        private static final long CACHE_HIT_OFFSET = 76;
        private static final long PADDING_OFFSET = 77;

        SmallResponseRaw(MemorySegment segment) {
            super(segment);
        }

        @Override
        public long byteSize() {
            return BYTE_SIZE;
        }

        public byte getVersion() {
            return getByte(VERSION_OFFSET);
        }

        public String getValue() {
            return getFixedString(VALUE_OFFSET, VALUE_BUFFER_SIZE, VALUE_LEN_OFFSET);
        }

        public int getValueLen() {
            return getInt(VALUE_LEN_OFFSET);
        }

        public int getTtlSeconds() {
            return getInt(TTL_SECONDS_OFFSET);
        }

        public byte getCacheHit() {
            return getByte(CACHE_HIT_OFFSET);
        }
    }
}
