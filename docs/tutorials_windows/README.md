# Rustbridge Tutorials (Windows)

Step-by-step tutorials for building, distributing, and consuming rustbridge plugins on Windows.

> **Note**: This is the Windows version of the tutorials, using PowerShell commands. For Linux/macOS, see [docs/tutorials/](../tutorials/).

## Prerequisites

Before starting, ensure you have:

1. **Rust 1.90+** with the Windows MSVC toolchain:
   ```powershell
   rustup default stable-msvc
   rustup update
   ```

2. **Visual Studio Build Tools** (for the C++ toolchain):
   - Download from [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
   - Install "Desktop development with C++"

3. **The rustbridge CLI**:
   ```powershell
   cargo install --git https://github.com/example/rustbridge rustbridge-cli
   ```

4. **A workspace directory**:
   ```powershell
   mkdir $env:USERPROFILE\rustbridge-workspace
   cd $env:USERPROFILE\rustbridge-workspace
   ```

## Tutorial Structure

### Part 1: Building Plugins

#### [Chapter 1: Regex Plugin](./01-regex-plugin/README.md)
Build your first rustbridge plugin—a regex engine with caching. You'll learn:
- Project scaffolding with `rustbridge new`
- Implementing the Plugin trait
- Message types and serialization
- LRU caching for performance
- Configuration from the host

#### [Chapter 3: JSON Plugin](./03-json-plugin/README.md)
Build a JSON toolkit plugin with validation and formatting. You'll learn:
- Multiple message types in one plugin
- Error handling patterns
- Testing strategies

### Part 2: Consuming Plugins

#### [Chapter 2: Kotlin Consumer](./02-kotlin-consumer/README.md)
Call your regex plugin from Kotlin using FFM. You'll learn:
- Java FFM (Foreign Function & Memory) API
- Gradle project setup
- Logging callbacks
- Type-safe wrapper classes

#### [Chapter 4: Java Consumer](./04-java-consumer/README.md)
Call the JSON plugin from Java. You'll learn:
- Both FFM (Java 21+) and JNI (Java 17+) approaches
- Error handling patterns
- JSON serialization with Gson

### Part 3: Production Topics

#### [Chapter 5: Production Bundles](./05-production-bundles/README.md)
Prepare plugins for production deployment. You'll learn:
- Code signing with minisign
- JSON schema generation
- Build metadata embedding
- SBOM generation

#### [Chapter 6: Native Builds](./06-native-builds/README.md)
Build plugins for Windows platforms. You'll learn:
- Building for Windows x64
- Building for Windows ARM64 (if available)
- Creating multi-platform bundles

### Part 4: Advanced Patterns

#### [Chapter 7: Backpressure Queues](./07-backpressure-queues/README.md)
Implement bounded queues with flow control. You'll learn:
- Serialized plugin access patterns
- Backpressure for memory control
- Future/Promise-based async calls
- C#, Java/JNI, and Python consumers

#### [Chapter 8: Binary Transport](./08-binary-transport/README.md)
Build an image thumbnail generator using binary transport. You'll learn:
- When to use binary vs JSON transport
- C-compatible struct layouts
- Direct memory manipulation in each language
- Performance optimization

## Quick Start

If you want to jump right in:

```powershell
cd $env:USERPROFILE\rustbridge-workspace

# Create a new plugin
rustbridge new hello-plugin
cd hello-plugin

# Build it
cargo build --release

# Create a bundle (use version 0.1.0 to match generated Cargo.toml)
rustbridge bundle create `
  --name hello-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\hello_plugin.dll `
  --output hello-plugin-0.1.0.rbp

# List bundle contents
rustbridge bundle list hello-plugin-0.1.0.rbp
```

> **Note**: The `rustbridge new` command generates projects with the version from the CLI. The generated `Cargo.toml` may use a slightly different rustbridge version than documented. Always check the generated files.

## Conventions Used

Throughout these tutorials:

- **PowerShell** is used for all commands (also works in Windows Terminal)
- **Backtick (`)** is used for line continuation in PowerShell
- **`$env:USERPROFILE\rustbridge-workspace`** is the working directory
- **`.dll`** is the Windows library extension
- **`windows-x86_64`** is the platform identifier for 64-bit Windows

## Getting Help

- [CLAUDE.md](../../CLAUDE.md) - Quick reference for development commands
- [docs/ARCHITECTURE.md](../ARCHITECTURE.md) - System architecture overview
- [docs/BUNDLE_FORMAT.md](../BUNDLE_FORMAT.md) - Bundle specification
- [docs/TRANSPORT.md](../TRANSPORT.md) - JSON and binary transport

## Feedback

Found an issue or have suggestions? Please open an issue at the project repository.
