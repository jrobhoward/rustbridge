package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Verify that logging works correctly after reload.
 * This test specifically checks that the callback is re-registered on reload.
 */
@Timeout(value = 60, unit = TimeUnit.SECONDS)  // Prevent tests from hanging indefinitely
class ReloadLoggingVerificationTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Logging works after reload with new callback")
    void testLoggingWorksAfterReload() throws PluginException, InterruptedException {
        List<String> firstLoadLogs = new ArrayList<>();
        List<String> secondLoadLogs = new ArrayList<>();

        // First load with callback
        LogCallback callback1 = (level, target, message) -> {
            firstLoadLogs.add("[FIRST] " + message);
        };

        System.out.println("=== First Load ===");
        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback1);

        // Set to INFO level (in case previous test changed it)
        plugin1.setLogLevel(LogLevel.INFO);

        // Wait for initialization to complete
        Thread.sleep(200);
        firstLoadLogs.clear();

        // Make a call that generates logs
        System.out.println("Calling user.create on first load...");
        plugin1.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");
        Thread.sleep(200);

        int firstLoadCount = firstLoadLogs.size();
        System.out.println("First load captured " + firstLoadCount + " logs");
        for (String log : firstLoadLogs) {
            System.out.println("  " + log);
        }

        assertTrue(firstLoadCount > 0, "First load should capture logs");

        // Shutdown
        System.out.println("\n=== Shutting Down ===");
        plugin1.close();
        Thread.sleep(500);

        // Reload with NEW callback
        LogCallback callback2 = (level, target, message) -> {
            secondLoadLogs.add("[SECOND] " + message);
        };

        System.out.println("\n=== Second Load (Reload) ===");
        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback2);

        // Wait for initialization
        Thread.sleep(200);
        secondLoadLogs.clear();

        // Make a call that generates logs
        System.out.println("Calling user.create on second load...");
        plugin2.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");
        Thread.sleep(200);

        int secondLoadCount = secondLoadLogs.size();
        System.out.println("Second load captured " + secondLoadCount + " logs");
        for (String log : secondLoadLogs) {
            System.out.println("  " + log);
        }

        plugin2.close();

        // CRITICAL ASSERTION: Second load should capture logs
        assertTrue(secondLoadCount > 0,
                "CRITICAL: Logging should work after reload (got " + secondLoadCount + " logs)");

        System.out.println("\n✓ SUCCESS: Logging works after reload!");
    }

    @Test
    @DisplayName("Log level persists across reload")
    void testLogLevelPersistsAcrossReload() throws PluginException, InterruptedException {
        List<String> debugLogs = new ArrayList<>();

        LogCallback callback = (level, target, message) -> {
            if (level == LogLevel.DEBUG) {
                debugLogs.add(message);
            }
        };

        // First load - set to DEBUG
        System.out.println("=== First Load ===");
        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback);
        plugin1.setLogLevel(LogLevel.DEBUG);
        Thread.sleep(100);

        plugin1.close();
        Thread.sleep(500);

        // Reload - level should PERSIST at DEBUG
        System.out.println("\n=== Reload ===");
        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback);

        // IMPORTANT: Log level persists across reload (global state)
        debugLogs.clear();
        plugin2.call("echo", "{\"message\": \"test\"}");
        Thread.sleep(100);
        int debugAtReload = debugLogs.size();

        System.out.println("DEBUG logs after reload: " + debugAtReload);
        assertTrue(debugAtReload > 0, "DEBUG level should persist across reload");

        // Can change to ERROR to reduce verbosity
        plugin2.setLogLevel(LogLevel.ERROR);
        Thread.sleep(100);
        debugLogs.clear();

        plugin2.call("echo", "{\"message\": \"test2\"}");
        Thread.sleep(100);
        int debugAtError = debugLogs.size();

        // Reset to INFO for other tests
        plugin2.setLogLevel(LogLevel.INFO);
        plugin2.close();

        System.out.println("DEBUG logs at ERROR level: " + debugAtError);
        assertEquals(0, debugAtError, "Should not see DEBUG logs at ERROR level");

        System.out.println("\n✓ SUCCESS: Log level persists and can be changed after reload!");
    }
}
