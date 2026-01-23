#![allow(non_snake_case)]

use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestRequest {
    name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestResponse {
    id: u64,
}

// RequestEnvelope tests

#[test]
fn RequestEnvelope___new___sets_type_tag_and_payload() {
    let req = RequestEnvelope::new("user.create", serde_json::json!({"name": "John"}));

    assert_eq!(req.type_tag, "user.create");
    assert!(req.request_id.is_none());
    assert!(req.correlation_id.is_none());
}

#[test]
fn RequestEnvelope___with_request_id___sets_id() {
    let req = RequestEnvelope::new("test", serde_json::json!({})).with_request_id(123);

    assert_eq!(req.request_id, Some(123));
}

#[test]
fn RequestEnvelope___with_correlation_id___sets_correlation_id() {
    let req = RequestEnvelope::new("test", serde_json::json!({})).with_correlation_id("corr-456");

    assert_eq!(req.correlation_id, Some("corr-456".to_string()));
}

#[test]
fn RequestEnvelope___from_typed___serializes_payload() {
    let req = RequestEnvelope::from_typed(
        "user.create",
        &TestRequest {
            name: "John".to_string(),
        },
    )
    .unwrap();

    let payload: TestRequest = req.payload_as().unwrap();

    assert_eq!(payload.name, "John");
}

#[test]
fn RequestEnvelope___to_bytes_from_bytes___roundtrip() {
    let original = RequestEnvelope::new("test.echo", serde_json::json!({"data": "hello"}))
        .with_request_id(456)
        .with_correlation_id("corr-789");

    let bytes = original.to_bytes().unwrap();
    let decoded = RequestEnvelope::from_bytes(&bytes).unwrap();

    assert_eq!(decoded.type_tag, original.type_tag);
    assert_eq!(decoded.request_id, original.request_id);
    assert_eq!(decoded.correlation_id, original.correlation_id);
}

#[test]
fn RequestEnvelope___payload_as___deserializes_typed_payload() {
    let req = RequestEnvelope::new("test", serde_json::json!({"name": "Test"}));

    let payload: TestRequest = req.payload_as().unwrap();

    assert_eq!(payload.name, "Test");
}

// ResponseEnvelope tests

#[test]
fn ResponseEnvelope___success___creates_success_response() {
    let resp = ResponseEnvelope::success(serde_json::json!({"result": true}));

    assert!(resp.is_success());
    assert_eq!(resp.status, ResponseStatus::Success);
    assert!(resp.payload.is_some());
    assert!(resp.error_code.is_none());
}

#[test]
fn ResponseEnvelope___success_typed___serializes_payload() {
    let resp = ResponseEnvelope::success_typed(&TestResponse { id: 42 }).unwrap();

    assert!(resp.is_success());
    let payload: TestResponse = resp.payload_as().unwrap().unwrap();
    assert_eq!(payload.id, 42);
}

#[test]
fn ResponseEnvelope___success_raw___parses_json_bytes() {
    let json_bytes = br#"{"id": 123}"#;

    let resp = ResponseEnvelope::success_raw(json_bytes).unwrap();

    assert!(resp.is_success());
    let payload: TestResponse = resp.payload_as().unwrap().unwrap();
    assert_eq!(payload.id, 123);
}

#[test]
fn ResponseEnvelope___error___creates_error_response() {
    let resp = ResponseEnvelope::error(404, "Not found");

    assert!(!resp.is_success());
    assert_eq!(resp.status, ResponseStatus::Error);
    assert_eq!(resp.error_code, Some(404));
    assert_eq!(resp.error_message, Some("Not found".to_string()));
    assert!(resp.payload.is_none());
}

#[test]
fn ResponseEnvelope___from_error___converts_plugin_error() {
    let plugin_err = rustbridge_core::PluginError::ConfigError("bad config".into());

    let resp = ResponseEnvelope::from_error(&plugin_err);

    assert!(!resp.is_success());
    assert_eq!(resp.error_code, Some(4));
    assert!(resp.error_message.unwrap().contains("bad config"));
}

#[test]
fn ResponseEnvelope___with_request_id___sets_id() {
    let resp = ResponseEnvelope::success(serde_json::json!({})).with_request_id(999);

    assert_eq!(resp.request_id, Some(999));
}

#[test]
fn ResponseEnvelope___to_bytes_from_bytes___roundtrip() {
    let original =
        ResponseEnvelope::success(serde_json::json!({"result": true})).with_request_id(123);

    let bytes = original.to_bytes().unwrap();
    let decoded = ResponseEnvelope::from_bytes(&bytes).unwrap();

    assert_eq!(decoded.status, original.status);
    assert_eq!(decoded.request_id, original.request_id);
}

#[test]
fn ResponseEnvelope___payload_as___returns_none_for_error_response() {
    let resp = ResponseEnvelope::error(500, "Error");

    let payload: Option<TestResponse> = resp.payload_as().unwrap();

    assert!(payload.is_none());
}

#[test]
fn ResponseEnvelope___default___returns_success_with_null_payload() {
    let resp = ResponseEnvelope::default();

    assert!(resp.is_success());
    assert_eq!(resp.payload, Some(serde_json::Value::Null));
}

// ResponseStatus tests

#[test]
fn ResponseStatus___success___serializes_to_snake_case() {
    let json = serde_json::to_string(&ResponseStatus::Success).unwrap();

    assert_eq!(json, r#""success""#);
}

#[test]
fn ResponseStatus___error___serializes_to_snake_case() {
    let json = serde_json::to_string(&ResponseStatus::Error).unwrap();

    assert_eq!(json, r#""error""#);
}
