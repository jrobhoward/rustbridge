//! JSON Serialization Baseline Benchmarks
//!
//! This benchmark establishes baseline performance metrics for JSON serialization
//! and deserialization using serde_json. These measurements will be compared against
//! C struct transport to evaluate the performance benefit of binary serialization.
//!
//! # Payload Sizes
//!
//! - **Small**: ~100 bytes (config lookup, feature flags)
//! - **Medium**: ~1KB (user records, API entities)
//! - **Large**: ~100KB (batch queries, data exports)

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};

// ============================================================================
// Benchmark Message Types (mirrors hello-plugin)
// ============================================================================

/// Small request (~100 bytes JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SmallRequest {
    key: String,
    flags: u32,
}

/// Small response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SmallResponse {
    value: String,
    ttl_seconds: u32,
    cache_hit: bool,
}

/// Medium request (~1KB JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MediumRequest {
    user_id: u64,
    include_fields: Vec<String>,
    options: MediumOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MediumOptions {
    include_metadata: bool,
    include_permissions: bool,
    max_results: u32,
}

/// Medium response (~1KB JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MediumResponse {
    user_id: u64,
    username: String,
    email: String,
    display_name: String,
    metadata: Vec<KeyValue>,
    permissions: Vec<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyValue {
    key: String,
    value: String,
}

/// Large request
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LargeRequest {
    query_id: u64,
    filters: Vec<Filter>,
    page_size: u32,
    page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Filter {
    field: String,
    operator: String,
    value: String,
}

/// Large response (~100KB JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LargeResponse {
    query_id: u64,
    results: Vec<Record>,
    total_count: u64,
    next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record {
    id: u64,
    name: String,
    description: String,
    category: String,
    tags: Vec<String>,
    score: f64,
    metadata: Vec<KeyValue>,
    created_at: i64,
}

// ============================================================================
// Test Data Generators
// ============================================================================

fn create_small_request() -> SmallRequest {
    SmallRequest {
        key: "config.feature.enable_dark_mode".to_string(),
        flags: 0x0001,
    }
}

fn create_small_response() -> SmallResponse {
    SmallResponse {
        value: "value_for_config.feature.enable_dark_mode".to_string(),
        ttl_seconds: 3600,
        cache_hit: true,
    }
}

fn create_medium_request() -> MediumRequest {
    MediumRequest {
        user_id: 12345678,
        include_fields: vec![
            "username".to_string(),
            "email".to_string(),
            "display_name".to_string(),
            "metadata".to_string(),
            "permissions".to_string(),
            "created_at".to_string(),
            "updated_at".to_string(),
            "avatar_url".to_string(),
            "preferences".to_string(),
            "last_login".to_string(),
        ],
        options: MediumOptions {
            include_metadata: true,
            include_permissions: true,
            max_results: 100,
        },
    }
}

fn create_medium_response() -> MediumResponse {
    MediumResponse {
        user_id: 12345678,
        username: "user_12345678".to_string(),
        email: "user_12345678@example.com".to_string(),
        display_name: "User Number 12345678".to_string(),
        metadata: (0..10)
            .map(|i| KeyValue {
                key: format!("meta_key_{}", i),
                value: format!("meta_value_{}_for_user_12345678", i),
            })
            .collect(),
        permissions: vec![
            "read".to_string(),
            "write".to_string(),
            "delete".to_string(),
            "admin".to_string(),
            "users.read".to_string(),
            "users.write".to_string(),
            "reports.read".to_string(),
            "reports.write".to_string(),
            "settings.read".to_string(),
            "settings.write".to_string(),
            "audit.read".to_string(),
            "billing.read".to_string(),
            "billing.write".to_string(),
            "api.access".to_string(),
            "webhooks.manage".to_string(),
        ],
        created_at: 1700000000,
        updated_at: 1705000000,
    }
}

fn create_large_request() -> LargeRequest {
    LargeRequest {
        query_id: 9876543210,
        filters: (0..10)
            .map(|i| Filter {
                field: format!("field_{}", i),
                operator: "eq".to_string(),
                value: format!("value_{}", i),
            })
            .collect(),
        page_size: 1000,
        page_token: None,
    }
}

fn create_large_response(record_count: usize) -> LargeResponse {
    LargeResponse {
        query_id: 9876543210,
        results: (0..record_count)
            .map(|i| {
                let id = 9876543210u64 * 10000 + i as u64;
                Record {
                    id,
                    name: format!("Record {} for query 9876543210", i),
                    description: format!(
                        "This is a detailed description for record {}. \
                         It contains enough text to make the payload realistic. \
                         Query ID: 9876543210, Filters: 10",
                        i
                    ),
                    category: format!("category_{}", i % 10),
                    tags: vec![
                        format!("tag_a_{}", i % 5),
                        format!("tag_b_{}", i % 7),
                        format!("tag_c_{}", i % 3),
                    ],
                    score: (i as f64) * 0.1 + 0.5,
                    metadata: vec![
                        KeyValue {
                            key: "source".to_string(),
                            value: "benchmark".to_string(),
                        },
                        KeyValue {
                            key: "version".to_string(),
                            value: "1.0".to_string(),
                        },
                    ],
                    created_at: 1700000000 + (i as i64 * 1000),
                }
            })
            .collect(),
        total_count: (record_count * 10) as u64,
        next_page_token: Some(format!("token_9876543210_{}", record_count)),
    }
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_small_payload(c: &mut Criterion) {
    let mut group = c.benchmark_group("small_payload");

    let request = create_small_request();
    let response = create_small_response();

    // Measure serialized size
    let req_json = serde_json::to_vec(&request).unwrap();
    let resp_json = serde_json::to_vec(&response).unwrap();
    println!(
        "Small payload sizes: request={} bytes, response={} bytes",
        req_json.len(),
        resp_json.len()
    );

    group.throughput(Throughput::Bytes(req_json.len() as u64));

    group.bench_function("serialize_request", |b| {
        b.iter(|| serde_json::to_vec(black_box(&request)).unwrap())
    });

    group.bench_function("deserialize_request", |b| {
        b.iter(|| serde_json::from_slice::<SmallRequest>(black_box(&req_json)).unwrap())
    });

    group.throughput(Throughput::Bytes(resp_json.len() as u64));

    group.bench_function("serialize_response", |b| {
        b.iter(|| serde_json::to_vec(black_box(&response)).unwrap())
    });

    group.bench_function("deserialize_response", |b| {
        b.iter(|| serde_json::from_slice::<SmallResponse>(black_box(&resp_json)).unwrap())
    });

    // Round-trip benchmark (serialize + deserialize)
    group.bench_function("roundtrip_request", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&request)).unwrap();
            serde_json::from_slice::<SmallRequest>(black_box(&json)).unwrap()
        })
    });

    group.bench_function("roundtrip_response", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&response)).unwrap();
            serde_json::from_slice::<SmallResponse>(black_box(&json)).unwrap()
        })
    });

    group.finish();
}

