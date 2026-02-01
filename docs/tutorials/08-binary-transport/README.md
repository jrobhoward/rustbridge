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

```bash
cd ~/rustbridge-workspace

rustbridge new thumbnail-plugin --all
cd thumbnail-plugin
```

This creates:

```
thumbnail-plugin/
+-- Cargo.toml                      # Rust plugin
+-- src/
|   +-- lib.rs                      # Plugin implementation
+-- consumers/
    +-- kotlin/                     # Kotlin/FFM consumer
    +-- java-ffm/                   # Java FFM consumer
    +-- java-jni/                   # Java JNI consumer
    +-- csharp/                     # C# consumer
    +-- python/                     # Python consumer
```

## Binary Message Types

Binary messages use `#[repr(C)]` structs with fixed layouts. Let's define the thumbnail request and response types.

### Message ID

```rust
/// Message ID for thumbnail creation
pub const MSG_THUMBNAIL_CREATE: u32 = 100;
```

### Request Header

The request consists of a fixed header followed by variable-length image data:

```rust
/// Request header for thumbnail creation (24 bytes)
///
/// Layout:
///   Offset 0:  version (1 byte)
///   Offset 1:  _reserved (3 bytes, padding for alignment)
///   Offset 4:  target_width (4 bytes, u32)
///   Offset 8:  target_height (4 bytes, u32)
///   Offset 12: output_format (4 bytes, u32: 0=JPEG, 1=PNG, 2=WebP)
///   Offset 16: quality (4 bytes, u32: 1-100 for JPEG/WebP)
///   Offset 20: payload_size (4 bytes, u32: size of image data following header)
///   Total: 24 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThumbnailRequestHeader {
    /// Struct version for forward compatibility
    pub version: u8,
    /// Reserved for alignment
    pub _reserved: [u8; 3],
    /// Desired thumbnail width (0 = proportional to height)
    pub target_width: u32,
    /// Desired thumbnail height (0 = proportional to width)
    pub target_height: u32,
    /// Output format: 0=JPEG, 1=PNG, 2=WebP
    pub output_format: u32,
    /// Quality for JPEG/WebP (1-100), ignored for PNG
    pub quality: u32,
    /// Size of image data following this header
    pub payload_size: u32,
}

impl ThumbnailRequestHeader {
    pub const VERSION: u8 = 1;
    pub const SIZE: usize = 24;

    // Output format constants
    pub const FORMAT_JPEG: u32 = 0;
    pub const FORMAT_PNG: u32 = 1;
    pub const FORMAT_WEBP: u32 = 2;
}
```

### Response Header

The response also uses a header followed by the thumbnail data:

```rust
/// Response header for thumbnail creation (20 bytes)
///
/// Layout:
///   Offset 0:  version (1 byte)
///   Offset 1:  _reserved (3 bytes, padding for alignment)
///   Offset 4:  width (4 bytes, u32: actual thumbnail width)
///   Offset 8:  height (4 bytes, u32: actual thumbnail height)
///   Offset 12: format (4 bytes, u32: output format used)
///   Offset 16: payload_size (4 bytes, u32: size of thumbnail data following header)
///   Total: 20 bytes
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThumbnailResponseHeader {
    /// Struct version
    pub version: u8,
    /// Reserved for alignment
    pub _reserved: [u8; 3],
    /// Actual thumbnail width
    pub width: u32,
    /// Actual thumbnail height
    pub height: u32,
    /// Output format used
    pub format: u32,
    /// Size of thumbnail data following this header
    pub payload_size: u32,
}

impl ThumbnailResponseHeader {
    pub const VERSION: u8 = 1;
    pub const SIZE: usize = 20;
}
```

### Struct Layout Rules

Binary structs follow these rules for cross-language compatibility:

1. **Version field first**: Always include `version: u8` for forward compatibility
2. **Alignment padding**: Use `_reserved: [u8; N]` to align to 4-byte boundaries
3. **Little-endian**: All numeric types use little-endian byte order (Rust default on x86/ARM)
4. **Header + payload pattern**: Variable-length data follows a fixed header with `payload_size`
5. **No pointers**: Use inline arrays or header + payload, never heap allocations

## Plugin Implementation

Replace `src/lib.rs` with the thumbnail plugin:

