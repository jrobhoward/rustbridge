# Binary Transport Guide

This guide explains when and how to use binary transport for high-performance FFI communication in rustbridge.

## Overview

rustbridge supports two transport modes:

| Mode | Use Case | Serialization | Overhead |
|------|----------|---------------|----------|
| **JSON** | General purpose, flexibility | serde_json | Higher (~654ns/call) |
| **Binary** | Performance-critical paths | Zero-copy structs | Lower (~92ns/call) |

**Benchmark Results (M3 Max, single-threaded):**

| Metric | JSON | Binary | Improvement |
|--------|------|--------|-------------|
| Full cycle latency | 654 ns | 92 ns | **7.1x faster** |
| Allocation overhead | Dynamic | Fixed | Predictable |
| Message size | Variable | Fixed | Cacheable |

## When to Use Binary Transport

**Use binary transport when:**
- Request/response latency is critical (< 1μs target)
- Message structure is fixed and well-defined
- Payload fits in fixed-size buffers
- Calling from C/C++/Java FFM where struct mapping is natural

**Stick with JSON when:**
- Schema flexibility is needed
- Message sizes vary significantly
- Developer ergonomics matter more than raw performance
- Debugging/logging readability is important

## Defining Binary Messages (Rust)

Binary messages are `#[repr(C)]` structs with fixed layouts.

### Example: Request/Response Pair

```rust
use rustbridge_ffi::register_binary_handler;

/// Message ID for this binary message type
pub const MSG_LOOKUP: u32 = 1;

/// Request struct (must be #[repr(C)])
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LookupRequest {
    /// Struct version for forward compatibility
    pub version: u8,
    /// Reserved for alignment
    pub _reserved: [u8; 3],
    /// Key to look up (fixed-size buffer)
    pub key: [u8; 64],
    /// Length of key string
    pub key_len: u32,
    /// Request flags
    pub flags: u32,
}

impl LookupRequest {
    pub const VERSION: u8 = 1;

    pub fn new(key: &str, flags: u32) -> Self {
        let mut key_buf = [0u8; 64];
        let key_bytes = key.as_bytes();
        let len = key_bytes.len().min(64);
        key_buf[..len].copy_from_slice(&key_bytes[..len]);

        Self {
            version: Self::VERSION,
            _reserved: [0; 3],
            key: key_buf,
            key_len: len as u32,
            flags,
        }
    }
}

/// Response struct
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct LookupResponse {
    pub version: u8,
    pub _reserved: [u8; 3],
    pub value: [u8; 64],
    pub value_len: u32,
    pub ttl_seconds: u32,
}
```

### Layout Rules

1. **Version field first** - Always include a `version: u8` field for forward compatibility
2. **Alignment padding** - Use `_reserved: [u8; N]` to maintain natural alignment
3. **Fixed-size strings** - Use `[u8; N]` with a separate `*_len: u32` field
4. **No pointers** - Use inline arrays, not heap allocations

## Registering Handlers

```rust
use rustbridge_core::PluginResult;
use rustbridge_ffi::{PluginHandle, register_binary_handler};

pub fn register_handlers() {
    register_binary_handler(MSG_LOOKUP, handle_lookup);
}

fn handle_lookup(_handle: &PluginHandle, request: &[u8]) -> PluginResult<Vec<u8>> {
    // Validate size
    if request.len() < std::mem::size_of::<LookupRequest>() {
        return Err(PluginError::HandlerError("Request too small".into()));
    }

    // Parse request (safe: validated size, repr(C) struct)
    let req = unsafe { &*(request.as_ptr() as *const LookupRequest) };

    // Validate version
    if req.version != LookupRequest::VERSION {
        return Err(PluginError::HandlerError(format!(
            "Unsupported version: {} (expected {})",
            req.version, LookupRequest::VERSION
        )));
    }

    // Process and build response
    let response = LookupResponse { /* ... */ };

    // Return as bytes
    let bytes = unsafe {
        std::slice::from_raw_parts(
            &response as *const _ as *const u8,
            std::mem::size_of::<LookupResponse>(),
        )
    };
    Ok(bytes.to_vec())
}
```

## Generating C Headers

Use the CLI to generate C headers from Rust structs:

```bash
rustbridge generate-header \
    --source src/messages.rs \
    --output include/messages.h
```

**Generated header example:**

