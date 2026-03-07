# Building from Source

This guide is for **contributors** who want to build and develop rustbridge from source. If you just want to use rustbridge in your project, see the [Installation Guide](./INSTALL.md) for installing from published packages.

## Prerequisites

- Rust 1.90+ installed
- Java 21+ (for Java/Kotlin), .NET 8.0+ (for C#), Python 3.10+ (for Python)
- Go 1.21+ (for Go), Erlang/OTP 27+ (for Erlang)

## 1. Set Up Workspace

Create a workspace directory for rustbridge development.

**Linux/macOS** - Add to your shell profile (`~/.bashrc`, `~/.zshrc`, or `~/.profile`):

```bash
export RUSTBRIDGE_WORKSPACE="$HOME/rustbridge-workspace"
```

Then reload your shell and create the directory:

```bash
source ~/.bashrc  # or ~/.zshrc
mkdir -p $RUSTBRIDGE_WORKSPACE
```

**Windows** - Set in PowerShell (one-time):

```powershell
[Environment]::SetEnvironmentVariable("RUSTBRIDGE_WORKSPACE", "$HOME\rustbridge-workspace", "User")
```

Then open a new terminal and create the directory.

## 2. Clone and Install the CLI

```bash
cd $RUSTBRIDGE_WORKSPACE
git clone https://github.com/jrobhoward/rustbridge.git
cd rustbridge
cargo install --force --path crates/rustbridge-cli
rustbridge --version  # Verify installation
```

## 3. Build Host Language Libraries

### Java/Kotlin

Publish to MavenLocal so your local projects can resolve the dependencies:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-java
./gradlew publishToMavenLocal
```

Then add `mavenLocal()` **before** `mavenCentral()` in your consumer project's `build.gradle.kts`:

```kotlin
repositories {
    mavenLocal()  // Use local build
    mavenCentral()
}
```

For Maven, add a local repository:

```xml
<repositories>
    <repository>
        <id>local</id>
        <url>file://${user.home}/.m2/repository</url>
    </repository>
</repositories>
```

### C#

Build and reference locally via ProjectReference:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-csharp
dotnet build
```

Reference the built projects directly in your `.csproj`:

```xml
<ItemGroup>
    <ProjectReference Include="/path/to/rustbridge/rustbridge-csharp/RustBridge.Core/RustBridge.Core.csproj" />
    <ProjectReference Include="/path/to/rustbridge/rustbridge-csharp/RustBridge.Native/RustBridge.Native.csproj" />
</ItemGroup>
```

Or create local NuGet packages:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-csharp
dotnet pack -o ./packages
```

Then add the local feed to your `NuGet.config`:

```xml
<configuration>
  <packageSources>
    <add key="local" value="./packages" />
  </packageSources>
</configuration>
```

### Python

Install in editable mode:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-python
pip install -e .

# Or with development dependencies (pytest, mypy, etc.)
pip install -e ".[dev]"
```

Editable mode (`-e`) allows changes to the rustbridge Python code to take effect immediately without reinstalling.

### Go

Use a `replace` directive in your consumer project's `go.mod`:

```go
module myproject

go 1.21

require github.com/jrobhoward/rustbridge-go v0.0.0

replace github.com/jrobhoward/rustbridge-go => /path/to/rustbridge/rustbridge-go
```

### Erlang

Build directly from the source directory:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-erlang
rebar3 compile
```

The pre-hooks in `rebar.config` handle building the Rust port driver automatically.

### Rust

Use a path dependency in your consumer project's `Cargo.toml`:

```toml
[dependencies]
rustbridge-consumer = { path = "/path/to/rustbridge/crates/rustbridge-consumer" }
```

**Note**: Rust consumers should be created as standalone projects with `cargo new` to avoid Cargo workspace conflicts.

## Verify Installation

```bash
rustbridge --version
```

You should see version output (e.g., `rustbridge 1.0.0`).

## What's Next?

- [Getting Started Guide](./GETTING_STARTED.md) - Build your first plugin
- [Contributing Guide](../CONTRIBUTING.md) - How to contribute
- [Testing Guide](./TESTING.md) - Testing conventions
