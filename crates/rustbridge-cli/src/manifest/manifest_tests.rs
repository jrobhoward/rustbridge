#![allow(non_snake_case)]

use super::*;

// Manifest parsing tests

#[test]
fn Manifest___from_str___parses_valid_toml() {
    let toml = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
description = "A test plugin"

[messages."user.create"]
description = "Create a user"
request_schema = "schemas/CreateUserRequest.json"
response_schema = "schemas/CreateUserResponse.json"

[platforms]
linux-x86_64 = "libtestplugin.so"
darwin-aarch64 = "libtestplugin.dylib"
"#;

    let manifest = Manifest::from_str(toml).unwrap();

    assert_eq!(manifest.plugin.name, "test-plugin");
    assert_eq!(manifest.plugin.version, "1.0.0");
    assert!(manifest.messages.contains_key("user.create"));
    assert!(manifest.platforms.contains_key("linux-x86_64"));
}

#[test]
fn Manifest___from_str___parses_minimal_manifest() {
    let toml = r#"
[plugin]
name = "minimal"
version = "0.5.0"
"#;

    let manifest = Manifest::from_str(toml).unwrap();

    assert_eq!(manifest.plugin.name, "minimal");
    assert!(manifest.messages.is_empty());
    assert!(manifest.platforms.is_empty());
}

// Manifest validation tests

#[test]
fn Manifest___validate___accepts_valid_manifest() {
    let manifest = Manifest {
        plugin: PluginSection {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            authors: vec![],
        },
        messages: HashMap::new(),
        platforms: HashMap::new(),
    };

    assert!(manifest.validate().is_ok());
}

#[test]
fn Manifest___validate___rejects_empty_name() {
    let manifest = Manifest {
        plugin: PluginSection {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            authors: vec![],
        },
        messages: HashMap::new(),
        platforms: HashMap::new(),
    };

    assert!(manifest.validate().is_err());
}

#[test]
fn Manifest___validate___rejects_empty_version() {
    let manifest = Manifest {
        plugin: PluginSection {
            name: "test".to_string(),
            version: "".to_string(),
            description: None,
            authors: vec![],
        },
        messages: HashMap::new(),
        platforms: HashMap::new(),
    };

    assert!(manifest.validate().is_err());
}

#[test]
fn Manifest___validate___rejects_invalid_version_format() {
    let manifest = Manifest {
        plugin: PluginSection {
            name: "test".to_string(),
            version: "1".to_string(),
            description: None,
            authors: vec![],
        },
        messages: HashMap::new(),
        platforms: HashMap::new(),
    };

    assert!(manifest.validate().is_err());
}

#[test]
fn Manifest___validate___rejects_invalid_platform() {
    let mut platforms = HashMap::new();
    platforms.insert("invalid-platform".to_string(), "lib.so".to_string());

    let manifest = Manifest {
        plugin: PluginSection {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            authors: vec![],
        },
        messages: HashMap::new(),
        platforms,
    };

    assert!(manifest.validate().is_err());
}

#[test]
fn Manifest___validate___accepts_valid_platforms() {
    let mut platforms = HashMap::new();
    platforms.insert("linux-x86_64".to_string(), "lib.so".to_string());
    platforms.insert("darwin-aarch64".to_string(), "lib.dylib".to_string());

    let manifest = Manifest {
        plugin: PluginSection {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            authors: vec![],
        },
        messages: HashMap::new(),
        platforms,
    };

    assert!(manifest.validate().is_ok());
}
