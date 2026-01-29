# Section 3: Logging Callbacks

In this section, you'll capture plugin logs in your Kotlin application.

## The Logging Flow

When the plugin calls `tracing::info!()` or `tracing::debug!()`, those messages can be forwarded to your application:

```
Plugin (Rust)                    Host (Kotlin)
─────────────────                ─────────────────
tracing::info!("started")  ───▶  logCallback(INFO, "started")
                                       │
                                       ▼
                                 println("[INFO] started")
```

## Set Up a Log Callback

Update your Main.kt:

```kotlin
package com.example

import com.rustbridge.BundleLoader
import com.rustbridge.LogLevel
import com.rustbridge.ffm.FfmPluginLoader

fun main(args: Array<String>) {
    val bundlePath = "regex-plugin-1.0.0.rbp"

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary()

    FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
        // Set up logging callback BEFORE init
        plugin.setLogCallback { level, message ->
            val levelStr = when (level) {
                LogLevel.TRACE -> "TRACE"
                LogLevel.DEBUG -> "DEBUG"
                LogLevel.INFO -> "INFO"
                LogLevel.WARN -> "WARN"
                LogLevel.ERROR -> "ERROR"
            }
            println("[PLUGIN $levelStr] $message")
        }

        // Set the minimum log level
        plugin.setLogLevel(LogLevel.DEBUG)

        // Now init - you'll see the startup message
        plugin.init()

        // Make some calls
        val request1 = """{"pattern": "\\d+", "text": "test123"}"""
        plugin.call("match", request1)

        val request2 = """{"pattern": "\\d+", "text": "456"}"""
        plugin.call("match", request2)

        // Shutdown logs the cache size
        plugin.shutdown()
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
[PLUGIN INFO] regex-plugin started cache_size=100
[PLUGIN DEBUG] Processing match request pattern=\d+ text_len=7
[PLUGIN DEBUG] Match completed pattern=\d+ matches=true cached=false
[PLUGIN DEBUG] Processing match request pattern=\d+ text_len=3
[PLUGIN DEBUG] Match completed pattern=\d+ matches=true cached=true
[PLUGIN INFO] regex-plugin stopped cached_patterns=1
```

## Log Levels

rustbridge uses standard log levels:

| Level | Use Case |
|-------|----------|
| TRACE | Very detailed debugging |
| DEBUG | Development debugging |
| INFO | Normal operational messages |
| WARN | Potential issues |
| ERROR | Errors that need attention |

Set the minimum level based on your needs:

```kotlin
// Production - only warnings and errors
plugin.setLogLevel(LogLevel.WARN)

// Development - include debug messages
plugin.setLogLevel(LogLevel.DEBUG)

// Troubleshooting - everything
plugin.setLogLevel(LogLevel.TRACE)
```

## Integrate with SLF4J

For production applications, integrate with your logging framework:

```kotlin
import org.slf4j.LoggerFactory

val pluginLogger = LoggerFactory.getLogger("regex-plugin")

plugin.setLogCallback { level, message ->
    when (level) {
        LogLevel.TRACE -> pluginLogger.trace(message)
        LogLevel.DEBUG -> pluginLogger.debug(message)
        LogLevel.INFO -> pluginLogger.info(message)
        LogLevel.WARN -> pluginLogger.warn(message)
        LogLevel.ERROR -> pluginLogger.error(message)
    }
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

The plugin uses structured logging with key=value pairs. You can parse these:

```kotlin
plugin.setLogCallback { level, message ->
    // Parse "Processing match request pattern=\d+ text_len=7"
    val parts = message.split(" ")
    val baseMessage = parts.takeWhile { !it.contains("=") }.joinToString(" ")
    val fields = parts.dropWhile { !it.contains("=") }
        .associate {
            val (k, v) = it.split("=", limit = 2)
            k to v
        }

    println("Message: $baseMessage")
    println("Fields: $fields")
}
```

## Disable Logging

To disable logging entirely:

```kotlin
plugin.setLogCallback(null)
// or
plugin.setLogLevel(LogLevel.ERROR)  // Only critical errors
```

## What's Next?

Raw JSON strings are error-prone. In the next section, you'll use Kotlin data classes for type-safe calls.

[Continue to Section 4: Type-Safe Calls →](./04-type-safe-calls.md)