```rust
//! thumbnail-plugin - Image thumbnail generator using binary transport

use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing, PluginHandle, register_binary_handler};
use std::io::Cursor;

// ============================================================================
// Message Types (JSON - for comparison)
// ============================================================================

/// JSON request for echo (for testing)
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "echo")]
pub struct EchoRequest {
    pub message: String,
}

/// JSON response for echo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResponse {
    pub message: String,
    pub length: usize,
}

// ============================================================================
// Binary Message Types
// ============================================================================

/// Message ID for thumbnail creation
pub const MSG_THUMBNAIL_CREATE: u32 = 100;

/// Request header for thumbnail creation (24 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThumbnailRequestHeader {
    pub version: u8,
    pub _reserved: [u8; 3],
    pub target_width: u32,
    pub target_height: u32,
    pub output_format: u32,
    pub quality: u32,
    pub payload_size: u32,
}

impl ThumbnailRequestHeader {
    pub const VERSION: u8 = 1;
    pub const SIZE: usize = 24;

    pub const FORMAT_JPEG: u32 = 0;
    pub const FORMAT_PNG: u32 = 1;
    pub const FORMAT_WEBP: u32 = 2;
}

/// Response header for thumbnail creation (20 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ThumbnailResponseHeader {
    pub version: u8,
    pub _reserved: [u8; 3],
    pub width: u32,
    pub height: u32,
    pub format: u32,
    pub payload_size: u32,
}

impl ThumbnailResponseHeader {
    pub const VERSION: u8 = 1;
    pub const SIZE: usize = 20;
}

// ============================================================================
// Plugin Implementation
// ============================================================================

#[derive(Default)]
pub struct ThumbnailPlugin;

impl ThumbnailPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_echo(&self, req: EchoRequest) -> PluginResult<EchoResponse> {
        tracing::debug!("Handling echo: {:?}", req);
        Ok(EchoResponse {
            length: req.message.len(),
            message: req.message,
        })
    }
}

#[async_trait]
impl Plugin for ThumbnailPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("thumbnail-plugin starting...");

        // Register binary message handlers
        register_binary_handlers();

        tracing::info!("thumbnail-plugin started successfully");
        Ok(())
    }

    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        match type_tag {
            "echo" => {
                let req: EchoRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_echo(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("thumbnail-plugin stopped");
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["echo"]
    }
}

// ============================================================================
// Binary Handler
// ============================================================================

fn handle_thumbnail_create(_handle: &PluginHandle, request: &[u8]) -> PluginResult<Vec<u8>> {
    // Validate minimum size
    if request.len() < ThumbnailRequestHeader::SIZE {
        return Err(PluginError::HandlerError(format!(
            "Request too small: {} bytes, need at least {}",
            request.len(),
            ThumbnailRequestHeader::SIZE
        )));
    }

    // Parse header (safe: validated size, repr(C) struct)
    let header = unsafe { &*(request.as_ptr() as *const ThumbnailRequestHeader) };

    // Validate version
    if header.version != ThumbnailRequestHeader::VERSION {
        return Err(PluginError::HandlerError(format!(
            "Unsupported version: {}, expected {}",
            header.version,
            ThumbnailRequestHeader::VERSION
        )));
    }

    // Validate payload size
    let expected_size = ThumbnailRequestHeader::SIZE + header.payload_size as usize;
    if request.len() < expected_size {
        return Err(PluginError::HandlerError(format!(
            "Request size mismatch: {} bytes, expected {}",
            request.len(),
            expected_size
        )));
    }

    // Extract image data
    let image_data = &request[ThumbnailRequestHeader::SIZE..expected_size];

    tracing::debug!(
        "Creating thumbnail: {}x{}, format={}, quality={}, input_size={}",
        header.target_width,
        header.target_height,
        header.output_format,
        header.quality,
        image_data.len()
    );

    // Decode the input image
    let img = image::load_from_memory(image_data)
        .map_err(|e| PluginError::HandlerError(format!("Failed to decode image: {}", e)))?;

    // Calculate target dimensions
    let (target_w, target_h) = match (header.target_width, header.target_height) {
        (0, 0) => (100, 100), // Default to 100x100
        (0, h) => {
            // Proportional width
            let ratio = h as f32 / img.height() as f32;
            ((img.width() as f32 * ratio) as u32, h)
        }
        (w, 0) => {
            // Proportional height
            let ratio = w as f32 / img.width() as f32;
            (w, (img.height() as f32 * ratio) as u32)
        }
        (w, h) => (w, h),
    };

    // Resize the image
    let thumbnail = img.thumbnail(target_w, target_h);

    // Encode to requested format
    let mut output = Vec::new();
    let format = match header.output_format {
        ThumbnailRequestHeader::FORMAT_PNG => {
            thumbnail
                .write_to(&mut Cursor::new(&mut output), image::ImageFormat::Png)
                .map_err(|e| PluginError::HandlerError(format!("Failed to encode PNG: {}", e)))?;
            ThumbnailRequestHeader::FORMAT_PNG
        }
        ThumbnailRequestHeader::FORMAT_WEBP => {
            // WebP requires the webp feature; fall back to JPEG
            let quality = header.quality.clamp(1, 100) as u8;
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality);
            thumbnail
                .write_with_encoder(encoder)
                .map_err(|e| PluginError::HandlerError(format!("Failed to encode JPEG: {}", e)))?;
            ThumbnailRequestHeader::FORMAT_JPEG // Fallback
        }
        _ => {
            // Default to JPEG
            let quality = header.quality.clamp(1, 100) as u8;
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality);
            thumbnail
                .write_with_encoder(encoder)
                .map_err(|e| PluginError::HandlerError(format!("Failed to encode JPEG: {}", e)))?;
            ThumbnailRequestHeader::FORMAT_JPEG
        }
    };

    tracing::debug!(
        "Thumbnail created: {}x{}, output_size={}",
        thumbnail.width(),
        thumbnail.height(),
        output.len()
    );

    // Build response: header + thumbnail bytes
    let response_header = ThumbnailResponseHeader {
        version: ThumbnailResponseHeader::VERSION,
        _reserved: [0; 3],
        width: thumbnail.width(),
        height: thumbnail.height(),
        format,
        payload_size: output.len() as u32,
    };

    // Serialize response header
    let header_bytes = unsafe {
        std::slice::from_raw_parts(
            &response_header as *const _ as *const u8,
            ThumbnailResponseHeader::SIZE,
        )
    };

    // Combine header + thumbnail
    let mut response = Vec::with_capacity(ThumbnailResponseHeader::SIZE + output.len());
    response.extend_from_slice(header_bytes);
    response.extend_from_slice(&output);

    Ok(response)
}

// ============================================================================
// FFI Entry Points
// ============================================================================

/// Register all binary message handlers
fn register_binary_handlers() {
    register_binary_handler(MSG_THUMBNAIL_CREATE, handle_thumbnail_create);
}

rustbridge_entry!(ThumbnailPlugin::new);
pub use rustbridge::ffi_exports::*;
```

