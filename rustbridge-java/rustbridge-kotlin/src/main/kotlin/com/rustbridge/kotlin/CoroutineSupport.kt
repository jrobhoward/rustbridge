@file:JvmName("CoroutineSupport")

package com.rustbridge.kotlin

import com.rustbridge.Plugin
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

/**
 * Call the plugin asynchronously using Kotlin coroutines.
 *
 * This suspend function moves the blocking plugin call to [Dispatchers.IO],
 * allowing it to be called from coroutine contexts without blocking.
 *
 * ```kotlin
 * suspend fun fetchData(): EchoResponse {
 *     return plugin.callAsync<EchoRequest, EchoResponse>("echo", EchoRequest("hello"))
 * }
 *
 * // Usage in a coroutine scope
 * launch {
 *     val response = fetchData()
 *     println(response.message)
 * }
 * ```
 *
 * @param T the request type
 * @param R the response type
 * @param typeTag the message type identifier
 * @param request the request object
 * @return the deserialized response
 * @throws PluginException if the call fails
 */
suspend inline fun <reified T, reified R> Plugin.callAsync(
    typeTag: String,
    request: T
): R = withContext(Dispatchers.IO) {
    call<T, R>(typeTag, request)
}

/**
 * Call the plugin asynchronously with a JSON request and typed response.
 *
 * @param R the response type
 * @param typeTag the message type identifier
 * @param requestJson the JSON request string
 * @return the deserialized response
 */
suspend inline fun <reified R> Plugin.callAsync(
    typeTag: String,
    requestJson: String
): R = withContext(Dispatchers.IO) {
    callAs<R>(typeTag, requestJson)
}

/**
 * Call the plugin asynchronously and return a Kotlin [Result].
 *
 * ```kotlin
 * val result = plugin.callAsyncResult<Req, Resp>("echo", request)
 * result.onSuccess { println("Got: ${it.message}") }
 *       .onFailure { println("Error: ${it.message}") }
 * ```
 *
 * @param T the request type
 * @param R the response type
 * @param typeTag the message type identifier
 * @param request the request object
 * @return [Result.success] with the response, or [Result.failure] with the exception
 */
suspend inline fun <reified T, reified R> Plugin.callAsyncResult(
    typeTag: String,
    request: T
): Result<R> = withContext(Dispatchers.IO) {
    runCatching { call<T, R>(typeTag, request) }
}

/**
 * Call the plugin asynchronously and return a [PluginResult].
 *
 * ```kotlin
 * when (val result = plugin.callAsyncSafe<Req, Resp>("echo", request)) {
 *     is PluginResult.Success -> println("Got: ${result.value}")
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
suspend inline fun <reified T, reified R> Plugin.callAsyncSafe(
    typeTag: String,
    request: T
): PluginResult<R> = withContext(Dispatchers.IO) {
    callSafe<T, R>(typeTag, request)
}

/**
 * Call the plugin asynchronously, returning null on failure.
 *
 * @param T the request type
 * @param R the response type
 * @param typeTag the message type identifier
 * @param request the request object
 * @return the response, or null if the call failed
 */
suspend inline fun <reified T, reified R> Plugin.callAsyncOrNull(
    typeTag: String,
    request: T
): R? = withContext(Dispatchers.IO) {
    callOrNull<T, R>(typeTag, request)
}

/**
 * Call the plugin asynchronously with raw JSON, returning raw JSON response.
 *
 * This is useful when you need full control over serialization.
 *
 * @param typeTag the message type identifier
 * @param requestJson the JSON request string
 * @return the raw JSON response string
 */
suspend fun Plugin.callAsyncJson(
    typeTag: String,
    requestJson: String
): String = withContext(Dispatchers.IO) {
    call(typeTag, requestJson)
}

/**
 * Execute multiple plugin calls concurrently.
 *
 * ```kotlin
 * val results = plugin.callAllAsync(
 *     "echo" to EchoRequest("hello"),
 *     "echo" to EchoRequest("world"),
 *     "echo" to EchoRequest("!")
 * )
 * results.forEach { println(it) }
 * ```
 *
 * @param T the request type
 * @param R the response type
 * @param calls pairs of (typeTag, request)
 * @return list of responses in the same order as inputs
 */
suspend inline fun <reified T, reified R> Plugin.callAllAsync(
    vararg calls: Pair<String, T>
): List<R> = withContext(Dispatchers.IO) {
    // Note: These run sequentially on IO dispatcher
    // For true parallelism, use async {} blocks in calling code
    calls.map { (typeTag, request) -> call<T, R>(typeTag, request) }
}
