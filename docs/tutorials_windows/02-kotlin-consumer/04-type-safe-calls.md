# Section 4: Type-Safe Calls

In this section, you'll create a wrapper class that provides a type-safe Kotlin API for the regex plugin.

## Create the Wrapper Class

Create `src\main\kotlin\com\example\RegexPlugin.kt`:

```kotlin
package com.example

import com.rustbridge.Plugin
import com.rustbridge.PluginException
import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import java.io.Closeable

/**
 * Type-safe wrapper for the regex plugin.
 *
 * Provides idiomatic Kotlin API with proper error handling.
 */
class RegexPlugin(private val plugin: Plugin) : Closeable {

    private val json = Json {
        ignoreUnknownKeys = true
        encodeDefaults = true
    }

    /**
     * Match a pattern against text.
     *
     * @param pattern The regex pattern
     * @param text The text to search
     * @return Match result with position information
     * @throws RegexPluginException on invalid pattern or plugin error
     */
    fun match(pattern: String, text: String): MatchResult {
        val request = MatchRequest(pattern, text)
        val response = callTyped<MatchRequest, MatchResponse>("match", request)

        return if (response.matched) {
            MatchResult.Found(
                text = response.matchText!!,
                range = response.start!!..<response.end!!
            )
        } else {
            MatchResult.NotFound
        }
    }

    /**
     * Find all matches of a pattern in text.
     *
     * @param pattern The regex pattern
     * @param text The text to search
     * @return List of all matched strings
     * @throws RegexPluginException on invalid pattern or plugin error
     */
    fun findAll(pattern: String, text: String): List<String> {
        val request = FindAllRequest(pattern, text)
        val response = callTyped<FindAllRequest, FindAllResponse>("find_all", request)
        return response.matches
    }

    /**
     * Check if a pattern matches anywhere in text.
     *
     * @param pattern The regex pattern
     * @param text The text to search
     * @return true if pattern matches
     */
    fun matches(pattern: String, text: String): Boolean {
        return match(pattern, text) is MatchResult.Found
    }

    /**
     * Get cache statistics.
     */
    fun stats(): CacheStats {
        val response = callTyped<StatsRequest, StatsResponse>("stats", StatsRequest)
        return CacheStats(
            cachedPatterns = response.cachedPatterns,
            cacheCapacity = response.cacheCapacity,
            totalRequests = response.totalRequests,
            cacheHits = response.cacheHits,
            cacheMisses = response.cacheMisses
        )
    }

    override fun close() {
        plugin.close()
    }

    private inline fun <reified Req, reified Resp> callTyped(
        typeTag: String,
        request: Req
    ): Resp {
        try {
            val requestJson = json.encodeToString(request)
            val responseJson = plugin.call(typeTag, requestJson)
            return json.decodeFromString(responseJson)
        } catch (e: PluginException) {
            throw RegexPluginException(e.message ?: "Plugin call failed", e)
        }
    }
}

/**
 * Result of a match operation.
 */
sealed class MatchResult {
    /** Pattern matched at the given range. */
    data class Found(
        val text: String,
        val range: IntRange
    ) : MatchResult()

    /** Pattern did not match. */
    data object NotFound : MatchResult()
}

/**
 * Cache statistics.
 */
data class CacheStats(
    val cachedPatterns: Int,
    val cacheCapacity: Int,
    val totalRequests: Long,
    val cacheHits: Long,
    val cacheMisses: Long
) {
    val hitRate: Double
        get() = if (totalRequests > 0) {
            cacheHits.toDouble() / totalRequests * 100
        } else {
            0.0
        }
}

/**
 * Exception thrown by RegexPlugin operations.
 */
class RegexPluginException(
    message: String,
    cause: Throwable? = null
) : Exception(message, cause)
```

## Update Main to Use the Wrapper

Replace `src\main\kotlin\com\example\Main.kt`:

