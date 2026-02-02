# Section 3: Prettify Message

In this section, you'll add a second message type that formats JSON with indentation.

## Add the Message Types

Add the prettify request and response types after the validate types in `src/lib.rs`:

```rust
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
```

## Update the Handler

Add the prettify handler to the `match` statement:

```rust
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

            // Parse the JSON
            let value: serde_json::Value = serde_json::from_str(&req.json)
                .map_err(|e| PluginError::HandlerError(format!("Invalid JSON: {}", e)))?;

            // Format with the requested indentation
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
```

## Add the Import

Add the `PrettyFormatter` import at the top of the file:

```rust
use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};
use serde::Serialize;
use serde_json::ser::PrettyFormatter;
```

## Update Supported Types

Update the `supported_types` method:

```rust
fn supported_types(&self) -> Vec<&'static str> {
    vec!["validate", "prettify"]
}
```

## Add Tests

Add tests for the prettify functionality:

```rust
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
    assert!(result.result.contains("  ")); // 2-space indent
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

    assert!(result.result.contains("    ")); // 4-space indent
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
```

## Build and Test

```bash
cargo fmt
cargo build --release
cargo test
```

You should see all 6 tests passing:

```
running 6 tests
test tests::prettify___compact_json___returns_formatted ... ok
test tests::prettify___nested_object___formats_correctly ... ok
test tests::prettify___with_custom_indent___uses_indent ... ok
test tests::validate___empty_object___returns_true ... ok
test tests::validate___invalid_json___returns_false ... ok
test tests::validate___valid_json___returns_true ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## What's Next?

In the next section, we'll improve error handling for invalid JSON input.

[Continue to Section 4: Error Handling →](./04-error-handling.md)
