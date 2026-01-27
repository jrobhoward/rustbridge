# Packaging Plugins

This guide covers advanced bundle operations: creating multi-platform bundles, combining bundles from CI builds, creating slimmed releases, and code signing.

For creating a basic single-platform bundle, see [Creating Plugins](../creating-plugins/README.md).

## Prerequisites

- **rustbridge CLI** installed: `cargo install rustbridge-cli`
- One or more compiled native libraries (`.so`, `.dylib`, `.dll`)

## Bundle Basics

A `.rbp` bundle is a ZIP archive containing:
- `manifest.json` - Metadata, checksums, build info
- `lib/{platform}/{variant}/` - Native libraries
- `schema/` - Optional message schemas
- `docs/` - Optional documentation

### Platform Identifiers

| Platform | Identifier | Library Name |
|----------|------------|--------------|
| Linux x86_64 | `linux-x86_64` | `lib{name}.so` |
| Linux ARM64 | `linux-aarch64` | `lib{name}.so` |
| macOS Intel | `darwin-x86_64` | `lib{name}.dylib` |
| macOS Apple Silicon | `darwin-aarch64` | `lib{name}.dylib` |
| Windows x64 | `windows-x86_64` | `{name}.dll` |
| Windows ARM64 | `windows-aarch64` | `{name}.dll` |

### Variants

Each platform can have multiple **variants** (different build configurations):
- `release` - Optimized production build (mandatory, default)
- `debug` - Debug symbols, no optimization
- Custom variants like `nightly`, `opt-size`, etc.

## Creating Multi-Platform Bundles

### Single Command (Cross-Compile)

If you've cross-compiled for multiple targets:

```bash
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/x86_64-unknown-linux-gnu/release/libmy_plugin.so \
  --lib linux-aarch64:target/aarch64-unknown-linux-gnu/release/libmy_plugin.so \
  --lib darwin-x86_64:target/x86_64-apple-darwin/release/libmy_plugin.dylib \
  --lib darwin-aarch64:target/aarch64-apple-darwin/release/libmy_plugin.dylib \
  --lib windows-x86_64:target/x86_64-pc-windows-msvc/release/my_plugin.dll \
  --output my-plugin-1.0.0.rbp
```

### Including Multiple Variants

Include both release and debug variants:

```bash
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:release:target/release/libmy_plugin.so \
  --lib linux-x86_64:debug:target/debug/libmy_plugin.so \
  --lib darwin-aarch64:release:target/release/libmy_plugin.dylib \
  --lib darwin-aarch64:debug:target/debug/libmy_plugin.dylib \
  --output my-plugin-1.0.0.rbp
```

Format: `--lib PLATFORM:VARIANT:PATH` or `--lib PLATFORM:PATH` (defaults to release)

## Combining Bundles

When building on CI, each platform typically produces its own bundle. Combine them into one:

```bash
# Combine bundles from different CI jobs
rustbridge bundle combine \
  my-plugin-linux.rbp \
  my-plugin-macos.rbp \
  my-plugin-windows.rbp \
  --output my-plugin-1.0.0.rbp
```

### Schema Validation

By default, combining validates that all bundles have matching schemas:

```bash
# Error if schemas differ (default)
rustbridge bundle combine ... --schema-mismatch error

# Warn but continue
rustbridge bundle combine ... --schema-mismatch warn

# Skip schema check entirely
rustbridge bundle combine ... --schema-mismatch ignore
```

### Re-Signing Combined Bundles

```bash
rustbridge bundle combine \
  linux.rbp macos.rbp windows.rbp \
  --output combined.rbp \
  --sign-key ~/.rustbridge/signing.key
```

## Creating Slimmed Bundles

Extract a subset for production deployment:

```bash
# Keep only release variant, specific platforms
rustbridge bundle slim \
  --input developer-bundle.rbp \
  --output production.rbp \
  --platforms linux-x86_64,darwin-aarch64 \
  --variants release

# Exclude documentation
rustbridge bundle slim \
  --input full.rbp \
  --output minimal.rbp \
  --variants release \
  --exclude-docs
```

### Re-Signing Slimmed Bundles

```bash
rustbridge bundle slim \
  --input developer.rbp \
  --output production.rbp \
  --variants release \
  --sign-key ~/.rustbridge/signing.key
```

## Code Signing

