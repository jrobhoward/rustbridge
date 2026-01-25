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

#[test]
fn set_init_params___stores_params() {
    let mut config = PluginConfig::default();

    config.set_init_params(serde_json::json!({
        "migrations_path": "/db/migrations",
        "seed_data": true
    }));

    assert!(config.init_params.is_some());
    let params = config.init_params.as_ref().unwrap();
    assert_eq!(params["migrations_path"], "/db/migrations");
    assert_eq!(params["seed_data"], true);
}

#[test]
fn get_init_param___extracts_typed_value() {
    let mut config = PluginConfig::default();
    config.set_init_params(serde_json::json!({
        "database": {
            "url": "postgres://localhost/test",
            "pool_size": 10
        },
        "cache_enabled": true
    }));

    #[derive(Deserialize, PartialEq, Debug)]
    struct DatabaseConfig {
        url: String,
        pool_size: u32,
    }

    let db_config: Option<DatabaseConfig> = config.get_init_param("database");
    assert!(db_config.is_some());
    let db_config = db_config.unwrap();
    assert_eq!(db_config.url, "postgres://localhost/test");
    assert_eq!(db_config.pool_size, 10);

    let cache_enabled: Option<bool> = config.get_init_param("cache_enabled");
    assert_eq!(cache_enabled, Some(true));

    let missing: Option<String> = config.get_init_param("nonexistent");
    assert_eq!(missing, None);
}

#[test]
fn get_init_param___returns_none_when_no_init_params() {
    let config = PluginConfig::default();

    let value: Option<String> = config.get_init_param("anything");
    assert_eq!(value, None);
}

#[test]
fn init_params_as___deserializes_entire_params() {
    let mut config = PluginConfig::default();

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct InitParams {
        setup_mode: String,
        enable_features: Vec<String>,
        max_retries: u32,
    }

    let params = InitParams {
        setup_mode: "development".to_string(),
        enable_features: vec!["feature_a".to_string(), "feature_b".to_string()],
        max_retries: 3,
    };

    config.set_init_params(serde_json::to_value(&params).unwrap());

    let deserialized: Option<InitParams> = config.init_params_as();
    assert!(deserialized.is_some());
    assert_eq!(deserialized.unwrap(), params);
}

#[test]
fn init_params_as___returns_none_when_not_set() {
    let config = PluginConfig::default();

    #[derive(Deserialize)]
    struct InitParams {
        value: String,
    }

    let params: Option<InitParams> = config.init_params_as();
    assert!(params.is_none());
}

#[test]
fn from_json___deserializes_init_params() {
    let json = r#"{
        "data": {"key": "value"},
        "init_params": {
            "setup_mode": "production",
            "seed_data": false
        },
        "log_level": "debug",
        "max_concurrent_ops": 500
    }"#;

    let config = PluginConfig::from_json(json.as_bytes()).unwrap();

    assert_eq!(config.log_level, "debug");
    assert_eq!(config.max_concurrent_ops, 500);
    assert!(config.init_params.is_some());

    let setup_mode: Option<String> = config.get_init_param("setup_mode");
    assert_eq!(setup_mode, Some("production".to_string()));

    let seed_data: Option<bool> = config.get_init_param("seed_data");
    assert_eq!(seed_data, Some(false));
}
