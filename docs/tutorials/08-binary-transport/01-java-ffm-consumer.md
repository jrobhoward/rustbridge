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

## Install rustbridge Java Libraries

If you haven't already, install the Java libraries to Maven local:

```bash
cd ~/rustbridge-workspace/rustbridge/rustbridge-java
./gradlew publishToMavenLocal
```

## Verify the Generated Consumer

```bash
cd ~/rustbridge-workspace/thumbnail-plugin/consumers/java-ffm
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

import com.rustbridge.BundleLoader;
import com.rustbridge.ffm.FfmPluginLoader;
import com.google.gson.Gson;

public class Main {
    public static void main(String[] args) throws Exception {
        String bundlePath = "thumbnail-plugin-1.0.0.rbp";

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        try (var plugin = FfmPluginLoader.load(bundleLoader.extractLibrary())) {
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
- JSON calls work the same as JNI
- FFM enables direct memory access for binary transport

## Define Struct Layouts

Create `src/main/java/com/example/ThumbnailStructs.java`:

```java
package com.example;

import java.lang.foreign.*;
import java.lang.invoke.VarHandle;
import java.nio.charset.StandardCharsets;

/**
 * Binary struct layouts for thumbnail plugin.
 *
 * These layouts must match the Rust #[repr(C)] structs exactly.
 */
public final class ThumbnailStructs {

    private ThumbnailStructs() {} // Utility class

    // Message ID for thumbnail creation
    public static final int MSG_THUMBNAIL_CREATE = 100;

