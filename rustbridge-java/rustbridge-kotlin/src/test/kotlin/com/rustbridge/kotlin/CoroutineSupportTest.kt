package com.rustbridge.kotlin

import com.rustbridge.PluginConfig
import com.rustbridge.ffm.FfmPluginLoader
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.test.runTest
import org.junit.jupiter.api.*
import org.junit.jupiter.api.Assertions.*
import java.nio.file.Path
import java.util.concurrent.TimeUnit

/**
 * Tests for Kotlin coroutine support extensions.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation::class)
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class CoroutineSupportTest {

    companion object {
        private lateinit var pluginPath: Path

        @JvmStatic
        @BeforeAll
        fun setupPluginPath() {
            pluginPath = TestPluginLoader.findHelloPluginLibrary()
            println("Using plugin: $pluginPath")
        }
    }

    data class EchoRequest(val message: String)
    data class EchoResponse(val message: String, val length: Int)

    private lateinit var plugin: com.rustbridge.Plugin

    @BeforeEach
    fun loadPlugin() {
        val config = PluginConfig.defaults().workerThreads(4)
        plugin = FfmPluginLoader.load(pluginPath, config, null)
    }

    @AfterEach
    fun closePlugin() {
        plugin.close()
    }

    // ==================== callAsync() tests ====================

    @Test
    @Order(1)
    @DisplayName("callAsync___typed_request___returns_response")
    fun callAsync___typed_request___returns_response() = runTest {
        val request = EchoRequest("async test")

        val response = plugin.callAsync<EchoRequest, EchoResponse>("echo", request)

        assertEquals("async test", response.message)
        assertEquals(10, response.length)
    }

    @Test
    @Order(2)
    @DisplayName("callAsync___json_request___returns_typed_response")
    fun callAsync___json_request___returns_typed_response() = runTest {
        val response = plugin.callAsync<EchoResponse>("echo", """{"message": "json async"}""")

        assertEquals("json async", response.message)
    }

    @Test
    @Order(3)
    @DisplayName("callAsyncJson___returns_raw_json")
    fun callAsyncJson___returns_raw_json() = runTest {
        val responseJson = plugin.callAsyncJson("echo", """{"message": "raw async"}""")

        assertTrue(responseJson.contains("raw async"))
        assertTrue(responseJson.contains("\"length\""))
    }

    // ==================== callAsyncResult() tests ====================

    @Test
    @Order(10)
    @DisplayName("callAsyncResult___success___returns_success")
    fun callAsyncResult___success___returns_success() = runTest {
        val result = plugin.callAsyncResult<EchoRequest, EchoResponse>("echo", EchoRequest("result"))

        assertTrue(result.isSuccess)
        assertEquals("result", result.getOrNull()?.message)
    }

    @Test
    @Order(11)
    @DisplayName("callAsyncResult___failure___returns_failure")
    fun callAsyncResult___failure___returns_failure() = runTest {
        val result = plugin.callAsyncResult<EchoRequest, EchoResponse>("nonexistent", EchoRequest("test"))

        assertTrue(result.isFailure)
    }

    // ==================== callAsyncSafe() tests ====================

    @Test
    @Order(20)
    @DisplayName("callAsyncSafe___success___returns_success_pluginresult")
    fun callAsyncSafe___success___returns_success_pluginresult() = runTest {
        val result = plugin.callAsyncSafe<EchoRequest, EchoResponse>("echo", EchoRequest("safe async"))

        assertTrue(result.isSuccess)
        assertEquals("safe async", (result as PluginResult.Success).value.message)
    }

    @Test
    @Order(21)
    @DisplayName("callAsyncSafe___failure___returns_error_pluginresult")
    fun callAsyncSafe___failure___returns_error_pluginresult() = runTest {
        val result = plugin.callAsyncSafe<EchoRequest, EchoResponse>("nonexistent", EchoRequest("test"))

        assertTrue(result.isError)
    }

    // ==================== callAsyncOrNull() tests ====================

    @Test
    @Order(30)
    @DisplayName("callAsyncOrNull___success___returns_response")
    fun callAsyncOrNull___success___returns_response() = runTest {
        val response = plugin.callAsyncOrNull<EchoRequest, EchoResponse>("echo", EchoRequest("nullable async"))

        assertNotNull(response)
        assertEquals("nullable async", response?.message)
    }

    @Test
    @Order(31)
    @DisplayName("callAsyncOrNull___failure___returns_null")
    fun callAsyncOrNull___failure___returns_null() = runTest {
        val response = plugin.callAsyncOrNull<EchoRequest, EchoResponse>("nonexistent", EchoRequest("test"))

        assertNull(response)
    }

    // ==================== Concurrent async tests ====================

    @Test
    @Order(40)
    @DisplayName("callAsync___concurrent_calls___all_succeed")
    fun callAsync___concurrent_calls___all_succeed() = runTest {
        val requests = (1..10).map { EchoRequest("concurrent $it") }

        val responses = requests.map { request ->
            async { plugin.callAsync<EchoRequest, EchoResponse>("echo", request) }
        }.awaitAll()

        assertEquals(10, responses.size)
        responses.forEachIndexed { index, response ->
            assertEquals("concurrent ${index + 1}", response.message)
        }
    }

    @Test
    @Order(41)
    @DisplayName("callAllAsync___multiple_calls___returns_all_responses")
    fun callAllAsync___multiple_calls___returns_all_responses() = runTest {
        val responses = plugin.callAllAsync<EchoRequest, EchoResponse>(
            "echo" to EchoRequest("first"),
            "echo" to EchoRequest("second"),
            "echo" to EchoRequest("third")
        )

        assertEquals(3, responses.size)
        assertEquals("first", responses[0].message)
        assertEquals("second", responses[1].message)
        assertEquals("third", responses[2].message)
    }
}
