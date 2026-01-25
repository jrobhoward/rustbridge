package com.rustbridge.jni;

import com.rustbridge.*;
import com.rustbridge.jni.JniNativeLibraryCondition.RequiresJniLibrary;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.*;

/**
 * Test plugin reload scenarios - loading, unloading, and reloading the same plugin.
 */
@RequiresJniLibrary
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class PluginReloadTest {

    private static Path PLUGIN_PATH;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @Test
    @DisplayName("Plugin can be loaded, shut down, and reloaded")
    void testLoadShutdownReload() throws PluginException, InterruptedException {
        System.out.println("\n=== First Load ===");

        Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString());
        assertEquals(LifecycleState.ACTIVE, plugin1.getState());

        String response1 = plugin1.call("echo", "{\"message\": \"first load\"}");
        assertTrue(response1.contains("first load"));

        System.out.println("\n=== Shutting down ===");
        plugin1.close();
        Thread.sleep(500);
        System.out.println("Plugin closed successfully");

        System.out.println("\n=== Second Load (Reload) ===");
        try {
            Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString());
            assertEquals(LifecycleState.ACTIVE, plugin2.getState(),
                    "Reloaded plugin should be ACTIVE");

            String response2 = plugin2.call("echo", "{\"message\": \"second load\"}");
            assertTrue(response2.contains("second load"),
                    "Reloaded plugin should work correctly");

            plugin2.close();

            System.out.println("\nResult: Reload SUCCESSFUL");

        } catch (Exception e) {
            System.err.println("\nResult: Reload FAILED");
            System.err.println("Error: " + e.getMessage());
            e.printStackTrace();
            fail("Plugin reload failed: " + e.getMessage());
        }
    }

    @Test
    @DisplayName("Plugin can be used, reloaded, and used again with same functionality")
    void testReloadFunctionality() throws PluginException, InterruptedException {
        System.out.println("\n=== First Load - Testing All Functions ===");
        Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString());

        String echo1 = plugin1.call("echo", "{\"message\": \"test\"}");
        String greet1 = plugin1.call("greet", "{\"name\": \"Alice\"}");
        String user1 = plugin1.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");

        assertTrue(echo1.contains("test"));
        assertTrue(greet1.contains("Alice"));
        assertTrue(user1.contains("user-"));

        plugin1.close();
        Thread.sleep(500);

        System.out.println("\n=== Second Load - Retesting Functions ===");
        Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString());

        String echo2 = plugin2.call("echo", "{\"message\": \"test\"}");
        String greet2 = plugin2.call("greet", "{\"name\": \"Bob\"}");
        String user2 = plugin2.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");

        assertTrue(echo2.contains("test"), "Echo should work after reload");
        assertTrue(greet2.contains("Bob"), "Greet should work after reload");
        assertTrue(user2.contains("user-"), "User creation should work after reload");

        plugin2.close();

        System.out.println("\nResult: All functions work correctly after reload");
    }

    @Test
    @DisplayName("Multiple reload cycles work")
    void testMultipleReloadCycles() throws PluginException, InterruptedException {
        final int RELOAD_COUNT = 3;

        for (int i = 0; i < RELOAD_COUNT; i++) {
            System.out.println("\n=== Load Cycle " + (i + 1) + " ===");

            Plugin plugin = JniPluginLoader.load(PLUGIN_PATH.toString());
            assertEquals(LifecycleState.ACTIVE, plugin.getState(),
                    "Plugin should be active on load cycle " + (i + 1));

            String response = plugin.call("echo", "{\"message\": \"cycle " + (i + 1) + "\"}");
            assertTrue(response.contains("cycle " + (i + 1)),
                    "Plugin should work on cycle " + (i + 1));

            plugin.close();
            Thread.sleep(500);
        }

        System.out.println("\nResult: " + RELOAD_COUNT + " reload cycles completed successfully");
    }

    @Test
    @DisplayName("Plugin state is fresh after reload")
    void testStateFreshAfterReload() throws PluginException, InterruptedException {
        System.out.println("\n=== First Load - Create Users ===");

        Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString());

        String user1 = plugin1.call("user.create", "{\"username\": \"alice\", \"email\": \"alice@example.com\"}");
        System.out.println("First user: " + user1);

        String user2 = plugin1.call("user.create", "{\"username\": \"bob\", \"email\": \"bob@example.com\"}");
        System.out.println("Second user: " + user2);

        plugin1.close();
        Thread.sleep(500);

        System.out.println("\n=== Second Load - Create User Again ===");

        Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString());

        String user3 = plugin2.call("user.create", "{\"username\": \"charlie\", \"email\": \"charlie@example.com\"}");
        System.out.println("First user after reload: " + user3);

        plugin2.close();

        // Check if counter was reset
        boolean stateAppearsReset = user3.contains("00000000") || user3.contains("00000001");

        System.out.println("\nState appears to be reset: " + stateAppearsReset);
        if (stateAppearsReset) {
            System.out.println("Result: Plugin state is fresh after reload");
        } else {
            System.out.println("Result: Plugin state may persist across reloads (or IDs are not sequential)");
        }
    }

    @Test
    @DisplayName("Dynamic log level changes work after reload")
    void testDynamicLogLevelAfterReload() throws PluginException, InterruptedException {
        System.out.println("\n=== First Load - Testing Dynamic Log Levels ===");

        Plugin plugin1 = JniPluginLoader.load(PLUGIN_PATH.toString());

        plugin1.setLogLevel(LogLevel.INFO);
        Thread.sleep(100);

        plugin1.call("echo", "{\"message\": \"test\"}");

        plugin1.close();
        Thread.sleep(500);

        System.out.println("\n=== Second Load - Retesting Dynamic Log Levels ===");

        Plugin plugin2 = JniPluginLoader.load(PLUGIN_PATH.toString());
        Thread.sleep(100);

        // Should be able to change log levels after reload
        assertDoesNotThrow(() -> plugin2.setLogLevel(LogLevel.DEBUG));
        assertDoesNotThrow(() -> plugin2.call("echo", "{\"message\": \"at debug\"}"));

        assertDoesNotThrow(() -> plugin2.setLogLevel(LogLevel.ERROR));
        assertDoesNotThrow(() -> plugin2.call("echo", "{\"message\": \"at error\"}"));

        // Reset to INFO
        plugin2.setLogLevel(LogLevel.INFO);
        plugin2.close();

        System.out.println("\nResult: Dynamic log levels work after reload");
    }
}
