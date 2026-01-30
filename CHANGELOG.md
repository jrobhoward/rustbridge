# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- `rustbridge new` now supports multi-language consumer generation with flags:
  - `--kotlin`, `--java-ffm`, `--java-jni`, `--csharp`, `--python`, `--all`
  - Generated consumers placed in `consumers/` subdirectory
- `templates/rust/` - New canonical Rust plugin template with placeholder substitution
- Tutorial Chapter 3: Building a JSON Plugin (scaffold, validate message, prettify message, error handling)
- Tutorial Chapter 4: Calling from Java (project setup, type-safe calls with records/Gson, error handling)
- Tutorial Chapter 5: Production Bundles (code signing, JSON schemas, build metadata, SBOM)
- Tutorial Chapter 6: Cross-Compilation (platform overview, native toolchains, cross-compilation)
- `examples/json-plugin/` - Reference implementation for JSON validation and prettification
- `--license PATH` flag for `rustbridge bundle create` to include the plugin's own LICENSE file
- `--metadata KEY=VALUE` flag for `rustbridge bundle create` for arbitrary custom metadata

### Changed
- `rustbridge new` now uses embedded templates via `include_str!` instead of `cargo-generate`
  - No external dependency on `cargo-generate` required
  - Templates are versioned with the CLI binary
- Tutorials and documentation updated to use `rustbridge new` instead of `cargo generate`
- Generated projects use Option B structure: Rust plugin at root, consumers in `consumers/`

### Removed
- `templates/plugin/` - Replaced by `templates/rust/`
- `templates/tutorial-plugin/` - No longer needed; tutorials use `rustbridge new` + `examples/regex-plugin` as reference

## [0.6.1] - 2025-01-29

### Added
- Tutorial system with step-by-step guides for building plugins
  - Chapter 1: Building a Regex Plugin (scaffold, matching, LRU cache, configuration)
  - Chapter 2: Calling from Kotlin (setup, JSON calls, logging, type-safe wrappers, benchmarking)
- `templates/tutorial-plugin/` - cargo-generate template with configurable features (regex, cache, config, logging)
- `examples/regex-plugin/` - Complete reference implementation with LRU caching and configuration
- Comprehensive tests for structured logging with key=value fields

### Changed
- Updated dependencies to latest versions:
  - `tokio` 1.43 → 1.49
  - `thiserror` 1.0 → 2.0
  - `darling` 0.20 → 0.23
  - `toml` 0.8 → 0.9
  - `uuid` 1.11 → 1.20
  - `once_cell` 1.19 → 1.21
  - `dashmap` 5.5 → 6.1
  - `zip` 2.2 → 7.2
  - `criterion` 0.5 → 0.8
- `templates/tutorial-plugin/` - Unified basic and completed templates with `completed` boolean option
  - Default (`false`) generates basic echo plugin (replaces manual `cp` of `templates/plugin`)
  - With `-d completed=true` generates full regex plugin with LRU caching
- Templates and examples now use proper `use` imports instead of fully qualified paths
- Templates generate clippy-clean code (fixed `field_reassign_with_default` warnings)
- `templates/plugin/` - Fixed tokio test dependency for standalone template usage
- Kotlin tutorials updated with correct API usage (FfmPluginLoader, LogCallback, PluginConfig)
- Tutorial Section 5 now demonstrates bundle variants (single .rbp with both debug and release builds)
- Added permissive licenses to `deny.toml`: Zlib, bzip2-1.0.6, CC0-1.0, MIT-0

### Fixed
- **CRITICAL**: Structured logging fields (e.g., `cache_size = 100`) now appear in log messages
  - `rustbridge-logging` MessageVisitor was discarding all fields except "message"
  - Now properly collects and formats all fields as `key=value` pairs
- **CRITICAL**: Log level from PluginConfig is now correctly applied during initialization
  - `rustbridge-ffi` was initializing logging before parsing config
  - Reordered initialization to parse config first, then apply log level
- Clarified in README that `rustbridge.toml` is a development-time config file, not the bundle manifest (`manifest.json`)
- Fixed JSON escaping examples in Kotlin tutorials (regex backslashes in JSON strings)

