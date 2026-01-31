# Section 2: JSON Schemas

In this section, you'll generate and embed JSON schemas for your message types.

## Why Schemas?

JSON schemas provide:
- **Documentation** - Human-readable API documentation
- **Validation** - Consumers can validate requests before sending
- **Code generation** - Generate client types in any language
- **IDE support** - Autocomplete and validation in editors

## Add Schema Generation

Update `Cargo.toml` to enable schema generation:

```toml
[dependencies]
rustbridge = { version = "0.7.0", features = ["schema"] }
schemars = "0.8"
```

## Derive JsonSchema

Update your message types in `src\lib.rs`:

```rust
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Message)]
#[message(tag = "validate")]
pub struct ValidateRequest {
    /// The JSON string to validate
    pub json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidateResponse {
    /// Whether the JSON is valid
    pub valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
    /// Line number of error (1-indexed)
    pub line: Option<usize>,
    /// Column number of error (1-indexed)
    pub column: Option<usize>,
}

// ... add JsonSchema to other types ...
```

## Generate Schema Files

```powershell
# Build with schema feature
cargo build --release

# Generate schemas
rustbridge schema generate `
  --crate . `
  --output schemas\
```

This creates:

```
schemas\
├── validate_request.json
├── validate_response.json
├── prettify_request.json
├── prettify_response.json
├── minify_request.json
└── minify_response.json
```

## Example Schema

`schemas\validate_request.json`:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "ValidateRequest",
  "description": "Request to validate JSON",
  "type": "object",
  "required": ["json"],
  "properties": {
    "json": {
      "type": "string",
      "description": "The JSON string to validate"
    }
  }
}
```

## Embed Schemas in Bundle

```powershell
rustbridge bundle create `
  --name json-plugin `
  --version 1.0.0 `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --schemas schemas\ `
  --sign-key $env:USERPROFILE\.rustbridge\signing.key `
  --output json-plugin-1.0.0.rbp
```

## List Bundle with Schemas

```powershell
rustbridge bundle list json-plugin-1.0.0.rbp
```

```
json-plugin-1.0.0.rbp
├── manifest.json
├── manifest.json.minisig
├── schemas\
│   ├── validate_request.json
│   ├── validate_response.json
│   └── ...
└── lib\
    └── windows-x86_64\
        └── ...
```

## Using Schemas in Consumers

Extract and use schemas for validation:

```kotlin
// Kotlin with everit-json-schema
val bundleLoader = BundleLoader.builder()
    .bundlePath("json-plugin-1.0.0.rbp")
    .build()

val schemaPath = bundleLoader.extractSchema("validate_request.json")
val schema = SchemaLoader.load(JSONObject(schemaPath.readText()))

// Validate before sending
schema.validate(JSONObject(requestJson))
```

## What's Next?

In the next section, you'll add build metadata.

[Continue to Section 3: Build Metadata →](./03-build-metadata.md)
