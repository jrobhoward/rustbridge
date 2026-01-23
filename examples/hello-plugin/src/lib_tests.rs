#![allow(non_snake_case)]

use super::*;
use rustbridge_core::PluginConfig;

fn create_test_context() -> PluginContext {
    PluginContext::new(PluginConfig::default())
}

// HelloPlugin::handle_request echo tests

#[tokio::test]
async fn HelloPlugin___echo_request___returns_message_and_length() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&EchoRequest {
        message: "Hello, World!".to_string(),
    })
    .unwrap();

    let response = plugin.handle_request(&ctx, "echo", &request).await.unwrap();

    let echo_response: EchoResponse = serde_json::from_slice(&response).unwrap();
    assert_eq!(echo_response.message, "Hello, World!");
    assert_eq!(echo_response.length, 13);
}

// HelloPlugin::handle_request greet tests

#[tokio::test]
async fn HelloPlugin___greet_request___returns_greeting_with_name() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&GreetRequest {
        name: "Alice".to_string(),
    })
    .unwrap();

    let response = plugin
        .handle_request(&ctx, "greet", &request)
        .await
        .unwrap();

    let greet_response: GreetResponse = serde_json::from_slice(&response).unwrap();
    assert!(greet_response.greeting.contains("Alice"));
    assert!(greet_response.greeting.contains("rustbridge"));
}

// HelloPlugin::handle_request user.create tests

#[tokio::test]
async fn HelloPlugin___create_user_request___returns_user_id_and_timestamp() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&CreateUserRequest {
        username: "john".to_string(),
        email: "john@example.com".to_string(),
    })
    .unwrap();

    let response = plugin
        .handle_request(&ctx, "user.create", &request)
        .await
        .unwrap();

    let user_response: CreateUserResponse = serde_json::from_slice(&response).unwrap();
    assert!(user_response.user_id.starts_with("user-"));
    assert!(!user_response.created_at.is_empty());
}

// HelloPlugin::handle_request math.add tests

#[tokio::test]
async fn HelloPlugin___add_request___returns_sum() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&AddRequest { a: 10, b: 32 }).unwrap();

    let response = plugin
        .handle_request(&ctx, "math.add", &request)
        .await
        .unwrap();

    let add_response: AddResponse = serde_json::from_slice(&response).unwrap();
    assert_eq!(add_response.result, 42);
}

// HelloPlugin::handle_request unknown type tests

#[tokio::test]
async fn HelloPlugin___unknown_type_tag___returns_error() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();

    let result = plugin.handle_request(&ctx, "unknown.type", b"{}").await;

    assert!(matches!(result, Err(PluginError::UnknownMessageType(_))));
}

// HelloPlugin lifecycle tests

#[tokio::test]
async fn HelloPlugin___on_start_and_stop___succeeds() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();

    plugin.on_start(&ctx).await.unwrap();

    plugin.on_stop(&ctx).await.unwrap();
}

// HelloPlugin::metadata tests

#[test]
fn HelloPlugin___metadata___returns_correct_name_and_version() {
    let plugin = HelloPlugin::new();

    let metadata = plugin.metadata().unwrap();

    assert_eq!(metadata.name, "hello-plugin");
    assert_eq!(metadata.version, "0.1.0");
}

// HelloPlugin::supported_types tests

#[test]
fn HelloPlugin___supported_types___includes_all_handlers() {
    let plugin = HelloPlugin::new();

    let types = plugin.supported_types();

    assert!(types.contains(&"echo"));
    assert!(types.contains(&"greet"));
    assert!(types.contains(&"user.create"));
    assert!(types.contains(&"math.add"));
    assert!(types.contains(&"bench.small"));
    assert!(types.contains(&"bench.medium"));
    assert!(types.contains(&"bench.large"));
}

// ============================================================================
// Benchmark Handler Tests
// ============================================================================

// HelloPlugin::handle_request bench.small tests

#[tokio::test]
async fn HelloPlugin___bench_small_request___returns_value_and_ttl() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&SmallRequest {
        key: "test_key".to_string(),
        flags: 0x0001,
    })
    .unwrap();

    let response = plugin
        .handle_request(&ctx, "bench.small", &request)
        .await
        .unwrap();

    let resp: SmallResponse = serde_json::from_slice(&response).unwrap();
    assert!(resp.value.contains("test_key"));
    assert_eq!(resp.ttl_seconds, 3600);
    assert!(resp.cache_hit); // flags & 1 != 0
}

#[tokio::test]
async fn HelloPlugin___bench_small_request___cache_miss_when_flag_not_set() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&SmallRequest {
        key: "another_key".to_string(),
        flags: 0x0000, // No cache flag
    })
    .unwrap();

    let response = plugin
        .handle_request(&ctx, "bench.small", &request)
        .await
        .unwrap();

    let resp: SmallResponse = serde_json::from_slice(&response).unwrap();
    assert!(!resp.cache_hit);
}

