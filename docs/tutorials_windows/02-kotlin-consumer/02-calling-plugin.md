# Section 2: Calling the Plugin

In this section, you'll make structured calls to the regex plugin using JSON serialization.

## Define Message Types

Create `src\main\kotlin\com\example\Messages.kt`:

```kotlin
package com.example

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

// ============================================================================
// Match Message
// ============================================================================

@Serializable
data class MatchRequest(
    val pattern: String,
    val text: String
)

@Serializable
data class MatchResponse(
    val matched: Boolean,
    @SerialName("match_text")
    val matchText: String? = null,
    val start: Int? = null,
    val end: Int? = null
)

// ============================================================================
// Find All Message
// ============================================================================

@Serializable
data class FindAllRequest(
    val pattern: String,
    val text: String
)

@Serializable
data class FindAllResponse(
    val matches: List<String>,
    val count: Int
)

// ============================================================================
// Stats Message
// ============================================================================

@Serializable
object StatsRequest

@Serializable
data class StatsResponse(
    @SerialName("cached_patterns")
    val cachedPatterns: Int,
    @SerialName("cache_capacity")
    val cacheCapacity: Int,
    @SerialName("total_requests")
    val totalRequests: Long,
    @SerialName("cache_hits")
    val cacheHits: Long,
    @SerialName("cache_misses")
    val cacheMisses: Long
) {
    val hitRate: Double
        get() = if (totalRequests > 0) {
            cacheHits.toDouble() / totalRequests * 100
        } else {
            0.0
        }
}
```

## Update Main

Replace `src\main\kotlin\com\example\Main.kt`:

```kotlin
package com.example

import com.rustbridge.BundleLoader
import com.rustbridge.ffm.FfmPluginLoader
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json

private val json = Json {
    ignoreUnknownKeys = true
    encodeDefaults = true
}

fun main() {
    println("=== Kotlin Consumer - Regex Plugin Demo ===\n")

    val bundlePath = "regex-plugin-1.0.0.rbp"

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary().toString()

    FfmPluginLoader.load(libraryPath).use { plugin ->

        // Demo 1: Simple match
        println("Demo 1: Simple match")
        val matchReq = MatchRequest(
            pattern = "\\d+",
            text = "abc123def456"
        )
        val matchResp: MatchResponse = plugin.callTyped("match", matchReq)
        println("  Pattern: ${matchReq.pattern}")
        println("  Text: ${matchReq.text}")
        println("  Matched: ${matchResp.matched}")
        println("  Match: ${matchResp.matchText} at [${matchResp.start}, ${matchResp.end})")

        // Demo 2: Find all matches
        println("\nDemo 2: Find all matches")
        val findReq = FindAllRequest(
            pattern = "[A-Z][a-z]+",
            text = "Hello World from Kotlin"
        )
        val findResp: FindAllResponse = plugin.callTyped("find_all", findReq)
        println("  Pattern: ${findReq.pattern}")
        println("  Text: ${findReq.text}")
        println("  Found ${findResp.count} matches: ${findResp.matches}")

        // Demo 3: Email extraction
        println("\nDemo 3: Email extraction")
        val emailReq = FindAllRequest(
            pattern = "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}",
            text = "Contact us at hello@example.com or support@rustbridge.dev"
        )
        val emailResp: FindAllResponse = plugin.callTyped("find_all", emailReq)
        println("  Found ${emailResp.count} emails:")
        emailResp.matches.forEach { println("    - $it") }

        // Demo 4: No match case
        println("\nDemo 4: No match case")
        val noMatchReq = MatchRequest(
            pattern = "xyz",
            text = "abc123"
        )
        val noMatchResp: MatchResponse = plugin.callTyped("match", noMatchReq)
        println("  Pattern: ${noMatchReq.pattern}")
        println("  Matched: ${noMatchResp.matched}")

        // Demo 5: Cache statistics
        println("\nDemo 5: Cache statistics")
        val statsResp: StatsResponse = plugin.callTyped("stats", StatsRequest)
        println("  Cached patterns: ${statsResp.cachedPatterns}/${statsResp.cacheCapacity}")
        println("  Requests: ${statsResp.totalRequests}")
        println("  Hits: ${statsResp.cacheHits}, Misses: ${statsResp.cacheMisses}")
        println("  Hit rate: ${"%.1f".format(statsResp.hitRate)}%")
    }

    bundleLoader.close()
    println("\n=== Demo Complete ===")
}

// Extension function for typed calls
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
=== Kotlin Consumer - Regex Plugin Demo ===

Demo 1: Simple match
  Pattern: \d+
  Text: abc123def456
  Matched: true
  Match: 123 at [3, 6)

Demo 2: Find all matches
  Pattern: [A-Z][a-z]+
  Text: Hello World from Kotlin
  Found 4 matches: [Hello, World, Kotlin]

Demo 3: Email extraction
  Found 2 emails:
    - hello@example.com
    - support@rustbridge.dev

Demo 4: No match case
  Pattern: xyz
  Matched: false

Demo 5: Cache statistics
  Cached patterns: 4/100
  Requests: 5
  Hits: 0, Misses: 5
  Hit rate: 0.0%

=== Demo Complete ===
```

## Understanding the Code

### SerialName Annotation

Map Kotlin camelCase to Rust snake_case:

```kotlin
@SerialName("match_text")
val matchText: String? = null
```

### Extension Function

The `callTyped` extension provides type-safe calls:

```kotlin
private inline fun <reified Req, reified Resp> Plugin.callTyped(
    typeTag: String,
    request: Req
): Resp {
    val requestJson = json.encodeToString(request)
    val responseJson = this.call(typeTag, requestJson)
    return json.decodeFromString(responseJson)
}
```

### Nullable Fields

Optional fields use Kotlin's null safety:

```kotlin
val matchText: String? = null
```

## What's Next?

In the next section, you'll add logging callbacks to receive debug output from the plugin.

[Continue to Section 3: Logging Callbacks →](./03-logging-callbacks.md)
