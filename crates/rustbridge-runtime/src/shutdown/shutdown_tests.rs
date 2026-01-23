#![allow(non_snake_case)]

use super::*;

// ShutdownHandle tests

#[test]
fn ShutdownHandle___new___not_triggered() {
    let handle = ShutdownHandle::new();

    assert!(!handle.is_triggered());
}

#[test]
fn ShutdownHandle___trigger___sets_triggered() {
    let handle = ShutdownHandle::new();

    handle.trigger();

    assert!(handle.is_triggered());
}

#[test]
fn ShutdownHandle___trigger___idempotent() {
    let handle = ShutdownHandle::new();

    handle.trigger();
    handle.trigger();
    handle.trigger();

    assert!(handle.is_triggered());
}

#[test]
fn ShutdownHandle___signal___returns_connected_signal() {
    let handle = ShutdownHandle::new();

    let signal = handle.signal();

    assert!(!signal.is_triggered());
    handle.trigger();
    assert!(signal.is_triggered());
}

#[test]
fn ShutdownHandle___default___creates_untriggered_handle() {
    let handle = ShutdownHandle::default();

    assert!(!handle.is_triggered());
}

// ShutdownSignal tests

#[test]
fn ShutdownSignal___is_triggered___reflects_handle_state() {
    let handle = ShutdownHandle::new();
    let signal = handle.signal();

    assert!(!signal.is_triggered());

    handle.trigger();

    assert!(signal.is_triggered());
}

#[test]
fn ShutdownSignal___clone___shares_triggered_state() {
    let handle = ShutdownHandle::new();
    let signal1 = handle.signal();
    let signal2 = signal1.clone();

    assert!(!signal1.is_triggered());
    assert!(!signal2.is_triggered());

    handle.trigger();

    assert!(signal1.is_triggered());
    assert!(signal2.is_triggered());
}

#[tokio::test]
async fn ShutdownSignal___wait___blocks_until_triggered() {
    let handle = ShutdownHandle::new();
    let mut signal = handle.signal();

    let handle_clone = handle.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        handle_clone.trigger();
    });

    signal.wait().await;

    assert!(signal.is_triggered());
}

#[tokio::test]
async fn ShutdownSignal___wait___returns_immediately_if_already_triggered() {
    let handle = ShutdownHandle::new();
    handle.trigger();
    let mut signal = handle.signal();

    tokio::time::timeout(std::time::Duration::from_millis(10), signal.wait())
        .await
        .expect("Should not timeout");
}

#[tokio::test]
async fn ShutdownSignal___notified___completes_on_trigger() {
    let handle = ShutdownHandle::new();
    let mut signal = handle.signal();

    let handle_clone = handle.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        handle_clone.trigger();
    });

    signal.notified().await;

    assert!(signal.is_triggered());
}
