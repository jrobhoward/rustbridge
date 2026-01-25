package com.rustbridge.jni;

import com.rustbridge.*;
import com.rustbridge.jni.JniNativeLibraryCondition.RequiresJniLibrary;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test that log levels can be changed dynamically at runtime.
 * <p>
 * Note: JNI implementation does not yet support log callbacks, so these tests
 * verify that setLogLevel can be called without errors rather than verifying
 * actual log filtering behavior.
 */
@RequiresJniLibrary
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class DynamicLogLevelTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @Order(1)
    @DisplayName("Log level can be set to all valid levels")
    void log_level___set_all_levels___no_error() throws PluginException {
        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString())) {
            // Should be able to set all log levels without error
            assertDoesNotThrow(() -> plugin.setLogLevel(LogLevel.TRACE));
            assertDoesNotThrow(() -> plugin.setLogLevel(LogLevel.DEBUG));
            assertDoesNotThrow(() -> plugin.setLogLevel(LogLevel.INFO));
            assertDoesNotThrow(() -> plugin.setLogLevel(LogLevel.WARN));
            assertDoesNotThrow(() -> plugin.setLogLevel(LogLevel.ERROR));

            // Plugin should remain functional after log level changes
            String response = plugin.call("echo", "{\"message\": \"test\"}");
            assertNotNull(response);
            assertTrue(response.contains("test"));
        }
    }

    @Test
    @Order(2)
    @DisplayName("Log level changes don't affect plugin functionality")
    void log_level___change_during_use___plugin_remains_functional() throws PluginException {
        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString())) {
            // Make calls at different log levels
            plugin.setLogLevel(LogLevel.ERROR);
            String response1 = plugin.call("echo", "{\"message\": \"at error level\"}");
            assertTrue(response1.contains("at error level"));

            plugin.setLogLevel(LogLevel.DEBUG);
            String response2 = plugin.call("echo", "{\"message\": \"at debug level\"}");
            assertTrue(response2.contains("at debug level"));

            plugin.setLogLevel(LogLevel.TRACE);
            String response3 = plugin.call("echo", "{\"message\": \"at trace level\"}");
            assertTrue(response3.contains("at trace level"));

            // All calls should succeed regardless of log level
            assertEquals(LifecycleState.ACTIVE, plugin.getState());
        }
    }

    @Test
    @Order(3)
    @DisplayName("Log level can be changed rapidly")
    void log_level___rapid_changes___no_error() throws PluginException {
        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString())) {
            LogLevel[] levels = {
                    LogLevel.TRACE, LogLevel.DEBUG, LogLevel.INFO,
                    LogLevel.WARN, LogLevel.ERROR
            };

            // Rapidly cycle through log levels
            for (int i = 0; i < 100; i++) {
                plugin.setLogLevel(levels[i % levels.length]);
            }

            // Plugin should still work
            String response = plugin.call("greet", "{\"name\": \"Test\"}");
            assertNotNull(response);
        }
    }

    @Test
    @Order(4)
    @DisplayName("Log level can be set during concurrent calls")
    void log_level___concurrent_calls___no_error() throws PluginException, InterruptedException {
        try (Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString())) {
            Thread logLevelThread = new Thread(() -> {
                LogLevel[] levels = {LogLevel.DEBUG, LogLevel.INFO, LogLevel.WARN};
                for (int i = 0; i < 50; i++) {
                    plugin.setLogLevel(levels[i % levels.length]);
                    try {
                        Thread.sleep(10);
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        return;
                    }
                }
            });

            Thread callThread = new Thread(() -> {
                for (int i = 0; i < 50; i++) {
                    try {
                        plugin.call("echo", "{\"message\": \"concurrent " + i + "\"}");
                    } catch (PluginException e) {
                        // Ignore
                    }
                    try {
                        Thread.sleep(10);
                    } catch (InterruptedException e) {
                        Thread.currentThread().interrupt();
                        return;
                    }
                }
            });

            logLevelThread.start();
            callThread.start();

            logLevelThread.join();
            callThread.join();

            // Plugin should remain in ACTIVE state
            assertEquals(LifecycleState.ACTIVE, plugin.getState());
        }
    }
}
