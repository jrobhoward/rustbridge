#![allow(non_snake_case)]

use super::*;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;

static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);
static CALLBACK_TEST_LOCK: Mutex<()> = Mutex::new(());

extern "C" fn test_callback(
    _level: u8,
    _target: *const std::ffi::c_char,
    _message: *const u8,
    _message_len: usize,
) {
    CALL_COUNT.fetch_add(1, Ordering::SeqCst);
}

// LogCallbackManager tests

#[test]
fn LogCallbackManager___new___default_level_is_info() {
    let manager = LogCallbackManager::new();

    assert_eq!(manager.level(), LogLevel::Info);
}

#[test]
fn LogCallbackManager___new___no_callback_set() {
    let manager = LogCallbackManager::new();

    assert!(manager.get_callback().is_none());
}

#[test]
fn LogCallbackManager___set_level___changes_level() {
    let manager = LogCallbackManager::new();

    manager.set_level(LogLevel::Debug);

    assert_eq!(manager.level(), LogLevel::Debug);
}

#[test]
fn LogCallbackManager___is_enabled___respects_level_ordering() {
    let manager = LogCallbackManager::new();
    manager.set_level(LogLevel::Debug);

    assert!(manager.is_enabled(LogLevel::Debug));
    assert!(manager.is_enabled(LogLevel::Info));
    assert!(manager.is_enabled(LogLevel::Warn));
    assert!(manager.is_enabled(LogLevel::Error));
    assert!(!manager.is_enabled(LogLevel::Trace));
}

#[test]
fn LogCallbackManager___set_callback___sets_callback() {
    let manager = LogCallbackManager::new();

    manager.set_callback(Some(test_callback));

    assert!(manager.get_callback().is_some());
}

#[test]
fn LogCallbackManager___set_callback_none___clears_callback() {
    let manager = LogCallbackManager::new();
    manager.set_callback(Some(test_callback));

    manager.set_callback(None);

    assert!(manager.get_callback().is_none());
}

#[test]
fn LogCallbackManager___log___invokes_callback_when_enabled() {
    let _guard = CALLBACK_TEST_LOCK.lock().unwrap();
    CALL_COUNT.store(0, Ordering::SeqCst);
    let manager = LogCallbackManager::new();
    manager.set_level(LogLevel::Info);
    manager.set_callback(Some(test_callback));

    manager.log(LogLevel::Info, "test::module", "Test message");

    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn LogCallbackManager___log___skips_callback_when_level_too_low() {
    let _guard = CALLBACK_TEST_LOCK.lock().unwrap();
    CALL_COUNT.store(0, Ordering::SeqCst);
    let manager = LogCallbackManager::new();
    manager.set_level(LogLevel::Info);
    manager.set_callback(Some(test_callback));

    manager.log(LogLevel::Debug, "test::module", "Debug message");

    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 0);
}

#[test]
fn LogCallbackManager___log___higher_levels_pass_filter() {
    let _guard = CALLBACK_TEST_LOCK.lock().unwrap();
    CALL_COUNT.store(0, Ordering::SeqCst);
    let manager = LogCallbackManager::new();
    manager.set_level(LogLevel::Info);
    manager.set_callback(Some(test_callback));

    manager.log(LogLevel::Error, "test::module", "Error message");

    assert_eq!(CALL_COUNT.load(Ordering::SeqCst), 1);
}

#[test]
fn LogCallbackManager___log___no_panic_without_callback() {
    let manager = LogCallbackManager::new();

    manager.log(LogLevel::Info, "test", "message");
}

#[test]
fn LogCallbackManager___default___same_as_new() {
    let manager = LogCallbackManager::default();

    assert_eq!(manager.level(), LogLevel::Info);
    assert!(manager.get_callback().is_none());
}
