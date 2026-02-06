# rustbridge CLI Reference

The `rustbridge` command-line tool scaffolds plugin projects, packages them into portable `.rbp` bundles, and manages code signing keys.

For installation instructions, see [INSTALL.md](./INSTALL.md). For an end-to-end walkthrough, see [GETTING_STARTED.md](./GETTING_STARTED.md).

## Commands

| Command | Purpose |
|---------|---------|
| [`new`](#new) | Scaffold a plugin project with optional consumer projects |
| [`pack`](#pack) | Auto-detect and bundle a plugin from the current directory |
| [`promote`](#promote) | Slim a dev bundle to a signed release bundle |
| [`bundle`](#bundle) | Create, inspect, combine, slim, or extract bundles manually |
| [`keygen`](#keygen) | Generate a minisign key pair for code signing |
| [`generate-header`](#generate-header) | Generate a C header from Rust `#[repr(C)]` structs |

---

## `new`

Scaffold a new plugin project with a working Rust crate and optional consumer projects for host languages.

```bash
rustbridge new <NAME> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<NAME>` | Project name (used as directory name and crate name) |

**Options:**

| Option | Description |
|--------|-------------|
| `-p, --path <PATH>` | Project directory (default: `./<name>`) |
| `--kotlin` | Also generate Kotlin consumer project (requires Java 21+) |
| `--java-ffm` | Also generate Java FFM consumer project (requires Java 21+) |
| `--csharp` | Also generate C# consumer project (requires .NET 8+) |
| `--python` | Also generate Python consumer project |
| `--all` | Generate all consumer projects (Java, Kotlin, C#, Python) |

**Examples:**

```bash
# Rust plugin only
rustbridge new my-plugin

# Plugin with all consumer projects
rustbridge new my-plugin --all

# Plugin with specific consumers
rustbridge new my-plugin --kotlin --python

# Specify a custom directory
rustbridge new my-plugin --path /tmp/projects/my-plugin
```

The generated project is ready to build immediately with `cargo build --release`. Consumer projects are placed in a `consumers/` subdirectory, each with their own build system (Gradle, dotnet, pip).

See the [Getting Started Guide](./GETTING_STARTED.md) for a full walkthrough of building and running a scaffolded project.

---

## `pack`

Auto-detect plugin metadata from `Cargo.toml` and create a `.rbp` bundle in one step. This is the recommended way to package plugins during development.

```bash
rustbridge pack [OPTIONS]
```

Run this from within a plugin project directory. It reads `name`, `version`, and `lib.name` from `Cargo.toml`, detects the current platform, locates the built library under `target/`, and delegates to the bundle creation logic.

**Options:**

| Option | Description |
|--------|-------------|
| `--dev` | Create a dev bundle (includes both release and debug libraries, unsigned) |
| `--sign-key <PATH>` | Path to signing key (default: `~/.rustbridge/signing.key`) |
| `--no-sign` | Do not sign the bundle |
| `--schema-source <SOURCE:NAME>` | Auto-generate JSON Schema from Rust source and embed in bundle (dev mode only) |
| `--header-source <SOURCE:NAME>` | Auto-generate C header from Rust source and embed in bundle (dev mode only) |

**Examples:**

```bash
# Signed release bundle (default)
rustbridge pack

# Unsigned bundle for local development
rustbridge pack --no-sign

# Dev bundle with both release and debug libraries
rustbridge pack --dev

# Embed auto-generated schema from Rust source
rustbridge pack --no-sign --schema-source src/messages.rs:schema.json
```

Output is written to `target/bundle/<name>-<version>.rbp` (or `<name>-<version>-dev.rbp` in dev mode).

**Cargo.toml metadata:** You can configure `pack` defaults in your plugin's `Cargo.toml` under `[package.metadata.rustbridge]`:

```toml
[package.metadata.rustbridge]
schema-source = "src/messages.rs:schema.json"
header-source = "src/binary_messages.rs:messages.h"
```

When these fields are present, `pack` automatically embeds the generated schema or header without requiring command-line flags.

---

## `promote`

Slim a dev bundle down to a release-only bundle, optionally signing it. This supports a workflow where developers create unsigned dev bundles locally, then promote them to signed release bundles for distribution.

```bash
rustbridge promote <INPUT> [OPTIONS]
```

**Arguments:**

| Argument | Description |
|----------|-------------|
| `<INPUT>` | Input bundle path (typically a `-dev.rbp` bundle) |

**Options:**

| Option | Description |
|--------|-------------|
| `-o, --output <OUTPUT>` | Output bundle path (default: derived from input, strips `-dev` suffix) |
| `--sign-key <PATH>` | Path to signing key (default: `~/.rustbridge/signing.key`) |
| `--no-sign` | Do not sign the bundle |

**Examples:**

```bash
# Promote dev bundle to signed release (auto-derives output name)
# my-plugin-0.1.0-dev.rbp → my-plugin-0.1.0.rbp
rustbridge promote target/bundle/my-plugin-0.1.0-dev.rbp

# Explicit output path
rustbridge promote my-plugin-dev.rbp -o dist/my-plugin.rbp

# Promote without signing
rustbridge promote my-plugin-dev.rbp --no-sign
```

**Typical workflow:**

```
cargo build --release && cargo build     # Build release + debug
rustbridge pack --dev                    # Dev bundle (both variants, unsigned)
  ↓ test locally with dev bundle
rustbridge promote <name>-dev.rbp        # Strip debug, sign for release
```

---

## `bundle`

Low-level commands for creating, inspecting, combining, slimming, and extracting `.rbp` bundles. For most single-platform development, [`pack`](#pack) is simpler. Use `bundle` subcommands when you need explicit control over platforms, variants, or multi-bundle merging.

```bash
rustbridge bundle <SUBCOMMAND>
```

For the `.rbp` archive structure and manifest schema, see [BUNDLE_FORMAT.md](./BUNDLE_FORMAT.md).

### `bundle create`

Create a bundle from explicitly specified libraries.

```bash
rustbridge bundle create --name <NAME> --version <VERSION> [OPTIONS]
```

**Required options:**

| Option | Description |
|--------|-------------|
| `-n, --name <NAME>` | Plugin name |
| `-v, --version <VERSION>` | Plugin version (semver) |

**Library options:**

| Option | Description |
|--------|-------------|
| `-l, --lib <PLATFORM[:VARIANT]:PATH>` | Library to include (repeatable). Format: `PLATFORM:PATH` (release) or `PLATFORM:VARIANT:PATH` |

**Output options:**

| Option | Description |
|--------|-------------|
| `-o, --output <OUTPUT>` | Output bundle path (default: `<name>-<version>.rbp`) |
| `--sign-key <KEY_PATH>` | Path to signing key |
| `--no-metadata` | Skip automatic build metadata collection |

**Schema and documentation options:**

| Option | Description |
|--------|-------------|
| `-s, --schema <SOURCE:ARCHIVE_NAME>` | Schema file to include (repeatable) |
| `--generate-header <SOURCE:HEADER_NAME>` | Auto-generate C header from Rust source and embed |
| `--generate-schema <SOURCE:SCHEMA_NAME>` | Auto-generate JSON Schema from Rust source and embed |
| `--notices <PATH>` | License notices file to include |
| `--license <PATH>` | Plugin's own license file to include |
| `--sbom <SOURCE:ARCHIVE_NAME>` | SBOM file to include (repeatable) |
| `--metadata <KEY=VALUE>` | Custom metadata as KEY=VALUE (repeatable) |

**Examples:**

```bash
# Single-platform bundle
rustbridge bundle create \
  --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --output my-plugin-1.0.0.rbp

# Multi-platform with signing and schema
rustbridge bundle create \
  --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --lib darwin-aarch64:target/release/libmy_plugin.dylib \
  --sign-key ~/.rustbridge/signing.key \
  --generate-schema src/messages.rs:schema.json \
  --license LICENSE

# Include debug variant alongside release
rustbridge bundle create \
  --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --lib linux-x86_64:debug:target/debug/libmy_plugin.so

# Attach custom metadata and SBOM
rustbridge bundle create \
  --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --metadata repository=https://github.com/user/project \
  --metadata ci_job_id=12345 \
  --sbom sbom.cdx.json:sbom.cdx.json
```

**Platform identifiers:** `linux-x86_64`, `linux-aarch64`, `darwin-x86_64`, `darwin-aarch64`, `windows-x86_64`, `windows-aarch64`

### `bundle combine`

Merge multiple single-platform bundles into one multi-platform bundle. Useful in CI pipelines where each platform builds independently.

```bash
rustbridge bundle combine --output <OUTPUT> <BUNDLE1> <BUNDLE2> [BUNDLES...]
```

| Option | Description |
|--------|-------------|
| `-o, --output <OUTPUT>` | Output bundle path |
| `--sign-key <KEY_PATH>` | Re-sign the combined bundle |
| `--schema-mismatch <MODE>` | Schema mismatch handling: `error` (default), `warn`, `ignore` |

**Example:**

```bash
rustbridge bundle combine \
  --output my-plugin-1.0.0.rbp \
  --sign-key ~/.rustbridge/signing.key \
  my-plugin-linux.rbp my-plugin-macos.rbp my-plugin-windows.rbp
```

### `bundle slim`

Create a new bundle containing only a subset of platforms or variants from an existing bundle.

```bash
rustbridge bundle slim --input <INPUT> --output <OUTPUT> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-i, --input <INPUT>` | Input bundle path |
| `-o, --output <OUTPUT>` | Output bundle path |
| `--platforms <PLATFORMS>` | Platforms to keep (comma-separated) |
| `--variants <VARIANTS>` | Variants to keep (comma-separated, default: `release`) |
| `--exclude-docs` | Exclude documentation files |
| `--sign-key <KEY_PATH>` | Re-sign the slimmed bundle |

**Example:**

```bash
# Extract only Linux libraries from a multi-platform bundle
rustbridge bundle slim \
  --input my-plugin-1.0.0.rbp \
  --output my-plugin-linux.rbp \
  --platforms linux-x86_64,linux-aarch64
```

### `bundle list`

Display the contents and metadata of a bundle.

```bash
rustbridge bundle list [OPTIONS] <BUNDLE>
```

| Option | Description |
|--------|-------------|
| `--show-build` | Show build info (git commit, compiler, timestamp) |
| `--show-variants` | Show all variants (release, debug, etc.) |

**Example:**

```bash
rustbridge bundle list my-plugin-1.0.0.rbp
rustbridge bundle list --show-build --show-variants my-plugin-1.0.0.rbp
```

### `bundle extract`

Extract a native library from a bundle for the current (or specified) platform.

```bash
rustbridge bundle extract [OPTIONS] <BUNDLE>
```

| Option | Description |
|--------|-------------|
| `-p, --platform <PLATFORM>` | Target platform (default: current platform) |
| `--variant <VARIANT>` | Variant to extract (default: `release`) |
| `-o, --output <OUTPUT>` | Output directory (default: `.`) |

**Example:**

```bash
# Extract for current platform
rustbridge bundle extract my-plugin-1.0.0.rbp

# Extract a specific platform's debug library
rustbridge bundle extract my-plugin-1.0.0.rbp \
  --platform linux-aarch64 --variant debug -o ./libs/
```

---

## `keygen`

Generate a minisign key pair for signing bundles. The secret key is used with `--sign-key` options in `pack`, `promote`, and `bundle` commands. Distribute the public key to consumers for signature verification.

```bash
rustbridge keygen [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-o, --output <OUTPUT>` | Output path for secret key (default: `~/.rustbridge/signing.key`) |
| `-f, --force` | Force overwrite if key already exists |

**Example:**

```bash
# Generate key pair at default location
rustbridge keygen

# Custom output path
rustbridge keygen --output ./keys/my-signing.key
```

The public key is written alongside the secret key with a `.pub` extension. See [Tutorial 05: Production Bundles](./tutorials/05-production-bundles/README.md) for a walkthrough of the signing workflow.

---

## `generate-header`

Generate a C header file from Rust `#[repr(C)]` structs. This is used for binary transport where host languages need matching struct definitions.

```bash
rustbridge generate-header --source <SOURCE> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `-s, --source <SOURCE>` | Path to Rust source file containing `#[repr(C)]` structs |
| `-o, --output <OUTPUT>` | Output path for generated C header (default: `messages.h`) |
| `-v, --verify` | Verify the generated header compiles with a C compiler |

**Example:**

```bash
rustbridge generate-header --source src/binary_messages.rs --output include/messages.h --verify
```

See [TRANSPORT.md](./TRANSPORT.md) for details on binary transport and when C headers are needed.

---

## Common Workflows

### Local development

```bash
rustbridge new my-plugin --all        # Scaffold with all consumers
cd my-plugin
cargo build --release                  # Build the native library
rustbridge pack --no-sign              # Quick unsigned bundle
```

### Dev/release pipeline

```bash
cargo build --release && cargo build   # Build release + debug
rustbridge pack --dev                  # Dev bundle (both variants, unsigned)
# ... test locally ...
rustbridge promote target/bundle/my-plugin-0.1.0-dev.rbp   # Signed release
```

### CI multi-platform merge

```bash
# On each CI runner:
cargo build --release
rustbridge pack --no-sign

# After all platforms complete:
rustbridge bundle combine \
  --output my-plugin-1.0.0.rbp \
  --sign-key ~/.rustbridge/signing.key \
  linux-build/my-plugin-*.rbp \
  macos-build/my-plugin-*.rbp \
  windows-build/my-plugin-*.rbp
```

## Related Documentation

- [GETTING_STARTED.md](./GETTING_STARTED.md) - End-to-end walkthrough using the CLI
- [BUNDLE_FORMAT.md](./BUNDLE_FORMAT.md) - `.rbp` archive structure and manifest schema
- [TRANSPORT.md](./TRANSPORT.md) - JSON and binary transport (context for `generate-header`)
- [Tutorial 05: Production Bundles](./tutorials/05-production-bundles/README.md) - Code signing, schemas, SBOMs
- [Tutorial 06: Cross-Compilation](./tutorials/06-cross-compilation/README.md) - Multi-platform builds
