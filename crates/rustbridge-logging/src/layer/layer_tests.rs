#![allow(non_snake_case)]

use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use tracing_subscriber::prelude::*;

static TEST_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
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
