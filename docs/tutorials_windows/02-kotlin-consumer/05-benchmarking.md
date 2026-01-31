# Section 5: Benchmarking

In this section, you'll measure the performance improvement from the pattern cache.

## Create a Benchmark

Create `src\main\kotlin\com\example\Benchmark.kt`:

```kotlin
package com.example

import com.rustbridge.BundleLoader
import com.rustbridge.ffm.FfmPluginLoader
import kotlin.system.measureNanoTime

fun main() {
    println("=== Regex Plugin Cache Benchmark ===\n")

    val bundlePath = "regex-plugin-1.0.0.rbp"

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary().toString()
    val plugin = FfmPluginLoader.load(libraryPath)

    RegexPlugin(plugin).use { regex ->

        val patterns = listOf(
            "\\d+",
            "[a-z]+",
            "[A-Z][a-z]*",
            "\\w+@\\w+\\.\\w+",
            "https?://[^\\s]+"
        )
        val text = "Hello World 123 test@example.com https://rustbridge.dev 456 Kotlin"
        val iterations = 1000

        // Warm up
        println("Warming up...")
        repeat(100) {
            patterns.forEach { pattern ->
                regex.findAll(pattern, text)
            }
        }

        // Clear stats baseline (make a fresh call to each pattern)
        patterns.forEach { pattern ->
            regex.findAll(pattern, text)
        }

        val statsBefore = regex.stats()

        // Benchmark with cache hits
        println("Running benchmark ($iterations iterations per pattern)...\n")

        val timings = mutableMapOf<String, Long>()

        patterns.forEach { pattern ->
            val nanos = measureNanoTime {
                repeat(iterations) {
                    regex.findAll(pattern, text)
                }
            }
            timings[pattern] = nanos
        }

        // Results
        println("Results (${iterations} iterations each):")
        println("-".repeat(60))

        var totalNanos = 0L
        timings.forEach { (pattern, nanos) ->
            val avgMicros = nanos / iterations / 1000.0
            totalNanos += nanos
            println("  %-30s %8.2f µs/call".format(
                pattern.take(30),
                avgMicros
            ))
        }

        println("-".repeat(60))
        val avgMicros = totalNanos / (patterns.size * iterations) / 1000.0
        println("  %-30s %8.2f µs/call".format("Average", avgMicros))

        // Cache statistics
        val statsAfter = regex.stats()
        val newHits = statsAfter.cacheHits - statsBefore.cacheHits
        val newMisses = statsAfter.cacheMisses - statsBefore.cacheMisses
        val totalCalls = newHits + newMisses

        println("\nCache Statistics:")
        println("  Total calls: $totalCalls")
        println("  Cache hits: $newHits")
        println("  Cache misses: $newMisses")
        println("  Hit rate: ${"%.1f".format(newHits.toDouble() / totalCalls * 100)}%")

        // Throughput
        val totalSeconds = totalNanos / 1_000_000_000.0
        val callsPerSecond = (patterns.size * iterations) / totalSeconds
        println("\nThroughput: ${"%.0f".format(callsPerSecond)} calls/second")
    }

    bundleLoader.close()
    println("\n=== Benchmark Complete ===")
}
```

## Update Build Configuration

Add a separate run task for the benchmark in `build.gradle.kts`:

```kotlin
tasks.register<JavaExec>("benchmark") {
    group = "application"
    description = "Run the cache benchmark"
    mainClass.set("com.example.BenchmarkKt")
    classpath = sourceSets["main"].runtimeClasspath
}
```

## Run the Benchmark

```powershell
.\gradlew.bat benchmark
```

Expected output:

```
=== Regex Plugin Cache Benchmark ===

Warming up...
Running benchmark (1000 iterations per pattern)...

Results (1000 iterations each):
------------------------------------------------------------
  \d+                             12.34 µs/call
  [a-z]+                          11.89 µs/call
  [A-Z][a-z]*                     13.21 µs/call
  \w+@\w+\.\w+                    15.67 µs/call
  https?://[^\s]+                 14.23 µs/call
------------------------------------------------------------
  Average                         13.47 µs/call

Cache Statistics:
  Total calls: 5000
  Cache hits: 4995
  Cache misses: 5
  Hit rate: 99.9%

Throughput: 74239 calls/second

=== Benchmark Complete ===
```

## Understanding the Results

### Cache Effectiveness

With 1000 iterations per pattern:
- 5 patterns × 1 = 5 cache misses (first call compiles)
- 5 patterns × 999 = 4995 cache hits (subsequent calls reuse)

### Latency Components

Each call includes:
- FFI boundary crossing (~2 µs)
- JSON serialization (~1-2 µs)
- Regex matching (varies by pattern)
- JSON deserialization (~1-2 µs)
- FFI return (~2 µs)

### Cache Miss Cost

Without caching, each call would also include:
- Regex compilation (10-100+ µs depending on pattern complexity)

The cache provides significant speedup for repeated patterns.

## Comparing With/Without Cache

To see the cache benefit, you could:

1. Modify the plugin to disable caching
2. Re-run the benchmark
3. Compare the average call time

Typically, cached calls are 5-10x faster than uncached for complex patterns.

## Summary

You've completed the Kotlin consumer tutorial! You learned:

- Setting up a Kotlin project with rustbridge
- Making JSON-based plugin calls
- Receiving log callbacks from Rust
- Creating type-safe wrapper classes
- Benchmarking plugin performance

## What's Next?

Continue to [Chapter 3: JSON Plugin](../03-json-plugin/README.md) to build a more complex plugin with multiple message types.
