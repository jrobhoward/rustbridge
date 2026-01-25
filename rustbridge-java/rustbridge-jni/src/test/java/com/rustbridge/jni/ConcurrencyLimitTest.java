package com.rustbridge.jni;

import com.rustbridge.*;
import com.rustbridge.jni.JniNativeLibraryCondition.RequiresJniLibrary;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for concurrency limiting and backpressure.
 */
@RequiresJniLibrary
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class ConcurrencyLimitTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Concurrency limit exceeded returns error")
    void concurrency_limit___exceeded___returns_error() throws Exception {
        PluginConfig config = PluginConfig.defaults()
                .maxConcurrentOps(2);

        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            ExecutorService executor = Executors.newFixedThreadPool(15);
            AtomicInteger successCount = new AtomicInteger(0);
            AtomicInteger errorCount = new AtomicInteger(0);

            List<Future<String>> futures = new ArrayList<>();
            for (int i = 0; i < 15; i++) {
                final int id = i;
                Future<String> future = executor.submit(() -> {
                    try {
                        String result = plugin.call("test.sleep", "{\"duration_ms\": 300}");
                        successCount.incrementAndGet();
                        return result;
                    } catch (PluginException e) {
                        errorCount.incrementAndGet();
                        throw e;
                    }
                });
                futures.add(future);
                Thread.sleep(10);
            }

            for (Future<String> future : futures) {
                try {
                    future.get(10, TimeUnit.SECONDS);
                } catch (ExecutionException e) {
                    // Expected
                } catch (TimeoutException e) {
                    fail("Request timed out");
                }
            }

            executor.shutdown();
            executor.awaitTermination(5, TimeUnit.SECONDS);

            System.out.println("Success: " + successCount.get() + ", Errors: " + errorCount.get());

            int total = successCount.get() + errorCount.get();
            assertEquals(15, total, "Total requests should be 15");
            assertTrue(successCount.get() >= 2 && successCount.get() <= 6,
                    "Expected 2-6 successful requests, got " + successCount.get());
            assertTrue(errorCount.get() >= 9, "Expected at least 9 rejected requests, got " + errorCount.get());

            long rejectedCount = plugin.getRejectedRequestCount();
            assertTrue(rejectedCount >= 9, "Rejected count should be at least 9");
            assertEquals(errorCount.get(), rejectedCount, "Error count should match rejected count");

            System.out.println("Rejected count: " + rejectedCount);
        }
    }

    @Test
    @DisplayName("Concurrency limit of zero means unlimited")
    void concurrency_limit___unlimited___all_succeed() throws Exception {
        PluginConfig config = PluginConfig.defaults()
                .maxConcurrentOps(0);

        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            ExecutorService executor = Executors.newFixedThreadPool(20);
            AtomicInteger successCount = new AtomicInteger(0);
            AtomicInteger errorCount = new AtomicInteger(0);

            List<Future<String>> futures = new ArrayList<>();
            for (int i = 0; i < 20; i++) {
                final int id = i;
                Future<String> future = executor.submit(() -> {
                    try {
                        String result = plugin.call("greet", "{\"name\": \"User" + id + "\"}");
                        successCount.incrementAndGet();
                        return result;
                    } catch (PluginException e) {
                        errorCount.incrementAndGet();
                        throw e;
                    }
                });
                futures.add(future);
            }

            for (Future<String> future : futures) {
                try {
                    future.get(5, TimeUnit.SECONDS);
                } catch (ExecutionException e) {
                    fail("No requests should fail with unlimited concurrency: " + e.getCause().getMessage());
                }
            }

            executor.shutdown();
            executor.awaitTermination(5, TimeUnit.SECONDS);

            assertEquals(20, successCount.get(), "All requests should succeed");
            assertEquals(0, errorCount.get(), "No requests should fail");
            assertEquals(0, plugin.getRejectedRequestCount(), "No requests should be rejected");
        }
    }

    @Test
    @DisplayName("Permits released after completion allows sequential calls")
    void concurrency_limit___permit_released___after_completion() throws Exception {
        PluginConfig config = PluginConfig.defaults()
                .maxConcurrentOps(1);

        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            for (int i = 0; i < 10; i++) {
                String result = plugin.call("greet", "{\"name\": \"User" + i + "\"}");
                assertNotNull(result);
            }

            assertEquals(0, plugin.getRejectedRequestCount(), "No requests should be rejected for sequential calls");
        }
    }

    @Test
    @DisplayName("Rejected request count tracks rejected requests")
    void rejected_request_count___tracks_rejected_requests() throws Exception {
        PluginConfig config = PluginConfig.defaults()
                .maxConcurrentOps(2);

        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            ExecutorService executor = Executors.newFixedThreadPool(15);

            // Use test.sleep to ensure requests overlap and hit the limit
            CountDownLatch startLatch = new CountDownLatch(1);
            List<Future<String>> futures = new ArrayList<>();

            for (int i = 0; i < 15; i++) {
                final int id = i;
                Future<String> future = executor.submit(() -> {
                    try {
                        startLatch.await();
                        // Use sleep handler to ensure concurrent execution
                        return plugin.call("test.sleep", "{\"duration_ms\": 200}");
                    } catch (PluginException e) {
                        return null; // Expected for rejected requests
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        return null;
                    }
                });
                futures.add(future);
            }

            // Release all threads at once
            startLatch.countDown();

            // Wait for all to complete
            for (Future<String> future : futures) {
                try {
                    future.get(10, TimeUnit.SECONDS);
                } catch (Exception e) {
                    // Ignore - some will timeout or fail
                }
            }

            executor.shutdown();
            executor.awaitTermination(5, TimeUnit.SECONDS);

            long rejectedCount = plugin.getRejectedRequestCount();
            assertTrue(rejectedCount > 0, "Some requests should have been rejected with limit of 2 and 15 concurrent requests");

            System.out.println("Rejected count: " + rejectedCount);
        }
    }
}
