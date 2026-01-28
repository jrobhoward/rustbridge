# Getting Started: Kotlin

This guide walks you through using rustbridge plugins from Kotlin with idiomatic patterns.

## Prerequisites

- **Kotlin 1.9+** - For modern language features
- **Java 21+** - Required for FFM (Foreign Function & Memory API)
- **Gradle** - For dependency management
- **A rustbridge plugin** - Either a `.rbp` bundle or native library

## Project Setup

### Gradle (Kotlin DSL)

```kotlin
plugins {
    kotlin("jvm") version "2.0.0"
    application
}

application {
    mainClass.set("com.example.MainKt")
}

repositories {
    mavenLocal()  // For local development (see below)
    mavenCentral()
}

dependencies {
    implementation("com.rustbridge:rustbridge-core:0.5.0")
    implementation("com.rustbridge:rustbridge-ffm:0.5.0")
    implementation("com.rustbridge:rustbridge-kotlin:0.5.0")  // Kotlin extensions
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.15.2")

    // Optional: for coroutine support
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.3")

    testImplementation(kotlin("test"))
}

kotlin {
    jvmToolchain(21)
}

tasks.test {
    useJUnitPlatform()
}

// Required for FFM native access
tasks.withType<JavaExec> {
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
}
```

> **Important**: The `--enable-native-access=ALL-UNNAMED` flag is required for FFM to call native code. Without it, you'll get `IllegalCallerException`.

## The rustbridge-kotlin Module

The `rustbridge-kotlin` module provides idiomatic Kotlin extensions:

- **Type-safe calls** with reified generics (`call<T, R>()`)
- **Coroutine support** with suspend functions (`callAsync<T, R>()`)
- **Result-based error handling** with sealed classes (`PluginResult`)
- **DSL for configuration** (`pluginConfig { ... }`)
- **Plugin lifecycle helpers** (`withPlugin`, `lazyPlugin`)

## Local Development

When working with rustbridge source code (not published to Maven Central), publish to MavenLocal first:

```bash
cd rustbridge-java
./gradlew publishToMavenLocal
```

The `mavenLocal()` repository in the build file above will then resolve the local artifacts.

## Loading a Plugin

### From Bundle (Recommended)

```kotlin
import com.rustbridge.BundleLoader
import com.rustbridge.ffm.FfmPluginLoader

fun main() {
    val bundleLoader = BundleLoader.builder()
        .bundlePath("my-plugin-1.0.0.rbp")
        .verifySignatures(false)  // Set true for production
        .build()

    val libraryPath = bundleLoader.extractLibrary()

    FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
        val response = plugin.call("echo", """{"message": "Hello"}""")
        println(response)
    }

    bundleLoader.close()
}
```

### From Raw Library

```kotlin
import com.rustbridge.ffm.FfmPluginLoader

val pluginPath = "target/release/libmyplugin.so"

FfmPluginLoader.load(pluginPath).use { plugin ->
    val response = plugin.call("echo", """{"message": "Hello"}""")
    println(response)
}
```

## Type-Safe Calls with Data Classes

The `rustbridge-kotlin` module provides built-in type-safe extensions:

```kotlin
import com.rustbridge.kotlin.call  // Extension functions

data class EchoRequest(val message: String)
data class EchoResponse(val message: String, val length: Int)

// Using built-in extension (recommended)
FfmPluginLoader.load(pluginPath).use { plugin ->
    val response: EchoResponse = plugin.call("echo", EchoRequest("Hello, Kotlin!"))
    println("Message: ${response.message}")
    println("Length: ${response.length}")
}
```

Available type-safe call variants:

```kotlin
// Typed request and response
val response: EchoResponse = plugin.call("echo", EchoRequest("Hello"))

// JSON request, typed response
val response: EchoResponse = plugin.call("echo", """{"message": "Hello"}""")

// Typed request, JSON response
val json: String = plugin.callJson("echo", EchoRequest("Hello"))

// With Result wrapper (no exceptions)
val result: Result<EchoResponse> = plugin.callResult("echo", EchoRequest("Hello"))

// Null on failure
val response: EchoResponse? = plugin.callOrNull("echo", EchoRequest("Hello"))

// Default on failure
val response: EchoResponse = plugin.callOrDefault("echo", request) { EchoResponse("", 0) }
```

## Configuration

### Using Java Builder

```kotlin
import com.rustbridge.PluginConfig
import com.rustbridge.LogLevel

val config = PluginConfig.defaults()
    .logLevel(LogLevel.DEBUG)
    .workerThreads(4)
    .maxConcurrentOps(100)
    .shutdownTimeoutMs(5000)

FfmPluginLoader.load(pluginPath, config).use { plugin ->
    // Plugin configured...
}
```

### Using Kotlin DSL (Recommended)

```kotlin
import com.rustbridge.kotlin.pluginConfig
import com.rustbridge.kotlin.sec
import com.rustbridge.LogLevel

val config = pluginConfig {
    logLevel = LogLevel.DEBUG
    workerThreads = 4
    maxConcurrentOps = 100
    shutdownTimeout = 5.sec  // Kotlin Duration extension
    data {
        "custom_key" to "custom_value"
    }
    initParams {
        "init_key" to 42
    }
}
```

## Logging with Lambdas

```kotlin
import com.rustbridge.LogCallback

val callback = LogCallback { level, target, message ->
    println("[$level] $target: $message")
}

FfmPluginLoader.load(pluginPath, config, callback).use { plugin ->
    plugin.call("echo", """{"message": "test"}""")
}
```

## Coroutines Integration

The `rustbridge-kotlin` module provides built-in suspend functions:

