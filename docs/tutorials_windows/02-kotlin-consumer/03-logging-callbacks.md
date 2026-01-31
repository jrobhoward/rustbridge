# Section 3: Logging Callbacks

In this section, you'll receive log messages from the plugin using callbacks.

## Understanding Plugin Logging

The plugin uses Rust's `tracing` crate for logging:

```rust
tracing::debug!("Matching pattern '{}' against text", req.pattern);
tracing::trace!("Cache hit for pattern: {}", pattern);
```

These logs can be forwarded to the host application via callbacks.

## Add Logging Support

Update `src\main\kotlin\com\example\Main.kt`:

```kotlin
package com.example

import com.rustbridge.BundleLoader
import com.rustbridge.LogCallback
import com.rustbridge.LogLevel
import com.rustbridge.ffm.FfmPluginLoader
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

private val json = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}

fun main() {
    println("=== Kotlin Consumer - Logging Demo ===\n")

    val bundlePath = "regex-plugin-1.0.0.rbp"

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary().toString()

    // Create a log callback
    val logCallback = LogCallback { level, target, message ->
        val levelStr = when (level) {
            LogLevel.TRACE -> "TRACE"
            LogLevel.DEBUG -> "DEBUG"
            LogLevel.INFO -> "INFO"
            LogLevel.WARN -> "WARN"
            LogLevel.ERROR -> "ERROR"
            else -> "???"
        }
        println("  [$levelStr] $target: $message")
    }

    // Load with logging enabled
    FfmPluginLoader.builder()
        .libraryPath(libraryPath)
        .logCallback(logCallback)
        .logLevel(LogLevel.DEBUG)  // Capture DEBUG and above
        .build()
        .use { plugin ->

            println("Making plugin calls with logging enabled:\n")

            // Call 1
            println("Call 1: Match digits")
            val resp1: MatchResponse = plugin.callTyped(
                "match",
                MatchRequest("\\d+", "abc123")
            )
            println("  Result: matched=${resp1.matched}\n")

            // Call 2 (same pattern - should be cache hit)
            println("Call 2: Match digits (cache hit expected)")
            val resp2: MatchResponse = plugin.callTyped(
                "match",
                MatchRequest("\\d+", "xyz789")
            )
            println("  Result: matched=${resp2.matched}\n")

            // Call 3 (different pattern)
            println("Call 3: Find all words")
            val resp3: FindAllResponse = plugin.callTyped(
                "find_all",
                FindAllRequest("[a-z]+", "hello world")
            )
            println("  Result: count=${resp3.count}\n")

            // Stats
            println("Cache statistics:")
            val stats: StatsResponse = plugin.callTyped("stats", StatsRequest)
            println("  Hits: ${stats.cacheHits}, Misses: ${stats.cacheMisses}")
        }

    bundleLoader.close()
    println("\n=== Demo Complete ===")
}

private inline fun <reified Req, reified Resp> com.rustbridge.Plugin.callTyped(
    typeTag: String,
    request: Req
): Resp {
    val requestJson = json.encodeToString(request)
    val responseJson = this.call(typeTag, requestJson)
    return json.decodeFromString(responseJson)
}
```

## Run the Demo

```powershell
.\gradlew.bat run
```

Expected output:

```
=== Kotlin Consumer - Logging Demo ===

Making plugin calls with logging enabled:

Call 1: Match digits
  [DEBUG] regex_plugin: Matching pattern '\d+' against text
  [TRACE] regex_plugin: Cache miss for pattern: \d+
  Result: matched=true

Call 2: Match digits (cache hit expected)
  [DEBUG] regex_plugin: Matching pattern '\d+' against text
  [TRACE] regex_plugin: Cache hit for pattern: \d+
  Result: matched=true

Call 3: Find all words
  [DEBUG] regex_plugin: Finding all matches for pattern '[a-z]+'
  [TRACE] regex_plugin: Cache miss for pattern: [a-z]+
  Result: count=2

Cache statistics:
  Hits: 1, Misses: 3

=== Demo Complete ===
```

## Log Levels

Control verbosity with `LogLevel`:

| Level | Description |
|-------|-------------|
| `TRACE` | Very detailed debugging (cache hits/misses) |
| `DEBUG` | Debug information (request details) |
| `INFO` | General operational info (startup/shutdown) |
| `WARN` | Warnings |
| `ERROR` | Errors |

```kotlin
.logLevel(LogLevel.TRACE)  // Most verbose
.logLevel(LogLevel.INFO)   // Default - skip debug details
```

## Filtering by Target

Filter logs by module:

```kotlin
val logCallback = LogCallback { level, target, message ->
    // Only show logs from regex_plugin, not rustbridge internals
    if (target.startsWith("regex_plugin")) {
        println("[$level] $message")
    }
}
```

## Production Considerations

In production, integrate with your logging framework:

```kotlin
import org.slf4j.LoggerFactory

val logger = LoggerFactory.getLogger("RustPlugin")

val logCallback = LogCallback { level, target, message ->
    when (level) {
        LogLevel.ERROR -> logger.error("[{}] {}", target, message)
        LogLevel.WARN -> logger.warn("[{}] {}", target, message)
        LogLevel.INFO -> logger.info("[{}] {}", target, message)
        LogLevel.DEBUG -> logger.debug("[{}] {}", target, message)
        LogLevel.TRACE -> logger.trace("[{}] {}", target, message)
        else -> {}
    }
}
```

## What's Next?

In the next section, you'll create a type-safe wrapper class for the regex plugin.

[Continue to Section 4: Type-Safe Calls →](./04-type-safe-calls.md)