    // Output format constants
    public static final int FORMAT_JPEG = 0;
    public static final int FORMAT_PNG = 1;
    public static final int FORMAT_WEBP = 2;

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
    public static final StructLayout REQUEST_HEADER_LAYOUT = MemoryLayout.structLayout(
        ValueLayout.JAVA_BYTE.withName("version"),
        MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_BYTE).withName("_reserved"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("target_width"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("target_height"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("output_format"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("quality"),
        ValueLayout.JAVA_INT_UNALIGNED.withName("payload_size")
    ).withName("ThumbnailRequestHeader");

    public static final long REQUEST_HEADER_SIZE = REQUEST_HEADER_LAYOUT.byteSize(); // 24

    // VarHandles for request header fields
    private static final VarHandle VH_REQ_VERSION =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("version"));
    private static final VarHandle VH_REQ_TARGET_WIDTH =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("target_width"));
    private static final VarHandle VH_REQ_TARGET_HEIGHT =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("target_height"));
    private static final VarHandle VH_REQ_OUTPUT_FORMAT =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("output_format"));
    private static final VarHandle VH_REQ_QUALITY =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("quality"));
    private static final VarHandle VH_REQ_PAYLOAD_SIZE =
        REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("payload_size"));

    /**
     * Create a thumbnail request with image data.
     *
     * @param arena Arena for memory allocation
     * @param targetWidth Desired width (0 = proportional)
     * @param targetHeight Desired height (0 = proportional)
     * @param outputFormat FORMAT_JPEG, FORMAT_PNG, or FORMAT_WEBP
     * @param quality Quality 1-100 (for JPEG/WebP)
     * @param imageData Raw image bytes
     * @return MemorySegment containing header + image data
     */
    public static MemorySegment createRequest(
            Arena arena,
            int targetWidth,
            int targetHeight,
            int outputFormat,
            int quality,
            byte[] imageData) {

        // Allocate header + payload
        long totalSize = REQUEST_HEADER_SIZE + imageData.length;
        MemorySegment request = arena.allocate(totalSize);

        // Set header fields
        VH_REQ_VERSION.set(request, 0L, (byte) 1);
        VH_REQ_TARGET_WIDTH.set(request, 0L, targetWidth);
        VH_REQ_TARGET_HEIGHT.set(request, 0L, targetHeight);
        VH_REQ_OUTPUT_FORMAT.set(request, 0L, outputFormat);
        VH_REQ_QUALITY.set(request, 0L, quality);
        VH_REQ_PAYLOAD_SIZE.set(request, 0L, imageData.length);

        // Copy image data after header
        MemorySegment.copy(imageData, 0, request, ValueLayout.JAVA_BYTE,
            REQUEST_HEADER_SIZE, imageData.length);

        return request;
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
                case FORMAT_JPEG -> "JPEG";
                case FORMAT_PNG -> "PNG";
                case FORMAT_WEBP -> "WebP";
                default -> "Unknown";
            };
        }
    }

    /**
     * Parse a thumbnail response from native memory.
     *
     * @param response MemorySegment containing response data
     * @return Parsed ThumbnailResponse
     */
    public static ThumbnailResponse parseResponse(MemorySegment response) {
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

import com.rustbridge.BundleLoader;
import com.rustbridge.ffm.FfmPluginLoader;
import com.example.ThumbnailStructs.ThumbnailResponse;

import java.io.IOException;
import java.lang.foreign.Arena;
import java.nio.file.Files;
import java.nio.file.Path;

import static com.example.ThumbnailStructs.*;

public class Main {

    public static void main(String[] args) throws Exception {
        System.out.println("=== Binary Transport Demo (Java FFM) ===\n");

        String bundlePath = "thumbnail-plugin-1.0.0.rbp";
        String imagePath = "test-image.jpg";

        // Load the test image
        byte[] imageData = loadImage(imagePath);
        System.out.printf("Loaded image: %s (%d bytes)%n%n", imagePath, imageData.length);

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        try (var plugin = FfmPluginLoader.load(bundleLoader.extractLibrary())) {

            // Demo 1: Create JPEG thumbnail
            System.out.println("Demo 1: Create JPEG thumbnail (100x100)");
            try (Arena arena = Arena.ofConfined()) {
                var request = createRequest(
                    arena,
                    100,           // target width
                    100,           // target height
                    FORMAT_JPEG,   // output format
                    85,            // quality
                    imageData
                );

                long startTime = System.nanoTime();
                var responseSegment = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
                long elapsed = System.nanoTime() - startTime;

                ThumbnailResponse response = parseResponse(responseSegment);

                System.out.printf("  Thumbnail: %dx%d %s (%d bytes)%n",
                    response.width(), response.height(),
                    response.formatName(), response.thumbnailData().length);
                System.out.printf("  Processing time: %.2f ms%n", elapsed / 1_000_000.0);

                // Save the thumbnail
                saveThumbnail(response.thumbnailData(), "thumbnail-100x100.jpg");
                System.out.println("  Saved: thumbnail-100x100.jpg");

                // Free native memory
                plugin.freeBuffer(responseSegment);
            }

            // Demo 2: Create PNG thumbnail (different dimensions)
            System.out.println("\nDemo 2: Create PNG thumbnail (200x0 = proportional height)");
            try (Arena arena = Arena.ofConfined()) {
                var request = createRequest(
                    arena,
                    200,           // target width
                    0,             // 0 = calculate proportionally
                    FORMAT_PNG,    // output format
                    0,             // quality ignored for PNG
                    imageData
                );

                long startTime = System.nanoTime();
                var responseSegment = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
                long elapsed = System.nanoTime() - startTime;

                ThumbnailResponse response = parseResponse(responseSegment);

                System.out.printf("  Thumbnail: %dx%d %s (%d bytes)%n",
                    response.width(), response.height(),
                    response.formatName(), response.thumbnailData().length);
                System.out.printf("  Processing time: %.2f ms%n", elapsed / 1_000_000.0);

                saveThumbnail(response.thumbnailData(), "thumbnail-200xN.png");
                System.out.println("  Saved: thumbnail-200xN.png");

                plugin.freeBuffer(responseSegment);
            }

            // Demo 3: Performance comparison
            System.out.println("\nDemo 3: Performance comparison (10 iterations)");
            int iterations = 10;

            try (Arena arena = Arena.ofConfined()) {
                var request = createRequest(
                    arena, 100, 100, FORMAT_JPEG, 80, imageData
                );

                // Warm up
                for (int i = 0; i < 3; i++) {
                    var resp = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
                    plugin.freeBuffer(resp);
                }

                // Measure
                long totalTime = 0;
                for (int i = 0; i < iterations; i++) {
                    long start = System.nanoTime();
                    var resp = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
                    totalTime += System.nanoTime() - start;
                    plugin.freeBuffer(resp);
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
    mavenLocal()
    mavenCentral()
}

dependencies {
    implementation("com.rustbridge:rustbridge-core:0.7.0")
    implementation("com.rustbridge:rustbridge-ffm:0.7.0")
    implementation("com.google.code.gson:gson:2.11.0")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(21))
    }
}

application {
    mainClass.set("com.example.Main")
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

### Struct Layout Precision

The FFM StructLayout must exactly match the Rust struct:

```java
public static final StructLayout REQUEST_HEADER_LAYOUT = MemoryLayout.structLayout(
    ValueLayout.JAVA_BYTE.withName("version"),           // 1 byte
    MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_BYTE).withName("_reserved"), // 3 bytes
    ValueLayout.JAVA_INT_UNALIGNED.withName("target_width"),  // 4 bytes
    // ...
);
```

Key points:
- `JAVA_INT_UNALIGNED` for fields not on natural alignment
- `sequenceLayout(3, JAVA_BYTE)` for the `_reserved` array
- Total size must be 24 bytes for request, 20 for response

### Memory Management

```java
try (Arena arena = Arena.ofConfined()) {
    var request = createRequest(arena, ...);
    var response = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
    // Process response...
    plugin.freeBuffer(response);  // Free native memory!
}
```

- **Arena**: Manages request memory lifetime (freed when arena closes)
- **Response**: Allocated by Rust, must be freed with `freeBuffer()`
- **Copy early**: Copy response data to Java arrays before freeing

### VarHandle Access

VarHandles provide type-safe field access:

```java
private static final VarHandle VH_REQ_VERSION =
    REQUEST_HEADER_LAYOUT.varHandle(MemoryLayout.PathElement.groupElement("version"));

// Usage:
VH_REQ_VERSION.set(request, 0L, (byte) 1);
byte version = (byte) VH_REQ_VERSION.get(response, 0L);
```

## Error Handling

Handle common errors:

```java
try {
    var response = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
    // Check for error in response...
} catch (PluginException e) {
    if (e.getErrorCode() == 2) {
        System.err.println("Invalid request format");
    } else if (e.getErrorCode() == 5) {
        System.err.println("Handler error: " + e.getMessage());
    }
}
```

## What's Next?

Continue to the Java JNI implementation, which uses ByteBuffer instead of FFM.

[Continue to Section 2: Java JNI Consumer](./02-java-jni-consumer.md)
