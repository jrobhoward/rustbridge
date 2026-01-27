# Getting Started with rustbridge

rustbridge is a framework for building Rust shared libraries that can be called from any language. It provides a clean async API, automatic memory management, and portable plugin bundles.

## Quick Overview

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Host App      │     │   .rbp Bundle    │     │  Rust Plugin    │
│  (Java/C#/Py)   │────▶│  (portable ZIP)  │────▶│  (native lib)   │
│                 │     │                  │     │                 │
│  plugin.call()  │     │  manifest.json   │     │  async/await    │
│                 │     │  lib/*.so/dll    │     │  JSON messages  │
└─────────────────┘     └──────────────────┘     └─────────────────┘
```

## Choose Your Path

### I want to create a plugin

Start here if you're building a Rust library to be consumed by other languages.

**[Creating Plugins Guide →](./creating-plugins/README.md)**

Covers:
- Project setup (from template or manual)
- Defining message types
- Implementing the Plugin trait
- Building your plugin
- Creating a basic `.rbp` bundle

### I want to package and distribute plugins

Start here if you need to create multi-platform bundles, combine bundles, or sign for production.

**[Packaging Guide →](./packaging/README.md)**

Covers:
- Multi-platform bundles
- Combining platform-specific bundles
- Creating slimmed release bundles
- Code signing with minisign
- CI/CD integration

### I want to use a plugin from my application

Start here if you have a `.rbp` bundle and want to call it from your application.

**[Using Plugins Guide →](./using-plugins/README.md)**

Language-specific guides:

| Language | Guide | Requirements |
|----------|-------|--------------|
| Kotlin | [KOTLIN.md](./using-plugins/KOTLIN.md) | Java 21+ |
| Java (FFM) | [JAVA_FFM.md](./using-plugins/JAVA_FFM.md) | Java 21+ (recommended) |
| Java (JNI) | [JAVA_JNI.md](./using-plugins/JAVA_JNI.md) | Java 8+ (fallback) |
| C# | [CSHARP.md](./using-plugins/CSHARP.md) | .NET 6.0+ |
| Python | [PYTHON.md](./using-plugins/PYTHON.md) | Python 3.9+ |

## Prerequisites

Before starting with any path, ensure you have:

### For Creating Plugins
- **Rust 1.85.0+** (2024 edition) - [Install](https://rustup.rs/)
- **rustbridge-cli** - `cargo install rustbridge-cli`

### For Using Plugins
- Your target language runtime (see table above)
- A `.rbp` bundle file (or raw native library for development)

## Example: End-to-End Flow

```bash
# 1. Create a plugin (Rust developer)
rustbridge new my-plugin
cd my-plugin
cargo build --release
rustbridge bundle create \
  --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --output my-plugin-1.0.0.rbp

# 2. Use the plugin (Java developer)
# Add rustbridge-ffm dependency, then:
```

```java
var loader = BundleLoader.builder()
    .bundlePath("my-plugin-1.0.0.rbp")
    .verifySignatures(false)
    .build();

try (var plugin = FfmPluginLoader.load(loader.extractLibrary())) {
    String response = plugin.call("echo", "{\"message\": \"Hello\"}");
    System.out.println(response);
}
```

## Additional Resources

- [Architecture Overview](./ARCHITECTURE.md) - System design and concepts
- [Bundle Format](./BUNDLE_FORMAT.md) - `.rbp` file specification
- [Transport](./TRANSPORT.md) - JSON and binary message formats
- [Memory Model](./MEMORY_MODEL.md) - Memory ownership patterns
- [Testing Guide](./TESTING.md) - Testing conventions
- [Debugging](./DEBUGGING.md) - Troubleshooting tips

## Getting Help

- **Documentation**: See the `docs/` directory for detailed guides
- **Examples**: Check `examples/` for working code
- **Issues**: Report bugs at https://github.com/yourorg/rustbridge/issues
