//! Dynamic log level reloading support

use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use rustbridge_core::LogLevel;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::reload;

/// Handle for dynamically reloading the log level filter
pub struct ReloadHandle {
    handle: Mutex<Option<reload::Handle<LevelFilter, tracing_subscriber::Registry>>>,
}

impl ReloadHandle {
    /// Create a new reload handle
    pub fn new() -> Self {
        Self {
            handle: Mutex::new(None),
        }
    }

    /// Get the global reload handle
    pub fn global() -> &'static ReloadHandle {
        static INSTANCE: OnceCell<ReloadHandle> = OnceCell::new();
        INSTANCE.get_or_init(ReloadHandle::new)
    }

    /// Set the reload handle (called during initialization)
    pub fn set_handle(&self, handle: reload::Handle<LevelFilter, tracing_subscriber::Registry>) {
        *self.handle.lock() = Some(handle);
    }

    /// Reload the filter to use a new log level
    pub fn reload_level(&self, level: LogLevel) -> Result<(), String> {
        let guard = self.handle.lock();
        if let Some(handle) = guard.as_ref() {
            let filter = convert_level_to_filter(level);
            handle
                .reload(filter)
                .map_err(|e| format!("Failed to reload filter: {}", e))?;
            Ok(())
        } else {
            Err("Reload handle not initialized".to_string())
        }
    }
}

impl Default for ReloadHandle {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert LogLevel to tracing LevelFilter
fn convert_level_to_filter(level: LogLevel) -> LevelFilter {
    match level {
        LogLevel::Trace => LevelFilter::TRACE,
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
        LogLevel::Off => LevelFilter::OFF,
    }
}
