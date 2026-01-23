#![allow(non_snake_case)]

use super::*;

// ============================================================================
// RbString Tests
// ============================================================================

#[test]
fn RbString___none___creates_null_string() {
    let s = RbString::none();

    assert!(!s.is_present());
    assert!(s.is_empty());
    assert!(s.data.is_null());
    assert_eq!(s.len, 0);
}

#[test]
fn RbString___from_static___creates_valid_reference() {
    let s = RbString::from_static("hello\0");

    assert!(s.is_present());
    assert!(!s.is_empty());
    assert_eq!(s.len, 6); // includes \0 in literal
}

#[test]
fn RbString___as_str___returns_string_slice() {
    let data = b"hello\0";
    let s = RbString {
        len: 5,
        data: data.as_ptr(),
    };

    let result = unsafe { s.as_str() };

    assert_eq!(result, Some("hello"));
}

#[test]
fn RbString___as_str___returns_none_for_null() {
    let s = RbString::none();

    let result = unsafe { s.as_str() };

    assert!(result.is_none());
}

#[test]
fn RbString___to_string___copies_data() {
    let data = b"test string\0";
    let s = RbString {
        len: 11,
        data: data.as_ptr(),
    };

    let result = unsafe { s.to_string() };

    assert_eq!(result, Some("test string".to_string()));
}

#[test]
fn RbString___default___is_none() {
    let s = RbString::default();

    assert!(!s.is_present());
}

// ============================================================================
// RbBytes Tests
// ============================================================================

#[test]
fn RbBytes___none___creates_null_bytes() {
    let b = RbBytes::none();

    assert!(!b.is_present());
    assert!(b.is_empty());
    assert!(b.data.is_null());
    assert_eq!(b.len, 0);
}

#[test]
fn RbBytes___from_static___creates_valid_reference() {
    let b = RbBytes::from_static(&[1, 2, 3, 4, 5]);

    assert!(b.is_present());
    assert!(!b.is_empty());
    assert_eq!(b.len, 5);
}

#[test]
fn RbBytes___as_slice___returns_byte_slice() {
    let data = [0xDE, 0xAD, 0xBE, 0xEF];
    let b = RbBytes {
        len: 4,
        data: data.as_ptr(),
    };

    let result = unsafe { b.as_slice() };

    assert_eq!(result, Some(&[0xDE, 0xAD, 0xBE, 0xEF][..]));
}

#[test]
fn RbBytes___as_slice___returns_none_for_null() {
    let b = RbBytes::none();

    let result = unsafe { b.as_slice() };

    assert!(result.is_none());
}

#[test]
fn RbBytes___to_vec___copies_data() {
    let data = [1, 2, 3];
    let b = RbBytes {
        len: 3,
        data: data.as_ptr(),
    };

    let result = unsafe { b.to_vec() };

    assert_eq!(result, Some(vec![1, 2, 3]));
}

// ============================================================================
// RbStringOwned Tests
// ============================================================================

#[test]
fn RbStringOwned___empty___creates_null_string() {
    let s = RbStringOwned::empty();

    assert!(s.data.is_null());
    assert_eq!(s.len, 0);
    assert_eq!(s.capacity, 0);
}

#[test]
fn RbStringOwned___from_string___takes_ownership() {
    let original = String::from("hello world");
    let s = RbStringOwned::from_string(original);

    assert!(!s.data.is_null());
    assert_eq!(s.len, 11);
    assert!(s.capacity >= 12); // At least len + null terminator

    // Verify data is accessible
    let borrowed = s.as_borrowed();
    let str_val = unsafe { borrowed.as_str() };
    assert_eq!(str_val, Some("hello world"));

    // Clean up
    let mut s = s;
    unsafe { s.free() };
}

#[test]
fn RbStringOwned___from_slice___copies_data() {
    let s = RbStringOwned::from_slice("test");

    assert!(!s.data.is_null());
    assert_eq!(s.len, 4);

    let borrowed = s.as_borrowed();
    let str_val = unsafe { borrowed.as_str() };
    assert_eq!(str_val, Some("test"));

    let mut s = s;
    unsafe { s.free() };
}

#[test]
fn RbStringOwned___free___deallocates_memory() {
    let mut s = RbStringOwned::from_slice("to be freed");

    unsafe { s.free() };

    assert!(s.data.is_null());
    assert_eq!(s.len, 0);
    assert_eq!(s.capacity, 0);
}

#[test]
fn RbStringOwned___free___safe_to_call_on_empty() {
    let mut s = RbStringOwned::empty();

    unsafe { s.free() }; // Should not panic

    assert!(s.data.is_null());
}

