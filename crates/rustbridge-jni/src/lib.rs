//! rustbridge-jni - JNI bindings for rustbridge
//!
//! This crate provides JNI (Java Native Interface) bindings for rustbridge,
//! enabling Java 17+ applications to load and interact with rustbridge plugins.
//!
//! # Architecture
//!
//! The JNI bridge works by:
//! 1. Loading the target plugin library dynamically
//! 2. Calling `plugin_create` to instantiate the plugin
//! 3. Calling `plugin_init` to initialize it with config
//! 4. Forwarding all calls through the FFI layer
//!
//! # Java Classes
//!
//! This library implements native methods for:
//! - `com.rustbridge.jni.JniPluginLoader` - Plugin loading
//! - `com.rustbridge.jni.JniPlugin` - Plugin operations
//!
//! # Important Note
//!
//! This crate does NOT link against rustbridge-ffi directly. Instead, it defines
//! minimal FFI types locally and loads all symbols dynamically from the plugin.
//! This avoids symbol resolution conflicts between the JNI bridge and the plugin,
//! which is critical for features like concurrency limiting to work correctly.

mod error;
mod ffi_types;
mod loader;

use error::JniError;
use jni::JNIEnv;
use jni::objects::{JByteArray, JClass, JString};
use jni::sys::{JNI_FALSE, JNI_TRUE, jboolean, jint, jlong};
use loader::LoadedPlugin;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// Global registry of loaded plugins
// Maps handle ID to Arc<LoadedPlugin> (which keeps the library loaded)
// Using Arc allows us to clone references without holding the mutex during calls
static LOADED_PLUGINS: Mutex<Option<HashMap<u64, Arc<LoadedPlugin>>>> = Mutex::new(None);

fn get_plugins() -> std::sync::MutexGuard<'static, Option<HashMap<u64, Arc<LoadedPlugin>>>> {
    LOADED_PLUGINS.lock().unwrap()
}

fn init_plugins() {
    let mut guard = get_plugins();
    if guard.is_none() {
        *guard = Some(HashMap::new());
    }
}

fn register_plugin(handle: u64, plugin: LoadedPlugin) {
    init_plugins();
    let mut guard = get_plugins();
    if let Some(ref mut map) = *guard {
        map.insert(handle, Arc::new(plugin));
    }
}

fn remove_plugin(handle: u64) -> Option<Arc<LoadedPlugin>> {
    let mut guard = get_plugins();
    guard.as_mut().and_then(|map| map.remove(&handle))
}

/// Get a plugin reference (Arc-cloned to avoid holding the mutex during the call)
fn get_plugin(handle: u64) -> Option<Arc<LoadedPlugin>> {
    let guard = get_plugins();
    guard
        .as_ref()
        .and_then(|map| map.get(&handle).map(Arc::clone))
}

fn with_plugin<F, T>(handle: u64, f: F) -> Option<T>
where
    F: FnOnce(&LoadedPlugin) -> T,
{
    // Clone the Arc BEFORE releasing the mutex to allow concurrent calls
    let plugin = get_plugin(handle)?;
    // Mutex is released here, allowing other threads to proceed
    Some(f(&plugin))
}

// ============================================================================
// JniPluginLoader native methods
// ============================================================================

/// Load a plugin from a library path.
///
/// # Parameters
/// - `library_path`: Path to the plugin shared library
/// - `config_json`: JSON configuration bytes (can be null)
///
/// # Returns
/// Handle to the loaded plugin (as jlong), or throws PluginException on failure
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustbridge_jni_JniPluginLoader_nativeLoadPlugin<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    library_path: JString<'local>,
    config_json: JByteArray<'local>,
) -> jlong {
    match load_plugin_impl(&mut env, library_path, config_json) {
        Ok(handle) => handle as jlong,
        Err(e) => {
            throw_plugin_exception(&mut env, e.code(), &e.to_string());
            0
        }
    }
}

