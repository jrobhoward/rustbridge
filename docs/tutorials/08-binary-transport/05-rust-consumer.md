# Section 5: Rust Consumer

In this section, you'll implement binary transport in Rust using `rustbridge-consumer`. This demonstrates how to use the same binary protocol from a Rust host application.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new thumbnail-plugin --all`
2. Replace `src/lib.rs` with the thumbnail plugin implementation
3. Add the `image` dependency to `Cargo.toml`
4. Build the plugin and create the bundle

## Create a Rust Consumer Project

Unlike other language consumers (which live under `consumers/`), the Rust consumer should be created as a
**separate standalone project**. This avoids Cargo workspace conflicts that occur when placing a Rust
project inside another Rust project.

```bash
cd $RUSTBRIDGE_WORKSPACE
cargo new thumbnail-rust-consumer
cd thumbnail-rust-consumer
```

## Add Dependencies

Update `Cargo.toml`:

```toml
[package]
name = "thumbnail-consumer"
version = "0.1.0"
edition = "2024"

[dependencies]
rustbridge-consumer = "1.0.0"
```

## Define Binary Structs

Create the binary struct definitions matching the plugin. These mirror the Rust plugin's types:

```rust
//! thumbnail-consumer - Binary transport demo

use rustbridge_consumer::{ConsumerError, ConsumerResult, NativePluginLoader};
use std::fs;
use std::io::Write;
use std::time::Instant;

// ============================================================================
// Binary Message Types (matching the plugin)
// ============================================================================

/// Message ID for thumbnail creation
const MSG_THUMBNAIL_CREATE: u32 = 100;

/// Output format constants
const FORMAT_JPEG: u32 = 0;
const FORMAT_PNG: u32 = 1;
#[allow(dead_code)]
const FORMAT_WEBP: u32 = 2;

/// Request header for thumbnail creation (24 bytes)
///
/// Layout:
///   Offset 0:  version (1 byte)
///   Offset 1:  _reserved (3 bytes)
///   Offset 4:  target_width (4 bytes)
///   Offset 8:  target_height (4 bytes)
///   Offset 12: output_format (4 bytes)
///   Offset 16: quality (4 bytes)
///   Offset 20: payload_size (4 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ThumbnailRequestHeader {
    version: u8,
    _reserved: [u8; 3],
    target_width: u32,
    target_height: u32,
    output_format: u32,
    quality: u32,
    payload_size: u32,
}

impl ThumbnailRequestHeader {
    const VERSION: u8 = 1;
    const SIZE: usize = 24;

    fn new(target_width: u32, target_height: u32, output_format: u32, quality: u32, payload_size: u32) -> Self {
        Self {
            version: Self::VERSION,
            _reserved: [0; 3],
            target_width,
            target_height,
            output_format,
            quality,
            payload_size,
        }
    }

    fn to_bytes(self) -> [u8; Self::SIZE] {
        // SAFETY: ThumbnailRequestHeader is repr(C) with known layout
        unsafe { std::mem::transmute(self) }
    }
}

/// Response header for thumbnail creation (20 bytes)
///
/// Layout:
///   Offset 0:  version (1 byte)
///   Offset 1:  _reserved (3 bytes)
///   Offset 4:  width (4 bytes)
///   Offset 8:  height (4 bytes)
///   Offset 12: format (4 bytes)
///   Offset 16: payload_size (4 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ThumbnailResponseHeader {
    version: u8,
    _reserved: [u8; 3],
    width: u32,
    height: u32,
    format: u32,
    payload_size: u32,
}

impl ThumbnailResponseHeader {
    const VERSION: u8 = 1;
    const SIZE: usize = 20;

    fn from_bytes(data: &[u8]) -> Option<&Self> {
        if data.len() < Self::SIZE {
            return None;
        }
        // SAFETY: Validated size, repr(C) struct
        Some(unsafe { &*(data.as_ptr() as *const Self) })
    }

    fn format_name(&self) -> &'static str {
        match self.format {
            FORMAT_JPEG => "JPEG",
            FORMAT_PNG => "PNG",
            FORMAT_WEBP => "WebP",
            _ => "Unknown",
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Create a thumbnail request with image data.
fn create_request(
    target_width: u32,
    target_height: u32,
    output_format: u32,
    quality: u32,
    image_data: &[u8],
) -> Vec<u8> {
    let header = ThumbnailRequestHeader::new(
        target_width,
        target_height,
        output_format,
        quality,
        image_data.len() as u32,
    );

    let mut request = Vec::with_capacity(ThumbnailRequestHeader::SIZE + image_data.len());
    request.extend_from_slice(&header.to_bytes());
    request.extend_from_slice(image_data);
    request
}

/// Parse a thumbnail response.
fn parse_response(data: &[u8]) -> Result<(ThumbnailResponseHeader, &[u8]), ConsumerError> {
    let header = ThumbnailResponseHeader::from_bytes(data)
        .ok_or_else(|| ConsumerError::InvalidResponse(format!(
            "Response too small: {} bytes", data.len()
        )))?;

    if header.version != ThumbnailResponseHeader::VERSION {
        return Err(ConsumerError::InvalidResponse(format!(
            "Unsupported version: {}", header.version
        )));
    }

    let expected_size = ThumbnailResponseHeader::SIZE + header.payload_size as usize;
    if data.len() < expected_size {
        return Err(ConsumerError::InvalidResponse(format!(
            "Response size mismatch: {} bytes, expected {}",
            data.len(),
            expected_size
        )));
    }

    let thumbnail_data = &data[ThumbnailResponseHeader::SIZE..expected_size];
    Ok((*header, thumbnail_data))
}

// ============================================================================
// Demo
// ============================================================================

fn main() -> ConsumerResult<()> {
    println!("=== Binary Transport Demo (Rust) ===\n");

    // Load the plugin from bundle
    let bundle_path = "../thumbnail-plugin/thumbnail-plugin-0.1.0.rbp";
    let image_path = "test-image.jpg";

    // Load the test image
    let image_data = fs::read(image_path)?;
    println!("Loaded image: {} ({} bytes)\n", image_path, image_data.len());

    // Load the plugin
    let plugin = NativePluginLoader::load_bundle(bundle_path)?;

    // Check if binary transport is available
    if !plugin.has_binary_transport() {
        return Err(ConsumerError::MissingSymbol(
            "Binary transport not available in this plugin".into(),
        ));
    }

    // Demo 1: Create JPEG thumbnail
    println!("Demo 1: Create JPEG thumbnail (100x100)");
    {
        let request = create_request(100, 100, FORMAT_JPEG, 85, &image_data);

        let start = Instant::now();
        let response = plugin.call_raw(MSG_THUMBNAIL_CREATE, &request)?;
        let elapsed = start.elapsed();

        let (header, thumbnail_data) = parse_response(&response)?;

        println!(
            "  Thumbnail: {}x{} {} ({} bytes)",
            header.width,
            header.height,
            header.format_name(),
            thumbnail_data.len()
        );
        println!("  Processing time: {:.2?}", elapsed);

        // Save the thumbnail
        let mut file = fs::File::create("thumbnail-100x100.jpg")?;
        file.write_all(thumbnail_data)?;
        println!("  Saved: thumbnail-100x100.jpg");
    }

    // Demo 2: Create PNG thumbnail (proportional height)
    println!("\nDemo 2: Create PNG thumbnail (200x0 = proportional height)");
    {
        let request = create_request(200, 0, FORMAT_PNG, 0, &image_data);

        let start = Instant::now();
        let response = plugin.call_raw(MSG_THUMBNAIL_CREATE, &request)?;
        let elapsed = start.elapsed();

        let (header, thumbnail_data) = parse_response(&response)?;

        println!(
            "  Thumbnail: {}x{} {} ({} bytes)",
            header.width,
            header.height,
            header.format_name(),
            thumbnail_data.len()
        );
        println!("  Processing time: {:.2?}", elapsed);

        let mut file = fs::File::create("thumbnail-200xN.png")?;
        file.write_all(thumbnail_data)?;
        println!("  Saved: thumbnail-200xN.png");
    }

    // Demo 3: Performance comparison
    println!("\nDemo 3: Performance comparison (10 iterations)");
    {
        let iterations = 10;
        let request = create_request(100, 100, FORMAT_JPEG, 80, &image_data);

        // Warm up
        for _ in 0..3 {
            let _ = plugin.call_raw(MSG_THUMBNAIL_CREATE, &request)?;
        }

        // Measure
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = plugin.call_raw(MSG_THUMBNAIL_CREATE, &request)?;
        }
        let total = start.elapsed();

        let avg_ms = total.as_secs_f64() * 1000.0 / iterations as f64;
        println!("  Average time per thumbnail: {:.2} ms", avg_ms);
        println!("  Throughput: {:.1} thumbnails/sec", 1000.0 / avg_ms);
    }

    // Shutdown
    plugin.shutdown()?;

    println!("\n=== Demo Complete ===");
    Ok(())
}
```

