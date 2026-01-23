#![allow(non_snake_case)]

use super::*;

// FfiBuffer tests

#[test]
fn FfiBuffer___empty___creates_null_buffer() {
    let buf = FfiBuffer::empty();

    assert!(buf.data.is_null());
    assert_eq!(buf.len, 0);
    assert_eq!(buf.capacity, 0);
    assert_eq!(buf.error_code, 0);
}

#[test]
fn FfiBuffer___empty___is_empty_returns_true() {
    let buf = FfiBuffer::empty();

    assert!(buf.is_empty());
}

#[test]
fn FfiBuffer___empty___is_error_returns_false() {
    let buf = FfiBuffer::empty();

    assert!(!buf.is_error());
}

#[test]
fn FfiBuffer___from_vec___transfers_ownership() {
    let data = vec![1u8, 2, 3, 4, 5];

    let mut buf = FfiBuffer::from_vec(data);

    assert!(!buf.data.is_null());
    assert_eq!(buf.len, 5);
    assert!(buf.capacity >= 5);

    unsafe { buf.free() };
}

#[test]
fn FfiBuffer___from_vec___preserves_data() {
    let data = vec![1u8, 2, 3, 4, 5];

    let mut buf = FfiBuffer::from_vec(data);

    unsafe {
        let slice = buf.as_slice();
        assert_eq!(slice, &[1, 2, 3, 4, 5]);
        buf.free();
    }
}

#[test]
fn FfiBuffer___from_vec___is_empty_returns_false() {
    let mut buf = FfiBuffer::from_vec(vec![1, 2, 3]);

    assert!(!buf.is_empty());

    unsafe { buf.free() };
}

#[test]
fn FfiBuffer___error___sets_error_code() {
    let mut buf = FfiBuffer::error(404, "Not found");

    assert!(buf.is_error());
    assert_eq!(buf.error_code, 404);

    unsafe { buf.free() };
}

#[test]
fn FfiBuffer___error___contains_message() {
    let mut buf = FfiBuffer::error(500, "Server error");

    unsafe {
        let slice = buf.as_slice();
        assert_eq!(std::str::from_utf8(slice).unwrap(), "Server error");
        buf.free();
    }
}

#[test]
fn FfiBuffer___error___is_not_empty() {
    let mut buf = FfiBuffer::error(404, "Not found");

    assert!(!buf.is_empty());

    unsafe { buf.free() };
}

#[test]
fn FfiBuffer___success_json___serializes_value() {
    #[derive(serde::Serialize)]
    struct TestData {
        value: i32,
    }

    let mut buf = FfiBuffer::success_json(&TestData { value: 42 });

    assert!(!buf.is_error());
    unsafe {
        let slice = buf.as_slice();
        let s = std::str::from_utf8(slice).unwrap();
        assert!(s.contains("42"));
        buf.free();
    }
}

#[test]
fn FfiBuffer___free___clears_pointer() {
    let mut buf = FfiBuffer::from_vec(vec![1, 2, 3]);

    unsafe { buf.free() };

    assert!(buf.data.is_null());
    assert_eq!(buf.len, 0);
    assert_eq!(buf.capacity, 0);
}

#[test]
fn FfiBuffer___free___double_free_is_safe() {
    let mut buf = FfiBuffer::from_vec(vec![1, 2, 3]);

    unsafe {
        buf.free();
        buf.free();
    }

    assert!(buf.data.is_null());
}

#[test]
fn FfiBuffer___default___same_as_empty() {
    let buf = FfiBuffer::default();

    assert!(buf.data.is_null());
    assert_eq!(buf.len, 0);
    assert!(buf.is_empty());
}

#[test]
fn FfiBuffer___as_slice___empty_buffer_returns_empty_slice() {
    let buf = FfiBuffer::empty();

    unsafe {
        let slice = buf.as_slice();
        assert!(slice.is_empty());
    }
}
