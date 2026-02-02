# Section 4: Error Handling

In this section, we'll ensure the plugin returns meaningful errors when given invalid JSON.

## Current Behavior

The `prettify` handler already returns an error for invalid JSON:

```rust
let value: serde_json::Value = serde_json::from_str(&req.json)
    .map_err(|e| PluginError::HandlerError(format!("Invalid JSON: {}", e)))?;
```

This converts the serde error into a `PluginError::HandlerError` with a descriptive message.

## Add an Error Test

Add a test to verify the error handling works:

```rust
#[tokio::test]
async fn prettify___invalid_json___returns_error() {
    let plugin = JsonPlugin;
    let ctx = PluginContext::new(PluginConfig::default());

    let request = serde_json::to_vec(&PrettifyRequest {
        json: r#"{"broken": }"#.to_string(),
        indent: 2,
    })
    .unwrap();

    let result = plugin.handle_request(&ctx, "prettify", &request).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Invalid JSON"));
}
```

## Validate vs Prettify: Different Error Strategies

Notice the two message types handle invalid JSON differently:

| Message | Invalid JSON Behavior |
|---------|----------------------|
| `validate` | Returns `{ "valid": false }` |
| `prettify` | Returns an error |

This is intentional:
- **Validation** is asking "is this valid?" — the answer is simply "no"
- **Prettifying** requires valid JSON to work — invalid input is an error condition

## Build and Test

```bash
cargo fmt
cargo test
```

All 7 tests should pass.

## Complete Source Code

Here's the final `src/lib.rs`:

```rust
//! json-plugin - A rustbridge plugin for JSON validation and formatting

use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};
use serde::Serialize;
use serde_json::ser::PrettyFormatter;

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

/// Request to pretty-print a JSON string
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "prettify")]
pub struct PrettifyRequest {
    /// The JSON string to format
    pub json: String,
    /// Number of spaces for indentation (default: 2)
    #[serde(default = "default_indent")]
    pub indent: usize,
}

fn default_indent() -> usize {
    2
}

/// Response containing the formatted JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrettifyResponse {
    /// The pretty-printed JSON string
    pub result: String,
}

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

                let valid = serde_json::from_str::<serde_json::Value>(&req.json).is_ok();

                let response = ValidateResponse { valid };
                Ok(serde_json::to_vec(&response)?)
            }
            "prettify" => {
                let req: PrettifyRequest = serde_json::from_slice(payload)?;
                tracing::debug!(json_len = req.json.len(), indent = req.indent, "Prettifying JSON");

                let value: serde_json::Value = serde_json::from_str(&req.json)
                    .map_err(|e| PluginError::HandlerError(format!("Invalid JSON: {}", e)))?;

                let indent_bytes = " ".repeat(req.indent).into_bytes();
                let formatter = PrettyFormatter::with_indent(&indent_bytes);
                let mut buf = Vec::new();
                let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);
                value.serialize(&mut serializer)?;

                let result = String::from_utf8(buf)
                    .map_err(|e| PluginError::HandlerError(format!("UTF-8 error: {}", e)))?;

                let response = PrettifyResponse { result };
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
        vec!["validate", "prettify"]
    }
}

// ============================================================================
// FFI Entry Point
// ============================================================================

rustbridge_entry!(JsonPlugin::default);
pub use rustbridge::ffi_exports::*;

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
#[allow(non_snake_case)]
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

    #[tokio::test]
    async fn prettify___compact_json___returns_formatted() {
        let plugin = JsonPlugin;
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&PrettifyRequest {
            json: r#"{"name":"test","value":42}"#.to_string(),
            indent: 2,
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "prettify", &request).await.unwrap();
        let result: PrettifyResponse = serde_json::from_slice(&response).unwrap();

        assert!(result.result.contains('\n'));
        assert!(result.result.contains("  "));
    }

    #[tokio::test]
    async fn prettify___with_custom_indent___uses_indent() {
        let plugin = JsonPlugin;
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&PrettifyRequest {
            json: r#"{"a":1}"#.to_string(),
            indent: 4,
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "prettify", &request).await.unwrap();
        let result: PrettifyResponse = serde_json::from_slice(&response).unwrap();

        assert!(result.result.contains("    "));
    }

    #[tokio::test]
    async fn prettify___nested_object___formats_correctly() {
        let plugin = JsonPlugin;
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&PrettifyRequest {
            json: r#"{"outer":{"inner":"value"}}"#.to_string(),
            indent: 2,
        })
        .unwrap();

        let response = plugin.handle_request(&ctx, "prettify", &request).await.unwrap();
        let result: PrettifyResponse = serde_json::from_slice(&response).unwrap();

        let expected = r#"{
  "outer": {
    "inner": "value"
  }
}"#;
        assert_eq!(result.result, expected);
    }

    #[tokio::test]
    async fn prettify___invalid_json___returns_error() {
        let plugin = JsonPlugin;
        let ctx = PluginContext::new(PluginConfig::default());

        let request = serde_json::to_vec(&PrettifyRequest {
            json: r#"{"broken": }"#.to_string(),
            indent: 2,
        })
        .unwrap();

        let result = plugin.handle_request(&ctx, "prettify", &request).await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid JSON"));
    }
}
```

## Create the Bundle

Build the release version and create a bundle for use in Chapter 4:

```bash
cargo build --release

# Linux
rustbridge bundle create \
  --name json-plugin \
  --version 0.1.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --output json-plugin-0.1.0.rbp

# macOS (Apple Silicon)
rustbridge bundle create \
  --name json-plugin \
  --version 0.1.0 \
  --lib darwin-aarch64:target/release/libjson_plugin.dylib \
  --output json-plugin-0.1.0.rbp

# Windows
rustbridge bundle create \
  --name json-plugin \
  --version 0.1.0 \
  --lib windows-x86_64:target/release/json_plugin.dll \
  --output json-plugin-0.1.0.rbp
```

Verify the bundle:

```bash
rustbridge bundle list json-plugin-0.1.0.rbp
```

## Summary

You've built a JSON plugin with two message types:

| Message | Purpose | Invalid JSON |
|---------|---------|--------------|
| `validate` | Check if string is valid JSON | Returns `{ "valid": false }` |
| `prettify` | Format JSON with indentation | Returns error |

## Next Steps

Continue to [Chapter 4: Calling from Java](../04-java-consumer/README.md) to call your plugin from a Java application using FFM.
