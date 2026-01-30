# Section 3: Add LRU Caching

In this section, you'll add an LRU (Least Recently Used) cache to store compiled regex patterns, dramatically improving
performance for repeated patterns.

## Why LRU?

An LRU cache automatically evicts the least-recently-used entries when it reaches capacity. This is ideal for regex
patterns because:

1. **Bounded memory**: The cache won't grow unbounded
2. **Temporal locality**: Recently used patterns are likely to be used again
3. **Automatic cleanup**: Rarely-used patterns are evicted

## Add the LRU Dependency

Edit `Cargo.toml`:

```toml
[dependencies]
rustbridge = "0.6"
serde = { version = "1.0", features = ["derive"] }
regex = "1.10"

# Add this line
lru = "0.12"
```

## Update the Response Type

Add a `cached` field so callers know whether they got a cache hit:

```rust
/// Regex match response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResponse {
    /// Whether the pattern matched the text
    pub matches: bool,
    /// Whether the pattern was retrieved from cache
    pub cached: bool,
}
```

## Update the Plugin Struct

The plugin now needs state to hold the cache. We'll wrap it in a `Mutex` for thread safety:

```rust
use lru::LruCache;
use regex::Regex;
use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};
use std::num::NonZeroUsize;
use std::sync::Mutex;

// ... message types already changed ...

// ============================================================================
// Plugin Implementation
// ============================================================================

const DEFAULT_CACHE_SIZE: usize = 100;

/// Regex plugin with LRU caching
pub struct RegexPlugin {
    /// LRU cache of compiled regexes (pattern -> Regex)
    cache: Mutex<LruCache<String, Regex>>,
}

impl Default for RegexPlugin {
    fn default() -> Self {
        let cache_size = NonZeroUsize::new(DEFAULT_CACHE_SIZE).expect("default is non-zero");
        Self {
            cache: Mutex::new(LruCache::new(cache_size)),
        }
    }
}
```

## Implement Cache-Aware Matching

Update `handle_request`:

```rust
async fn handle_request(
    &self,
    _ctx: &PluginContext,
    type_tag: &str,
    payload: &[u8],
) -> PluginResult<Vec<u8>> {
    match type_tag {
        "match" => {
            let req: MatchRequest = serde_json::from_slice(payload)?;

            // Lock the cache
            let mut cache = self.cache.lock().expect("cache lock poisoned");

            // Check if regex is in cache
            let (regex, cached) = if let Some(regex) = cache.get(&req.pattern) {
                (regex, true)
            } else {
                // Compile the regex and add it to the cache
                let regex = Regex::new(&req.pattern).map_err(|e| {
                    PluginError::HandlerError(format!("Invalid regex pattern: {}", e))
                })?;
                cache.put(req.pattern.clone(), regex);
                (cache.get(&req.pattern).expect("just inserted"), false)
            };

            let matches = regex.is_match(&req.text);

            let response = MatchResponse { matches, cached };
            Ok(serde_json::to_vec(&response)?)
        }
        _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
    }
}
```

## Add Logging

Add some tracing to see what's happening:

```rust
async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
    tracing::info!("regex-plugin started with cache size: {}", DEFAULT_CACHE_SIZE);
    Ok(())
}

async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
    let cache = self.cache.lock().expect("cache lock poisoned");
    tracing::info!(
        cached_patterns = cache.len(),
        "regex-plugin stopped"
    );
    Ok(())
}
```

And in `handle_request`, after `regex.is_match()` is called:

```rust
tracing::debug!(
    pattern = %req.pattern,
    matches,
    cached,
    "Match completed"
);
```

## Update Tests

Add tests to check the `cached` field:

```rust
#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handle_request___first_request___not_cached() {
        let plugin = RegexPlugin::default();
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&MatchRequest {
            pattern: r"\d+".to_string(),
            text: "test123".to_string(),
        })
            .unwrap();

        let response = plugin.handle_request(&ctx, "match", &request).await.unwrap();
        let match_response: MatchResponse = serde_json::from_slice(&response).unwrap();

        assert!(match_response.matches);
        assert!(!match_response.cached);  // First time, not cached
    }

    #[tokio::test]
    async fn handle_request___same_pattern_twice___second_is_cached() {
        let plugin = RegexPlugin::default();
        let ctx = PluginContext::new(PluginConfig::default());

        // First request
        let request = serde_json::to_vec(&MatchRequest {
            pattern: r"\d+".to_string(),
            text: "test123".to_string(),
        })
            .unwrap();

        let _ = plugin.handle_request(&ctx, "match", &request).await.unwrap();

        // Second request with same pattern
        let request = serde_json::to_vec(&MatchRequest {
            pattern: r"\d+".to_string(),
            text: "456".to_string(),
        })
            .unwrap();

        let response = plugin.handle_request(&ctx, "match", &request).await.unwrap();
        let match_response: MatchResponse = serde_json::from_slice(&response).unwrap();

        assert!(match_response.matches);
        assert!(match_response.cached);  // Second time, from cache!
    }
}
```

## Benchmark Cache Effectiveness

Add a benchmark to compare small vs. large cache performance:

```rust
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

    // Create a plugin with the default cache
    let plugin = RegexPlugin::default();
    let ctx = PluginContext::new(PluginConfig::default());

    // Warm up the cache
    for pattern in &patterns {
        let request = serde_json::to_vec(&MatchRequest {
            pattern: pattern.to_string(),
            text: sample.to_string(),
        })
            .unwrap();
        let _ = plugin.handle_request(&ctx, "match", &request).await;
    }

    // Benchmark with cache hits
    let start = Instant::now();
    for _ in 0..iterations {
        for pattern in &patterns {
            let request = serde_json::to_vec(&MatchRequest {
                pattern: pattern.to_string(),
                text: sample.to_string(),
            })
                .unwrap();
            let _ = plugin.handle_request(&ctx, "match", &request).await;
        }
    }
    let elapsed = start.elapsed();

    println!("\n=== Cache Effectiveness ===");
    println!("Patterns: {}, Iterations: {}", patterns.len(), iterations);
    println!("Total time: {:?}", elapsed);
    println!(
        "Per request: {:?}",
        elapsed / (patterns.len() * iterations) as u32
    );
}
```

Run with:

```bash
cargo test bench___cache_effectiveness -- --nocapture
```

You should see significant improvement over the uncached version.
On my dev machine the per-request duration dropped from ~1ms per request to ~12µs.

## Build and Test

```bash
cargo test
```

All tests should pass.

## What's Next?

The cache size is hardcoded to 100. In the next section, we'll make it configurable so hosts can tune it for their
workload.

[Continue to Section 4: Make It Configurable →](./04-configuration.md)
