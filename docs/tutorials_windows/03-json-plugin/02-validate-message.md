# Section 2: Validate Message

In this section, you'll implement the JSON validation handler.

## Add the Plugin Implementation

Continue editing `src\lib.rs`:

```rust
//! json-plugin - JSON validation and formatting

use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};

// ... (message types from previous section) ...

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

rustbridge_entry!(JsonPlugin::new);
pub use rustbridge::ffi_exports::*;
```

## Build and Test

```powershell
cargo build --release
```

## Test Validation Manually

Create a quick test bundle:

```powershell
rustbridge bundle create `
  --name json-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --output json-plugin-0.1.0.rbp
```

## Understanding Error Reporting

The `serde_json` error includes position information:

```rust
Err(e) => {
    Ok(ValidateResponse {
        valid: false,
        error: Some(e.to_string()),
        line: Some(e.line()),    // 1-indexed line
        column: Some(e.column()), // 1-indexed column
    })
}
```

Example error response for `{"name": "test",}` (trailing comma):

```json
{
  "valid": false,
  "error": "trailing comma at line 1 column 17",
  "line": 1,
  "column": 17
}
```

## What's Next?

In the next section, you'll add prettify and minify handlers.

[Continue to Section 3: Prettify Message →](./03-prettify-message.md)
