package com.rustbridge.jni;

import com.rustbridge.*;
import com.rustbridge.jni.JniNativeLibraryCondition.RequiresJniLibrary;
import org.junit.jupiter.api.*;

import java.nio.file.Path;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * End-to-end integration tests for hello-plugin using JNI bindings.
 * <p>
 * These tests verify the complete FFI stack from Java to Rust and back:
 * - Plugin loading and initialization
 * - Request/response handling
 * - Error handling
 * - Lifecycle management
 */
@RequiresJniLibrary
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class JniPluginIntegrationTest {

    private static Path PLUGIN_PATH;
    private Plugin plugin;

    @BeforeAll
    static void setupPluginPath() {
        PLUGIN_PATH = TestPluginLoader.findHelloPluginLibrary();
        System.out.println("Using plugin: " + PLUGIN_PATH);
    }

    @BeforeEach
    void loadPlugin() throws PluginException {
        PluginConfig config = PluginConfig.defaults()
                .workerThreads(2);
        plugin = JniPluginLoader.load(PLUGIN_PATH.toString(), config);
    }

    @AfterEach
    void closePlugin() {
        if (plugin != null) {
            plugin.close();
            plugin = null;
        }
    }

    @Test
    @Order(1)
    @DisplayName("Plugin loads and initializes successfully")
    void testPluginLoads() {
        assertNotNull(plugin);
        assertEquals(LifecycleState.ACTIVE, plugin.getState());
    }

    @Test
    @Order(2)
    @DisplayName("Echo handler returns message with length")
    void testEchoHandler() throws PluginException {
        String request = "{\"message\": \"Hello, World!\"}";

        String response = plugin.call("echo", request);

        assertNotNull(response);
        assertTrue(response.contains("Hello, World!"));
        assertTrue(response.contains("\"length\":13"));
    }

    @Test
    @Order(3)
    @DisplayName("Greet handler returns personalized greeting")
    void testGreetHandler() throws PluginException {
        String request = "{\"name\": \"Alice\"}";

        String response = plugin.call("greet", request);

        assertNotNull(response);
        assertTrue(response.contains("Hello, Alice!") || response.contains("greeting"));
    }

    @Test
    @Order(4)
    @DisplayName("User create handler creates user with ID")
    void testUserCreateHandler() throws PluginException {
        String request = "{\"username\": \"testuser\", \"email\": \"test@example.com\"}";

        String response = plugin.call("user.create", request);

        assertNotNull(response);
        System.out.println("User create response: " + response);
        assertTrue(response.contains("user_id") || response.contains("\"id\":"),
                "Response should contain user_id or id field, got: " + response);
    }

    @Test
    @Order(5)
    @DisplayName("Math add handler computes sum")
    void testMathAddHandler() throws PluginException {
        String request = "{\"a\": 42, \"b\": 58}";

        String response = plugin.call("math.add", request);

        assertNotNull(response);
        assertTrue(response.contains("100") || response.contains("\"result\":100"));
    }

    @Test
    @Order(6)
    @DisplayName("Invalid type tag returns error")
    void testInvalidTypeTag() {
        String request = "{\"test\": true}";

        PluginException exception = assertThrows(PluginException.class, () -> {
            plugin.call("invalid.type.tag", request);
        });

        assertEquals(6, exception.getErrorCode(),
                "Expected UnknownMessageType(6), got: " + exception.getErrorCode());
    }

    @Test
    @Order(7)
    @DisplayName("Invalid JSON in request returns error")
    void testInvalidJson() {
        String invalidJson = "{broken json}";

        PluginException exception = assertThrows(PluginException.class, () -> {
            plugin.call("echo", invalidJson);
        });

        assertEquals(5, exception.getErrorCode(),
                "Expected SerializationError(5), got: " + exception.getErrorCode());
        assertTrue(exception.getMessage().contains("serialization") ||
                        exception.getMessage().contains("key must be a string"),
                "Expected JSON error message, got: " + exception.getMessage());
    }

    @Test
    @Order(8)
    @DisplayName("Empty message field in echo returns error")
    void testEmptyRequest() {
        PluginException exception = assertThrows(PluginException.class, () -> {
            plugin.call("echo", "{}");
        });

        assertTrue(exception.getMessage().contains("missing field") ||
                        exception.getMessage().contains("message"),
                "Expected missing field error, got: " + exception.getMessage());
    }

    @Test
    @Order(9)
    @DisplayName("Multiple sequential calls work correctly")
    void testMultipleCalls() throws PluginException {
        String response1 = plugin.call("echo", "{\"message\": \"First\"}");
        String response2 = plugin.call("echo", "{\"message\": \"Second\"}");
        String response3 = plugin.call("greet", "{\"name\": \"Bob\"}");

        assertNotNull(response1);
        assertNotNull(response2);
        assertNotNull(response3);
        assertTrue(response1.contains("First"));
        assertTrue(response2.contains("Second"));
        assertTrue(response3.contains("Bob"));
    }

    @Test
    @Order(10)
    @DisplayName("Plugin state remains ACTIVE after successful calls")
    void testStateRemainsActive() throws PluginException {
        plugin.call("echo", "{\"message\": \"test\"}");
        LifecycleState state = plugin.getState();

        assertEquals(LifecycleState.ACTIVE, state);
    }

    @Test
    @Order(11)
    @DisplayName("Log level can be changed")
    void testSetLogLevel() {
        assertDoesNotThrow(() -> {
            plugin.setLogLevel(LogLevel.DEBUG);
            plugin.setLogLevel(LogLevel.INFO);
            plugin.setLogLevel(LogLevel.WARN);
        });
    }

    @Test
    @Order(12)
    @DisplayName("Plugin can be closed and reopened")
    void testPluginCloseAndReopen() throws PluginException {
        String request = "{\"message\": \"test\"}";

        String response1 = plugin.call("echo", request);
        assertNotNull(response1);

        plugin.close();

        assertThrows(IllegalStateException.class, () -> {
            plugin.call("echo", request);
        });

        plugin = JniPluginLoader.load(PLUGIN_PATH.toString());

        String response2 = plugin.call("echo", request);

        assertNotNull(response2);
    }

    @Test
    @Order(13)
    @DisplayName("Large request payload is handled")
    void testLargePayload() throws PluginException {
        StringBuilder largeMessage = new StringBuilder();
        for (int i = 0; i < 10000; i++) {
            largeMessage.append("This is a test message. ");
        }
        String request = "{\"message\": \"" + largeMessage.toString() + "\"}";

        String response = plugin.call("echo", request);

        assertNotNull(response);
        assertTrue(response.length() > 100000);
    }

    @Test
    @Order(14)
    @DisplayName("Concurrent calls are handled safely")
    void testConcurrentCalls() throws InterruptedException {
        AtomicInteger successCount = new AtomicInteger(0);
        AtomicInteger errorCount = new AtomicInteger(0);
        int numThreads = 10;
        int callsPerThread = 10;

        Thread[] threads = new Thread[numThreads];

        for (int i = 0; i < numThreads; i++) {
            final int threadId = i;
            threads[i] = new Thread(() -> {
                for (int j = 0; j < callsPerThread; j++) {
                    try {
                        String request = "{\"message\": \"Thread-" + threadId + "-" + j + "\"}";
                        String response = plugin.call("echo", request);
                        if (response != null) {
                            successCount.incrementAndGet();
                        }
                    } catch (Exception e) {
                        errorCount.incrementAndGet();
                        System.err.println("Error in thread " + threadId + ": " + e.getMessage());
                    }
                }
            });
            threads[i].start();
        }

        for (Thread thread : threads) {
            thread.join();
        }

        System.out.println("Concurrent test: " + successCount.get() + " successes, " + errorCount.get() + " errors");
        assertEquals(numThreads * callsPerThread, successCount.get(),
                "Expected all calls to succeed, but " + errorCount.get() + " failed");
        assertEquals(0, errorCount.get(), "No errors should occur");
    }
}