## Run the Demo

First, build the plugin and create the bundle if you haven't:

```bash
cd $RUSTBRIDGE_WORKSPACE/thumbnail-plugin
cargo build --release
rustbridge pack --no-sign
```

Copy a test image to the consumer directory:

```bash
cp test-image.jpg $RUSTBRIDGE_WORKSPACE/thumbnail-rust-consumer/
```

Then run the consumer:

```bash
cd $RUSTBRIDGE_WORKSPACE/thumbnail-rust-consumer
cargo run --release
```

Expected output:

```
=== Binary Transport Demo (Rust) ===

Loaded image: test-image.jpg (45678 bytes)

Demo 1: Create JPEG thumbnail (100x100)
  Thumbnail: 100x75 JPEG (2847 bytes)
  Processing time: 12.34ms
  Saved: thumbnail-100x100.jpg

Demo 2: Create PNG thumbnail (200x0 = proportional height)
  Thumbnail: 200x150 PNG (18234 bytes)
  Processing time: 15.67ms
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

## Key Observations

### Struct Layout Precision

The Rust consumer uses `#[repr(C)]` structs that exactly match the plugin:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct ThumbnailRequestHeader {
    version: u8,           // 1 byte
    _reserved: [u8; 3],    // 3 bytes (alignment padding)
    target_width: u32,     // 4 bytes
    target_height: u32,    // 4 bytes
    output_format: u32,    // 4 bytes
    quality: u32,          // 4 bytes
    payload_size: u32,     // 4 bytes
}                          // Total: 24 bytes
```

Key points:
- `#[repr(C)]` ensures C-compatible layout
- `_reserved` provides alignment padding
- `transmute` for zero-copy serialization (careful with endianness!)

