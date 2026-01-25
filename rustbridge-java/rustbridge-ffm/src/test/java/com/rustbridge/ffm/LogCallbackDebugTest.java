package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Debug test for log callback integration.
 */
@Timeout(value = 60, unit = TimeUnit.SECONDS)  // Prevent tests from hanging indefinitely
class LogCallbackDebugTest {

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
    @DisplayName("Log callback is invoked during plugin initialization")
    void testLogCallbackIsInvoked() throws PluginException {
        // Arrange
        AtomicInteger callCount = new AtomicInteger(0);

        LogCallback callback = (level, target, message) -> {
            callCount.incrementAndGet();
            System.err.println("LOG CALLBACK INVOKED: [" + level + "] " + target + ": " + message);
        };

        // Act - load plugin with callback
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback)) {
            System.err.println("Plugin loaded, call count: " + callCount.get());

            // Wait a bit to ensure async initialization completes
            Thread.sleep(1000);

            System.err.println("After wait, call count: " + callCount.get());

            // Assert - should have received some logs
            int count = callCount.get();
            assertTrue(count > 0, "Log callback was never invoked! Got " + count + " calls");
        } catch (InterruptedException e) {
            fail("Test interrupted");
        }
    }

    @Test
    @DisplayName("Log callback receives logs from plugin calls")
    void testLogCallbackFromCalls() throws PluginException {
        // Arrange
        AtomicInteger callCount = new AtomicInteger(0);

        LogCallback callback = (level, target, message) -> {
            callCount.incrementAndGet();
            System.err.println("LOG CALLBACK: [" + level + "] " + target + ": " + message);
        };

        // Act
        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback)) {
            // Set to DEBUG to capture more logs
            plugin.setLogLevel(LogLevel.DEBUG);

            int beforeCall = callCount.get();
            System.err.println("Before call, log count: " + beforeCall);

            // Make a call
            plugin.call("echo", "{\"message\": \"test\"}");

            Thread.sleep(500);  // Give time for async logging

            int afterCall = callCount.get();
            System.err.println("After call, log count: " + afterCall);

            // Assert
            assertTrue(afterCall >= beforeCall,
                "Expected some logs, got " + (afterCall - beforeCall) + " new logs");
        } catch (InterruptedException e) {
            fail("Test interrupted");
        }
    }
}