fn bench_medium_payload(c: &mut Criterion) {
    let mut group = c.benchmark_group("medium_payload");

    let request = create_medium_request();
    let response = create_medium_response();

    let req_json = serde_json::to_vec(&request).unwrap();
    let resp_json = serde_json::to_vec(&response).unwrap();
    println!(
        "Medium payload sizes: request={} bytes, response={} bytes",
        req_json.len(),
        resp_json.len()
    );

    group.throughput(Throughput::Bytes(req_json.len() as u64));

    group.bench_function("serialize_request", |b| {
        b.iter(|| serde_json::to_vec(black_box(&request)).unwrap())
    });

    group.bench_function("deserialize_request", |b| {
        b.iter(|| serde_json::from_slice::<MediumRequest>(black_box(&req_json)).unwrap())
    });

    group.throughput(Throughput::Bytes(resp_json.len() as u64));

    group.bench_function("serialize_response", |b| {
        b.iter(|| serde_json::to_vec(black_box(&response)).unwrap())
    });

    group.bench_function("deserialize_response", |b| {
        b.iter(|| serde_json::from_slice::<MediumResponse>(black_box(&resp_json)).unwrap())
    });

    group.bench_function("roundtrip_request", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&request)).unwrap();
            serde_json::from_slice::<MediumRequest>(black_box(&json)).unwrap()
        })
    });

    group.bench_function("roundtrip_response", |b| {
        b.iter(|| {
            let json = serde_json::to_vec(black_box(&response)).unwrap();
            serde_json::from_slice::<MediumResponse>(black_box(&json)).unwrap()
        })
    });

    group.finish();
}