fn load_plugin_impl(
    env: &mut JNIEnv,
    library_path: JString,
    config_json: JByteArray,
) -> Result<u64, JniError> {
    // Get library path as Rust string
    let path: String = env
        .get_string(&library_path)
        .map_err(|e| JniError::StringConversion(e.to_string()))?
        .into();

    // Get config bytes (if provided)
    let config_bytes: Option<Vec<u8>> = if config_json.is_null() {
        None
    } else {
        let len = env
            .get_array_length(&config_json)
            .map_err(|e| JniError::ArrayAccess(e.to_string()))?;
        if len == 0 {
            None
        } else {
            let mut bytes = vec![0u8; len as usize];
            env.get_byte_array_region(&config_json, 0, bytemuck_cast_slice_mut(&mut bytes))
                .map_err(|e| JniError::ArrayAccess(e.to_string()))?;
            Some(bytes)
        }
    };

    // Load the plugin
    let loaded = loader::load_plugin(&path, config_bytes.as_deref())?;
    let handle = loaded.handle();

    // Register for cleanup
    register_plugin(handle, loaded);

    Ok(handle)
}

// ============================================================================
// JniPlugin native methods
// ============================================================================

/// Get the current state of a plugin.
///
/// # Returns
/// State code (0=Installed, 1=Starting, 2=Active, 3=Stopping, 4=Stopped, 5=Failed)
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustbridge_jni_JniPlugin_nativeGetState<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jint {
    with_plugin(handle as u64, |plugin| plugin.get_state() as jint).unwrap_or(255)
}

/// Make a synchronous call to the plugin.
///
/// # Parameters
/// - `handle`: Plugin handle
/// - `type_tag`: Message type identifier
/// - `request`: JSON request string
///
/// # Returns
/// JSON response string, or throws PluginException on failure
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustbridge_jni_JniPlugin_nativeCall<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    type_tag: JString<'local>,
    request: JString<'local>,
) -> JString<'local> {
    match call_impl(&mut env, handle, type_tag, request) {
        Ok(response) => response,
        Err(e) => {
            throw_plugin_exception(&mut env, e.code(), &e.to_string());
            JString::default()
        }
    }
}

fn call_impl<'local>(
    env: &mut JNIEnv<'local>,
    handle: jlong,
    type_tag: JString<'local>,
    request: JString<'local>,
) -> Result<JString<'local>, JniError> {
    // Get type tag as C string
    let type_tag_str: String = env
        .get_string(&type_tag)
        .map_err(|e| JniError::StringConversion(e.to_string()))?
        .into();

    // Get request as bytes
    let request_str: String = env
        .get_string(&request)
        .map_err(|e| JniError::StringConversion(e.to_string()))?
        .into();
    let request_bytes = request_str.as_bytes();

    // Create C string for type tag
    let type_tag_cstr = std::ffi::CString::new(type_tag_str)
        .map_err(|e| JniError::StringConversion(e.to_string()))?;

    // Get the plugin from registry and make the call
    let buffer = with_plugin(handle as u64, |plugin| {
        plugin.call(
            type_tag_cstr.as_ptr(),
            request_bytes.as_ptr(),
            request_bytes.len(),
        )
    })
    .ok_or_else(|| JniError::PluginCall {
        code: 1,
        message: "Invalid plugin handle".to_string(),
    })?;

    // Check for errors
    if buffer.is_error() {
        let error_code = buffer.error_code;
        // SAFETY: buffer contains valid data from plugin_call
        let error_msg = unsafe {
            let slice = buffer.as_slice();
            std::str::from_utf8(slice).unwrap_or("Unknown error")
        };
        let err = JniError::PluginCall {
            code: error_code,
            message: error_msg.to_string(),
        };

        // Free the buffer through the plugin
        // SAFETY: buffer is a valid FfiBuffer from plugin_call
        unsafe {
            let mut buf = buffer;
            buf.free();
        }

        return Err(err);
    }

    // Parse the response envelope to extract the data
    // SAFETY: buffer contains valid data from plugin_call
    let response_bytes = unsafe { buffer.as_slice() };
    let response_str = extract_response_data(response_bytes)?;

    // Free the buffer
    // SAFETY: buffer is a valid FfiBuffer from plugin_call
    unsafe {
        let mut buf = buffer;
        buf.free();
    }

    // Create Java string from response
    env.new_string(&response_str)
        .map_err(|e| JniError::StringConversion(e.to_string()))
}

