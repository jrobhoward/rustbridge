# rustbridge Plugin Template

A minimal Rust plugin template for rustbridge.

## Prerequisites

- **Rust 1.90.0+** (2024 edition)
- **rustbridge CLI** - For creating bundles

## Quick Start

1. **Copy this template** to your project location (from the rustbridge repo):
   ```bash
   cp -r templates/plugin ~/my-plugin
   cd ~/my-plugin
   ```

2. **Build**
   ```bash
   cargo build --release
   ```

3. **Create a bundle**
   ```bash
   # Linux
   rustbridge bundle create \
     --name my-plugin \
     --version 0.1.0 \
     --lib linux-x86_64:target/release/libmy_plugin.so \
     --output my-plugin-0.1.0.rbp

   # macOS
   rustbridge bundle create \
     --name my-plugin \
     --version 0.1.0 \
     --lib darwin-aarch64:target/release/libmy_plugin.dylib \
     --output my-plugin-0.1.0.rbp

   # Windows
   rustbridge bundle create \
     --name my-plugin \
     --version 0.1.0 \
     --lib windows-x86_64:target/release/my_plugin.dll \
     --output my-plugin-0.1.0.rbp
   ```

4. **Use from a consumer template**

   Copy `my-plugin-0.1.0.rbp` to a consumer project and run it.

## What's Included

This template implements a simple "echo" message type:

- **Request**: `{"message": "Hello"}`
- **Response**: `{"message": "Hello", "length": 5}`

## Adding Message Types

1. Define request/response structs in `src/lib.rs`:

   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize, Message)]
   #[message(tag = "math.add")]
   pub struct AddRequest {
       pub a: i64,
       pub b: i64,
   }

   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct AddResponse {
       pub result: i64,
   }
   ```

2. Add a handler in `handle_request`:

   ```rust
   "math.add" => {
       let req: AddRequest = serde_json::from_slice(payload)?;
       let response = AddResponse { result: req.a + req.b };
       Ok(serde_json::to_vec(&response)?)
   }
   ```

3. Update `supported_types`:

   ```rust
   fn supported_types(&self) -> Vec<&'static str> {
       vec!["echo", "math.add"]
   }
   ```

## Project Structure

```
├── Cargo.toml       # Dependencies and crate config
└── src/
    └── lib.rs       # Plugin implementation
```

## Testing

```bash
cargo test
```

## Documentation

- [rustbridge Documentation](https://github.com/jrobhoward/rustbridge)
- [Creating Plugins Guide](https://github.com/jrobhoward/rustbridge/blob/main/docs/creating-plugins/README.md)

## License

MIT
