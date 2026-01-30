# Section 3: Cross-Compilation

In this section, you'll cross-compile your plugin from a single machine to multiple target platforms.

## When to Cross-Compile

Cross-compilation is useful when:

- You don't have access to target hardware
- You want to build everything locally
- Your CI doesn't support all targets

**Caveat**: Cross-compilation can be tricky with system dependencies. For most projects, native CI builds are simpler.

## Using `cross`

[cross](https://github.com/cross-rs/cross) may be the easiest way to cross-compile Rust projects (I haven't tried it).
It uses Docker containers with pre-configured toolchains.

### Install cross

```bash
cargo install cross --git https://github.com/cross-rs/cross
```

Requires Docker or Podman to be installed and running.

### Cross-Compile for Linux ARM64

From your development machine (any platform):

```bash
cd ~/rustbridge-workspace/json-plugin

# Build for Linux ARM64
cross build --release --target aarch64-unknown-linux-gnu
```

The output is at `target/aarch64-unknown-linux-gnu/release/libjson_plugin.so`.

### Cross-Compile for Multiple Targets

```bash
# Linux x86_64
cross build --release --target x86_64-unknown-linux-gnu

# Linux ARM64
cross build --release --target aarch64-unknown-linux-gnu

# Linux MUSL (statically linked)
cross build --release --target x86_64-unknown-linux-musl
```

### Create Bundle from Cross-Compiled Libraries

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/x86_64-unknown-linux-gnu/release/libjson_plugin.so \
  --lib linux-aarch64:target/aarch64-unknown-linux-gnu/release/libjson_plugin.so \
  --output json-plugin-linux.rbp
```

## Cross-Compiling Without cross

For targets not supported by `cross`, or to avoid Docker:

### Linux to Linux ARM64

Install the cross-compilation toolchain:

```bash
# Ubuntu/Debian
sudo apt install gcc-aarch64-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-gnu
```

Configure cargo to use the linker (create or edit `.cargo/config.toml`):

```toml
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
```

Build:

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

### macOS Cross-Architecture

On Apple Silicon, build for Intel:

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

On Intel Mac, build for Apple Silicon:

```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

No additional toolchain needed - Xcode handles it.

### Linux to Windows (mingw)

```bash
# Ubuntu/Debian
sudo apt install mingw-w64

# Add target
rustup target add x86_64-pc-windows-gnu
```

Configure linker:

```toml
# .cargo/config.toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

Build:

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

**Note**: This produces GNU ABI binaries. For MSVC ABI (recommended for Windows), build natively or use CI.

## Cross-Compilation Challenges

### C Dependencies

If your plugin depends on C libraries (via `-sys` crates), cross-compilation becomes harder:

| Dependency  | Challenge                                    |
|-------------|----------------------------------------------|
| OpenSSL     | Need cross-compiled OpenSSL or use `rustls`  |
| SQLite      | Need cross-compiled SQLite or static linking |
| System libs | May not be available for target              |

**Solutions**:

1. Use pure-Rust alternatives (e.g. `rustls` instead of `openssl`)
2. Vendor and statically link dependencies
3. Use `cross` containers (pre-configured)
4. Build natively on CI

### Pure Rust Projects

rustbridge plugins that only use Rust crates (no C dependencies) cross-compile easily:

```toml
# Cargo.toml - these are all pure Rust
[dependencies]
rustbridge = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

## Build Script for Cross-Compilation

```bash
#!/bin/bash
# cross-build.sh - Build for all Linux targets using cross
set -e

VERSION=${1:-"1.0.0"}
TARGETS=(
    "x86_64-unknown-linux-gnu:linux-x86_64"
    "aarch64-unknown-linux-gnu:linux-aarch64"
)

for TARGET_PAIR in "${TARGETS[@]}"; do
    RUST_TARGET="${TARGET_PAIR%%:*}"
    PLATFORM_ID="${TARGET_PAIR##*:}"

    echo "Building for $RUST_TARGET..."
    cross build --release --target "$RUST_TARGET"
done

# Create combined bundle
LIBS=""
for TARGET_PAIR in "${TARGETS[@]}"; do
    RUST_TARGET="${TARGET_PAIR%%:*}"
    PLATFORM_ID="${TARGET_PAIR##*:}"
    LIBS="$LIBS --lib ${PLATFORM_ID}:target/${RUST_TARGET}/release/libjson_plugin.so"
done

rustbridge bundle create \
    --name json-plugin \
    --version "$VERSION" \
    $LIBS \
    --output json-plugin-linux-$VERSION.rbp

echo "Created json-plugin-linux-$VERSION.rbp"
```

## Verifying Cross-Compiled Binaries

Check the binary format:

```bash
# Check architecture
file target/aarch64-unknown-linux-gnu/release/libjson_plugin.so
# Output: ELF 64-bit LSB shared object, ARM aarch64, ...

file target/x86_64-unknown-linux-gnu/release/libjson_plugin.so
# Output: ELF 64-bit LSB shared object, x86-64, ...
```

Check dependencies:

```bash
# For Linux binaries
readelf -d target/x86_64-unknown-linux-gnu/release/libjson_plugin.so | grep NEEDED
```

## Summary

Cross-compilation options:

| Method               | Best For         | Complexity |
|----------------------|------------------|------------|
| `cross`              | Linux targets    | Low        |
| Native cross-compile | macOS both archs | Low        |
| Manual toolchain     | Specific needs   | High       |

For production releases, building natively on each target platform (whether locally or in CI) is often simpler and more reliable than cross-compilation.

## What's Next?

You've completed the cross-compilation tutorial. You now have the tools to build multi-platform bundles using:
- Native toolchains on each platform
- Cross-compilation with `cross` or manual toolchains
- Bundle combining with `rustbridge bundle combine`

See the [Appendix: Java JNI](../appendix-java-jni/README.md) for Java 17-20 support without FFM.
