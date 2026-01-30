# Section 3: Logging Callbacks

In this section, you'll capture plugin logs in your Kotlin application.

## The Logging Flow

When the plugin calls `tracing::info!()` or `tracing::debug!()`, those messages can be returned back to your
application through a function callback:

```
Plugin (Rust)                    Host (Kotlin)
─────────────────                ─────────────────
tracing::info!("started")  ───▶  logCallback(INFO, "regex_plugin", "started")
                                       │
                                       ▼
                                 println("[INFO] regex_plugin: started")
```

## Set Up a Log Callback

Update your Main.kt to pass a log callback when loading the plugin:

```kotlin
package com.example

import com.rustbridge.BundleLoader
import com.rustbridge.LogCallback
import com.rustbridge.LogLevel
import com.rustbridge.PluginConfig
import com.rustbridge.ffm.FfmPluginLoader
import java.nio.file.Path

fun main(args: Array<String>) {
    val bundlePath = "regex-plugin-1.0.0.rbp"

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary()

    // Create a log callback
    val logCallback = LogCallback { level, target, message ->
        println("[$level] $target: $message")
    }

    // Create config with DEBUG level
    val config = PluginConfig.defaults()
        .logLevel(LogLevel.DEBUG)

    // Load plugin with config and callback
    FfmPluginLoader.load(Path.of(libraryPath.toString()), config, logCallback).use { plugin ->
        // Make some calls - you'll see debug logs
        val request1 = """{"pattern": "\\d+", "text": "test123"}"""
        val response1 = plugin.call("match", request1)
        println("Response: $response1\n")

        val request2 = """{"pattern": "\\d+", "text": "456"}"""
        val response2 = plugin.call("match", request2)
        println("Response: $response2\n")
    }

    bundleLoader.close()
}
```

## Run and See Logs

```bash
./gradlew run
```

Output:

```
[INFO] regex_plugin: regex-plugin started cache_size=100
[INFO] rustbridge_ffi::handle: Plugin started successfully
[DEBUG] regex_plugin: Match completed pattern=\d+ matches=true cached=false
Response: {"cached":false,"matches":true}

[DEBUG] regex_plugin: Match completed pattern=\d+ matches=true cached=true
Response: {"cached":true,"matches":true}

[INFO] regex_plugin: regex-plugin stopped cached_patterns=1
[INFO] rustbridge_runtime::runtime: Initiating runtime shutdown
[INFO] rustbridge_runtime::runtime: Runtime shutdown complete
[INFO] rustbridge_ffi::handle: Plugin shutdown complete
```

Notice how the second call shows `cached=true`!

## Log Levels

rustbridge uses standard log levels:

| Level | Use Case                    |
|-------|-----------------------------|
| TRACE | Very detailed debugging     |
| DEBUG | Development debugging       |
| INFO  | Normal operational messages |
| WARN  | Potential issues            |
| ERROR | Errors that need attention  |

Set the minimum level in the config:

```kotlin
// Production - only warnings and errors
val config = PluginConfig.defaults()
    .logLevel(LogLevel.WARN)

// Development - include debug messages
val config = PluginConfig.defaults()
    .logLevel(LogLevel.DEBUG)

// Troubleshooting - everything
val config = PluginConfig.defaults()
    .logLevel(LogLevel.TRACE)
```

You can also change the log level at runtime:

```kotlin
plugin.setLogLevel(LogLevel.TRACE)
```

## Integrate with SLF4J

For production applications, integrate with your logging framework:

```kotlin
import org.slf4j.LoggerFactory

val logCallback = LogCallback { level, target, message ->
    val logger = LoggerFactory.getLogger(target)
    when (level) {
        LogLevel.TRACE -> logger.trace(message)
        LogLevel.DEBUG -> logger.debug(message)
        LogLevel.INFO -> logger.info(message)
        LogLevel.WARN -> logger.warn(message)
        LogLevel.ERROR -> logger.error(message)
    }
}

FfmPluginLoader.load(Path.of(libraryPath.toString()), config, logCallback).use { plugin ->
    // Now plugin logs go through SLF4J
}
```

Add SLF4J to `build.gradle.kts`:

```kotlin
dependencies {
    // ... existing dependencies ...
    implementation("org.slf4j:slf4j-api:2.0.9")
    runtimeOnly("ch.qos.logback:logback-classic:1.4.14")
}
```

## Structured Logging

The plugin uses structured logging with key=value pairs. The `target` parameter contains the rust module's path (package
level):

```kotlin
val logCallback = LogCallback { level, target, message ->
    println("[$level] $target")

    // Parse "Processing match request pattern=\d+ text_len=7"
    val parts = message.split(" ")
    val baseMessage = parts.takeWhile { !it.contains("=") }.joinToString(" ")
    val fields = parts.dropWhile { !it.contains("=") }
        .associate {
            val (k, v) = it.split("=", limit = 2)
            k to v
        }

    println("  Message: $baseMessage")
    if (fields.isNotEmpty()) {
        println("  Fields: $fields")
    }
}
```

## What's Next?

Raw JSON strings are error-prone. In the next section, you'll use Kotlin data classes for type-safe calls.

[Continue to Section 4: Type-Safe Calls →](./04-type-safe-calls.md)
