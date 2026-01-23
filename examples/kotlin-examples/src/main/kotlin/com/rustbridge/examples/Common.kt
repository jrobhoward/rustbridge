package com.rustbridge.examples

import com.google.gson.Gson
import com.rustbridge.Plugin
import java.nio.file.Paths

/**
 * Common data classes and utility functions shared across examples.
 */

// Data classes matching Rust message types
data class EchoRequest(val message: String)
data class EchoResponse(val message: String, val length: Int)

data class GreetRequest(val name: String)
data class GreetResponse(val greeting: String)

data class CreateUserRequest(val username: String, val email: String)
data class CreateUserResponse(val user_id: String, val created_at: String)

data class AddRequest(val a: Long, val b: Long)
data class AddResponse(val result: Long)

/**
 * Extension function for type-safe plugin calls.
 *
 * This function handles JSON serialization/deserialization automatically,
 * providing a clean, type-safe API for calling plugin methods.
 */
inline fun <reified T> Plugin.callTyped(messageType: String, request: Any): T {
    val gson = Gson()
    val requestJson = gson.toJson(request)
    val responseJson = this.call(messageType, requestJson)
    return gson.fromJson(responseJson, T::class.java)
}

/**
 * Locate the hello-plugin library based on the platform.
 *
 * Searches common build output locations for the plugin library.
 */
fun findPluginPath(): String {
    val osName = System.getProperty("os.name").lowercase()
    val libraryName = when {
        osName.contains("linux") -> "libhello_plugin.so"
        osName.contains("mac") || osName.contains("darwin") -> "libhello_plugin.dylib"
        osName.contains("windows") -> "hello_plugin.dll"
        else -> error("Unsupported OS: $osName")
    }

    // Try common locations (hello-plugin is in workspace, so target is at workspace root)
    val locations = listOf(
        "../../target/release/$libraryName",        // From examples/kotlin-examples
        "../../target/debug/$libraryName",          // Debug build
        "../../../target/release/$libraryName",     // From kotlin-examples/build dirs
        libraryName                                  // Current directory
    )

    return locations
        .map { Paths.get(it).toAbsolutePath().normalize().toString() }
        .firstOrNull { java.io.File(it).exists() }
        ?: error("Could not find $libraryName. Build hello-plugin first with: cargo build --release")
}
