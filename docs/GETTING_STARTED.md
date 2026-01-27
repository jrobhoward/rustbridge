# Getting Started with rustbridge

rustbridge is a framework for building Rust shared libraries that can be called from any language. This guide walks you through creating a plugin, packaging it, and running it from your language of choice.

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│  Rust Plugin    │────▶│   .rbp Bundle    │────▶│   Host App      │
│  (you write)    │     │  (portable ZIP)  │     │  (Kotlin/C#/Py) │
│                 │     │                  │     │                 │
│  cargo build    │     │  rustbridge      │     │  plugin.call()  │
│                 │     │  bundle create   │     │                 │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

---

## Prerequisites & Directory Structure

> **Important**: This guide assumes all work is done in `~/rustbridge-workspace/`. The templates and commands use this path structure. If you use a different location, adjust the paths accordingly.

```
~/rustbridge-workspace/
├── rustbridge/          # Cloned repository
├── my-plugin/           # Your Rust plugin (Step 2)
├── my-kotlin-app/       # Consumer app (Step 4)
├── my-java-app/
├── my-csharp-app/
└── my-python-app/
```

> **Package Availability**: rustbridge libraries are currently installed from source. Once the APIs stabilize, packages will be published to Maven Central (Java/Kotlin), NuGet (C#), and PyPI (Python) for easier installation.

---

## Step 1: Clone rustbridge and Install Tools

Create your workspace and clone the repository:

```bash
mkdir -p ~/rustbridge-workspace
cd ~/rustbridge-workspace
git clone https://github.com/jrobhoward/rustbridge.git
cd rustbridge
```

### Install the CLI

```bash
cargo install --path crates/rustbridge-cli
rustbridge --version
```

### Install Host Language Libraries

Choose your target language:

**Kotlin / Java:**
```bash
cd ~/rustbridge-workspace/rustbridge/rustbridge-java
./gradlew publishToMavenLocal
```

**C#:**
```bash
cd ~/rustbridge-workspace/rustbridge/rustbridge-csharp
dotnet build
```

**Python:**
```bash
cd ~/rustbridge-workspace/rustbridge/rustbridge-python
pip install -e .
```
> Note: You may want to use a virtual environment. See Step 4 (Python) for details.

---

## Step 2: Create a Plugin

Copy the plugin template:

```bash
cp -r ~/rustbridge-workspace/rustbridge/templates/plugin ~/rustbridge-workspace/my-plugin
cd ~/rustbridge-workspace/my-plugin
```

Build it:

```bash
cargo build --release
```

This creates:
- **Linux**: `target/release/libmy_plugin.so`
- **macOS**: `target/release/libmy_plugin.dylib`
- **Windows**: `target/release/my_plugin.dll`

---

## Step 3: Create a Bundle

Package your plugin as a portable `.rbp` bundle:

```bash
cd ~/rustbridge-workspace/my-plugin
```

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
```

---

## Step 4: Run from Your Language

Copy the consumer template for your language, add your `.rbp` file, and run.

### Kotlin

```bash
cp -r ~/rustbridge-workspace/rustbridge/templates/kotlin ~/rustbridge-workspace/my-kotlin-app
cd ~/rustbridge-workspace/my-kotlin-app
cp ~/rustbridge-workspace/my-plugin/my-plugin-1.0.0.rbp .
./gradlew run
```

### Java - Java 21+ (Recommended)

```bash
cp -r ~/rustbridge-workspace/rustbridge/templates/java-ffm ~/rustbridge-workspace/my-java-app
cd ~/rustbridge-workspace/my-java-app
cp ~/rustbridge-workspace/my-plugin/my-plugin-1.0.0.rbp .
./gradlew run
```

> Uses the Foreign Function & Memory (FFM) API. For Java 17-20, see [Java JNI](#java-jni---java-17-20) below.

### C#

```bash
cp -r ~/rustbridge-workspace/rustbridge/templates/csharp ~/rustbridge-workspace/my-csharp-app
cd ~/rustbridge-workspace/my-csharp-app
cp ~/rustbridge-workspace/my-plugin/my-plugin-1.0.0.rbp .
dotnet run
```

### Python

```bash
cp -r ~/rustbridge-workspace/rustbridge/templates/python ~/rustbridge-workspace/my-python-app
cd ~/rustbridge-workspace/my-python-app
cp ~/rustbridge-workspace/my-plugin/my-plugin-1.0.0.rbp .

# Create and activate virtual environment
python3 -m venv .venv
source .venv/bin/activate  # Linux/macOS
# .venv\Scripts\activate   # Windows

# Install rustbridge from the cloned repo
pip install -e ~/rustbridge-workspace/rustbridge/rustbridge-python

python main.py
```

### Java (JNI) - Java 17-20

> **Note**: JNI is provided for legacy Java compatibility (8-17). If you're using Java 21+, prefer the [FFM approach](#java---java-21-recommended) above—it's simpler and doesn't require building a separate bridge library.

```bash
cp -r ~/rustbridge-workspace/rustbridge/templates/java-jni ~/rustbridge-workspace/my-java-app
cd ~/rustbridge-workspace/my-java-app
cp ~/rustbridge-workspace/my-plugin/my-plugin-1.0.0.rbp .

# Build the JNI bridge
cd ~/rustbridge-workspace/rustbridge
cargo build --release -p rustbridge-jni

# Update java.library.path in build.gradle.kts to point to:
# ~/rustbridge-workspace/rustbridge/target/release

cd ~/rustbridge-workspace/my-java-app
./gradlew run
```

---

## What Just Happened?

1. **You built a Rust plugin** that exports FFI functions
2. **You packaged it** into a portable `.rbp` bundle
3. **You loaded it** from another language via FFM/JNI/P/Invoke/ctypes
4. **You called a function** using JSON messages

The template plugin implements an "echo" message type:
- **Request**: `{"message": "Hello"}`
- **Response**: `{"message": "Hello", "length": 5}`

---

## Next Steps

### Evolve Your Plugin

The language guides walk you through evolving the echo template into a calculator with multiple message types:

| Language | Guide |
|----------|-------|
| Kotlin | [KOTLIN.md](./using-plugins/KOTLIN.md) |
| Java (FFM) | [JAVA_FFM.md](./using-plugins/JAVA_FFM.md) |
| Java (JNI) | [JAVA_JNI.md](./using-plugins/JAVA_JNI.md) |
| C# | [CSHARP.md](./using-plugins/CSHARP.md) |
| Python | [PYTHON.md](./using-plugins/PYTHON.md) |

### Learn More

- **[Creating Plugins](./creating-plugins/README.md)** - Deep dive into plugin development
- **[Packaging](./packaging/README.md)** - Multi-platform bundles, signing, CI/CD
- **[Binary Transport](./TRANSPORT.md)** - 7x faster than JSON for performance-critical paths
- **[Architecture](./ARCHITECTURE.md)** - System design and concepts

---

## Templates Reference

All templates are in the `templates/` directory:

| Template | Description | Requirements |
|----------|-------------|--------------|
| `templates/plugin` | Rust plugin starter | Rust 1.85+ |
| `templates/kotlin` | Kotlin consumer | Java 21+ |
| `templates/java-ffm` | Java FFM consumer (recommended) | Java 21+ |
| `templates/java-jni` | Java JNI consumer (legacy) | Java 17+ |
| `templates/csharp` | C# consumer | .NET 6.0+ |
| `templates/python` | Python consumer | Python 3.9+ |

---

## Troubleshooting

### "command not found: rustbridge"

The CLI isn't in your PATH. Either:
- Run `cargo install --path crates/rustbridge-cli` again
- Or use the full path: `~/rustbridge-workspace/rustbridge/target/release/rustbridge`

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

### Java: "IllegalCallerException" or "Preview features not enabled"

Add JVM arguments for FFM:

```kotlin
tasks.withType<JavaExec> {
    jvmArgs("--enable-preview", "--enable-native-access=ALL-UNNAMED")
}
```

### "Platform not supported"

Your bundle doesn't include a library for your OS/architecture. Rebuild with the correct platform flag (e.g., `linux-x86_64`, `darwin-aarch64`).

### C#: Project reference not found

The C# template references rustbridge projects at `~/rustbridge-workspace/rustbridge/rustbridge-csharp/`. If you cloned to a different location, update the `<ProjectReference>` paths in `RustBridgeConsumer.csproj`.
