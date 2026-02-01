# Section 3: Prettify Message

In this section, you'll add the prettify and minify handlers.

## Add Prettify Handler

Update `src\lib.rs` to add the new handlers:

```rust
impl JsonPlugin {
    // ... existing code ...

    fn handle_prettify(&self, req: PrettifyRequest) -> PluginResult<PrettifyResponse> {
        tracing::debug!("Prettifying JSON with indent={}", req.indent);

        // Parse the JSON
        let value: serde_json::Value = serde_json::from_str(&req.json)
            .map_err(|e| PluginError::HandlerError(format!("Invalid JSON: {}", e)))?;

        // Create custom formatter with requested indent
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

        // Parse and re-serialize without whitespace
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
```

## Update Request Router

Add the new message types to `handle_request`:

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

fn supported_types(&self) -> Vec<&'static str> {
    vec!["validate", "prettify", "minify"]
}
```

## Add the Missing Import

At the top of the file, add:

```rust
use serde::Serialize;
```

## Build

```powershell
cargo build --release
```

## Update the Bundle

```powershell
rustbridge bundle create `
  --name json-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --output json-plugin-0.1.0.rbp
```

## What's Next?

In the next section, you'll improve error handling.

[Continue to Section 4: Error Handling →](./04-error-handling.md)
