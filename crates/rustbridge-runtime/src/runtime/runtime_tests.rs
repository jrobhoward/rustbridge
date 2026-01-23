#![allow(non_snake_case)]

use super::*;

// RuntimeConfig tests

#[test]
fn RuntimeConfig___default___has_expected_values() {
    let config = RuntimeConfig::default();

    assert!(config.worker_threads.is_none());
    assert_eq!(config.thread_name, "rustbridge-worker");
    assert!(config.enable_io);
    assert!(config.enable_time);
    assert_eq!(config.max_blocking_threads, 512);
}

#[test]
fn RuntimeConfig___with_worker_threads___sets_threads() {
    let config = RuntimeConfig::new().with_worker_threads(4);

    assert_eq!(config.worker_threads, Some(4));
}

#[test]
fn RuntimeConfig___with_thread_name___sets_name() {
    let config = RuntimeConfig::new().with_thread_name("custom-worker");

    assert_eq!(config.thread_name, "custom-worker");
}

#[test]
fn RuntimeConfig___builder_chain___combines_options() {
    let config = RuntimeConfig::new()
        .with_worker_threads(2)
        .with_thread_name("test-worker");

    assert_eq!(config.worker_threads, Some(2));
    assert_eq!(config.thread_name, "test-worker");
}

// AsyncRuntime tests

#[test]
fn AsyncRuntime___with_defaults___creates_runtime() {
    let runtime = AsyncRuntime::with_defaults().unwrap();

    assert!(!runtime.is_shutting_down());
}

#[test]
fn AsyncRuntime___new___with_custom_config() {
    let config = RuntimeConfig::new().with_worker_threads(2);

    let runtime = AsyncRuntime::new(config).unwrap();

    assert_eq!(runtime.config().worker_threads, Some(2));
}

#[test]
fn AsyncRuntime___block_on___executes_future() {
    let runtime = AsyncRuntime::with_defaults().unwrap();

    let result = runtime.block_on(async { 42 });

    assert_eq!(result, 42);
}

#[test]
fn AsyncRuntime___spawn___executes_task() {
    let runtime = AsyncRuntime::with_defaults().unwrap();

    let handle = runtime.spawn(async { 123 });
    let result = runtime.block_on(handle).unwrap();

    assert_eq!(result, 123);
}

#[test]
fn AsyncRuntime___shutdown_signal___returns_signal() {
    let runtime = AsyncRuntime::with_defaults().unwrap();

    let signal = runtime.shutdown_signal();

    assert!(!signal.is_triggered());
}

#[test]
fn AsyncRuntime___shutdown___triggers_signal() {
    let runtime = AsyncRuntime::with_defaults().unwrap();
    let signal = runtime.shutdown_signal();

    runtime.shutdown(std::time::Duration::from_millis(10)).unwrap();

    assert!(signal.is_triggered());
}

#[test]
fn AsyncRuntime___is_shutting_down___initially_false() {
    let runtime = AsyncRuntime::with_defaults().unwrap();

    assert!(!runtime.is_shutting_down());
}

// RuntimeHolder tests

#[test]
fn RuntimeHolder___new___creates_uninitialized_holder() {
    let holder = RuntimeHolder::new();

    let result = holder.with(|_| ());

    assert!(result.is_err());
}

#[test]
fn RuntimeHolder___init___initializes_runtime() {
    let holder = RuntimeHolder::new();

    holder.init(RuntimeConfig::default()).unwrap();
    let result = holder.with(|rt| rt.block_on(async { 42 }));

    assert_eq!(result.unwrap(), 42);
}

#[test]
fn RuntimeHolder___init___twice_returns_error() {
    let holder = RuntimeHolder::new();

    holder.init(RuntimeConfig::default()).unwrap();
    let result = holder.init(RuntimeConfig::default());

    assert!(result.is_err());
}

#[test]
fn RuntimeHolder___shutdown___removes_runtime() {
    let holder = RuntimeHolder::new();
    holder.init(RuntimeConfig::default()).unwrap();

    holder.shutdown(std::time::Duration::from_millis(10)).unwrap();

    assert!(holder.with(|_| ()).is_err());
}

#[test]
fn RuntimeHolder___default___creates_uninitialized_holder() {
    let holder = RuntimeHolder::default();

    assert!(holder.with(|_| ()).is_err());
}