/// Extract the payload from a response envelope JSON.
///
/// ResponseEnvelope format:
/// - `status`: "success" or "error"
/// - `payload`: The response data (on success)
/// - `error_code`: Error code (on failure)
/// - `error_message`: Error message (on failure)
fn extract_response_data(bytes: &[u8]) -> Result<String, JniError> {
    // Parse the response envelope
    let envelope: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|e| JniError::JsonParse(e.to_string()))?;

    // Check status field
    let status = envelope
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("error");

    if status == "error" {
        let code = envelope
            .get("error_code")
            .and_then(|c| c.as_u64())
            .unwrap_or(11) as u32;
        let message = envelope
            .get("error_message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error")
            .to_string();
        return Err(JniError::PluginCall { code, message });
    }

    // Extract the payload field
    let payload = envelope
        .get("payload")
        .ok_or_else(|| JniError::JsonParse("Missing 'payload' field in response".to_string()))?;

    // Return the payload as a JSON string
    serde_json::to_string(payload).map_err(|e| JniError::JsonParse(e.to_string()))
}

/// Check if binary transport is supported.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustbridge_jni_JniPlugin_nativeHasBinaryTransport<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jboolean {
    let result =
        with_plugin(handle as u64, |plugin| plugin.has_binary_transport()).unwrap_or(false);
    if result { JNI_TRUE } else { JNI_FALSE }
}

/// Make a raw binary call to the plugin.
///
/// # Parameters
/// - `handle`: Plugin handle
/// - `message_id`: Binary message ID
/// - `request`: Request bytes
///
/// # Returns
/// Response bytes, or throws PluginException on failure
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustbridge_jni_JniPlugin_nativeCallRaw<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    message_id: jint,
    request: JByteArray<'local>,
) -> JByteArray<'local> {
    match call_raw_impl(&mut env, handle, message_id, request) {
        Ok(response) => response,
        Err(e) => {
            throw_plugin_exception(&mut env, e.code(), &e.to_string());
            JByteArray::default()
        }
    }
}

fn call_raw_impl<'local>(
    env: &mut JNIEnv<'local>,
    handle: jlong,
    message_id: jint,
    request: JByteArray<'local>,
) -> Result<JByteArray<'local>, JniError> {
    // Get request bytes
    let request_len = env
        .get_array_length(&request)
        .map_err(|e| JniError::ArrayAccess(e.to_string()))?;

    let mut request_bytes = vec![0u8; request_len as usize];
    env.get_byte_array_region(&request, 0, bytemuck_cast_slice_mut(&mut request_bytes))
        .map_err(|e| JniError::ArrayAccess(e.to_string()))?;

    // Make the raw call
    let response = with_plugin(handle as u64, |plugin| {
        plugin.call_raw(
            message_id as u32,
            request_bytes.as_ptr(),
            request_bytes.len(),
        )
    })
    .ok_or_else(|| JniError::PluginCall {
        code: 1,
        message: "Invalid plugin handle".to_string(),
    })?
    .ok_or_else(|| JniError::PluginCall {
        code: 6,
        message: "Binary transport not supported by this plugin".to_string(),
    })?;

    // Check for errors
    if response.is_error() {
        let error_code = response.error_code;
        // SAFETY: response contains valid data from plugin_call_raw
        let error_msg = unsafe {
            let slice = response.as_slice();
            std::str::from_utf8(slice).unwrap_or("Unknown error")
        };
        let err = JniError::PluginCall {
            code: error_code,
            message: error_msg.to_string(),
        };

        // Free the response
        // SAFETY: response is a valid RbResponse from plugin_call_raw
        unsafe {
            let mut resp = response;
            resp.free();
        }

        return Err(err);
    }

    // Get response bytes
    // SAFETY: response contains valid data from plugin_call_raw
    let response_bytes = unsafe { response.as_slice() };

    // Create Java byte array
    let result = env
        .new_byte_array(response_bytes.len() as i32)
        .map_err(|e| JniError::ArrayAccess(e.to_string()))?;

    env.set_byte_array_region(&result, 0, bytemuck_cast_slice(response_bytes))
        .map_err(|e| JniError::ArrayAccess(e.to_string()))?;

    // Free the response
    // SAFETY: response is a valid RbResponse from plugin_call_raw
    unsafe {
        let mut resp = response;
        resp.free();
    }

    Ok(result)
}

