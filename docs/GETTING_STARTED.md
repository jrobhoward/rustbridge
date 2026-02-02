# Getting Started with rustbridge

rustbridge is a framework for building Rust shared libraries that can be bundled and called from various languages. This
guide walks you through creating a plugin, packaging it, and running it from your language of choice.

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Rust Plugin    │────▶│   .rbp Bundle    │────▶│   Host App      │
│  (you write)    │     │  (portable ZIP)  │     │  (JVM/C#/Py)    │
│                 │     │                  │     │                 │
│  cargo build    │     │  rustbridge      │     │  plugin.call()  │
│                 │     │  bundle create   │     │                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

---

## Prerequisites

**Before starting this guide**, complete the [Install from Source](./INSTALL.md) guide:

1. Set up your workspace (`$RUSTBRIDGE_WORKSPACE`)
2. Clone the repository and install the CLI
3. Install host language libraries for your target language(s)

Verify your installation:

```bash
rustbridge --version  # Should show rustbridge 0.8.0 or later
```

---

## Step 1: Build a Plugin

Generate a plugin with consumer projects for all supported languages:

```bash
# Run from any directory where you want to create your plugin
rustbridge new my-plugin --all
cd my-plugin
```

This creates a Rust plugin at the root with a `consumers/` directory containing ready-to-run projects for Kotlin, Java,
C#, and Python.

> **Tip**: You can generate only the languages you need by replacing `--all` with one or more of: `--kotlin`,
`--java-ffm`, `--csharp`, `--python`.
> Or omit all flags for a Rust-only plugin.

> **Tip**: If you're a git user, at this point, you may want to run
`git init && git add . && git commit -m "Initial plugin scaffold"`.

Build it:

```bash
cargo build --release
```

What we've done so far creates a standard shared library:

- **Linux**: `target/release/libmy_plugin.so`
- **macOS**: `target/release/libmy_plugin.dylib`
- **Windows**: `target/release/my_plugin.dll`

---

## Step 2: Create a Bundle

Now use the `rustbridge` CLI to package your plugin as a portable `.rbp` file (run from your plugin directory):

```bash
# Linux
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --output my-plugin-1.0.0.rbp
```

```bash
# macOS (Apple Silicon)
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib darwin-aarch64:target/release/libmy_plugin.dylib \
  --output my-plugin-1.0.0.rbp
```

```bash
# Windows
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib windows-x86_64:target/release/my_plugin.dll \
  --output my-plugin-1.0.0.rbp
```

Verify:

```bash
rustbridge bundle list my-plugin-1.0.0.rbp

# additional/optional inspection
unzip -l my-plugin-1.0.0.rbp
unzip -p my-plugin-1.0.0.rbp manifest.json
```

---

## Step 3: Run from Your Language

If you ran `rustbridge new my-plugin --all` in Step 1, you already have consumer projects in `consumers/`.
Pick your language below and run it.

### Kotlin

```bash
cd consumers/kotlin
cp ../../my-plugin-1.0.0.rbp .
./gradlew run
```

### Java (FFM) - Java 21+

> **Note**: Java 21 requires `--enable-preview` flag. Java 22+ is recommended for stable FFM APIs.

```bash
cd consumers/java-ffm
cp ../../my-plugin-1.0.0.rbp .
./gradlew run
```

### C#

```bash
cd consumers/csharp
cp ../../my-plugin-1.0.0.rbp .
dotnet run
```

### Python

```bash
cd consumers/python
cp ../../my-plugin-1.0.0.rbp .

# Create and activate virtual environment
python3 -m venv .venv
source .venv/bin/activate  # Linux/macOS
# .venv\Scripts\activate   # Windows

# Install rustbridge Python library
pip install -e $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-python

python main.py
```

---

## What Just Happened?

1. **You built a Rust plugin** that exports FFI functions
2. **You packaged it** into a portable `.rbp` bundle
3. **You loaded it** from another language via FFM/PInvoke/ctypes
4. **You called a function** using JSON messages

The template plugin implements an "echo" message type:

- **Request**: `{"message": "Hello"}`
- **Response**: `{"message": "Hello", "length": 5}`

---

## Next Steps

### Tutorials

Follow the step-by-step tutorials to learn advanced topics:

- **[Tutorials](./tutorials/README.md)** - Production bundles, cross-compilation, backpressure, and binary transport

### Language Guides

The language guides below walk you through evolving the echo template into a  
calculator with multiple message types:

> **Note:** These guides are generally accurate but may not reflect the latest
> changes. For the most current approach, see the                              
[Tutorials](./tutorials/README.md).

| Language   | Guide                                                      |
|------------|------------------------------------------------------------|
| Kotlin     | [KOTLIN.md](./using-plugins/KOTLIN.md)                     |
| Java       | [JAVA_FFM.md](./using-plugins/JAVA_FFM.md) (Java 21+)      |
| C#         | [CSHARP.md](./using-plugins/CSHARP.md)                     |
| Python     | [PYTHON.md](./using-plugins/PYTHON.md)                     |

### Learn More

- **[Creating Plugins](./creating-plugins/README.md)** - Deep dive into plugin development
- **[Packaging](./packaging/README.md)** - Multi-platform bundles, signing, CI/CD
- **[Binary Transport](./TRANSPORT.md)** - 7x faster than JSON for performance-critical paths
- **[Architecture](./ARCHITECTURE.md)** - System design and concepts

---

## Templates Reference

The `rustbridge new` command generates projects from templates embedded in the CLI:

```bash
rustbridge new my-plugin                    # Rust plugin only
rustbridge new my-plugin --kotlin           # Rust + Kotlin consumer
rustbridge new my-plugin --java-ffm         # Rust + Java FFM consumer
rustbridge new my-plugin --csharp           # Rust + C# consumer
rustbridge new my-plugin --python           # Rust + Python consumer
rustbridge new my-plugin --all              # Rust + all consumers
```

| Template  | Description                       | Requirements |
|-----------|-----------------------------------|--------------|
| rust      | Rust plugin                       | Rust 1.90+   |
| kotlin    | Kotlin consumer                   | Java 21+     |
| java-ffm  | Java FFM consumer                 | Java 21+     |
| csharp    | C# consumer                       | .NET 8.0+    |
| python    | Python consumer                   | Python 3.10+ |

---

## Troubleshooting

### "command not found: rustbridge"

The CLI isn't in your PATH. Either:

- Run `cargo install --path crates/rustbridge-cli` again
- Or use the full path: `/path/to/rustbridge/target/release/rustbridge`

### "Plugin library not found" or "symbol not found"

Your plugin is missing FFI exports. Ensure your `lib.rs` includes:

```rust
pub use rustbridge_ffi::{
    plugin_call,
    plugin_free_buffer,
    plugin_get_rejected_count,
    plugin_get_state,
    plugin_init,
    plugin_set_log_level,
    plugin_shutdown,
};
```

### Java: "IllegalCallerException"

Add JVM arguments for FFM (Java 22+):

```kotlin
tasks.withType<JavaExec> {
    jvmArgs("--enable-native-access=ALL-UNNAMED")
}
```

### "Platform not supported"

Your bundle doesn't include a library for your OS/architecture. Rebuild with the correct platform flag (e.g.,
`linux-x86_64`, `darwin-aarch64`).

### C#: Project reference not found

The C# template references rustbridge projects relative to where you cloned the repo. If paths don't match,
update the `<ProjectReference>` paths in `RustBridgeConsumer.csproj`.

### C#: Metadata file could not be found

If you see errors like `Metadata file 'RustBridge.Core.dll' could not be found`, this typically occurs when
RustBridge was previously built from a different path. Run `dotnet clean` in the `rustbridge-csharp/` directory,
then rebuild your consumer:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-csharp
dotnet clean
cd /path/to/your/consumer
dotnet build
```
