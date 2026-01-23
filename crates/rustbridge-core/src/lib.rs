//! rustbridge-core - Core traits, types, and lifecycle management
//!
//! This crate provides the foundational types for building rustbridge plugins:
//! - [`Plugin`] trait for implementing plugin logic
//! - [`LifecycleState`] for managing plugin lifecycle
//! - [`PluginError`] for error handling
//! - [`PluginConfig`] for plugin configuration

mod config;
mod error;
mod lifecycle;
mod plugin;
mod request;

pub use config::{PluginConfig, PluginMetadata};
pub use error::{PluginError, PluginResult};
pub use lifecycle::LifecycleState;
pub use plugin::{Plugin, PluginContext};
pub use request::{RequestContext, ResponseBuilder};

/// Log levels for FFI callbacks
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace = 0,
    Debug = 1,
    Info = 2,
    Warn = 3,
    Error = 4,
    Off = 5,
}

impl LogLevel {
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => LogLevel::Trace,
            1 => LogLevel::Debug,
            2 => LogLevel::Info,
            3 => LogLevel::Warn,
            4 => LogLevel::Error,
            _ => LogLevel::Off,
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "TRACE"),
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
            LogLevel::Off => write!(f, "OFF"),
        }
    }
}

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        LogLevel, LifecycleState, Plugin, PluginConfig, PluginContext, PluginError, PluginResult,
        RequestContext, ResponseBuilder,
    };
}

#[cfg(test)]
mod lib_tests;