```kotlin
import com.rustbridge.kotlin.callAsync
import com.rustbridge.kotlin.callAsyncResult
import kotlinx.coroutines.*

// Usage
runBlocking {
    FfmPluginLoader.load(pluginPath).use { plugin ->
        // Suspending call
        val response: EchoResponse = plugin.callAsync("echo", EchoRequest("Hello"))
        println(response)

        // With Result wrapper
        val result: Result<EchoResponse> = plugin.callAsyncResult("echo", EchoRequest("Hello"))
        result.onSuccess { println(it) }
              .onFailure { println("Error: ${it.message}") }
    }
}
```

### Concurrent Calls

```kotlin
runBlocking {
    FfmPluginLoader.load(pluginPath).use { plugin ->
        val results = (1..100).map { i ->
            async(Dispatchers.IO) {
                plugin.callTyped<EchoResponse>("echo", EchoRequest("Message $i"))
            }
        }.awaitAll()

        println("Completed ${results.size} calls")
    }
}
```

## Error Handling

```kotlin
import com.rustbridge.PluginException

try {
    plugin.call("invalid.type", "{}")
} catch (e: PluginException) {
    when (e.errorCode) {
        6 -> println("Unknown message type: ${e.message}")
        7 -> println("Handler error: ${e.message}")
        13 -> println("Too many concurrent requests")
        else -> println("Error (${e.errorCode}): ${e.message}")
    }
}
```

### Result-Based Error Handling

The `rustbridge-kotlin` module provides a `PluginResult` sealed class:

```kotlin
import com.rustbridge.kotlin.callSafe
import com.rustbridge.kotlin.PluginResult

// Using built-in PluginResult
val result: PluginResult<EchoResponse> = plugin.callSafe("echo", request)

when (result) {
    is PluginResult.Success -> println("Result: ${result.value}")
    is PluginResult.Error -> println("Error ${result.code}: ${result.message}")
}

// PluginResult utility methods
result.getOrNull()                           // Returns value or null
result.getOrThrow()                          // Returns value or throws
result.getOrElse { EchoResponse("", 0) }     // Returns value or default
result.map { it.message }                    // Transform success value
result.recover { "fallback" }               // Recover from error
result.onSuccess { println(it) }            // Side effect on success
result.onError { println(it.message) }      // Side effect on error
result.toResult()                           // Convert to kotlin.Result
```

## Binary Transport

```kotlin
import java.lang.foreign.*
import java.nio.charset.StandardCharsets

const val MSG_ECHO = 1

val ECHO_REQUEST_LAYOUT: StructLayout = MemoryLayout.structLayout(
    ValueLayout.JAVA_BYTE.withName("version"),
    MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_BYTE).withName("_reserved"),
    MemoryLayout.sequenceLayout(256, ValueLayout.JAVA_BYTE).withName("message"),
    ValueLayout.JAVA_INT.withName("message_len")
)

fun Plugin.callBinaryEcho(message: String): Int {
    Arena.ofConfined().use { arena ->
        val request = arena.allocate(ECHO_REQUEST_LAYOUT)
        request.set(ValueLayout.JAVA_BYTE, 0, 1.toByte())  // version

        val msgBytes = message.toByteArray(StandardCharsets.UTF_8)
        MemorySegment.copy(msgBytes, 0, request, 4, msgBytes.size)
        request.set(ValueLayout.JAVA_INT, 260, msgBytes.size)

        val response = callRaw(MSG_ECHO, request, 268)
        return response.get(ValueLayout.JAVA_INT, 264)
    }
}
```

## Plugin Lifecycle Helpers

The `rustbridge-kotlin` module provides convenient lifecycle management:

```kotlin
import com.rustbridge.kotlin.loadPlugin
import com.rustbridge.kotlin.withPlugin
import com.rustbridge.kotlin.lazyPlugin

// DSL-style loading
val plugin = loadPlugin(pluginPath) {
    logLevel = LogLevel.DEBUG
    workerThreads = 4
}

// Auto-closing block
withPlugin(pluginPath) { plugin ->
    val response: EchoResponse = plugin.call("echo", EchoRequest("Hello"))
    println(response)
}

// Receiver-style block
withPluginContext(pluginPath) {
    val response: EchoResponse = call("echo", EchoRequest("Hello"))
    println(response)
}

// Lazy initialization
val lazyPlugin by lazyPlugin(pluginPath) {
    logLevel = LogLevel.INFO
}
```

## Complete Example

```kotlin
import com.rustbridge.*
import com.rustbridge.ffm.FfmPluginLoader
import com.rustbridge.kotlin.*
import kotlinx.coroutines.*

data class AddRequest(val a: Long, val b: Long)
data class AddResponse(val result: Long)

fun main() = runBlocking {
    val config = pluginConfig {
        logLevel = LogLevel.INFO
    }

    val callback = LogCallback { level, _, message ->
        println("[$level] $message")
    }

    FfmPluginLoader.load(
        "target/release/libcalculator_plugin.so",
        config,
        callback
    ).use { plugin ->
        // Single call using built-in extension
        val response: AddResponse = plugin.call("math.add", AddRequest(42, 58))
        println("42 + 58 = ${response.result}")

        // Concurrent calls using built-in async extension
        val results = (1..10).map { i ->
            async(Dispatchers.IO) {
                plugin.callAsync<AddRequest, AddResponse>("math.add", AddRequest(i.toLong(), i.toLong()))
            }
        }.awaitAll()

        results.forEachIndexed { i, r ->
            println("${i + 1} + ${i + 1} = ${r.result}")
        }
    }
}
```

## Related Documentation

- [JAVA_FFM.md](./JAVA_FFM.md) - Java FFM details
- [../TRANSPORT.md](../TRANSPORT.md) - Transport layer details
- [../MEMORY_MODEL.md](../MEMORY_MODEL.md) - Memory ownership
