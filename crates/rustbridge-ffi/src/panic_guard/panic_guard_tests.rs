#![allow(non_snake_case)]

use super::*;

#[test]
fn catch_panic___successful_function___returns_ok() {
    let result = catch_panic(0, || 42);

    assert!(result.is_ok());
    match result {
        Ok(value) => assert_eq!(value, 42),
        Err(_) => panic!("Expected Ok, got Err"),
    }
}

#[test]
fn catch_panic___panicking_function___returns_err() {
    let result: Result<(), FfiBuffer> = catch_panic(0, || {
        panic!("Test panic");
    });

    assert!(result.is_err());
    match result {
        Err(error_buffer) => {
            assert_eq!(error_buffer.error_code, 11); // InternalError
            assert!(error_buffer.is_error());
        }
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}

#[test]
fn catch_panic___panic_with_string___includes_message() {
    let result: Result<(), FfiBuffer> = catch_panic(0, || {
        panic!("Custom panic message");
    });

    assert!(result.is_err());
    match result {
        Err(mut error_buffer) => {
            // Convert buffer data to string to check message
            let error_msg = unsafe {
                let slice = std::slice::from_raw_parts(error_buffer.data, error_buffer.len);
                String::from_utf8_lossy(slice).to_string()
            };

            assert!(error_msg.contains("Custom panic message"));

            // Clean up
            unsafe { error_buffer.free() };
        }
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}

#[test]
fn catch_panic___panic_with_str___includes_message() {
    let result: Result<(), FfiBuffer> = catch_panic(0, || {
        panic!("str panic");
    });

    assert!(result.is_err());
    match result {
        Err(mut error_buffer) => {
            let error_msg = unsafe {
                let slice = std::slice::from_raw_parts(error_buffer.data, error_buffer.len);
                String::from_utf8_lossy(slice).to_string()
            };

            assert!(error_msg.contains("str panic"));

            unsafe { error_buffer.free() };
        }
        Ok(_) => panic!("Expected Err, got Ok"),
    }
}

#[test]
fn panic_to_string___str_payload___formats_correctly() {
    let panic_payload: Box<dyn std::any::Any + Send> = Box::new("test panic");
    let result = panic_to_string(&panic_payload);

    assert_eq!(result, "Plugin panicked: test panic");
}

#[test]
fn panic_to_string___string_payload___formats_correctly() {
    let panic_payload: Box<dyn std::any::Any + Send> = Box::new("owned string".to_string());
    let result = panic_to_string(&panic_payload);

    assert_eq!(result, "Plugin panicked: owned string");
}

#[test]
fn panic_to_string___unknown_payload___returns_fallback() {
    let panic_payload: Box<dyn std::any::Any + Send> = Box::new(42);
    let result = panic_to_string(&panic_payload);

    assert_eq!(result, "Plugin panicked with unknown payload");
}

#[test]
fn install_panic_hook___can_be_called_multiple_times() {
    // Should not panic when called multiple times
    install_panic_hook();
    install_panic_hook();
    install_panic_hook();
}

// Note: Testing that the panic hook actually logs requires integration tests
// with a mock logging callback, which would be in a separate test file
