# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Tutorial: Chapter 8 - Binary Transport with image thumbnail generator plugin
  - Java FFM, Java JNI, Kotlin, C#, and Python consumers demonstrating binary FFI
  - Variable-length binary response handling (header + payload pattern)
- Tutorial: Chapter 7 - Backpressure Queues for bounded queue flow control
  - C#, Java/JNI, and Python implementations demonstrating blocking producers when queues are full

### Changed
- Updated all version references from 0.5.0/0.6.0 to 0.7.0 across documentation and templates

## [0.7.0] - 2026-01-30

### Changed
- Java: Replaced `System.err.println` with slf4j logging in FFM plugin loader
- Java: Consolidated `ObjectMapper` instances to use shared `JsonMapper.getInstance()`
- Java: Extracted platform detection logic to `PlatformUtil` utility class
- C#: Optimized `MinisignVerifier.Verify(ReadOnlySpan<byte>)` to avoid array allocation
- Rust: Optimized `RequestEnvelope.payload_as()` and `ResponseEnvelope.payload_as()` to deserialize without cloning JSON
- Java FFM: Simplified binary transport to single `callRawBytes(int, BinaryStruct)` method
  - Removed complex `callRaw` variants and `callRawZeroCopy`
  - `callRawBytes` returns `byte[]` for simplicity while remaining high-performance
- Java FFM `BinaryStruct`: Now uses unaligned memory access to support heap-backed segments
  - Enables wrapping `byte[]` arrays returned from `callRawBytes` with `MemorySegment.ofArray()`

### Removed
- `rustbridge build` CLI command (redundant wrapper around `cargo build`)
- Bundle manifest API fields (reserved for future use):
  - `api.messages` - message schema definitions
  - `api.min_rustbridge_version` - version constraint
  - `api.transports` - transport types
- Java FFM: Removed `callRaw`, `callRawZeroCopy`, and `RawResponse` inner class

### Added
- Test coverage improved for FFI, bundle, and edge cases
  - Rust FFI integration tests (23 tests): plugin lifecycle, concurrent calls, unicode handling
  - CLI bundle integration tests (43 tests): platform parsing, manifest validation, transport codec
  - C# edge case tests (10 tests): dispose handling, concurrent access, missing DLL handling
  - Python bundle loader tests (7 new tests): corrupted/truncated bundles, invalid manifests
- JNI bridge bundling support for self-contained Java 17+ distribution
  - New `bridges` field in bundle manifest for including bridge libraries
  - CLI: `--jni-lib PLATFORM[:VARIANT]:PATH` flag for `rustbridge bundle create`
  - Rust: `BundleBuilder::add_jni_library()` and `add_jni_library_variant()` methods
  - Java: `BundleLoader.hasJniBridge()` and `extractJniBridge()` methods
  - Java: `JniPluginLoader.loadFromBundle()` for automatic bridge loading
  - Python/C#: Manifest parsing and extraction methods for API parity
- `hasBinaryTransport()` method to Java FFM `FfmPlugin` and C# `IPlugin`/`NativePlugin`
  - Checks if binary transport symbols are available in the loaded library
  - Java FFM and C# now handle optional binary transport symbols gracefully

### Fixed
- C#: `BundleLoader.Dispose()` now uses try-finally to ensure both streams are disposed
- C#: `MinisignVerifier` constructor now validates null/empty public key with proper exceptions
- Java: JNI static initializer no longer throws harsh `RuntimeException` on missing library
- Rust: Documented async API placeholders (`plugin_call_async`, `plugin_cancel_async`) with status and planned behavior
- Minisign signature verification in Java, C#, and Python consumers
  - Fixed BLAKE2b-512 prehashing for "ED" algorithm signatures
  - Fixed public key format parsing (algorithm ID "Ed" + key ID + public key)
  - Fixed ambiguous `HashAlgorithm` reference in C# verifier
- Bundle CLI now correctly extracts public key from .pub file (line 2 only)
- Tutorial code examples: added missing imports, `throws` declarations, and `#[allow(non_snake_case)]`
- Tutorial documentation fixes for Chapters 5-6:
  - Fixed non-existent CLI commands (`bundle info` → `bundle list --show-build`)
  - Fixed non-existent CLI flags (`--schema-only`, `--sbom-only`)
  - Fixed `cargo sbom` format argument (`cdx` → `cyclone_dx_json_1_6`)
  - Removed `cargo-spdx` references (use `cargo sbom --output-format spdx_json_2_3`)
  - Removed musl target references (musl doesn't support cdylib/shared libraries)
  - Fixed Java FFM commands: added `--enable-preview` and `--enable-native-access=ALL-UNNAMED`
  - Fixed Shadow JAR filename in cross-compilation tutorial

### Added
- Oracle-based minisign verification tests across all languages
  - Rust tests verify signature generation and format
  - Java, C#, Python tests verify against reference vectors from Rust minisign crate

## [0.6.2] - 2025-01-29

### Added
- `rustbridge new` now supports multi-language consumer generation with flags:
  - `--kotlin`, `--java-ffm`, `--java-jni`, `--csharp`, `--python`, `--all`
  - Generated consumers placed in `consumers/` subdirectory
- Rust plugin template with placeholder substitution (embedded in CLI)
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
  - Templates moved from `templates/` to `crates/rustbridge-cli/templates/` for crates.io compatibility
- Tutorials and documentation updated to use `rustbridge new` instead of `cargo generate`
- Generated projects use Option B structure: Rust plugin at root, consumers in `consumers/`

### Removed
- `templates/` (workspace root) - Templates moved to `crates/rustbridge-cli/templates/`
- `templates/plugin/` - Replaced by embedded `rust` template
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

[Unreleased]: https://github.com/jrobhoward/rustbridge/compare/v0.7.0...HEAD
[0.7.0]: https://github.com/jrobhoward/rustbridge/compare/v0.6.2...v0.7.0
[0.6.2]: https://github.com/jrobhoward/rustbridge/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/jrobhoward/rustbridge/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/jrobhoward/rustbridge/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/jrobhoward/rustbridge/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/jrobhoward/rustbridge/releases/tag/v0.5.0
