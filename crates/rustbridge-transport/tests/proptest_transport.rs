//! Property-based tests for transport serialization
//!
//! Tests that envelopes serialize and deserialize correctly with any valid data,
//! and that serialization roundtrips preserve all fields.

use proptest::prelude::*;
use rustbridge_transport::{RequestEnvelope, ResponseEnvelope, ResponseStatus};

// Strategy: Generate valid JSON values (simple types for test speed)
fn arb_json_value() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        Just(serde_json::Value::Null),
        any::<bool>().prop_map(serde_json::Value::Bool),
        any::<i32>().prop_map(|i| serde_json::Value::Number(i.into())),
        ".*".prop_map(serde_json::Value::String),
    ]
}

// Strategy: Generate valid type tags (non-empty strings)
fn arb_type_tag() -> impl Strategy<Value = String> {
    "[a-zA-Z][a-zA-Z0-9._-]{0,99}"
}

proptest! {
    /// Property: RequestEnvelope can serialize and deserialize losslessly
    #[test]
    fn proptest_request_envelope_roundtrip(
        type_tag in arb_type_tag(),
        payload in arb_json_value(),
        request_id in any::<Option<u64>>(),
        correlation_id in ".*"
    ) {
        let mut envelope = RequestEnvelope::new(type_tag.clone(), payload.clone());
        if let Some(id) = request_id {
            envelope = envelope.with_request_id(id);
        }
        if !correlation_id.is_empty() {
            envelope = envelope.with_correlation_id(correlation_id.clone());
        }

        // Serialize to bytes
        let bytes = envelope
            .to_bytes()
            .expect("Serialization should succeed for valid data");

        // Deserialize from bytes
        let recovered = RequestEnvelope::from_bytes(&bytes)
            .expect("Deserialization should succeed for valid serialized data");

        // Verify all fields are preserved
        prop_assert_eq!(recovered.type_tag, type_tag);
        prop_assert_eq!(recovered.payload, payload);
        prop_assert_eq!(recovered.request_id, request_id);
        prop_assert_eq!(recovered.correlation_id, if correlation_id.is_empty() { None } else { Some(correlation_id) });
    }

    /// Property: ResponseEnvelope success path preserves data
    #[test]
    fn proptest_response_envelope_success_roundtrip(
        payload in arb_json_value(),
        request_id in any::<Option<u64>>()
    ) {
        let mut envelope = ResponseEnvelope::success(payload.clone());
        if let Some(id) = request_id {
            envelope = envelope.with_request_id(id);
        }

        let bytes = envelope
            .to_bytes()
            .expect("Serialization should succeed");

        let recovered = ResponseEnvelope::from_bytes(&bytes)
            .expect("Deserialization should succeed");

        prop_assert_eq!(recovered.status, ResponseStatus::Success);
        // Note: Null payloads may be serialized as missing fields, so we check appropriately
        if payload.is_null() {
            prop_assert!(recovered.payload.is_none() || recovered.payload == Some(payload));
        } else {
            prop_assert_eq!(recovered.payload, Some(payload));
        }
        prop_assert_eq!(recovered.error_code, None);
        prop_assert_eq!(recovered.error_message, None);
        prop_assert_eq!(recovered.request_id, request_id);
    }

    /// Property: ResponseEnvelope error path preserves error info
    #[test]
    fn proptest_response_envelope_error_roundtrip(code in 1u32..1000, message in ".*") {
        let envelope = ResponseEnvelope::error(code, message.clone());

        let bytes = envelope
            .to_bytes()
            .expect("Serialization should succeed");

        let recovered = ResponseEnvelope::from_bytes(&bytes)
            .expect("Deserialization should succeed");

        prop_assert_eq!(recovered.status, ResponseStatus::Error);
        prop_assert_eq!(recovered.error_code, Some(code));
        prop_assert_eq!(recovered.error_message, Some(message));
        prop_assert_eq!(recovered.payload, None);
    }

    /// Property: Large request/response pairs handle encoding correctly
    #[test]
    fn proptest_large_payload(data in prop::collection::vec(any::<u8>(), 100..10_000)) {
        let json_bytes = serde_json::to_value(&data).expect("Should convert to JSON");
        let envelope = RequestEnvelope::new("large_test", json_bytes.clone());

        let bytes = envelope.to_bytes().expect("Should serialize");
        let recovered = RequestEnvelope::from_bytes(&bytes).expect("Should deserialize");

        prop_assert_eq!(recovered.payload, json_bytes);
    }
}

#[test]
fn test_empty_type_tag() {
    let envelope = RequestEnvelope::new("", serde_json::Value::Null);
    let bytes = envelope
        .to_bytes()
        .expect("Even empty type tags should serialize");
    let recovered =
        RequestEnvelope::from_bytes(&bytes).expect("Should deserialize even with empty type tag");
    assert_eq!(recovered.type_tag, "");
}

#[test]
fn test_null_payload() {
    let envelope = RequestEnvelope::new("test", serde_json::Value::Null);
    let bytes = envelope.to_bytes().expect("Should serialize");
    let recovered = RequestEnvelope::from_bytes(&bytes).expect("Should deserialize");
    assert_eq!(recovered.payload, serde_json::Value::Null);
}
