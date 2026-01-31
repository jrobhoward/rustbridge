# Section 2: Java JNI Consumer

In this section, you'll implement binary transport in Java using JNI with ByteBuffer. This approach works with Java 17+ and provides an alternative to FFM for binary data handling.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new thumbnail-plugin --all`
2. Replace `src/lib.rs` with the thumbnail plugin implementation
3. Add the `image` dependency to `Cargo.toml`
4. Build the plugin and create the bundle
5. Copy the bundle to `consumers/java-jni/`
6. Copy a test image to `consumers/java-jni/`

## Install rustbridge Java Libraries

If you haven't already, install the Java libraries to Maven local:

```bash
cd ~/rustbridge-workspace/rustbridge/rustbridge-java
./gradlew publishToMavenLocal
```

## Verify the Generated Consumer

```bash
cd ~/rustbridge-workspace/thumbnail-plugin/consumers/java-jni
./gradlew run
```

You should see the basic echo response:

```
Response: Hello from Java JNI!
Length: 20
```

## Define Struct Helpers

Create `src/main/java/com/example/ThumbnailStructs.java`:

```java
package com.example;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;

/**
 * Binary struct helpers for thumbnail plugin.
 *
 * Uses ByteBuffer for manual struct layout matching Rust #[repr(C)] structs.
 */
public final class ThumbnailStructs {

    private ThumbnailStructs() {} // Utility class

    // Message ID for thumbnail creation
    public static final int MSG_THUMBNAIL_CREATE = 100;

    // Header sizes
    public static final int REQUEST_HEADER_SIZE = 24;
    public static final int RESPONSE_HEADER_SIZE = 20;

    // Output format constants
    public static final int FORMAT_JPEG = 0;
    public static final int FORMAT_PNG = 1;
    public static final int FORMAT_WEBP = 2;

    // ========================================================================
    // Request Header Layout (24 bytes)
    // ========================================================================
    //
    //   Offset 0:  version (u8)
    //   Offset 1:  _reserved (3 bytes)
    //   Offset 4:  target_width (u32, little-endian)
    //   Offset 8:  target_height (u32, little-endian)
    //   Offset 12: output_format (u32, little-endian)
    //   Offset 16: quality (u32, little-endian)
    //   Offset 20: payload_size (u32, little-endian)
    //
    // ========================================================================

    /**
     * Create a thumbnail request with image data.
     *
     * @param targetWidth Desired width (0 = proportional)
     * @param targetHeight Desired height (0 = proportional)
     * @param outputFormat FORMAT_JPEG, FORMAT_PNG, or FORMAT_WEBP
     * @param quality Quality 1-100 (for JPEG/WebP)
     * @param imageData Raw image bytes
     * @return byte[] containing header + image data
     */
    public static byte[] createRequest(
            int targetWidth,
            int targetHeight,
            int outputFormat,
            int quality,
            byte[] imageData) {

        // Allocate buffer for header + payload
        byte[] request = new byte[REQUEST_HEADER_SIZE + imageData.length];
        ByteBuffer buffer = ByteBuffer.wrap(request);
        buffer.order(ByteOrder.LITTLE_ENDIAN); // Rust uses little-endian on x86/ARM

        // Write header
        buffer.put((byte) 1);              // version
        buffer.put((byte) 0);              // _reserved[0]
        buffer.put((byte) 0);              // _reserved[1]
        buffer.put((byte) 0);              // _reserved[2]
        buffer.putInt(targetWidth);        // target_width
        buffer.putInt(targetHeight);       // target_height
        buffer.putInt(outputFormat);       // output_format
        buffer.putInt(quality);            // quality
        buffer.putInt(imageData.length);   // payload_size

        // Copy image data after header
        buffer.put(imageData);

        return request;
    }

    // ========================================================================
    // Response Header Layout (20 bytes)
    // ========================================================================
    //
    //   Offset 0:  version (u8)
    //   Offset 1:  _reserved (3 bytes)
    //   Offset 4:  width (u32, little-endian)
    //   Offset 8:  height (u32, little-endian)
    //   Offset 12: format (u32, little-endian)
    //   Offset 16: payload_size (u32, little-endian)
    //
    // ========================================================================

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
     * Parse a thumbnail response from bytes.
     *
     * @param response Response bytes from plugin
     * @return Parsed ThumbnailResponse
     */
    public static ThumbnailResponse parseResponse(byte[] response) {
        // Validate minimum size
        if (response.length < RESPONSE_HEADER_SIZE) {
            throw new IllegalArgumentException(
                "Response too small: " + response.length + " bytes, need at least " +
                RESPONSE_HEADER_SIZE);
        }

        ByteBuffer buffer = ByteBuffer.wrap(response);
        buffer.order(ByteOrder.LITTLE_ENDIAN);

        // Read header
        byte version = buffer.get();
        if (version != 1) {
            throw new IllegalArgumentException("Unsupported version: " + version);
        }

        buffer.get(); // _reserved[0]
        buffer.get(); // _reserved[1]
        buffer.get(); // _reserved[2]

        int width = buffer.getInt();
        int height = buffer.getInt();
        int format = buffer.getInt();
        int payloadSize = buffer.getInt();

        // Validate total size
        int expectedSize = RESPONSE_HEADER_SIZE + payloadSize;
        if (response.length < expectedSize) {
            throw new IllegalArgumentException(
                "Response size mismatch: " + response.length +
                " bytes, expected " + expectedSize);
        }

        // Copy thumbnail data
        byte[] thumbnailData = new byte[payloadSize];
        buffer.get(thumbnailData);

        return new ThumbnailResponse(width, height, format, thumbnailData);
    }

    /**
     * Debug helper: print request header contents.
     */
    public static void debugRequest(byte[] request) {
        if (request.length < REQUEST_HEADER_SIZE) {
            System.out.println("Request too small for header");
            return;
        }

        ByteBuffer buffer = ByteBuffer.wrap(request);
        buffer.order(ByteOrder.LITTLE_ENDIAN);

        System.out.println("Request Header:");
        System.out.println("  version: " + buffer.get());
        buffer.get(); buffer.get(); buffer.get(); // skip reserved
        System.out.println("  target_width: " + buffer.getInt());
        System.out.println("  target_height: " + buffer.getInt());
        System.out.println("  output_format: " + buffer.getInt());
        System.out.println("  quality: " + buffer.getInt());
        System.out.println("  payload_size: " + buffer.getInt());
    }
}
```

