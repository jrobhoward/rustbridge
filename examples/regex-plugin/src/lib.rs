//! regex-plugin - A rustbridge plugin demonstrating regex matching with LRU caching
//!
//! This example shows how to build a production-quality plugin with:
//! - Regex pattern matching
//! - LRU cache for compiled patterns
//! - Configuration support via PluginFactory
//! - Structured logging

use lru::LruCache;
use regex::Regex;
use rustbridge::prelude::*;
use std::num::NonZeroUsize;
use std::sync::Mutex;

// ============================================================================
// Message Types
// ============================================================================

/// Regex match request message
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "match")]
pub struct MatchRequest {
    /// The regex pattern to match against
    pub pattern: String,
    /// The text to test
    pub text: String,
}

/// Regex match response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResponse {
    /// Whether the pattern matched the text
    pub matches: bool,
    /// Whether the pattern was retrieved from cache
    pub cached: bool,
}

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

// ============================================================================
// Plugin Implementation
// ============================================================================

const DEFAULT_CACHE_SIZE: usize = 100;

/// Regex plugin with LRU caching
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

impl PluginFactory for RegexPlugin {
    /// Called when config.data contains JSON configuration
    fn create_configured(config: &PluginConfig) -> PluginResult<Self> {
        let config_data: PluginConfigData = rustbridge::serde_json::from_value(config.data.clone())?;

        let cache_size = NonZeroUsize::new(config_data.cache_size)
            .unwrap_or_else(|| NonZeroUsize::new(DEFAULT_CACHE_SIZE).expect("default is non-zero"));

        rustbridge::tracing::info!(
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
        rustbridge::tracing::info!(
            cache_size = self.cache_size,
            "regex-plugin started"
        );
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
                let req: MatchRequest = rustbridge::serde_json::from_slice(payload)?;

                rustbridge::tracing::debug!(
                    pattern = %req.pattern,
                    text_len = req.text.len(),
                    "Processing match request"
                );

                let mut cache = self.cache.lock().expect("cache lock poisoned");

                // Check if regex is in cache
                let (regex, cached) = if let Some(regex) = cache.get(&req.pattern) {
                    (regex, true)
                } else {
                    // Compile the regex and add it to the cache
                    let regex = Regex::new(&req.pattern).map_err(|e| {
                        rustbridge::tracing::warn!(
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

                rustbridge::tracing::debug!(
                    pattern = %req.pattern,
                    matches,
                    cached,
                    "Match completed"
                );

                let response = MatchResponse { matches, cached };
                Ok(rustbridge::serde_json::to_vec(&response)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        let cache = self.cache.lock().expect("cache lock poisoned");
        rustbridge::tracing::info!(
            cached_patterns = cache.len(),
            "regex-plugin stopped"
        );
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["match"]
    }
}

// ============================================================================
// FFI Entry Point
// ============================================================================

// Generate FFI entry point - the macro calls PluginFactory::create()
// which handles null config automatically
rustbridge_entry!(RegexPlugin::create);

// Re-export FFI functions for the shared library
pub use rustbridge::ffi_exports::*;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn create___with_null_config___uses_default_cache_size() {
        let config = PluginConfig::default();

        let plugin = RegexPlugin::create(&config).unwrap();

        assert_eq!(plugin.cache_size, DEFAULT_CACHE_SIZE);
    }

    #[tokio::test]
    async fn create___with_custom_config___uses_configured_cache_size() {
        let config_data = PluginConfigData { cache_size: 50 };
        let mut config = PluginConfig::default();
        config.data = rustbridge::serde_json::to_value(config_data).unwrap();

        let plugin = RegexPlugin::create(&config).unwrap();

        assert_eq!(plugin.cache_size, 50);
    }

    #[tokio::test]
    async fn handle_request___matching_pattern___returns_true() {
        let config = PluginConfig::default();
        let plugin = RegexPlugin::create(&config).unwrap();
        let ctx = PluginContext::new(config);

        plugin.on_start(&ctx).await.unwrap();

        let request = rustbridge::serde_json::to_vec(&MatchRequest {
            pattern: r"\d+".to_string(),
            text: "test123".to_string(),
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "match", &request).await.unwrap();
        let match_response: MatchResponse = rustbridge::serde_json::from_slice(&response).unwrap();

        assert!(match_response.matches);
        assert!(!match_response.cached);
    }

    #[tokio::test]
    async fn handle_request___non_matching_pattern___returns_false() {
        let config = PluginConfig::default();
        let plugin = RegexPlugin::create(&config).unwrap();
        let ctx = PluginContext::new(config);

        plugin.on_start(&ctx).await.unwrap();

        let request = rustbridge::serde_json::to_vec(&MatchRequest {
            pattern: r"^\d+$".to_string(),
            text: "test123".to_string(),
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "match", &request).await.unwrap();
        let match_response: MatchResponse = rustbridge::serde_json::from_slice(&response).unwrap();

        assert!(!match_response.matches);
    }

    #[tokio::test]
    async fn handle_request___same_pattern_twice___second_is_cached() {
        let config = PluginConfig::default();
        let plugin = RegexPlugin::create(&config).unwrap();
        let ctx = PluginContext::new(config);

        plugin.on_start(&ctx).await.unwrap();

        // First request - should compile and cache
        let request = rustbridge::serde_json::to_vec(&MatchRequest {
            pattern: r"\d+".to_string(),
            text: "test123".to_string(),
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "match", &request).await.unwrap();
        let first: MatchResponse = rustbridge::serde_json::from_slice(&response).unwrap();

        assert!(!first.cached);

        // Second request with same pattern - should be cached
        let request = rustbridge::serde_json::to_vec(&MatchRequest {
            pattern: r"\d+".to_string(),
            text: "456".to_string(),
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "match", &request).await.unwrap();
        let second: MatchResponse = rustbridge::serde_json::from_slice(&response).unwrap();

        assert!(second.cached);
        assert!(second.matches);
    }

    #[tokio::test]
    async fn handle_request___invalid_regex___returns_error() {
        let config = PluginConfig::default();
        let plugin = RegexPlugin::create(&config).unwrap();
        let ctx = PluginContext::new(config);

        plugin.on_start(&ctx).await.unwrap();

        let request = rustbridge::serde_json::to_vec(&MatchRequest {
            pattern: r"[invalid".to_string(),
            text: "test".to_string(),
        })
        .unwrap();

        let result = plugin.handle_request(&ctx, "match", &request).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn handle_request___unknown_type___returns_error() {
        let config = PluginConfig::default();
        let plugin = RegexPlugin::create(&config).unwrap();
        let ctx = PluginContext::new(config);

        plugin.on_start(&ctx).await.unwrap();

        let result = plugin.handle_request(&ctx, "unknown", b"{}").await;

        assert!(matches!(result, Err(PluginError::UnknownMessageType(_))));
    }

    #[tokio::test]
    async fn supported_types___returns_match() {
        let plugin = RegexPlugin::default();

        assert_eq!(plugin.supported_types(), vec!["match"]);
    }

    /// Benchmark demonstrating cache effectiveness
    #[tokio::test]
    async fn bench___cache_effectiveness() {
        use std::time::Instant;

        let patterns = vec![
            r"^\d{4}-\d{2}-\d{2}$",  // date pattern
            r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",  // email
            r"^https?://[^\s/$.?#].[^\s]*$",  // URL
            r"^\+?[1-9]\d{1,14}$",  // phone number
        ];
        let sample = "2024-01-15";
        let iterations = 1000;

        // Small cache (size 2) - will have cache misses cycling through 4 patterns
        let small_config = {
            let config_data = PluginConfigData { cache_size: 2 };
            let mut config = PluginConfig::default();
            config.data = rustbridge::serde_json::to_value(config_data).unwrap();
            config
        };
        let small_plugin = RegexPlugin::create(&small_config).unwrap();
        let small_ctx = PluginContext::new(small_config);

        let start = Instant::now();
        for _ in 0..iterations {
            for pattern in &patterns {
                let request = rustbridge::serde_json::to_vec(&MatchRequest {
                    pattern: pattern.to_string(),
                    text: sample.to_string(),
                })
                .unwrap();
                let _ = small_plugin.handle_request(&small_ctx, "match", &request).await;
            }
        }
        let small_cache_time = start.elapsed();

        // Large cache (size 100) - should have cache hits after first pass
        let large_config = {
            let config_data = PluginConfigData { cache_size: 100 };
            let mut config = PluginConfig::default();
            config.data = rustbridge::serde_json::to_value(config_data).unwrap();
            config
        };
        let large_plugin = RegexPlugin::create(&large_config).unwrap();
        let large_ctx = PluginContext::new(large_config);

        let start = Instant::now();
        for _ in 0..iterations {
            for pattern in &patterns {
                let request = rustbridge::serde_json::to_vec(&MatchRequest {
                    pattern: pattern.to_string(),
                    text: sample.to_string(),
                })
                .unwrap();
                let _ = large_plugin.handle_request(&large_ctx, "match", &request).await;
            }
        }
        let large_cache_time = start.elapsed();

        println!("\n=== Cache Effectiveness Benchmark ===");
        println!("Patterns: {}, Iterations: {}", patterns.len(), iterations);
        println!("Small cache (size 2):   {:?}", small_cache_time);
        println!("Large cache (size 100): {:?}", large_cache_time);
        println!(
            "Speedup: {:.2}x",
            small_cache_time.as_secs_f64() / large_cache_time.as_secs_f64()
        );

        // The large cache should be faster since it doesn't re-compile regexes
        assert!(
            large_cache_time < small_cache_time,
            "Large cache should be faster than small cache"
        );
    }
}
