//! Integration tests for bundle variant loading.
//!
//! These tests require variant-log-plugin bundles to be built first:
//! ```bash
//! ./scripts/build-variant-test-bundles.sh
//! ```

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]

use rustbridge_consumer::{LogCallbackFn, LogLevel, NativePluginLoader, PluginConfig};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

// ============================================================================
// Test Message Types (matching variant-log-plugin)
// ============================================================================

#[derive(Debug, Serialize)]
struct IdentifyRequest {}

#[derive(Debug, Deserialize)]
struct IdentifyResponse {
    variant: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn base_bundle_path() -> PathBuf {
    workspace_root()
        .join("target")
        .join("variant-test-bundles")
        .join("variant-log-plugin-base-0.9.1.rbp")
}

fn extended_bundle_path() -> PathBuf {
    workspace_root()
        .join("target")
        .join("variant-test-bundles")
        .join("variant-log-plugin-extended-0.9.1.rbp")
}

fn bundles_available() -> bool {
    base_bundle_path().exists() && extended_bundle_path().exists()
}

// ============================================================================
// Serial Variant Tests
// ============================================================================

#[test]
#[ignore = "requires variant-log-plugin bundles: ./scripts/build-variant-test-bundles.sh"]
fn load_bundle_variant___base_release___returns_release_variant() {
    if !bundles_available() {
        eprintln!("Skipping test: variant-log-plugin bundles not built");
        return;
    }

    let config = PluginConfig {
        log_level: LogLevel::Info,
        ..PluginConfig::default()
    };
    let plugin = NativePluginLoader::load_bundle_variant_with_config(
        base_bundle_path(),
        "release",
        &config,
        None,
    )
    .unwrap();

    let response: IdentifyResponse = plugin.call_typed("identify", &IdentifyRequest {}).unwrap();

    assert_eq!(response.variant, "release");

    plugin.shutdown().unwrap();
}

#[test]
#[ignore = "requires variant-log-plugin bundles: ./scripts/build-variant-test-bundles.sh"]
fn load_bundle_variant___base_debug___returns_debug_variant() {
    if !bundles_available() {
        eprintln!("Skipping test: variant-log-plugin bundles not built");
        return;
    }

    let config = PluginConfig {
        log_level: LogLevel::Info,
        ..PluginConfig::default()
    };
    let plugin = NativePluginLoader::load_bundle_variant_with_config(
        base_bundle_path(),
        "debug",
        &config,
        None,
    )
    .unwrap();

    let response: IdentifyResponse = plugin.call_typed("identify", &IdentifyRequest {}).unwrap();

    assert_eq!(response.variant, "debug");

    plugin.shutdown().unwrap();
}

#[test]
#[ignore = "requires variant-log-plugin bundles: ./scripts/build-variant-test-bundles.sh"]
fn load_bundle_variant___extended_release___returns_release_extended_variant() {
    if !bundles_available() {
        eprintln!("Skipping test: variant-log-plugin bundles not built");
        return;
    }

    let config = PluginConfig {
        log_level: LogLevel::Info,
        ..PluginConfig::default()
    };
    let plugin = NativePluginLoader::load_bundle_variant_with_config(
        extended_bundle_path(),
        "release",
        &config,
        None,
    )
    .unwrap();

    let response: IdentifyResponse = plugin.call_typed("identify", &IdentifyRequest {}).unwrap();

    assert_eq!(response.variant, "release+extended-info");

    plugin.shutdown().unwrap();
}

#[test]
#[ignore = "requires variant-log-plugin bundles: ./scripts/build-variant-test-bundles.sh"]
fn load_bundle_variant___extended_debug___returns_debug_extended_variant() {
    if !bundles_available() {
        eprintln!("Skipping test: variant-log-plugin bundles not built");
        return;
    }

    let config = PluginConfig {
        log_level: LogLevel::Info,
        ..PluginConfig::default()
    };
    let plugin = NativePluginLoader::load_bundle_variant_with_config(
        extended_bundle_path(),
        "debug",
        &config,
        None,
    )
    .unwrap();

    let response: IdentifyResponse = plugin.call_typed("identify", &IdentifyRequest {}).unwrap();

    assert_eq!(response.variant, "debug+extended-info");

    plugin.shutdown().unwrap();
}

// ============================================================================
// Unload/Reload Test
// ============================================================================

#[test]
#[ignore = "requires variant-log-plugin bundles: ./scripts/build-variant-test-bundles.sh"]
fn load_bundle_variant___unload_and_reload___works() {
    if !bundles_available() {
        eprintln!("Skipping test: variant-log-plugin bundles not built");
        return;
    }

    let config = PluginConfig {
        log_level: LogLevel::Info,
        ..PluginConfig::default()
    };

    // Load release variant, call, then shut down and drop
    let plugin = NativePluginLoader::load_bundle_variant_with_config(
        base_bundle_path(),
        "release",
        &config,
        None,
    )
    .unwrap();

    let response: IdentifyResponse = plugin.call_typed("identify", &IdentifyRequest {}).unwrap();
    assert_eq!(response.variant, "release");

    plugin.shutdown().unwrap();
    drop(plugin);

    // Load debug variant from the same bundle
    let plugin = NativePluginLoader::load_bundle_variant_with_config(
        base_bundle_path(),
        "debug",
        &config,
        None,
    )
    .unwrap();

    let response: IdentifyResponse = plugin.call_typed("identify", &IdentifyRequest {}).unwrap();
    assert_eq!(response.variant, "debug");

    plugin.shutdown().unwrap();
}

// ============================================================================
// Log Callback Verification Test
// ============================================================================

#[test]
#[ignore = "requires variant-log-plugin bundles: ./scripts/build-variant-test-bundles.sh"]
fn load_bundle_variant___log_callback___captures_variant_text() {
    if !bundles_available() {
        eprintln!("Skipping test: variant-log-plugin bundles not built");
        return;
    }

    let captured_messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = captured_messages.clone();

    let callback: LogCallbackFn = Arc::new(move |_level, _target, message| {
        captured_clone.lock().unwrap().push(message.to_string());
    });

    let config = PluginConfig {
        log_level: LogLevel::Info,
        ..PluginConfig::default()
    };
    let plugin = NativePluginLoader::load_bundle_variant_with_config(
        extended_bundle_path(),
        "release",
        &config,
        Some(callback),
    )
    .unwrap();

    let _response: IdentifyResponse = plugin.call_typed("identify", &IdentifyRequest {}).unwrap();

    plugin.shutdown().unwrap();

    let messages = captured_messages.lock().unwrap();
    let has_variant_text = messages
        .iter()
        .any(|msg| msg.contains("release+extended-info"));
    assert!(
        has_variant_text,
        "Expected log messages to contain 'release+extended-info', got: {:?}",
        *messages,
    );
}

// ============================================================================
// Parallel Test
// ============================================================================

#[test]
#[ignore = "requires variant-log-plugin bundles: ./scripts/build-variant-test-bundles.sh"]
fn load_bundle_variant___parallel_from_two_bundles___both_respond_correctly() {
    if !bundles_available() {
        eprintln!("Skipping test: variant-log-plugin bundles not built");
        return;
    }

    let base_path = base_bundle_path();
    let extended_path = extended_bundle_path();

    let handle_base = std::thread::spawn(move || {
        let log_count = Arc::new(AtomicUsize::new(0));
        let log_count_clone = log_count.clone();

        let callback: LogCallbackFn = Arc::new(move |_level, _target, _message| {
            log_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let config = PluginConfig {
            log_level: LogLevel::Info,
            ..PluginConfig::default()
        };
        let plugin = NativePluginLoader::load_bundle_variant_with_config(
            &base_path,
            "release",
            &config,
            Some(callback),
        )
        .unwrap();

        let response: IdentifyResponse =
            plugin.call_typed("identify", &IdentifyRequest {}).unwrap();
        assert_eq!(response.variant, "release");

        plugin.shutdown().unwrap();

        assert!(log_count.load(Ordering::SeqCst) > 0);
    });

    let handle_extended = std::thread::spawn(move || {
        let log_count = Arc::new(AtomicUsize::new(0));
        let log_count_clone = log_count.clone();

        let callback: LogCallbackFn = Arc::new(move |_level, _target, _message| {
            log_count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let config = PluginConfig {
            log_level: LogLevel::Info,
            ..PluginConfig::default()
        };
        let plugin = NativePluginLoader::load_bundle_variant_with_config(
            &extended_path,
            "release",
            &config,
            Some(callback),
        )
        .unwrap();

        let response: IdentifyResponse =
            plugin.call_typed("identify", &IdentifyRequest {}).unwrap();
        assert_eq!(response.variant, "release+extended-info");

        plugin.shutdown().unwrap();

        assert!(log_count.load(Ordering::SeqCst) > 0);
    });

    handle_base.join().unwrap();
    handle_extended.join().unwrap();
}

// ============================================================================
// Error Test
// ============================================================================

#[test]
#[ignore = "requires variant-log-plugin bundles: ./scripts/build-variant-test-bundles.sh"]
fn load_bundle_variant___nonexistent_variant___returns_error() {
    if !bundles_available() {
        eprintln!("Skipping test: variant-log-plugin bundles not built");
        return;
    }

    let result = NativePluginLoader::load_bundle_variant_with_config(
        base_bundle_path(),
        "nonexistent",
        &PluginConfig::default(),
        None,
    );

    assert!(result.is_err());
}
