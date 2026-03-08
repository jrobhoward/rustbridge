# Versioning

rustbridge is a monorepo with six language ecosystems. Each ecosystem is versioned independently and published to its own package registry. This document describes the versioning policy, current versions, and release process.

## Current Versions

| Ecosystem | Version | Registry | Package(s) |
|-----------|---------|----------|------------|
| Rust | 1.0.1 | [crates.io](https://crates.io/crates/rustbridge) | `rustbridge`, `rustbridge-core`, `rustbridge-transport`, `rustbridge-ffi`, `rustbridge-runtime`, `rustbridge-logging`, `rustbridge-macros`, `rustbridge-bundle`, `rustbridge-consumer`, `rustbridge-cli` + 2 examples |
| Java/Kotlin | 1.0.0 | [Maven Central](https://central.sonatype.com/namespace/io.github.jrobhoward.rustbridge) | `rustbridge-core`, `rustbridge-ffm`, `rustbridge-kotlin` |
| C# | 1.0.2 | [NuGet](https://www.nuget.org/packages/RustBridge.Core) | `RustBridge.Core`, `RustBridge.Native` |
| Python | 1.0.1 | [PyPI](https://pypi.org/project/rustbridge/) | `rustbridge` |
| Erlang | 1.0.0 | hex.pm | `rustbridge` |
| Go | 0.10.0 | [Go modules](https://pkg.go.dev/github.com/jrobhoward/rustbridge-go) | `github.com/jrobhoward/rustbridge-go` (git-tagged) |

## Why Independent Versioning?

The ecosystems have no build-time dependencies on each other. A C# consumer doesn't import a Rust crate version; a Python package doesn't reference a Java artifact. Each is published to a separate registry with its own release cadence.

The only shared contract is the **C ABI** — a stable set of exported function signatures (`plugin_init`, `plugin_call`, `plugin_free_buffer`, etc.). Consumers detect optional features (like binary transport) at runtime via symbol lookup, so a Python 1.0.1 consumer can load plugins built with Rust 1.0.0 or 1.0.1.

Independent versioning avoids publishing empty releases (e.g., bumping Java to 1.0.2 when nothing changed in Java, just because C# got a fix).

## Versioning Rules

### Rust Crates (lock-step)

All 12 Rust workspace crates share a single version declared in the root `Cargo.toml`:

```toml
[workspace.package]
version = "1.0.1"
```

Individual crates inherit this via `version.workspace = true`. When any Rust crate changes, all are published together at the same version. This is enforced by the Cargo workspace — you cannot publish workspace members at different versions.

### Java/Kotlin (lock-step within ecosystem)

All Java/Kotlin modules share a version declared in `rustbridge-java/build.gradle.kts`:

```kotlin
allprojects {
    version = "1.0.0"
}
```

`rustbridge-core`, `rustbridge-ffm`, and `rustbridge-kotlin` are always published together.

### C# (lock-step within ecosystem)

`RustBridge.Core` and `RustBridge.Native` share a version. Both `.csproj` files must be updated together:

- `rustbridge-csharp/RustBridge.Core/RustBridge.Core.csproj`
- `rustbridge-csharp/RustBridge.Native/RustBridge.Native.csproj`

### Python

Single package with version in `rustbridge-python/pyproject.toml`.

### Erlang

Version in `rustbridge-erlang/src/rustbridge.app.src`.

### Go

Version is determined by git tags (e.g., `rustbridge-go/v0.10.0`). No version in `go.mod`.

## CLI Template Versions

The `rustbridge new` CLI scaffolds consumer projects with package references. Templates live in `crates/rustbridge-cli/templates/` and must reference the **latest published version** of each ecosystem:

| Template | File | Version field |
|----------|------|---------------|
| Rust plugin | `rust/Cargo.toml.tmpl` | `rustbridge = "1.0"` (semver-compatible range) |
| Java FFM | `java-ffm/build.gradle.kts` | `rustbridge-core:X.Y.Z`, `rustbridge-ffm:X.Y.Z` |
| Kotlin | `kotlin/build.gradle.kts` | `rustbridge-core:X.Y.Z`, `rustbridge-ffm:X.Y.Z`, `rustbridge-kotlin:X.Y.Z` |
| C# | `csharp/Consumer.csproj.tmpl` | `RustBridge.Core` Version, `RustBridge.Native` Version |
| Python | `python/requirements.txt` | `rustbridge` (unpinned — latest from PyPI) |

When publishing a new version of any ecosystem, update the corresponding CLI template if the template pins a specific version.

## ABI Compatibility

The C ABI is the integration surface between Rust plugins and host language consumers. It consists of:

**Required exports (stable since 0.5.0):**
- `plugin_init` — Initialize a plugin instance
- `plugin_call` — JSON message transport
- `plugin_free_buffer` — Free Rust-allocated response buffers
- `plugin_shutdown` — Graceful shutdown

**Optional exports (feature-detected at runtime):**
- `plugin_call_raw` / `rb_response_free` — Binary transport (since 0.5.0)
- `plugin_set_log_level` — Runtime log level changes
- `plugin_get_state` — Lifecycle state queries
- `plugin_get_rejected_count` — Concurrency monitoring

Consumers check for optional symbols at load time. If a symbol is missing, the feature is unavailable but the plugin still works for JSON transport. This means consumers and plugins do not need to be at the same version — they just need ABI compatibility.

**Breaking ABI changes** (adding/removing/modifying required exports) require a coordinated major version bump across all ecosystems.

## CHANGELOG Conventions

The project maintains a single [CHANGELOG.md](../CHANGELOG.md) for all ecosystems. Conventions:

1. **Entries are tagged with ecosystem prefixes**: `**C#**:`, `**Python**:`, `**Rust**:`, `**Java/Kotlin**:`, `**Go**:`, `**Erlang**:`, `**Docs**:`, `**CLI**:`, etc.

2. **Release headers list which packages were published**:
   ```
   ## 2026-03-08

   **Published:** C# NuGet 1.0.2, Python PyPI 1.0.1
   ```

3. **Pre-1.0 releases** used lock-step `[X.Y.Z]` headers (all ecosystems shared one version).

4. **Post-1.0 releases** use date-based headers with ecosystem version annotations, since versions diverge.

## Release Process

### Rust Crates

```bash
# 1. Update version in root Cargo.toml [workspace.package]
# 2. cargo publish for each crate in dependency order:
cargo publish -p rustbridge-core
cargo publish -p rustbridge-transport
cargo publish -p rustbridge-macros
cargo publish -p rustbridge-logging
cargo publish -p rustbridge-runtime
cargo publish -p rustbridge-ffi
cargo publish -p rustbridge-bundle
cargo publish -p rustbridge-consumer
cargo publish -p rustbridge
cargo publish -p rustbridge-cli
```

### Java/Kotlin

```bash
cd rustbridge-java
# Update version in build.gradle.kts
./gradlew publishAllPublicationsToMavenCentralRepository
```

### C#

```bash
cd rustbridge-csharp
# Update version in both .csproj files
dotnet pack -c Release
dotnet nuget push RustBridge.Core/bin/Release/RustBridge.Core.X.Y.Z.nupkg --api-key KEY --source https://api.nuget.org/v3/index.json
dotnet nuget push RustBridge.Native/bin/Release/RustBridge.Native.X.Y.Z.nupkg --api-key KEY --source https://api.nuget.org/v3/index.json
```

### Python

```bash
cd rustbridge-python
source .venv/bin/activate
# Update version in pyproject.toml
python -m build
python -m twine upload dist/rustbridge-X.Y.Z*
```

### Go

```bash
# Tag the release in git
git tag rustbridge-go/vX.Y.Z
git push origin rustbridge-go/vX.Y.Z
```