Code signing ensures bundle authenticity and integrity. rustbridge uses [minisign](https://jedisct1.github.io/minisign/) for signatures.

### Generate a Signing Key

```bash
rustbridge keygen --output ~/.rustbridge/signing.key
```

This creates:
- `~/.rustbridge/signing.key` - Secret key (keep secure!)
- `~/.rustbridge/signing.key.pub` - Public key (distribute with your plugin)

### Sign During Bundle Creation

```bash
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --sign-key ~/.rustbridge/signing.key \
  --output my-plugin-1.0.0.rbp
```

### What Gets Signed

- `manifest.json` → `manifest.json.minisig`
- Each library file → `lib/{platform}/{variant}/*.minisig`

### Verification

Host loaders verify signatures by default:

```java
// Java - verification enabled by default
BundleLoader.builder()
    .bundlePath("my-plugin.rbp")
    .verifySignatures(true)  // default
    .build();

// Override public key (defense in depth)
BundleLoader.builder()
    .bundlePath("my-plugin.rbp")
    .publicKey("RWS...")  // trusted key, not from manifest
    .build();
```

## Adding Schemas

Include schema files for documentation and type generation:

```bash
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --schema schema/messages.json:messages.json \
  --schema schema/messages.h:messages.h \
  --output my-plugin-1.0.0.rbp
```

Format: `--schema SOURCE_PATH:ARCHIVE_NAME`

### Auto-Generate Schemas

Generate schemas from Rust source during bundle creation:

```bash
# Generate C header from Rust structs
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --generate-header src/messages.rs:messages.h \
  --output my-plugin-1.0.0.rbp

# Generate JSON Schema
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --generate-schema src/messages.rs:schema.json \
  --output my-plugin-1.0.0.rbp
```

## Adding License Notices

Include third-party license notices:

```bash
rustbridge bundle create \
  --name my-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libmy_plugin.so \
  --notices NOTICES.txt \
  --output my-plugin-1.0.0.rbp
```

The file will be included at `docs/NOTICES.txt` in the bundle.

## Build Metadata

By default, bundles include build metadata (git commit, compiler version, timestamp):

```bash
# Include metadata (default)
rustbridge bundle create ...

# Skip metadata collection
rustbridge bundle create ... --no-metadata
```

Metadata is stored in `manifest.json`:

```json
{
  "build_info": {
    "built_by": "GitHub Actions",
    "built_at": "2025-01-26T10:30:00Z",
    "host": "x86_64-unknown-linux-gnu",
    "compiler": "rustc 1.85.0",
    "git": {
      "commit": "abc123...",
      "branch": "main",
      "dirty": false
    }
  }
}
```

## Inspecting Bundles

### List Contents

```bash
rustbridge bundle list my-plugin.rbp
```

### Detailed Info

```bash
# Show build metadata
rustbridge bundle list my-plugin.rbp --show-build

# Show all variants
rustbridge bundle list my-plugin.rbp --show-variants
```

### Extract a Library

```bash
# Extract for current platform (release variant)
rustbridge bundle extract my-plugin.rbp --output ./lib/

# Extract specific platform/variant
rustbridge bundle extract my-plugin.rbp \
  --platform linux-x86_64 \
  --variant debug \
  --output ./lib/
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build Plugin

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            platform: linux-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            platform: darwin-aarch64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            platform: windows-x86_64

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Create Bundle
        run: |
          cargo install rustbridge-cli
          rustbridge bundle create \
            --name my-plugin \
            --version ${{ github.ref_name }} \
            --lib ${{ matrix.platform }}:target/${{ matrix.target }}/release/libmy_plugin.* \
            --output my-plugin-${{ matrix.platform }}.rbp

      - uses: actions/upload-artifact@v4
        with:
          name: bundle-${{ matrix.platform }}
          path: my-plugin-*.rbp

  combine:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4
        with:
          pattern: bundle-*
          merge-multiple: true

      - name: Combine Bundles
        run: |
          cargo install rustbridge-cli
          rustbridge bundle combine \
            my-plugin-linux-x86_64.rbp \
            my-plugin-darwin-aarch64.rbp \
            my-plugin-windows-x86_64.rbp \
            --output my-plugin-${{ github.ref_name }}.rbp

      - uses: actions/upload-artifact@v4
        with:
          name: combined-bundle
          path: my-plugin-*.rbp
```

### Signing in CI

Store your signing key as a GitHub secret:

```yaml
- name: Create Signed Bundle
  env:
    SIGNING_KEY: ${{ secrets.RUSTBRIDGE_SIGNING_KEY }}
  run: |
    echo "$SIGNING_KEY" > signing.key
    rustbridge bundle create \
      --name my-plugin \
      --version ${{ github.ref_name }} \
      --lib linux-x86_64:target/release/libmy_plugin.so \
      --sign-key signing.key \
      --output my-plugin.rbp
    rm signing.key
```

## Next Steps

- **[Bundle Format Specification](../BUNDLE_FORMAT.md)** - Detailed format reference
- **[Using Plugins](../using-plugins/README.md)** - Load bundles from host languages
- **[Code Generation](../CODE_GENERATION.md)** - Generate typed clients from schemas
