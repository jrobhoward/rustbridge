# Section 1: Java FFM Consumer

In this section, you'll implement binary transport in Java using the Foreign Function & Memory (FFM) API. FFM provides direct access to native memory with type-safe struct layouts, making it ideal for binary transport.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new thumbnail-plugin --all`
2. Replace `src/lib.rs` with the thumbnail plugin implementation
3. Add the `image` dependency to `Cargo.toml`
4. Build the plugin and create the bundle
5. Copy the bundle to `consumers/java-ffm/`
6. Copy a test image to `consumers/java-ffm/`

## Verify the Generated Consumer

```bash
cd $RUSTBRIDGE_WORKSPACE/thumbnail-plugin/consumers/java-ffm
./gradlew run
```

You should see the basic echo response:

```
Response: Hello from Java FFM!
Length: 19
```

## Understanding the Generated Code

Look at `src/main/java/com/example/Main.java`:

```java
package com.example;

import io.github.jrobhoward.rustbridge.BundleLoader;
import io.github.jrobhoward.rustbridge.ffm.FfmPluginLoader;
import com.google.gson.Gson;

public class Main {
    public static void main(String[] args) throws Exception {
        String bundlePath = "thumbnail-plugin-0.1.0.rbp";

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        try (var plugin = FfmPluginLoader.load(bundleLoader.extractLibrary().toString())) {
            // JSON call example
            var request = new Gson().toJson(new EchoRequest("Hello from Java FFM!"));
            var response = plugin.call("echo", request);
            System.out.println("Response: " + response);
        }

        bundleLoader.close();
    }

    record EchoRequest(String message) {}
}
```

Key points:
- `FfmPluginLoader.load()` uses Java 21+ FFM for native access
- JSON calls work the same as other language bindings
- `FfmPlugin` provides `callRawBytes()` for binary transport

## Define Struct Layouts

Create `src/main/java/com/example/ThumbnailRequest.java`:

```java
package com.example;

import io.github.jrobhoward.rustbridge.ffm.BinaryStruct;

import java.lang.foreign.*;

/**
 * Binary request for thumbnail creation.
 *
 * Contains a 24-byte header followed by variable-length image data.
 * Extends BinaryStruct for use with FfmPlugin.callRawBytes().
 *
 * Header layout (24 bytes):
 *   Offset 0:  version (u8)
 *   Offset 1:  _reserved (3 bytes)
 *   Offset 4:  target_width (u32)
 *   Offset 8:  target_height (u32)
 *   Offset 12: output_format (u32)
 *   Offset 16: quality (u32)
 *   Offset 20: payload_size (u32)
 */
public class ThumbnailRequest extends BinaryStruct {

    public static final int HEADER_SIZE = 24;

    // Output format constants
    public static final int FORMAT_JPEG = 0;
    public static final int FORMAT_PNG = 1;
    public static final int FORMAT_WEBP = 2;

    // Message ID for thumbnail creation
    public static final int MSG_THUMBNAIL_CREATE = 100;

    private final long totalSize;

    private ThumbnailRequest(MemorySegment segment, long totalSize) {
        super(segment);
        this.totalSize = totalSize;
    }

    @Override
    public long byteSize() {
        return totalSize;
    }

    /**
     * Create a thumbnail request with image data.
     *
     * @param arena Arena for memory allocation
     * @param targetWidth Desired width (0 = proportional)
     * @param targetHeight Desired height (0 = proportional)
     * @param outputFormat FORMAT_JPEG, FORMAT_PNG, or FORMAT_WEBP
     * @param quality Quality 1-100 (for JPEG/WebP)
     * @param imageData Raw image bytes
     * @return ThumbnailRequest ready for callRawBytes()
     */
    public static ThumbnailRequest create(
            Arena arena,
            int targetWidth,
            int targetHeight,
            int outputFormat,
            int quality,
            byte[] imageData) {

        long totalSize = HEADER_SIZE + imageData.length;
        MemorySegment segment = arena.allocate(totalSize);

        // Set header fields
        segment.set(ValueLayout.JAVA_BYTE, 0, (byte) 1);  // version
        segment.set(ValueLayout.JAVA_INT_UNALIGNED, 4, targetWidth);
        segment.set(ValueLayout.JAVA_INT_UNALIGNED, 8, targetHeight);
        segment.set(ValueLayout.JAVA_INT_UNALIGNED, 12, outputFormat);
        segment.set(ValueLayout.JAVA_INT_UNALIGNED, 16, quality);
        segment.set(ValueLayout.JAVA_INT_UNALIGNED, 20, imageData.length);

        // Copy image data after header
        MemorySegment.copy(imageData, 0, segment, ValueLayout.JAVA_BYTE,
            HEADER_SIZE, imageData.length);

        return new ThumbnailRequest(segment, totalSize);
    }
}
```