## Update Main.java

Replace `src/main/java/com/example/Main.java`:

```java
package com.example;

import com.rustbridge.BundleLoader;
import com.rustbridge.jni.JniPluginLoader;
import com.example.ThumbnailStructs.ThumbnailResponse;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import static com.example.ThumbnailStructs.*;

public class Main {

    public static void main(String[] args) throws Exception {
        System.out.println("=== Binary Transport Demo (Java JNI) ===\n");

        String bundlePath = "thumbnail-plugin-1.0.0.rbp";
        String imagePath = "test-image.jpg";

        // Load the test image
        byte[] imageData = loadImage(imagePath);
        System.out.printf("Loaded image: %s (%d bytes)%n%n", imagePath, imageData.length);

        BundleLoader bundleLoader = BundleLoader.builder()
            .bundlePath(bundlePath)
            .verifySignatures(false)
            .build();

        try (var plugin = JniPluginLoader.load(bundleLoader.extractLibrary().toString())) {

            // Demo 1: Create JPEG thumbnail
            System.out.println("Demo 1: Create JPEG thumbnail (100x100)");
            {
                byte[] request = createRequest(
                    100,           // target width
                    100,           // target height
                    FORMAT_JPEG,   // output format
                    85,            // quality
                    imageData
                );

                long startTime = System.nanoTime();
                byte[] responseBytes = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
                long elapsed = System.nanoTime() - startTime;

                ThumbnailResponse response = parseResponse(responseBytes);

                System.out.printf("  Thumbnail: %dx%d %s (%d bytes)%n",
                    response.width(), response.height(),
                    response.formatName(), response.thumbnailData().length);
                System.out.printf("  Processing time: %.2f ms%n", elapsed / 1_000_000.0);

                // Save the thumbnail
                saveThumbnail(response.thumbnailData(), "thumbnail-jni-100x100.jpg");
                System.out.println("  Saved: thumbnail-jni-100x100.jpg");
            }

            // Demo 2: Create PNG thumbnail (proportional height)
            System.out.println("\nDemo 2: Create PNG thumbnail (200x0 = proportional height)");
            {
                byte[] request = createRequest(
                    200,           // target width
                    0,             // 0 = calculate proportionally
                    FORMAT_PNG,    // output format
                    0,             // quality ignored for PNG
                    imageData
                );

                long startTime = System.nanoTime();
                byte[] responseBytes = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
                long elapsed = System.nanoTime() - startTime;

                ThumbnailResponse response = parseResponse(responseBytes);

                System.out.printf("  Thumbnail: %dx%d %s (%d bytes)%n",
                    response.width(), response.height(),
                    response.formatName(), response.thumbnailData().length);
                System.out.printf("  Processing time: %.2f ms%n", elapsed / 1_000_000.0);

                saveThumbnail(response.thumbnailData(), "thumbnail-jni-200xN.png");
                System.out.println("  Saved: thumbnail-jni-200xN.png");
            }

            // Demo 3: Different quality settings
            System.out.println("\nDemo 3: Quality comparison (JPEG at 20, 50, 90)");
            for (int quality : new int[]{20, 50, 90}) {
                byte[] request = createRequest(150, 150, FORMAT_JPEG, quality, imageData);
                byte[] responseBytes = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
                ThumbnailResponse response = parseResponse(responseBytes);

                System.out.printf("  Quality %d: %d bytes%n",
                    quality, response.thumbnailData().length);

                saveThumbnail(response.thumbnailData(),
                    "thumbnail-jni-q" + quality + ".jpg");
            }

            // Demo 4: Performance comparison
            System.out.println("\nDemo 4: Performance comparison (10 iterations)");
            int iterations = 10;

            byte[] request = createRequest(100, 100, FORMAT_JPEG, 80, imageData);

            // Warm up
            for (int i = 0; i < 3; i++) {
                plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
            }

            // Measure
            long totalTime = 0;
            for (int i = 0; i < iterations; i++) {
                long start = System.nanoTime();
                plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
                totalTime += System.nanoTime() - start;
            }

            double avgMs = (totalTime / iterations) / 1_000_000.0;
            System.out.printf("  Average time per thumbnail: %.2f ms%n", avgMs);
            System.out.printf("  Throughput: %.1f thumbnails/sec%n", 1000.0 / avgMs);
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
    implementation("com.rustbridge:rustbridge-jni:0.7.0")
    implementation("com.google.code.gson:gson:2.11.0")
}

java {
    toolchain {
        languageVersion.set(JavaLanguageVersion.of(17))
    }
}

application {
    mainClass.set("com.example.Main")
}
```

