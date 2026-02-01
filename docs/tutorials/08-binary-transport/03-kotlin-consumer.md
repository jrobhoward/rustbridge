# Section 3: Kotlin Consumer

In this section, you'll implement binary transport in Kotlin using FFM with idiomatic Kotlin patterns including extension functions and data classes.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new thumbnail-plugin --all`
2. Replace `src/lib.rs` with the thumbnail plugin implementation
3. Add the `image` dependency to `Cargo.toml`
4. Build the plugin and create the bundle
5. Copy the bundle to `consumers/kotlin/`
6. Copy a test image to `consumers/kotlin/`

## Install rustbridge Java Libraries

If you haven't already, install the Java libraries to Maven local:

```bash
cd ~/rustbridge-workspace/rustbridge/rustbridge-java
./gradlew publishToMavenLocal
```

## Verify the Generated Consumer

```bash
cd ~/rustbridge-workspace/thumbnail-plugin/consumers/kotlin
./gradlew run
```

You should see the basic echo response:

```
Response: Hello from Kotlin!
Length: 17
```

## Define Struct Layouts

Create `src/main/kotlin/com/example/ThumbnailStructs.kt`:

```kotlin
package com.example

import java.lang.foreign.*
import java.lang.invoke.VarHandle

/**
 * Binary struct layouts for thumbnail plugin.
 *
 * These layouts must match the Rust #[repr(C)] structs exactly.
 */
object ThumbnailStructs {

    // Message ID for thumbnail creation
    const val MSG_THUMBNAIL_CREATE: Int = 100

    // Output format constants
    const val FORMAT_JPEG: Int = 0
    const val FORMAT_PNG: Int = 1
    const val FORMAT_WEBP: Int = 2

    // ========================================================================
    // ThumbnailRequestHeader (24 bytes)
    // ========================================================================

    /**
     * Layout matching Rust ThumbnailRequestHeader:
     *
     *   Offset 0:  version (u8)
     *   Offset 1:  _reserved (3 bytes)
     *   Offset 4:  target_width (u32)
     *   Offset 8:  target_height (u32)
     *   Offset 12: output_format (u32)
     *   Offset 16: quality (u32)
     *   Offset 20: payload_size (u32)
     *   Total: 24 bytes
     */
    val REQUEST_HEADER_LAYOUT: StructLayout = MemoryLayout.structLayout(
        ValueLayout.JAVA_BYTE.withName("version"),
        MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_BYTE).withName("_reserved"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("target_width"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("target_height"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("output_format"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("quality"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("payload_size")
    ).withName("ThumbnailRequestHeader")

    val REQUEST_HEADER_SIZE: Long = REQUEST_HEADER_LAYOUT.byteSize() // 24

    // VarHandles for request header fields
    private val VH_REQ_VERSION: VarHandle =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("version"))
    private val VH_REQ_TARGET_WIDTH: VarHandle =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("target_width"))
    private val VH_REQ_TARGET_HEIGHT: VarHandle =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("target_height"))
    private val VH_REQ_OUTPUT_FORMAT: VarHandle =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("output_format"))
    private val VH_REQ_QUALITY: VarHandle =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("quality"))
    private val VH_REQ_PAYLOAD_SIZE: VarHandle =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("payload_size"))

    /**
     * Create a thumbnail request with image data.
     */
    fun createRequest(
        arena: Arena,
        targetWidth: Int,
        targetHeight: Int,
        outputFormat: Int,
        quality: Int,
        imageData: ByteArray
    ): MemorySegment {
        // Allocate header + payload
        val totalSize = REQUEST_HEADER_SIZE + imageData.size
        val request = arena.allocate(totalSize)

        // Set header fields
        VH_REQ_VERSION.set(request, 0L, 1.toByte())
        VH_REQ_TARGET_WIDTH.set(request, 0L, targetWidth)
        VH_REQ_TARGET_HEIGHT.set(request, 0L, targetHeight)
        VH_REQ_OUTPUT_FORMAT.set(request, 0L, outputFormat)
        VH_REQ_QUALITY.set(request, 0L, quality)
        VH_REQ_PAYLOAD_SIZE.set(request, 0L, imageData.size)

        // Copy image data after header
        MemorySegment.copy(
            imageData, 0,
            request, ValueLayout.JAVA_BYTE,
            REQUEST_HEADER_SIZE, imageData.size
        )

        return request
    }

    // ========================================================================
    // ThumbnailResponseHeader (20 bytes)
    // ========================================================================

    /**
     * Layout matching Rust ThumbnailResponseHeader:
     *
     *   Offset 0:  version (u8)
     *   Offset 1:  _reserved (3 bytes)
     *   Offset 4:  width (u32)
     *   Offset 8:  height (u32)
     *   Offset 12: format (u32)
     *   Offset 16: payload_size (u32)
     *   Total: 20 bytes
     */
    val RESPONSE_HEADER_LAYOUT: StructLayout = MemoryLayout.structLayout(
        ValueLayout.JAVA_BYTE.withName("version"),
        MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_BYTE).withName("_reserved"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("width"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("height"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("format"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("payload_size")
    ).withName("ThumbnailResponseHeader")

    val RESPONSE_HEADER_SIZE: Long = RESPONSE_HEADER_LAYOUT.byteSize() // 20

    // VarHandles for response header fields
    private val VH_RESP_VERSION: VarHandle =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("version"))
    private val VH_RESP_WIDTH: VarHandle =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("width"))
    private val VH_RESP_HEIGHT: VarHandle =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("height"))
    private val VH_RESP_FORMAT: VarHandle =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("format"))
    private val VH_RESP_PAYLOAD_SIZE: VarHandle =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("payload_size"))

    /**
     * Parse a thumbnail response from native memory.
     */
    fun parseResponse(response: MemorySegment): ThumbnailResponse {
        // Validate minimum size
        require(response.byteSize() >= RESPONSE_HEADER_SIZE) {
            "Response too small: ${response.byteSize()} bytes"
        }

        // Read header fields
        val version = VH_RESP_VERSION.get(response, 0L) as Byte
        require(version.toInt() == 1) { "Unsupported version: $version" }

        val width = VH_RESP_WIDTH.get(response, 0L) as Int
        val height = VH_RESP_HEIGHT.get(response, 0L) as Int
        val format = VH_RESP_FORMAT.get(response, 0L) as Int
        val payloadSize = VH_RESP_PAYLOAD_SIZE.get(response, 0L) as Int

        // Validate total size
        val expectedSize = RESPONSE_HEADER_SIZE + payloadSize
        require(response.byteSize() >= expectedSize) {
            "Response size mismatch: ${response.byteSize()} bytes, expected $expectedSize"
        }

        // Copy thumbnail data to Kotlin array
        val thumbnailData = ByteArray(payloadSize)
        MemorySegment.copy(
            response, ValueLayout.JAVA_BYTE,
            RESPONSE_HEADER_SIZE, thumbnailData, 0, payloadSize
        )

        return ThumbnailResponse(width, height, OutputFormat.fromInt(format), thumbnailData)
    }
}

/**
 * Output format enum with human-readable names.
 */
enum class OutputFormat(val value: Int) {
    JPEG(0),
    PNG(1),
    WEBP(2);

    companion object {
        fun fromInt(value: Int): OutputFormat = entries.find { it.value == value } ?: JPEG
    }
}

/**
 * Parsed thumbnail response.
 */
data class ThumbnailResponse(
    val width: Int,
    val height: Int,
    val format: OutputFormat,
    val thumbnailData: ByteArray
) {
    val dimensions: String get() = "${width}x${height}"
    val sizeKb: Double get() = thumbnailData.size / 1024.0

    override fun equals(other: Any?): Boolean {
        if (this === other) return true
        if (other !is ThumbnailResponse) return false
        return width == other.width && height == other.height &&
               format == other.format && thumbnailData.contentEquals(other.thumbnailData)
    }

    override fun hashCode(): Int {
        var result = width
        result = 31 * result + height
        result = 31 * result + format.hashCode()
        result = 31 * result + thumbnailData.contentHashCode()
        return result
    }
}
```

