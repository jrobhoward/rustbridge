#![allow(non_snake_case)]

use super::*;
use test_case::test_case;

// ============================================================================
// Parameterized error code mapping tests
// ============================================================================

/// Test that each error variant maps to the correct error code
#[test_case(
    PluginError::InvalidState {
        expected: "a".into(),
        actual: "b".into(),
    },
    1,
    "InvalidState"
)]
#[test_case(PluginError::InitializationFailed("test".into()), 2, "InitializationFailed")]
#[test_case(PluginError::ShutdownFailed("test".into()), 3, "ShutdownFailed")]
#[test_case(PluginError::ConfigError("test".into()), 4, "ConfigError")]
#[test_case(PluginError::SerializationError("test".into()), 5, "SerializationError")]
#[test_case(PluginError::UnknownMessageType("test".into()), 6, "UnknownMessageType")]
#[test_case(PluginError::HandlerError("test".into()), 7, "HandlerError")]
#[test_case(PluginError::RuntimeError("test".into()), 8, "RuntimeError")]
#[test_case(PluginError::Cancelled, 9, "Cancelled")]
#[test_case(PluginError::Timeout, 10, "Timeout")]
#[test_case(PluginError::Internal("test".into()), 11, "Internal")]
#[test_case(PluginError::FfiError("test".into()), 12, "FfiError")]
#[test_case(PluginError::TooManyRequests, 13, "TooManyRequests")]
fn PluginError___variant___maps_to_correct_code(
    error: PluginError,
    expected_code: u32,
    _variant_name: &str,
) {
    assert_eq!(
        error.error_code(),
        expected_code,
        "{} should map to code {}",
        _variant_name,
        expected_code
    );
}

// ============================================================================
// Parameterized from_code mapping tests
// ============================================================================

#[test_case(1, "InvalidState")]
#[test_case(2, "InitializationFailed")]
#[test_case(3, "ShutdownFailed")]
#[test_case(4, "ConfigError")]
#[test_case(5, "SerializationError")]
#[test_case(6, "UnknownMessageType")]
#[test_case(7, "HandlerError")]
#[test_case(8, "RuntimeError")]
#[test_case(9, "Cancelled")]
#[test_case(10, "Timeout")]
#[test_case(11, "Internal")]
#[test_case(12, "FfiError")]
#[test_case(13, "TooManyRequests")]
fn PluginError___from_code___creates_correct_variant(code: u32, _expected_variant: &str) {
    let error = PluginError::from_code(code, "test message".into());

    // Verify it creates an error with the expected code
    assert_eq!(
        error.error_code(),
        code,
        "from_code({}) should create error with code {}",
        code,
        code
    );
}

// ============================================================================
// Parameterized error message preservation tests
// ============================================================================

#[test_case(
    PluginError::InvalidState {
        expected: "Active".into(),
        actual: "Starting".into(),
    },
    "Active"
)]
#[test_case(PluginError::InitializationFailed("Runtime init failed".into()), "Runtime init failed")]
#[test_case(PluginError::ShutdownFailed("Timeout during shutdown".into()), "Timeout during shutdown")]
#[test_case(PluginError::ConfigError("Invalid JSON".into()), "Invalid JSON")]
#[test_case(PluginError::SerializationError("UTF-8 error".into()), "UTF-8 error")]
#[test_case(PluginError::UnknownMessageType("foo.bar.baz".into()), "foo.bar.baz")]
#[test_case(PluginError::HandlerError("Custom error".into()), "Custom error")]
#[test_case(PluginError::RuntimeError("Runtime panic".into()), "Runtime panic")]
#[test_case(PluginError::Internal("Internal issue".into()), "Internal issue")]
#[test_case(PluginError::FfiError("FFI call failed".into()), "FFI call failed")]
fn PluginError___message_variants___preserve_details(error: PluginError, expected_part: &str) {
    let display = error.to_string();

    assert!(
        display.contains(expected_part),
        "Error message '{}' should contain '{}'",
        display,
        expected_part
    );
}
