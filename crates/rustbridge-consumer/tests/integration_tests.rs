//! Integration tests for rustbridge-consumer.
//!
//! These tests require the hello-plugin to be built first:
//! ```bash
//! cargo build --release -p hello-plugin
//! ```

#![allow(non_snake_case)]
#![allow(clippy::unwrap_used)]

use rustbridge_consumer::{
    ConsumerError, LifecycleState, LogLevel, NativePluginLoader, PluginConfig,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

// ============================================================================
// Test Message Types (matching hello-plugin)
// ============================================================================

#[derive(Debug, Serialize)]
struct EchoRequest {
    message: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct EchoResponse {
    message: String,
    length: usize,
}

#[derive(Debug, Serialize)]
struct GreetRequest {
    name: String,
}

#[derive(Debug, Deserialize)]
struct GreetResponse {
    greeting: String,
}

#[derive(Debug, Serialize)]
struct AddRequest {
    a: i64,
    b: i64,
}

#[derive(Debug, Deserialize)]
struct AddResponse {
    result: i64,
}

#[derive(Debug, Serialize)]
struct CreateUserRequest {
    username: String,
    email: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserResponse {
    user_id: String,
    created_at: String,
}

#[derive(Debug, Serialize)]
struct SmallRequest {
    key: String,
    flags: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct SmallResponse {
    value: String,
    ttl_seconds: u32,
    cache_hit: bool,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Get the path to the hello-plugin shared library.
fn hello_plugin_path() -> PathBuf {
    let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    #[cfg(target_os = "linux")]
    let lib_name = "libhello_plugin.so";
    #[cfg(target_os = "macos")]
    let lib_name = "libhello_plugin.dylib";
    #[cfg(target_os = "windows")]
    let lib_name = "hello_plugin.dll";
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    let lib_name = "libhello_plugin.so";

    base.join("target").join("release").join(lib_name)
}

/// Check if the hello-plugin is available.
fn plugin_available() -> bool {
    hello_plugin_path().exists()
}

// ============================================================================
// Loading Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built: cargo build --release -p hello-plugin"]
fn NativePluginLoader___load___hello_plugin___returns_active_plugin() {
    if !plugin_available() {
        eprintln!("Skipping test: hello-plugin not built");
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    assert_eq!(plugin.state(), LifecycleState::Active);
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePluginLoader___load_with_config___applies_log_level() {
    if !plugin_available() {
        eprintln!("Skipping test: hello-plugin not built");
        return;
    }

    let config = PluginConfig {
        log_level: LogLevel::Debug,
        ..PluginConfig::default()
    };

    let plugin = NativePluginLoader::load_with_config(hello_plugin_path(), &config, None).unwrap();

    assert_eq!(plugin.state(), LifecycleState::Active);
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePluginLoader___load_with_config___with_log_callback___receives_logs() {
    if !plugin_available() {
        eprintln!("Skipping test: hello-plugin not built");
        return;
    }

    let log_count = Arc::new(AtomicUsize::new(0));
    let log_count_clone = log_count.clone();

    let callback: rustbridge_consumer::LogCallbackFn =
        Arc::new(move |_level, _target, _message| {
            log_count_clone.fetch_add(1, Ordering::SeqCst);
        });

    let config = PluginConfig {
        log_level: LogLevel::Debug,
        ..PluginConfig::default()
    };

    let plugin =
        NativePluginLoader::load_with_config(hello_plugin_path(), &config, Some(callback)).unwrap();

    // Make a call to ensure log messages are generated
    let _response: EchoResponse = plugin
        .call_typed(
            "echo",
            &EchoRequest {
                message: "log test".to_string(),
            },
        )
        .unwrap();

    assert!(log_count.load(Ordering::SeqCst) > 0);
    assert_eq!(plugin.state(), LifecycleState::Active);
}

// ============================================================================
// JSON Call Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call___echo___returns_message_with_length() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let response = plugin
        .call("echo", r#"{"message": "Hello, World!"}"#)
        .unwrap();

    assert!(response.contains("Hello, World!"));
    assert!(response.contains("13")); // length
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call_typed___echo___deserializes_correctly() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let response: EchoResponse = plugin
        .call_typed(
            "echo",
            &EchoRequest {
                message: "Test".to_string(),
            },
        )
        .unwrap();

    assert_eq!(response.message, "Test");
    assert_eq!(response.length, 4);
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call_typed___greet___returns_greeting() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let response: GreetResponse = plugin
        .call_typed(
            "greet",
            &GreetRequest {
                name: "Rustacean".to_string(),
            },
        )
        .unwrap();

    assert!(response.greeting.contains("Rustacean"));
    assert!(response.greeting.contains("Welcome to rustbridge"));
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call_typed___math_add___returns_sum() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let response: AddResponse = plugin
        .call_typed("math.add", &AddRequest { a: 40, b: 2 })
        .unwrap();

    assert_eq!(response.result, 42);
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call_typed___user_create___returns_user_id() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let response: CreateUserResponse = plugin
        .call_typed(
            "user.create",
            &CreateUserRequest {
                username: "johndoe".to_string(),
                email: "john@example.com".to_string(),
            },
        )
        .unwrap();

    assert!(response.user_id.starts_with("user-"));
    assert!(!response.created_at.is_empty());
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call___unknown_type___returns_error() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let result = plugin.call("unknown.type", "{}");

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, ConsumerError::CallFailed(_)));
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call___invalid_json___returns_error() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let result = plugin.call("echo", "not valid json");

    assert!(result.is_err());
}

// ============================================================================
// Lifecycle Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___state___after_load___is_active() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    assert_eq!(plugin.state(), LifecycleState::Active);
    assert!(plugin.state().can_handle_requests());
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___shutdown___transitions_to_stopped() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    plugin.shutdown().unwrap();

    assert_eq!(plugin.state(), LifecycleState::Stopped);
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___shutdown___can_be_called_multiple_times() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    plugin.shutdown().unwrap();
    plugin.shutdown().unwrap(); // Should be idempotent

    assert_eq!(plugin.state(), LifecycleState::Stopped);
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___drop___calls_shutdown() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();
    drop(plugin);

    // If we get here without panic, shutdown was successful
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call___after_shutdown___returns_not_active_error() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();
    plugin.shutdown().unwrap();

    let result = plugin.call("echo", r#"{"message": "test"}"#);

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, ConsumerError::NotActive(_)));
}

// ============================================================================
// Log Level Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___set_log_level___changes_level() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    plugin.set_log_level(LogLevel::Trace);
    plugin.set_log_level(LogLevel::Error);

    // If we get here without panic, set_log_level works
    assert_eq!(plugin.state(), LifecycleState::Active);
}

// ============================================================================
// Rejected Request Count Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___rejected_request_count___starts_at_zero() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    assert_eq!(plugin.rejected_request_count(), 0);
}

// ============================================================================
// Binary Transport Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___has_binary_transport___returns_true() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    // hello-plugin should have binary transport
    assert!(plugin.has_binary_transport());
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call_raw___small_benchmark___returns_valid_response() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    // Build a SmallRequestRaw (76 bytes): version(1) + reserved(3) + key(64) + key_len(4) + flags(4)
    let mut request = [0u8; 76];
    request[0] = 1; // version
    let key = b"test-key";
    request[4..4 + key.len()].copy_from_slice(key);
    request[68..72].copy_from_slice(&(key.len() as u32).to_ne_bytes()); // key_len
    request[72..76].copy_from_slice(&1u32.to_ne_bytes()); // flags = 1 (cache_hit)

    let response = plugin.call_raw(1, &request).unwrap(); // message_id 1 = MSG_BENCH_SMALL

    // SmallResponseRaw is 80 bytes
    assert_eq!(response.len(), 80);

    // Parse response fields
    let version = response[0];
    assert_eq!(version, 1);

    let value_len = u32::from_ne_bytes(response[68..72].try_into().unwrap()) as usize;
    assert!(value_len > 0);
    let value = std::str::from_utf8(&response[4..4 + value_len]).unwrap();
    assert!(value.contains("test-key"));

    let cache_hit = response[76];
    assert_eq!(cache_hit, 1); // flags & 1 != 0
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call_raw___unknown_message_id___returns_error() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let request = [0u8; 76];
    let result = plugin.call_raw(999, &request);

    assert!(result.is_err());
}

// ============================================================================
// Unicode Handling Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call___unicode_message___handles_correctly() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let response: EchoResponse = plugin
        .call_typed(
            "echo",
            &EchoRequest {
                message: "こんにちは世界 🌍".to_string(),
            },
        )
        .unwrap();

    assert_eq!(response.message, "こんにちは世界 🌍");
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call___empty_message___handles_correctly() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let response: EchoResponse = plugin
        .call_typed(
            "echo",
            &EchoRequest {
                message: String::new(),
            },
        )
        .unwrap();

    assert_eq!(response.message, "");
    assert_eq!(response.length, 0);
}

