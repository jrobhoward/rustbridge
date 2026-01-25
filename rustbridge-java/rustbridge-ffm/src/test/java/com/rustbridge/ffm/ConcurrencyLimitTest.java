package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Tests for concurrency limiting and backpressure.
 */
@Timeout(value = 60, unit = TimeUnit.SECONDS)  // Prevent tests from hanging indefinitely
class ConcurrencyLimitTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        // Find the hello-plugin library
        String osName = System.getProperty("os.name").toLowerCase();
        String libraryName;

        if (osName.contains("linux")) {
            libraryName = "libhello_plugin.so";
        } else if (osName.contains("mac") || osName.contains("darwin")) {
            libraryName = "libhello_plugin.dylib";
        } else if (osName.contains("windows")) {
            libraryName = "hello_plugin.dll";
        } else {
            throw new RuntimeException("Unsupported OS: " + osName);
        }

        // Look in target/release first, then target/debug
        Path releasePath = Paths.get("../../target/release").resolve(libraryName);
        Path debugPath = Paths.get("../../target/debug").resolve(libraryName);

        if (releasePath.toFile().exists()) {
            PLUGIN_PATH = releasePath.toAbsolutePath();
        } else if (debugPath.toFile().exists()) {
            PLUGIN_PATH = debugPath.toAbsolutePath();
        } else {
            fail("Could not find hello-plugin library. Build it with: cargo build -p hello-plugin");
        }

        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Concurrency limit exceeded returns error")
    void concurrency_limit___exceeded___returns_error() throws Exception {
        PluginConfig config = PluginConfig.defaults()
                .maxConcurrentOps(2);

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            ExecutorService executor = Executors.newFixedThreadPool(15);
            AtomicInteger successCount = new AtomicInteger(0);
            AtomicInteger errorCount = new AtomicInteger(0);

            // Submit 15 requests, staggered to ensure we hit the limit
            List<Future<String>> futures = new ArrayList<>();
            for (int i = 0; i < 15; i++) {
                final int id = i;
                Future<String> future = executor.submit(() -> {
                    try {
                        // Use sleep handler to hold permits longer (300ms)
                        String result = plugin.call("test.sleep", "{\"duration_ms\": 300}");
                        successCount.incrementAndGet();
                        return result;
                    } catch (PluginException e) {
                        errorCount.incrementAndGet();
                        throw e;
                    }
                });
                futures.add(future);

                // Small delay to stagger requests (first few will succeed, rest will be rejected)
                Thread.sleep(10);
            }

            // Wait for all to complete
            for (Future<String> future : futures) {
                try {
                    future.get(10, TimeUnit.SECONDS);
                } catch (ExecutionException e) {
                    // Expected - some will fail due to concurrency limit
                } catch (TimeoutException e) {
                    fail("Request timed out");
                }
            }

            executor.shutdown();
            executor.awaitTermination(5, TimeUnit.SECONDS);

            System.out.println("Success: " + successCount.get() + ", Errors: " + errorCount.get());

            // With limit of 2 and 15 requests staggered by 10ms with 300ms sleep each:
            // - First 2 start immediately and hold permits for 300ms
            // - By the time they finish, we've submitted all 15 requests (15 * 10ms = 150ms)
            // - Most of the remaining 13 will be rejected immediately
            // - After first 2 complete (~300ms), next batch can start but queue is mostly failed
            // Expected: 2-4 succeed (first batch), 11-13 rejected (arrived while busy)
            int total = successCount.get() + errorCount.get();
            assertEquals(15, total, "Total requests should be 15");
            assertTrue(successCount.get() >= 2 && successCount.get() <= 6,
                "Expected 2-6 successful requests, got " + successCount.get());
            assertTrue(errorCount.get() >= 9, "Expected at least 9 rejected requests, got " + errorCount.get());

            // Check rejected count
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
                .maxConcurrentOps(0);  // Unlimited

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            ExecutorService executor = Executors.newFixedThreadPool(20);
            AtomicInteger successCount = new AtomicInteger(0);
            AtomicInteger errorCount = new AtomicInteger(0);

            // Submit many concurrent requests
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

            // Wait for all to complete
            for (Future<String> future : futures) {
                try {
                    future.get(5, TimeUnit.SECONDS);
                } catch (ExecutionException e) {
                    fail("No requests should fail with unlimited concurrency: " + e.getCause().getMessage());
                }
            }

            executor.shutdown();
            executor.awaitTermination(5, TimeUnit.SECONDS);

            // All should succeed
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

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            // Make several sequential calls
            for (int i = 0; i < 10; i++) {
                String result = plugin.call("greet", "{\"name\": \"User" + i + "\"}");
                assertNotNull(result);
            }

            // All should succeed since they're sequential
            assertEquals(0, plugin.getRejectedRequestCount(), "No requests should be rejected for sequential calls");
        }
    }

    @Test
    @DisplayName("Rejected request count tracks rejected requests")
    void rejected_request_count___tracks_rejected_requests() throws Exception {
        PluginConfig config = PluginConfig.defaults()
                .maxConcurrentOps(2);

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString(), config)) {
            ExecutorService executor = Executors.newFixedThreadPool(20);

            // Submit many concurrent requests simultaneously
            CountDownLatch startLatch = new CountDownLatch(1);
            List<Future<String>> futures = new ArrayList<>();

            for (int i = 0; i < 20; i++) {
                final int id = i;
                Future<String> future = executor.submit(() -> {
                    try {
                        // Wait for all threads to be ready
                        startLatch.await();
                        return plugin.call("greet", "{\"name\": \"User" + id + "\"}");
                    } catch (PluginException e) {
                        return null; // Ignore errors for this test
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        return null;
                    }
                });
                futures.add(future);
            }

            // Release all threads at once to create contention
            startLatch.countDown();

            // Wait for all to complete
            for (Future<String> future : futures) {
                try {
                    future.get(5, TimeUnit.SECONDS);
                } catch (Exception e) {
                    // Ignore
                }
            }

            executor.shutdown();
            executor.awaitTermination(5, TimeUnit.SECONDS);

            // Check that some requests were rejected
            long rejectedCount = plugin.getRejectedRequestCount();
            assertTrue(rejectedCount > 0, "Some requests should have been rejected with limit of 2 and 20 concurrent requests");

            System.out.println("Rejected count: " + rejectedCount);
        }
    }
}
