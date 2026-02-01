# Section 1: Project Scaffold

In this section, you'll create a new rustbridge plugin project using the CLI scaffolding command.

## Create the Project

Open PowerShell and run:

```powershell
cd $env:USERPROFILE\rustbridge-workspace

rustbridge new regex-plugin
cd regex-plugin
```

> **Note**: The CLI generates a project with its bundled templates. Check the generated `Cargo.toml` for the actual rustbridge version being used.

This creates a new plugin project with the following structure:

```
regex-plugin\
├── Cargo.toml
├── src\
│   └── lib.rs
└── .gitignore
```

## Examine the Generated Code

Open `src\lib.rs` in your editor. You'll see the basic plugin scaffold:

```rust
//! regex-plugin - A rustbridge plugin

use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};

// ============================================================================
// Message Types
// ============================================================================

/// Request to echo a message back
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "echo")]
pub struct EchoRequest {
    pub message: String,
}

/// Response from echo request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResponse {
    pub message: String,
    pub length: usize,
}

// ============================================================================
// Plugin Implementation
// ============================================================================

#[derive(Default)]
pub struct RegexPlugin;

impl RegexPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    fn handle_echo(&self, req: EchoRequest) -> PluginResult<EchoResponse> {
        tracing::debug!("Handling echo: {:?}", req);
        Ok(EchoResponse {
            length: req.message.len(),
            message: req.message,
        })
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
            "echo" => {
                let req: EchoRequest = serde_json::from_slice(payload)?;
                let resp = self.handle_echo(req)?;
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
        vec!["echo"]
    }
}

// Generate the FFI entry point
rustbridge_entry!(RegexPlugin::new);
pub use rustbridge::ffi_exports::*;
```

## Key Components

### Message Types

Messages are defined as structs with `Serialize` and `Deserialize`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "echo")]
pub struct EchoRequest {
    pub message: String,
}
```

The `#[message(tag = "echo")]` attribute defines the type tag used in `handle_request`.

### Plugin Trait

The `Plugin` trait defines the lifecycle:

- `on_start` - Called when the plugin is loaded
- `handle_request` - Routes incoming messages to handlers
- `on_stop` - Called when the plugin is unloaded
- `supported_types` - Lists handled message types (for introspection)

### FFI Entry Point

The `rustbridge_entry!` macro generates the C-compatible entry points:

```rust
rustbridge_entry!(RegexPlugin::new);
pub use rustbridge::ffi_exports::*;
```

## Build the Plugin

Verify the scaffold builds:

```powershell
cargo build --release
```

This creates `target\release\regex_plugin.dll`.

## Create a Test Bundle

Create a bundle to verify everything works:

```powershell
rustbridge bundle create `
  --name regex-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\regex_plugin.dll `
  --output regex-plugin-0.1.0.rbp
```

List the bundle contents:

```powershell
rustbridge bundle list regex-plugin-0.1.0.rbp
```

```
regex-plugin-0.1.0.rbp
├── manifest.json
└── lib\
    └── windows-x86_64\
        └── release\
            └── regex_plugin.dll
```

## What's Next?

In the next section, you'll replace the echo handler with actual regex matching.

[Continue to Section 2: Basic Matching →](./02-basic-matching.md)
