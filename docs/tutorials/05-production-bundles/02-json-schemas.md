# Section 2: JSON Schemas

In this section, you'll add JSON schema generation to your plugin and embed schemas in bundles.

## Why Embed Schemas?

Embedded schemas provide:

- **Validation**: Consumers can validate requests before sending
- **Documentation**: Schema files describe the plugin's API
- **Code Generation**: Generate type-safe clients from schemas
- **Tooling**: IDE support, schema explorers, API documentation

## Add Schema Generation to Your Plugin

We'll update the json-plugin from Chapter 3 to generate schemas automatically
using [schemars](https://docs.rs/schemars).

### Step 1: Add the schemars Dependency

```bash
cd ~/rustbridge-workspace/json-plugin
cargo add schemars
```

This adds to your `Cargo.toml`:

```toml
[dependencies]
schemars = "0.8"
```

### Step 2: Derive JsonSchema on Message Types

Update `src/lib.rs` to add `#[derive(JsonSchema)]` to your request and response types:

```rust
use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};
use schemars::JsonSchema;  // Add this import
use serde::Serialize;
use serde_json::ser::PrettyFormatter;

/// Request to validate a JSON string
#[derive(Debug, Clone, Serialize, Deserialize, Message, JsonSchema)]  // Add JsonSchema
#[message(tag = "validate")]
pub struct ValidateRequest {
    /// The string to validate as JSON
    pub json: String,
}

/// Response indicating whether the JSON is valid
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]  // Add JsonSchema
pub struct ValidateResponse {
    /// True if the input is valid JSON
    pub valid: bool,
}

/// Request to pretty-print a JSON string
#[derive(Debug, Clone, Serialize, Deserialize, Message, JsonSchema)]  // Add JsonSchema
#[message(tag = "prettify")]
pub struct PrettifyRequest {
    /// The JSON string to format
    pub json: String,
    /// Number of spaces for indentation (default: 2)
    #[serde(default = "default_indent")]
    pub indent: usize,
}

/// Response containing the formatted JSON
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]  // Add JsonSchema
pub struct PrettifyResponse {
    /// The pretty-printed JSON string
    pub result: String,
}

/// Default indentation for prettify (2 spaces)
fn default_indent() -> usize {
    2
}

// ... rest of implementation unchanged
```

The doc comments (`///`) become `description` fields in the generated schema.

### Step 3: Build and Verify

```bash
cargo build --release
```

## Generate Schema During Bundle Creation

Use `--generate-schema` to auto-generate the JSON Schema:

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 0.1.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --generate-schema src/lib.rs:messages.json \
  --sign-key ~/.rustbridge/signing.key \
  --output json-plugin-0.1.0.rbp
```

The `--generate-schema` flag:

- Parses your Rust source file
- Finds types with `#[derive(JsonSchema)]`
- Generates a JSON Schema file
- Embeds it in the bundle

## Verify Schema Inclusion

```bash
rustbridge bundle list json-plugin-0.1.0.rbp
```

```
json-plugin-0.1.0.rbp
├── manifest.json
├── manifest.json.minisig
├── lib/
│   └── linux-x86_64/
│       └── release/
│           ├── libjson_plugin.so
│           └── libjson_plugin.so.minisig
└── schema/
    └── messages.json                    ← Generated schema
```

## Generated Schema Example

The generated `messages.json` will look like:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "definitions": {
    "ValidateRequest": {
      "type": "object",
      "description": "Request to validate a JSON string",
      "properties": {
        "json": {
          "type": "string",
          "description": "The string to validate as JSON"
        }
      },
      "required": [
        "json"
      ]
    },
    "ValidateResponse": {
      "type": "object",
      "description": "Response indicating whether the JSON is valid",
      "properties": {
        "valid": {
          "type": "boolean",
          "description": "True if the input is valid JSON"
        }
      },
      "required": [
        "valid"
      ]
    },
    "PrettifyRequest": {
      "type": "object",
      "description": "Request to pretty-print a JSON string",
      "properties": {
        "json": {
          "type": "string",
          "description": "The JSON string to format"
        },
        "indent": {
          "type": "integer",
          "format": "uint",
          "description": "Number of spaces for indentation (default: 2)"
        }
      },
      "required": [
        "json"
      ]
    },
    "PrettifyResponse": {
      "type": "object",
      "description": "Response containing the formatted JSON",
      "properties": {
        "result": {
          "type": "string",
          "description": "The pretty-printed JSON string"
        }
      },
      "required": [
        "result"
      ]
    }
  }
}
```

## Extract Schema from Bundle

Consumers can extract schemas for tooling. Since `.rbp` bundles are ZIP archives, you can use standard tools:

```bash
# Extract just the schema file
unzip -j json-plugin-0.1.0.rbp "schema/*" -d ./schemas/