## [0.6.0] - 2025-01-28

### Added
- New `rustbridge` facade crate - single dependency for plugin development
  - Re-exports core types, macros, and FFI functions
  - Includes common dependencies: `async-trait`, `serde`, `serde_json`, `tokio`, `tracing`
  - Use `rustbridge::prelude::*` for convenient imports
  - Use `rustbridge::ffi_exports::*` for FFI function re-exports

### Changed
- All README documentation links now use absolute GitHub URLs (fixes broken links on crates.io)
- Plugin template (`rustbridge new`) now uses single `rustbridge` dependency instead of multiple crates
- Version bumped to 0.6.0

## [0.5.1] - 2025-01-27

### Added
- Published all crates to [crates.io](https://crates.io/crates/rustbridge-core)
- CI status badge in README

### Changed
- Plugin templates now use crates.io dependencies instead of git

## [0.5.0] - 2025-01-26

Initial public release.

### Added

#### Core Framework
- OSGI-inspired plugin lifecycle (Installed → Starting → Active → Stopping → Stopped → Failed)
- Async-first design built on Tokio runtime
- JSON-based message transport with typed envelopes
- Binary transport for performance-critical paths (7x faster than JSON)
- Concurrency limiting with configurable `max_concurrent_ops`
- Graceful shutdown with configurable timeout

#### Rust Crates
- `rustbridge-core`: Core traits (`Plugin`, `PluginFactory`), types, and lifecycle management
- `rustbridge-transport`: JSON and binary codec with message envelopes
- `rustbridge-ffi`: C ABI exports, buffer management, panic guards
- `rustbridge-runtime`: Tokio integration and async task management
- `rustbridge-logging`: Tracing integration with FFI callbacks to host languages
- `rustbridge-macros`: Procedural macros (`rustbridge_entry!`, `#[derive(Message)]`)
- `rustbridge-bundle`: `.rbp` bundle creation, loading, and signature verification
- `rustbridge-cli`: Command-line tool for bundle operations
- `rustbridge-jni`: JNI bridge for Java 17+ support

#### Bundle Format (.rbp)
- ZIP-based portable plugin distribution
- Multi-platform library support (Linux, macOS, Windows; x86_64, ARM)
- Multi-variant support (release, debug, custom variants)
- SHA256 checksums for integrity verification
- Optional minisign code signing
- Build metadata collection (git info, compiler version, timestamps)
- SBOM support (CycloneDX and SPDX formats)

#### Java/Kotlin Bindings
- FFM implementation for Java 21+ (recommended, faster)
- JNI implementation for Java 17+ (fallback)
- Kotlin extensions and type-safe DSL
- Bundle loader with automatic platform detection
- Signature verification support

#### C# Bindings
- P/Invoke-based native plugin loader (.NET 8.0+)
- Bundle loader with platform detection
- Minisign signature verification
- Binary transport support

#### Python Bindings
- ctypes-based native plugin loader (Python 3.10+)
- Bundle loader with platform detection
- Minisign signature verification
- Binary transport support

#### Tooling
- `rustbridge bundle create` command for building bundles
- `rustbridge bundle info` command for inspecting bundles
- Pre-commit validation scripts (Linux/macOS/Windows)
- Property-based testing with proptest

#### Documentation
- Architecture overview and design decisions
- Memory model and ownership patterns
- Plugin lifecycle state machine
- Testing conventions for all platforms
- Getting started tutorial with templates
- Language-specific usage guides

#### Templates
- Rust plugin template
- Java FFM consumer template
- Java JNI consumer template
- Kotlin consumer template
- C# consumer template
- Python consumer template

### Security
- Panic guards at FFI boundary (never panic across FFI)
- Lock safety enforcement (`await_holding_lock = "deny"`)
- No `.unwrap()` or `.expect()` in production code
- Minisign signature verification for bundle integrity

[Unreleased]: https://github.com/jrobhoward/rustbridge/compare/v0.6.1...HEAD
[0.6.1]: https://github.com/jrobhoward/rustbridge/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/jrobhoward/rustbridge/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/jrobhoward/rustbridge/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/jrobhoward/rustbridge/releases/tag/v0.5.0