Create `src/main/java/com/example/ThumbnailStructs.java`:

```java
package com.example;

import java.lang.foreign.*;
import java.lang.invoke.VarHandle;

/**
 * Response parsing for thumbnail plugin.
 *
 * Parses the byte[] returned by callRawBytes() into a ThumbnailResponse.
 */
public final class ThumbnailStructs {

    private ThumbnailStructs() {} // Utility class

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
    public static final StructLayout RESPONSE_HEADER_LAYOUT = MemoryLayout.structLayout(
        ValueLayout.JAVA_BYTE.withName("version"),
        MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_BYTE).withName("_reserved"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("width"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("height"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("format"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("payload_size")
    ).withName("ThumbnailResponseHeader");

    public static final long RESPONSE_HEADER_SIZE = RESPONSE_HEADER_LAYOUT.byteSize(); // 20

    // VarHandles for response header fields
    private static final VarHandle VH_RESP_VERSION =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("version"));
    private static final VarHandle VH_RESP_WIDTH =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("width"));
    private static final VarHandle VH_RESP_HEIGHT =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("height"));
    private static final VarHandle VH_RESP_FORMAT =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("format"));
    private static final VarHandle VH_RESP_PAYLOAD_SIZE =
        RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("payload_size"));

    /**
     * Parsed thumbnail response.
     */
    public record ThumbnailResponse(
        int width,
        int height,
        int format,
        byte[] thumbnailData
    ) {
        public String formatName() {
            return switch (format) {
                case ThumbnailRequest.FORMAT_JPEG -> "JPEG";
                case ThumbnailRequest.FORMAT_PNG -> "PNG";
                case ThumbnailRequest.FORMAT_WEBP -> "WebP";
                default -> "Unknown";
            };
        }
    }

    /**
     * Parse a thumbnail response from a byte array.
     *
     * @param responseBytes byte[] returned by callRawBytes()
     * @return Parsed ThumbnailResponse
     */
    public static ThumbnailResponse parseResponse(byte[] responseBytes) {
        // Wrap byte[] as a MemorySegment for VarHandle access
        MemorySegment response = MemorySegment.ofArray(responseBytes);

        // Validate minimum size
        if (response.byteSize() < RESPONSE_HEADER_SIZE) {
            throw new IllegalArgumentException(
                "Response too small: " + response.byteSize() + " bytes");
        }

        // Read header fields
        byte version = (byte) VH_RESP_VERSION.get(response, 0L);
        if (version != 1) {
            throw new IllegalArgumentException("Unsupported version: " + version);
        }

        int width = (int) VH_RESP_WIDTH.get(response, 0L);
        int height = (int) VH_RESP_HEIGHT.get(response, 0L);
        int format = (int) VH_RESP_FORMAT.get(response, 0L);
        int payloadSize = (int) VH_RESP_PAYLOAD_SIZE.get(response, 0L);

        // Validate total size
        long expectedSize = RESPONSE_HEADER_SIZE + payloadSize;
        if (response.byteSize() < expectedSize) {
            throw new IllegalArgumentException(
                "Response size mismatch: " + response.byteSize() +
                " bytes, expected " + expectedSize);
        }

        // Copy thumbnail data to Java array
        byte[] thumbnailData = new byte[payloadSize];
        MemorySegment.copy(response, ValueLayout.JAVA_BYTE,
            RESPONSE_HEADER_SIZE, thumbnailData, 0, payloadSize);

        return new ThumbnailResponse(width, height, format, thumbnailData);
    }
}
```

## Update Main.java

Replace `src/main/java/com/example/Main.java`:

