@file:JvmName("PluginLoaderExtensions")

package com.rustbridge.kotlin

import com.rustbridge.LogCallback
import com.rustbridge.Plugin
import com.rustbridge.PluginConfig
import com.rustbridge.ffm.FfmPluginLoader
import java.nio.file.Path

/**
 * Load a plugin with Kotlin DSL configuration.
 *
 * ```kotlin
 * val plugin = loadPlugin("/path/to/plugin.so") {
 *     workerThreads = 4
 *     logLevel = LogLevel.DEBUG
 *     shutdownTimeout = 10.seconds
 * }
 *
 * plugin.use {
 *     val response = it.call<Req, Resp>("echo", request)
 * }
 * ```
 *
 * @param libraryPath path to the plugin shared library
 * @param logCallback optional log callback
 * @param config configuration builder block
 * @return the loaded plugin
 */
fun loadPlugin(
    libraryPath: String,
    logCallback: LogCallback? = null,
    config: PluginConfigBuilder.() -> Unit = {}
): Plugin {
    val pluginConfig = pluginConfig(config)
    return FfmPluginLoader.load(Path.of(libraryPath), pluginConfig, logCallback)
}

/**
 * Load a plugin with Kotlin DSL configuration.
 *
 * @param libraryPath path to the plugin shared library
 * @param logCallback optional log callback
 * @param config configuration builder block
 * @return the loaded plugin
 */
fun loadPlugin(
    libraryPath: Path,
    logCallback: LogCallback? = null,
    config: PluginConfigBuilder.() -> Unit = {}
): Plugin {
    val pluginConfig = pluginConfig(config)
    return FfmPluginLoader.load(libraryPath, pluginConfig, logCallback)
}

/**
 * Load a plugin with an existing [PluginConfig].
 *
 * @param libraryPath path to the plugin shared library
 * @param config the plugin configuration
 * @param logCallback optional log callback
 * @return the loaded plugin
 */
fun loadPlugin(
    libraryPath: String,
    config: PluginConfig,
    logCallback: LogCallback? = null
): Plugin {
    return FfmPluginLoader.load(Path.of(libraryPath), config, logCallback)
}

/**
 * Load a plugin and execute a block, automatically closing the plugin afterwards.
 *
 * This is the recommended way to use plugins in Kotlin, ensuring proper resource cleanup.
 *
 * ```kotlin
 * usePlugin("/path/to/plugin.so") {
 *     workerThreads = 4
 *     logLevel = LogLevel.DEBUG
 * }.use { plugin ->
 *     val response = plugin.call<Req, Resp>("echo", request)
 *     println(response)
 * }
 * ```
 *
 * Or more concisely:
 *
 * ```kotlin
 * withPlugin("/path/to/plugin.so") { plugin ->
 *     val response = plugin.call<Req, Resp>("echo", request)
 *     println(response)
 * }
 * ```
 *
 * @param libraryPath path to the plugin shared library
 * @param logCallback optional log callback
 * @param config configuration builder block
 * @param block the code to execute with the plugin
 * @return the result of the block
 */
inline fun <R> withPlugin(
    libraryPath: String,
    logCallback: LogCallback? = null,
    noinline config: PluginConfigBuilder.() -> Unit = {},
    block: (Plugin) -> R
): R {
    return loadPlugin(libraryPath, logCallback, config).use(block)
}

/**
 * Load a plugin and execute a block with the plugin as receiver.
 *
 * ```kotlin
 * withPlugin("/path/to/plugin.so") {
 *     // 'this' is the Plugin
 *     val response = call<Req, Resp>("echo", request)
 *     println(response)
 * }
 * ```
 *
 * @param libraryPath path to the plugin shared library
 * @param logCallback optional log callback
 * @param config configuration builder block
 * @param block the code to execute with the plugin as receiver
 * @return the result of the block
 */
inline fun <R> withPluginContext(
    libraryPath: String,
    logCallback: LogCallback? = null,
    noinline config: PluginConfigBuilder.() -> Unit = {},
    block: Plugin.() -> R
): R {
    return loadPlugin(libraryPath, logCallback, config).use { it.block() }
}

/**
 * Create a lazy plugin that is loaded on first access.
 *
 * ```kotlin
 * val plugin by lazyPlugin("/path/to/plugin.so") {
 *     workerThreads = 4
 * }
 *
 * // Plugin is loaded here on first use
 * val response = plugin.call<Req, Resp>("echo", request)
 * ```
 *
 * Note: You are responsible for calling `close()` on the plugin when done.
 *
 * @param libraryPath path to the plugin shared library
 * @param logCallback optional log callback
 * @param config configuration builder block
 * @return a lazy delegate that loads the plugin on first access
 */
fun lazyPlugin(
    libraryPath: String,
    logCallback: LogCallback? = null,
    config: PluginConfigBuilder.() -> Unit = {}
): Lazy<Plugin> = lazy {
    loadPlugin(libraryPath, logCallback, config)
}