## Add Extension Functions

Create `src/main/kotlin/com/example/ThumbnailExtensions.kt`:

```kotlin
package com.example

import com.rustbridge.Plugin
import java.lang.foreign.Arena
import java.lang.foreign.MemorySegment
import kotlin.time.Duration
import kotlin.time.measureTimedValue

/**
 * Extension functions for thumbnail plugin operations.
 */

/**
 * Create a thumbnail from image data.
 *
 * @param imageData Raw image bytes (JPEG, PNG, etc.)
 * @param width Target width (0 = proportional to height)
 * @param height Target height (0 = proportional to width)
 * @param format Output format (default: JPEG)
 * @param quality Quality 1-100 for JPEG/WebP (default: 85)
 * @return ThumbnailResponse with the generated thumbnail
 */
fun Plugin.createThumbnail(
    imageData: ByteArray,
    width: Int = 100,
    height: Int = 100,
    format: OutputFormat = OutputFormat.JPEG,
    quality: Int = 85
): ThumbnailResponse {
    Arena.ofConfined().use { arena ->
        val request = ThumbnailStructs.createRequest(
            arena, width, height, format.value, quality, imageData
        )

        val response = callRaw(ThumbnailStructs.MSG_THUMBNAIL_CREATE, request)
        try {
            return ThumbnailStructs.parseResponse(response)
        } finally {
            freeBuffer(response)
        }
    }
}

/**
 * Create a thumbnail and measure processing time.
 */
fun Plugin.createThumbnailTimed(
    imageData: ByteArray,
    width: Int = 100,
    height: Int = 100,
    format: OutputFormat = OutputFormat.JPEG,
    quality: Int = 85
): Pair<ThumbnailResponse, Duration> {
    val (result, duration) = measureTimedValue {
        createThumbnail(imageData, width, height, format, quality)
    }
    return result to duration
}

/**
 * Create thumbnails at multiple sizes.
 */
fun Plugin.createThumbnailSizes(
    imageData: ByteArray,
    sizes: List<Pair<Int, Int>>,
    format: OutputFormat = OutputFormat.JPEG,
    quality: Int = 85
): List<ThumbnailResponse> {
    return sizes.map { (w, h) ->
        createThumbnail(imageData, w, h, format, quality)
    }
}

/**
 * Create thumbnails at multiple quality levels.
 */
fun Plugin.createThumbnailQualities(
    imageData: ByteArray,
    width: Int,
    height: Int,
    qualities: List<Int>
): List<Pair<Int, ThumbnailResponse>> {
    return qualities.map { quality ->
        quality to createThumbnail(imageData, width, height, OutputFormat.JPEG, quality)
    }
}
```