```java
package com.example;

import io.github.jrobhoward.rustbridge.BundleLoader;
import io.github.jrobhoward.rustbridge.ffm.FfmPlugin;
import io.github.jrobhoward.rustbridge.ffm.FfmPluginLoader;
import com.example.ThumbnailStructs.ThumbnailResponse;

import java.io.IOException;
import java.lang.foreign.Arena;
import java.nio.file.Files;
import java.nio.file.Path;

import static com.example.ThumbnailRequest.*;

public class Main {

    public static void main(String[] args) throws Exception {
        System.out.println("=== Binary Transport Demo (Java FFM) ===\n");

        String bundlePath = "thumbnail-plugin-0.1.0.rbp";
        String imagePath = "test-image.jpg";

        // Load the test image
        byte[] imageData = loadImage(imagePath);
        System.out.printf("Loaded image: %s (%d bytes)%n%n", imagePath, imageData.length);

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        try (var plugin = FfmPluginLoader.load(bundleLoader.extractLibrary().toString())) {
            // Cast to FfmPlugin for binary transport access
            FfmPlugin ffmPlugin = (FfmPlugin) plugin;

            // Demo 1: Create JPEG thumbnail
            System.out.println("Demo 1: Create JPEG thumbnail (100x100)");
            try (Arena arena = Arena.ofConfined()) {
                var request = ThumbnailRequest.create(
                    arena,
                    100,           // target width
                    100,           // target height
                    FORMAT_JPEG,   // output format
                    85,            // quality
                    imageData
                );

                long startTime = System.nanoTime();
                byte[] responseBytes = ffmPlugin.callRawBytes(MSG_THUMBNAIL_CREATE, request);
                long elapsed = System.nanoTime() - startTime;

                ThumbnailResponse response = ThumbnailStructs.parseResponse(responseBytes);

                System.out.printf("  Thumbnail: %dx%d %s (%d bytes)%n",
                    response.width(), response.height(),
                    response.formatName(), response.thumbnailData().length);
                System.out.printf("  Processing time: %.2f ms%n", elapsed / 1_000_000.0);

                // Save the thumbnail
                saveThumbnail(response.thumbnailData(), "thumbnail-100x100.jpg");
                System.out.println("  Saved: thumbnail-100x100.jpg");
            }

            // Demo 2: Create PNG thumbnail (different dimensions)
            System.out.println("\nDemo 2: Create PNG thumbnail (200x0 = proportional height)");
            try (Arena arena = Arena.ofConfined()) {
                var request = ThumbnailRequest.create(
                    arena,
                    200,           // target width
                    0,             // 0 = calculate proportionally
                    FORMAT_PNG,    // output format
                    0,             // quality ignored for PNG
                    imageData
                );

                long startTime = System.nanoTime();
                byte[] responseBytes = ffmPlugin.callRawBytes(MSG_THUMBNAIL_CREATE, request);
                long elapsed = System.nanoTime() - startTime;

                ThumbnailResponse response = ThumbnailStructs.parseResponse(responseBytes);

                System.out.printf("  Thumbnail: %dx%d %s (%d bytes)%n",
                    response.width(), response.height(),
                    response.formatName(), response.thumbnailData().length);
                System.out.printf("  Processing time: %.2f ms%n", elapsed / 1_000_000.0);

                saveThumbnail(response.thumbnailData(), "thumbnail-200xN.png");
                System.out.println("  Saved: thumbnail-200xN.png");
            }

            // Demo 3: Performance comparison
            System.out.println("\nDemo 3: Performance comparison (10 iterations)");
            int iterations = 10;

            try (Arena arena = Arena.ofConfined()) {
                var request = ThumbnailRequest.create(
                    arena, 100, 100, FORMAT_JPEG, 80, imageData
                );

                // Warm up
                for (int i = 0; i < 3; i++) {
                    ffmPlugin.callRawBytes(MSG_THUMBNAIL_CREATE, request);
                }

                // Measure
                long totalTime = 0;
                for (int i = 0; i < iterations; i++) {
                    long start = System.nanoTime();
                    ffmPlugin.callRawBytes(MSG_THUMBNAIL_CREATE, request);
                    totalTime += System.nanoTime() - start;
                }

                double avgMs = (totalTime / iterations) / 1_000_000.0;
                System.out.printf("  Average time per thumbnail: %.2f ms%n", avgMs);
                System.out.printf("  Throughput: %.1f thumbnails/sec%n", 1000.0 / avgMs);
            }
        }

        bundleLoader.close();
        System.out.println("\n=== Demo Complete ===");
    }

    private static byte[] loadImage(String path) throws IOException {
        Path imagePath = Path.of(path);
        if (!Files.exists(imagePath)) {
            throw new IOException("Image not found: " + path +
                "\nPlease copy a test image to the current directory.");
        }
        return Files.readAllBytes(imagePath);
    }

    private static void saveThumbnail(byte[] data, String filename) throws IOException {
        Files.write(Path.of(filename), data);
    }
}
```

