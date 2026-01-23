#![allow(non_snake_case)]

use super::*;

#[test]
fn PluginConfig___default___has_expected_values() {
    let config = PluginConfig::default();

    assert_eq!(config.log_level, "info");
    assert_eq!(config.max_concurrent_ops, 1000);
    assert_eq!(config.shutdown_timeout_ms, 5000);
    assert!(config.worker_threads.is_none());
}

#[test]
fn PluginConfig___from_json___parses_log_level() {
    let json = r#"{"log_level": "debug"}"#;

    let config = PluginConfig::from_json(json.as_bytes()).unwrap();

    assert_eq!(config.log_level, "debug");
}

#[test]
fn PluginConfig___from_json___parses_data_field() {
    let json = r#"{"data": {"key": "value"}}"#;

    let config = PluginConfig::from_json(json.as_bytes()).unwrap();

    assert_eq!(config.get::<String>("key"), Some("value".to_string()));
}

#[test]
fn PluginConfig___from_json___parses_worker_threads() {
    let json = r#"{"worker_threads": 4}"#;

    let config = PluginConfig::from_json(json.as_bytes()).unwrap();

    assert_eq!(config.worker_threads, Some(4));
}

#[test]
fn PluginConfig___from_empty_bytes___returns_defaults() {
    let config = PluginConfig::from_json(&[]).unwrap();

    assert_eq!(config.log_level, "info");
    assert_eq!(config.max_concurrent_ops, 1000);
}

#[test]
fn PluginConfig___set_and_get___roundtrips_value() {
    let mut config = PluginConfig::new();

    config.set("test_key", 42).unwrap();

    assert_eq!(config.get::<i32>("test_key"), Some(42));
}

#[test]
fn PluginConfig___set___works_when_data_is_null() {
    let mut config = PluginConfig::new();
    config.data = serde_json::Value::Null;

    config.set("key", "value").unwrap();

    assert_eq!(config.get::<String>("key"), Some("value".to_string()));
}

#[test]
fn PluginConfig___get___returns_none_for_missing_key() {
    let config = PluginConfig::default();

    let result = config.get::<String>("nonexistent");

    assert!(result.is_none());
}

#[test]
fn PluginConfig___get___returns_none_for_wrong_type() {
    let mut config = PluginConfig::new();
    config.set("string_key", "not a number").unwrap();

    let result = config.get::<i32>("string_key");

    assert!(result.is_none());
}

#[test]
fn PluginMetadata___new___sets_name_and_version() {
    let metadata = PluginMetadata::new("test-plugin", "1.0.0");

    assert_eq!(metadata.name, "test-plugin");
    assert_eq!(metadata.version, "1.0.0");
    assert!(metadata.description.is_none());
    assert!(metadata.authors.is_empty());
}
