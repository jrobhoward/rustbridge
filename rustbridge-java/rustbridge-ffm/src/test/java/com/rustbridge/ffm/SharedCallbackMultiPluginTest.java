package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test that multiple plugins can coexist with a shared callback.
 *
 * This verifies the reference-counting behavior where:
 * - Multiple plugins can be loaded simultaneously
 * - All plugins share the same log callback
 * - Shutting down one plugin doesn't affect others
 * - The callback is only cleared when the last plugin shuts down
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
    @DisplayName("Multiple plugins share the same callback")
    void testMultiplePluginsShareCallback() throws PluginException, InterruptedException {
        // Thread-safe list for collecting all logs from all plugins
        List<String> allLogs = new CopyOnWriteArrayList<>();

        // Single shared callback for all plugins
        LogCallback sharedCallback = (level, target, message) -> {
            allLogs.add(message);
        };

        System.out.println("\n=== Loading 3 Plugins with Shared Callback ===");

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

        // Shut down plugin1
        System.out.println("\n=== Shutting Down Plugin 1 ===");
        plugin1.close();
        Thread.sleep(100);
        allLogs.clear();

        // Plugins 2 and 3 should still be able to log
        System.out.println("Plugins 2 and 3 should still log...");
        plugin2.call("user.create", "{\"username\": \"user4\", \"email\": \"user4@example.com\"}");
        Thread.sleep(50);

        plugin3.call("user.create", "{\"username\": \"user5\", \"email\": \"user5@example.com\"}");
        Thread.sleep(100);

        int logsAfterPlugin1Shutdown = allLogs.size();
        System.out.println("Logs after plugin1 shutdown: " + logsAfterPlugin1Shutdown);
        assertTrue(logsAfterPlugin1Shutdown >= 2,
                  "Plugins 2 and 3 should still be able to log after plugin1 shuts down");

        // Shut down plugin2
        System.out.println("\n=== Shutting Down Plugin 2 ===");
        plugin2.close();
        Thread.sleep(100);
        allLogs.clear();

        // Plugin 3 should still be able to log
        System.out.println("Plugin 3 should still log...");
        plugin3.call("user.create", "{\"username\": \"user6\", \"email\": \"user6@example.com\"}");
        Thread.sleep(100);

        int logsAfterPlugin2Shutdown = allLogs.size();
        System.out.println("Logs after plugin2 shutdown: " + logsAfterPlugin2Shutdown);
        assertTrue(logsAfterPlugin2Shutdown >= 1,
                  "Plugin 3 should still be able to log after plugin2 shuts down");

        // Shut down plugin3 (the last one)
        System.out.println("\n=== Shutting Down Plugin 3 (Last Plugin) ===");
        plugin3.close();
        Thread.sleep(100);

        System.out.println("\n✓ SUCCESS: Multiple plugins coexist with shared callback!");
    }

    @Test
    @DisplayName("Callback is cleared only when last plugin exits")
    void testCallbackClearedOnlyWhenLastPluginExits() throws PluginException, InterruptedException {
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

        // Both plugins work
        plugin1.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");
        plugin2.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");
        Thread.sleep(100);

        assertTrue(logs.size() >= 2, "Both plugins should generate logs");

        // Close first plugin
        System.out.println("\n=== Closing First Plugin ===");
        plugin1.close();
        Thread.sleep(100);
        logs.clear();

        // Second plugin should still work
        plugin2.call("user.create", "{\"username\": \"charlie\", \"email\": \"charlie@example.com\"}");
        Thread.sleep(100);

        assertTrue(logs.size() >= 1, "Second plugin should still log after first closes");

        // Reload a new plugin while plugin2 is still active
        System.out.println("\n=== Loading Third Plugin While Second is Active ===");
        Plugin plugin3 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback);
        Thread.sleep(100);
        logs.clear();

        // Both should work
        plugin2.call("user.create", "{\"username\": \"dave\", \"email\": \"dave@example.com\"}");
        plugin3.call("user.create", "{\"username\": \"eve\", \"email\": \"eve@example.com\"}");
        Thread.sleep(100);

        assertTrue(logs.size() >= 2, "Both active plugins should log");

        // Clean up
        plugin2.close();
        plugin3.close();

        System.out.println("\n✓ SUCCESS: Reference counting works correctly!");
    }
}
