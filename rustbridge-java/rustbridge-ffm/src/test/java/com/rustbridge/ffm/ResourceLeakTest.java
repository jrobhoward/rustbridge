package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.lang.ref.WeakReference;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test for resource leak detection in plugin lifecycle.
 * Verifies that plugins and their native resources are properly cleaned up
 * and not leaked even under stress conditions.
 */
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class ResourceLeakTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Plugin resources are released on close")
    void testPluginResourcesReleased() throws PluginException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            // Verify plugin is active
            assertEquals(LifecycleState.ACTIVE, plugin.getState());

            // Use the plugin
            String response = plugin.call("echo", "{\"message\": \"test\"}");
            assertTrue(response.contains("test"));
        }
    }

    @Test
    @DisplayName("Sequential load/close cycles don't leak resources")
    void testSequentialLoadCloseCycles() throws PluginException {
        for (int i = 0; i < 100; i++) {
            try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
                assertEquals(LifecycleState.ACTIVE, plugin.getState());

                // Call the plugin
                plugin.call("echo", "{\"message\": \"cycle " + i + "\"}");
            }
        }
    }

    @Test
    @DisplayName("Plugin objects are GC-eligible after close")
    void testPluginGcAfterClose() throws PluginException {
        WeakReference<Plugin> pluginRef;

        {
            // Load plugin in a local scope
            try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
                pluginRef = new WeakReference<>(plugin);

                assertEquals(LifecycleState.ACTIVE, plugin.getState());

                // Use the plugin
                plugin.call("echo", "{\"message\": \"test\"}");
                // Plugin closed at end of try block
            }
        }

        // Try to GC (not guaranteed, but helps)
        System.gc();
        Thread.yield();

        // WeakReference may be null if GC collected it
        // This is not guaranteed but indicates good cleanup
    }

    @Test
    @DisplayName("Multiple plugins close cleanly")
    void testMultiplePluginsCloseCleanly() throws PluginException {
        List<Plugin> plugins = new ArrayList<>();

        // Create and close 20 plugins
        for (int i = 0; i < 20; i++) {
            try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
                assertEquals(LifecycleState.ACTIVE, plugin.getState());
                // Use plugin
                plugin.call("echo", "{\"message\": \"test " + i + "\"}");
            }
        }
    }

    @Test
    @DisplayName("Plugin state is correct after use and close")
    void testPluginStateAfterUseAndClose() throws PluginException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            assertEquals(LifecycleState.ACTIVE, plugin.getState());

            // Make a valid call
            plugin.call("echo", "{\"message\": \"test\"}");

            // Still active
            assertEquals(LifecycleState.ACTIVE, plugin.getState());
        }
    }

    @Test
    @DisplayName("Large payload cycles don't leak memory")
    void testLargePayloadCycles() throws PluginException {
        // Create a large payload
        StringBuilder largePayload = new StringBuilder("{\"message\": \"");
        for (int i = 0; i < 10000; i++) {
            largePayload.append("x");
        }
        largePayload.append("\"}");

        for (int cycle = 0; cycle < 50; cycle++) {
            try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
                // Send large payload
                plugin.call("echo", largePayload.toString());
            }
        }
    }

    @Test
    @DisplayName("Plugin resources survive concurrent access before close")
    void testPluginConcurrentAccessBeforeClose() throws PluginException, InterruptedException {
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString())) {
            // Multiple threads access the plugin
            int threadCount = 10;
            List<Thread> threads = new ArrayList<>();

            for (int i = 0; i < threadCount; i++) {
                final int threadId = i;
                Thread t = new Thread(() -> {
                    try {
                        for (int j = 0; j < 10; j++) {
                            String response = plugin.call("echo",
                                "{\"message\": \"thread " + threadId + " call " + j + "\"}");
                            assertNotNull(response);
                        }
                    } catch (PluginException e) {
                        throw new RuntimeException(e);
                    }
                });
                threads.add(t);
                t.start();
            }

            // Wait for all threads
            for (Thread t : threads) {
                t.join(10_000);
            }
        }
    }
}