## Update build.gradle.kts

Ensure FFM is enabled:

```kotlin
plugins {
    java
    application
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("io.github.jrobhoward.rustbridge:rustbridge-core:1.0.0")
    implementation("io.github.jrobhoward.rustbridge:rustbridge-ffm:1.0.0")
    implementation("com.google.code.gson:gson:2.11.0")
}

java {
    toolchain {
        // Java 22+ required for FFM
        languageVersion.set(JavaLanguageVersion.of(22))
    }
}

application {
    mainClass.set("com.example.Main")
}

tasks.withType<JavaExec> {
    // Required for FFM native access
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}
```

## Run the Demo

```bash
./gradlew run
```

Expected output:

```
=== Binary Transport Demo (Java FFM) ===

Loaded image: test-image.jpg (45678 bytes)

Demo 1: Create JPEG thumbnail (100x100)
  Thumbnail: 100x75 JPEG (2847 bytes)
  Processing time: 12.34 ms
  Saved: thumbnail-100x100.jpg

Demo 2: Create PNG thumbnail (200x0 = proportional height)
  Thumbnail: 200x150 PNG (18234 bytes)
  Processing time: 15.67 ms
  Saved: thumbnail-200xN.png

Demo 3: Performance comparison (10 iterations)
  Average time per thumbnail: 8.45 ms
  Throughput: 118.3 thumbnails/sec

=== Demo Complete ===
```

## Verify the Output

Check that the thumbnails were created:

```bash
ls -la thumbnail-*.jpg thumbnail-*.png
```

You can view them with any image viewer to verify correct resizing.

## Key Observations

### BinaryStruct for Requests

The `ThumbnailRequest` extends `BinaryStruct`, which wraps a `MemorySegment`:

```java
public class ThumbnailRequest extends BinaryStruct {
    public static ThumbnailRequest create(Arena arena, ...) {
        MemorySegment segment = arena.allocate(totalSize);
        // Populate header + image data
        return new ThumbnailRequest(segment, totalSize);
    }
}
```

Key points:
- `BinaryStruct` is the required type for `callRawBytes()`
- The segment contains header bytes + variable-length payload
- Arena manages request memory lifetime

### Memory Management

```java
try (Arena arena = Arena.ofConfined()) {
    var request = ThumbnailRequest.create(arena, ...);
    byte[] responseBytes = ffmPlugin.callRawBytes(MSG_THUMBNAIL_CREATE, request);
    ThumbnailResponse response = ThumbnailStructs.parseResponse(responseBytes);
    // responseBytes is a managed Java byte[] — no manual freeing needed
}
```

- **Arena**: Manages request memory lifetime (freed when arena closes)
- **Response**: `callRawBytes()` returns a Java `byte[]` — memory is managed by the JVM
- **No manual freeing**: Unlike raw FFI calls, `callRawBytes()` handles native memory cleanup internally

### VarHandle Access

VarHandles provide type-safe field access for parsing responses:

```java
private static final VarHandle VH_RESP_VERSION =
    RESPONSE_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("version"));

// Usage with byte[] via MemorySegment.ofArray():
MemorySegment response = MemorySegment.ofArray(responseBytes);
byte version = (byte) VH_RESP_VERSION.get(response, 0L);
```

## Error Handling

Handle common errors:

```java
try {
    byte[] responseBytes = ffmPlugin.callRawBytes(MSG_THUMBNAIL_CREATE, request);
    ThumbnailResponse response = ThumbnailStructs.parseResponse(responseBytes);
} catch (PluginException e) {
    if (e.getErrorCode() == 2) {
        System.err.println("Invalid request format");
    } else if (e.getErrorCode() == 5) {
        System.err.println("Handler error: " + e.getMessage());
    }
}
```

## What's Next?

Continue to the Kotlin implementation, which uses FFM with idiomatic extension functions.

[Continue to Section 2: Kotlin Consumer](./02-kotlin-consumer.md)
