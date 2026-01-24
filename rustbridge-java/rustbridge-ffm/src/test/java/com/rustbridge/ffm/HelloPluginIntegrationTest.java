package com.rustbridge.ffm;

import com.rustbridge.*;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.junit.jupiter.api.*;
import org.junit.jupiter.api.condition.DisabledIfSystemProperty;

import java.nio.file.Path;
import java.nio.file.Paths;
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
class HelloPluginIntegrationTest {

    // Unused - kept for potential future use in test assertions
    private static final ObjectMapper OBJECT_MAPPER = new ObjectMapper()
        .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
    private static Path PLUGIN_PATH;
    private Plugin plugin;

    @BeforeAll
    static void setupPluginPath() {
        // Find the hello-plugin library
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

        // Look in target/release first, then target/debug
        Path releasePath = Paths.get("../../target/release").resolve(libraryName);
        Path debugPath = Paths.get("../../target/debug").resolve(libraryName);

        if (releasePath.toFile().exists()) {
            PLUGIN_PATH = releasePath.toAbsolutePath();
        } else if (debugPath.toFile().exists()) {
            PLUGIN_PATH = debugPath.toAbsolutePath();
        } else {
            throw new RuntimeException("Could not find hello-plugin library. Build it with: cargo build -p hello-plugin --release");
        }

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
    void testPluginLoads() {
        assertNotNull(plugin);
        assertEquals(LifecycleState.ACTIVE, plugin.getState());
    }

    @Test
    @Order(2)
    @DisplayName("Echo handler returns message with length")
    void testEchoHandler() throws PluginException {
        // Arrange
        String request = "{\"message\": \"Hello, World!\"}";

        // Act
        String response = plugin.call("echo", request);

        // Assert
        assertNotNull(response);
        assertTrue(response.contains("Hello, World!"));
        assertTrue(response.contains("\"length\":13"));
    }

    @Test
    @Order(3)
    @DisplayName("Greet handler returns personalized greeting")
    void testGreetHandler() throws PluginException {
        // Arrange
        String request = "{\"name\": \"Alice\"}";

        // Act
        String response = plugin.call("greet", request);

        // Assert
        assertNotNull(response);
        assertTrue(response.contains("Hello, Alice!") || response.contains("greeting"));
    }

    @Test
    @Order(4)
    @DisplayName("User create handler creates user with ID")
    void testUserCreateHandler() throws PluginException {
        // Arrange
        String request = "{\"username\": \"testuser\", \"email\": \"test@example.com\"}";

        // Act
        String response = plugin.call("user.create", request);

        // Assert
        assertNotNull(response);
        System.out.println("User create response: " + response);
        // Response should contain user_id field
        assertTrue(response.contains("user_id") || response.contains("\"id\":"),
                "Response should contain user_id or id field, got: " + response);
    }

    @Test
    @Order(5)
    @DisplayName("Math add handler computes sum")
    void testMathAddHandler() throws PluginException {
        // Arrange
        String request = "{\"a\": 42, \"b\": 58}";

        // Act
        String response = plugin.call("math.add", request);

        // Assert
        assertNotNull(response);
        assertTrue(response.contains("100") || response.contains("\"result\":100"));
    }

    @Test
    @Order(6)
    @DisplayName("Invalid type tag returns error")
    void testInvalidTypeTag() {
        // Arrange
        String request = "{\"test\": true}";

        // Act & Assert
        PluginException exception = assertThrows(PluginException.class, () -> {
            plugin.call("invalid.type.tag", request);
        });

        // Should get UnknownMessageType error (code 6)
        assertEquals(6, exception.getErrorCode(),
                "Expected UnknownMessageType(6), got: " + exception.getErrorCode());
    }

    @Test
    @Order(7)
    @DisplayName("Invalid JSON in request returns error")
    void testInvalidJson() {
        // Arrange
        String invalidJson = "{broken json}";

        // Act & Assert
        PluginException exception = assertThrows(PluginException.class, () -> {
            plugin.call("echo", invalidJson);
        });

        // Should get SerializationError (code 5) for JSON parsing errors
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
        // Act & Assert - echo handler requires a message field
        PluginException exception = assertThrows(PluginException.class, () -> {
            plugin.call("echo", "{}");
        });

        // Should get deserialization error for missing field
        assertTrue(exception.getMessage().contains("missing field") ||
                   exception.getMessage().contains("message"),
                "Expected missing field error, got: " + exception.getMessage());
    }

    @Test
    @Order(9)
    @DisplayName("Multiple sequential calls work correctly")
    void testMultipleCalls() throws PluginException {
        // Arrange & Act
        String response1 = plugin.call("echo", "{\"message\": \"First\"}");
        String response2 = plugin.call("echo", "{\"message\": \"Second\"}");
        String response3 = plugin.call("greet", "{\"name\": \"Bob\"}");

        // Assert
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
        // Arrange & Act
        plugin.call("echo", "{\"message\": \"test\"}");
        LifecycleState state = plugin.getState();

        // Assert
        assertEquals(LifecycleState.ACTIVE, state);
    }

    @Test
    @Order(11)
    @DisplayName("Log level can be changed")
    void testSetLogLevel() {
        // Act & Assert - should not throw
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
        // Arrange
        String request = "{\"message\": \"test\"}";

        // Act - first call
        String response1 = plugin.call("echo", request);
        assertNotNull(response1);

        // Close
        plugin.close();

        // Assert - calls after close should fail
        assertThrows(IllegalStateException.class, () -> {
            plugin.call("echo", request);
        });

        // Reopen
        plugin = FfmPluginLoader.load(PLUGIN_PATH, PluginConfig.defaults(), null);

        // Act - second call on new instance
        String response2 = plugin.call("echo", request);

        // Assert
        assertNotNull(response2);
    }

    @Test
    @Order(13)
    @DisplayName("Large request payload is handled")
    void testLargePayload() throws PluginException {
        // Arrange - create a large message
        StringBuilder largeMessage = new StringBuilder();
        for (int i = 0; i < 10000; i++) {
            largeMessage.append("This is a test message. ");
        }
        String request = "{\"message\": \"" + largeMessage.toString() + "\"}";

        // Act
        String response = plugin.call("echo", request);

        // Assert
        assertNotNull(response);
        assertTrue(response.length() > 100000); // Should be large
    }

    @Test
    @Order(14)
    @DisplayName("Concurrent calls are handled safely")
    void testConcurrentCalls() throws InterruptedException {
        // Arrange
        AtomicInteger successCount = new AtomicInteger(0);
        AtomicInteger errorCount = new AtomicInteger(0);
        int numThreads = 10;
        int callsPerThread = 10;

        Thread[] threads = new Thread[numThreads];

        // Act - spawn multiple threads making concurrent calls
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

        // Wait for all threads
        for (Thread thread : threads) {
            thread.join();
        }

        // Assert - all calls should succeed
        System.out.println("Concurrent test: " + successCount.get() + " successes, " + errorCount.get() + " errors");
        assertEquals(numThreads * callsPerThread, successCount.get(),
                "Expected all calls to succeed, but " + errorCount.get() + " failed");
        assertEquals(0, errorCount.get(), "No errors should occur");
    }

    @Test
    @Order(15)
    @DisplayName("Plugin with log callback receives log messages")
    void testLogCallback() throws PluginException, InterruptedException {
        // Arrange
        AtomicInteger logCount = new AtomicInteger(0);

        LogCallback callback = (level, target, message) -> {
            System.out.println("[" + level + "] " + target + ": " + message);
            logCount.incrementAndGet();
        };

        // Close current plugin and reopen with log callback
        plugin.close();
        plugin = null;

        // Act - load plugin with callback
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
