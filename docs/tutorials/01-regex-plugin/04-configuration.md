# Section 4: Make It Configurable

In this section, you'll allow the host application to configure the cache size at plugin initialization time.

## Define Configuration Data

Between `// Message Types` and `// Plugin Implementation`, add a new struct to represent the plugin's JSON configuration:

```rust
// ============================================================================
// Plugin Configuration
// ============================================================================

/// Plugin configuration data (parsed from config.data JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigData {
    /// Maximum number of compiled regex patterns to cache
    pub cache_size: usize,
}

impl Default for PluginConfigData {
    fn default() -> Self {
        Self { cache_size: 100 }
    }
}
```

When the host initializes the plugin, they can now pass:

```json
{
  "cache_size": 500
}
```

## Update the Plugin Struct

Add a field to track the configured cache size (useful for logging):

```rust
/// Regex plugin with configurable LRU caching
pub struct RegexPlugin {
    /// LRU cache of compiled regexes (pattern -> Regex)
    cache: Mutex<LruCache<String, Regex>>,
    /// The configured cache size (for logging)
    cache_size: usize,
}

impl Default for RegexPlugin {
    fn default() -> Self {
        let cache_size = NonZeroUsize::new(DEFAULT_CACHE_SIZE).expect("default is non-zero");
        Self {
            cache: Mutex::new(LruCache::new(cache_size)),
            cache_size: cache_size.get(),
        }
    }
}
```

## Implement PluginFactory

Add the factory implementation:

```rust
impl PluginFactory for RegexPlugin {
    /// Called when config.data contains JSON configuration
    fn create_configured(config: &PluginConfig) -> PluginResult<Self> {
        // Parse the configuration
        let config_data: PluginConfigData = serde_json::from_value(config.data.clone())?;

        // Handle invalid cache size gracefully
        let cache_size = NonZeroUsize::new(config_data.cache_size)
            .unwrap_or_else(|| NonZeroUsize::new(DEFAULT_CACHE_SIZE).expect("default is non-zero"));

        tracing::info!(
            cache_size = cache_size.get(),
            "Creating regex plugin with custom configuration"
        );

        Ok(Self {
            cache: Mutex::new(LruCache::new(cache_size)),
            cache_size: cache_size.get(),
        })
    }
}
```

The trait provides a default `create()` implementation that:

- Returns `Self::default()` if `config.data` is null
- Calls `create_configured()` if `config.data` has a value

## Update Logging

Update `on_start` to log the configured cache size:

```rust
async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
    tracing::info!(
        cache_size = self.cache_size,
        "regex-plugin started"
    );
    Ok(())
}
```

## Update the FFI Entry Point

Change from `default` to `create`:

```rust
rustbridge_entry!(RegexPlugin::create);
pub use rustbridge::ffi_exports::*;
```

## Add Configuration Tests

```rust
#[tokio::test]
async fn create___with_null_config___uses_default_cache_size() {
    let config = PluginConfig::default();  // data is Value::Null

    let plugin = RegexPlugin::create(&config).unwrap();

    assert_eq!(plugin.cache_size, DEFAULT_CACHE_SIZE);
}

#[tokio::test]
async fn create___with_custom_config___uses_configured_cache_size() {
    let config_data = PluginConfigData { cache_size: 50 };
    let config = PluginConfig {
        data: serde_json::to_value(config_data).unwrap(),
        ..Default::default()
    };

    let plugin = RegexPlugin::create(&config).unwrap();

    assert_eq!(plugin.cache_size, 50);
}

#[tokio::test]
async fn create___with_zero_cache_size___uses_default() {
    let config_data = PluginConfigData { cache_size: 0 };  // Invalid!
    let config = PluginConfig {
        data: serde_json::to_value(config_data).unwrap(),
        ..Default::default()
    };

    let plugin = RegexPlugin::create(&config).unwrap();

    // Zero is invalid for NonZeroUsize, so we fall back to default
    assert_eq!(plugin.cache_size, DEFAULT_CACHE_SIZE);
}
```

## Complete lib.rs

Here's the complete implementation:

