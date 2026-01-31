# Section 2: Basic Matching

In this section, you'll implement the core regex matching functionality.

## Add the Regex Dependency

Edit `Cargo.toml` to add the `regex` crate:

```toml
[package]
name = "regex-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
rustbridge = { version = "0.7.0" }
regex = "1.10"

[profile.release]
lto = true
codegen-units = 1
```

## Define Message Types

Replace the echo message types in `src\lib.rs` with regex-specific types:

```rust
//! regex-plugin - A regex engine with caching

use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};
use regex::Regex;

// ============================================================================
// Message Types
// ============================================================================

/// Request to match a pattern against text
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "match")]
pub struct MatchRequest {
    /// The regex pattern to compile and match
    pub pattern: String,
    /// The text to search
    pub text: String,
}

/// Response from a match request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchResponse {
    /// Whether the pattern matched
    pub matched: bool,
    /// The matched substring (if any)
    pub match_text: Option<String>,
    /// Start position of match
    pub start: Option<usize>,
    /// End position of match
    pub end: Option<usize>,
}

/// Request to find all matches
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "find_all")]
pub struct FindAllRequest {
    pub pattern: String,
    pub text: String,
}

/// Response with all matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindAllResponse {
    /// All matched substrings
    pub matches: Vec<String>,
    /// Count of matches
    pub count: usize,
}
```

## Implement the Handlers

Update the plugin implementation:

```rust
// ============================================================================
// Plugin Implementation
// ============================================================================

#[derive(Default)]
pub struct RegexPlugin;

impl RegexPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_match(&self, req: MatchRequest) -> PluginResult<MatchResponse> {
        tracing::debug!("Matching pattern '{}' against text", req.pattern);

        // Compile the regex (we'll optimize this later)
        let regex = Regex::new(&req.pattern)
            .map_err(|e| PluginError::HandlerError(format!("Invalid regex: {}", e)))?;

        // Find the first match
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

        let regex = Regex::new(&req.pattern)
            .map_err(|e| PluginError::HandlerError(format!("Invalid regex: {}", e)))?;

        let matches: Vec<String> = regex
            .find_iter(&req.text)
            .map(|m| m.as_str().to_string())
            .collect();

        let count = matches.len();

        Ok(FindAllResponse { matches, count })
    }
}

#[async_trait]
impl Plugin for RegexPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("regex-plugin started");
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
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("regex-plugin stopped");
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["match", "find_all"]
    }
}

// Generate the FFI entry point
rustbridge_entry!(RegexPlugin::new);
pub use rustbridge::ffi_exports::*;
```

## Build and Test

```powershell
cargo build --release
```

If compilation succeeds, update the bundle:

```powershell
rustbridge bundle create `
  --name regex-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\regex_plugin.dll `
  --output regex-plugin-0.1.0.rbp
```

## Understanding the Code

### Error Handling

Invalid regex patterns are converted to `PluginError::HandlerError`:

```rust
let regex = Regex::new(&req.pattern)
    .map_err(|e| PluginError::HandlerError(format!("Invalid regex: {}", e)))?;
```

This returns a structured error to the host rather than panicking.

### Message Routing

The `handle_request` method routes by `type_tag`:

```rust
match type_tag {
    "match" => { /* ... */ }
    "find_all" => { /* ... */ }
    _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
}
```

Unknown message types return an error rather than silently failing.

## Current Limitation

Currently, we compile the regex on every request. For frequently-used patterns, this is inefficient. In the next section, we'll add an LRU cache to reuse compiled patterns.

## What's Next?

[Continue to Section 3: LRU Cache →](./03-lru-cache.md)
