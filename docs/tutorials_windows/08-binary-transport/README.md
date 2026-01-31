# Chapter 8: Binary Transport

In this chapter, you'll build an image thumbnail generator plugin that uses binary transport for efficient data transfer.

## What You'll Build

A plugin that:
- Accepts raw image bytes (JPEG, PNG, WebP)
- Resizes images to specified dimensions
- Returns thumbnail bytes directly (no base64 encoding)

## Why Binary Transport?

| Metric | JSON | Binary | Improvement |
|--------|------|--------|-------------|
| Latency (small payload) | ~650 ns | ~90 ns | **7.1x faster** |
| Image size (10 KB) | ~13.5 KB | ~10.02 KB | **26% smaller** |

Binary transport bypasses JSON serialization overhead, making it ideal for large payloads.

## Project Setup

```powershell
cd $env:USERPROFILE\rustbridge-workspace

rustbridge new thumbnail-plugin --all
cd thumbnail-plugin
```

## Binary Message Types

Binary messages use `#[repr(C)]` structs with fixed layouts. See the [Linux tutorial](../tutorials/08-binary-transport/README.md#binary-message-types) for complete struct definitions.

## Build and Bundle

```powershell
# Build the plugin
cargo build --release

# Create a bundle
rustbridge bundle create `
  --name thumbnail-plugin `
  --version 1.0.0 `
  --lib windows-x86_64:target\release\thumbnail_plugin.dll `
  --output thumbnail-plugin-1.0.0.rbp

# Copy to each consumer directory
Copy-Item thumbnail-plugin-1.0.0.rbp consumers\kotlin\
Copy-Item thumbnail-plugin-1.0.0.rbp consumers\java-ffm\
Copy-Item thumbnail-plugin-1.0.0.rbp consumers\java-jni\
Copy-Item thumbnail-plugin-1.0.0.rbp consumers\csharp\
Copy-Item thumbnail-plugin-1.0.0.rbp consumers\python\
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

### [04: C# Consumer](./04-csharp-consumer.md)

C# using `StructLayout` and unsafe code for memory mapping.

### [05: Python Consumer](./05-python-consumer.md)

Python using ctypes for struct definitions.

## Key Concepts

### Header + Payload Pattern

```
+------------------+------------------------+
| Header (fixed)   | Payload (variable)     |
| 24 bytes         | N bytes (from header)  |
+------------------+------------------------+
```

### Memory Safety

1. **Validate sizes**: Check buffer sizes before casting
2. **Check versions**: Reject unknown versions
3. **Rust allocates, host frees**: Copy response data immediately

## What You'll Learn

- When to use binary vs JSON transport
- Designing C-compatible struct layouts
- Binary buffer handling in each language
- Performance optimization

## Next Steps

For Java/Kotlin implementations, see the [Linux tutorial](../tutorials/08-binary-transport/) as those sections don't require Windows-specific changes.