// ============================================================================
// Benchmark Handler Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call___bench_small___returns_response() {
    if !plugin_available() {
        return;
    }

    let plugin = NativePluginLoader::load(hello_plugin_path()).unwrap();

    let response: SmallResponse = plugin
        .call_typed(
            "bench.small",
            &SmallRequest {
                key: "test-key".to_string(),
                flags: 1,
            },
        )
        .unwrap();

    assert!(response.value.contains("test-key"));
    assert!(response.cache_hit); // flags & 1 != 0
}

// ============================================================================
// Multiple Plugins Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePluginLoader___load___multiple_instances___independent() {
    if !plugin_available() {
        return;
    }

    let plugin1 = NativePluginLoader::load(hello_plugin_path()).unwrap();
    let plugin2 = NativePluginLoader::load(hello_plugin_path()).unwrap();

    // Both should be active
    assert_eq!(plugin1.state(), LifecycleState::Active);
    assert_eq!(plugin2.state(), LifecycleState::Active);

    // Shutdown one, the other should still work
    plugin1.shutdown().unwrap();
    assert_eq!(plugin1.state(), LifecycleState::Stopped);
    assert_eq!(plugin2.state(), LifecycleState::Active);

    // Plugin 2 should still work
    let response: EchoResponse = plugin2
        .call_typed(
            "echo",
            &EchoRequest {
                message: "still works".to_string(),
            },
        )
        .unwrap();
    assert_eq!(response.message, "still works");
}

