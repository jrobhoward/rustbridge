# Chapter 8: Binary Transport

In this chapter, you'll build an image thumbnail generator plugin that uses binary transport for efficient data transfer. Binary transport bypasses JSON serialization overhead, making it ideal for large payloads like images where base64 encoding would add ~33% size overhead.

## What You'll Build

```
+-----------------------------------------------------------------------------+
|                       Binary Transport Architecture                          |
+-----------------------------------------------------------------------------+
|                                                                              |
|  Host Application                                                            |
|  ----------------                                                            |
|                                                                              |
|  +------------------------+       +-----------------------------------+      |
|  | Image File (10 KB)     |       | ThumbnailRequestHeader (24 bytes) |      |
|  | PNG/JPEG/WebP          |  -->  | + Raw Image Bytes (10 KB)         |      |
|  +------------------------+       +-----------------------------------+      |
|                                              |                               |
|                                              | plugin.call_raw(100, data)   |
|                                              v                               |
|  +-----------------------------------------------------------------------------+
|  |                           FFI Boundary                                      |
|  +-----------------------------------------------------------------------------+
|                                              |                               |
|                                              v                               |
|  Rust Plugin                                                                 |
|  -----------                                                                 |
|                                                                              |
|  +-----------------------------------+       +---------------------------+   |
|  | Parse header (24 bytes)           |       | ThumbnailResponseHeader   |   |
|  | Decode image (image crate)        |  -->  | (20 bytes) + Thumbnail    |   |
|  | Resize to target dimensions       |       | Bytes (2 KB)              |   |
|  | Encode to output format           |       +---------------------------+   |
|  +-----------------------------------+                                       |
|                                                                              |
+-----------------------------------------------------------------------------+
```

## Why Binary Transport?

### JSON Transport (Default)

JSON is the default transport for rustbridge. It's flexible, human-readable, and works well for most use cases:

```rust
// Request: {"image_data": "base64_encoded_string...", "width": 100, "height": 100}
// Response: {"thumbnail": "base64_encoded_thumbnail...", "width": 100, "height": 75}
```

For a 10 KB image:
- Base64 encoding adds ~33% overhead (10 KB becomes ~13.3 KB)
- JSON parsing/serialization adds ~650 ns latency
- Total request size: ~13.5 KB (with JSON envelope)

### Binary Transport (Opt-in)

Binary transport uses C-compatible structs for direct memory transfer:

```rust
// Request: 24-byte header + raw image bytes (10 KB)
// Response: 20-byte header + raw thumbnail bytes
```

For the same 10 KB image:
- No encoding overhead (10 KB stays 10 KB)
- Minimal parsing (~90 ns latency)
- Total request size: ~10 KB (header + raw bytes)

### Performance Comparison

| Metric | JSON | Binary | Improvement |
|--------|------|--------|-------------|
| Latency (small payload) | ~650 ns | ~90 ns | **7.1x faster** |
| Image size (10 KB) | ~13.5 KB | ~10.02 KB | **26% smaller** |
| Image size (1 MB) | ~1.33 MB | ~1.00 MB | **25% smaller** |

### When to Use Binary Transport

Binary transport is ideal when:
- Handling large binary payloads (images, audio, video, files)
- High-frequency calls (>10K ops/sec)
- Message structure is fixed and well-defined
- Performance is more important than flexibility

When NOT to use binary transport:
- Schema flexibility is needed (evolving APIs)
- Debugging readability matters
- Small payloads where JSON overhead is negligible
- Cross-language compatibility is the priority

## Project Setup

Scaffold a new project with all consumer types:

```powershell
cd $env:USERPROFILE\rustbridge-workspace

rustbridge new thumbnail-plugin --all
cd thumbnail-plugin
```

This creates:

```
thumbnail-plugin\
+-- Cargo.toml                      # Rust plugin
+-- src\
|   +-- lib.rs                      # Plugin implementation
+-- consumers\
    +-- kotlin\                     # Kotlin/FFM consumer
    +-- java-ffm\                   # Java FFM consumer
    +-- java-jni\                   # Java JNI consumer
    +-- csharp\                     # C# consumer
    +-- python\                     # Python consumer
```