## Add Dependencies

Update `Cargo.toml` to add the `image` crate:

```toml
[package]
name = "thumbnail-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
rustbridge = { version = "0.7.0" }
image = { version = "0.25", default-features = false, features = ["jpeg", "png"] }

[profile.release]
lto = true
codegen-units = 1
```

## Build and Bundle

```bash
# Build the plugin
cargo build --release

# Create a bundle
rustbridge bundle create \
  --name thumbnail-plugin \
  --version 0.1.0 \
  --lib linux-x86_64:target/release/libthumbnail_plugin.so \
  --output thumbnail-plugin-0.1.0.rbp

# Copy to each consumer directory
cp thumbnail-plugin-0.1.0.rbp consumers/kotlin/
cp thumbnail-plugin-0.1.0.rbp consumers/java-ffm/
cp thumbnail-plugin-0.1.0.rbp consumers/java-jni/
cp thumbnail-plugin-0.1.0.rbp consumers/csharp/
cp thumbnail-plugin-0.1.0.rbp consumers/python/
```

> **Note**: Adjust the `--lib` platform identifier for your OS:
> - Linux: `linux-x86_64:target/release/libthumbnail_plugin.so`
> - macOS (Intel): `darwin-x86_64:target/release/libthumbnail_plugin.dylib`
> - macOS (Apple Silicon): `darwin-aarch64:target/release/libthumbnail_plugin.dylib`
> - Windows: `windows-x86_64:target/release/thumbnail_plugin.dll`

## Create a Test Image

For testing, create a small test image. You can use any JPEG or PNG file, or create one programmatically:

```bash
# Option 1: Download a sample image
curl -L -o consumers/test-image.jpg \
  "https://picsum.photos/800/600.jpg"

# Option 2: Create a test image with ImageMagick (if installed)
convert -size 800x600 gradient:blue-red consumers/test-image.jpg
```

Copy to all consumer directories:

```bash
cp consumers/test-image.jpg consumers/kotlin/
cp consumers/test-image.jpg consumers/java-ffm/
cp consumers/test-image.jpg consumers/java-jni/
cp consumers/test-image.jpg consumers/csharp/
cp consumers/test-image.jpg consumers/python/
```

## Sections

Implement the binary transport consumer in each language:

### [01: Java FFM Consumer](./01-java-ffm-consumer.md)

Java 21+ using the Foreign Function & Memory API for direct struct manipulation.

### [02: Java JNI Consumer](./02-java-jni-consumer.md)

Java 17+ using JNI with ByteBuffer for binary data handling.

### [03: Kotlin Consumer](./03-kotlin-consumer.md)

Kotlin with FFM and idiomatic extension functions.

### [04: C# Consumer](./04-csharp-consumer.md)

C# using unsafe structs and StructLayout for memory mapping.

### [05: Python Consumer](./05-python-consumer.md)

Python using ctypes for struct definitions and binary data handling.

## Prerequisites

Before starting this chapter:

- **Completed Chapter 1** (understanding plugin structure and message types)
- **Read docs/TRANSPORT.md** (binary transport concepts)
- **Language-specific setup**:
  - Java FFM: JDK 21+
  - Java JNI: JDK 17+
  - Kotlin: JDK 21+
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