```rust
//! regex-plugin - A rustbridge plugin demonstrating regex matching with LRU caching

use lru::LruCache;
use regex::Regex;
use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};
use std::num::NonZeroUsize;
use std::sync::Mutex;

// ============================================================================
// Message Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "match")]
pub struct MatchRequest {
    pub pattern: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResponse {
    pub matches: bool,
    pub cached: bool,
}

// ============================================================================
// Plugin Configuration
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigData {
    pub cache_size: usize,
}

impl Default for PluginConfigData {
    fn default() -> Self {
        Self { cache_size: 100 }
    }
}

// ============================================================================
// Plugin Implementation
// ============================================================================

const DEFAULT_CACHE_SIZE: usize = 100;

pub struct RegexPlugin {
    cache: Mutex<LruCache<String, Regex>>,
    cache_size: usize,
}

impl Default for RegexPlugin {
    fn default() -> Self {
        let cache_size = NonZeroUsize::new(DEFAULT_CACHE_SIZE).expect("default is non-zero");
        Self {
            cache: Mutex::new(LruCache::new(cache_size)),
            cache_size: cache_size.get(),
        }
    }
}

impl PluginFactory for RegexPlugin {
    fn create_configured(config: &PluginConfig) -> PluginResult<Self> {
        let config_data: PluginConfigData = serde_json::from_value(config.data.clone())?;

        let cache_size = NonZeroUsize::new(config_data.cache_size)
            .unwrap_or_else(|| NonZeroUsize::new(DEFAULT_CACHE_SIZE).expect("default is non-zero"));

        tracing::info!(
            cache_size = cache_size.get(),
            "Creating regex plugin with custom configuration"
        );

        Ok(Self {
            cache: Mutex::new(LruCache::new(cache_size)),
            cache_size: cache_size.get(),
        })
    }
}

#[async_trait]
impl Plugin for RegexPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!(cache_size = self.cache_size, "regex-plugin started");
        Ok(())
    }

    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        match type_tag {
            "match" => {
                let req: MatchRequest = serde_json::from_slice(payload)?;

                tracing::debug!(
                    pattern = %req.pattern,
                    text_len = req.text.len(),
                    "Processing match request"
                );

                let mut cache = self.cache.lock().expect("cache lock poisoned");

                let (regex, cached) = if let Some(regex) = cache.get(&req.pattern) {
                    (regex, true)
                } else {
                    let regex = Regex::new(&req.pattern).map_err(|e| {
                        tracing::warn!(
                            pattern = %req.pattern,
                            error = %e,
                            "Invalid regex pattern"
                        );
                        PluginError::HandlerError(format!("Invalid regex pattern: {}", e))
                    })?;
                    cache.put(req.pattern.clone(), regex);
                    (cache.get(&req.pattern).expect("just inserted"), false)
                };

                let matches = regex.is_match(&req.text);

                tracing::debug!(
                    pattern = %req.pattern,
                    matches,
                    cached,
                    "Match completed"
                );

                let response = MatchResponse { matches, cached };
                Ok(serde_json::to_vec(&response)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        let cache = self.cache.lock().expect("cache lock poisoned");
        tracing::info!(cached_patterns = cache.len(), "regex-plugin stopped");
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["match"]
    }
}

// ============================================================================
// FFI Entry Point
// ============================================================================

rustbridge_entry!(RegexPlugin::create);
pub use rustbridge::ffi_exports::*;
```

## Build and Test

```bash
cargo build --release
cargo test
```

## Create a Bundle

Now let's package it for distribution:

```bash
# Linux
rustbridge bundle create \
  --name regex-plugin \
  --version 0.1.0 \
  --lib linux-x86_64:target/release/libregex_plugin.so \
  --output regex-plugin-0.1.0.rbp
```

```bash
# macOS
rustbridge bundle create \
  --name regex-plugin \
  --version 0.1.0 \
  --lib darwin-aarch64:target/release/libregex_plugin.dylib \
  --output regex-plugin-0.1.0.rbp
```

```bash
# Windows
rustbridge bundle create \
  --name regex-plugin \
  --version 0.1.0 \
  --lib windows-x86_64:target/release/regex_plugin.dll \
  --output regex-plugin-0.1.0.rbp
```

Verify:

```bash
rustbridge bundle list regex-plugin-0.1.0.rbp
```

## Summary

You've built a complete plugin with:

- **Regex matching**: Pattern validation and matching
- **LRU caching**: Compiled patterns are cached for performance
- **Configuration**: Cache size is configurable at initialization
- **Logging**: Structured tracing at info and debug levels

## What's Next?

In Chapter 2, you'll call this plugin from Kotlin with type-safe wrappers and logging callbacks.

[Continue to Chapter 2: Calling from Kotlin →](../02-kotlin-consumer/README.md)
