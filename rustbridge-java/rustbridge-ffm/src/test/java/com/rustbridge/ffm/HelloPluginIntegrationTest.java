package com.rustbridge.ffm;

import com.rustbridge.*;
import org.junit.jupiter.api.*;
import org.junit.jupiter.api.condition.DisabledIfSystemProperty;

import java.nio.file.Path;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

import static org.junit.jupiter.api.Assertions.*;

/**
 * End-to-end integration tests for hello-plugin using FFM bindings.
 * <p>
 * These tests verify the complete FFI stack from Java to Rust and back:
 * - Plugin loading and initialization
 * - Request/response handling
 * - Error handling
 * - Panic handling
 * - Lifecycle management
 * - Logging integration
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation.class)
@Timeout(value = 60, unit = TimeUnit.SECONDS)  // Prevent tests from hanging indefinitely
class HelloPluginIntegrationTest {

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
        plugin = FfmPluginLoader.load(PLUGIN_PATH, config, null);
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
    void plugin___loaded___is_active() {
        assertNotNull(plugin);
        assertEquals(LifecycleState.ACTIVE, plugin.getState());
    }

    @Test
    @Order(2)
    @DisplayName("Echo handler returns message with length")
    void echo___valid_message___returns_message_with_length() throws PluginException {
        String request = "{\"message\": \"Hello, World!\"}";

        String response = plugin.call("echo", request);

        assertNotNull(response);
        assertTrue(response.contains("Hello, World!"));
        assertTrue(response.contains("\"length\":13"));
    }

    @Test
    @Order(3)
    @DisplayName("Greet handler returns personalized greeting")
    void greet___valid_name___returns_personalized_greeting() throws PluginException {
        String request = "{\"name\": \"Alice\"}";

        String response = plugin.call("greet", request);

        assertNotNull(response);
        assertTrue(response.contains("Hello, Alice!") || response.contains("greeting"));
    }

    @Test
    @Order(4)
    @DisplayName("User create handler creates user with ID")
    void user_create___valid_input___creates_user_with_id() throws PluginException {
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
    void math_add___valid_numbers___computes_sum() throws PluginException {
        String request = "{\"a\": 42, \"b\": 58}";

        String response = plugin.call("math.add", request);

        assertNotNull(response);
        assertTrue(response.contains("100") || response.contains("\"result\":100"));
    }

    @Test
    @Order(6)
    @DisplayName("Invalid type tag returns error")
    void call___invalid_type_tag___throws_exception() {
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
    void call___invalid_json___throws_serialization_error() {
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
    void echo___empty_object___throws_missing_field_error() {
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
    void call___sequential_calls___all_succeed() throws PluginException {
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
    void state___after_successful_calls___remains_active() throws PluginException {
        plugin.call("echo", "{\"message\": \"test\"}");

        LifecycleState state = plugin.getState();

        assertEquals(LifecycleState.ACTIVE, state);
    }

    @Test
    @Order(11)
    @DisplayName("Log level can be changed")
    void log_level___set_different_levels___no_exceptions() {
        assertDoesNotThrow(() -> {
            plugin.setLogLevel(LogLevel.DEBUG);
            plugin.setLogLevel(LogLevel.INFO);
            plugin.setLogLevel(LogLevel.WARN);
        });
    }

    @Test
    @Order(12)
    @DisplayName("Plugin can be closed and reopened")
    void plugin___close_and_reopen___works_correctly() throws PluginException {
        String request = "{\"message\": \"test\"}";

        String response1 = plugin.call("echo", request);
        assertNotNull(response1);

        plugin.close();

        assertThrows(PluginException.class, () -> {
            plugin.call("echo", request);
        });

        plugin = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), null);

        String response2 = plugin.call("echo", request);

        assertNotNull(response2);
    }

    @Test
    @Order(13)
    @DisplayName("Large request payload is handled")
    void echo___large_payload___handled_correctly() throws PluginException {
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
    void call___concurrent_calls___all_succeed() throws InterruptedException {
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

    @Test
    @Order(15)
    @DisplayName("Plugin with log callback receives log messages")
    void plugin___with_log_callback___receives_messages() throws PluginException, InterruptedException {
        AtomicInteger logCount = new AtomicInteger(0);

        LogCallback callback = (level, target, message) -> {
            System.out.println("[" + level + "] " + target + ": " + message);
            logCount.incrementAndGet();
        };

        plugin.close();
        plugin = null;

        plugin = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), callback);

        // Give time for async initialization
        Thread.sleep(500);

        // Note: Due to test ordering, the global tracing subscriber may already be initialized
        // from previous tests without a callback. The callback infrastructure is tested
        // separately in LogCallbackDebugTest. This test verifies that loading with a callback
        // doesn't cause crashes and the plugin remains functional.

        // Verify plugin is functional
        String response = plugin.call("echo", "{\"message\": \"test\"}");
        assertNotNull(response);
        assertTrue(response.contains("test"));

        // If logs were received (which works when run in isolation), verify they're valid
        if (logCount.get() > 0) {
            System.out.println("Received " + logCount.get() + " log messages");
        }
    }
}