## Update Main.kt

Replace `src/main/kotlin/com/example/Main.kt`:

```kotlin
package com.example

import com.rustbridge.BundleLoader
import com.rustbridge.ffm.FfmPluginLoader
import java.io.File
import kotlin.time.Duration
import kotlin.time.measureTime

fun main() {
    println("=== Binary Transport Demo (Kotlin) ===\n")

    val bundlePath = "thumbnail-plugin-0.1.0.rbp"
    val imagePath = "test-image.jpg"

    // Load the test image
    val imageFile = File(imagePath)
    require(imageFile.exists()) {
        "Image not found: $imagePath\nPlease copy a test image to the current directory."
    }
    val imageData = imageFile.readBytes()
    println("Loaded image: $imagePath (${imageData.size} bytes)\n")

    val bundleLoader = BundleLoader.builder()
        .bundlePath(bundlePath)
        .verifySignatures(false)
        .build()

    FfmPluginLoader.load(bundleLoader.extractLibrary()).use { plugin ->

        // Demo 1: Basic thumbnail creation
        println("Demo 1: Create JPEG thumbnail (100x100)")
        val (thumb1, time1) = plugin.createThumbnailTimed(
            imageData,
            width = 100,
            height = 100,
            format = OutputFormat.JPEG,
            quality = 85
        )
        println("  Thumbnail: ${thumb1.dimensions} ${thumb1.format} (${thumb1.thumbnailData.size} bytes)")
        println("  Processing time: ${time1.inWholeMilliseconds} ms")
        File("thumbnail-kt-100x100.jpg").writeBytes(thumb1.thumbnailData)
        println("  Saved: thumbnail-kt-100x100.jpg")

        // Demo 2: Proportional sizing
        println("\nDemo 2: Proportional sizing (width=200, height=0)")
        val thumb2 = plugin.createThumbnail(
            imageData,
            width = 200,
            height = 0,  // Proportional
            format = OutputFormat.PNG
        )
        println("  Thumbnail: ${thumb2.dimensions} ${thumb2.format} (${thumb2.thumbnailData.size} bytes)")
        File("thumbnail-kt-200xN.png").writeBytes(thumb2.thumbnailData)
        println("  Saved: thumbnail-kt-200xN.png")

        // Demo 3: Multiple sizes at once
        println("\nDemo 3: Multiple sizes")
        val sizes = listOf(50 to 50, 100 to 100, 150 to 150, 200 to 200)
        val thumbs = plugin.createThumbnailSizes(imageData, sizes)
        thumbs.forEach { thumb ->
            println("  ${thumb.dimensions}: ${thumb.thumbnailData.size} bytes")
        }

        // Demo 4: Quality comparison
        println("\nDemo 4: Quality comparison (JPEG)")
        val qualities = listOf(20, 50, 80, 95)
        val qualityResults = plugin.createThumbnailQualities(imageData, 150, 150, qualities)
        qualityResults.forEach { (quality, thumb) ->
            println("  Quality $quality: ${thumb.thumbnailData.size} bytes (%.1f KB)".format(thumb.sizeKb))
            File("thumbnail-kt-q$quality.jpg").writeBytes(thumb.thumbnailData)
        }

        // Demo 5: Performance benchmark
        println("\nDemo 5: Performance benchmark (20 iterations)")
        val iterations = 20

        // Warm up
        repeat(3) {
            plugin.createThumbnail(imageData, 100, 100)
        }

        // Measure
        val totalTime = measureTime {
            repeat(iterations) {
                plugin.createThumbnail(imageData, 100, 100)
            }
        }

        val avgMs = totalTime.inWholeMilliseconds.toDouble() / iterations
        println("  Total time: ${totalTime.inWholeMilliseconds} ms")
        println("  Average per thumbnail: %.2f ms".format(avgMs))
        println("  Throughput: %.1f thumbnails/sec".format(1000.0 / avgMs))

        // Demo 6: Format comparison
        println("\nDemo 6: Format comparison (150x150)")
        for (format in OutputFormat.entries) {
            val quality = if (format == OutputFormat.PNG) 0 else 80
            val (thumb, time) = plugin.createThumbnailTimed(
                imageData, 150, 150, format, quality
            )
            println("  ${format.name}: ${thumb.thumbnailData.size} bytes in ${time.inWholeMilliseconds} ms")
        }
    }

    bundleLoader.close()
    println("\n=== Demo Complete ===")
}
```

