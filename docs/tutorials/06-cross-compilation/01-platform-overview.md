# Section 1: Platform Overview

In this section, you'll learn about supported platforms, library naming conventions, and Rust target triples.

## Supported Platforms

rustbridge supports six platform configurations:

| Platform            | Identifier        | Rust Target                 | Library Name      |
|---------------------|-------------------|-----------------------------|-------------------|
| Linux x86_64        | `linux-x86_64`    | `x86_64-unknown-linux-gnu`  | `lib{name}.so`    |
| Linux ARM64         | `linux-aarch64`   | `aarch64-unknown-linux-gnu` | `lib{name}.so`    |
| macOS Intel         | `darwin-x86_64`   | `x86_64-apple-darwin`       | `lib{name}.dylib` |
| macOS Apple Silicon | `darwin-aarch64`  | `aarch64-apple-darwin`      | `lib{name}.dylib` |
| Windows x64         | `windows-x86_64`  | `x86_64-pc-windows-msvc`    | `{name}.dll`      |
| Windows ARM64       | `windows-aarch64` | `aarch64-pc-windows-msvc`   | `{name}.dll`      |

## Library Naming Conventions

### Unix (Linux, macOS)

Libraries use the `lib` prefix:

- Source: `json_plugin` (crate name with underscores)
- Output: `libjson_plugin.so` or `libjson_plugin.dylib`

### Windows

No `lib` prefix:

- Source: `json_plugin`
- Output: `json_plugin.dll`

## Rust Target Triples

A Rust target triple has three or four parts: `{arch}-{vendor}-{os}[-{env}]`

| Part     | Example                      | Description                |
|----------|------------------------------|----------------------------|
| `arch`   | `x86_64`, `aarch64`          | CPU architecture           |
| `vendor` | `unknown`, `apple`, `pc`     | Vendor/manufacturer        |
| `os`     | `linux`, `darwin`, `windows` | Operating system           |
| `env`    | `gnu`, `msvc`                | ABI/environment (optional) |

> **Note**: `musl` targets (`x86_64-unknown-linux-musl`) do not support shared libraries (`cdylib`), so they cannot be used for rustbridge plugins.

### Common Targets

```bash
# List installed targets
rustup target list --installed

# List all available targets
rustup target list
```

## Adding Rust Targets

Install targets for cross-compilation:

```bash
# Linux ARM64
rustup target add aarch64-unknown-linux-gnu

# macOS (both architectures)
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin

# Windows
rustup target add x86_64-pc-windows-msvc
```

## Platform Detection

rustbridge loaders automatically detect the current platform:

### How Detection Works

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Detect OS      │ ──▶ │  Detect Arch    │ ──▶ │  Build Key      │
│  (linux/darwin/ │     │  (x86_64/       │     │  (linux-x86_64) │
│   windows)      │     │   aarch64)      │     │                 │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

### Java Detection

```java
// Simplified - actual implementation handles edge cases
String os = System.getProperty("os.name").toLowerCase();
String arch = System.getProperty("os.arch");

String osKey = os.contains("linux") ? "linux" :
               os.contains("mac") ? "darwin" :
               os.contains("win") ? "windows" : "unknown";

String archKey = arch.equals("amd64") || arch.equals("x86_64") ? "x86_64" :
                 arch.equals("aarch64") || arch.equals("arm64") ? "aarch64" : "unknown";

String platformKey = osKey + "-" + archKey;  // e.g., "linux-x86_64"
```

### Python Detection

```python
import platform

os_map = {"Linux": "linux", "Darwin": "darwin", "Windows": "windows"}
arch_map = {"x86_64": "x86_64", "AMD64": "x86_64", "arm64": "aarch64", "aarch64": "aarch64"}

os_key = os_map.get(platform.system(), "unknown")
arch_key = arch_map.get(platform.machine(), "unknown")

platform_key = f"{os_key}-{arch_key}"  # e.g., "darwin-aarch64"
```

## Build Output Locations

When you build with `cargo build --release`:

```
target/
└── release/
    └── libjson_plugin.so    ← Native platform
```

When you build with a specific target:

```bash
cargo build --release --target x86_64-unknown-linux-gnu
```

```
target/
└── x86_64-unknown-linux-gnu/
    └── release/
        └── libjson_plugin.so
```

## Bundle Library Paths

Inside a bundle, libraries are organized by platform and variant:

```
my-plugin-1.0.0.rbp
└── lib/
    ├── linux-x86_64/
    │   ├── release/
    │   │   └── libmy_plugin.so
    │   └── debug/                 ← Optional variant
    │       └── libmy_plugin.so
    └── darwin-aarch64/
        └── release/
            └── libmy_plugin.dylib
```

## Minimum Platform Requirements

| Platform | Minimum Version | Notes                   |
|----------|-----------------|-------------------------|
| Linux    | glibc 2.17+     | CentOS 7, Ubuntu 14.04+ |
| macOS    | 10.12+          | Sierra or later         |
| Windows  | Windows 10+     | MSVC runtime required   |

## Summary

Key concepts:

- **Platform identifiers**: `{os}-{arch}` (e.g., `linux-x86_64`)
- **Rust targets**: `{arch}-{vendor}-{os}[-{env}]` (e.g., `x86_64-unknown-linux-gnu`)
- **Library names**: Unix uses `lib` prefix, Windows doesn't
- **Build output**: `target/{target}/release/` for cross-compilation

## What's Next?

In the next section, you'll build natively on each platform.

[Continue to Section 2: Native Toolchains →](./02-native-toolchains.md)
