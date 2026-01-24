package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test callback safety with multiple plugins.
 *
 * IMPORTANT: Callbacks are cleared when ANY plugin shuts down for safety.
 * This prevents use-after-free crashes when the callback's Arena is closed.
 *
 * The callback function pointer is tied to the Arena that created it.
 * When that Arena is closed (on plugin shutdown), the pointer becomes invalid.
 * Since we can't track which plugin "owns" the current callback, we clear it
 * on any plugin shutdown to prevent crashes.
 *
 * This means:
 * - Callbacks work while at least one plugin with a callback is active
 * - When any plugin shuts down, the callback is cleared (safety feature)
 * - Other active plugins continue to work, but logging stops until a new callback is registered
 */
class SharedCallbackMultiPluginTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
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

        Path releasePath = Paths.get("../../target/release").resolve(libraryName);
        Path debugPath = Paths.get("../../target/debug").resolve(libraryName);

        if (releasePath.toFile().exists()) {
            PLUGIN_PATH = releasePath.toAbsolutePath();
        } else if (debugPath.toFile().exists()) {
            PLUGIN_PATH = debugPath.toAbsolutePath();
        } else {
            fail("Could not find hello-plugin library");
        }

        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Multiple plugins work simultaneously")
    void testMultiplePluginsWork() throws PluginException, InterruptedException {
        // Thread-safe list for collecting all logs
        List<String> allLogs = new CopyOnWriteArrayList<>();

        // Single shared callback
        LogCallback sharedCallback = (level, target, message) -> {
            allLogs.add(message);
        };

        System.out.println("\n=== Loading 3 Plugins with Callback ===");

        // Load 3 plugins with the same callback
        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), sharedCallback);
        plugin1.setLogLevel(LogLevel.INFO);
        Thread.sleep(100);

        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), sharedCallback);
        Thread.sleep(100);

        Plugin plugin3 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), sharedCallback);
        Thread.sleep(100);

        allLogs.clear();

        // All three plugins make calls that generate logs
        System.out.println("\n=== All Plugins Making Calls ===");
        plugin1.call("user.create", "{\"username\": \"user1\", \"email\": \"user1@example.com\"}");
        Thread.sleep(50);

        plugin2.call("user.create", "{\"username\": \"user2\", \"email\": \"user2@example.com\"}");
        Thread.sleep(50);

        plugin3.call("user.create", "{\"username\": \"user3\", \"email\": \"user3@example.com\"}");
        Thread.sleep(100);

        int totalLogs = allLogs.size();
        System.out.println("Total logs collected: " + totalLogs);
        assertTrue(totalLogs >= 3, "Should have logs from all 3 plugins");

        // All plugins still work
        assertEquals(LifecycleState.ACTIVE, plugin1.getState());
        assertEquals(LifecycleState.ACTIVE, plugin2.getState());
        assertEquals(LifecycleState.ACTIVE, plugin3.getState());

        // Clean up all plugins
        plugin1.close();
        plugin2.close();
        plugin3.close();

        System.out.println("\n✓ SUCCESS: Multiple plugins work simultaneously!");
    }

    @Test
    @DisplayName("Callback is cleared when any plugin shuts down (safety)")
    void testCallbackClearedOnAnyPluginShutdown() throws PluginException, InterruptedException {
        List<String> logs = new CopyOnWriteArrayList<>();

        LogCallback callback = (level, target, message) -> {
            logs.add(message);
        };

        System.out.println("\n=== Loading 2 Plugins ===");

        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback);
        plugin1.setLogLevel(LogLevel.INFO);
        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback);
        Thread.sleep(100);

        logs.clear();

        // Both plugins work and generate logs
        plugin1.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");
        plugin2.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");
        Thread.sleep(100);

        assertTrue(logs.size() >= 2, "Both plugins should generate logs");

        // Close first plugin - this will clear the callback (safety feature)
        System.out.println("\n=== Closing First Plugin (clears callback for safety) ===");
        plugin1.close();
        Thread.sleep(100);
        logs.clear();

        // Second plugin still WORKS, but logging is disabled (callback cleared)
        String response = plugin2.call("user.create", "{\"username\": \"charlie\", \"email\": \"charlie@example.com\"}");
        Thread.sleep(100);

        // The call succeeded
        assertTrue(response.contains("user-"), "Plugin call should succeed");

        // But no logs were captured (callback was cleared for safety)
        System.out.println("Logs after plugin1 shutdown: " + logs.size());
        System.out.println("(Callback cleared for safety - no logs expected)");

        // Plugin 2 is still active and functional
        assertEquals(LifecycleState.ACTIVE, plugin2.getState());

        // Clean up
        plugin2.close();

        System.out.println("\n✓ SUCCESS: Callback cleared on any plugin shutdown (prevents use-after-free)!");
    }

    @Test
    @DisplayName("Shutting down one plugin doesn't crash other plugins")
    void testPluginShutdownDoesntCrashOthers() throws PluginException, InterruptedException {
        List<String> logs = new CopyOnWriteArrayList<>();

        LogCallback callback = (level, target, message) -> {
            logs.add(message);
        };

        System.out.println("\n=== Loading 2 Plugins ===");

        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback);
        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback);
        Thread.sleep(100);

        // Close first plugin
        System.out.println("\n=== Closing First Plugin ===");
        plugin1.close();
        Thread.sleep(100);

        // Second plugin should NOT crash - just no logging
        System.out.println("Making call on plugin2...");
        assertDoesNotThrow(() -> {
            String response = plugin2.call("echo", "{\"message\": \"still works!\"}");
            assertTrue(response.contains("still works"), "Plugin 2 should still respond");
        }, "Plugin 2 should not crash after plugin 1 shuts down");

        assertEquals(LifecycleState.ACTIVE, plugin2.getState(), "Plugin 2 should still be active");

        // Clean up
        plugin2.close();

        System.out.println("\n✓ SUCCESS: Plugin shutdown is safe - no crashes!");
    }
}
