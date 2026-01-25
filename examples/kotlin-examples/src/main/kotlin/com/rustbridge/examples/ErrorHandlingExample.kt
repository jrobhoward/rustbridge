package com.rustbridge.examples

import com.google.gson.Gson
import com.rustbridge.LogLevel
import com.rustbridge.Plugin
import com.rustbridge.PluginConfig
import com.rustbridge.PluginException
import com.rustbridge.ffm.FfmPluginLoader

/**
 * Example demonstrating error handling with rustbridge.
 *
 * This example shows:
 * - Handling unknown message types
 * - Using sealed classes for type-safe results
 * - Kotlin's `runCatching` for exception handling
 * - Pattern matching with `when` expressions
 */

/**
 * Type-safe result wrapper using sealed classes.
 */
sealed class PluginResult<out T> {
    data class Success<T>(val value: T) : PluginResult<T>()
    data class Error(val exception: PluginException) : PluginResult<Nothing>()
}

/**
 * Extension function for safe calls that return a Result.
 */
inline fun <reified T> Plugin.callSafe(messageType: String, request: Any): PluginResult<T> {
    return runCatching {
        val gson = Gson()
        val requestJson = gson.toJson(request)
        val responseJson = this.call(messageType, requestJson)
        gson.fromJson(responseJson, T::class.java)
    }.fold(
        onSuccess = { PluginResult.Success(it) },
        onFailure = { exception ->
            when (exception) {
                is PluginException -> PluginResult.Error(exception)
                else -> throw exception
            }
        }
    )
}

/**
 * Extension function using Kotlin's Result type.
 */
inline fun <reified T> Plugin.callResult(messageType: String, request: Any): Result<T> {
    return runCatching {
        val gson = Gson()
        val requestJson = gson.toJson(request)
        val responseJson = this.call(messageType, requestJson)
        gson.fromJson(responseJson, T::class.java)
    }
}

fun main() {
    println("=== rustbridge Kotlin Error Handling Example ===\n")

    val pluginPath = findPluginPath()
    val config = PluginConfig().logLevel(LogLevel.WARN)

    FfmPluginLoader.load(pluginPath, config).use { plugin ->

        // Example 1: Handling unknown message types with try-catch
        println("1. Try-Catch Error Handling:")
        println("----------------------------")
        try {
            val gson = Gson()
            val request = gson.toJson(EchoRequest("test"))
            plugin.call("unknown.message.type", request)
            println("   ✗ Should have thrown an exception")
        } catch (e: PluginException) {
            println("   ✓ Caught exception: ${e.message}")
            println("   Error code: ${e.errorCode}")
        }
        println()

        // Example 2: Using runCatching
        println("2. RunCatching Error Handling:")
        println("------------------------------")

        val validResult = runCatching {
            plugin.callResult<EchoResponse>("echo", EchoRequest("Hello!"))
        }
        validResult.fold(
            onSuccess = { println("   ✓ Valid call succeeded") },
            onFailure = { println("   ✗ Valid call failed: ${it.message}") }
        )

        val invalidResult = runCatching {
            plugin.callResult<EchoResponse>("invalid.type", EchoRequest("test"))
        }
        invalidResult.fold(
            onSuccess = { println("   ✗ Invalid call should have failed") },
            onFailure = { println("   ✓ Invalid call failed as expected: ${it.message}") }
        )
        println()

        // Example 3: Using sealed class results
        println("3. Sealed Class Result Pattern:")
        println("-------------------------------")

        val results = listOf(
            "echo" to EchoRequest("Valid message"),
            "invalid.type" to EchoRequest("Invalid message"),
            "echo" to EchoRequest("Another valid message")
        )

        results.forEach { (messageType, request) ->
            when (val result = plugin.callSafe<EchoResponse>(messageType, request)) {
                is PluginResult.Success -> {
                    println("   ✓ $messageType: ${result.value.message}")
                }

                is PluginResult.Error -> {
                    println("   ✗ $messageType: Error ${result.exception.errorCode} - ${result.exception.message}")
                }
            }
        }
        println()

        // Example 4: Using Kotlin's Result type with higher-order functions
        println("4. Result Type with Higher-Order Functions:")
        println("-------------------------------------------")

        val messages = listOf("Hello", "World", "Kotlin", "rustbridge")
        val responses = messages
            .map { msg -> plugin.callResult<EchoResponse>("echo", EchoRequest(msg)) }
            .mapNotNull { it.getOrNull() }
            .map { it.message }

        println("   Successful responses: $responses")
        println()

        // Example 5: Chaining with Result
        println("5. Chaining Operations with Result:")
        println("-----------------------------------")

        val chainedResult = plugin.callResult<EchoResponse>("echo", EchoRequest("Chain Test"))
            .map { response ->
                "Processed: ${response.message} (length: ${response.length})"
            }
            .onSuccess { println("   ✓ Success: $it") }
            .onFailure { println("   ✗ Failure: ${it.message}") }

        println()

        // Example 6: Custom error recovery
        println("6. Error Recovery:")
        println("-----------------")

        fun Plugin.echoWithFallback(message: String): EchoResponse {
            return callResult<EchoResponse>("echo", EchoRequest(message))
                .getOrElse {
                    // Fallback response
                    EchoResponse(message = "Fallback: $message", length = message.length)
                }
        }

        val recovered = plugin.echoWithFallback("Test message")
        println("   Response: ${recovered.message}")
        println()
    }

    println("=== Example Complete ===")
}

