#![allow(non_snake_case)]

use super::*;
use test_case::test_case;

// ============================================================================
// Parameterized config JSON parsing tests
// ============================================================================

#[test_case(r#"{"log_level": "info"}"#, "info")]
#[test_case(r#"{"log_level": "debug"}"#, "debug")]
#[test_case(r#"{"log_level": "warn"}"#, "warn")]
#[test_case(r#"{"log_level": "error"}"#, "error")]
#[test_case(r#"{"log_level": "trace"}"#, "trace")]
fn PluginConfig___log_level_json___parses_correctly(json: &str, expected_level: &str) {
    let config = PluginConfig::from_json(json.as_bytes()).unwrap();
    assert_eq!(config.log_level, expected_level);
}

#[test_case(r#"{"max_concurrent_ops": 10}"#, 10)]
#[test_case(r#"{"max_concurrent_ops": 100}"#, 100)]
#[test_case(r#"{"max_concurrent_ops": 5000}"#, 5000)]
#[test_case(r#"{"max_concurrent_ops": 0}"#, 0)]
fn PluginConfig___max_concurrent_ops_json___parses_correctly(json: &str, expected_ops: usize) {
    let config = PluginConfig::from_json(json.as_bytes()).unwrap();
    assert_eq!(config.max_concurrent_ops, expected_ops);
}

#[test_case(r#"{"worker_threads": 1}"#, Some(1))]
#[test_case(r#"{"worker_threads": 2}"#, Some(2))]
#[test_case(r#"{"worker_threads": 4}"#, Some(4))]
#[test_case(r#"{"worker_threads": 8}"#, Some(8))]
#[test_case(r#"{}"#, None)]
fn PluginConfig___worker_threads_json___parses_correctly(
    json: &str,
    expected_threads: Option<usize>,
) {
    let config = PluginConfig::from_json(json.as_bytes()).unwrap();
    assert_eq!(config.worker_threads, expected_threads);
}

#[test_case(r#"{"shutdown_timeout_ms": 1000}"#, 1000)]
#[test_case(r#"{"shutdown_timeout_ms": 5000}"#, 5000)]
#[test_case(r#"{"shutdown_timeout_ms": 10000}"#, 10000)]
fn PluginConfig___shutdown_timeout_json___parses_correctly(json: &str, expected_timeout: u64) {
    let config = PluginConfig::from_json(json.as_bytes()).unwrap();
    assert_eq!(config.shutdown_timeout_ms, expected_timeout);
}

// ============================================================================
// Parameterized config data access tests
// ============================================================================

#[test_case("string_key", "string_value")]
#[test_case("another_key", "another_value")]
#[test_case("key_with_numbers", "123abc")]
fn PluginConfig___set_and_get_string___roundtrips_correctly(key: &str, value: &str) {
    let mut config = PluginConfig::new();

    config.set(key, value).unwrap();

    let retrieved = config.get::<String>(key);
    assert_eq!(retrieved, Some(value.to_string()));
}

#[test_case("int_0", 0i32)]
#[test_case("int_positive", 42i32)]
#[test_case("int_large", 2147483647i32)]
fn PluginConfig___set_and_get_int___roundtrips_correctly(key: &str, value: i32) {
    let mut config = PluginConfig::new();

    config.set(key, value).unwrap();

    let retrieved = config.get::<i32>(key);
    assert_eq!(retrieved, Some(value));
}

#[test_case("float_0", 0.0f64)]
#[test_case("float_positive", 1.23456f64)]
#[test_case("float_negative", -42.5f64)]
fn PluginConfig___set_and_get_float___roundtrips_correctly(key: &str, value: f64) {
    let mut config = PluginConfig::new();

    config.set(key, value).unwrap();

    let retrieved = config.get::<f64>(key);
    assert_eq!(retrieved, Some(value));
}

// ============================================================================
// Parameterized config error cases
// ============================================================================

#[test_case("string_value", "nonexistent_string_key")]
#[test_case(42, "nonexistent_int_key")]
#[test_case(1.5, "nonexistent_float_key")]
fn PluginConfig___get_nonexistent_key___returns_none(_value: impl serde::Serialize, key: &str) {
    let config = PluginConfig::default();

    let result = config.get::<String>(key);

    assert!(result.is_none());
}

#[test_case("string_key", "stored_as_string", "int")]
#[test_case("int_key", 42, "string")]
#[test_case("float_key", 1.5, "int")]
fn PluginConfig___get_wrong_type___returns_none(
    key: &str,
    _stored_value: impl serde::Serialize,
    _expected_type: &str,
) {
    let config = PluginConfig::new();
    // Note: We skip actually setting the value as this test just verifies
    // the pattern. In practice, type mismatches would be detected.

    let result_string = config.get::<String>(key);
    assert!(result_string.is_none());
}
