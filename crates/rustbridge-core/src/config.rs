//! Plugin configuration types

use serde::{Deserialize, Serialize};

/// Plugin configuration passed during initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Plugin-specific configuration data
    ///
    /// General runtime configuration such as database URLs, API keys, feature flags, etc.
    /// Accessible throughout the plugin's lifetime.
    #[serde(default)]
    pub data: serde_json::Value,

    /// Initialization parameters
    ///
    /// Structured data passed to the plugin during initialization.
    /// Intended for one-time setup parameters that are only needed during `on_start()`.
    /// Provides better separation from runtime configuration in `data`.
    #[serde(default)]
    pub init_params: Option<serde_json::Value>,

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
            init_params: None,
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
        self.data
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Set a value in the configuration data
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<(), serde_json::Error> {
        // Ensure data is an object
        if !self.data.is_object() {
            self.data = serde_json::json!({});
        }
        // Now we can safely get the object
        #[allow(clippy::unwrap_used)] // Safe: we just set data to an empty object above
        let obj = self.data.as_object_mut().unwrap();
        obj.insert(key.to_string(), serde_json::to_value(value)?);
        Ok(())
    }

    /// Get a typed value from initialization parameters
    ///
    /// Returns `None` if init_params is not set or the key doesn't exist.
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[derive(Deserialize)]
    /// struct DatabaseInit {
    ///     migrations_path: String,
    ///     seed_data: bool,
    /// }
    ///
    /// async fn on_start(&self, ctx: &PluginContext) -> PluginResult<()> {
    ///     if let Some(db_init) = ctx.config().get_init_param::<DatabaseInit>("database") {
    ///         if db_init.seed_data {
    ///             self.seed_database(&db_init.migrations_path).await?;
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn get_init_param<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.init_params
            .as_ref()?
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get the entire initialization parameters as a typed value
    ///
    /// Returns `None` if init_params is not set.
    ///
    /// # Example
    ///
    /// ```ignore
    /// #[derive(Deserialize)]
    /// struct InitParams {
    ///     setup_mode: String,
    ///     enable_features: Vec<String>,
    /// }
    ///
    /// async fn on_start(&self, ctx: &PluginContext) -> PluginResult<()> {
    ///     if let Some(params) = ctx.config().init_params_as::<InitParams>() {
    ///         // Use initialization parameters...
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn init_params_as<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        let init_params = self.init_params.as_ref()?;
        serde_json::from_value(init_params.clone()).ok()
    }

    /// Set initialization parameters
    ///
    /// This is typically called by the host application before initializing the plugin.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut config = PluginConfig::default();
    /// config.set_init_params(serde_json::json!({
    ///     "setup_mode": "development",
    ///     "seed_data": true
    /// }));
    /// ```
    pub fn set_init_params(&mut self, params: serde_json::Value) {
        self.init_params = Some(params);
    }
}

/// Plugin metadata from bundle manifest
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

#[cfg(test)]
#[path = "config/config_parameterized_tests.rs"]
mod config_parameterized_tests;
