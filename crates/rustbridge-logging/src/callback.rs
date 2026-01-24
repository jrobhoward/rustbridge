//! FFI log callback management

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use rustbridge_core::LogLevel;
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

/// FFI callback function type for logging
///
/// # Parameters
/// - `level`: Log level (0=Trace, 1=Debug, 2=Info, 3=Warn, 4=Error)
/// - `target`: Log target (module path), null-terminated C string
/// - `message`: Log message, pointer to UTF-8 bytes
/// - `message_len`: Length of the message in bytes
///
/// # Safety
/// The callback is invoked from Rust code. The `target` string is null-terminated.
/// The `message` pointer is valid for `message_len` bytes during the callback.
pub type LogCallback = extern "C" fn(
    level: u8,
    target: *const std::ffi::c_char,
    message: *const u8,
    message_len: usize,
);

/// Global log callback manager
static CALLBACK_MANAGER: OnceCell<LogCallbackManager> = OnceCell::new();

/// Manager for FFI log callbacks
///
/// Each plugin can register its own log callback. The callback is cleared
/// when the plugin that registered it shuts down to prevent use-after-free
/// (the callback function pointer is tied to the plugin's FFI arena lifetime).
///
/// Log level is shared globally and persists across plugin reload cycles.
pub struct LogCallbackManager {
    callback: RwLock<Option<LogCallback>>,
    level: AtomicU8,
    /// Number of active plugins using this callback manager
    ref_count: AtomicUsize,
    /// Whether the current callback was registered by a plugin (vs None)
    has_callback: std::sync::atomic::AtomicBool,
}

impl LogCallbackManager {
    /// Create a new callback manager
    pub fn new() -> Self {
        Self {
            callback: RwLock::new(None),
            level: AtomicU8::new(LogLevel::Info as u8),
            ref_count: AtomicUsize::new(0),
            has_callback: std::sync::atomic::AtomicBool::new(false),
        }
    }

    /// Get the global callback manager instance
    pub fn global() -> &'static LogCallbackManager {
        CALLBACK_MANAGER.get_or_init(LogCallbackManager::new)
    }

    /// Set the log callback
    pub fn set_callback(&self, callback: Option<LogCallback>) {
        let mut guard = self.callback.write();
        *guard = callback;
    }

    /// Register a plugin with the callback manager
    ///
    /// This increments the reference count and optionally sets the callback.
    /// If a callback is provided, it replaces any existing callback.
    ///
    /// **Important**: The callback function pointer is tied to the plugin's
    /// FFI arena lifetime. When the plugin shuts down, the callback becomes
    /// invalid and must be cleared before the arena is closed.
    ///
    /// This should be called during plugin initialization.
    pub fn register_plugin(&self, callback: Option<LogCallback>) {
        // Increment reference count
        self.ref_count.fetch_add(1, Ordering::SeqCst);

        // Set callback if provided
        if let Some(cb) = callback {
            let mut guard = self.callback.write();
            *guard = Some(cb);
            self.has_callback
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }

    /// Unregister a plugin from the callback manager
    ///
    /// **Critical**: This ALWAYS clears the callback to prevent use-after-free.
    /// The callback function pointer is tied to the plugin's FFI arena, which
    /// will be closed immediately after this function returns. If we didn't
    /// clear the callback, any subsequent logging would call an invalid pointer.
    ///
    /// This means that with multiple plugins, the last one to unregister will
    /// disable logging for any remaining plugins until they re-register a callback.
    /// This is a safety trade-off: we prioritize crash prevention over convenience.
    ///
    /// Note: The reload handle is NOT cleared because logging initialization
    /// only happens once per process. The reload handle must persist across
    /// plugin reload cycles.
    ///
    /// This should be called during plugin shutdown.
    pub fn unregister_plugin(&self) {
        // ALWAYS clear the callback first, before decrementing ref count.
        // This prevents use-after-free when the plugin's arena is closed.
        {
            let mut guard = self.callback.write();
            *guard = None;
            self.has_callback
                .store(false, std::sync::atomic::Ordering::SeqCst);
        } // Write lock must be released BEFORE logging to avoid deadlock

        let prev_count = self.ref_count.fetch_sub(1, Ordering::SeqCst);

        if prev_count == 1 {
            tracing::debug!("Last plugin unregistered, cleared log callback");
        } else {
            tracing::debug!(
                "Plugin unregistered, cleared log callback (safety). {} plugins remaining",
                prev_count - 1
            );
        }
    }

    /// Get the current reference count (number of active plugins)
    pub fn plugin_count(&self) -> usize {
        self.ref_count.load(Ordering::SeqCst)
    }

    /// Get the current log callback
    pub fn get_callback(&self) -> Option<LogCallback> {
        *self.callback.read()
    }

    /// Set the log level
    pub fn set_level(&self, level: LogLevel) {
        self.level.store(level as u8, Ordering::SeqCst);
    }

    /// Get the current log level
    pub fn level(&self) -> LogLevel {
        LogLevel::from_u8(self.level.load(Ordering::SeqCst))
    }

    /// Check if a log level is enabled
    pub fn is_enabled(&self, level: LogLevel) -> bool {
        level >= self.level()
    }

    /// Invoke the callback if set and level is enabled
    ///
    /// # Safety
    /// The callback must be a valid function pointer that follows the
    /// LogCallback signature contract.
    pub fn log(&self, level: LogLevel, target: &str, message: &str) {
        // Check if level is enabled
        if !self.is_enabled(level) {
            return;
        }

        // Get callback
        let callback = match self.get_callback() {
            Some(cb) => cb,
            None => return,
        };

        // Prepare target as C string
        let target_cstring = match std::ffi::CString::new(target) {
            Ok(s) => s,
            Err(_) => return, // Invalid target string
        };

        // Invoke callback
        callback(
            level as u8,
            target_cstring.as_ptr(),
            message.as_ptr(),
            message.len(),
        );
    }
}

impl Default for LogCallbackManager {
    fn default() -> Self {
        Self::new()
    }
}

// Ensure LogCallbackManager is thread-safe
unsafe impl Send for LogCallbackManager {}
unsafe impl Sync for LogCallbackManager {}

#[cfg(test)]
#[path = "callback/callback_tests.rs"]
mod callback_tests;