```c
#ifndef MESSAGES_RS_H
#define MESSAGES_RS_H

#include <stdint.h>
#include <stdbool.h>

#define MSG_LOOKUP ((uint32_t)1)

typedef struct LookupRequest {
    uint8_t version;
    uint8_t _reserved[3];
    uint8_t key[64];
    uint32_t key_len;
    uint32_t flags;
} LookupRequest;

typedef struct LookupResponse {
    uint8_t version;
    uint8_t _reserved[3];
    uint8_t value[64];
    uint32_t value_len;
    uint32_t ttl_seconds;
} LookupResponse;

#endif
```

## Java FFM Usage

### Define Struct Wrapper

```java
import com.rustbridge.ffm.BinaryStruct;
import java.lang.foreign.*;

public class LookupRequest extends BinaryStruct {
    public static final byte VERSION = 1;

    public static final StructLayout LAYOUT = MemoryLayout.structLayout(
        ValueLayout.JAVA_BYTE.withName("version"),
        MemoryLayout.sequenceLayout(3, ValueLayout.JAVA_BYTE).withName("_reserved"),
        MemoryLayout.sequenceLayout(64, ValueLayout.JAVA_BYTE).withName("key"),
        ValueLayout.JAVA_INT.withName("key_len"),
        ValueLayout.JAVA_INT.withName("flags")
    );

    public LookupRequest(Arena arena) {
        super(arena.allocate(LAYOUT));
        setByte(0, VERSION);  // Set version
    }

    public void setKey(String key) {
        setFixedString(key, 4, 64, 68);  // offset=4, maxLen=64, lenOffset=68
    }

    public void setFlags(int flags) {
        setInt(72, flags);
    }

    @Override
    public long byteSize() {
        return LAYOUT.byteSize();
    }
}
```

### Call Plugin

```java
try (Arena arena = Arena.ofConfined()) {
    FfmPlugin plugin = loader.load(pluginPath, config);

    // Create request
    LookupRequest request = new LookupRequest(arena);
    request.setKey("my_key");
    request.setFlags(0x01);

    // Call with binary transport
    MemorySegment response = plugin.callRaw(
        MSG_LOOKUP,
        request,
        80  // Expected response size
    );

    // Parse response
    String value = getFixedString(response, 4, 64, 68);
    int ttl = response.get(ValueLayout.JAVA_INT, 72);
}
```

## Versioning Policy

Binary structs use a version field for forward compatibility:

1. **Version field** - First byte of every struct
2. **Check on receive** - Handlers validate version and reject unknown versions
3. **Increment on change** - Bump version when struct layout changes
4. **Reserved fields** - Use `_reserved` padding for future expansion

### Adding Fields

```rust
// Version 1
#[repr(C)]
pub struct MyRequest {
    pub version: u8,
    pub _reserved: [u8; 7],  // Extra reserved space
    pub field_a: u32,
    pub field_b: u32,
}

// Version 2 - uses reserved space
#[repr(C)]
pub struct MyRequestV2 {
    pub version: u8,
    pub new_flag: u8,        // New field in reserved space
    pub _reserved: [u8; 6],
    pub field_a: u32,
    pub field_b: u32,
}
```

### Handler Compatibility

```rust
fn handle_request(request: &[u8]) -> PluginResult<Vec<u8>> {
    let version = request[0];

    match version {
        1 => handle_v1(request),
        2 => handle_v2(request),
        _ => Err(PluginError::HandlerError(
            format!("Unsupported version: {}", version)
        )),
    }
}
```

## Performance Tips

1. **Reuse allocations** - Keep request structs alive across calls
2. **Batch operations** - Group multiple lookups into array requests
3. **Align on 8 bytes** - Ensures efficient memory access
4. **Avoid copies** - Use `&[u8]` slices when possible

## Error Handling

Binary transport uses `RbResponse` with error codes:

| Error Code | Meaning |
|------------|---------|
| 0 | Success |
| 2 | InvalidRequest (bad size/version) |
| 4 | HandlerNotFound (unknown message ID) |
| 5 | HandlerError (processing failed) |
| 11 | InternalError (panic caught) |

## Migration Checklist

When migrating a JSON endpoint to binary:

- [ ] Define `#[repr(C)]` request/response structs with version field
- [ ] Add message ID constant
- [ ] Implement and register binary handler
- [ ] Generate C header with CLI
- [ ] Create Java/C# struct wrappers
- [ ] Update client code to use `callRaw()`
- [ ] Benchmark to verify improvement

## See Also

- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture
- [examples/hello-plugin/src/binary_messages.rs](../examples/hello-plugin/src/binary_messages.rs) - Reference implementation
