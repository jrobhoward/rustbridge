# Section 1: Scaffold the Project

In this section, you'll generate a new plugin project using cargo-generate and verify it builds correctly.

## Generate the Project

We'll use the tutorial template to create a minimal starting point:

```bash
cd ~/rustbridge-workspace

cargo generate --git https://github.com/jrobhoward/rustbridge templates/tutorial-plugin --name regex-plugin
```

When prompted, choose these options for now (we'll add features incrementally):

| Prompt | Value | Reason |
|--------|-------|--------|
| Include regex matching? | **false** | We'll add this manually |
| Include LRU cache? | **false** | We'll add this in section 3 |
| Include configuration support? | **false** | We'll add this in section 4 |
| Include logging examples? | **false** | We'll add this throughout |

> **Tip**: If you want to skip ahead and see the completed plugin, generate with all options set to `true`.

## Explore the Generated Project

```bash
cd regex-plugin
```

The generated structure:

```
regex-plugin/
├── Cargo.toml       # Dependencies and crate config
├── src/
│   └── lib.rs       # Plugin implementation
├── .gitignore
├── LICENSE
└── README.md
```

## Understand the Plugin Structure

Open `src/lib.rs`. The template provides a basic echo plugin:

```rust
use rustbridge::prelude::*;

// Message types
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
#[message(tag = "echo")]
pub struct EchoRequest {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoResponse {
    pub message: String,
    pub length: usize,
}

// Plugin implementation
#[derive(Default)]
pub struct RegexPluginPlugin;

#[async_trait]
impl Plugin for RegexPluginPlugin {
    async fn on_start(&self, _ctx: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    async fn handle_request(
        &self,
        _ctx: &PluginContext,
        type_tag: &str,
        payload: &[u8],
    ) -> PluginResult<Vec<u8>> {
        match type_tag {
            "echo" => { /* ... */ }
            _ => Err(PluginError::UnknownMessageType(type_tag.to_string())),
        }
    }

    async fn on_stop(&self, _ctx: &PluginContext) -> PluginResult<()> {
        Ok(())
    }

    fn supported_types(&self) -> Vec<&'static str> {
        vec!["echo"]
    }
}

// FFI entry point
rustbridge_entry!(RegexPluginPlugin::default);
pub use rustbridge::ffi_exports::*;
```

Key components:

1. **Message types**: Request/response structs with Serde derives
2. **`#[message(tag = "...")]`**: Associates the type with a string tag for routing
3. **Plugin struct**: Holds state (empty for now)
4. **Plugin trait**: Lifecycle hooks and request handling
5. **FFI exports**: Required for the shared library to work

## Build and Test

Build the project:

```bash
cargo build --release
```

Run the tests:

```bash
cargo test
```

You should see output like:

```
running 1 test
test tests::handle_request___echo___returns_message_with_length ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## What's Next?

In the next section, we'll replace the echo functionality with regex pattern matching.

[Continue to Section 2: Basic Regex Matching →](./02-basic-matching.md)