```kotlin
package com.example

import com.rustbridge.BundleLoader
import com.rustbridge.ffm.FfmPluginLoader

fun main() {
    println("=== Kotlin Consumer - Type-Safe API Demo ===\n")

    val bundlePath = "regex-plugin-1.0.0.rbp"

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary().toString()
    val plugin = FfmPluginLoader.load(libraryPath)

    // Use the type-safe wrapper
    RegexPlugin(plugin).use { regex ->

        // Demo 1: Simple match with sealed class result
        println("Demo 1: Match with sealed class result")
        when (val result = regex.match("\\d+", "abc123def")) {
            is MatchResult.Found -> {
                println("  Found '${result.text}' at ${result.range}")
            }
            is MatchResult.NotFound -> {
                println("  No match")
            }
        }

        // Demo 2: Convenient matches() check
        println("\nDemo 2: Boolean match check")
        val hasDigits = regex.matches("\\d", "hello123")
        val hasUppercase = regex.matches("[A-Z]", "hello")
        println("  Contains digits: $hasDigits")
        println("  Contains uppercase: $hasUppercase")

        // Demo 3: Find all with list result
        println("\nDemo 3: Find all matches")
        val words = regex.findAll("[a-zA-Z]+", "Hello, World! How are you?")
        println("  Words found: $words")

        // Demo 4: Email validation pattern
        println("\nDemo 4: Email validation")
        val emailPattern = "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"
        val testEmails = listOf(
            "valid@example.com",
            "invalid-email",
            "another.valid@test.org"
        )
        testEmails.forEach { email ->
            val isValid = regex.matches(emailPattern, email)
            println("  $email: ${if (isValid) "valid" else "invalid"}")
        }

        // Demo 5: Error handling
        println("\nDemo 5: Error handling")
        try {
            regex.match("[invalid(", "test")
        } catch (e: RegexPluginException) {
            println("  Caught expected error: ${e.message?.take(50)}...")
        }

        // Demo 6: Cache statistics
        println("\nDemo 6: Cache statistics")
        val stats = regex.stats()
        println("  Patterns cached: ${stats.cachedPatterns}/${stats.cacheCapacity}")
        println("  Hit rate: ${"%.1f".format(stats.hitRate)}%")
    }

    bundleLoader.close()
    println("\n=== Demo Complete ===")
}
```

## Run the Demo

```powershell
.\gradlew.bat run
```

Expected output:

```
=== Kotlin Consumer - Type-Safe API Demo ===

Demo 1: Match with sealed class result
  Found '123' at 3..5

Demo 2: Boolean match check
  Contains digits: true
  Contains uppercase: false

Demo 3: Find all matches
  Words found: [Hello, World, How, are, you]

Demo 4: Email validation
  valid@example.com: valid
  invalid-email: invalid
  another.valid@test.org: valid

Demo 5: Error handling
  Caught expected error: Invalid regex: regex parse error...

Demo 6: Cache statistics
  Patterns cached: 3/100
  Hit rate: 25.0%

=== Demo Complete ===
```

## Understanding the Design

### Sealed Class for Results

Kotlin sealed classes model the match/no-match outcome:

```kotlin
sealed class MatchResult {
    data class Found(val text: String, val range: IntRange) : MatchResult()
    data object NotFound : MatchResult()
}
```

This allows exhaustive `when` expressions without `else`.

### Extension-Friendly Design

The wrapper takes a `Plugin` instance, allowing composition:

```kotlin
class RegexPlugin(private val plugin: Plugin) : Closeable
```

### Exception Wrapping

Plugin exceptions are wrapped in domain-specific exceptions:

```kotlin
catch (e: PluginException) {
    throw RegexPluginException(e.message ?: "Plugin call failed", e)
}
```

## What's Next?

In the next section, you'll benchmark the cache performance.

[Continue to Section 5: Benchmarking →](./05-benchmarking.md)
