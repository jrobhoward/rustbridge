# Section 2: Validate Message

In this section, you'll implement a message type that validates whether a string contains valid JSON.

## Define the Message Types

Replace the echo message types in `src/lib.rs` with validation messages:

```rust
use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};

// ============================================================================
// Message Types
// ============================================================================

/// Request to validate a JSON string
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "validate")]
pub struct ValidateRequest {
    /// The string to validate as JSON
    pub json: String,
}

/// Response indicating whether the JSON is valid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResponse {
    /// True if the input is valid JSON
    pub valid: bool,
}
```

## Implement the Handler

Update the plugin implementation to handle validation requests:

```rust
// ============================================================================
// Plugin Implementation
// ============================================================================

#[derive(Default)]
pub struct JsonPlugin;

#[async_trait]
impl Plugin for JsonPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("json-plugin started");
        Ok(())
    }

    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        match type_tag {
            "validate" => {
                let req: ValidateRequest = serde_json::from_slice(payload)?;

                tracing::debug!(json_len = req.json.len(), "Validating JSON");

                // Try to parse the JSON - if it succeeds, it's valid
                let valid = serde_json::from_str::<serde_json::Value>(&req.json).is_ok();

                let response = ValidateResponse { valid };
                Ok(serde_json::to_vec(&response)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("json-plugin stopped");
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["validate"]
    }
}

// ============================================================================
// FFI Entry Point
// ============================================================================

rustbridge_entry!(JsonPlugin::default);
pub use rustbridge::ffi_exports::*;
```

## Add Tests

Replace the test module with validation tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validate___valid_json___returns_true() {
        let plugin = JsonPlugin;
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&ValidateRequest {
            json: r#"{"name": "test", "value": 42}"#.to_string(),
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "validate", &request).await.unwrap();
        let result: ValidateResponse = serde_json::from_slice(&response).unwrap();

        assert!(result.valid);
    }

    #[tokio::test]
    async fn validate___invalid_json___returns_false() {
        let plugin = JsonPlugin;
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&ValidateRequest {
            json: r#"{"name": "test" invalid}"#.to_string(),
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "validate", &request).await.unwrap();
        let result: ValidateResponse = serde_json::from_slice(&response).unwrap();

        assert!(!result.valid);
    }

    #[tokio::test]
    async fn validate___empty_object___returns_true() {
        let plugin = JsonPlugin;
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&ValidateRequest {
            json: "{}".to_string(),
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "validate", &request).await.unwrap();
        let result: ValidateResponse = serde_json::from_slice(&response).unwrap();

        assert!(result.valid);
    }
}
```

## Build and Test

```bash
cargo fmt
cargo build --release
cargo test
```

You should see:

```
running 3 tests
test tests::validate___empty_object___returns_true ... ok
test tests::validate___invalid_json___returns_false ... ok
test tests::validate___valid_json___returns_true ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## What's Next?

In the next section, we'll add a second message type for pretty-printing JSON.

[Continue to Section 3: Prettify Message →](./03-prettify-message.md)