## Update build.gradle.kts

```kotlin
plugins {
    kotlin("jvm") version "2.0.0"
    application
}

repositories {
    mavenLocal()
    mavenCentral()
}

dependencies {
    implementation("com.rustbridge:rustbridge-core:0.7.0")
    implementation("com.rustbridge:rustbridge-ffm:0.7.0")
    implementation("com.rustbridge:rustbridge-kotlin:0.7.0")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.17.2")
}

kotlin {
    jvmToolchain(21)
}

application {
    mainClass.set("com.example.MainKt")
}

tasks.withType<JavaExec> {
    // Required for Foreign Function & Memory API
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}
```

## Run the Demo

```bash
./gradlew run
```

Expected output:

```
=== Binary Transport Demo (Kotlin) ===

Loaded image: test-image.jpg (45678 bytes)

Demo 1: Create JPEG thumbnail (100x100)
  Thumbnail: 100x75 JPEG (2847 bytes)
  Processing time: 12 ms
  Saved: thumbnail-kt-100x100.jpg

Demo 2: Proportional sizing (width=200, height=0)
  Thumbnail: 200x150 PNG (18234 bytes)
  Saved: thumbnail-kt-200xN.png

Demo 3: Multiple sizes
  50x37: 987 bytes
  100x75: 2847 bytes
  150x112: 5234 bytes
  200x150: 8456 bytes

Demo 4: Quality comparison (JPEG)
  Quality 20: 1234 bytes (1.2 KB)
  Quality 50: 2567 bytes (2.5 KB)
  Quality 80: 4123 bytes (4.0 KB)
  Quality 95: 7890 bytes (7.7 KB)

Demo 5: Performance benchmark (20 iterations)
  Total time: 168 ms
  Average per thumbnail: 8.40 ms
  Throughput: 119.0 thumbnails/sec

Demo 6: Format comparison (150x150)
  JPEG: 5234 bytes in 8 ms
  PNG: 12456 bytes in 15 ms
  WEBP: 5234 bytes in 8 ms

=== Demo Complete ===
```