## Binary Message Types

Binary messages use `#[repr(C)]` structs with fixed layouts. See the plugin implementation in the [Linux tutorial](../../tutorials/08-binary-transport/README.md#binary-message-types) for complete struct definitions and Rust code.

## Build and Bundle

```powershell
# Build the plugin
cargo build --release

# Create a bundle
rustbridge bundle create `
  --name thumbnail-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\thumbnail_plugin.dll `
  --output thumbnail-plugin-0.1.0.rbp

# Copy to each consumer directory
Copy-Item thumbnail-plugin-0.1.0.rbp consumers\kotlin\
Copy-Item thumbnail-plugin-0.1.0.rbp consumers\java-ffm\
Copy-Item thumbnail-plugin-0.1.0.rbp consumers\java-jni\
Copy-Item thumbnail-plugin-0.1.0.rbp consumers\csharp\
Copy-Item thumbnail-plugin-0.1.0.rbp consumers\python\
```

## Create a Test Image

Download or copy a test image:

```powershell
# Copy a test image to consumer directories
Copy-Item path\to\test-image.jpg consumers\kotlin\
Copy-Item path\to\test-image.jpg consumers\java-ffm\
Copy-Item path\to\test-image.jpg consumers\java-jni\
Copy-Item path\to\test-image.jpg consumers\csharp\
Copy-Item path\to\test-image.jpg consumers\python\
```

## Sections

Implement the binary transport consumer in each language:

### [01: Java FFM Consumer](./01-java-ffm-consumer.md)

Java 22+ using the Foreign Function & Memory API for direct struct manipulation.

### [02: Java JNI Consumer](./02-java-jni-consumer.md)

Java 17+ using JNI with ByteBuffer for binary data handling (for Java 22+, prefer FFM).

### [03: Kotlin Consumer](./03-kotlin-consumer.md)

Kotlin with FFM (requires Java 22+) and idiomatic extension functions.

### [04: C# Consumer](./04-csharp-consumer.md)

C# using unsafe structs and StructLayout for memory mapping.

### [05: Python Consumer](./05-python-consumer.md)

Python using ctypes for struct definitions and binary data handling.

## Prerequisites

Before starting this chapter:

- **Completed Chapter 1** (understanding plugin structure and message types)
- **Read docs/TRANSPORT.md** (binary transport concepts)
- **Language-specific setup**:
  - Java FFM: JDK 22+ (recommended)
  - Java JNI: JDK 17+ (use FFM for 22+)
  - Kotlin: JDK 22+ (uses FFM)
  - C#: .NET 8.0+
  - Python: Python 3.10+

## Key Concepts

### Header + Payload Pattern

Binary messages with variable-length data use a fixed header followed by the payload:

```
+------------------+------------------------+
| Header (fixed)   | Payload (variable)     |
| 24 bytes         | N bytes (from header)  |
+------------------+------------------------+
```

The header contains a `payload_size` field that tells the receiver how many bytes follow.

### Version Field

Every binary struct starts with a `version: u8` field:

```rust
#[repr(C)]
pub struct MyRequest {
    pub version: u8,       // Always first
    pub _reserved: [u8; 3], // Alignment padding
    // ... other fields
}
```

This allows handlers to support multiple versions and reject unknown versions gracefully.

### Memory Safety

Binary transport requires careful memory handling:

1. **Validate sizes**: Always check buffer sizes before casting to structs
2. **Check versions**: Reject unknown versions early
3. **Rust allocates, host frees**: The plugin allocates response memory; the host must free it
4. **Copy immediately**: Copy response data to managed memory before freeing

## What You'll Learn

By completing this chapter, you'll understand:

- When to use binary transport vs JSON
- Designing C-compatible struct layouts
- Header + payload pattern for variable data
- Binary buffer handling in each language
- Memory ownership across the FFI boundary
- Performance comparison between transports

## Next Steps

Start with the Java FFM consumer, which provides the clearest example of struct layout handling.

[Continue to Section 1: Java FFM Consumer](./01-java-ffm-consumer.md)
