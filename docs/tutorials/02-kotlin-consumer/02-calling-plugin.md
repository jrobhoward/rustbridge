# Section 2: Calling the Plugin

In this section, you'll load the regex plugin bundle and make JSON calls.

## Update Main.kt

Replace the contents of `src/main/kotlin/com/example/Main.kt`:

```kotlin
package com.example

import com.rustbridge.BundleLoader
import com.rustbridge.ffm.FfmPluginLoader

fun main(args: Array<String>) {
    // Path to your plugin bundle
    val bundlePath = "regex-plugin-1.0.0.rbp"

    // Load the bundle and extract the library for this platform
    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)  // Set true in production with signed bundles
        .build()

    val libraryPath = bundleLoader.extractLibrary()
    println("Extracted library: $libraryPath")

    // Load the plugin
    FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
        // Make a raw JSON call
        val requestJson = """{"pattern": "\\d+", "text": "test123"}"""
        println("\nRequest: $requestJson")

        val responseJson = plugin.call("match", requestJson)
        println("Response: $responseJson")
    }

    bundleLoader.close()
}
```

## Run the Application

```bash
./gradlew run
```

Output:

```
Extracted library: /tmp/rustbridge-bundles/regex-plugin/1.0.0/linux-x86_64/libregex_plugin.so
Request: {"pattern": "\\d+", "text": "test123"}
Response: {"matches":true,"cached":false}
```

## Make Multiple Calls

Let's see the cache in action:

```kotlin
FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
    // First call - compiles the pattern
    val pattern = """^\d{4}-\d{2}-\d{2}$"""

    println("\nFirst call:")
    var request = """{"pattern": "$pattern", "text": "2024-01-15"}"""
    var response = plugin.call("match", request)
    println("Response: $response")

    // Second call - uses cached pattern
    println("\nSecond call (same pattern):")
    request = """{"pattern": "$pattern", "text": "2024-12-25"}"""
    response = plugin.call("match", request)
    println("Response: $response")

    // Third call - different pattern
    println("\nThird call (different pattern):")
    request = """{"pattern": "[a-z]+", "text": "hello"}"""
    response = plugin.call("match", request)
    println("Response: $response")
}
```

Output:

```
First call:
Response: {"matches":true,"cached":false}

Second call (same pattern):
Response: {"matches":true,"cached":true}

Third call (different pattern):
Response: {"matches":true,"cached":false}
```

Notice how `"cached": true` appears on the second call!

## Handle Errors

What happens with an invalid regex?

```kotlin
// Invalid regex pattern
println("\nInvalid pattern:")
val request = """{"pattern": "[invalid", "text": "test"}"""
try {
    val response = plugin.call("match", request)
    println("Response: $response")
} catch (e: Exception) {
    println("Error: ${e.message}")
}
```

Output:

```
Invalid pattern:
Error: Handler error: Invalid regex pattern: regex parse error: ...
```

## Pass Configuration

To configure the cache size, use `initWithConfig`:

```kotlin
FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
    // Configure with a smaller cache
    val config = """{"cache_size": 10}"""
    plugin.initWithConfig(config)

    // Now make calls...
}
```

Or using the Kotlin DSL:

```kotlin
FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
    plugin.initWithConfig {
        put("cache_size", 10)
    }

    // Now make calls...
}
```

## What's Next?

The raw JSON calls work, but they're error-prone. In the next section, you'll add logging callbacks to see what's happening inside the plugin.

[Continue to Section 3: Logging Callbacks →](./03-logging-callbacks.md)
