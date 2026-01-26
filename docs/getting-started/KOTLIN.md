# Getting Started: Kotlin

This guide walks you through using rustbridge plugins from Kotlin with idiomatic patterns.

## Prerequisites

- **Kotlin 1.9+** - For modern language features
- **Java 21+** - For FFM (or Java 8+ for JNI)
- **Gradle** - For dependency management
- **A rustbridge plugin** - Either a `.rbp` bundle or native library

## Add Dependencies

### Gradle (Kotlin DSL)

```kotlin
plugins {
    kotlin("jvm") version "1.9.0"
    kotlin("plugin.serialization") version "1.9.0"  // Optional: for kotlinx.serialization
}

dependencies {
    implementation("com.rustbridge:rustbridge-core:0.1.0")
    implementation("com.rustbridge:rustbridge-ffm:0.1.0")  // For Java 21+
    // OR
    // implementation("com.rustbridge:rustbridge-jni:0.1.0")  // For Java 8+

    // Serialization (pick one)
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.15.2")
    // OR
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.6.0")
}
```

## Loading a Plugin

### From Bundle (Recommended)

```kotlin
import com.rustbridge.BundleLoader
import com.rustbridge.ffm.FfmPluginLoader
import com.rustbridge.PluginConfig

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

### Using Jackson

```kotlin
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue

data class EchoRequest(val message: String)
data class EchoResponse(val message: String, val length: Int)

val mapper = jacksonObjectMapper()

// Extension function for type-safe calls
inline fun <reified T> com.rustbridge.Plugin.callTyped(
    messageType: String,
    request: Any
): T {
    val requestJson = mapper.writeValueAsString(request)
    val responseJson = call(messageType, requestJson)
    return mapper.readValue(responseJson)
}

// Usage
FfmPluginLoader.load(pluginPath).use { plugin ->
    val response = plugin.callTyped<EchoResponse>(
        "echo",
        EchoRequest("Hello, Kotlin!")
    )
    println("Message: ${response.message}")
    println("Length: ${response.length}")
}
```

### Using kotlinx.serialization

```kotlin
import kotlinx.serialization.Serializable
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

@Serializable
data class EchoRequest(val message: String)

@Serializable
data class EchoResponse(val message: String, val length: Int)

val json = Json { ignoreUnknownKeys = true }

inline fun <reified T> com.rustbridge.Plugin.callTyped(
    messageType: String,
    request: Any
): T {
    val requestJson = json.encodeToString(request)
    val responseJson = call(messageType, requestJson)
    return json.decodeFromString(responseJson)
}
```

## Configuration

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

Wrap plugin calls in suspending functions for async usage:

```kotlin
import kotlinx.coroutines.*

suspend fun <T> Plugin.callAsync(
    messageType: String,
    request: Any
): T = withContext(Dispatchers.IO) {
    callTyped(messageType, request)
}

// Usage
runBlocking {
    FfmPluginLoader.load(pluginPath).use { plugin ->
        val response = plugin.callAsync<EchoResponse>("echo", EchoRequest("Hello"))
        println(response)
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

```kotlin
sealed class PluginResult<out T> {
    data class Success<T>(val value: T) : PluginResult<T>()
    data class Error(val code: Int, val message: String) : PluginResult<Nothing>()
}

inline fun <reified T> Plugin.callSafe(
    messageType: String,
    request: Any
): PluginResult<T> {
    return try {
        PluginResult.Success(callTyped(messageType, request))
    } catch (e: PluginException) {
        PluginResult.Error(e.errorCode, e.message ?: "Unknown error")
    }
}

// Usage
when (val result = plugin.callSafe<EchoResponse>("echo", request)) {
    is PluginResult.Success -> println("Result: ${result.value}")
    is PluginResult.Error -> println("Error ${result.code}: ${result.message}")
}
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

## DSL-Style API

Create a Kotlin DSL for plugin calls:

```kotlin
class PluginScope(private val plugin: Plugin) {
    inline fun <reified T> call(
        messageType: String,
        builder: RequestBuilder.() -> Unit
    ): T {
        val request = RequestBuilder().apply(builder).build()
        return plugin.callTyped(messageType, request)
    }
}

class RequestBuilder {
    private val map = mutableMapOf<String, Any>()

    operator fun String.invoke(value: Any) {
        map[this] = value
    }

    fun build(): Map<String, Any> = map
}

fun Plugin.scope(block: PluginScope.() -> Unit) {
    PluginScope(this).block()
}

// Usage
FfmPluginLoader.load(pluginPath).use { plugin ->
    plugin.scope {
        val response: EchoResponse = call("echo") {
            "message"("Hello, DSL!")
        }
        println(response)
    }
}
```

## Complete Example

```kotlin
import com.rustbridge.*
import com.rustbridge.ffm.FfmPluginLoader
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import kotlinx.coroutines.*

data class AddRequest(val a: Long, val b: Long)
data class AddResponse(val result: Long)

val mapper = jacksonObjectMapper()

inline fun <reified T> Plugin.callTyped(messageType: String, request: Any): T {
    val requestJson = mapper.writeValueAsString(request)
    val responseJson = call(messageType, requestJson)
    return mapper.readValue(responseJson)
}

fun main() = runBlocking {
    val config = PluginConfig.defaults()
        .logLevel(LogLevel.INFO)

    val callback = LogCallback { level, _, message ->
        println("[$level] $message")
    }

    FfmPluginLoader.load(
        "target/release/libcalculator_plugin.so",
        config,
        callback
    ).use { plugin ->
        // Single call
        val response = plugin.callTyped<AddResponse>(
            "math.add",
            AddRequest(42, 58)
        )
        println("42 + 58 = ${response.result}")

        // Concurrent calls
        val results = (1..10).map { i ->
            async(Dispatchers.IO) {
                plugin.callTyped<AddResponse>("math.add", AddRequest(i.toLong(), i.toLong()))
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
- [JAVA_JNI.md](./JAVA_JNI.md) - Java JNI details
- [../TRANSPORT.md](../TRANSPORT.md) - Transport layer details
- [../MEMORY_MODEL.md](../MEMORY_MODEL.md) - Memory ownership
