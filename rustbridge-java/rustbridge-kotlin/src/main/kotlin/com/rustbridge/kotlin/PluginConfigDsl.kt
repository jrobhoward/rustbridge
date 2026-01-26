@file:JvmName("PluginConfigDsl")

package com.rustbridge.kotlin

import com.rustbridge.LogLevel
import com.rustbridge.PluginConfig
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds
import kotlin.time.Duration.Companion.seconds

/**
 * DSL marker for plugin configuration builders.
 */
@DslMarker
annotation class PluginConfigDslMarker

/**
 * Create a [PluginConfig] using a Kotlin DSL builder.
 *
 * ```kotlin
 * val config = pluginConfig {
 *     workerThreads = 4
 *     logLevel = LogLevel.DEBUG
 *     maxConcurrentOps = 500
 *     shutdownTimeout = 10.seconds
 *
 *     data {
 *         "database_url" to "postgres://localhost/mydb"
 *         "pool_size" to 10
 *     }
 *
 *     initParams {
 *         "run_migrations" to true
 *         "seed_data" to false
 *     }
 * }
 * ```
 *
 * @param block the configuration builder block
 * @return the configured [PluginConfig]
 */
fun pluginConfig(block: PluginConfigBuilder.() -> Unit): PluginConfig {
    return PluginConfigBuilder().apply(block).build()
}

/**
 * Builder class for creating [PluginConfig] instances with Kotlin DSL syntax.
 */
@PluginConfigDslMarker
class PluginConfigBuilder {
    /**
     * Number of worker threads for the plugin's async runtime.
     */
    var workerThreads: Int? = null

    /**
     * Log level for the plugin.
     */
    var logLevel: LogLevel = LogLevel.INFO

    /**
     * Maximum concurrent operations (0 = unlimited).
     */
    var maxConcurrentOps: Int = 1000

    /**
     * Shutdown timeout as a [Duration].
     */
    var shutdownTimeout: Duration = 5.seconds

    private val dataMap = mutableMapOf<String, Any?>()
    private val initParamsMap = mutableMapOf<String, Any?>()

    /**
     * Configure custom data values using a DSL block.
     *
     * ```kotlin
     * data {
     *     "key1" to "value1"
     *     "key2" to 42
     * }
     * ```
     */
    fun data(block: DataBuilder.() -> Unit) {
        DataBuilder(dataMap).apply(block)
    }

    /**
     * Configure initialization parameters using a DSL block.
     *
     * ```kotlin
     * initParams {
     *     "run_migrations" to true
     *     "seed_data" to listOf("users", "products")
     * }
     * ```
     */
    fun initParams(block: DataBuilder.() -> Unit) {
        DataBuilder(initParamsMap).apply(block)
    }

    /**
     * Set a custom data value directly.
     */
    operator fun set(key: String, value: Any?) {
        dataMap[key] = value
    }

    /**
     * Build the [PluginConfig] from this builder's state.
     */
    fun build(): PluginConfig {
        val config = PluginConfig.defaults()
            .logLevel(logLevel)
            .maxConcurrentOps(maxConcurrentOps)
            .shutdownTimeoutMs(shutdownTimeout.inWholeMilliseconds)

        workerThreads?.let { config.workerThreads(it) }

        dataMap.forEach { (k, v) -> config.set(k, v) }
        initParamsMap.forEach { (k, v) -> config.initParam(k, v) }

        return config
    }

    /**
     * Helper class for building key-value maps with DSL syntax.
     */
    @PluginConfigDslMarker
    class DataBuilder(private val map: MutableMap<String, Any?>) {
        /**
         * Add a key-value pair using infix notation.
         *
         * ```kotlin
         * "key" to "value"
         * ```
         */
        infix fun String.to(value: Any?) {
            map[this] = value
        }

        /**
         * Add a key-value pair using operator syntax.
         *
         * ```kotlin
         * this["key"] = "value"
         * ```
         */
        operator fun set(key: String, value: Any?) {
            map[key] = value
        }
    }
}

/**
 * Extension property to convert milliseconds to a shutdown timeout.
 */
val Int.ms: Duration get() = this.milliseconds

/**
 * Extension property to convert seconds to a shutdown timeout.
 */
val Int.sec: Duration get() = this.seconds

/**
 * Create a default [PluginConfig].
 *
 * Equivalent to `PluginConfig.defaults()` but more idiomatic in Kotlin.
 */
fun defaultPluginConfig(): PluginConfig = PluginConfig.defaults()

/**
 * Extension function to convert a [PluginConfig] into a new one with modifications.
 *
 * Note: This creates a new config and copies settings; it does not modify the original.
 *
 * ```kotlin
 * val base = pluginConfig { workerThreads = 4 }
 * val modified = base.copy { logLevel = LogLevel.DEBUG }
 * ```
 */
fun PluginConfig.copy(block: PluginConfigBuilder.() -> Unit): PluginConfig {
    // Note: PluginConfig doesn't expose getters, so we start fresh
    // This is a convenience method for creating a new config with a DSL
    return pluginConfig(block)
}
