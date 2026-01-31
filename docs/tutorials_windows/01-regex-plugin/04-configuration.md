# Section 4: Configuration

In this section, you'll make the cache size configurable from the host application.

## Add Configuration Support

Update `src\lib.rs` to read configuration from the plugin context:

```rust
//! regex-plugin - A regex engine with configurable LRU caching

use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};
use lru::LruCache;
use regex::Regex;
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
    pub matched: bool,
    pub match_text: Option<String>,
    pub start: Option<usize>,
    pub end: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "find_all")]
pub struct FindAllRequest {
    pub pattern: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindAllResponse {
    pub matches: Vec<String>,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "stats")]
pub struct StatsRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    pub cached_patterns: usize,
    pub cache_capacity: usize,
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

// ============================================================================
// Configuration
// ============================================================================

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexConfig {
    /// Maximum number of compiled patterns to cache
    #[serde(default = "default_cache_size")]
    pub cache_size: usize,
}

fn default_cache_size() -> usize {
    100
}

impl Default for RegexConfig {
    fn default() -> Self {
        Self {
            cache_size: default_cache_size(),
        }
    }
}

// ============================================================================
// Plugin Implementation
// ============================================================================

pub struct RegexPlugin {
    cache: Mutex<Option<LruCache<String, Regex>>>,
    config: Mutex<RegexConfig>,
    stats: Mutex<CacheStats>,
}

#[derive(Default)]
struct CacheStats {
    total_requests: u64,
    cache_hits: u64,
    cache_misses: u64,
}

impl Default for RegexPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl RegexPlugin {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(None), // Initialized in on_start
            config: Mutex::new(RegexConfig::default()),
            stats: Mutex::new(CacheStats::default()),
        }
    }

    fn init_cache(&self, size: usize) -> PluginResult<()> {
        let cache_size = NonZeroUsize::new(size)
            .ok_or_else(|| PluginError::HandlerError("Cache size must be > 0".to_string()))?;

        let mut cache = self.cache.lock().map_err(|_| {
            PluginError::HandlerError("Cache lock poisoned".to_string())
        })?;

        *cache = Some(LruCache::new(cache_size));
        Ok(())
    }

    fn get_or_compile(&self, pattern: &str) -> PluginResult<Regex> {
        let mut cache_guard = self.cache.lock().map_err(|_| {
            PluginError::HandlerError("Cache lock poisoned".to_string())
        })?;

        let cache = cache_guard.as_mut().ok_or_else(|| {
            PluginError::HandlerError("Cache not initialized".to_string())
        })?;

        let mut stats = self.stats.lock().map_err(|_| {
            PluginError::HandlerError("Stats lock poisoned".to_string())
        })?;

        stats.total_requests += 1;

        if let Some(regex) = cache.get(pattern) {
            stats.cache_hits += 1;
            tracing::trace!("Cache hit for pattern: {}", pattern);
            return Ok(regex.clone());
        }

        stats.cache_misses += 1;
        tracing::trace!("Cache miss for pattern: {}", pattern);

        let regex = Regex::new(pattern)
            .map_err(|e| PluginError::HandlerError(format!("Invalid regex: {}", e)))?;

        cache.put(pattern.to_string(), regex.clone());

        Ok(regex)
    }

    fn handle_match(&self, req: MatchRequest) -> PluginResult<MatchResponse> {
        tracing::debug!("Matching pattern '{}' against text", req.pattern);

        let regex = self.get_or_compile(&req.pattern)?;

        match regex.find(&req.text) {
            Some(m) => Ok(MatchResponse {
                matched: true,
                match_text: Some(m.as_str().to_string()),
                start: Some(m.start()),
                end: Some(m.end()),
            }),
            None => Ok(MatchResponse {
                matched: false,
                match_text: None,
                start: None,
                end: None,
            }),
        }
    }

    fn handle_find_all(&self, req: FindAllRequest) -> PluginResult<FindAllResponse> {
        tracing::debug!("Finding all matches for pattern '{}'", req.pattern);

        let regex = self.get_or_compile(&req.pattern)?;

        let matches: Vec<String> = regex
            .find_iter(&req.text)
            .map(|m| m.as_str().to_string())
            .collect();

        let count = matches.len();

        Ok(FindAllResponse { matches, count })
    }

    fn handle_stats(&self) -> PluginResult<StatsResponse> {
        let cache_guard = self.cache.lock().map_err(|_| {
            PluginError::HandlerError("Cache lock poisoned".to_string())
        })?;

        let cache = cache_guard.as_ref().ok_or_else(|| {
            PluginError::HandlerError("Cache not initialized".to_string())
        })?;

        let stats = self.stats.lock().map_err(|_| {
            PluginError::HandlerError("Stats lock poisoned".to_string())
        })?;

        Ok(StatsResponse {
            cached_patterns: cache.len(),
            cache_capacity: cache.cap().get(),
            total_requests: stats.total_requests,
            cache_hits: stats.cache_hits,
            cache_misses: stats.cache_misses,
        })
    }
}

#[async_trait]
impl Plugin for RegexPlugin {
    async fn on_start(&self, ctx: &PluginContext) -> PluginResult<()> {
        // Read configuration from context
        let config: RegexConfig = ctx.config()
            .map(|json| serde_json::from_value(json).unwrap_or_default())
            .unwrap_or_default();

        tracing::info!(
            "regex-plugin starting with cache size {}",
            config.cache_size
        );

        // Store config
        {
            let mut cfg = self.config.lock().map_err(|_| {
                PluginError::HandlerError("Config lock poisoned".to_string())
            })?;
            *cfg = config.clone();
        }

        // Initialize cache with configured size
        self.init_cache(config.cache_size)?;

        tracing::info!("regex-plugin started successfully");
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
                let resp = self.handle_match(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            "find_all" => {
                let req: FindAllRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_find_all(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            "stats" => {
                let resp = self.handle_stats()?;
                Ok(serde_json::to_vec(&resp)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        let stats = self.handle_stats()?;
        tracing::info!(
            "regex-plugin stopped: {} patterns cached, {}/{} requests from cache ({:.1}%)",
            stats.cached_patterns,
            stats.cache_hits,
            stats.total_requests,
            if stats.total_requests > 0 {
                (stats.cache_hits as f64 / stats.total_requests as f64) * 100.0
            } else {
                0.0
            }
        );
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["match", "find_all", "stats"]
    }
}

rustbridge_entry!(RegexPlugin::new);
pub use rustbridge::ffi_exports::*;
```

