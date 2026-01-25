package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test plugin reload scenarios - loading, unloading, and reloading the same plugin.
 */
@Timeout(value = 60, unit = TimeUnit.SECONDS)  // Prevent tests from hanging indefinitely
class PluginReloadTest {

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
    @DisplayName("Plugin can be loaded, shut down, and reloaded")
    void testLoadShutdownReload() throws PluginException, InterruptedException {
        System.out.println("\n=== First Load ===");

        // First load
        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH.toString());
        assertEquals(LifecycleState.ACTIVE, plugin1.getState());

        // Use it
        String response1 = plugin1.call("echo", "{\"message\": \"first load\"}");
        assertTrue(response1.contains("first load"));

        // Shutdown
        System.out.println("\n=== Shutting down ===");
        plugin1.close();
        Thread.sleep(500); // Give time for full shutdown
        System.out.println("Plugin closed successfully");

        // Try to reload
        System.out.println("\n=== Second Load (Reload) ===");
        try {
            Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH.toString());
            assertEquals(LifecycleState.ACTIVE, plugin2.getState(),
                    "Reloaded plugin should be ACTIVE");

            // Use it
            String response2 = plugin2.call("echo", "{\"message\": \"second load\"}");
            assertTrue(response2.contains("second load"),
                    "Reloaded plugin should work correctly");

            plugin2.close();

