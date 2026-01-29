# Section 5: Benchmarking

In this section, you'll measure plugin performance to understand cache effectiveness and the difference between debug and release builds.

## Debug vs Release Builds

Rust has two main build profiles:

| Profile | Command | Optimizations | Use Case |
|---------|---------|---------------|----------|
| Debug | `cargo build` | None | Development |
| Release | `cargo build --release` | Full | Production |

Release builds are typically **10-100x faster** for compute-heavy operations.

## Create a Benchmark

Add a benchmark to your Kotlin code:

```kotlin
fun benchmark(plugin: com.rustbridge.Plugin) {
    val patterns = listOf(
        """\d{4}-\d{2}-\d{2}""",  // Date
        """^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$""",  // Email
        """^https?://[^\s/$.?#].[^\s]*$""",  // URL
        """^\+?[1-9]\d{1,14}$""",  // Phone
    )

    val samples = listOf(
        "2024-01-15",
        "user@example.com",
        "https://github.com",
        "+1234567890",
    )

    val iterations = 10_000

    // Warm up the cache
    println("Warming up cache...")
    for ((pattern, sample) in patterns.zip(samples)) {
        val request = MatchRequest(pattern, sample)
        plugin.callTyped<MatchResponse>("match", request)
    }

    // Benchmark with cache hits
    println("Benchmarking $iterations iterations...")
    val start = System.nanoTime()

    for (i in 0 until iterations) {
        for ((pattern, sample) in patterns.zip(samples)) {
            val request = MatchRequest(pattern, sample)
            plugin.callTyped<MatchResponse>("match", request)
        }
    }

    val elapsed = System.nanoTime() - start
    val totalCalls = iterations * patterns.size
    val avgNanos = elapsed / totalCalls

    println("\n=== Benchmark Results ===")
    println("Total calls: $totalCalls")
    println("Total time: ${elapsed / 1_000_000} ms")
    println("Avg per call: $avgNanos ns (${avgNanos / 1000} µs)")
    println("Calls/second: ${1_000_000_000L * totalCalls / elapsed}")
}
```

## Compare Debug vs Release

First, build and benchmark with a debug build:

```bash
cd ~/rustbridge-workspace/regex-plugin

# Debug build
cargo build

# Create debug bundle
rustbridge bundle create \
  --name regex-plugin \
  --version 1.0.0-debug \
  --lib linux-x86_64:target/debug/libregex_plugin.so \
  --output ~/rustbridge-workspace/regex-kotlin-app/regex-plugin-debug.rbp
```

Then release:

```bash
# Release build
cargo build --release

# Create release bundle
rustbridge bundle create \
  --name regex-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libregex_plugin.so \
  --output ~/rustbridge-workspace/regex-kotlin-app/regex-plugin-release.rbp
```

Update Main.kt to run both:

```kotlin
fun main(args: Array<String>) {
    println("=== Debug Build ===")
    runBenchmark("regex-plugin-debug.rbp")

    println("\n=== Release Build ===")
    runBenchmark("regex-plugin-release.rbp")
}

fun runBenchmark(bundlePath: String) {
    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary()

    FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
        plugin.setLogLevel(LogLevel.WARN)  // Reduce logging noise
        plugin.init()
        benchmark(plugin)
    }

    bundleLoader.close()
}
```

Typical results:

```
=== Debug Build ===
Avg per call: 15000 ns (15 µs)
Calls/second: 66,666

=== Release Build ===
Avg per call: 800 ns (0.8 µs)
Calls/second: 1,250,000
```

Release is ~20x faster in this example!

## Measure Cache Effectiveness

Compare performance with and without cache hits:

```kotlin
fun cacheEffectivenessBenchmark(plugin: com.rustbridge.Plugin) {
    val pattern = """^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"""
    val sample = "user@example.com"
    val iterations = 10_000

    // Without cache - use unique patterns each time
    println("Without cache (compiling each time)...")
    val noCacheStart = System.nanoTime()
    for (i in 0 until iterations) {
        // Add a unique suffix to force recompilation
        val uniquePattern = "$pattern$i"
        val request = MatchRequest(uniquePattern, sample)
        try {
            plugin.callTyped<MatchResponse>("match", request)
        } catch (e: Exception) {
            // Invalid patterns will error, that's fine
        }
    }
    val noCacheTime = System.nanoTime() - noCacheStart

    // With cache - same pattern each time
    println("With cache (reusing compiled pattern)...")

    // First call to populate cache
    plugin.callTyped<MatchResponse>("match", MatchRequest(pattern, sample))

    val cacheStart = System.nanoTime()
    for (i in 0 until iterations) {
        val request = MatchRequest(pattern, sample)
        plugin.callTyped<MatchResponse>("match", request)
    }
    val cacheTime = System.nanoTime() - cacheStart

    println("\n=== Cache Effectiveness ===")
    println("Without cache: ${noCacheTime / 1_000_000} ms")
    println("With cache: ${cacheTime / 1_000_000} ms")
    println("Speedup: ${noCacheTime.toDouble() / cacheTime}x")
}
```

