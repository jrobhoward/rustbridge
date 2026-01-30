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
Extracted library: /tmp/rustbridge-7070899839138620082/libregex_plugin.so

Request: {"pattern": "\\d+", "text": "test123"}
Response: {"cached":false,"matches":true}
```

## Make Multiple Calls

Let's see the cache in action. Replace the `FfmPluginLoader.load()` call:

```kotlin
FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
    // First call - compiles the pattern
    println("\nFirst call:")
    var request = """{"pattern": "^\\d{4}-\\d{2}-\\d{2}$", "text": "2024-01-15"}"""
    var response = plugin.call("match", request)
    println("Response: $response")

    // Second call - uses cached pattern
    println("\nSecond call (same pattern):")
    request = """{"pattern": "^\\d{4}-\\d{2}-\\d{2}$", "text": "2024-12-25"}"""
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

Notice how `"cached": true` appears on the second call.

If you run into problems, check that you didn't run into JSON backslash escaping problems. Verify the number of
backslashes in `var request`'s value matches what's on this page.

## Handle Errors

What happens with an invalid regex?
Try commenting out the original `var request` and replace it with this line

```kotlin
// Invalid regex pattern
var request = """{"pattern": "[invalid", "text": "test"}"""
```

Output:

```
Exception in thread "main" com.rustbridge.PluginException: {"status":"error","error_code":7,"error_message":"handler error: Invalid regex pattern: regex parse error:\n    [invalid\n    ^\nerror: unclosed character class"}
        at com.rustbridge.ffm.FfmPlugin.parseResultBuffer(FfmPlugin.java:512)
        at com.rustbridge.ffm.FfmPlugin.call(FfmPlugin.java:111)
        at com.example.MainKt.main(Main.kt:25)
```

Remove the invalid `var request` line and uncomment the original.

## Pass Configuration

To configure the cache size, pass a `PluginConfig` when loading:

```kotlin
import com.rustbridge.PluginConfig

// Create config with custom cache size
val config = PluginConfig.defaults()
    .set("cache_size", 10)

// Load plugin with config
FfmPluginLoader.load(libraryPath.toString(), config).use { plugin ->
    // Now make calls...
    val request = """{"pattern": "\\d+", "text": "test123"}"""
    val response = plugin.call("match", request)
    println("Response: $response")
}
```

## What's Next?

The raw JSON calls work, but they're error-prone when you try to escape regex patterns into JSON. We'll address that in
section 4 of this tutorial. In the next section, you'll add logging callbacks to see what's happening inside the plugin.

[Continue to Section 3: Logging Callbacks →](./03-logging-callbacks.md)
