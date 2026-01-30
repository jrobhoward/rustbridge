# Section 1: Scaffold the Project

In this section, you'll create a new plugin project for the JSON validator/prettifier.

## Create the Project

Since Chapter 4 covers calling from Java, we'll include the Java FFM consumer:

```bash
cd ~/rustbridge-workspace

rustbridge new json-plugin --java-ffm

cd json-plugin
```

> **Tip**: Use `--all` to generate consumers for all languages, or omit flags for Rust-only.

## Explore the Project

The project structure:

```
json-plugin/
├── Cargo.toml           # Dependencies and crate config
├── rustbridge.toml      # Plugin metadata
├── src/
│   └── lib.rs           # Plugin implementation
├── consumers/
│   └── java-ffm/        # Java FFM consumer (from --java-ffm flag)
│       ├── build.gradle.kts
│       └── src/main/java/...
└── .gitignore
```

## Review the Starting Point

Open `src/lib.rs`. You'll see the basic echo plugin from the template:

```rust
use rustbridge::prelude::*;
use rustbridge::{serde_json, tracing};

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

#[derive(Default)]
pub struct JsonPlugin;

#[async_trait]
impl Plugin for JsonPlugin {
    // ... echo implementation
}
```

We'll replace this with our JSON validation and prettifying logic.

## Build and Test

Verify the scaffold builds:

```bash
cargo build --release
cargo test
```

## What's Next?

In the next section, we'll replace the echo functionality with JSON validation.

[Continue to Section 2: Validate Message →](./02-validate-message.md)
