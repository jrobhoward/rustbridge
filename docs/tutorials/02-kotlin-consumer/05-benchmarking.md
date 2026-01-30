# Section 5: Benchmarking

In this section, you'll measure plugin performance to understand cache effectiveness and the difference between debug
and release builds.

## Debug vs Release Builds

Rust has two main build profiles:

| Profile | Command                 | Optimizations | Use Case    |
|---------|-------------------------|---------------|-------------|
| Debug   | `cargo build`           | None          | Development |
| Release | `cargo build --release` | Full          | Production  |

Release builds are typically **10-100x faster** for compute-heavy operations.

## Create a Bundle with Both Variants

Instead of creating separate bundles, we can package both debug and release builds in a single `.rbp` file:

```bash
cd ~/rustbridge-workspace/regex-plugin

# Build both variants
cargo build                # Debug
cargo build --release      # Release

# Create bundle with both variants
rustbridge bundle create \
  --name regex-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libregex_plugin.so \
  --lib linux-x86_64:debug:target/debug/libregex_plugin.so \
  --output regex-plugin-1.0.0.rbp

# Copy to Kotlin app
cp regex-plugin-1.0.0.rbp ~/rustbridge-workspace/regex-kotlin-app/
```

> **Note**: Format is `PLATFORM:VARIANT:PATH`. If variant is omitted, it defaults to "release".

## Complete Benchmark Code

Replace the entire contents of `src/main/kotlin/com/example/Main.kt` with:

```kotlin
package com.example

import com.fasterxml.jackson.module.kotlin.jacksonObjectMapper
import com.fasterxml.jackson.module.kotlin.readValue
import com.rustbridge.BundleLoader
import com.rustbridge.LogLevel
import com.rustbridge.Plugin
import com.rustbridge.PluginConfig
import com.rustbridge.ffm.FfmPluginLoader
import java.nio.file.Path

data class MatchRequest(val pattern: String, val text: String)
data class MatchResponse(val matches: Boolean, val cached: Boolean)

val mapper = jacksonObjectMapper()

inline fun <reified T> Plugin.callTyped(
    messageType: String,
    request: Any
): T {
    val requestJson = mapper.writeValueAsString(request)
    val responseJson = call(messageType, requestJson)
    return mapper.readValue(responseJson)
}

fun main(args: Array<String>) {
    val bundlePath = "regex-plugin-1.0.0.rbp"

    println("=== Debug Build ===")
    runBenchmark(bundlePath, "debug")

    println("\n=== Release Build ===")
    runBenchmark(bundlePath, "release")

    println("\n=== Cache Effectiveness (Release) ===")
    cacheEffectivenessBenchmark(bundlePath)
}

fun runBenchmark(bundlePath: String, variant: String) {
    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    // Detect platform and extract the specified variant
    val platform = detectPlatform()
    val tempDir = java.nio.file.Files.createTempDirectory("rustbridge-bench")
    val libraryPath = bundleLoader.extractLibrary(platform, variant, tempDir)

    // Configure with WARN level to reduce noise
    val config = PluginConfig.defaults()
        .logLevel(LogLevel.WARN)

    FfmPluginLoader.load(libraryPath, config, null).use { plugin ->
        benchmark(plugin)
    }

    bundleLoader.close()
    tempDir.toFile().deleteRecursively()
}

fun benchmark(plugin: Plugin) {
    val patterns = listOf(
        """\d{4}-\d{2}-\d{2}""",
        """^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$""",
        """^https?://[^\s/$.?#].[^\s]*$""",
        """^\+?[1-9]\d{1,14}$""",
    )
    val samples = listOf("2024-01-15", "user@example.com", "https://github.com", "+1234567890")
    val iterations = 10_000

    // Warm up the cache
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

    println("Total calls: $totalCalls")
    println("Total time: ${elapsed / 1_000_000} ms")
    println("Avg per call: $avgNanos ns (${avgNanos / 1000} µs)")
    println("Calls/second: ${1_000_000_000L * totalCalls / elapsed}")
}

fun cacheEffectivenessBenchmark(bundlePath: String) {
    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    val platform = detectPlatform()
    val tempDir = java.nio.file.Files.createTempDirectory("rustbridge-bench")
    val libraryPath = bundleLoader.extractLibrary(platform, "release", tempDir)

    val config = PluginConfig.defaults()
        .logLevel(LogLevel.WARN)

    FfmPluginLoader.load(libraryPath, config, null).use { plugin ->
        val pattern = """^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"""
        val sample = "user@example.com"
        val iterations = 10_000

        // Without cache - use unique patterns each time
        println("Without cache (compiling each time)...")
        val noCacheStart = System.nanoTime()
        for (i in 0 until iterations) {
            val uniquePattern = "$pattern$i"
            try {
                plugin.callTyped<MatchResponse>("match", MatchRequest(uniquePattern, sample))
            } catch (e: Exception) {
                // Invalid patterns will error, that's fine
            }
        }
        val noCacheTime = System.nanoTime() - noCacheStart

        // With cache - same pattern each time
        println("With cache (reusing compiled pattern)...")
        plugin.callTyped<MatchResponse>("match", MatchRequest(pattern, sample))

        val cacheStart = System.nanoTime()
        for (i in 0 until iterations) {
            plugin.callTyped<MatchResponse>("match", MatchRequest(pattern, sample))
        }
        val cacheTime = System.nanoTime() - cacheStart

        println("Without cache: ${noCacheTime / 1_000_000} ms")
        println("With cache: ${cacheTime / 1_000_000} ms")
        println("Speedup: ${"%.1f".format(noCacheTime.toDouble() / cacheTime)}x")
    }

    bundleLoader.close()
    tempDir.toFile().deleteRecursively()
}

fun detectPlatform(): String {
    val os = System.getProperty("os.name").lowercase()
    val arch = System.getProperty("os.arch").lowercase()

    val osName = when {
        os.contains("linux") -> "linux"
        os.contains("mac") || os.contains("darwin") -> "darwin"
        os.contains("windows") -> "windows"
        else -> throw UnsupportedOperationException("Unsupported OS: $os")
    }

    val archName = when {
        arch.contains("amd64") || arch.contains("x86_64") -> "x86_64"
        arch.contains("aarch64") || arch.contains("arm64") -> "aarch64"
        else -> throw UnsupportedOperationException("Unsupported architecture: $arch")
    }

    return "$osName-$archName"
}
```

