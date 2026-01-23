//! FFI log callback management

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use rustbridge_core::LogLevel;
use std::sync::atomic::{AtomicU8, Ordering};

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
pub struct LogCallbackManager {
    callback: RwLock<Option<LogCallback>>,
    level: AtomicU8,
}

impl LogCallbackManager {
    /// Create a new callback manager
    pub fn new() -> Self {
        Self {
            callback: RwLock::new(None),
            level: AtomicU8::new(LogLevel::Info as u8),
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
