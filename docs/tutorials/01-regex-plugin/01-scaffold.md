# Section 1: Scaffold the Project

In this section, you'll create a new plugin project and verify it builds correctly.

## Create the Project

We'll start with the basic plugin template and build up features incrementally.

We include `--kotlin` since Chapter 2 covers calling the plugin from Kotlin:

```bash
cd ~/rustbridge-workspace

rustbridge new regex-plugin --kotlin

cd regex-plugin
```

> **Tip**: If you're a git user, at this point, you may want to run `git init && git add . && git commit -m "Initial scaffold"`.
> At the end of each tutorial section, you can commit your progress.


> **Tip**: Now would also be a good time to load the project in your IDE or editor of choice.
> I recommend RustRover or Visual Studio Code.

## Explore the Project

The project structure:

```
regex-plugin/
├── Cargo.toml           # Dependencies and crate config
├── src/
│   └── lib.rs           # Plugin implementation
├── consumers/
│   └── kotlin/          # Kotlin consumer (from --kotlin flag)
│       ├── build.gradle.kts
│       └── src/main/kotlin/...
└── .gitignore
```

## Understand the Plugin Structure

Open `src/lib.rs`. The template provides a basic echo plugin:

```rust
use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};

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
pub struct RegexPlugin;

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
                let response = EchoResponse {
                    length: req.message.len(),
                    message: req.message,
                };
                Ok(serde_json::to_vec(&response)?)
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

// FFI entry point
rustbridge_entry!(RegexPlugin::default);
pub use rustbridge::ffi_exports::*;
```

Key components:

1. **Message types**: Request/response structs with Serde derives
2. **`#[message(tag = "...")]`**: Associates the type with a string tag for routing
3. **Plugin struct**: Holds state (empty for now)
4. **Plugin trait**: Lifecycle hooks and request handling
5. **FFI exports**: Required for the shared library to work

## Build and Test

Format the generated code:

```bash
cargo fmt
```
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
test tests::test_echo ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## What's Next?

In the next section, we'll replace the echo functionality with regex pattern matching.

[Continue to Section 2: Basic Regex Matching →](./02-basic-matching.md)