## Run the Benchmark

```bash
cd ~/rustbridge-workspace/regex-kotlin-app
./gradlew run
```

Typical output:

```
=== Debug Build ===
Total calls: 40000
Total time: 1736 ms
Avg per call: 43413 ns (43 µs)
Calls/second: 23034

=== Release Build ===
Total calls: 40000
Total time: 257 ms
Avg per call: 6426 ns (6 µs)
Calls/second: 155611

=== Cache Effectiveness (Release) ===
Without cache (compiling each time)...
With cache (reusing compiled pattern)...
Without cache: 842 ms
With cache: 62 ms
Speedup: 13.4x
```

**Key findings:**

- Release is significantly faster than debug
- Caching provides ~13x speedup

## Optimization Tips

1. **Always use release builds** in production
2. **Batch operations** when possible to reduce FFI crossings
3. **Size your cache appropriately** based on your pattern diversity
4. **Disable debug logging** in production (`LogLevel.WARN` or higher)

## Summary

You've now completed the tutorial! You've learned:

1. **Chapter 1**: Build a Rust plugin with regex matching, LRU caching, configuration, and logging
2. **Chapter 2**: Call it from Kotlin with type-safe wrappers, logging callbacks, and benchmarking

## When to Optimize

JSON transport is fast enough for most use cases. Before resorting to binary transport:

1. **Profile first** - Measure actual bottlenecks in your application
2. **Check your workload** - Is the plugin the bottleneck, or is it I/O, database, etc.?
3. **Consider caching** - As shown above, caching can provide 10x+ speedups

The JSON overhead is negligible compared to most business logic. Premature optimization often adds complexity without
meaningful benefit.

## Next Steps

- **Sign your bundles** for production deployment
- **Add more message types** to your plugin
- **Read the architecture docs** for deeper understanding

See the [rustbridge documentation](../../README.md) for more advanced topics.
