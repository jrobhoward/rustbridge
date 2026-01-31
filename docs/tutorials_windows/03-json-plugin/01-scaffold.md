# Section 1: Project Scaffold

In this section, you'll create the JSON plugin project.

## Create the Project

```powershell
cd $env:USERPROFILE\rustbridge-workspace

rustbridge new json-plugin
cd json-plugin
```

## Update Cargo.toml

```toml
[package]
name = "json-plugin"
version = "1.0.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
rustbridge = { version = "0.7.0" }

[profile.release]
lto = true
codegen-units = 1
```

## Define Message Types

Replace `src\lib.rs`:

```rust
//! json-plugin - JSON validation and formatting

use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};

// ============================================================================
// Message Types
// ============================================================================

/// Request to validate JSON
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "validate")]
pub struct ValidateRequest {
    /// The JSON string to validate
    pub json: String,
}

/// Response from validation
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Request to prettify JSON
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "prettify")]
pub struct PrettifyRequest {
    /// The JSON string to prettify
    pub json: String,
    /// Number of spaces for indentation (default: 2)
    #[serde(default = "default_indent")]
    pub indent: usize,
}

fn default_indent() -> usize {
    2
}

/// Response from prettify
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrettifyResponse {
    /// The prettified JSON
    pub json: String,
}

/// Request to minify JSON
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "minify")]
pub struct MinifyRequest {
    /// The JSON string to minify
    pub json: String,
}

/// Response from minify
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinifyResponse {
    /// The minified JSON
    pub json: String,
    /// Size reduction in bytes
    pub bytes_saved: usize,
}
```

## Build to Verify

```powershell
cargo build --release
```

## What's Next?

In the next section, you'll implement the validation handler.

[Continue to Section 2: Validate Message →](./02-validate-message.md)
