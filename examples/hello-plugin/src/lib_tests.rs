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

    let response = plugin
        .handle_request(&ctx, "echo", &request)
        .await
        .unwrap();

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

    let result = plugin
        .handle_request(&ctx, "unknown.type", b"{}")
        .await;

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
}
