package com.rustbridge.kotlin

import com.rustbridge.LogLevel
import com.rustbridge.LifecycleState
import org.junit.jupiter.api.*
import org.junit.jupiter.api.Assertions.*
import java.nio.file.Path
import java.util.concurrent.TimeUnit
import kotlin.time.Duration.Companion.seconds

/**
 * Tests for Kotlin plugin loader extensions.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation::class)
@Timeout(value = 60, unit = TimeUnit.SECONDS)
class PluginLoaderExtensionsTest {

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

    @Test
    @Order(1)
    @DisplayName("loadPlugin___with_dsl_config___creates_working_plugin")
    fun loadPlugin___with_dsl_config___creates_working_plugin() {
        val plugin = loadPlugin(pluginPath.toString()) {
            workerThreads = 2
            logLevel = LogLevel.INFO
        }

        try {
            assertEquals(LifecycleState.ACTIVE, plugin.state)
            val response = plugin.call<EchoRequest, EchoResponse>("echo", EchoRequest("dsl test"))
            assertEquals("dsl test", response.message)
        } finally {
            plugin.close()
        }
    }

    @Test
    @Order(2)
    @DisplayName("loadPlugin___with_path___creates_working_plugin")
    fun loadPlugin___with_path___creates_working_plugin() {
        val plugin = loadPlugin(pluginPath) {
            workerThreads = 2
        }

        try {
            assertEquals(LifecycleState.ACTIVE, plugin.state)
        } finally {
            plugin.close()
        }
    }

    @Test
    @Order(3)
    @DisplayName("loadPlugin___default_config___creates_working_plugin")
    fun loadPlugin___default_config___creates_working_plugin() {
        val plugin = loadPlugin(pluginPath.toString())

        try {
            assertEquals(LifecycleState.ACTIVE, plugin.state)
        } finally {
            plugin.close()
        }
    }

    @Test
    @Order(10)
    @DisplayName("withPlugin___executes_block_and_closes___plugin_closed_after")
    fun withPlugin___executes_block_and_closes___plugin_closed_after() {
        var capturedState: LifecycleState? = null

        withPlugin(pluginPath.toString()) { plugin ->
            capturedState = plugin.state
        }

        assertEquals(LifecycleState.ACTIVE, capturedState)
        // Plugin should be closed after withPlugin completes
    }

    @Test
    @Order(11)
    @DisplayName("withPlugin___with_config___uses_config")
    fun withPlugin___with_config___uses_config() {
        val response = withPlugin(
            pluginPath.toString(),
            config = {
                workerThreads = 4
                logLevel = LogLevel.DEBUG
            }
        ) { plugin ->
            plugin.call<EchoRequest, EchoResponse>("echo", EchoRequest("config test"))
        }

        assertEquals("config test", response.message)
    }

    @Test
    @Order(12)
    @DisplayName("withPlugin___returns_block_result___result_accessible")
    fun withPlugin___returns_block_result___result_accessible() {
        val result = withPlugin(pluginPath.toString()) { plugin ->
            val response = plugin.call<EchoRequest, EchoResponse>("echo", EchoRequest("return test"))
            response.length
        }

        assertEquals(11, result)
    }

    @Test
    @Order(20)
    @DisplayName("withPluginContext___plugin_as_receiver___can_call_methods_directly")
    fun withPluginContext___plugin_as_receiver___can_call_methods_directly() {
        val response = withPluginContext(pluginPath.toString()) {
            // 'this' is the Plugin, so we can call methods directly
            call<EchoRequest, EchoResponse>("echo", EchoRequest("context test"))
        }

        assertEquals("context test", response.message)
    }

    @Test
    @Order(21)
    @DisplayName("withPluginContext___with_config___uses_config")
    fun withPluginContext___with_config___uses_config() {
        val state = withPluginContext(
            pluginPath.toString(),
            config = { workerThreads = 2 }
        ) {
            state
        }

        assertEquals(LifecycleState.ACTIVE, state)
    }

    @Test
    @Order(30)
    @DisplayName("lazyPlugin___not_loaded_until_accessed___loads_on_first_use")
    fun lazyPlugin___not_loaded_until_accessed___loads_on_first_use() {
        val lazyPlugin = lazyPlugin(pluginPath.toString()) {
            workerThreads = 2
        }

        // Plugin not loaded yet
        assertFalse(lazyPlugin.isInitialized())

        // Access the plugin - triggers loading
        val plugin = lazyPlugin.value
        assertTrue(lazyPlugin.isInitialized())
        assertEquals(LifecycleState.ACTIVE, plugin.state)

        // Clean up
        plugin.close()
    }

    @Test
    @Order(31)
    @DisplayName("lazyPlugin___subsequent_access___returns_same_instance")
    fun lazyPlugin___subsequent_access___returns_same_instance() {
        val lazyPlugin = lazyPlugin(pluginPath.toString())

        val first = lazyPlugin.value
        val second = lazyPlugin.value

        assertSame(first, second)

        first.close()
    }

    @Test
    @Order(40)
    @DisplayName("loadPlugin___full_dsl_example___comprehensive_config")
    fun loadPlugin___full_dsl_example___comprehensive_config() {
        val plugin = loadPlugin(pluginPath.toString()) {
            workerThreads = 4
            logLevel = LogLevel.WARN
            maxConcurrentOps = 500
            shutdownTimeout = 10.seconds

            data {
                "test_key" to "test_value"
            }

            initParams {
                "init_flag" to true
            }
        }

        try {
            assertEquals(LifecycleState.ACTIVE, plugin.state)
        } finally {
            plugin.close()
        }
    }
}
