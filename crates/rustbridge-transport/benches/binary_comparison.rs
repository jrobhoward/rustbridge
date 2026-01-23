//! Binary vs JSON Transport Comparison Benchmarks
//!
//! This benchmark compares the performance of JSON serialization against
//! direct C struct access for small payloads. This helps determine whether
//! binary transport is worth the added complexity.
//!
//! # What We're Measuring
//!
//! 1. **JSON path**: serialize request → deserialize → process → serialize response → deserialize
//! 2. **Binary path**: copy struct → process → copy struct
//!
//! The binary path avoids all serialization overhead but requires fixed-size buffers.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};

// ============================================================================
// JSON Message Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SmallRequestJson {
    key: String,
    flags: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SmallResponseJson {
    value: String,
    ttl_seconds: u32,
    cache_hit: bool,
}

// ============================================================================
// Binary (C Struct) Message Types
// ============================================================================

/// Small request as C struct (72 bytes fixed)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SmallRequestRaw {
    key: [u8; 64],
    key_len: u32,
    flags: u32,
}

impl SmallRequestRaw {
    fn new(key: &str, flags: u32) -> Self {
        let mut key_buf = [0u8; 64];
        let key_bytes = key.as_bytes();
        let len = key_bytes.len().min(64);
        key_buf[..len].copy_from_slice(&key_bytes[..len]);

        Self {
            key: key_buf,
            key_len: len as u32,
            flags,
        }
    }

    fn key_str(&self) -> &str {
        let len = self.key_len.min(64) as usize;
        unsafe { std::str::from_utf8_unchecked(&self.key[..len]) }
    }
}

/// Small response as C struct (76 bytes fixed)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SmallResponseRaw {
    value: [u8; 64],
    value_len: u32,
    ttl_seconds: u32,
    cache_hit: u8,
    _padding: [u8; 3],
}

impl SmallResponseRaw {
    fn new(value: &str, ttl_seconds: u32, cache_hit: bool) -> Self {
        let mut value_buf = [0u8; 64];
        let value_bytes = value.as_bytes();
        let len = value_bytes.len().min(64);
        value_buf[..len].copy_from_slice(&value_bytes[..len]);

        Self {
            value: value_buf,
            value_len: len as u32,
            ttl_seconds,
            cache_hit: if cache_hit { 1 } else { 0 },
            _padding: [0; 3],
        }
    }
}

// ============================================================================
// Test Data
// ============================================================================

fn create_json_request() -> SmallRequestJson {
    SmallRequestJson {
        key: "config.feature.enable_dark_mode".to_string(),
        flags: 0x0001,
    }
}

fn create_raw_request() -> SmallRequestRaw {
    SmallRequestRaw::new("config.feature.enable_dark_mode", 0x0001)
}

// ============================================================================
// Processing Logic (same for both paths)
// ============================================================================

fn process_request_json(req: &SmallRequestJson) -> SmallResponseJson {
    SmallResponseJson {
        value: format!("value_for_{}", req.key),
        ttl_seconds: 3600,
        cache_hit: req.flags & 1 != 0,
    }
}

fn process_request_raw(req: &SmallRequestRaw) -> SmallResponseRaw {
    let key = req.key_str();
    let value = format!("value_for_{}", key);
    SmallResponseRaw::new(&value, 3600, req.flags & 1 != 0)
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_json_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_roundtrip");

    let json_request = create_json_request();
    let raw_request = create_raw_request();

    // JSON: Full roundtrip (serialize → deserialize → process → serialize → deserialize)
    group.bench_function("json_full_cycle", |b| {
        b.iter(|| {
            // Serialize request (what host does)
            let req_bytes = serde_json::to_vec(black_box(&json_request)).unwrap();

            // Deserialize request (what plugin does)
            let req: SmallRequestJson = serde_json::from_slice(black_box(&req_bytes)).unwrap();

            // Process
            let resp = process_request_json(&req);

            // Serialize response (what plugin does)
            let resp_bytes = serde_json::to_vec(black_box(&resp)).unwrap();

            // Deserialize response (what host does)
            let _: SmallResponseJson = serde_json::from_slice(black_box(&resp_bytes)).unwrap();
        })
    });

    // Binary: Full roundtrip (copy struct → process → copy struct)
    group.bench_function("binary_full_cycle", |b| {
        b.iter(|| {
            // "Serialize" request (just copy bytes)
            let req_bytes: [u8; std::mem::size_of::<SmallRequestRaw>()] =
                unsafe { std::mem::transmute_copy(black_box(&raw_request)) };

            // "Deserialize" request (interpret bytes as struct)
            let req: &SmallRequestRaw =
                unsafe { &*(black_box(req_bytes.as_ptr()) as *const SmallRequestRaw) };

            // Process (includes string allocation for fair comparison)
            let resp = process_request_raw(req);

            // "Serialize" response (just copy bytes)
            let _resp_bytes: [u8; std::mem::size_of::<SmallResponseRaw>()] =
                unsafe { std::mem::transmute_copy(black_box(&resp)) };
        })
    });

    // Binary: Zero-copy path (no copying, just pointer casting)
    group.bench_function("binary_zero_copy", |b| {
        // Pre-allocate request bytes
        let req_bytes: [u8; std::mem::size_of::<SmallRequestRaw>()] =
            unsafe { std::mem::transmute_copy(&raw_request) };

        b.iter(|| {
            // Zero-copy "deserialize" (just cast pointer)
            let req: &SmallRequestRaw =
                unsafe { &*(black_box(req_bytes.as_ptr()) as *const SmallRequestRaw) };

            // Process
            let resp = process_request_raw(req);

            // Return response (could be zero-copy too if caller provides buffer)
            black_box(&resp);
        })
    });

    group.finish();
}

