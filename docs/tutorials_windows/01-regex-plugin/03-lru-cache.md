# Section 3: LRU Cache

In this section, you'll add an LRU (Least Recently Used) cache to avoid recompiling regex patterns.

## Add the LRU Dependency

Update `Cargo.toml`:

```toml
[dependencies]
rustbridge = { version = "0.7.0" }
regex = "1.10"
lru = "0.12"
```

## Update the Plugin State

Modify `src\lib.rs` to add a cache:

```rust
//! regex-plugin - A regex engine with LRU caching

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

/// Request cache statistics
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "stats")]
pub struct StatsRequest;

/// Cache statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatsResponse {
    /// Current number of cached patterns
    pub cached_patterns: usize,
    /// Maximum cache capacity
    pub cache_capacity: usize,
    /// Total requests served
    pub total_requests: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
}

// ============================================================================
// Plugin Implementation
// ============================================================================

pub struct RegexPlugin {
    cache: Mutex<LruCache<String, Regex>>,
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
    const DEFAULT_CACHE_SIZE: usize = 100;

    pub fn new() -> Self {
        let cache_size = NonZeroUsize::new(Self::DEFAULT_CACHE_SIZE)
            .expect("cache size must be non-zero");

        Self {
            cache: Mutex::new(LruCache::new(cache_size)),
            stats: Mutex::new(CacheStats::default()),
        }
    }

    fn get_or_compile(&self, pattern: &str) -> PluginResult<Regex> {
        // Lock cache briefly to check/update
        let mut cache = self.cache.lock().map_err(|_| {
            PluginError::HandlerError("Cache lock poisoned".to_string())
        })?;

        let mut stats = self.stats.lock().map_err(|_| {
            PluginError::HandlerError("Stats lock poisoned".to_string())
        })?;

        stats.total_requests += 1;

        // Check if pattern is cached
        if let Some(regex) = cache.get(pattern) {
            stats.cache_hits += 1;
            tracing::trace!("Cache hit for pattern: {}", pattern);
            return Ok(regex.clone());
        }

        stats.cache_misses += 1;
        tracing::trace!("Cache miss for pattern: {}", pattern);

        // Compile and cache
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
        let cache = self.cache.lock().map_err(|_| {
            PluginError::HandlerError("Cache lock poisoned".to_string())
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
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!(
            "regex-plugin started with cache capacity {}",
            Self::DEFAULT_CACHE_SIZE
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

## Build and Test

```powershell
cargo build --release
```

## Understanding the Implementation

### LRU Cache

The LRU cache evicts the least-recently-used entries when full:

```rust
cache: Mutex<LruCache<String, Regex>>
```

We use `Mutex` for thread-safe access from multiple host threads.

### Lock Safety

Locks are held briefly and released before any external calls:

```rust
fn get_or_compile(&self, pattern: &str) -> PluginResult<Regex> {
    let mut cache = self.cache.lock()?;
    // ... quick check/update ...
    Ok(regex.clone())  // Clone so we can release the lock
}
```

The cloned `Regex` is returned so the lock is released before the caller uses it.

### Statistics

The `stats` message lets hosts monitor cache effectiveness:

```json
{
  "cached_patterns": 42,
  "cache_capacity": 100,
  "total_requests": 1000,
  "cache_hits": 850,
  "cache_misses": 150
}
```

## What's Next?

In the next section, you'll make the cache size configurable from the host.

[Continue to Section 4: Configuration →](./04-configuration.md)
