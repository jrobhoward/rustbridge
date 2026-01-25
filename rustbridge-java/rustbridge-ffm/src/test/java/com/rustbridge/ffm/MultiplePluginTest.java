package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test loading multiple plugin instances in the same process.
 * This verifies whether plugins share global state or are properly isolated.
 */
@Timeout(value = 60, unit = TimeUnit.SECONDS)  // Prevent tests from hanging indefinitely
class MultiplePluginTest {

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
    @DisplayName("Two plugins can be loaded simultaneously")
    void testTwoPluginsSimultaneously() throws PluginException {
        // Arrange - load two instances of the same plugin
        try (Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH.toString());
             Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH.toString())) {

            // Act - both should work independently
            String response1 = plugin1.call("echo", "{\"message\": \"from plugin 1\"}");
            String response2 = plugin2.call("echo", "{\"message\": \"from plugin 2\"}");

            // Assert
            assertTrue(response1.contains("from plugin 1"), "Plugin 1 should echo its message");
            assertTrue(response2.contains("from plugin 2"), "Plugin 2 should echo its message");

            // Both should be in Active state
            assertEquals(LifecycleState.ACTIVE, plugin1.getState());
            assertEquals(LifecycleState.ACTIVE, plugin2.getState());
        }
    }

    @Test
    @DisplayName("Multiple plugins have independent log callbacks")
    void testMultiplePluginsIndependentLogCallbacks() throws PluginException {
        // Arrange - separate log collectors for each plugin
        List<String> plugin1Logs = new ArrayList<>();
        List<String> plugin2Logs = new ArrayList<>();

        LogCallback callback1 = (level, target, message) -> {
            plugin1Logs.add("[P1] " + level + ": " + message);
        };

        LogCallback callback2 = (level, target, message) -> {
            plugin2Logs.add("[P2] " + level + ": " + message);
        };

        // Act - load two plugins with different callbacks
        try (Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback1);
             Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback2)) {

            Thread.sleep(100); // Let initialization logs settle

            plugin1Logs.clear();
            plugin2Logs.clear();

            // Make calls to each plugin
            plugin1.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");
            plugin2.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");

            Thread.sleep(100);

            // Assert
            System.out.println("\nPlugin 1 logs (" + plugin1Logs.size() + " total):");
            for (String log : plugin1Logs) {
                System.out.println("  " + log);
            }

            System.out.println("\nPlugin 2 logs (" + plugin2Logs.size() + " total):");
            for (String log : plugin2Logs) {
                System.out.println("  " + log);
            }

            // If callbacks are independent, each should only see its own logs
            // If callbacks are shared, both will see all logs
            boolean callback1GotPlugin1Logs = plugin1Logs.stream()
                    .anyMatch(log -> log.contains("alice"));
            boolean callback1GotPlugin2Logs = plugin1Logs.stream()
                    .anyMatch(log -> log.contains("bob"));
            boolean callback2GotPlugin1Logs = plugin2Logs.stream()
                    .anyMatch(log -> log.contains("alice"));
            boolean callback2GotPlugin2Logs = plugin2Logs.stream()
                    .anyMatch(log -> log.contains("bob"));

            System.out.println("\nCallback isolation check:");
            System.out.println("  Callback1 got plugin1 logs: " + callback1GotPlugin1Logs);
            System.out.println("  Callback1 got plugin2 logs: " + callback1GotPlugin2Logs);
            System.out.println("  Callback2 got plugin1 logs: " + callback2GotPlugin1Logs);
            System.out.println("  Callback2 got plugin2 logs: " + callback2GotPlugin2Logs);

            // We expect BOTH callbacks to see ALL logs (shared state)
            // OR each callback to only see its own logs (isolated state)
            // Let's check which one is true
            if (callback1GotPlugin1Logs && callback1GotPlugin2Logs &&
                callback2GotPlugin1Logs && callback2GotPlugin2Logs) {
                System.out.println("\nResult: Log callbacks are SHARED between plugins");
            } else if (callback1GotPlugin1Logs && !callback1GotPlugin2Logs &&
                      !callback2GotPlugin1Logs && callback2GotPlugin2Logs) {
                System.out.println("\nResult: Log callbacks are ISOLATED per plugin");
            } else {
                System.out.println("\nResult: INCONSISTENT - unexpected behavior");
            }

        } catch (InterruptedException e) {
            fail("Test interrupted");
        }
    }

    @Test
    @DisplayName("Multiple plugins have independent log levels")
    void testMultiplePluginsIndependentLogLevels() throws PluginException {
        // Arrange
        AtomicInteger plugin1DebugCount = new AtomicInteger(0);
        AtomicInteger plugin2DebugCount = new AtomicInteger(0);

        LogCallback callback1 = (level, target, message) -> {
            if (level == LogLevel.DEBUG) {
                plugin1DebugCount.incrementAndGet();
            }
        };

        LogCallback callback2 = (level, target, message) -> {
            if (level == LogLevel.DEBUG) {
                plugin2DebugCount.incrementAndGet();
            }
        };

        try (Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback1);
             Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback2)) {

            Thread.sleep(100);

            // Set plugin1 to DEBUG, plugin2 to ERROR
            plugin1.setLogLevel(LogLevel.DEBUG);
            plugin2.setLogLevel(LogLevel.ERROR);
            Thread.sleep(100);

            plugin1DebugCount.set(0);
            plugin2DebugCount.set(0);

            // Make calls that generate DEBUG logs
            plugin1.call("echo", "{\"message\": \"test1\"}");
            plugin2.call("echo", "{\"message\": \"test2\"}");
            Thread.sleep(100);

            int plugin1Debug = plugin1DebugCount.get();
            int plugin2Debug = plugin2DebugCount.get();

            System.out.println("\nLog level isolation check:");
            System.out.println("  Plugin1 (DEBUG level) saw " + plugin1Debug + " DEBUG logs");
            System.out.println("  Plugin2 (ERROR level) saw " + plugin2Debug + " DEBUG logs");

            // If log levels are independent:
            //   plugin1 should see DEBUG logs (>0)
            //   plugin2 should NOT see DEBUG logs (0)
            // If log levels are shared:
            //   Both see the same amount of DEBUG logs

            if (plugin1Debug > 0 && plugin2Debug == 0) {
                System.out.println("\nResult: Log levels are ISOLATED per plugin ✓");
            } else if (plugin1Debug > 0 && plugin2Debug > 0) {
                System.out.println("\nResult: Log levels are SHARED between plugins (last setLogLevel wins)");
            } else {
                System.out.println("\nResult: Unexpected - plugin1 should see DEBUG logs");
            }

        } catch (InterruptedException e) {
            fail("Test interrupted");
        }
    }

    @Test
    @DisplayName("Shutting down one plugin doesn't affect another")
    void testShutdownOnePluginDoesntAffectOther() throws PluginException, InterruptedException {
        Plugin plugin1 = FfmPluginLoader.load(PLUGIN_PATH.toString());
        Plugin plugin2 = FfmPluginLoader.load(PLUGIN_PATH.toString());

        try {
            // Both should work
            assertDoesNotThrow(() -> plugin1.call("echo", "{\"message\": \"test1\"}"));
            assertDoesNotThrow(() -> plugin2.call("echo", "{\"message\": \"test2\"}"));

            // Close plugin1
            plugin1.close();
            Thread.sleep(100);

            // Plugin2 should still be ACTIVE and functional
            assertEquals(LifecycleState.ACTIVE, plugin2.getState(),
                        "Plugin2 should still be active");
            assertDoesNotThrow(() -> plugin2.call("echo", "{\"message\": \"test3\"}"),
                              "Plugin2 should still work after plugin1 shutdown");

        } finally {
            // Clean up
            try {
                plugin2.close();
            } catch (Exception e) {
                // Ignore
            }
        }
    }
}
