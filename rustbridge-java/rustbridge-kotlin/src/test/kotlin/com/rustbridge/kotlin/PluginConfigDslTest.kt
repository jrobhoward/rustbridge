package com.rustbridge.kotlin

import com.rustbridge.LogLevel
import org.junit.jupiter.api.*
import org.junit.jupiter.api.Assertions.*
import java.util.concurrent.TimeUnit
import kotlin.time.Duration.Companion.seconds

/**
 * Tests for Kotlin PluginConfig DSL.
 */
@TestMethodOrder(MethodOrderer.OrderAnnotation::class)
@Timeout(value = 30, unit = TimeUnit.SECONDS)
class PluginConfigDslTest {

    @Test
    @Order(1)
    @DisplayName("pluginConfig___default_values___creates_valid_config")
    fun pluginConfig___default_values___creates_valid_config() {
        val config = pluginConfig { }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"log_level\""))
        assertTrue(json.contains("\"max_concurrent_ops\""))
        assertTrue(json.contains("\"shutdown_timeout_ms\""))
    }

    @Test
    @Order(2)
    @DisplayName("pluginConfig___worker_threads___sets_value")
    fun pluginConfig___worker_threads___sets_value() {
        val config = pluginConfig {
            workerThreads = 8
        }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"worker_threads\":8"))
    }

    @Test
    @Order(3)
    @DisplayName("pluginConfig___log_level___sets_value")
    fun pluginConfig___log_level___sets_value() {
        val config = pluginConfig {
            logLevel = LogLevel.DEBUG
        }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"log_level\":\"debug\""))
    }

    @Test
    @Order(4)
    @DisplayName("pluginConfig___max_concurrent_ops___sets_value")
    fun pluginConfig___max_concurrent_ops___sets_value() {
        val config = pluginConfig {
            maxConcurrentOps = 500
        }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"max_concurrent_ops\":500"))
    }

    @Test
    @Order(5)
    @DisplayName("pluginConfig___shutdown_timeout_duration___sets_value")
    fun pluginConfig___shutdown_timeout_duration___sets_value() {
        val config = pluginConfig {
            shutdownTimeout = 15.seconds
        }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"shutdown_timeout_ms\":15000"))
    }

    @Test
    @Order(10)
    @DisplayName("pluginConfig___data_block___sets_custom_values")
    fun pluginConfig___data_block___sets_custom_values() {
        val config = pluginConfig {
            data {
                "database_url" to "postgres://localhost/test"
                "pool_size" to 10
                "debug_mode" to true
            }
        }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"database_url\""))
        assertTrue(json.contains("postgres://localhost/test"))
        assertTrue(json.contains("\"pool_size\""))
        assertTrue(json.contains("\"debug_mode\""))
    }

    @Test
    @Order(11)
    @DisplayName("pluginConfig___init_params_block___sets_init_values")
    fun pluginConfig___init_params_block___sets_init_values() {
        val config = pluginConfig {
            initParams {
                "run_migrations" to true
                "seed_tables" to listOf("users", "products")
            }
        }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"init_params\""))
        assertTrue(json.contains("\"run_migrations\""))
        assertTrue(json.contains("\"seed_tables\""))
    }

    @Test
    @Order(20)
    @DisplayName("pluginConfig___full_configuration___all_values_set")
    fun pluginConfig___full_configuration___all_values_set() {
        val config = pluginConfig {
            workerThreads = 4
            logLevel = LogLevel.WARN
            maxConcurrentOps = 100
            shutdownTimeout = 30.seconds

            data {
                "app_name" to "test-app"
                "version" to "1.0.0"
            }

            initParams {
                "migrate" to true
            }
        }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"worker_threads\":4"))
        assertTrue(json.contains("\"log_level\":\"warn\""))
        assertTrue(json.contains("\"max_concurrent_ops\":100"))
        assertTrue(json.contains("\"shutdown_timeout_ms\":30000"))
        assertTrue(json.contains("\"app_name\""))
        assertTrue(json.contains("\"init_params\""))
    }

    @Test
    @Order(30)
    @DisplayName("defaultPluginConfig___creates_default_config")
    fun defaultPluginConfig___creates_default_config() {
        val config = defaultPluginConfig()

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"log_level\":\"info\""))
        assertTrue(json.contains("\"max_concurrent_ops\":1000"))
    }

    @Test
    @Order(31)
    @DisplayName("ms_extension___converts_to_duration")
    fun ms_extension___converts_to_duration() {
        val config = pluginConfig {
            shutdownTimeout = 5000.ms
        }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"shutdown_timeout_ms\":5000"))
    }

    @Test
    @Order(32)
    @DisplayName("sec_extension___converts_to_duration")
    fun sec_extension___converts_to_duration() {
        val config = pluginConfig {
            shutdownTimeout = 10.sec
        }

        val json = String(config.toJsonBytes())

        assertTrue(json.contains("\"shutdown_timeout_ms\":10000"))
    }
}
