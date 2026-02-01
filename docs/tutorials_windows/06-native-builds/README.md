# Chapter 6: Native Builds

In this chapter, you'll build your plugins for Windows platforms and create production bundles.

> **Note**: This chapter focuses on native Windows builds. For cross-compilation to Linux/macOS, see the [Linux tutorials](../tutorials/06-cross-compilation/README.md) or use CI/CD with platform-specific runners.

## What You'll Build

Native Windows bundles for:
- Windows x64 (x86_64)
- Windows ARM64 (if you have an ARM device)

## Prerequisites

- Completed [Chapter 3: JSON Plugin](../03-json-plugin/README.md)
- Visual Studio Build Tools installed

## Building for Windows x64

This is the default target on most Windows machines:

```powershell
cd $env:USERPROFILE\rustbridge-workspace\json-plugin

# Build for the current platform (typically x64)
cargo build --release

# Create bundle
rustbridge bundle create `
  --name json-plugin `
  --version 0.1.0 `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --output json-plugin-windows-x86_64.rbp
```

## Building for Windows ARM64

If you're on an ARM64 Windows device (Surface Pro X, etc.):

```powershell
# Build natively on ARM64
cargo build --release

# Create bundle
rustbridge bundle create `
  --name json-plugin `
  --version 0.1.0 `
  --lib windows-aarch64:target\release\json_plugin.dll `
  --output json-plugin-windows-aarch64.rbp
```

## Cross-Compiling to ARM64 from x64

If you're on x64 and want to build for ARM64 Windows:

```powershell
# Add the ARM64 target
rustup target add aarch64-pc-windows-msvc

# Build for ARM64
cargo build --release --target aarch64-pc-windows-msvc

# Create bundle
rustbridge bundle create `
  --name json-plugin `
  --version 0.1.0 `
  --lib windows-aarch64:target\aarch64-pc-windows-msvc\release\json_plugin.dll `
  --output json-plugin-windows-aarch64.rbp
```

> **Note**: Cross-compiling to ARM64 requires the ARM64 build tools from Visual Studio.

## Combining Platform Bundles

After building for both platforms, combine them:

```powershell
rustbridge bundle combine `
  json-plugin-windows-x86_64.rbp `
  json-plugin-windows-aarch64.rbp `
  --output json-plugin-0.1.0.rbp `
  --sign-key $env:USERPROFILE\.rustbridge\signing.key
```

## Verify the Combined Bundle

```powershell
rustbridge bundle list json-plugin-0.1.0.rbp
```

```
json-plugin-0.1.0.rbp
├── manifest.json
├── manifest.json.minisig
└── lib\
    ├── windows-x86_64\
    │   └── release\
    │       └── json_plugin.dll
    └── windows-aarch64\
        └── release\
            └── json_plugin.dll
```

## Build Script

Create `build-all.ps1`:

```powershell
param(
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

Write-Host "Building json-plugin v$Version for Windows platforms" -ForegroundColor Cyan

# Build for x64
Write-Host "`nBuilding for Windows x64..." -ForegroundColor Yellow
cargo build --release

# Create x64 bundle
rustbridge bundle create `
  --name json-plugin `
  --version $Version `
  --lib windows-x86_64:target\release\json_plugin.dll `
  --output "json-plugin-windows-x86_64.rbp"

# Check if ARM64 target is available
$hasArm64 = (rustup target list --installed) -contains "aarch64-pc-windows-msvc"

if ($hasArm64) {
    Write-Host "`nBuilding for Windows ARM64..." -ForegroundColor Yellow
    cargo build --release --target aarch64-pc-windows-msvc

    rustbridge bundle create `
      --name json-plugin `
      --version $Version `
      --lib windows-aarch64:target\aarch64-pc-windows-msvc\release\json_plugin.dll `
      --output "json-plugin-windows-aarch64.rbp"

    # Combine both
    Write-Host "`nCombining bundles..." -ForegroundColor Yellow
    rustbridge bundle combine `
      json-plugin-windows-x86_64.rbp `
      json-plugin-windows-aarch64.rbp `
      --output "json-plugin-$Version.rbp"

    Remove-Item json-plugin-windows-x86_64.rbp
    Remove-Item json-plugin-windows-aarch64.rbp
} else {
    Write-Host "`nARM64 target not installed. Creating x64-only bundle." -ForegroundColor Yellow
    Move-Item "json-plugin-windows-x86_64.rbp" "json-plugin-$Version.rbp"
}

Write-Host "`nCreated json-plugin-$Version.rbp" -ForegroundColor Green
rustbridge bundle list "json-plugin-$Version.rbp"
```

## CI/CD with GitHub Actions

For cross-platform builds, use GitHub Actions:

```yaml
name: Build

on: [push, pull_request]

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            platform: windows-x86_64
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            platform: linux-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            platform: darwin-aarch64

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Create bundle
        run: |
          rustbridge bundle create \
            --name json-plugin \
            --version ${{ github.ref_name }} \
            --lib ${{ matrix.platform }}:target/${{ matrix.target }}/release/json_plugin* \
            --output json-plugin-${{ matrix.platform }}.rbp

      - uses: actions/upload-artifact@v4
        with:
          name: bundle-${{ matrix.platform }}
          path: json-plugin-${{ matrix.platform }}.rbp

  combine:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4

      - name: Combine bundles
        run: |
          rustbridge bundle combine \
            bundle-*/json-plugin-*.rbp \
            --output json-plugin-${{ github.ref_name }}.rbp
```

## Summary

For Windows development:

1. **x64 builds** work out of the box
2. **ARM64 builds** require the ARM64 Visual Studio tools
3. **Cross-platform builds** are best done with CI/CD
4. **Bundle combining** creates single-file distribution

## What's Next?

Continue to [Chapter 7: Backpressure Queues](../07-backpressure-queues/README.md) to implement bounded queues with flow control.