fn bench_serialization_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_serialize_only");

    let json_request = create_json_request();
    let json_response = SmallResponseJson {
        value: "value_for_config.feature.enable_dark_mode".to_string(),
        ttl_seconds: 3600,
        cache_hit: true,
    };

    let raw_request = create_raw_request();
    let raw_response =
        SmallResponseRaw::new("value_for_config.feature.enable_dark_mode", 3600, true);

    // JSON serialize request
    group.bench_function("json_serialize_request", |b| {
        b.iter(|| serde_json::to_vec(black_box(&json_request)).unwrap())
    });

    // JSON serialize response
    group.bench_function("json_serialize_response", |b| {
        b.iter(|| serde_json::to_vec(black_box(&json_response)).unwrap())
    });

    // Binary "serialize" request (memcpy)
    group.bench_function("binary_serialize_request", |b| {
        b.iter(|| {
            let bytes: [u8; std::mem::size_of::<SmallRequestRaw>()] =
                unsafe { std::mem::transmute_copy(black_box(&raw_request)) };
            black_box(bytes)
        })
    });

    // Binary "serialize" response (memcpy)
    group.bench_function("binary_serialize_response", |b| {
        b.iter(|| {
            let bytes: [u8; std::mem::size_of::<SmallResponseRaw>()] =
                unsafe { std::mem::transmute_copy(black_box(&raw_response)) };
            black_box(bytes)
        })
    });

    group.finish();
}

fn bench_deserialization_only(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_deserialize_only");

    let json_request = create_json_request();
    let json_response = SmallResponseJson {
        value: "value_for_config.feature.enable_dark_mode".to_string(),
        ttl_seconds: 3600,
        cache_hit: true,
    };

    let raw_request = create_raw_request();
    let raw_response =
        SmallResponseRaw::new("value_for_config.feature.enable_dark_mode", 3600, true);

    // Pre-serialize for deserialization benchmarks
    let json_req_bytes = serde_json::to_vec(&json_request).unwrap();
    let json_resp_bytes = serde_json::to_vec(&json_response).unwrap();
    let raw_req_bytes: [u8; std::mem::size_of::<SmallRequestRaw>()] =
        unsafe { std::mem::transmute_copy(&raw_request) };
    let raw_resp_bytes: [u8; std::mem::size_of::<SmallResponseRaw>()] =
        unsafe { std::mem::transmute_copy(&raw_response) };

    // JSON deserialize request
    group.bench_function("json_deserialize_request", |b| {
        b.iter(|| serde_json::from_slice::<SmallRequestJson>(black_box(&json_req_bytes)).unwrap())
    });

    // JSON deserialize response
    group.bench_function("json_deserialize_response", |b| {
        b.iter(|| serde_json::from_slice::<SmallResponseJson>(black_box(&json_resp_bytes)).unwrap())
    });

    // Binary "deserialize" request (pointer cast)
    group.bench_function("binary_deserialize_request", |b| {
        b.iter(|| {
            let req: &SmallRequestRaw =
                unsafe { &*(black_box(raw_req_bytes.as_ptr()) as *const SmallRequestRaw) };
            black_box(req)
        })
    });

    // Binary "deserialize" response (pointer cast)
    group.bench_function("binary_deserialize_response", |b| {
        b.iter(|| {
            let resp: &SmallResponseRaw =
                unsafe { &*(black_box(raw_resp_bytes.as_ptr()) as *const SmallResponseRaw) };
            black_box(resp)
        })
    });

    group.finish();
}

fn bench_payload_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("payload_sizes");

    let json_request = create_json_request();
    let json_response = SmallResponseJson {
        value: "value_for_config.feature.enable_dark_mode".to_string(),
        ttl_seconds: 3600,
        cache_hit: true,
    };

    let json_req_bytes = serde_json::to_vec(&json_request).unwrap();
    let json_resp_bytes = serde_json::to_vec(&json_response).unwrap();

    println!("\n=== Payload Size Comparison ===");
    println!("JSON request:    {} bytes", json_req_bytes.len());
    println!(
        "Binary request:  {} bytes",
        std::mem::size_of::<SmallRequestRaw>()
    );
    println!("JSON response:   {} bytes", json_resp_bytes.len());
    println!(
        "Binary response: {} bytes",
        std::mem::size_of::<SmallResponseRaw>()
    );
    println!(
        "Binary overhead: {:.1}x request, {:.1}x response",
        std::mem::size_of::<SmallRequestRaw>() as f64 / json_req_bytes.len() as f64,
        std::mem::size_of::<SmallResponseRaw>() as f64 / json_resp_bytes.len() as f64
    );
    println!();

    // Dummy benchmark to ensure the group runs
    group.bench_function("noop", |b| b.iter(|| black_box(1)));

    group.finish();
}

criterion_group!(
    benches,
    bench_payload_sizes,
    bench_serialization_only,
    bench_deserialization_only,
    bench_json_roundtrip,
);

criterion_main!(benches);
