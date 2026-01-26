@file:JvmName("PluginExtensions")

package com.rustbridge.kotlin

import com.fasterxml.jackson.databind.DeserializationFeature
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import com.rustbridge.Plugin
import com.rustbridge.PluginException

/**
 * Shared Jackson ObjectMapper configured for Kotlin.
 */
@PublishedApi
internal val objectMapper = jacksonObjectMapper().apply {
    configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
}

/**
 * Call the plugin with a typed request and response using reified generics.
 *
 * This extension function provides type-safe plugin calls without explicit
 * class parameters, leveraging Kotlin's reified type parameters.
 *
 * ```kotlin
 * data class EchoRequest(val message: String)
 * data class EchoResponse(val message: String, val timestamp: Long)
 *
 * val response = plugin.call<EchoRequest, EchoResponse>("echo", EchoRequest("hello"))
 * println(response.message)
 * ```
 *
 * @param T the request type (will be serialized to JSON)
 * @param R the response type (will be deserialized from JSON)
 * @param typeTag the message type identifier
 * @param request the request object
 * @return the deserialized response
 * @throws PluginException if the call fails
 */
inline fun <reified T, reified R> Plugin.call(typeTag: String, request: T): R {
    val requestJson = objectMapper.writeValueAsString(request)
    val responseJson = call(typeTag, requestJson)
    return objectMapper.readValue(responseJson)
}

/**
 * Call the plugin with a JSON request and typed response.
 *
 * ```kotlin
 * val response = plugin.call<EchoResponse>("echo", """{"message": "hello"}""")
 * ```
 *
 * @param R the response type
 * @param typeTag the message type identifier
 * @param requestJson the JSON request string
 * @return the deserialized response
 */
inline fun <reified R> Plugin.call(typeTag: String, requestJson: String): R {
    val responseJson = call(typeTag, requestJson)
    return objectMapper.readValue(responseJson)
}

/**
 * Call the plugin with a typed request and return raw JSON response.
 *
 * Useful when you need to inspect the raw response or handle deserialization manually.
 *
 * @param T the request type
 * @param typeTag the message type identifier
 * @param request the request object
 * @return the raw JSON response string
 */
inline fun <reified T> Plugin.callJson(typeTag: String, request: T): String {
    val requestJson = objectMapper.writeValueAsString(request)
    return call(typeTag, requestJson)
}

/**
 * Call the plugin and return a Kotlin [Result], capturing any exceptions.
 *
 * This is useful for functional error handling without try-catch blocks.
 *
 * ```kotlin
 * plugin.callResult<EchoRequest, EchoResponse>("echo", request)
 *     .onSuccess { println("Got: ${it.message}") }
 *     .onFailure { println("Error: ${it.message}") }
 * ```
 *
 * @param T the request type
 * @param R the response type
 * @param typeTag the message type identifier
 * @param request the request object
 * @return [Result.success] with the response, or [Result.failure] with the exception
 */
inline fun <reified T, reified R> Plugin.callResult(typeTag: String, request: T): Result<R> {
    return runCatching { call<T, R>(typeTag, request) }
}

/**
 * Call the plugin and return a [PluginResult] sealed class for exhaustive pattern matching.
 *
 * ```kotlin
 * when (val result = plugin.callSafe<EchoRequest, EchoResponse>("echo", request)) {
 *     is PluginResult.Success -> println("Got: ${result.value.message}")
 *     is PluginResult.Error -> println("Error ${result.code}: ${result.message}")
 * }
 * ```
 *
 * @param T the request type
 * @param R the response type
 * @param typeTag the message type identifier
 * @param request the request object
 * @return [PluginResult.Success] or [PluginResult.Error]
 */
inline fun <reified T, reified R> Plugin.callSafe(typeTag: String, request: T): PluginResult<R> {
    return try {
        PluginResult.Success(call<T, R>(typeTag, request))
    } catch (e: PluginException) {
        PluginResult.Error(e.errorCode, e.message ?: "Unknown error", e)
    } catch (e: Exception) {
        PluginResult.Error(0, e.message ?: "Unknown error", e)
    }
}

/**
 * Call the plugin, returning null on failure instead of throwing.
 *
 * ```kotlin
 * val response = plugin.callOrNull<EchoRequest, EchoResponse>("echo", request)
 *     ?: return defaultResponse
 * ```
 *
 * @param T the request type
 * @param R the response type
 * @param typeTag the message type identifier
 * @param request the request object
 * @return the response, or null if the call failed
 */
inline fun <reified T, reified R> Plugin.callOrNull(typeTag: String, request: T): R? {
    return try {
        call<T, R>(typeTag, request)
    } catch (_: Exception) {
        null
    }
}

/**
 * Call the plugin, returning a default value on failure.
 *
 * ```kotlin
 * val response = plugin.callOrDefault("echo", request) { EchoResponse("fallback", 0) }
 * ```
 *
 * @param T the request type
 * @param R the response type
 * @param typeTag the message type identifier
 * @param request the request object
 * @param default lambda providing the default value on failure
 * @return the response, or the default value if the call failed
 */
inline fun <reified T, reified R> Plugin.callOrDefault(
    typeTag: String,
    request: T,
    default: () -> R
): R {
    return try {
        call<T, R>(typeTag, request)
    } catch (_: Exception) {
        default()
    }
}
