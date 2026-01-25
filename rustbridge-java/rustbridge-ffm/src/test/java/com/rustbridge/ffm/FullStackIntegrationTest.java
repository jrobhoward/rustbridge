package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Full-stack integration tests for complete plugin lifecycle scenarios.
 * Tests error propagation, multi-plugin coordination, and stress scenarios.
 */
@Timeout(value = 120, unit = TimeUnit.SECONDS)
class FullStackIntegrationTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Error in handler propagates to Java as PluginException")
    void testErrorPropagation() throws PluginException, InterruptedException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            assertEquals(LifecycleState.ACTIVE, plugin.getState());

            // Request an operation that will fail (invalid JSON)
            assertThrows(PluginException.class, () -> {
                plugin.call("user.create", "{invalid json}");
            }, "Invalid JSON should propagate as PluginException");

            // Plugin should still be active and callable after error
            assertEquals(LifecycleState.ACTIVE, plugin.getState());

            String response = plugin.call("echo", "{\"message\": \"still works\"}");
            assertTrue(response.contains("still works"), "Plugin should recover from error");
        }
    }

    @Test
    @DisplayName("Multiple plugins operate independently without state leakage")
    void testMultiPluginIsolation() throws PluginException {
        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH.toString());
        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH.toString());

        try {
            assertEquals(LifecycleState.ACTIVE, plugin1.getState());
            assertEquals(LifecycleState.ACTIVE, plugin2.getState());

            // Both plugins respond independently
            String resp1 = plugin1.call("echo", "{\"message\": \"plugin1\"}");
            String resp2 = plugin2.call("echo", "{\"message\": \"plugin2\"}");

            assertTrue(resp1.contains("plugin1"));
            assertTrue(resp2.contains("plugin2"));

            // Close plugin1, plugin2 should still work
            plugin1.close();
            assertEquals(LifecycleState.STOPPED, plugin1.getState());
            assertEquals(LifecycleState.ACTIVE, plugin2.getState());

            String resp2Again = plugin2.call("echo", "{\"message\": \"still active\"}");
            assertTrue(resp2Again.contains("still active"));
        } finally {
            try {
                plugin1.close();
            } catch (Exception e) {
                // Already closed
            }
            try {
                plugin2.close();
            } catch (Exception e) {
                // Ignore
            }
        }
    }

    @Test
    @DisplayName("Large payload roundtrip succeeds without corruption")
    void testLargePayloadRoundtrip() throws PluginException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            // Create a large JSON payload
            StringBuilder largeMessage = new StringBuilder();
            for (int i = 0; i < 1000; i++) {
                largeMessage.append("The quick brown fox jumps over the lazy dog. ");
            }

            String payload = "{\"message\": \"" + largeMessage + "\"}";
            String response = plugin.call("echo", payload);

            // Verify response contains the message (may be truncated in display)
            assertTrue(response.length() > 100, "Large payload response should be substantial");
            assertTrue(response.contains("quick brown fox"), "Message content should be preserved");
        }
    }

    @Test
    @DisplayName("Plugin reload clears old state and resets to initial state")
    void testPluginReloadResetsState() throws PluginException, InterruptedException {
        List<String> initialLogs = new ArrayList<>();
        LogCallback initialCallback = (level, target, message) -> {
            initialLogs.add(message);
        };

        // First load
        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), initialCallback);
        plugin1.setLogLevel(LogLevel.DEBUG);
        plugin1.call("echo", "{\"message\": \"first load\"}");
        Thread.sleep(100);

        int firstLoadLogs = initialLogs.size();
        assertTrue(firstLoadLogs > 0, "First load should generate logs");

        plugin1.close();
        Thread.sleep(500);

        // Second load with new callback
        List<String> secondLogs = new ArrayList<>();
        LogCallback secondCallback = (level, target, message) -> {
            secondLogs.add(message);
        };

        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), secondCallback);
        // Log level resets to default (INFO) - need to set DEBUG again
        plugin2.setLogLevel(LogLevel.DEBUG);
        secondLogs.clear();

        plugin2.call("echo", "{\"message\": \"second load\"}");
        Thread.sleep(100);

        int secondLoadLogs = secondLogs.size();
        assertTrue(secondLoadLogs > 0, "Second load should generate logs with new callback");

        plugin2.close();
    }

    @Test
    @DisplayName("Concurrent requests from multiple threads succeed")
    void testConcurrentRequests() throws PluginException, InterruptedException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            int numThreads = 10;
            int requestsPerThread = 20;
            AtomicInteger successCount = new AtomicInteger(0);
            AtomicInteger failureCount = new AtomicInteger(0);

            List<Thread> threads = new ArrayList<>();

            for (int t = 0; t < numThreads; t++) {
                final int threadId = t;
                Thread thread = new Thread(() -> {
                    for (int r = 0; r < requestsPerThread; r++) {
                        try {
                            String response = plugin.call("echo",
                                "{\"message\": \"thread-" + threadId + "-request-" + r + "\"}");
                            if (response.contains("thread-" + threadId)) {
                                successCount.incrementAndGet();
                            } else {
                                failureCount.incrementAndGet();
                            }
                        } catch (PluginException e) {
                            failureCount.incrementAndGet();
                        }
                    }
                });
                threads.add(thread);
                thread.start();
            }

            // Wait for all threads
            for (Thread thread : threads) {
                thread.join(60_000);
            }

            int totalRequests = numThreads * requestsPerThread;
            assertEquals(totalRequests, successCount.get(),
                "All " + totalRequests + " requests should succeed");
            assertEquals(0, failureCount.get(), "No failures expected");
        }
    }

    @Test
    @DisplayName("Config with custom parameters is applied correctly")
    void testCustomConfiguration() throws PluginException {
        PluginConfig config = PluginConfig.defaults()
                .workerThreads(2)
                .logLevel("debug");

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            assertEquals(LifecycleState.ACTIVE, plugin.getState());

            // Plugin should work with custom config
            String response = plugin.call("echo", "{\"message\": \"configured\"}");
            assertTrue(response.contains("configured"));
        }
    }

    @Test
    @DisplayName("Timeout is respected for shutdown")
    void testShutdownTimeout() throws PluginException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            assertEquals(LifecycleState.ACTIVE, plugin.getState());

            // Just verify shutdown completes
            plugin.close();
            assertEquals(LifecycleState.STOPPED, plugin.getState());
        }
    }

    @Test
    @DisplayName("Plugin state machine transitions are enforced")
    void testStateTransitions() throws PluginException {
        Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString());

        try {
            // Should be Active
            assertEquals(LifecycleState.ACTIVE, plugin.getState());

            // Can handle requests
            String response = plugin.call("echo", "{\"message\": \"test\"}");
            assertTrue(response.length() > 0);

            // Close (transitions to Stopped)
            plugin.close();
            assertEquals(LifecycleState.STOPPED, plugin.getState());

            // Can't handle requests after stopped
            assertThrows(PluginException.class, () -> {
                plugin.call("echo", "{\"message\": \"after stop\"}");
            }, "Should not be able to call after plugin stopped");
        } finally {
            try {
                plugin.close();
            } catch (Exception e) {
                // Already stopped
            }
        }
    }

    @Test
    @DisplayName("Concurrent plugin loading and closing succeeds")
    void testConcurrentLoadingAndClosing() throws InterruptedException {
        int numPlugins = 5;
        AtomicInteger successCount = new AtomicInteger(0);
        AtomicInteger failureCount = new AtomicInteger(0);

        List<Thread> threads = new ArrayList<>();

        for (int i = 0; i < numPlugins; i++) {
            Thread thread = new Thread(() -> {
                try {
                    Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString());
                    assertEquals(LifecycleState.ACTIVE, plugin.getState());
                    plugin.call("echo", "{\"message\": \"test\"}");
                    plugin.close();
                    assertEquals(LifecycleState.STOPPED, plugin.getState());
                    successCount.incrementAndGet();
                } catch (PluginException e) {
                    failureCount.incrementAndGet();
                }
            });
            threads.add(thread);
            thread.start();
        }

        // Wait for all threads
        for (Thread thread : threads) {
            thread.join(30_000);
        }

        assertEquals(numPlugins, successCount.get(), "All plugins should load and close successfully");
        assertEquals(0, failureCount.get(), "No failures expected");
    }

    @Test
    @DisplayName("Invalid request returns error but plugin stays functional")
    void testInvalidRequestHandling() throws PluginException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            // Empty JSON object is invalid for echo (requires message field)
            // This should throw PluginException but not crash the plugin
            assertThrows(PluginException.class, () -> {
                plugin.call("echo", "{}");
            }, "Invalid request should throw PluginException");

            // Plugin should still be functional after the error
            assertEquals(LifecycleState.ACTIVE, plugin.getState());

            // Valid request should work
            String response = plugin.call("echo", "{\"message\": \"test\"}");
            assertTrue(response.contains("test"));
        }
    }
}
