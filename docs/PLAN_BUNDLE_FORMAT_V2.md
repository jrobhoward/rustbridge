# Plan: Evolve .rbp Bundle Format

## Overview

Evolve the rustbridge `.rbp` bundle format to support multi-variant libraries, bundle combining/slimming, rich build metadata, and schema contract validation.

**Key simplification**: Since v1.0 was never publicly released, we make breaking changes directly to the format (no backward compatibility layer needed).

## Key Use Cases

1. **Multi-variant bundles** - release/debug/nightly builds in one bundle
2. **Bundle combining** - merge platform-specific or variant-specific bundles
3. **Bundle slimming** - extract production subset from developer bundle
4. **Rich metadata** - git info, compiler version, SBOM, license notices
5. **Schema validation** - ensure combined bundles share same contract
6. **Non-Rust support** - language-agnostic spec for Go/C/C++ authors
7. **Project templates** - scaffold new plugins with working examples

---

## Phase 1: Specification Document

**File:** Update `docs/BUNDLE_FORMAT.md` directly (breaking change)

### Manifest Schema

```json
{
  "bundle_version": "1.0",
  "plugin": {
    "name": "my-plugin",
    "version": "1.0.0",
    "description": "Example plugin",
    "authors": ["Developer <dev@example.com>"],
    "license": "MIT",
    "repository": "https://github.com/example/my-plugin"
  },
  "platforms": {
    "linux-x86_64": {
      "variants": {
        "release": {
          "library": "lib/linux-x86_64/release/libplugin.so",
          "checksum": "sha256:...",
          "build": { ... }
        },
        "debug": {
          "library": "lib/linux-x86_64/debug/libplugin.so",
          "checksum": "sha256:...",
          "build": { ... }
        }
      }
    }
  },
  "build_info": { ... },
  "sbom": { ... },
  "schema_checksum": "sha256:...",
  "notices": "docs/NOTICES.txt",
  "api": { ... },
  "schemas": { ... },
  "public_key": "RWS..."
}
```

### Default Variant Rules

- **`release` variant is mandatory** for each platform
- **`release` is the implicit default** - loaders use it unless told otherwise
- No `default_variant` field needed - it's always `release`
- Additional variants (debug, nightly, opt-size, etc.) are optional

This simplifies the spec: every platform must have at least `variants.release`.

### Variant Build Metadata (Optional & Flexible)

The `build` field under each variant is **entirely optional** and uses a **flexible schema** to support non-Rust developers:

```json
{
  "variants": {
    "release": {
      "library": "lib/linux-x86_64/release/libplugin.so",
      "checksum": "sha256:...",
      "build": {
        // All fields optional - use what's relevant to your toolchain
        "profile": "release",           // Rust: release/debug
        "opt_level": "3",               // Rust: 0-3, s, z
        "features": ["json", "binary"], // Rust: cargo features
        "go_tags": ["production"],      // Go: build tags
        "cflags": "-O3 -march=native",  // C/C++: compiler flags
        "compiler": "go 1.22",          // Any: compiler version
        "notes": "Built with LTO"       // Any: freeform notes
      }
    }
  }
}
```

**For non-Rust developers**: The `build` object accepts any JSON key-value pairs. Only `library` and `checksum` are required per variant.

### Build Info (Optional)

The top-level `build_info` is **optional** - useful for traceability but not required.

```json
{
  "build_info": {
    "built_by": "GitHub Actions",
    "built_at": "2025-01-26T10:30:00Z",
    "host": "x86_64-unknown-linux-gnu",
    "compiler": "rustc 1.85.0",
    "rustbridge_version": "0.2.0",
    "git": {
      "commit": "a1b2c3d4e5f6",
      "branch": "main",
      "tag": "v1.0.0",
      "dirty": false
    }
  }
}
```

**Git info is optional** - developers not using git can omit the `git` section entirely.

### Archive Structure

```
my-plugin-1.0.0.rbp
├── manifest.json
├── manifest.json.minisig
├── lib/
│   ├── linux-x86_64/
│   │   ├── release/
│   │   │   └── libplugin.so
│   │   └── debug/                    # Optional variant
│   │       └── libplugin.so
│   └── darwin-aarch64/
│       └── release/
│           └── libplugin.dylib
├── schema/
├── docs/
│   └── NOTICES.txt
└── sbom/
    ├── sbom.cdx.json                 # CycloneDX (optional)
    └── sbom.spdx.json                # SPDX (optional)
```

---

## Phase 2: Rust Implementation

### 2.1 Metadata Collection via `built` Crate

