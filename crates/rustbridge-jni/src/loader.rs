//! Dynamic library loading for plugins.

use crate::error::JniError;
use crate::ffi_types::{FfiBuffer, LogCallback};
use libloading::{Library, Symbol};
use std::ffi::{c_char, c_void};

/// A loaded plugin, keeping the library alive and providing access to its FFI functions.
pub struct LoadedPlugin {
    /// The loaded library (must be kept alive while plugin is in use).
    _library: Library,

    /// The FFI handle returned by plugin_init.
    handle: u64,

    /// Function pointers to the plugin's FFI functions.
    /// These must be called on the loaded library to use its PluginHandleManager.
    ffi: PluginFfi,
}

/// Function pointers to plugin FFI functions.
struct PluginFfi {
    call: PluginCallFn,
    get_state: PluginGetStateFn,
    set_log_level: PluginSetLogLevelFn,
    get_rejected_count: PluginGetRejectedCountFn,
    shutdown: PluginShutdownFn,
}

impl LoadedPlugin {
    /// Get the FFI handle for this plugin.
    pub fn handle(&self) -> u64 {
        self.handle
    }

    /// Call the plugin.
    pub fn call(
        &self,
        type_tag: *const c_char,
        request: *const u8,
        request_len: usize,
    ) -> FfiBuffer {
        // SAFETY: handle is valid, type_tag is a valid C string, request is valid
        unsafe { (self.ffi.call)(self.handle as *mut c_void, type_tag, request, request_len) }
    }

    /// Get the plugin state.
    pub fn get_state(&self) -> u8 {
        // SAFETY: handle is valid
        unsafe { (self.ffi.get_state)(self.handle as *mut c_void) }
    }

    /// Set the log level.
    pub fn set_log_level(&self, level: u8) {
        // SAFETY: handle is valid
        unsafe { (self.ffi.set_log_level)(self.handle as *mut c_void, level) }
    }

    /// Get the rejected request count.
    pub fn get_rejected_count(&self) -> u64 {
        // SAFETY: handle is valid
        unsafe { (self.ffi.get_rejected_count)(self.handle as *mut c_void) }
    }

    /// Shutdown the plugin.
    pub fn shutdown(&self) -> bool {
        // SAFETY: handle is valid
        unsafe { (self.ffi.shutdown)(self.handle as *mut c_void) }
    }
}

// Type signatures for FFI functions
type PluginCreateFn = unsafe extern "C" fn() -> *mut c_void;
type PluginInitFn = unsafe extern "C" fn(
    plugin_ptr: *mut c_void,
    config_json: *const u8,
    config_len: usize,
    log_callback: Option<LogCallback>,
) -> *mut c_void;
type PluginCallFn = unsafe extern "C" fn(
    handle: *mut c_void,
    type_tag: *const c_char,
    request: *const u8,
    request_len: usize,
) -> FfiBuffer;
type PluginGetStateFn = unsafe extern "C" fn(handle: *mut c_void) -> u8;
type PluginSetLogLevelFn = unsafe extern "C" fn(handle: *mut c_void, level: u8);
type PluginGetRejectedCountFn = unsafe extern "C" fn(handle: *mut c_void) -> u64;
type PluginShutdownFn = unsafe extern "C" fn(handle: *mut c_void) -> bool;

/// Load a plugin from a shared library.
///
/// # Parameters
/// - `library_path`: Path to the plugin shared library
/// - `config_json`: Optional JSON configuration bytes
///
/// # Returns
/// A LoadedPlugin containing the FFI handle and function pointers
pub fn load_plugin(
    library_path: &str,
    config_json: Option<&[u8]>,
) -> Result<LoadedPlugin, JniError> {
    // Load the library
    // SAFETY: We're loading a shared library. The caller is responsible for
    // ensuring the library path is valid and trusted.
    let library = unsafe { Library::new(library_path) }
        .map_err(|e| JniError::LibraryLoad(format!("{}: {}", library_path, e)))?;

    // Get all required symbols
    // SAFETY: We're getting function pointers from the loaded library.
    let create_fn: Symbol<PluginCreateFn> = unsafe { library.get(b"plugin_create\0") }
        .map_err(|e| JniError::SymbolNotFound(format!("plugin_create: {}", e)))?;

    let init_fn: Symbol<PluginInitFn> = unsafe { library.get(b"plugin_init\0") }
        .map_err(|e| JniError::SymbolNotFound(format!("plugin_init: {}", e)))?;

    let call_fn: Symbol<PluginCallFn> = unsafe { library.get(b"plugin_call\0") }
        .map_err(|e| JniError::SymbolNotFound(format!("plugin_call: {}", e)))?;

    let get_state_fn: Symbol<PluginGetStateFn> = unsafe { library.get(b"plugin_get_state\0") }
        .map_err(|e| JniError::SymbolNotFound(format!("plugin_get_state: {}", e)))?;

    let set_log_level_fn: Symbol<PluginSetLogLevelFn> =
        unsafe { library.get(b"plugin_set_log_level\0") }
            .map_err(|e| JniError::SymbolNotFound(format!("plugin_set_log_level: {}", e)))?;

    let get_rejected_count_fn: Symbol<PluginGetRejectedCountFn> =
        unsafe { library.get(b"plugin_get_rejected_count\0") }
            .map_err(|e| JniError::SymbolNotFound(format!("plugin_get_rejected_count: {}", e)))?;

    let shutdown_fn: Symbol<PluginShutdownFn> = unsafe { library.get(b"plugin_shutdown\0") }
        .map_err(|e| JniError::SymbolNotFound(format!("plugin_shutdown: {}", e)))?;

    // Store function pointers (they must outlive the library)
    // SAFETY: These function pointers are valid as long as the library is loaded
    let ffi = PluginFfi {
        call: *call_fn,
        get_state: *get_state_fn,
        set_log_level: *set_log_level_fn,
        get_rejected_count: *get_rejected_count_fn,
        shutdown: *shutdown_fn,
    };

    // Create the plugin instance
    // SAFETY: plugin_create returns a valid plugin pointer or null
    let plugin_ptr = unsafe { create_fn() };
    if plugin_ptr.is_null() {
        return Err(JniError::InitFailed(
            "plugin_create returned null".to_string(),
        ));
    }

    // Initialize the plugin
    let (config_ptr, config_len) = match config_json {
        Some(bytes) => (bytes.as_ptr(), bytes.len()),
        None => (std::ptr::null(), 0),
    };

    // SAFETY: plugin_ptr is valid, config_ptr is valid for config_len bytes
    let handle_ptr = unsafe { init_fn(plugin_ptr, config_ptr, config_len, None) };

    if handle_ptr.is_null() {
        return Err(JniError::InitFailed(
            "plugin_init returned null (check plugin logs for details)".to_string(),
        ));
    }

    let handle = handle_ptr as u64;

    Ok(LoadedPlugin {
        _library: library,
        handle,
        ffi,
    })
}
