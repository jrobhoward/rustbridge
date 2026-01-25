package com.rustbridge.jni;

import com.rustbridge.*;
import com.rustbridge.jni.JniNativeLibraryCondition.RequiresJniLibrary;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test loading multiple plugin instances in the same process.
 * This verifies whether plugins share global state or are properly isolated.
 */
@RequiresJniLibrary
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class MultiplePluginTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Two plugins can be loaded simultaneously")
    void testTwoPluginsSimultaneously() throws PluginException {
        try (Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString());
             Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString())) {

            String response1 = plugin1.call("echo", "{\"message\": \"from plugin 1\"}");
            String response2 = plugin2.call("echo", "{\"message\": \"from plugin 2\"}");

            assertTrue(response1.contains("from plugin 1"), "Plugin 1 should echo its message");
            assertTrue(response2.contains("from plugin 2"), "Plugin 2 should echo its message");

            assertEquals(LifecycleState.ACTIVE, plugin1.getState());
            assertEquals(LifecycleState.ACTIVE, plugin2.getState());
        }
    }

    @Test
    @DisplayName("Multiple plugins have independent log levels")
    void testMultiplePluginsIndependentLogLevels() throws PluginException {
        try (Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString());
             Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString())) {

            // Set different log levels
            plugin1.setLogLevel(LogLevel.DEBUG);
            plugin2.setLogLevel(LogLevel.ERROR);

            // Both should still work
            String response1 = plugin1.call("echo", "{\"message\": \"test1\"}");
            String response2 = plugin2.call("echo", "{\"message\": \"test2\"}");

            assertNotNull(response1);
            assertNotNull(response2);
        }
    }

    @Test
    @DisplayName("Shutting down one plugin doesn't affect another")
    void testShutdownOnePluginDoesntAffectOther() throws PluginException, InterruptedException {
        Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString());
        Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString());

        try {
            assertDoesNotThrow(() -> plugin1.call("echo", "{\"message\": \"test1\"}"));
            assertDoesNotThrow(() -> plugin2.call("echo", "{\"message\": \"test2\"}"));

            plugin1.close();
            Thread.sleep(100);

            assertEquals(LifecycleState.ACTIVE, plugin2.getState(),
                    "Plugin2 should still be active");
            assertDoesNotThrow(() -> plugin2.call("echo", "{\"message\": \"test3\"}"),
                    "Plugin2 should still work after plugin1 shutdown");

        } finally {
            try {
                plugin2.close();
            } catch (Exception e) {
                // Ignore
            }
        }
    }
}