/// Cast a &[u8] to &[i8] for JNI byte array operations.
fn bytemuck_cast_slice(bytes: &[u8]) -> &[i8] {
    // SAFETY: u8 and i8 have the same size and alignment
    unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const i8, bytes.len()) }
}

/// Set the log level for a plugin.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustbridge_jni_JniPlugin_nativeSetLogLevel<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
    level: jint,
) {
    with_plugin(handle as u64, |plugin| plugin.set_log_level(level as u8));
}

/// Get the number of rejected requests due to concurrency limits.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustbridge_jni_JniPlugin_nativeGetRejectedCount<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jlong {
    with_plugin(handle as u64, |plugin| plugin.get_rejected_count() as jlong).unwrap_or(0)
}

/// Shutdown a plugin instance.
///
/// # Returns
/// true on success, false on failure
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_rustbridge_jni_JniPlugin_nativeShutdown<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jboolean {
    // Remove from our registry and shutdown
    let result = if let Some(plugin) = remove_plugin(handle as u64) {
        plugin.shutdown()
    } else {
        false
    };

    if result { JNI_TRUE } else { JNI_FALSE }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Throw a PluginException in Java with the proper error code.
fn throw_plugin_exception(env: &mut JNIEnv, code: u32, message: &str) {
    let class_name = "com/rustbridge/PluginException";

    // Try to find the exception class
    let class = match env.find_class(class_name) {
        Ok(c) => c,
        Err(_) => {
            // Fallback to RuntimeException if PluginException not found
            let _ = env.throw_new("java/lang/RuntimeException", message);
            return;
        }
    };

    // Create the exception message as a Java string
    let jmessage = match env.new_string(message) {
        Ok(s) => s,
        Err(_) => {
            let _ = env.throw_new("java/lang/RuntimeException", message);
            return;
        }
    };

    // Create the exception using the constructor that takes (int, String)
    let exception = match env.new_object(
        &class,
        "(ILjava/lang/String;)V",
        &[
            jni::objects::JValue::Int(code as i32),
            jni::objects::JValue::Object(&jmessage),
        ],
    ) {
        Ok(e) => e,
        Err(_) => {
            // Fallback to message-only constructor
            let _ = env.throw_new(class, message);
            return;
        }
    };

    // Throw the exception
    use jni::objects::JThrowable;
    let throwable = unsafe { JThrowable::from_raw(exception.into_raw()) };
    let _ = env.throw(throwable);
}

/// Cast a &mut [u8] to &mut [i8] for JNI byte array operations.
fn bytemuck_cast_slice_mut(bytes: &mut [u8]) -> &mut [i8] {
    // SAFETY: u8 and i8 have the same size and alignment
    unsafe { std::slice::from_raw_parts_mut(bytes.as_mut_ptr() as *mut i8, bytes.len()) }
}
