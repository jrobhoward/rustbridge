package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;
import org.junit.jupiter.api.MethodOrderer;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test that log levels can be changed dynamically at runtime.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
@Timeout(value = 60, unit = TimeUnit.SECONDS)  // Prevent tests from hanging indefinitely
class DynamicLogLevelTest {

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
    @Order(1)
    @DisplayName("Log level can be changed dynamically at runtime")
    void testDynamicLogLevelChanges() throws PluginException {
        // Arrange - capture all log messages
        List<LogMessage> capturedLogs = new ArrayList<>();

        LogCallback callback = (level, target, message) -> {
            capturedLogs.add(new LogMessage(level, target, message));
        };

        // Act - start with INFO level
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback)) {
            plugin.setLogLevel(LogLevel.INFO);
            Thread.sleep(100); // Let log level change propagate

            // Clear initialization logs
            capturedLogs.clear();

            // Make calls that generate DEBUG and INFO logs
            // echo handler uses tracing::debug!
            plugin.call("echo", "{\"message\": \"test1\"}");
            // user.create handler uses tracing::info!
            plugin.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");

            Thread.sleep(100); // Let logs be captured

            // Assert - should see INFO logs but NOT DEBUG logs
            long debugCount = capturedLogs.stream()
                    .filter(log -> log.level == LogLevel.DEBUG)
                    .count();
            long infoCount = capturedLogs.stream()
                    .filter(log -> log.level == LogLevel.INFO)
                    .count();

            System.out.println("At INFO level - DEBUG logs: " + debugCount + ", INFO logs: " + infoCount);
            assertEquals(0, debugCount, "Should not see DEBUG logs when level is INFO");
            assertTrue(infoCount > 0, "Should see INFO logs when level is INFO");

            // Change to DEBUG level
            capturedLogs.clear();
            plugin.setLogLevel(LogLevel.DEBUG);
            Thread.sleep(100);

            // Make the same calls
            plugin.call("echo", "{\"message\": \"test2\"}");
            plugin.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");

            Thread.sleep(100);

            // Assert - should now see both DEBUG and INFO logs
            debugCount = capturedLogs.stream()
                    .filter(log -> log.level == LogLevel.DEBUG)
                    .count();
            infoCount = capturedLogs.stream()
                    .filter(log -> log.level == LogLevel.INFO)
                    .count();

            System.out.println("At DEBUG level - DEBUG logs: " + debugCount + ", INFO logs: " + infoCount);
            System.out.println("All captured logs at DEBUG level:");
            for (LogMessage log : capturedLogs) {
                System.out.println("  " + log);
            }
            assertTrue(debugCount > 0, "Should see DEBUG logs when level is DEBUG");
            assertTrue(infoCount > 0, "Should see INFO logs when level is DEBUG");

            // Change to ERROR level
            capturedLogs.clear();
            plugin.setLogLevel(LogLevel.ERROR);
            Thread.sleep(100);

            // Make the same calls
            plugin.call("echo", "{\"message\": \"test3\"}");
            plugin.call("user.create", "{\"username\": \"charlie\", \"email\": \"charlie@example.com\"}");

            Thread.sleep(100);

            // Assert - should see neither DEBUG nor INFO logs
            debugCount = capturedLogs.stream()
                    .filter(log -> log.level == LogLevel.DEBUG)
                    .count();
            infoCount = capturedLogs.stream()
                    .filter(log -> log.level == LogLevel.INFO)
                    .count();

            System.out.println("At ERROR level - DEBUG logs: " + debugCount + ", INFO logs: " + infoCount);
            assertEquals(0, debugCount, "Should not see DEBUG logs when level is ERROR");
            assertEquals(0, infoCount, "Should not see INFO logs when level is ERROR");

            // Reset to INFO level for subsequent tests
            plugin.setLogLevel(LogLevel.INFO);
            Thread.sleep(100);

        } catch (InterruptedException e) {
            fail("Test interrupted");
        }
    }

    @Test
    @Order(2)
    @DisplayName("Log level changes affect subsequent calls immediately")
    void testLogLevelChangesAreImmediate() throws PluginException {
        // Arrange
        AtomicInteger debugLogCount = new AtomicInteger(0);
        List<LogMessage> allLogs = new ArrayList<>();

        LogCallback callback = (level, target, message) -> {
            allLogs.add(new LogMessage(level, target, message));
            if (level == LogLevel.DEBUG) {
                debugLogCount.incrementAndGet();
            }
        };

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback)) {
            // Start with ERROR level (no DEBUG logs)
            plugin.setLogLevel(LogLevel.ERROR);
            Thread.sleep(100);
            debugLogCount.set(0);

            // Make a call - should not see DEBUG logs
            plugin.call("echo", "{\"message\": \"before\"}");
            Thread.sleep(100);

            int beforeChange = debugLogCount.get();
            assertEquals(0, beforeChange, "Should not see DEBUG logs at ERROR level");

            // Change to DEBUG level
            System.out.println("Changing to DEBUG level...");
            plugin.setLogLevel(LogLevel.DEBUG);
            Thread.sleep(100);

            // Clear previous logs to only see logs from the next call
            allLogs.clear();
            debugLogCount.set(0);

            // Make another call - should now see DEBUG logs
            System.out.println("Calling echo at DEBUG level...");
            String response = plugin.call("echo", "{\"message\": \"after\"}");
            System.out.println("Echo response: " + response);
            Thread.sleep(100);

            int afterChange = debugLogCount.get();
            System.out.println("Debug log count after change: " + afterChange);
            System.out.println("All logs captured after echo call:");
            for (LogMessage log : allLogs) {
                System.out.println("  " + log);
            }
            assertTrue(afterChange > 0, "Should see DEBUG logs immediately after changing to DEBUG level (got " + afterChange + ")");

        } catch (InterruptedException e) {
            fail("Test interrupted");
        }
    }

    /**
     * Helper class to capture log messages.
     */
    private static class LogMessage {
        final LogLevel level;
        final String target;
        final String message;

        LogMessage(LogLevel level, String target, String message) {
            this.level = level;
            this.target = target;
            this.message = message;
        }

        @Override
        public String toString() {
            return "[" + level + "] " + target + ": " + message;
        }
    }
}
