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

## Prerequisites & Directory Structure

> **Important**: The current version of this guide assumes all work is done in `~/rustbridge-workspace/`. The commands
> in this tutorial assume this path structure. If you use a different location, adjust the paths accordingly.

```
~/rustbridge-workspace/
├── rustbridge/          # Cloned repository
├── my-plugin/           # Your Rust plugin (Step 2)
├── my-kotlin-app/       # Consumer app (Step 4)
├── my-java-app/
├── my-csharp-app/
└── my-python-app/
```

### Prerequisites

- Rust 1.90+ installed
- Basic familiarity with Rust

---

## Step 1: Clone rustbridge and Install Tools

Create a workspace directory and clone the repository:

```bash
mkdir -p ~/rustbridge-workspace
cd ~/rustbridge-workspace
git clone https://github.com/jrobhoward/rustbridge.git
cd rustbridge
```

### Install the CLI

The rustbridge CLI simplifies plugin development.  
In this tutorial, we'll use it to bundle a shared library into a `.rbp` file.

```bash
cargo install --path crates/rustbridge-cli
rustbridge --version
rustbridge --help
```

### Install Host Language Libraries

> **Package Availability**: The host language libraries are currently built from source. Once the APIs and `.rbp` file
> format are stable (i.e. 1.0 release), packages may be published to Maven Central (Java/Kotlin), NuGet (C#), and PyPI (
> Python) for easier consumption.


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

> Note: If you see an error about `externally-managed-environment`, you may want to use a virtual environment.
> See Step 4 (Python) for details.

---

## Step 2: Build a Plugin

Generate a plugin with consumer projects for all supported languages:

```bash
cd ~/rustbridge-workspace

rustbridge new my-plugin --all

cd my-plugin
```

This creates a Rust plugin at the root with a `consumers/` directory containing ready-to-run projects for Kotlin, Java,
C#, and Python.

> **Tip**: You can generate only the languages you need by replacing `--all` with one or more of: `--kotlin`,
`--java-ffm`, `--java-jni`, `--csharp`, `--python`.
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

## Step 3: Create a Bundle

Now use the `rustbridge` CLI to package your plugin as a portable `.rbp` file:

```bash
cd ~/rustbridge-workspace/my-plugin
```

```bash
# Linux
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --jni-lib linux-x86_64:target/release/librustbridge_jni.so \
  --output my-plugin-1.0.0.rbp
```

```bash
# macOS (Apple Silicon)
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib darwin-aarch64:target/release/libmy_plugin.dylib \
  --jni-lib darwin-aarch64:target/release/librustbridge_jni.dylib \
  --output my-plugin-1.0.0.rbp
```

```bash
# Windows
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib windows-x86_64:target/release/my_plugin.dll \
  --jni-lib windows-x86_64:target/release/rustbridge_jni.dll \
  --output my-plugin-1.0.0.rbp
```

> **Note:** The `--jni-lib` flag bundles the JNI bridge library inside the `.rbp` file. This makes your bundle self-contained for Java users - they don't need to build or install the JNI bridge separately.
>
> **Building the JNI bridge:** The JNI bridge is part of the rustbridge framework (not your plugin). Build it from the rustbridge repository:
> ```bash
> cd ~/rustbridge-workspace/rustbridge
> cargo build --release -p rustbridge-jni
> # Library is at: target/release/librustbridge_jni.so (Linux)
> #                target/release/librustbridge_jni.dylib (macOS)
> #                target/release/rustbridge_jni.dll (Windows)
> ```

Verify:

```bash
rustbridge bundle list my-plugin-1.0.0.rbp

# additional/optional inspection
unzip -l my-plugin-1.0.0.rbp
unzip -p my-plugin-1.0.0.rbp manifest.json
```

---

## Step 4: Run from Your Language

If you ran `rustbridge new my-plugin --all` in Step 2, you already have consumer projects in `consumers/`.
Pick your language below and run it.

### Kotlin

```bash
cd ~/rustbridge-workspace/my-plugin/consumers/kotlin
cp ../../my-plugin-1.0.0.rbp .
./gradlew run
```

### Java (JNI) - Recommended for Java 17+

> JNI is the recommended approach for all Java versions (17+). It provides excellent compatibility across all LTS releases and the best binary transport performance.

```bash
# Build the JNI bridge first
cd ~/rustbridge-workspace/rustbridge
cargo build --release -p rustbridge-jni

cd ~/rustbridge-workspace/my-plugin/consumers/java-jni
cp ../../my-plugin-1.0.0.rbp .
# Update java.library.path in build.gradle.kts if needed
./gradlew run
```

### C#

```bash
cd ~/rustbridge-workspace/my-plugin/consumers/csharp
cp ../../my-plugin-1.0.0.rbp .
dotnet run
```

### Python

```bash
cd ~/rustbridge-workspace/my-plugin/consumers/python
cp ../../my-plugin-1.0.0.rbp .

# Create and activate virtual environment
python3 -m venv .venv
source .venv/bin/activate  # Linux/macOS
# .venv\Scripts\activate   # Windows

# Install rustbridge from the cloned repo
pip install -e ~/rustbridge-workspace/rustbridge/rustbridge-python

python main.py
```

### Java (FFM) - Java 21+

> **Note**: Java 21 requires `--enable-preview` flag. Java 22+ is recommended for stable FFM APIs.

```bash
cd ~/rustbridge-workspace/my-plugin/consumers/java-ffm
cp ../../my-plugin-1.0.0.rbp .
./gradlew run
```

---

## What Just Happened?

1. **You built a Rust plugin** that exports FFI functions
2. **You packaged it** into a portable `.rbp` bundle
3. **You loaded it** from another language via FFM/JNI/PInvoke/ctypes
4. **You called a function** using JSON messages

The template plugin implements an "echo" message type:

- **Request**: `{"message": "Hello"}`
- **Response**: `{"message": "Hello", "length": 5}`

---

## Next Steps

### Tutorials

Follow the step-by-step tutorials to evolve this basic echo plugin into something more useful:

- **[Tutorials](./tutorials/README.md)** - Build a regex plugin with caching, configuration, and call it from Kotlin

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

The C# template references rustbridge projects at `~/rustbridge-workspace/rustbridge/rustbridge-csharp/`. If you cloned
to a different location, update the `<ProjectReference>` paths in `RustBridgeConsumer.csproj`.