# Or extract everything
unzip json-plugin-0.1.0.rbp -d ./extracted/
```

Or programmatically:

### Java

```java
try (var loader = BundleLoader.builder()
        .bundlePath("json-plugin-0.1.0.rbp")
        .verifySignatures(false)  // Set true in production with public key
        .build()) {

    // Get schema as string
    String schema = loader.readSchema("messages.json");

    // Or extract to file
    Path schemaPath = loader.extractSchema("messages.json", Paths.get("./schemas/"));
}
```

### Python

```python
from rustbridge import BundleLoader

loader = BundleLoader(verify_signatures=False)  # Set True in production
schema = loader.read_schema("json-plugin-0.1.0.rbp", "messages.json")

# Or extract to file
schema_path = loader.extract_schema("json-plugin-0.1.0.rbp", "messages.json", "./schemas/")
```

## Using Schemas for Validation

### Java with networknt/json-schema-validator

Add the dependency to your `build.gradle.kts`:

```kotlin
dependencies {
    implementation("com.networknt:json-schema-validator:1.0.87")
}
```

Validate requests before calling the plugin:

```java
import com.networknt.schema.*;

// Load schema from bundle
String schemaJson = bundleLoader.getSchema("messages.json");
JsonSchema schema = JsonSchemaFactory
    .getInstance(SpecVersion.VersionFlag.V7)
    .getSchema(schemaJson);

// Validate a request
var request = new ValidateRequest("{\"test\": true}");
String requestJson = gson.toJson(request);

Set<ValidationMessage> errors = schema.validate(
    objectMapper.readTree(requestJson),
    "$defs/ValidateRequest"
);

if (errors.isEmpty()) {
    // Request is valid, send to plugin
    plugin.call("validate", requestJson);
} else {
    System.err.println("Validation errors: " + errors);
}
```

### Python with jsonschema

```python
import jsonschema
import json

bundle = BundleLoader("json-plugin-0.1.0.rbp")
schema = json.loads(bundle.get_schema("messages.json"))

# Validate a request
request = {"json": '{"test": true}'}
jsonschema.validate(
    request,
    schema["definitions"]["ValidateRequest"]
)
```

## Schema Compatibility

When combining bundles from different builds, rustbridge checks schema compatibility:

```bash
# Combining bundles with different schemas
rustbridge bundle combine \
  --output combined.rbp \
  linux-bundle.rbp macos-bundle.rbp

# Error: Schema checksum mismatch between bundles
```

Options for mismatched schemas:

```bash
# Warn but continue
rustbridge bundle combine --schema-mismatch warn ...

# Ignore schema differences
rustbridge bundle combine --schema-mismatch ignore ...
```

## Summary

You've learned to:

- Add `schemars` and `#[derive(JsonSchema)]` to message types
- Auto-generate schemas with `--generate-schema`
- Extract and use schemas in consumer applications
- Validate requests against schemas before calling plugins

## What's Next?

In the next section, you'll add build metadata for provenance tracking.

[Continue to Section 3: Build Metadata →](./03-build-metadata.md)