### Zero-Copy Parsing

Response parsing uses pointer casting for efficiency:

```rust
fn from_bytes(data: &[u8]) -> Option<&Self> {
    if data.len() < Self::SIZE {
        return None;
    }
    // SAFETY: Validated size, repr(C) struct
    Some(unsafe { &*(data.as_ptr() as *const Self) })
}
```

This avoids copying the header bytes; we just reinterpret the slice.

### Memory Ownership

The binary transport follows "Rust allocates, Rust frees":

```rust
// plugin.call_raw returns owned Vec<u8>
let response = plugin.call_raw(MSG_THUMBNAIL_CREATE, &request)?;

// We own the response data - no need to free it manually
// Vec<u8> is dropped automatically when it goes out of scope
```

Unlike the Java/C#/Python implementations that must call `freeBuffer()`, the Rust consumer receives an owned `Vec<u8>` that's automatically cleaned up.

## Error Handling

Handle binary transport errors:

```rust
match plugin.call_raw(MSG_THUMBNAIL_CREATE, &request) {
    Ok(response) => {
        let (header, data) = parse_response(&response)?;
        // Process thumbnail...
    }
    Err(ConsumerError::CallFailed(plugin_err)) => {
        eprintln!("Plugin error (code {}): {}", plugin_err.error_code(), plugin_err);
    }
    Err(ConsumerError::MissingSymbol(msg)) => {
        eprintln!("Binary transport not available: {msg}");
    }
    Err(e) => {
        eprintln!("Unexpected error: {e}");
    }
}
```

## Log Callback Integration

Capture plugin logs with a callback:

```rust
use rustbridge_consumer::{LogCallbackFn, LogLevel, NativePluginLoader, PluginConfig};
use std::sync::Arc;

let log_callback: LogCallbackFn = Arc::new(|level, target, message| {
    let level_str = match level {
        LogLevel::Trace => "TRACE",
        LogLevel::Debug => "DEBUG",
        LogLevel::Info => "INFO",
        LogLevel::Warn => "WARN",
        LogLevel::Error => "ERROR",
        LogLevel::Off => return, // Don't log if level is Off
    };
    println!("[{level_str}] {target}: {message}");
});

let config = PluginConfig {
    log_level: LogLevel::Debug,
    ..PluginConfig::default()
};

let plugin = NativePluginLoader::load_bundle_with_config(
    "thumbnail-plugin-0.1.0.rbp",
    &config,
    Some(log_callback),
)?;
```

## Performance Notes

Rust-to-Rust binary transport is extremely efficient:

| Operation | Overhead |
|-----------|----------|
| Header serialization | ~5 ns (transmute) |
| Request allocation | ~100 ns (Vec allocation) |
| FFI call overhead | ~50 ns |
| Response parsing | ~10 ns (pointer cast) |

The actual image processing dominates; FFI overhead is negligible.

## What's Next?

You've now implemented binary transport in Java FFM, Kotlin, C#, Python, and Rust. Each implementation demonstrates:

- Matching C-compatible struct layouts
- Header + payload pattern for variable data
- Zero-copy parsing where possible
- Proper memory management

You've completed the rustbridge tutorial series! Key takeaways:

1. **JSON transport** is flexible and works for most use cases
2. **Binary transport** provides 7x faster latency and ~25% smaller payloads for binary data
3. **Backpressure queues** control memory and throttle producers
4. **Rust consumers** can dynamically load plugins just like other language consumers

For more information, see:
- [docs/TRANSPORT.md](../../TRANSPORT.md) - Transport layer details
- [docs/ARCHITECTURE.md](../../ARCHITECTURE.md) - System architecture
- [docs/ERROR_HANDLING.md](../../ERROR_HANDLING.md) - Error handling patterns
