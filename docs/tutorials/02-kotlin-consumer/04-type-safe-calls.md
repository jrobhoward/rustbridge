# Section 4: Type-Safe Calls

In this section, you'll define Kotlin data classes and use extension functions for type-safe plugin calls.

## The Problem with Raw JSON

Raw JSON strings are error-prone:

```kotlin
// Easy to make typos
val request = """{"patern": "\\d+", "text": "test123"}"""  // Oops: "patern"

// No compile-time checking
val request = """{"pattern": 123, "text": "test123"}"""  // Wrong type for pattern
```

## Define Data Classes

Create data classes that match your plugin's message types:

```kotlin
package com.example

import com.fasterxml.jackson.annotation.JsonProperty

// Request type
data class MatchRequest(
    val pattern: String,
    val text: String
)

// Response type
data class MatchResponse(
    val matches: Boolean,
    val cached: Boolean
)

// Configuration type
data class PluginConfig(
    @JsonProperty("cache_size")
    val cacheSize: Int = 100
)
```

## Create a Type-Safe Extension

Add an extension function that handles serialization:

```kotlin
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue

val mapper = jacksonObjectMapper()

// Generic extension for type-safe calls
inline fun <reified T> com.rustbridge.Plugin.callTyped(
    messageType: String,
    request: Any
): T {
    val requestJson = mapper.writeValueAsString(request)
    val responseJson = call(messageType, requestJson)
    return mapper.readValue(responseJson)
}
```

## Use Type-Safe Calls

Now you can write:

```kotlin
fun main(args: Array<String>) {
    val bundlePath = "regex-plugin-1.0.0.rbp"

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary()

    FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
        plugin.setLogCallback { level, message ->
            println("[${level.name}] $message")
        }
        plugin.setLogLevel(LogLevel.INFO)

        // Type-safe configuration
        val config = PluginConfig(cacheSize = 50)
        plugin.initWithConfig(mapper.writeValueAsString(config))

        // Type-safe request/response
        val request = MatchRequest(
            pattern = """\d{4}-\d{2}-\d{2}""",
            text = "2024-01-15"
        )

        val response = plugin.callTyped<MatchResponse>("match", request)

        println("Matches: ${response.matches}")
        println("Cached: ${response.cached}")

        // Make another call with the same pattern
        val request2 = MatchRequest(
            pattern = """\d{4}-\d{2}-\d{2}""",
            text = "2024-12-25"
        )

        val response2 = plugin.callTyped<MatchResponse>("match", request2)

        println("\nSecond call:")
        println("Matches: ${response2.matches}")
        println("Cached: ${response2.cached}")  // Should be true!
    }

    bundleLoader.close()
}
```

## Complete Main.kt

Here's the complete file:

```kotlin
package com.example

import com.fasterxml.jackson.annotation.JsonProperty
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import com.rustbridge.BundleLoader
import com.rustbridge.LogLevel
import com.rustbridge.ffm.FfmPluginLoader

// Data classes matching plugin message types
data class MatchRequest(
    val pattern: String,
    val text: String
)

data class MatchResponse(
    val matches: Boolean,
    val cached: Boolean
)

data class PluginConfig(
    @JsonProperty("cache_size")
    val cacheSize: Int = 100
)

// JSON mapper
val mapper = jacksonObjectMapper()

// Type-safe extension
inline fun <reified T> com.rustbridge.Plugin.callTyped(
    messageType: String,
    request: Any
): T {
    val requestJson = mapper.writeValueAsString(request)
    val responseJson = call(messageType, requestJson)
    return mapper.readValue(responseJson)
}

fun main(args: Array<String>) {
    val bundlePath = "regex-plugin-1.0.0.rbp"

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary()

    FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
        // Logging
        plugin.setLogCallback { level, message ->
            println("[${level.name}] $message")
        }
        plugin.setLogLevel(LogLevel.INFO)

        // Configure with smaller cache
        val config = PluginConfig(cacheSize = 50)
        plugin.initWithConfig(mapper.writeValueAsString(config))

        // Test some patterns
        val patterns = listOf(
            """\d+""" to "test123",           // Digits
            """^[a-z]+$""" to "hello",        // Lowercase letters
            """^\d{4}-\d{2}-\d{2}$""" to "2024-01-15",  // Date
        )

        println("\n=== Pattern Matching ===")
        for ((pattern, text) in patterns) {
            val request = MatchRequest(pattern, text)
            val response = plugin.callTyped<MatchResponse>("match", request)
            println("'$text' matches '$pattern': ${response.matches}")
        }

        // Demonstrate caching
        println("\n=== Cache Demo ===")
        val datePattern = """\d{4}-\d{2}-\d{2}"""
        val dates = listOf("2024-01-15", "2024-06-01", "2024-12-25")

        for (date in dates) {
            val request = MatchRequest(datePattern, date)
            val response = plugin.callTyped<MatchResponse>("match", request)
            println("$date: matches=${response.matches}, cached=${response.cached}")
        }
    }

    bundleLoader.close()
}
```

## Run It

```bash
./gradlew run
```

Output:

```
[INFO] Creating regex plugin with custom configuration cache_size=50
[INFO] regex-plugin started cache_size=50

=== Pattern Matching ===
'test123' matches '\d+': true
'hello' matches '^[a-z]+$': true
'2024-01-15' matches '^\d{4}-\d{2}-\d{2}$': true

=== Cache Demo ===
2024-01-15: matches=true, cached=false
2024-06-01: matches=true, cached=true
2024-12-25: matches=true, cached=true
[INFO] regex-plugin stopped cached_patterns=4
```

## Error Handling

Type-safe calls also give better error handling:

```kotlin
try {
    val response = plugin.callTyped<MatchResponse>("match", request)
} catch (e: com.rustbridge.PluginException) {
    // Plugin returned an error (e.g., invalid regex)
    println("Plugin error: ${e.message}")
} catch (e: com.fasterxml.jackson.core.JsonProcessingException) {
    // JSON serialization/deserialization failed
    println("JSON error: ${e.message}")
}
```

## What's Next?

In the final section, you'll benchmark performance to understand debug vs release builds and cache effectiveness.

[Continue to Section 5: Benchmarking →](./05-benchmarking.md)
