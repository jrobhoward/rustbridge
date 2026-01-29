# Section 2: Basic Regex Matching

In this section, you'll replace the echo functionality with regex pattern matching.

## Add the Regex Dependency

Edit `Cargo.toml` to add the `regex` crate:

```toml
[dependencies]
rustbridge = "0.6"
serde = { version = "1.0", features = ["derive"] }

# Add this line
regex = "1.10"
```

## Define the Message Types

Replace the echo types in `src/lib.rs` with regex match types:

```rust
use regex::Regex;
use rustbridge::prelude::*;
use rustbridge::serde_json;

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
}
```

## Update the Plugin Struct

For now, we'll keep the plugin stateless. Later we'll add a cache.

```rust
// ============================================================================
// Plugin Implementation
// ============================================================================

/// Regex plugin - compiles patterns on each request
#[derive(Default)]
pub struct RegexPlugin;
```

## Implement Request Handling

Update the `handle_request` method:

```rust
#[async_trait]
impl Plugin for RegexPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
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
                // Deserialize the request
                let req: MatchRequest = serde_json::from_slice(payload)?;

                // Compile the regex (this can fail for invalid patterns)
                let regex = Regex::new(&req.pattern).map_err(|e| {
                    PluginError::HandlerError(format!("Invalid regex pattern: {}", e))
                })?;

                // Check if it matches
                let matches = regex.is_match(&req.text);

                // Build and serialize the response
                let response = MatchResponse { matches };
                Ok(serde_json::to_vec(&response)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["match"]
    }
}
```

## Update the FFI Entry Point

```rust
// ============================================================================
// FFI Entry Point
// ============================================================================

rustbridge_entry!(RegexPlugin::default);
pub use rustbridge::ffi_exports::*;
```

## Write Tests

Replace the test module:

```rust
#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handle_request___matching_pattern___returns_true() {
        let plugin = RegexPlugin::default();
        let ctx = PluginContext::new(PluginConfig::default());

        plugin.on_start(&ctx).await.unwrap();

        let request = serde_json::to_vec(&MatchRequest {
            pattern: r"\d+".to_string(),
            text: "test123".to_string(),
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "match", &request).await.unwrap();
        let match_response: MatchResponse = serde_json::from_slice(&response).unwrap();

        assert!(match_response.matches);
    }

    #[tokio::test]
    async fn handle_request___non_matching_pattern___returns_false() {
        let plugin = RegexPlugin::default();
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&MatchRequest {
            pattern: r"^\d+$".to_string(),  // Must be ALL digits
            text: "test123".to_string(),     // Has letters
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "match", &request).await.unwrap();
        let match_response: MatchResponse = serde_json::from_slice(&response).unwrap();

        assert!(!match_response.matches);
    }

    #[tokio::test]
    async fn handle_request___invalid_regex___returns_error() {
        let plugin = RegexPlugin::default();
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&MatchRequest {
            pattern: r"[invalid".to_string(),  // Unclosed bracket
            text: "test".to_string(),
        })
        .unwrap();

        let result = plugin.handle_request(&ctx, "match", &request).await;

        assert!(result.is_err());
    }
}
```

## Build and Test

```bash
cargo test
```

You should see:

```
running 3 tests
test tests::handle_request___matching_pattern___returns_true ... ok
test tests::handle_request___non_matching_pattern___returns_false ... ok
test tests::handle_request___invalid_regex___returns_error ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## The Problem: Recompilation

Every request compiles the regex pattern fresh. For repeated patterns, this is wasteful. Regex compilation is expensive - often the slowest part of matching.

Let's measure it:

```rust
#[tokio::test]
async fn bench___compilation_overhead() {
    use std::time::Instant;

    let plugin = RegexPlugin::default();
    let ctx = PluginContext::new(PluginConfig::default());

    let pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";  // Email regex
    let text = "user@example.com";
    let iterations = 1000;

    let request = serde_json::to_vec(&MatchRequest {
        pattern: pattern.to_string(),
        text: text.to_string(),
    })
    .unwrap();

    let start = Instant::now();
    for _ in 0..iterations {
        let _ = plugin.handle_request(&ctx, "match", &request).await;
    }
    let elapsed = start.elapsed();

    println!("\n=== Compilation Overhead ===");
    println!("Iterations: {}", iterations);
    println!("Total time: {:?}", elapsed);
    println!("Per request: {:?}", elapsed / iterations as u32);
}
```

Run with `--nocapture` to see output:

```bash
cargo test bench___compilation_overhead -- --nocapture
```

You'll see each request takes ~10-50 microseconds, mostly spent compiling.

## What's Next?

In the next section, we'll add an LRU cache to avoid recompiling the same patterns.

[Continue to Section 3: Add LRU Caching →](./03-lru-cache.md)