## Run the Demo

```bash
./gradlew run
```

Expected output:

```
=== Binary Transport Demo (Java JNI) ===

Loaded image: test-image.jpg (45678 bytes)

Demo 1: Create JPEG thumbnail (100x100)
  Thumbnail: 100x75 JPEG (2847 bytes)
  Processing time: 12.56 ms
  Saved: thumbnail-jni-100x100.jpg

Demo 2: Create PNG thumbnail (200x0 = proportional height)
  Thumbnail: 200x150 PNG (18234 bytes)
  Processing time: 15.89 ms
  Saved: thumbnail-jni-200xN.png

Demo 3: Quality comparison (JPEG at 20, 50, 90)
  Quality 20: 1234 bytes
  Quality 50: 2156 bytes
  Quality 90: 4567 bytes

Demo 4: Performance comparison (10 iterations)
  Average time per thumbnail: 8.67 ms
  Throughput: 115.4 thumbnails/sec

=== Demo Complete ===
```

## Key Observations

### ByteBuffer vs FFM

JNI uses ByteBuffer for binary data, which is simpler but less type-safe than FFM:

```java
// FFM: Type-safe struct layout
public static final StructLayout REQUEST_HEADER_LAYOUT = MemoryLayout.structLayout(
    ValueLayout.JAVA_BYTE.withName("version"),
    // ...
);

// JNI: Manual byte manipulation
ByteBuffer buffer = ByteBuffer.wrap(request);
buffer.order(ByteOrder.LITTLE_ENDIAN);
buffer.put((byte) 1);              // version
buffer.putInt(targetWidth);        // target_width
```

**Trade-offs:**
- FFM: Type-safe, compiler-checked layouts, better error messages
- JNI: Simpler setup, works with Java 17+, familiar ByteBuffer API

### Byte Order

Rust uses little-endian on x86 and ARM. Always set ByteBuffer order:

```java
buffer.order(ByteOrder.LITTLE_ENDIAN);
```

### Memory Management

JNI copies data to/from native memory automatically:

```java
// Request: Java byte[] copied to native memory by JNI
byte[] request = createRequest(...);
byte[] response = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
// Response: Copied from native memory to Java byte[]
// No manual memory management needed!
```

This is simpler than FFM but involves an extra copy.

### Struct Validation

Always validate response data:

```java
public static ThumbnailResponse parseResponse(byte[] response) {
    // 1. Validate minimum size
    if (response.length < RESPONSE_HEADER_SIZE) {
        throw new IllegalArgumentException("Response too small");
    }

    // 2. Check version
    byte version = buffer.get();
    if (version != 1) {
        throw new IllegalArgumentException("Unsupported version: " + version);
    }

    // 3. Validate payload size matches
    int expectedSize = RESPONSE_HEADER_SIZE + payloadSize;
    if (response.length < expectedSize) {
        throw new IllegalArgumentException("Response size mismatch");
    }
}
```

## Error Handling

```java
try {
    byte[] response = plugin.callRaw(MSG_THUMBNAIL_CREATE, request);
    ThumbnailResponse result = parseResponse(response);
} catch (PluginException e) {
    switch (e.getErrorCode()) {
        case 2 -> System.err.println("Invalid request: " + e.getMessage());
        case 4 -> System.err.println("Handler not found for message ID");
        case 5 -> System.err.println("Processing error: " + e.getMessage());
        default -> System.err.println("Unknown error: " + e);
    }
}
```

## Comparison: FFM vs JNI

| Aspect | FFM (Java 21+) | JNI (Java 17+) |
|--------|----------------|----------------|
| Java version | 21+ | 17+ |
| Type safety | High (StructLayout) | Low (manual offsets) |
| Memory management | Manual (Arena, freeBuffer) | Automatic (JNI copies) |
| Performance | Better (zero-copy possible) | Good (extra copy) |
| Complexity | Higher | Lower |
| Error messages | Better (named fields) | Basic |

**Recommendation:**
- Use FFM for new projects on Java 21+
- Use JNI for compatibility with Java 17-20
- JNI is sufficient for most binary transport use cases

## What's Next?

Continue to the Kotlin implementation, which uses FFM with idiomatic Kotlin patterns.

[Continue to Section 3: Kotlin Consumer](./03-kotlin-consumer.md)