Use the [`built`](https://crates.io/crates/built) crate instead of shell command scraping. It provides compile-time constants for:

**Always available:**
- Package version, name, authors, license
- Target triple, host triple, profile (release/debug)
- Rustc version, optimization level
- Enabled features
- CI detection (GitHub Actions, Travis, etc.)

**With `git2` feature:**
- Commit hash (full and short)
- Branch name, tag
- Dirty/clean status

**With `chrono` feature:**
- Build timestamp (RFC2822)

**Implementation:**
1. Add `built` with features `git2`, `chrono` to `rustbridge-cli`
2. Generate `built.rs` at build time via `build.rs`
3. Embed constants into bundles during `bundle create`

```toml
# Cargo.toml
[build-dependencies]
built = { version = "0.7", features = ["git2", "chrono"] }
```

This avoids shell scraping and works reliably across platforms.

### 2.2 Manifest Types (`crates/rustbridge-bundle/src/manifest.rs`)

```rust
/// Variant-specific library information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantInfo {
    pub library: String,
    pub checksum: String,
    /// Flexible build metadata - any JSON object
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<serde_json::Value>,
}

/// Platform with required release variant + optional others
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub variants: HashMap<String, VariantInfo>,
}

impl PlatformInfo {
    /// Get release variant (always present, validated on load)
    pub fn release(&self) -> &VariantInfo {
        self.variants.get("release").expect("release variant required")
    }

    /// Get variant by name, defaults to release
    pub fn variant(&self, name: &str) -> Option<&VariantInfo> {
        self.variants.get(name)
    }
}

/// Build information (all fields optional)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub built_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub built_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rustbridge_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,
}

/// Git information (all fields optional except commit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    pub commit: String,  // Required if git section present
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dirty: Option<bool>,
}

/// SBOM with paths to generated files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cyclonedx: Option<String>,  // e.g., "sbom/sbom.cdx.json"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spdx: Option<String>,       // e.g., "sbom/sbom.spdx.json"
}
```

### 2.3 Validation Rules

```rust
impl Manifest {
    pub fn validate(&self) -> BundleResult<()> {
        // ... existing checks ...

        // Each platform must have a "release" variant
        for (platform, info) in &self.platforms {
            if !info.variants.contains_key("release") {
                return Err(BundleError::InvalidManifest(format!(
                    "platform {platform}: 'release' variant is required"
                )));
            }
        }

        Ok(())
    }
}
```

---

## Phase 3: CLI Commands

### 3.1 Enhanced `bundle create`

```bash
# Simple: single release variant per platform
rustbridge bundle create \
  --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libplugin.so \
  --output my-plugin.rbp

# With debug variant
rustbridge bundle create \
  --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:release:target/release/libplugin.so \
  --lib linux-x86_64:debug:target/debug/libplugin.so \
  --output my-plugin.rbp

# With metadata and SBOM
rustbridge bundle create \
  --name my-plugin --version 1.0.0 \
  --lib linux-x86_64:target/release/libplugin.so \
  --sbom cyclonedx,spdx \
  --notices NOTICES.txt \
  --output my-plugin.rbp
```

**Arguments:**
```
--lib PLATFORM:PATH              # Adds as "release" variant
--lib PLATFORM:VARIANT:PATH      # Adds as named variant
--no-metadata                    # Skip build_info collection
--sbom FORMAT[,FORMAT]           # Generate SBOM: cyclonedx, spdx, or both
--notices PATH                   # License notices file
```

### 3.2 `bundle combine` - Schema Enforcement

```bash
# Default: ERROR if schemas don't match
rustbridge bundle combine \
  -o combined.rbp \
  linux.rbp macos.rbp windows.rbp

# Warn only (for development)
rustbridge bundle combine \
  -o combined.rbp \
  --schema-mismatch warn \
  linux.rbp macos.rbp windows.rbp

# Skip schema check entirely
rustbridge bundle combine \
  -o combined.rbp \
  --schema-mismatch ignore \
  linux.rbp macos.rbp windows.rbp
```

**Behavior:**
- Default (`--schema-mismatch error`): Fails if `schema_checksum` differs between bundles
- `--schema-mismatch warn`: Logs warning, continues
- `--schema-mismatch ignore`: No check

### 3.3 `bundle slim`

```bash
rustbridge bundle slim \
  -i developer.rbp \
  -o production.rbp \
  --platforms linux-x86_64,darwin-aarch64 \
  --variants release \
  --exclude-docs \
  --sign-key private.key
```

### 3.4 SBOM Generation

Use [`cyclonedx-bom`](https://crates.io/crates/cyclonedx-bom) library for programmatic generation:

```bash
--sbom cyclonedx        # Generate sbom/sbom.cdx.json
--sbom spdx             # Generate sbom/sbom.spdx.json
--sbom cyclonedx,spdx   # Generate both
```

Both CycloneDX and SPDX can be generated simultaneously - they're not mutually exclusive.

**Implementation:**
- Use `cyclonedx-bom` crate for CycloneDX 1.5 JSON output
- Use `cargo_metadata` to read Cargo.lock dependencies
- For SPDX, format manually or use community crate if available

### 3.5 Project Templates (`rustbridge new`)

Enhance the existing `rustbridge new` command to generate a complete, working plugin:

```bash
rustbridge new my-plugin
rustbridge new my-plugin --template minimal    # Bare minimum
rustbridge new my-plugin --template full       # With examples, tests, CI
```

**Generated structure:**
```
my-plugin/
├── Cargo.toml                    # Configured for cdylib
├── src/
│   └── lib.rs                    # Working plugin with echo handler
├── rustbridge.toml               # Plugin manifest
├── .github/
│   └── workflows/
│       └── build.yml             # CI template (optional)
├── NOTICES.txt                   # License attribution template
└── README.md
```

**Template sources:** Embed templates in the CLI binary or fetch from a templates directory in the repo.

---

## Phase 4: Host Loader Updates

### 4.1 All Languages - Simplified API

Since `release` is always the default:

```java
// Java - loads release variant automatically
Plugin plugin = BundleLoader.load("plugin.rbp");

// Load specific variant
Plugin plugin = BundleLoader.load("plugin.rbp", "debug");

// List available variants
List<String> variants = BundleLoader.listVariants("plugin.rbp", "linux-x86_64");
```

```csharp
// C# - loads release variant automatically
var plugin = BundleLoader.Load("plugin.rbp");

// Load specific variant
var plugin = BundleLoader.Load("plugin.rbp", variant: "debug");
```

```python
# Python - loads release variant automatically
plugin = BundleLoader.load("plugin.rbp")

# Load specific variant
plugin = BundleLoader.load("plugin.rbp", variant="debug")
```

---

## Phase 5: Documentation

1. Update `docs/BUNDLE_FORMAT.md` with new schema
2. Add "For Non-Rust Developers" section:
   - Manual bundle creation (zip + manifest.json)
   - Required FFI symbols table
   - Example Go/C plugin skeleton
3. Document variant conventions
4. Template usage guide

---

## Critical Files

| File | Changes |
|------|---------|
| `crates/rustbridge-bundle/src/manifest.rs` | New variant structure, flexible build metadata |
| `crates/rustbridge-bundle/src/builder.rs` | Variant support, `built` crate integration |
| `crates/rustbridge-bundle/src/loader.rs` | Variant extraction, release as default |
| `crates/rustbridge-cli/src/main.rs` | New commands: combine, slim |
| `crates/rustbridge-cli/src/bundle.rs` | Enhanced create with variants |
| `crates/rustbridge-cli/src/new.rs` | Project templates |
| `crates/rustbridge-cli/build.rs` | `built` crate integration |
| `rustbridge-java/.../BundleLoader.java` | Variant API |
| `rustbridge-csharp/.../BundleLoader.cs` | Variant API |
| `rustbridge-python/.../bundle_loader.py` | Variant API |
| `docs/BUNDLE_FORMAT.md` | Full specification update |

---

## Dependencies

Add to `crates/rustbridge-cli/Cargo.toml`:
```toml
[dependencies]
cyclonedx-bom = "0.5"          # SBOM generation
cargo_metadata = "0.18"         # Read Cargo.lock

[build-dependencies]
built = { version = "0.7", features = ["git2", "chrono"] }
```

---

## Verification

1. **Unit tests**: Manifest round-trips, validation (release required)
2. **Integration tests**:
   - Create bundle with release + debug variants
   - Load default (release) and explicit variant
   - Combine with matching/mismatching schemas
   - Slim to subset
3. **Template test**: `rustbridge new test-plugin && cd test-plugin && cargo build`
4. **Cross-language**: Create in Rust, load in Java/C#/Python

---

## Summary of Decisions

| Question | Decision |
|----------|----------|
| Backward compat? | No - make breaking changes to v1.0 |
| Default variant? | `release` is mandatory and implicit default |
| Build fields? | Flexible JSON object, all optional |
| Git info? | Optional, use `built` crate with `git2` feature |
| Schema mismatch in combine? | Error by default, `--schema-mismatch warn/ignore` flags |
| SBOM formats? | Both CycloneDX and SPDX can be generated |
| SBOM library? | `cyclonedx-bom` for programmatic generation |
| Templates? | Yes, enhance `rustbridge new` with working examples |

---

## References

- [`built` crate](https://docs.rs/built) - Compile-time build info
- [`cyclonedx-bom`](https://crates.io/crates/cyclonedx-bom) - SBOM generation library
- [`cargo-sbom`](https://docs.rs/crate/cargo-sbom) - Reference for SPDX/CycloneDX output (CLI only)
- [CycloneDX Rust Cargo](https://github.com/CycloneDX/cyclonedx-rust-cargo) - Official CycloneDX tooling
