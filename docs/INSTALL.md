# Install from Source

rustbridge is not yet published to package registries. Install from source to get started.
The tutorials/docs in this repo will make use of an environment variable (`RUSTBRIDGE_WORKSPACE`), so commands can be copied+pasted into a shell more easily.

## Prerequisites

- Rust 1.90+ installed
- Java 21+ (for Java/Kotlin), .NET 8.0+ (for C#), or Python 3.10+

## 1. Set Up Workspace

Create a workspace directory for rustbridge and your projects.

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

## 3. Install Host Language Libraries

Choose your target language(s):

### Java/Kotlin

Build and publish to local Maven:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-java
./gradlew publishToMavenLocal
```

Then in your project's `build.gradle.kts`:

```kotlin
repositories {
    mavenLocal()
}
dependencies {
    implementation("com.rustbridge:rustbridge-ffm:0.9.1")  // Java 21+
}
```

### C#

Build and reference locally:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-csharp
dotnet build
```

Reference the built DLLs in your project, or use a local NuGet source.

### Python

Install in development mode:

```bash
cd $RUSTBRIDGE_WORKSPACE/rustbridge/rustbridge-python
pip install -e .
```

## Verify Installation

After completing the steps above, verify:

```bash
rustbridge --version
```

You should see version output (e.g., `rustbridge 0.9.1`).

## What's Next?

Now that you have rustbridge installed, continue to the [Getting Started Guide](./GETTING_STARTED.md) to build your first plugin.
