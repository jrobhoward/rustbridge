# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/jrobhoward/rustbridge/compare/v0.5.1...HEAD
[0.5.1]: https://github.com/jrobhoward/rustbridge/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/jrobhoward/rustbridge/releases/tag/v0.5.0