## Build the Final Plugin

```powershell
cargo build --release
```

## Create the Production Bundle

```powershell
rustbridge bundle create `
  --name regex-plugin `
  --version 1.0.0 `
  --lib windows-x86_64:target\release\regex_plugin.dll `
  --output regex-plugin-1.0.0.rbp
```

## Host Configuration Example

The host application provides configuration when loading the plugin:

```kotlin
// Kotlin example
val config = mapOf("cache_size" to 500)
val plugin = PluginLoader.load(bundlePath, config)
```

```csharp
// C# example
var config = new { cache_size = 500 };
var plugin = NativePluginLoader.Load(libraryPath, config);
```

```python
# Python example
config = {"cache_size": 500}
plugin = BundleLoader().load(bundle_path, config=config)
```

## Understanding Configuration

### Default Values

Serde's `#[serde(default = "...")]` provides defaults for missing fields:

```rust
#[serde(default = "default_cache_size")]
pub cache_size: usize,
```

### Graceful Fallback

If the host provides invalid or no configuration, the plugin uses defaults:

```rust
let config: RegexConfig = ctx.config()
    .map(|json| serde_json::from_value(json).unwrap_or_default())
    .unwrap_or_default();
```

### Validation

The plugin validates configuration at startup:

```rust
let cache_size = NonZeroUsize::new(size)
    .ok_or_else(|| PluginError::HandlerError("Cache size must be > 0".to_string()))?;
```

## Summary

You've built a complete regex plugin with:

1. **Message Types** - Structured request/response with serde
2. **Pattern Caching** - LRU cache for compiled regexes
3. **Statistics** - Cache hit/miss monitoring
4. **Configuration** - Host-provided settings

## What's Next?

Continue to [Chapter 2: Kotlin Consumer](../02-kotlin-consumer/README.md) to call this plugin from Kotlin using the FFM API.