// ============================================================================
// RbBytesOwned Tests
// ============================================================================

#[test]
fn RbBytesOwned___empty___creates_null_buffer() {
    let b = RbBytesOwned::empty();

    assert!(b.data.is_null());
    assert_eq!(b.len, 0);
    assert_eq!(b.capacity, 0);
}

#[test]
fn RbBytesOwned___from_vec___takes_ownership() {
    let original = vec![1, 2, 3, 4, 5];
    let b = RbBytesOwned::from_vec(original);

    assert!(!b.data.is_null());
    assert_eq!(b.len, 5);
    assert!(b.capacity >= 5);

    let borrowed = b.as_borrowed();
    let slice = unsafe { borrowed.as_slice() };
    assert_eq!(slice, Some(&[1, 2, 3, 4, 5][..]));

    let mut b = b;
    unsafe { b.free() };
}

#[test]
fn RbBytesOwned___from_slice___copies_data() {
    let b = RbBytesOwned::from_slice(&[0xCA, 0xFE]);

    assert!(!b.data.is_null());
    assert_eq!(b.len, 2);

    let borrowed = b.as_borrowed();
    let slice = unsafe { borrowed.as_slice() };
    assert_eq!(slice, Some(&[0xCA, 0xFE][..]));

    let mut b = b;
    unsafe { b.free() };
}

#[test]
fn RbBytesOwned___free___deallocates_memory() {
    let mut b = RbBytesOwned::from_slice(&[1, 2, 3]);

    unsafe { b.free() };

    assert!(b.data.is_null());
    assert_eq!(b.len, 0);
    assert_eq!(b.capacity, 0);
}

// ============================================================================
// RbResponse Tests
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
struct TestStruct {
    x: u32,
    y: u32,
}

#[test]
fn RbResponse___success___creates_valid_response() {
    let data = TestStruct { x: 42, y: 100 };
    let resp = RbResponse::success(data);

    assert!(!resp.is_error());
    assert_eq!(resp.error_code, 0);
    assert_eq!(resp.len as usize, std::mem::size_of::<TestStruct>());
    assert!(!resp.data.is_null());

    let retrieved = unsafe { resp.as_ref::<TestStruct>() };
    assert_eq!(retrieved, Some(&TestStruct { x: 42, y: 100 }));

    let mut resp = resp;
    unsafe { resp.free() };
}

#[test]
fn RbResponse___error___creates_error_response() {
    let resp = RbResponse::error(5, "Something went wrong");

    assert!(resp.is_error());
    assert_eq!(resp.error_code, 5);
    assert!(!resp.data.is_null());

    let mut resp = resp;
    unsafe { resp.free() };
}

#[test]
fn RbResponse___empty___creates_null_response() {
    let resp = RbResponse::empty();

    assert!(!resp.is_error());
    assert!(resp.data.is_null());
    assert_eq!(resp.len, 0);
}

#[test]
fn RbResponse___as_ref___returns_none_for_error() {
    let resp = RbResponse::error(1, "error");

    let result = unsafe { resp.as_ref::<TestStruct>() };

    assert!(result.is_none());

    let mut resp = resp;
    unsafe { resp.free() };
}

#[test]
fn RbResponse___free___safe_to_call_on_empty() {
    let mut resp = RbResponse::empty();

    unsafe { resp.free() }; // Should not panic

    assert!(resp.data.is_null());
}

// ============================================================================
// Memory Layout Tests
// ============================================================================

#[test]
fn memory_layout___RbString___has_expected_size() {
    // On 64-bit: u32 (4) + padding (4) + ptr (8) = 16 bytes
    // On 32-bit: u32 (4) + ptr (4) = 8 bytes
    let size = std::mem::size_of::<RbString>();
    assert!(
        size == 8 || size == 16,
        "Unexpected RbString size: {}",
        size
    );
}

#[test]
fn memory_layout___RbBytes___has_expected_size() {
    let size = std::mem::size_of::<RbBytes>();
    assert!(size == 8 || size == 16, "Unexpected RbBytes size: {}", size);
}

#[test]
fn memory_layout___RbStringOwned___has_expected_size() {
    // On 64-bit: u32 (4) + ptr (8) + u32 (4) = variable with padding
    let size = std::mem::size_of::<RbStringOwned>();
    assert!(size >= 12, "RbStringOwned too small: {}", size);
}

#[test]
fn memory_layout___RbResponse___has_expected_size() {
    // u32 + u32 + u32 + ptr = variable with alignment
    let size = std::mem::size_of::<RbResponse>();
    assert!(size >= 16, "RbResponse too small: {}", size);
}
