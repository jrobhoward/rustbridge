# Section 2: Native Toolchains

In this section, you'll build your plugin natively on each target platform.

## Why Native Builds?

Native builds are the **recommended approach** because:

- No cross-compilation toolchain setup
- Native system libraries link correctly
- Platform-specific optimizations work
- Easier debugging if issues arise

The trade-off is needing access to each platform (physical, VM, or CI).

## Building on Linux

### Linux x86_64

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cd ~/rustbridge-workspace/json-plugin
cargo build --release

# Create bundle
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --output json-plugin-linux-x86_64.rbp
```

### Linux ARM64 (aarch64)

On an ARM64 machine (Raspberry Pi, AWS Graviton, etc.):

```bash
cargo build --release

rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-aarch64:target/release/libjson_plugin.so \
  --output json-plugin-linux-aarch64.rbp
```

## Building on macOS

### macOS Apple Silicon (M1/M2/M3)

```bash
cargo build --release

rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib darwin-aarch64:target/release/libjson_plugin.dylib \
  --output json-plugin-darwin-aarch64.rbp
```

### macOS Intel

On an Intel Mac:

```bash
cargo build --release

rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib darwin-x86_64:target/release/libjson_plugin.dylib \
  --output json-plugin-darwin-x86_64.rbp
```

### Universal macOS Binary (Optional)

Build for both architectures on Apple Silicon:

```bash
# Add Intel target
rustup target add x86_64-apple-darwin

# Build both
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Create universal binary with lipo
lipo -create \
  target/aarch64-apple-darwin/release/libjson_plugin.dylib \
  target/x86_64-apple-darwin/release/libjson_plugin.dylib \
  -output libjson_plugin.dylib

# Bundle with universal binary
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib darwin-aarch64:target/aarch64-apple-darwin/release/libjson_plugin.dylib \
  --lib darwin-x86_64:target/x86_64-apple-darwin/release/libjson_plugin.dylib \
  --output json-plugin-darwin.rbp
```

## Building on Windows

### Windows x64

Open PowerShell or Command Prompt:

```powershell
# Build
cargo build --release

# Create bundle
rustbridge bundle create `
  --name json-plugin `
  --version 1.0.0 `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --output json-plugin-windows-x86_64.rbp
```

### Visual Studio Build Tools

If you don't have Visual Studio, install the Build Tools:

1. Download [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Install "Desktop development with C++"
3. Restart your terminal

## Combining Platform Bundles

After building on each platform, combine into one bundle:

```bash
rustbridge bundle combine \
  json-plugin-linux-x86_64.rbp \
  json-plugin-linux-aarch64.rbp \
  json-plugin-darwin-aarch64.rbp \
  json-plugin-darwin-x86_64.rbp \
  json-plugin-windows-x86_64.rbp \
  --output json-plugin-1.0.0.rbp \
  --sign-key ~/.rustbridge/signing.key
```

## Verify the Combined Bundle

```bash
rustbridge bundle list json-plugin-1.0.0.rbp
```

```
json-plugin-1.0.0.rbp
├── manifest.json
├── manifest.json.minisig
└── lib/
    ├── linux-x86_64/
    │   └── release/
    │       ├── libjson_plugin.so
    │       └── libjson_plugin.so.minisig
    ├── linux-aarch64/
    │   └── release/
    │       └── libjson_plugin.so
    ├── darwin-x86_64/
    │   └── release/
    │       └── libjson_plugin.dylib
    ├── darwin-aarch64/
    │   └── release/
    │       └── libjson_plugin.dylib
    └── windows-x86_64/
        └── release/
            └── json_plugin.dll
```

## Build Scripts

Create a build script for each platform:

### Linux/macOS (`build.sh`)

```bash
#!/bin/bash
set -e

VERSION=${1:-"1.0.0"}
PLATFORM=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map to rustbridge platform identifiers
case "$PLATFORM" in
  linux) OS="linux" ;;
  darwin) OS="darwin" ;;
  *) echo "Unsupported OS: $PLATFORM"; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64) ARCH_ID="x86_64" ;;
  aarch64|arm64) ARCH_ID="aarch64" ;;
  *) echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

PLATFORM_ID="${OS}-${ARCH_ID}"

# Build
cargo build --release

# Determine library extension
if [ "$OS" = "darwin" ]; then
  EXT="dylib"
else
  EXT="so"
fi

# Create bundle
rustbridge bundle create \
  --name json-plugin \
  --version "$VERSION" \
  --lib "${PLATFORM_ID}:target/release/libjson_plugin.${EXT}" \
  --output "json-plugin-${PLATFORM_ID}.rbp"

echo "Created json-plugin-${PLATFORM_ID}.rbp"
```

### Windows (`build.ps1`)

```powershell
param(
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"

# Build
cargo build --release

# Create bundle
rustbridge bundle create `
    --name json-plugin `
    --version $Version `
    --lib "windows-x86_64:target\release\json_plugin.dll" `
    --output "json-plugin-windows-x86_64.rbp"

Write-Host "Created json-plugin-windows-x86_64.rbp"
```

## Summary

Native builds are straightforward:

1. Install Rust on each target platform
2. Run `cargo build --release`
3. Create a platform-specific bundle
4. Combine all bundles into one with `rustbridge bundle combine`

## What's Next?

In the next section, you'll learn about cross-compilation for when native builds aren't practical.

[Continue to Section 3: Cross-Compilation →](./03-cross-compilation.md)
