# Section 4: Error Handling

In this section, you'll improve error handling with structured error types.

## Complete Plugin Implementation

Here's the complete `src\lib.rs` with proper error handling:

```rust
//! json-plugin - JSON validation and formatting

use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};
use serde::Serialize;

// ============================================================================
// Message Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "validate")]
pub struct ValidateRequest {
    pub json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub error: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "prettify")]
pub struct PrettifyRequest {
    pub json: String,
    #[serde(default = "default_indent")]
    pub indent: usize,
}

fn default_indent() -> usize {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrettifyResponse {
    pub json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "minify")]
pub struct MinifyRequest {
    pub json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinifyResponse {
    pub json: String,
    pub bytes_saved: usize,
}

// ============================================================================
// Plugin Implementation
// ============================================================================

#[derive(Default)]
pub struct JsonPlugin;

impl JsonPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_validate(&self, req: ValidateRequest) -> PluginResult<ValidateResponse> {
        tracing::debug!("Validating JSON ({} bytes)", req.json.len());

        match serde_json::from_str::<serde_json::Value>(&req.json) {
            Ok(_) => {
                tracing::debug!("JSON is valid");
                Ok(ValidateResponse {
                    valid: true,
                    error: None,
                    line: None,
                    column: None,
                })
            }
            Err(e) => {
                tracing::debug!("JSON is invalid: {}", e);
                Ok(ValidateResponse {
                    valid: false,
                    error: Some(e.to_string()),
                    line: Some(e.line()),
                    column: Some(e.column()),
                })
            }
        }
    }

    fn handle_prettify(&self, req: PrettifyRequest) -> PluginResult<PrettifyResponse> {
        tracing::debug!("Prettifying JSON with indent={}", req.indent);

        // Validate indent range
        if req.indent > 8 {
            return Err(PluginError::HandlerError(
                "Indent must be 0-8 spaces".to_string()
            ));
        }

        let value: serde_json::Value = serde_json::from_str(&req.json)
            .map_err(|e| PluginError::HandlerError(format!("Invalid JSON: {}", e)))?;

        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(
            &" ".repeat(req.indent).into_bytes()
        );
        let mut serializer = serde_json::Serializer::with_formatter(&mut buf, formatter);

        value.serialize(&mut serializer)
            .map_err(|e| PluginError::HandlerError(format!("Serialization failed: {}", e)))?;

        let json = String::from_utf8(buf)
            .map_err(|e| PluginError::HandlerError(format!("UTF-8 error: {}", e)))?;

        Ok(PrettifyResponse { json })
    }

    fn handle_minify(&self, req: MinifyRequest) -> PluginResult<MinifyResponse> {
        tracing::debug!("Minifying JSON ({} bytes)", req.json.len());

        let value: serde_json::Value = serde_json::from_str(&req.json)
            .map_err(|e| PluginError::HandlerError(format!("Invalid JSON: {}", e)))?;

        let minified = serde_json::to_string(&value)
            .map_err(|e| PluginError::HandlerError(format!("Serialization failed: {}", e)))?;

        let bytes_saved = req.json.len().saturating_sub(minified.len());

        Ok(MinifyResponse {
            json: minified,
            bytes_saved,
        })
    }
}

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
                let resp = self.handle_validate(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            "prettify" => {
                let req: PrettifyRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_prettify(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            "minify" => {
                let req: MinifyRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_minify(req)?;
                Ok(serde_json::to_vec(&resp)?)
            }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        tracing::info!("json-plugin stopped");
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["validate", "prettify", "minify"]
    }
}

rustbridge_entry!(JsonPlugin::new);
pub use rustbridge::ffi_exports::*;
```

## Build the Final Plugin

```powershell
cargo build --release
```

## Create the Bundle

```powershell
rustbridge bundle create `
  --name json-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --output json-plugin-0.1.0.rbp
```

## Error Handling Patterns

### Validation vs Handler Errors

- **Validation errors** return success with error details (user input issues)
- **Handler errors** return `PluginError` (programming/system issues)

```rust
// Validation: return structured response
Ok(ValidateResponse {
    valid: false,
    error: Some(e.to_string()),
    ...
})

// Handler error: return error
Err(PluginError::HandlerError("Indent must be 0-8 spaces".to_string()))
```

### Input Validation

Validate inputs before processing:

```rust
if req.indent > 8 {
    return Err(PluginError::HandlerError(
        "Indent must be 0-8 spaces".to_string()
    ));
}
```

## Summary

You've built a JSON plugin with:

1. **Validate** - Check JSON syntax with error positions
2. **Prettify** - Format JSON with configurable indentation
3. **Minify** - Compact JSON and report size savings
4. **Error Handling** - Structured errors for user input, exceptions for system issues

## What's Next?

Continue to [Chapter 4: Java Consumer](../04-java-consumer/README.md) to call this plugin from Java.
