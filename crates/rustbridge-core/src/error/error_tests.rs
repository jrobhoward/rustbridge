#![allow(non_snake_case)]

use super::*;

#[test]
fn PluginError___config_error___returns_code_4() {
    let err = PluginError::ConfigError("test".into());

    let code = err.error_code();

    assert_eq!(code, 4);
}

#[test]
fn PluginError___unknown_message_type___displays_correctly() {
    let err = PluginError::UnknownMessageType("user.unknown".into());

    let display = err.to_string();

    assert_eq!(display, "unknown message type: user.unknown");
}

#[test]
fn PluginError___from_code_6___returns_unknown_message_type() {
    let err = PluginError::from_code(6, "test.message".into());

    assert!(matches!(err, PluginError::UnknownMessageType(_)));
}

#[test]
fn PluginError___from_code_unknown___returns_internal() {
    let err = PluginError::from_code(999, "unknown error".into());

    assert!(matches!(err, PluginError::Internal(_)));
}

#[test]
fn PluginError___all_variants___have_unique_codes() {
    let errors = vec![
        PluginError::InvalidState {
            expected: "a".into(),
            actual: "b".into(),
        },
        PluginError::InitializationFailed("".into()),
        PluginError::ShutdownFailed("".into()),
        PluginError::ConfigError("".into()),
        PluginError::SerializationError("".into()),
        PluginError::UnknownMessageType("".into()),
        PluginError::HandlerError("".into()),
        PluginError::RuntimeError("".into()),
        PluginError::Cancelled,
        PluginError::Timeout,
        PluginError::Internal("".into()),
        PluginError::FfiError("".into()),
    ];

    let codes: Vec<u32> = errors.iter().map(|e| e.error_code()).collect();
    let unique: std::collections::HashSet<u32> = codes.iter().copied().collect();

    assert_eq!(
        codes.len(),
        unique.len(),
        "All error codes should be unique"
    );
}

#[test]
fn PluginError___from_serde_error___converts_to_serialization_error() {
    let json_err = serde_json::from_str::<String>("invalid").unwrap_err();

    let plugin_err: PluginError = json_err.into();

    assert!(matches!(plugin_err, PluginError::SerializationError(_)));
}
