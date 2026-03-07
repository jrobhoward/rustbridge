# Installation

rustbridge is published to standard package registries. Install the CLI and language libraries for your target language(s).

## CLI

Install the rustbridge CLI from [crates.io](https://crates.io/crates/rustbridge-cli):

```bash
cargo install rustbridge-cli
rustbridge --version  # Verify installation
```

## Language Libraries

### Java/Kotlin

Add the dependency from [Maven Central](https://central.sonatype.com/namespace/io.github.jrobhoward.rustbridge) to your `build.gradle.kts`:

```kotlin
repositories {
    mavenCentral()
}
dependencies {
    implementation("io.github.jrobhoward.rustbridge:rustbridge-ffm:1.0.0")       // Java 21+
    // implementation("io.github.jrobhoward.rustbridge:rustbridge-kotlin:1.0.0")  // Optional: Kotlin extensions
}
```

Or for Maven:

```xml
<dependency>
    <groupId>io.github.jrobhoward.rustbridge</groupId>
    <artifactId>rustbridge-ffm</artifactId>
    <version>1.0.0</version>
</dependency>
```

### C#

Install from [NuGet](https://www.nuget.org/packages/RustBridge.Core):

```bash
dotnet add package RustBridge.Core
dotnet add package RustBridge.Native
```

### Python

Install from [PyPI](https://pypi.org/project/rustbridge/):

```bash
pip install rustbridge
```

### Rust

Add from [crates.io](https://crates.io/crates/rustbridge-consumer):

```bash
cargo add rustbridge-consumer
```

Or manually in `Cargo.toml`:

```toml
[dependencies]
rustbridge-consumer = "1.0"
```

**Note**: Rust consumers must be created as separate projects with `cargo new` to avoid Cargo workspace conflicts.

### Go

```bash
go get github.com/jrobhoward/rustbridge-go
```

### Erlang

Add as a git dependency in `rebar.config` (not yet published to hex.pm):

```erlang
{deps, [
    {rustbridge, {git, "https://github.com/jrobhoward/rustbridge.git", {branch, "main"}}}
]}.
```

## Building from Source

For contributors who want to build rustbridge from source, see the [Development Guide](./DEVELOPMENT.md).

## What's Next?

Now that you have rustbridge installed, continue to the [Getting Started Guide](./GETTING_STARTED.md) to build your first plugin.
