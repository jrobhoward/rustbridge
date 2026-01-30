# Section 3: Cross-Compilation

In this section, you'll cross-compile your plugin for ARM64 and create a portable bundle that runs on both x86_64 and
ARM64 Linux systems.

## Goal

Build a single `.rbp` bundle and Java JAR that work on:

- Your development machine (x86_64)
- A Raspberry Pi or other ARM64 device

## Prerequisites

This tutorial assumes you're on **Ubuntu 22.04 x86_64**. The concepts apply to other distributions with minor
adjustments.

You should have completed the json-plugin from earlier tutorials:

```
~/rustbridge-workspace/json-plugin/
```

## Step 1: Build for Native x86_64

First, build the plugin for your local architecture:

```bash
cd ~/rustbridge-workspace/json-plugin
cargo build --release
```

The library is at `target/release/libjson_plugin.so`.

## Step 2: Set Up ARM64 Cross-Compilation

Install the ARM64 cross-compiler and add the Rust target:

```bash
# Install cross-compiler
sudo apt install gcc-aarch64-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-gnu
```

Configure Cargo to use the cross-linker. Create or edit `.cargo/config.toml` in your project:

```bash
mkdir -p .cargo
cat > .cargo/config.toml << 'EOF'
[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"
EOF
```

## Step 3: Build for ARM64

```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

The ARM64 library is at `target/aarch64-unknown-linux-gnu/release/libjson_plugin.so`.

Verify the architectures:

```bash
file target/release/libjson_plugin.so
# ELF 64-bit LSB shared object, x86-64, ...

file target/aarch64-unknown-linux-gnu/release/libjson_plugin.so
# ELF 64-bit LSB shared object, ARM aarch64, ...
```

## Step 4: Create Multi-Platform Bundle

Bundle both libraries into a single signed `.rbp` file:

```bash
rustbridge bundle create \
  --name json-plugin \
  --version 1.0.0 \
  --lib linux-x86_64:target/release/libjson_plugin.so \
  --lib linux-aarch64:target/aarch64-unknown-linux-gnu/release/libjson_plugin.so \
  --sign-key ~/.rustbridge/signing.key \
  --output json-plugin-1.0.0.rbp
```

> **Note**: If you don't have a signing key, create one with `rustbridge keygen`.

Verify the bundle contains both platforms:

```bash
rustbridge bundle list json-plugin-1.0.0.rbp
```

```
json-plugin-1.0.0.rbp
├── manifest.json
└── lib/
    ├── linux-x86_64/
    │   └── release/
    │       └── libjson_plugin.so
    └── linux-aarch64/
        └── release/
            └── libjson_plugin.so
```

## Step 5: Build the Java Consumer JAR

Navigate to your Java consumer project and build a fat JAR with all dependencies:

```bash
cd ~/rustbridge-workspace/json-plugin/consumers/java-ffm
```

Add the Shadow plugin to `build.gradle.kts` if not already present:

```kotlin
plugins {
    java
    application
    id("com.gradleup.shadow") version "8.3.6"
}

application {
    mainClass.set("com.example.Main")
}
```

Build the fat JAR:

```bash
./gradlew shadowJar
```

The JAR is at `build/libs/json-plugin-java-ffm-1.0.0-all.jar` (the name includes the project name and version).

## Step 6: Run Locally (x86_64)

Test on your development machine:

```bash
cd ~/rustbridge-workspace/json-plugin

# Run with the bundle
java --enable-preview --enable-native-access=ALL-UNNAMED \
  -jar consumers/java-ffm/build/libs/json-plugin-java-ffm-1.0.0-all.jar \
  json-plugin-1.0.0.rbp
```

> **Note**: In Java 21, `--enable-preview` is required (FFM is preview), and `--enable-native-access=ALL-UNNAMED`
> suppresses native access warnings. In Java 22+, FFM is stable so only `--enable-native-access` is needed.

The `BundleLoader` automatically extracts the correct library for your platform.

## Step 7: Deploy to Raspberry Pi (ARM64)

Copy the two files to your Raspberry Pi:

```bash
scp json-plugin-1.0.0.rbp pi@raspberrypi:~/
scp consumers/java-ffm/build/libs/json-plugin-java-ffm-1.0.0-all.jar pi@raspberrypi:~/
```

SSH into the Pi and run:

```bash
ssh pi@raspberrypi
cd ~
java --enable-preview --enable-native-access=ALL-UNNAMED \
  -jar json-plugin-java-ffm-1.0.0-all.jar json-plugin-1.0.0.rbp
```

The same JAR and `.rbp` file work on both platforms—the bundle loader extracts the ARM64 library automatically.

> **Tip**: On Java 22+, FFM is stable, so you only need `--enable-native-access=ALL-UNNAMED`.

## Alternative: Using `cross`

If you prefer Docker-based cross-compilation, the [cross](https://github.com/cross-rs/cross) tool provides
pre-configured containers:

```bash
cargo install cross --git https://github.com/cross-rs/cross
cross build --release --target aarch64-unknown-linux-gnu
```

This avoids manual toolchain setup but requires Docker or Podman.

## Cross-Compilation Limitations

Cross-compilation works well for **pure Rust** plugins. If your plugin uses C libraries (via `-sys` crates), you'll
need:

- Cross-compiled versions of those libraries, or
- Pure Rust alternatives (e.g., `rustls` instead of `openssl`), or
- Native builds on each target platform

The json-plugin uses only pure Rust dependencies, so it cross-compiles without issues.

## Summary

You've learned to:

1. Cross-compile a Rust plugin for ARM64 from x86_64
2. Bundle multiple architectures into a single `.rbp` file
3. Build a portable Java JAR
4. Deploy and run on different platforms with the same files

The combination of multi-platform bundles and Java's "write once, run anywhere" makes deployment simple—copy two files
and run.

## What's Next?

You've completed the cross-compilation tutorial! See the [Appendix: Java JNI](../appendix-java-jni/README.md) for Java
17-20 support without FFM.
