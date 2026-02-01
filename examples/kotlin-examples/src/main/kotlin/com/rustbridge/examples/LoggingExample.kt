package com.rustbridge.examples

import com.rustbridge.LogCallback
import com.rustbridge.LogLevel
import com.rustbridge.Plugin
import com.rustbridge.PluginConfig
import com.rustbridge.ffm.FfmPluginLoader
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter

/**
 * Example demonstrating log callback integration.
 *
 * This example shows:
 * - Custom log callback implementation
 * - Log level filtering
 * - Integration with host application logging
 */

/**
 * Custom log callback that formats logs with timestamps and colors.
 */
class ColoredLogCallback : LogCallback {
    private val formatter = DateTimeFormatter.ofPattern("HH:mm:ss.SSS")

    override fun log(level: LogLevel, target: String, message: String) {
        val timestamp = LocalDateTime.now().format(formatter)
        val coloredLevel = when (level) {
            LogLevel.ERROR -> "\u001B[31mERROR\u001B[0m" // Red
            LogLevel.WARN -> "\u001B[33mWARN \u001B[0m" // Yellow
            LogLevel.INFO -> "\u001B[32mINFO \u001B[0m" // Green
            LogLevel.DEBUG -> "\u001B[36mDEBUG\u001B[0m" // Cyan
            LogLevel.TRACE -> "\u001B[37mTRACE\u001B[0m" // White
            else -> level.toString()
        }

        println("[$timestamp] $coloredLevel [$target] $message")
    }
}

/**
 * Simple counting log callback for testing.
 */
class CountingLogCallback : LogCallback {
    var errorCount = 0
        private set
    var warnCount = 0
        private set
    var infoCount = 0
        private set
    var debugCount = 0
        private set

    override fun log(level: LogLevel, target: String, message: String) {
        when (level) {
            LogLevel.ERROR -> errorCount++
            LogLevel.WARN -> warnCount++
            LogLevel.INFO -> infoCount++
            LogLevel.DEBUG -> debugCount++
            else -> {}
        }
        println("[$level] [$target] $message")
    }

    fun summary() = "Errors: $errorCount, Warnings: $warnCount, Info: $infoCount, Debug: $debugCount"
}

fun main() {
    println("=== rustbridge Kotlin Logging Example ===\n")

    val pluginPath = findPluginPath()

    // Example 1: Colored logging
    println("Example 1: Colored Log Output")
    println("--------------------------------")

    val coloredConfig = PluginConfig()
        .logLevel(LogLevel.DEBUG)
    // TODO: Add log callback support once FFM upcalls are implemented

    FfmPluginLoader.load(pluginPath, coloredConfig).use { plugin ->
        plugin.callTyped<GreetResponse>("greet", GreetRequest("Alice"))
    }

    println("\n")

    // Example 2: Counting logs
    println("Example 2: Counting Logs")
    println("------------------------")

    val counter = CountingLogCallback()
    val countingConfig = PluginConfig()
        .logLevel(LogLevel.INFO)
    // TODO: Add log callback support once FFM upcalls are implemented

    FfmPluginLoader.load(pluginPath, countingConfig).use { plugin ->
        // Perform multiple operations
        repeat(5) { i ->
            plugin.callTyped<GreetResponse>("greet", GreetRequest("User $i"))
        }
    }

    println("\nLog Summary: ${counter.summary()}")
    println()

    // Example 3: Log level filtering
    println("Example 3: Log Level Filtering")
    println("-------------------------------")

    println("\nWith log level = INFO:")
    val infoConfig = PluginConfig()
        .logLevel(LogLevel.INFO)
    // TODO: Add log callback support once FFM upcalls are implemented

    FfmPluginLoader.load(pluginPath, infoConfig).use { plugin ->
        plugin.callTyped<GreetResponse>("greet", GreetRequest("Info Level"))
    }

    println("\nWith log level = DEBUG:")
    val debugConfig = PluginConfig()
        .logLevel(LogLevel.DEBUG)
    // TODO: Add log callback support once FFM upcalls are implemented

    FfmPluginLoader.load(pluginPath, debugConfig).use { plugin ->
        plugin.callTyped<GreetResponse>("greet", GreetRequest("Debug Level"))
    }

    println("\n=== Example Complete ===")
}