// ============================================================================
// Thread Safety Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___call___from_multiple_threads___works() {
    if !plugin_available() {
        return;
    }

    use std::thread;

    let plugin = Arc::new(NativePluginLoader::load(hello_plugin_path()).unwrap());

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let plugin = Arc::clone(&plugin);
            thread::spawn(move || {
                for j in 0..10 {
                    let response: AddResponse = plugin
                        .call_typed("math.add", &AddRequest { a: i, b: j })
                        .unwrap();
                    assert_eq!(response.result, i + j);
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

// ============================================================================
// Error Path Tests
// ============================================================================

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePlugin___rejected_request_count___increments_under_contention() {
    if !plugin_available() {
        return;
    }

    use std::thread;

    let config = PluginConfig {
        max_concurrent_ops: 1, // Very low limit to force rejections
        ..PluginConfig::default()
    };
    let plugin =
        Arc::new(NativePluginLoader::load_with_config(hello_plugin_path(), &config, None).unwrap());

    let error_count = Arc::new(AtomicUsize::new(0));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let plugin = Arc::clone(&plugin);
            let error_count = Arc::clone(&error_count);
            thread::spawn(move || {
                for _ in 0..50 {
                    if plugin.call("echo", r#"{"message": "flood"}"#).is_err() {
                        error_count.fetch_add(1, Ordering::SeqCst);
                    }
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    let rejected = plugin.rejected_request_count();
    let errors = error_count.load(Ordering::SeqCst);

    // With max_concurrent_ops=1 and 8 threads, we expect some rejections
    assert!(
        rejected > 0,
        "Expected some rejected requests with max_concurrent_ops=1, got {rejected}"
    );
    assert_eq!(
        rejected, errors as u64,
        "Rejected count ({rejected}) should match error count ({errors})"
    );
}

#[test]
#[ignore = "requires hello-plugin to be built"]
fn NativePluginLoader___load_by_name___hello_plugin___loads_successfully() {
    if !plugin_available() {
        return;
    }

    // load_by_name searches ./target/release among other paths
    let plugin = NativePluginLoader::load_by_name("hello_plugin").unwrap();

    assert_eq!(plugin.state(), LifecycleState::Active);

    let response: EchoResponse = plugin
        .call_typed(
            "echo",
            &EchoRequest {
                message: "loaded by name".to_string(),
            },
        )
        .unwrap();

    assert_eq!(response.message, "loaded by name");
}

#[test]
fn NativePluginLoader___load_by_name___nonexistent___returns_error() {
    let result = NativePluginLoader::load_by_name("nonexistent_plugin_xyz");

    assert!(result.is_err());
}

#[test]
#[ignore = "requires hello-plugin bundle: cd examples/hello-plugin && rustbridge pack --no-sign"]
fn NativePluginLoader___load_bundle_with_verification___no_verify___loads_plugin() {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    // Search multiple bundle locations (rustbridge pack outputs to example's target/bundle/)
    let bundle_path = [
        workspace_root.join("examples/hello-plugin/target/bundle"),
        workspace_root.join("target/bundle"),
    ]
    .iter()
    .flat_map(|dir| {
        std::fs::read_dir(dir)
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension().is_some_and(|ext| ext == "rbp")
                    && p.file_name()
                        .is_some_and(|n| n.to_string_lossy().starts_with("hello-plugin"))
            })
    })
    .next();

    let Some(bundle_path) = bundle_path else {
        eprintln!("Skipping test: no hello-plugin .rbp bundle found");
        return;
    };

    let plugin = NativePluginLoader::load_bundle_with_verification(
        &bundle_path,
        &PluginConfig::default(),
        None,
        false, // no signature verification
        None,
    )
    .unwrap();

    assert_eq!(plugin.state(), LifecycleState::Active);

    let response: EchoResponse = plugin
        .call_typed(
            "echo",
            &EchoRequest {
                message: "from verified bundle".to_string(),
            },
        )
        .unwrap();

    assert_eq!(response.message, "from verified bundle");
}

#[test]
fn NativePluginLoader___load_bundle_with_verification___nonexistent___returns_error() {
    let result = NativePluginLoader::load_bundle_with_verification(
        "/nonexistent/bundle.rbp",
        &PluginConfig::default(),
        None,
        false,
        None,
    );

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, ConsumerError::Bundle(_)));
}

// ============================================================================
// Error Path Tests
// ============================================================================

#[test]
fn NativePluginLoader___load___nonexistent_path___returns_library_load_error() {
    let result = NativePluginLoader::load("/nonexistent/path/libplugin.so");

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, ConsumerError::LibraryLoad(_)));
}

#[test]
fn NativePluginLoader___load_bundle___nonexistent_bundle___returns_bundle_error() {
    let result = NativePluginLoader::load_bundle("/nonexistent/bundle.rbp");

    assert!(result.is_err());
    let err = result.err().unwrap();
    assert!(matches!(err, ConsumerError::Bundle(_)));
}
