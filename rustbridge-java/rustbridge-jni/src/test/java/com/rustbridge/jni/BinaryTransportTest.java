package com.rustbridge.jni;

import com.rustbridge.*;
import com.rustbridge.jni.JniNativeLibraryCondition.RequiresJniLibrary;
import org.junit.jupiter.api.*;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.nio.file.Path;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for binary transport (callRaw) in JNI bindings.
 * <p>
 * These tests verify the binary transport functionality which bypasses JSON
 * serialization for high-performance scenarios using fixed-size C structs.
 */
@RequiresJniLibrary
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class BinaryTransportTest {

    /**
     * Message ID for small benchmark (must match Rust MSG_BENCH_SMALL).
     */
    private static final int MSG_BENCH_SMALL = 1;

    private static Path PLUGIN_PATH;
    private JniPlugin plugin;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @BeforeEach
    void loadPlugin() throws PluginException {
        PluginConfig config = PluginConfig.defaults()
                .workerThreads(2);
        plugin = (JniPlugin) JniPluginLoader.load(PLUGIN_PATH.toString(), config);
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
    @DisplayName("callRaw___SmallBenchmark___ReturnsValidResponse")
    void callRaw___SmallBenchmark___ReturnsValidResponse() throws PluginException {
        byte[] request = SmallRequestRaw.create("test_key", 0x01);

        byte[] responseBytes = plugin.callRaw(MSG_BENCH_SMALL, request);
        SmallResponseRaw response = SmallResponseRaw.parse(responseBytes);

        assertEquals(SmallResponseRaw.CURRENT_VERSION, response.version);
        assertTrue(response.valueLen > 0);
        assertEquals(3600, response.ttlSeconds);
        assertEquals(1, response.cacheHit);
    }

    @Test
    @Order(3)
    @DisplayName("callRaw___SmallBenchmarkWithCacheMiss___ReturnsCacheMiss")
    void callRaw___SmallBenchmarkWithCacheMiss___ReturnsCacheMiss() throws PluginException {
        byte[] request = SmallRequestRaw.create("another_key", 0x00);

        byte[] responseBytes = plugin.callRaw(MSG_BENCH_SMALL, request);
        SmallResponseRaw response = SmallResponseRaw.parse(responseBytes);

        assertEquals(0, response.cacheHit);
    }

    @Test
    @Order(4)
    @DisplayName("callRaw___ConcurrentCalls___AllSucceed")
    void callRaw___ConcurrentCalls___AllSucceed() throws InterruptedException {
        int concurrentCalls = 100;
        AtomicInteger successCount = new AtomicInteger(0);
        AtomicInteger errorCount = new AtomicInteger(0);
        Thread[] threads = new Thread[concurrentCalls];

        for (int i = 0; i < concurrentCalls; i++) {
            final int index = i;
            threads[i] = new Thread(() -> {
                try {
                    String key = "key_" + index;
                    int flags = index % 2; // Alternate cache hit/miss
                    byte[] request = SmallRequestRaw.create(key, flags);

                    byte[] responseBytes = plugin.callRaw(MSG_BENCH_SMALL, request);
                    SmallResponseRaw response = SmallResponseRaw.parse(responseBytes);

                    if (response.version == SmallResponseRaw.CURRENT_VERSION) {
                        successCount.incrementAndGet();
                    }
                } catch (Exception e) {
                    errorCount.incrementAndGet();
                    System.err.println("Error in thread " + index + ": " + e.getMessage());
                }
            });
            threads[i].start();
        }

        for (Thread thread : threads) {
            thread.join();
        }

        System.out.println("Concurrent test: " + successCount.get() + " successes, " + errorCount.get() + " errors");
        assertEquals(concurrentCalls, successCount.get(),
                "Expected all calls to succeed, but " + errorCount.get() + " failed");
        assertEquals(0, errorCount.get(), "No errors should occur");
    }

    @Test
    @Order(5)
    @DisplayName("callRaw___ResponseValueContainsKey___ValueMatchesExpected")
    void callRaw___ResponseValueContainsKey___ValueMatchesExpected() throws PluginException {
        byte[] request = SmallRequestRaw.create("my_key", 0x01);

        byte[] responseBytes = plugin.callRaw(MSG_BENCH_SMALL, request);
        SmallResponseRaw response = SmallResponseRaw.parse(responseBytes);

        assertTrue(response.value.contains("my_key"), "Value should contain the key, got: " + response.value);
    }

    // ==================== Binary Struct Types (Java 8 compatible) ====================

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
    static class SmallRequestRaw {
        static final byte CURRENT_VERSION = 1;
        static final int KEY_BUFFER_SIZE = 64;
        static final int BYTE_SIZE = 76; // 1 + 3 + 64 + 4 + 4

        /**
         * Create a request byte array.
         */
        static byte[] create(String key, int flags) {
            ByteBuffer buffer = ByteBuffer.allocate(BYTE_SIZE);
            buffer.order(ByteOrder.LITTLE_ENDIAN);

            // version (u8)
            buffer.put(CURRENT_VERSION);

            // _reserved (3 bytes)
            buffer.put((byte) 0);
            buffer.put((byte) 0);
            buffer.put((byte) 0);

            // key (64 bytes, fixed buffer)
            byte[] keyBytes = key.getBytes(StandardCharsets.UTF_8);
            int keyLen = Math.min(keyBytes.length, KEY_BUFFER_SIZE);
            buffer.put(keyBytes, 0, keyLen);
            // Pad remaining bytes with zeros
            for (int i = keyLen; i < KEY_BUFFER_SIZE; i++) {
                buffer.put((byte) 0);
            }

            // key_len (u32)
            buffer.putInt(keyLen);

            // flags (u32)
            buffer.putInt(flags);

            return buffer.array();
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
    static class SmallResponseRaw {
        static final byte CURRENT_VERSION = 1;
        static final int VALUE_BUFFER_SIZE = 64;
        static final int BYTE_SIZE = 80; // 1 + 3 + 64 + 4 + 4 + 1 + 3

        final byte version;
        final String value;
        final int valueLen;
        final int ttlSeconds;
        final byte cacheHit;

        private SmallResponseRaw(byte version, String value, int valueLen, int ttlSeconds, byte cacheHit) {
            this.version = version;
            this.value = value;
            this.valueLen = valueLen;
            this.ttlSeconds = ttlSeconds;
            this.cacheHit = cacheHit;
        }

        /**
         * Parse a response from byte array.
         */
        static SmallResponseRaw parse(byte[] bytes) {
            if (bytes.length < BYTE_SIZE) {
                throw new IllegalArgumentException("Response too small: " + bytes.length + " < " + BYTE_SIZE);
            }

            ByteBuffer buffer = ByteBuffer.wrap(bytes);
            buffer.order(ByteOrder.LITTLE_ENDIAN);

            // version (u8)
            byte version = buffer.get();

            // _reserved (3 bytes)
            buffer.get();
            buffer.get();
            buffer.get();

            // value (64 bytes)
            byte[] valueBytes = new byte[VALUE_BUFFER_SIZE];
            buffer.get(valueBytes);

            // value_len (u32)
            int valueLen = buffer.getInt();

            // ttl_seconds (u32)
            int ttlSeconds = buffer.getInt();

            // cache_hit (u8)
            byte cacheHit = buffer.get();

            // Parse value string
            String value = new String(valueBytes, 0, Math.min(valueLen, VALUE_BUFFER_SIZE), StandardCharsets.UTF_8);

            return new SmallResponseRaw(version, value, valueLen, ttlSeconds, cacheHit);
        }
    }
}