fn bench_large_payload(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_payload");

    // Test with different record counts
    for record_count in [100, 500, 1000] {
        let request = create_large_request();
        let response = create_large_response(record_count);

        let req_json = serde_json::to_vec(&request).unwrap();
        let resp_json = serde_json::to_vec(&response).unwrap();

        if record_count == 1000 {
            println!(
                "Large payload sizes (1000 records): request={} bytes, response={} bytes ({:.1} KB)",
                req_json.len(),
                resp_json.len(),
                resp_json.len() as f64 / 1024.0
            );
        }

        group.throughput(Throughput::Bytes(resp_json.len() as u64));

        group.bench_with_input(
            BenchmarkId::new("serialize_response", record_count),
            &response,
            |b, resp| {
                b.iter(|| serde_json::to_vec(black_box(resp)).unwrap());
            },
        );

        group.bench_with_input(
            BenchmarkId::new("deserialize_response", record_count),
            &resp_json,
            |b, json| {
                b.iter(|| serde_json::from_slice::<LargeResponse>(black_box(json)).unwrap());
            },
        );

        group.bench_with_input(
            BenchmarkId::new("roundtrip_response", record_count),
            &response,
            |b, resp| {
                b.iter(|| {
                    let json = serde_json::to_vec(black_box(resp)).unwrap();
                    serde_json::from_slice::<LargeResponse>(black_box(&json)).unwrap()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark that simulates a full plugin call cycle:
/// deserialize request -> process -> serialize response
fn bench_full_cycle(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_cycle");

    // Small payload full cycle
    let small_req = create_small_request();
    let small_req_json = serde_json::to_vec(&small_req).unwrap();

    group.bench_function("small", |b| {
        b.iter(|| {
            // Deserialize request
            let req: SmallRequest = serde_json::from_slice(black_box(&small_req_json)).unwrap();

            // "Process" - create response
            let resp = SmallResponse {
                value: format!("value_for_{}", req.key),
                ttl_seconds: 3600,
                cache_hit: req.flags & 1 != 0,
            };

            // Serialize response
            serde_json::to_vec(black_box(&resp)).unwrap()
        })
    });

    // Medium payload full cycle
    let medium_req = create_medium_request();
    let medium_req_json = serde_json::to_vec(&medium_req).unwrap();

    group.bench_function("medium", |b| {
        b.iter(|| {
            let req: MediumRequest = serde_json::from_slice(black_box(&medium_req_json)).unwrap();

            let resp = MediumResponse {
                user_id: req.user_id,
                username: format!("user_{}", req.user_id),
                email: format!("user_{}@example.com", req.user_id),
                display_name: format!("User Number {}", req.user_id),
                metadata: if req.options.include_metadata {
                    (0..10)
                        .map(|i| KeyValue {
                            key: format!("meta_key_{}", i),
                            value: format!("meta_value_{}", i),
                        })
                        .collect()
                } else {
                    Vec::new()
                },
                permissions: if req.options.include_permissions {
                    vec!["read".to_string(), "write".to_string()]
                } else {
                    Vec::new()
                },
                created_at: 1700000000,
                updated_at: 1705000000,
            };

            serde_json::to_vec(black_box(&resp)).unwrap()
        })
    });

    // Large payload full cycle (100 records for reasonable bench time)
    let large_req = create_large_request();
    let large_req_json = serde_json::to_vec(&large_req).unwrap();

    group.bench_function("large_100_records", |b| {
        b.iter(|| {
            let req: LargeRequest = serde_json::from_slice(black_box(&large_req_json)).unwrap();

            let resp = LargeResponse {
                query_id: req.query_id,
                results: (0..100)
                    .map(|i| Record {
                        id: req.query_id * 10000 + i as u64,
                        name: format!("Record {}", i),
                        description: "Description text".to_string(),
                        category: format!("cat_{}", i % 10),
                        tags: vec!["a".to_string(), "b".to_string()],
                        score: i as f64 * 0.1,
                        metadata: vec![KeyValue {
                            key: "k".to_string(),
                            value: "v".to_string(),
                        }],
                        created_at: 1700000000,
                    })
                    .collect(),
                total_count: 1000,
                next_page_token: Some("token".to_string()),
            };

            serde_json::to_vec(black_box(&resp)).unwrap()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_small_payload,
    bench_medium_payload,
    bench_large_payload,
    bench_full_cycle,
);

criterion_main!(benches);
