#![allow(non_snake_case)]

use super::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct TestMessage {
    id: u64,
    name: String,
}

// JsonCodec tests

#[test]
fn JsonCodec___encode_decode___roundtrip_preserves_data() {
    let codec = JsonCodec::new();
    let original = TestMessage {
        id: 42,
        name: "test".to_string(),
    };

    let encoded = codec.encode(&original).unwrap();
    let decoded: TestMessage = codec.decode(&encoded).unwrap();

    assert_eq!(original, decoded);
}

#[test]
fn JsonCodec___pretty___output_contains_newlines() {
    let codec = JsonCodec::pretty();
    let msg = TestMessage {
        id: 1,
        name: "test".to_string(),
    };

    let encoded = codec.encode_string(&msg).unwrap();

    assert!(encoded.contains('\n'));
}

#[test]
fn JsonCodec___new___output_is_compact() {
    let codec = JsonCodec::new();
    let msg = TestMessage {
        id: 1,
        name: "test".to_string(),
    };

    let encoded = codec.encode_string(&msg).unwrap();

    assert!(!encoded.contains('\n'));
}

#[test]
fn JsonCodec___decode___invalid_json_returns_error() {
    let codec = JsonCodec::new();

    let result: Result<TestMessage, _> = codec.decode(b"invalid json");

    assert!(result.is_err());
}

#[test]
fn JsonCodec___content_type___returns_application_json() {
    let codec = JsonCodec::new();

    assert_eq!(codec.content_type(), "application/json");
}

#[test]
fn JsonCodec___decode_str___parses_json_string() {
    let codec = JsonCodec::new();
    let json = r#"{"id": 99, "name": "from_string"}"#;

    let result: TestMessage = codec.decode_str(json).unwrap();

    assert_eq!(result.id, 99);
    assert_eq!(result.name, "from_string");
}

#[test]
fn JsonCodec___default___creates_compact_codec() {
    let codec = JsonCodec::default();
    let msg = TestMessage {
        id: 1,
        name: "test".to_string(),
    };

    let encoded = codec.encode_string(&msg).unwrap();

    assert!(!encoded.contains('\n'));
}

// CodecError tests

#[test]
fn CodecError___from_serde_error___syntax_error_becomes_deserialization() {
    let err = serde_json::from_str::<TestMessage>("invalid").unwrap_err();

    let codec_err: CodecError = err.into();

    assert!(matches!(codec_err, CodecError::Deserialization(_)));
}

#[test]
fn CodecError___display___shows_error_message() {
    let err = CodecError::Serialization("test error".into());

    let display = err.to_string();

    assert!(display.contains("test error"));
}

#[test]
fn CodecError___into_plugin_error___converts_correctly() {
    let codec_err = CodecError::InvalidFormat("bad format".into());

    let plugin_err: rustbridge_core::PluginError = codec_err.into();

    assert!(matches!(
        plugin_err,
        rustbridge_core::PluginError::SerializationError(_)
    ));
}
