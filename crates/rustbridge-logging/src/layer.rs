//! Tracing layer that forwards to FFI callbacks

use crate::callback::LogCallbackManager;
use rustbridge_core::LogLevel;
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::LookupSpan;

/// Tracing layer that forwards log events to FFI callbacks
pub struct FfiLoggingLayer {
    manager: &'static LogCallbackManager,
}

impl FfiLoggingLayer {
    /// Create a new FFI logging layer using the global callback manager
    pub fn new() -> Self {
        Self {
            manager: LogCallbackManager::global(),
        }
    }

    /// Create a layer with a specific callback manager
    pub fn with_manager(manager: &'static LogCallbackManager) -> Self {
        Self { manager }
    }

    /// Convert tracing Level to our LogLevel
    fn convert_level(level: &Level) -> LogLevel {
        match *level {
            Level::TRACE => LogLevel::Trace,
            Level::DEBUG => LogLevel::Debug,
            Level::INFO => LogLevel::Info,
            Level::WARN => LogLevel::Warn,
            Level::ERROR => LogLevel::Error,
        }
    }
}

impl Default for FfiLoggingLayer {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> Layer<S> for FfiLoggingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        let metadata = event.metadata();
        let level = Self::convert_level(metadata.level());

        // Check if this level is enabled before doing any work
        if !self.manager.is_enabled(level) {
            return;
        }

        // Extract the message and fields from the event
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let message = visitor.into_message();
        let target = metadata.target();

        // Forward to the callback
        self.manager.log(level, target, &message);
    }

    fn enabled(&self, metadata: &tracing::Metadata<'_>, _ctx: Context<'_, S>) -> bool {
        let level = Self::convert_level(metadata.level());
        self.manager.is_enabled(level)
    }
}

/// Visitor to extract and format all fields from tracing events
#[derive(Default)]
struct MessageVisitor {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        let name = field.name();
        if name == "message" {
            self.message = Some(format!("{:?}", value));
        } else {
            // Collect all other fields as key=value pairs
            self.fields.push((name.to_string(), format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        let name = field.name();
        if name == "message" {
            self.message = Some(value.to_string());
        } else {
            // Collect all other fields as key=value pairs
            self.fields.push((name.to_string(), value.to_string()));
        }
    }

    fn record_i64(&mut self, field: &Field, value: i64) {
        let name = field.name();
        if name != "message" {
            self.fields.push((name.to_string(), value.to_string()));
        }
    }

    fn record_u64(&mut self, field: &Field, value: u64) {
        let name = field.name();
        if name != "message" {
            self.fields.push((name.to_string(), value.to_string()));
        }
    }

    fn record_bool(&mut self, field: &Field, value: bool) {
        let name = field.name();
        if name != "message" {
            self.fields.push((name.to_string(), value.to_string()));
        }
    }
}

impl MessageVisitor {
    /// Build the final formatted message with all fields
    fn into_message(self) -> String {
        let mut result = self.message.unwrap_or_default();

        // Append structured fields as key=value pairs
        if !self.fields.is_empty() {
            if !result.is_empty() {
                result.push(' ');
            }
            for (i, (key, value)) in self.fields.iter().enumerate() {
                if i > 0 {
                    result.push(' ');
                }
                result.push_str(&format!("{}={}", key, value));
            }
        }

        result
    }
}

/// Initialize the logging system with the FFI layer
///
/// This sets up tracing with the FFI logging layer. Call this once during
/// plugin initialization. Subsequent calls after the first initialization
/// are no-ops since the subscriber is global.
pub fn init_logging() {
    use once_cell::sync::OnceCell;
    use tracing_subscriber::filter::LevelFilter;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::reload;

    // Use OnceCell to ensure we only initialize once
    static INITIALIZED: OnceCell<()> = OnceCell::new();

    INITIALIZED.get_or_init(|| {
        let layer = FfiLoggingLayer::new();

        // Create a reloadable level filter
        let initial_level = LogCallbackManager::global().level();
        let initial_filter = match initial_level {
            LogLevel::Trace => LevelFilter::TRACE,
            LogLevel::Debug => LevelFilter::DEBUG,
            LogLevel::Info => LevelFilter::INFO,
            LogLevel::Warn => LevelFilter::WARN,
            LogLevel::Error => LevelFilter::ERROR,
            LogLevel::Off => LevelFilter::OFF,
        };

        let (filter, reload_handle) = reload::Layer::new(initial_filter);

        // Store the reload handle for later use
        crate::reload::ReloadHandle::global().set_handle(reload_handle);

        // Create subscriber with reloadable filter first, then FFI layer
        let subscriber = tracing_subscriber::registry().with(filter).with(layer);

        // Set as global default - ignore error if already set
        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

/// Initialize logging with a specific log level
#[allow(dead_code)] // Public API for plugin authors
pub fn init_logging_with_level(level: LogLevel) {
    LogCallbackManager::global().set_level(level);
    init_logging();
}

#[cfg(test)]
#[path = "layer/layer_tests.rs"]
mod layer_tests;
