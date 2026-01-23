//! Plugin configuration types

use serde::{Deserialize, Serialize};

/// Plugin configuration passed during initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin-specific configuration data
    #[serde(default)]
    pub data: serde_json::Value,

    /// Number of async worker threads (default: number of CPU cores)
    #[serde(default)]
    pub worker_threads: Option<usize>,

    /// Initial log level
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Maximum concurrent async operations
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_ops: usize,

    /// Shutdown timeout in milliseconds
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout_ms: u64,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_max_concurrent() -> usize {
    1000
}

fn default_shutdown_timeout() -> u64 {
    5000
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            data: serde_json::Value::Null,
            worker_threads: None,
            log_level: default_log_level(),
            max_concurrent_ops: default_max_concurrent(),
            shutdown_timeout_ms: default_shutdown_timeout(),
        }
    }
}

impl PluginConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration from JSON bytes
    pub fn from_json(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        serde_json::from_slice(bytes)
    }

    /// Get a typed value from the configuration data
    pub fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set a value in the configuration data
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), serde_json::Error> {
        // Ensure data is an object
        if !self.data.is_object() {
            self.data = serde_json::json!({});
        }
        // Now we can safely get the object
        let obj = self.data.as_object_mut().unwrap();
        obj.insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }
}

/// Plugin metadata from rustbridge.toml manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,

    /// Plugin version (semver)
    pub version: String,

    /// Plugin description
    #[serde(default)]
    pub description: Option<String>,

    /// Plugin author(s)
    #[serde(default)]
    pub authors: Vec<String>,

    /// Minimum rustbridge version required
    #[serde(default)]
    pub min_rustbridge_version: Option<String>,
}

impl PluginMetadata {
    /// Create new plugin metadata
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            authors: Vec::new(),
            min_rustbridge_version: None,
        }
    }
}

#[cfg(test)]
#[path = "config/config_tests.rs"]
mod config_tests;
