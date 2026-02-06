# {{project-name}} Rust Consumer

This is a Rust consumer application for rustbridge plugins.

## Prerequisites

- Rust 1.90.0+
- A rustbridge plugin bundle (.rbp file)

## Running

1. Copy your plugin bundle to this directory:
   ```bash
   cp ../path/to/your-plugin.rbp .
   ```

2. Update the bundle path in `src/main.rs` if needed.

3. Run the consumer:
   ```bash
   cargo run --release
   ```

## Customization

Edit `src/main.rs` to:
- Add your own request/response types
- Call different message types on the plugin
- Add logging callbacks
- Configure plugin options

## Example with Log Callback

```rust
use rustbridge_consumer::{LogCallbackFn, LogLevel};
use std::sync::Arc;

let log_callback: LogCallbackFn = Arc::new(|level, target, message| {
    println!("[{level:?}] {target}: {message}");
});

let plugin = NativePluginLoader::load_bundle_with_config(
    bundle_path,
    &PluginConfig::builder().log_level(LogLevel::Debug).build(),
    Some(log_callback),
)?;
```

## Example with Signature Verification

```rust
let plugin = NativePluginLoader::load_bundle_with_verification(
    bundle_path,
    &PluginConfig::default(),
    None,   // no log callback
    true,   // verify signatures
    None,   // use manifest's public key
)?;
```
