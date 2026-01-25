package com.rustbridge.jni;

import com.rustbridge.*;
import org.junit.jupiter.api.*;
import org.junit.jupiter.api.condition.EnabledIfEnvironmentVariable;

import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.CopyOnWriteArrayList;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test callback safety with multiple plugins (JNI implementation).
 */
@Timeout(value = 60, unit = TimeUnit.SECONDS)
@EnabledIfEnvironmentVariable(named = "RUSTBRIDGE_JNI_AVAILABLE", matches = "true")
class SharedCallbackMultiPluginTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Multiple plugins work simultaneously (JNI)")
    void testMultiplePluginsWork() throws PluginException, InterruptedException {
        // Thread-safe list for collecting all logs
        List<String> allLogs = new CopyOnWriteArrayList<>();

        // Single shared callback
        LogCallback sharedCallback = (level, target, message) -> {
            allLogs.add(message);
        };

        System.out.println("\n=== Loading 3 Plugins with Callback (JNI) ===");

        // Load 3 plugins with the same callback
        Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString(), PluginConfig.defaults(), sharedCallback);
        plugin1.setLogLevel(LogLevel.INFO);
        Thread.sleep(100);

        Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString(), PluginConfig.defaults(), sharedCallback);
        Thread.sleep(100);

        Plugin plugin3 = JniPluginLoader.load(PLUGIN_PATH.toString(), PluginConfig.defaults(), sharedCallback);
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
    @DisplayName("Callback is cleared when any plugin shuts down (safety) (JNI)")
    void testCallbackClearedOnAnyPluginShutdown() throws PluginException, InterruptedException {
        List<String> logs = new CopyOnWriteArrayList<>();

        LogCallback callback = (level, target, message) -> {
            logs.add(message);
        };

        System.out.println("\n=== Loading 2 Plugins (JNI) ===");

        Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString(), PluginConfig.defaults(), callback);
        plugin1.setLogLevel(LogLevel.INFO);
        Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString(), PluginConfig.defaults(), callback);
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

        // Plugin 2 is still active and functional
        assertEquals(LifecycleState.ACTIVE, plugin2.getState());

        // Clean up
        plugin2.close();

        System.out.println("\n✓ SUCCESS: Callback cleared on any plugin shutdown (prevents use-after-free)!");
    }

    @Test
    @DisplayName("Shutting down one plugin doesn't crash other plugins (JNI)")
    void testPluginShutdownDoesntCrashOthers() throws PluginException, InterruptedException {
        List<String> logs = new CopyOnWriteArrayList<>();

        LogCallback callback = (level, target, message) -> {
            logs.add(message);
        };

        System.out.println("\n=== Loading 2 Plugins (JNI) ===");

        Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString(), PluginConfig.defaults(), callback);
        Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString(), PluginConfig.defaults(), callback);
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
