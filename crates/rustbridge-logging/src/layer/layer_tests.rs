#![allow(non_snake_case)]

use super::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing_subscriber::prelude::*;

static TEST_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
static CAPTURED_MESSAGE: Mutex<Option<String>> = Mutex::new(None);
// Mutex to serialize tests that use the global manager
static GLOBAL_MANAGER_LOCK: Mutex<()> = Mutex::new(());

extern "C" fn test_log_callback(
    _level: u8,
    _target: *const std::ffi::c_char,
    _message: *const u8,
    _message_len: usize,
) {
    TEST_CALL_COUNT.fetch_add(1, Ordering::SeqCst);
}

extern "C" fn capture_message_callback(
    _level: u8,
    _target: *const std::ffi::c_char,
    message: *const u8,
    message_len: usize,
) {
    if !message.is_null() && message_len > 0 {
        let slice = unsafe { std::slice::from_raw_parts(message, message_len) };
        if let Ok(msg) = std::str::from_utf8(slice) {
            *CAPTURED_MESSAGE.lock().unwrap() = Some(msg.to_string());
        }
    }
}

// FfiLoggingLayer tests

#[test]
fn FfiLoggingLayer___convert_level___trace() {
    let result = FfiLoggingLayer::convert_level(&Level::TRACE);

    assert_eq!(result, LogLevel::Trace);
}

#[test]
fn FfiLoggingLayer___convert_level___debug() {
    let result = FfiLoggingLayer::convert_level(&Level::DEBUG);

    assert_eq!(result, LogLevel::Debug);
}

#[test]
fn FfiLoggingLayer___convert_level___info() {
    let result = FfiLoggingLayer::convert_level(&Level::INFO);

    assert_eq!(result, LogLevel::Info);
}

#[test]
fn FfiLoggingLayer___convert_level___warn() {
    let result = FfiLoggingLayer::convert_level(&Level::WARN);

    assert_eq!(result, LogLevel::Warn);
}

#[test]
fn FfiLoggingLayer___convert_level___error() {
    let result = FfiLoggingLayer::convert_level(&Level::ERROR);

    assert_eq!(result, LogLevel::Error);
}

#[test]
fn FfiLoggingLayer___new___uses_global_manager() {
    let _layer = FfiLoggingLayer::new();
    // Should not panic - verifies it can access global manager
}

#[test]
fn FfiLoggingLayer___default___same_as_new() {
    let _layer = FfiLoggingLayer::default();
    // Should not panic
}

#[test]
fn FfiLoggingLayer___with_callback___logs_info_and_error() {
    let _guard = GLOBAL_MANAGER_LOCK.lock().unwrap();

    let manager = LogCallbackManager::global();
    manager.set_callback(Some(test_log_callback));
    manager.set_level(LogLevel::Info);

    let layer = FfiLoggingLayer::new();
    let subscriber = tracing_subscriber::registry().with(layer);

    let before = TEST_CALL_COUNT.load(Ordering::SeqCst);
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("Test info message");
        tracing::debug!("Test debug message");
        tracing::error!("Test error message");
    });
    let after = TEST_CALL_COUNT.load(Ordering::SeqCst);

    // Info and Error should have been logged, Debug filtered
    assert_eq!(after - before, 2);

    manager.set_callback(None);
}

#[test]
fn FfiLoggingLayer___with_callback___filters_below_level() {
    let _guard = GLOBAL_MANAGER_LOCK.lock().unwrap();

    let manager = LogCallbackManager::global();
    manager.set_callback(Some(test_log_callback));
    manager.set_level(LogLevel::Warn);

    let layer = FfiLoggingLayer::new();
    let subscriber = tracing_subscriber::registry().with(layer);

    let before = TEST_CALL_COUNT.load(Ordering::SeqCst);
    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("Info message");
        tracing::debug!("Debug message");
        tracing::warn!("Warn message");
    });
    let after = TEST_CALL_COUNT.load(Ordering::SeqCst);

    assert_eq!(after - before, 1);

    manager.set_callback(None);
}

#[test]
fn FfiLoggingLayer___with_structured_fields___includes_fields_in_message() {
    let _guard = GLOBAL_MANAGER_LOCK.lock().unwrap();

    // Clear any previous captured message
    *CAPTURED_MESSAGE.lock().unwrap() = None;

    let manager = LogCallbackManager::global();
    manager.set_callback(Some(capture_message_callback));
    manager.set_level(LogLevel::Info);

    let layer = FfiLoggingLayer::new();
    let subscriber = tracing_subscriber::registry().with(layer);

    tracing::subscriber::with_default(subscriber, || {
        tracing::info!(cache_size = 100, "Plugin started");
    });

    let captured = CAPTURED_MESSAGE.lock().unwrap().clone();
    manager.set_callback(None);

    assert!(captured.is_some(), "Message should have been captured");
    let msg = captured.unwrap();

    // Verify message contains both the base message and structured fields
    assert!(
        msg.contains("Plugin started"),
        "Message should contain base text: {}",
        msg
    );
    assert!(
        msg.contains("cache_size=100"),
        "Message should contain structured field: {}",
        msg
    );
}

#[test]
fn FfiLoggingLayer___with_multiple_fields___includes_all_fields() {
    let _guard = GLOBAL_MANAGER_LOCK.lock().unwrap();

    *CAPTURED_MESSAGE.lock().unwrap() = None;

    let manager = LogCallbackManager::global();
    manager.set_callback(Some(capture_message_callback));
    manager.set_level(LogLevel::Debug);

    let layer = FfiLoggingLayer::new();
    let subscriber = tracing_subscriber::registry().with(layer);

    tracing::subscriber::with_default(subscriber, || {
        tracing::debug!(
            pattern = r"\d+",
            matches = true,
            cached = false,
            "Match completed"
        );
    });

    let captured = CAPTURED_MESSAGE.lock().unwrap().clone();
    manager.set_callback(None);

    assert!(captured.is_some(), "Message should have been captured");
    let msg = captured.unwrap();

    // Verify all fields are present
    assert!(
        msg.contains("Match completed"),
        "Message should contain base text: {}",
        msg
    );
    assert!(
        msg.contains("pattern="),
        "Message should contain pattern field: {}",
        msg
    );
    assert!(
        msg.contains("matches=true"),
        "Message should contain matches field: {}",
        msg
    );
    assert!(
        msg.contains("cached=false"),
        "Message should contain cached field: {}",
        msg
    );
}

#[test]
fn FfiLoggingLayer___message_only___no_extra_fields() {
    let _guard = GLOBAL_MANAGER_LOCK.lock().unwrap();

    *CAPTURED_MESSAGE.lock().unwrap() = None;

    let manager = LogCallbackManager::global();
    manager.set_callback(Some(capture_message_callback));
    manager.set_level(LogLevel::Info);

    let layer = FfiLoggingLayer::new();
    let subscriber = tracing_subscriber::registry().with(layer);

    tracing::subscriber::with_default(subscriber, || {
        tracing::info!("Simple message");
    });

    let captured = CAPTURED_MESSAGE.lock().unwrap().clone();
    manager.set_callback(None);

    assert!(captured.is_some(), "Message should have been captured");
    let msg = captured.unwrap();

    // Verify it's just the message, no extra fields
    assert_eq!(
        msg.trim(),
        "Simple message",
        "Message should be just the text: {}",
        msg
    );
}