// HelloPlugin::handle_request bench.medium tests

#[tokio::test]
async fn HelloPlugin___bench_medium_request___returns_user_with_metadata() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&MediumRequest {
        user_id: 12345,
        include_fields: vec!["username".to_string(), "email".to_string()],
        options: MediumOptions {
            include_metadata: true,
            include_permissions: true,
            max_results: 100,
        },
    })
    .unwrap();

    let response = plugin
        .handle_request(&ctx, "bench.medium", &request)
        .await
        .unwrap();

    let resp: MediumResponse = serde_json::from_slice(&response).unwrap();
    assert_eq!(resp.user_id, 12345);
    assert!(resp.username.contains("12345"));
    assert!(resp.email.contains("12345"));
    assert_eq!(resp.metadata.len(), 10);
    assert!(!resp.permissions.is_empty());
}

#[tokio::test]
async fn HelloPlugin___bench_medium_request___excludes_metadata_when_disabled() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&MediumRequest {
        user_id: 99999,
        include_fields: vec![],
        options: MediumOptions {
            include_metadata: false,
            include_permissions: false,
            max_results: 10,
        },
    })
    .unwrap();

    let response = plugin
        .handle_request(&ctx, "bench.medium", &request)
        .await
        .unwrap();

    let resp: MediumResponse = serde_json::from_slice(&response).unwrap();
    assert_eq!(resp.user_id, 99999);
    assert!(resp.metadata.is_empty());
    assert!(resp.permissions.is_empty());
}

// HelloPlugin::handle_request bench.large tests

#[tokio::test]
async fn HelloPlugin___bench_large_request___returns_records() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&LargeRequest {
        query_id: 42,
        filters: vec![Filter {
            field: "status".to_string(),
            operator: "eq".to_string(),
            value: "active".to_string(),
        }],
        page_size: 100,
        page_token: None,
    })
    .unwrap();

    let response = plugin
        .handle_request(&ctx, "bench.large", &request)
        .await
        .unwrap();

    let resp: LargeResponse = serde_json::from_slice(&response).unwrap();
    assert_eq!(resp.query_id, 42);
    assert_eq!(resp.results.len(), 100);
    assert!(resp.total_count > 0);
    assert!(resp.next_page_token.is_some());
}

#[tokio::test]
async fn HelloPlugin___bench_large_request___respects_page_size_limit() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&LargeRequest {
        query_id: 1,
        filters: vec![],
        page_size: 5000, // Over the 1000 limit
        page_token: None,
    })
    .unwrap();

    let response = plugin
        .handle_request(&ctx, "bench.large", &request)
        .await
        .unwrap();

    let resp: LargeResponse = serde_json::from_slice(&response).unwrap();
    assert_eq!(resp.results.len(), 1000); // Capped at 1000
}

#[tokio::test]
async fn HelloPlugin___bench_large_request___records_have_expected_fields() {
    let plugin = HelloPlugin::new();
    let ctx = create_test_context();
    let request = serde_json::to_vec(&LargeRequest {
        query_id: 7,
        filters: vec![],
        page_size: 10,
        page_token: None,
    })
    .unwrap();

    let response = plugin
        .handle_request(&ctx, "bench.large", &request)
        .await
        .unwrap();

    let resp: LargeResponse = serde_json::from_slice(&response).unwrap();
    let first_record = &resp.results[0];
    assert!(!first_record.name.is_empty());
    assert!(!first_record.description.is_empty());
    assert!(!first_record.category.is_empty());
    assert_eq!(first_record.tags.len(), 3);
    assert_eq!(first_record.metadata.len(), 2);
}

// Benchmark payload size verification tests

#[test]
fn benchmark_payloads___small___approximately_100_bytes() {
    let request = SmallRequest {
        key: "config.feature.enable_dark_mode".to_string(),
        flags: 0x0001,
    };
    let response = SmallResponse {
        value: "value_for_config.feature.enable_dark_mode".to_string(),
        ttl_seconds: 3600,
        cache_hit: true,
    };

    let req_json = serde_json::to_vec(&request).unwrap();
    let resp_json = serde_json::to_vec(&response).unwrap();

    // Small payloads should be under 200 bytes each
    assert!(req_json.len() < 200, "request: {} bytes", req_json.len());
    assert!(resp_json.len() < 200, "response: {} bytes", resp_json.len());
}

#[test]
fn benchmark_payloads___medium___approximately_1kb() {
    let response = MediumResponse {
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
        ],
        created_at: 1700000000,
        updated_at: 1705000000,
    };

    let resp_json = serde_json::to_vec(&response).unwrap();

    // Medium response should be roughly 500-2000 bytes
    assert!(resp_json.len() > 500, "response: {} bytes", resp_json.len());
    assert!(
        resp_json.len() < 2000,
        "response: {} bytes",
        resp_json.len()
    );
}