Typical results:

```
=== Cache Effectiveness ===
Without cache: 450 ms
With cache: 35 ms
Speedup: 12.8x
```

## Profile FFI Overhead

The FFI boundary (Kotlin → Rust) has some overhead. Measure it:

```kotlin
fun ffiOverheadBenchmark(plugin: com.rustbridge.Plugin) {
    // Minimal work - just parse JSON and return
    val request = MatchRequest("a", "a")
    val iterations = 100_000

    val start = System.nanoTime()
    for (i in 0 until iterations) {
        plugin.callTyped<MatchResponse>("match", request)
    }
    val elapsed = System.nanoTime() - start

    val avgNanos = elapsed / iterations
    println("\n=== FFI Overhead ===")
    println("Avg roundtrip: $avgNanos ns (${avgNanos / 1000} µs)")
}
```

Typical FFI overhead is 500-2000 ns per call, depending on payload size.

## Optimization Tips

1. **Always use release builds** in production
2. **Batch operations** when possible to reduce FFI crossings
3. **Use binary transport** for large payloads (7x faster than JSON)
4. **Size your cache appropriately** based on your pattern diversity
5. **Disable debug logging** in production (`LogLevel.WARN` or higher)

## Complete Benchmark Main.kt

```kotlin
package com.example

import com.fasterxml.jackson.annotation.JsonProperty
import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import com.rustbridge.BundleLoader
import com.rustbridge.LogLevel
import com.rustbridge.ffm.FfmPluginLoader

data class MatchRequest(val pattern: String, val text: String)
data class MatchResponse(val matches: Boolean, val cached: Boolean)

val mapper = jacksonObjectMapper()

inline fun <reified T> com.rustbridge.Plugin.callTyped(
    messageType: String,
    request: Any
): T {
    val requestJson = mapper.writeValueAsString(request)
    val responseJson = call(messageType, requestJson)
    return mapper.readValue(responseJson)
}

fun main(args: Array<String>) {
    val bundlePath = args.getOrElse(0) { "regex-plugin-1.0.0.rbp" }

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val libraryPath = bundleLoader.extractLibrary()

    FfmPluginLoader.load(libraryPath.toString()).use { plugin ->
        plugin.setLogLevel(LogLevel.WARN)
        plugin.init()

        benchmark(plugin)
    }

    bundleLoader.close()
}

fun benchmark(plugin: com.rustbridge.Plugin) {
    val patterns = listOf(
        """\d{4}-\d{2}-\d{2}""",
        """^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$""",
        """^https?://[^\s/$.?#].[^\s]*$""",
        """^\+?[1-9]\d{1,14}$""",
    )
    val samples = listOf("2024-01-15", "user@example.com", "https://github.com", "+1234567890")
    val iterations = 10_000

    // Warm up
    for ((pattern, sample) in patterns.zip(samples)) {
        plugin.callTyped<MatchResponse>("match", MatchRequest(pattern, sample))
    }

    // Benchmark
    val start = System.nanoTime()
    for (i in 0 until iterations) {
        for ((pattern, sample) in patterns.zip(samples)) {
            plugin.callTyped<MatchResponse>("match", MatchRequest(pattern, sample))
        }
    }
    val elapsed = System.nanoTime() - start

    val totalCalls = iterations * patterns.size
    val avgNanos = elapsed / totalCalls

    println("=== Benchmark Results ===")
    println("Total calls: $totalCalls")
    println("Total time: ${elapsed / 1_000_000} ms")
    println("Avg per call: $avgNanos ns (${avgNanos / 1000} µs)")
    println("Calls/second: ${1_000_000_000L * totalCalls / elapsed}")
}
```

Run with:

```bash
./gradlew run --args="regex-plugin-1.0.0.rbp"
```

## Summary

You've now completed the tutorial! You've learned:

1. **Chapter 1**: Build a Rust plugin with regex matching, LRU caching, configuration, and logging
2. **Chapter 2**: Call it from Kotlin with type-safe wrappers, logging callbacks, and benchmarking

## When to Optimize

JSON transport is fast enough for most use cases. Before optimizing:

1. **Profile first** - Measure actual bottlenecks in your application
2. **Check your workload** - Is the plugin the bottleneck, or is it I/O, database, etc.?
3. **Consider caching** - As shown above, caching can provide 10x+ speedups

The FFI overhead (~1-2µs per call) is negligible compared to most business logic. Premature optimization often adds complexity without meaningful benefit.

## Next Steps

- **Sign your bundles** for production deployment
- **Add more message types** to your plugin
- **Read the architecture docs** for deeper understanding

See the [rustbridge documentation](../../README.md) for more advanced topics.
