#![allow(non_snake_case)]

use super::*;

// Note: Full integration tests require a plugin implementation
// These tests verify the FFI surface

#[test]
fn plugin_call___null_handle___returns_error() {
    unsafe {
        let result = plugin_call(ptr::null_mut(), c"test".as_ptr(), ptr::null(), 0);

        assert!(result.is_error());

        let mut result = result;
        result.free();
    }
}

#[test]
fn plugin_call___invalid_handle___returns_error() {
    unsafe {
        let result = plugin_call(999 as FfiPluginHandle, c"test".as_ptr(), ptr::null(), 0);

        assert!(result.is_error());

        let mut result = result;
        result.free();
    }
}

#[test]
fn plugin_call___null_type_tag___returns_error() {
    unsafe {
        let result = plugin_call(1 as FfiPluginHandle, ptr::null(), ptr::null(), 0);

        assert!(result.is_error());

        let mut result = result;
        result.free();
    }
}

#[test]
fn plugin_free_buffer___null_pointer___does_not_crash() {
    unsafe {
        plugin_free_buffer(ptr::null_mut());
    }
}

#[test]
fn plugin_get_state___invalid_handle___returns_255() {
    unsafe {
        let state = plugin_get_state(999 as FfiPluginHandle);

        assert_eq!(state, 255);
    }
}

#[test]
fn plugin_shutdown___invalid_handle___returns_false() {
    unsafe {
        let result = plugin_shutdown(999 as FfiPluginHandle);

        assert!(!result);
    }
}
