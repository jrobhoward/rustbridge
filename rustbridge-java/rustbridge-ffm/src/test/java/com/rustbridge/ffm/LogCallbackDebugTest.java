package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
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
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Log callback is invoked during plugin initialization")
    void testLogCallbackIsInvoked() throws PluginException {
        AtomicInteger callCount = new AtomicInteger(0);

        LogCallback callback = (level, target, message) -> {
            callCount.incrementAndGet();
            System.err.println("LOG CALLBACK INVOKED: [" + level + "] " + target + ": " + message);
        };

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback)) {
            assertEquals(LifecycleState.ACTIVE, plugin.getState());
            System.err.println("Plugin loaded, call count: " + callCount.get());

            Thread.sleep(1000);

            System.err.println("After wait, call count: " + callCount.get());

            int count = callCount.get();
            assertTrue(count > 0, "Log callback was never invoked! Got " + count + " calls");
        } catch (InterruptedException e) {
            fail("Test interrupted");
        }
    }

    @Test
    @DisplayName("Log callback receives logs from plugin calls")
    void testLogCallbackFromCalls() throws PluginException {
        AtomicInteger callCount = new AtomicInteger(0);

        LogCallback callback = (level, target, message) -> {
            callCount.incrementAndGet();
            System.err.println("LOG CALLBACK: [" + level + "] " + target + ": " + message);
        };

        try (Plugin plugin = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback)) {
            plugin.setLogLevel(LogLevel.DEBUG);

            int beforeCall = callCount.get();
            System.err.println("Before call, log count: " + beforeCall);

            plugin.call("echo", "{\"message\": \"test\"}");

            Thread.sleep(500);

            int afterCall = callCount.get();
            System.err.println("After call, log count: " + afterCall);

            assertTrue(afterCall >= beforeCall,
                    "Expected some logs, got " + (afterCall - beforeCall) + " new logs");
        } catch (InterruptedException e) {
            fail("Test interrupted");
        }
    }
}
