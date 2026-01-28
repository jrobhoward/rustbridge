package com.rustbridge.kotlin

import com.rustbridge.PluginConfig
import com.rustbridge.ffm.FfmPluginLoader
import org.junit.jupiter.api.*
import org.junit.jupiter.api.Assertions.*
import java.nio.file.Path
import java.util.concurrent.TimeUnit

/**
 * Tests for Kotlin plugin extension functions.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation::class)
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class PluginExtensionsTest {

    companion object {
        private lateinit var pluginPath: Path

        @JvmStatic
        @BeforeAll
        fun setupPluginPath() {
            pluginPath = TestPluginLoader.findHelloPluginLibrary()
            println("Using plugin: $pluginPath")
        }
    }

    // Test data classes
    data class EchoRequest(val message: String)
    data class EchoResponse(val message: String, val length: Int)
    data class GreetRequest(val name: String)
    data class GreetResponse(val greeting: String)

    private lateinit var plugin: com.rustbridge.Plugin

    @BeforeEach
    fun loadPlugin() {
        val config = PluginConfig.defaults().workerThreads(2)
        plugin = FfmPluginLoader.load(pluginPath, config, null)
    }

    @AfterEach
    fun closePlugin() {
        plugin.close()
    }

    // ==================== call<T, R>() tests ====================

    @Test
    @Order(1)
    @DisplayName("call___typed_request_response___returns_deserialized_response")
    fun call___typed_request_response___returns_deserialized_response() {
        val request = EchoRequest("hello kotlin")

        val response = plugin.call<EchoRequest, EchoResponse>("echo", request)

        assertEquals("hello kotlin", response.message)
        assertEquals(12, response.length)
    }

    @Test
    @Order(2)
    @DisplayName("callAs___json_request_typed_response___returns_deserialized_response")
    fun callAs___json_request_typed_response___returns_deserialized_response() {
        val response = plugin.callAs<EchoResponse>("echo", """{"message": "json test"}""")

        assertEquals("json test", response.message)
        assertEquals(9, response.length)
    }

    @Test
    @Order(3)
    @DisplayName("callJson___typed_request___returns_json_string")
    fun callJson___typed_request___returns_json_string() {
        val request = EchoRequest("raw json")

        val responseJson = plugin.callJson("echo", request)

        assertTrue(responseJson.contains("raw json"))
        assertTrue(responseJson.contains("\"length\""))
    }

    // ==================== callResult() tests ====================

    @Test
    @Order(10)
    @DisplayName("callResult___success___returns_success_result")
    fun callResult___success___returns_success_result() {
        val request = EchoRequest("result test")

        val result = plugin.callResult<EchoRequest, EchoResponse>("echo", request)

        assertTrue(result.isSuccess)
        assertEquals("result test", result.getOrNull()?.message)
    }

    @Test
    @Order(11)
    @DisplayName("callResult___failure___returns_failure_result")
    fun callResult___failure___returns_failure_result() {
        val result = plugin.callResult<EchoRequest, EchoResponse>("nonexistent", EchoRequest("test"))

        assertTrue(result.isFailure)
        assertNotNull(result.exceptionOrNull())
    }

    // ==================== callSafe() tests ====================

    @Test
    @Order(20)
    @DisplayName("callSafe___success___returns_success_pluginresult")
    fun callSafe___success___returns_success_pluginresult() {
        val request = EchoRequest("safe test")

        val result = plugin.callSafe<EchoRequest, EchoResponse>("echo", request)

        assertTrue(result.isSuccess)
        assertFalse(result.isError)
        assertEquals("safe test", result.getOrNull()?.message)
    }

    @Test
    @Order(21)
    @DisplayName("callSafe___failure___returns_error_pluginresult")
    fun callSafe___failure___returns_error_pluginresult() {
        val result = plugin.callSafe<EchoRequest, EchoResponse>("nonexistent", EchoRequest("test"))

        assertTrue(result.isError)
        assertFalse(result.isSuccess)
        assertNull(result.getOrNull())

        val error = result as PluginResult.Error
        assertTrue(error.code > 0 || error.message.isNotEmpty())
    }

    @Test
    @Order(22)
    @DisplayName("callSafe___pattern_matching___exhaustive")
    fun callSafe___pattern_matching___exhaustive() {
        val result = plugin.callSafe<EchoRequest, EchoResponse>("echo", EchoRequest("match"))

        val message = when (result) {
            is PluginResult.Success -> result.value.message
            is PluginResult.Error -> "error: ${result.message}"
        }

        assertEquals("match", message)
    }

    // ==================== callOrNull() tests ====================

    @Test
    @Order(30)
    @DisplayName("callOrNull___success___returns_response")
    fun callOrNull___success___returns_response() {
        val response = plugin.callOrNull<EchoRequest, EchoResponse>("echo", EchoRequest("nullable"))

        assertNotNull(response)
        assertEquals("nullable", response?.message)
    }

    @Test
    @Order(31)
    @DisplayName("callOrNull___failure___returns_null")
    fun callOrNull___failure___returns_null() {
        val response = plugin.callOrNull<EchoRequest, EchoResponse>("nonexistent", EchoRequest("test"))

        assertNull(response)
    }

    // ==================== callOrDefault() tests ====================

    @Test
    @Order(40)
    @DisplayName("callOrDefault___success___returns_response")
    fun callOrDefault___success___returns_response() {
        val response = plugin.callOrDefault("echo", EchoRequest("default test")) {
            EchoResponse("fallback", 0)
        }

        assertEquals("default test", response.message)
    }

    @Test
    @Order(41)
    @DisplayName("callOrDefault___failure___returns_default")
    fun callOrDefault___failure___returns_default() {
        val response = plugin.callOrDefault("nonexistent", EchoRequest("test")) {
            EchoResponse("fallback", 0)
        }

        assertEquals("fallback", response.message)
        assertEquals(0, response.length)
    }

    // ==================== Multiple handler types ====================

    @Test
    @Order(50)
    @DisplayName("call___greet_handler___returns_greeting")
    fun call___greet_handler___returns_greeting() {
        val response = plugin.call<GreetRequest, GreetResponse>("greet", GreetRequest("Kotlin"))

        assertTrue(response.greeting.contains("Kotlin"))
    }
}