## Key Observations

### Idiomatic Kotlin Patterns

Extension functions make the API feel native to Kotlin:

```kotlin
// Clean, expressive API
val thumbnail = plugin.createThumbnail(
    imageData,
    width = 100,
    height = 100,
    format = OutputFormat.JPEG,
    quality = 85
)

// Timed version with destructuring
val (result, duration) = plugin.createThumbnailTimed(imageData, 100, 100)
println("Created ${result.dimensions} in ${duration.inWholeMilliseconds} ms")
```

### Data Classes

Kotlin data classes provide:
- Automatic `equals()`, `hashCode()`, `toString()`
- Destructuring support
- Copy with modifications

```kotlin
data class ThumbnailResponse(
    val width: Int,
    val height: Int,
    val format: OutputFormat,
    val thumbnailData: ByteArray
) {
    val dimensions: String get() = "${width}x${height}"
    val sizeKb: Double get() = thumbnailData.size / 1024.0
}
```

### Enum with Values

Enums with associated values and companion objects:

```kotlin
enum class OutputFormat(val value: Int) {
    JPEG(0), PNG(1), WEBP(2);

    companion object {
        fun fromInt(value: Int): OutputFormat =
            entries.find { it.value == value } ?: JPEG
    }
}
```

### Arena with use()

Kotlin's `use` extension ensures proper cleanup:

```kotlin
Arena.ofConfined().use { arena ->
    val request = ThumbnailStructs.createRequest(arena, ...)
    val response = plugin.callRaw(MSG_THUMBNAIL_CREATE, request)
    try {
        return ThumbnailStructs.parseResponse(response)
    } finally {
        plugin.freeBuffer(response)
    }
}
// arena automatically closed here
```

### Kotlin Time API

Using `kotlin.time` for measurements:

```kotlin
import kotlin.time.measureTimedValue
import kotlin.time.measureTime

val (result, duration) = measureTimedValue {
    plugin.createThumbnail(imageData, 100, 100)
}
println("Took ${duration.inWholeMilliseconds} ms")
```

## Error Handling

Kotlin-style error handling with `require` and `runCatching`:

```kotlin
// Validation
require(imageData.isNotEmpty()) { "Image data cannot be empty" }

// Safe parsing
val result = runCatching {
    plugin.createThumbnail(imageData, width, height)
}.getOrElse { e ->
    println("Failed to create thumbnail: ${e.message}")
    null
}
```

## What's Next?

Continue to the C# implementation, which uses unsafe structs and StructLayout.

[Continue to Section 4: C# Consumer](./04-csharp-consumer.md)