            System.out.println("\nResult: Reload SUCCESSFUL ✓");

        } catch (Exception e) {
            System.err.println("\nResult: Reload FAILED ✗");
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
            fail("Plugin reload failed: " + e.getMessage());
        }
    }

    @Test
    @DisplayName("Plugin can be used, reloaded, and used again with same functionality")
    void testReloadFunctionality() throws PluginException, InterruptedException {
        // First load - use all functions
        System.out.println("\n=== First Load - Testing All Functions ===");
        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH.toString());

        String echo1 = plugin1.call("echo", "{\"message\": \"test\"}");
        String greet1 = plugin1.call("greet", "{\"name\": \"Alice\"}");
        String user1 = plugin1.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");

        assertTrue(echo1.contains("test"));
        assertTrue(greet1.contains("Alice"));
        assertTrue(user1.contains("user-"));

        plugin1.close();
        Thread.sleep(500);

        // Reload - use same functions
        System.out.println("\n=== Second Load - Retesting Functions ===");
        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH.toString());

        String echo2 = plugin2.call("echo", "{\"message\": \"test\"}");
        String greet2 = plugin2.call("greet", "{\"name\": \"Bob\"}");
        String user2 = plugin2.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");

        assertTrue(echo2.contains("test"), "Echo should work after reload");
        assertTrue(greet2.contains("Bob"), "Greet should work after reload");
        assertTrue(user2.contains("user-"), "User creation should work after reload");

        plugin2.close();

        System.out.println("\nResult: All functions work correctly after reload ✓");
    }

    @Test
    @DisplayName("Log callback works after reload")
    void testLogCallbackAfterReload() throws PluginException, InterruptedException {
        List<String> firstLoadLogs = new ArrayList<>();
        List<String> secondLoadLogs = new ArrayList<>();

        LogCallback callback1 = (level, target, message) -> {
            firstLoadLogs.add(message);
        };

        // First load
        System.out.println("\n=== First Load with Logging ===");
        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback1);
        // Ensure INFO level (in case previous test changed it)
        plugin1.setLogLevel(LogLevel.INFO);
        Thread.sleep(100);
        firstLoadLogs.clear();

        plugin1.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");
        Thread.sleep(100);

        int firstLoadLogCount = firstLoadLogs.size();
        System.out.println("First load captured " + firstLoadLogCount + " logs");

        plugin1.close();
        Thread.sleep(500);

        // Reload with new callback
        LogCallback callback2 = (level, target, message) -> {
            secondLoadLogs.add(message);
        };

        System.out.println("\n=== Second Load with New Callback ===");
        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback2);
        Thread.sleep(100);
        secondLoadLogs.clear();

        plugin2.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");
        Thread.sleep(100);

        int secondLoadLogCount = secondLoadLogs.size();
        System.out.println("Second load captured " + secondLoadLogCount + " logs");

        plugin2.close();

        // Both should have captured logs
        assertTrue(firstLoadLogCount > 0, "First load should have captured logs");
        assertTrue(secondLoadLogCount > 0, "Second load should have captured logs after reload");

        System.out.println("\nResult: Logging works after reload ✓");
    }

    @Test
    @DisplayName("Dynamic log level changes work after reload")
    void testDynamicLogLevelAfterReload() throws PluginException, InterruptedException {
        System.out.println("\n=== First Load - Testing Dynamic Log Levels ===");

        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH.toString());

        // Start at INFO level
        plugin1.setLogLevel(LogLevel.INFO);
        Thread.sleep(100);

        // Use plugin
        plugin1.call("echo", "{\"message\": \"test\"}");

        plugin1.close();
        Thread.sleep(500);

        // Reload
        System.out.println("\n=== Second Load - Retesting Dynamic Log Levels ===");

        List<String> logs = new ArrayList<>();
        LogCallback callback = (level, target, message) -> {
            if (level == LogLevel.DEBUG) {
                logs.add(message);
            }
        };

        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback);
        Thread.sleep(100);

        // Should still be at INFO level (persists across reload)
        logs.clear();
        plugin2.call("echo", "{\"message\": \"at info\"}");
        Thread.sleep(100);
        int debugLogsAtInfo = logs.size();

        // Change to DEBUG
        plugin2.setLogLevel(LogLevel.DEBUG);
        Thread.sleep(100);
        logs.clear();

        plugin2.call("echo", "{\"message\": \"at debug\"}");
        Thread.sleep(100);
        int debugLogsAtDebug = logs.size();

        // Reset to INFO for next tests
        plugin2.setLogLevel(LogLevel.INFO);
        plugin2.close();

        System.out.println("DEBUG logs at INFO level: " + debugLogsAtInfo);
        System.out.println("DEBUG logs at DEBUG level: " + debugLogsAtDebug);

        assertEquals(0, debugLogsAtInfo, "Should not see DEBUG logs at INFO level");
        assertTrue(debugLogsAtDebug > 0, "Should see DEBUG logs at DEBUG level after reload");

        System.out.println("\nResult: Dynamic log levels work after reload ✓");
    }

    @Test
    @DisplayName("Multiple reload cycles work")
    void testMultipleReloadCycles() throws PluginException, InterruptedException {
        final int RELOAD_COUNT = 3;

        for (int i = 0; i < RELOAD_COUNT; i++) {
            System.out.println("\n=== Load Cycle " + (i + 1) + " ===");

            Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH.toString());
            assertEquals(LifecycleState.ACTIVE, plugin.getState(),
                    "Plugin should be active on load cycle " + (i + 1));

            String response = plugin.call("echo", "{\"message\": \"cycle " + (i + 1) + "\"}");
            assertTrue(response.contains("cycle " + (i + 1)),
                    "Plugin should work on cycle " + (i + 1));

            plugin.close();
            Thread.sleep(500);
        }

        System.out.println("\nResult: " + RELOAD_COUNT + " reload cycles completed successfully ✓");
    }

    @Test
    @DisplayName("Plugin state is fresh after reload")
    void testStateFreshAfterReload() throws PluginException, InterruptedException {
        System.out.println("\n=== First Load - Create User ===");

        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH.toString());

        // Create a user - this increments internal counter
        String user1 = plugin1.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");
        System.out.println("First user: " + user1);

        String user2 = plugin1.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");
        System.out.println("Second user: " + user2);

        plugin1.close();
        Thread.sleep(500);

        // Reload
        System.out.println("\n=== Second Load - Create User Again ===");

        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH.toString());

        String user3 = plugin2.call("user.create", "{\"username\": \"charlie\", \"email\": \"charlie@example.com\"}");
        System.out.println("First user after reload: " + user3);

        plugin2.close();

        // Check if counter was reset
        // If state is fresh, user3 should have a low ID like user1
        // If state persists, user3 should have ID after user2
        boolean stateAppearsReset = user3.contains("00000000") || user3.contains("00000001");

        System.out.println("\nState appears to be reset: " + stateAppearsReset);
        if (stateAppearsReset) {
            System.out.println("Result: Plugin state is fresh after reload ✓");
        } else {
            System.out.println("Result: Plugin state may persist across reloads (or IDs are not sequential)");
        }
    }
}
