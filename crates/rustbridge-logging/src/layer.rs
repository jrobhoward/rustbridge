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

        // Extract the message from the event
        let mut visitor = MessageVisitor::default();
        event.record(&mut visitor);

        let message = visitor.message.unwrap_or_default();
        let target = metadata.target();

        // Forward to the callback
        self.manager.log(level, target, &message);
    }

    fn enabled(&self, metadata: &tracing::Metadata<'_>, _ctx: Context<'_, S>) -> bool {
        let level = Self::convert_level(metadata.level());
        self.manager.is_enabled(level)
    }
}

/// Visitor to extract the message field from tracing events
#[derive(Default)]
struct MessageVisitor {
    message: Option<String>,
}

impl Visit for MessageVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message = Some(format!("{:?}", value));
        } else if self.message.is_none() {
            // If no "message" field, use the first field
            self.message = Some(format!("{:?}", value));
        }
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        // Use "message" field if present, otherwise use first field encountered
        if field.name() == "message" || self.message.is_none() {
            self.message = Some(value.to_string());
        }
    }
}

/// Initialize the logging system with the FFI layer
///
/// This sets up tracing with the FFI logging layer. Call this once during
/// plugin initialization.
pub fn init_logging() {
    use tracing_subscriber::prelude::*;

    let layer = FfiLoggingLayer::new();

    // Create a subscriber with the FFI layer
    let subscriber = tracing_subscriber::registry().with(layer);

    // Try to set as global default (ignore error if already set)
    let _ = tracing::subscriber::set_global_default(subscriber);
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
