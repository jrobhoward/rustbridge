# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- **Docs**: Update all documentation for published package registries (crates.io, Maven Central, PyPI, NuGet)
- **Docs**: Add `docs/DEVELOPMENT.md` build-from-source guide for contributors
- **Docs**: Rewrite `docs/INSTALL.md` to reference published packages as primary installation path
- **Templates**: C# template now uses NuGet `PackageReference` by default instead of `ProjectReference`
- **Templates**: Python template now uses `pip install rustbridge` from PyPI

## [1.0.0] - 2026-03-07

### Changed
- **Java/Kotlin**: Rename Maven groupId and Java packages from `com.rustbridge` to `io.github.jrobhoward.rustbridge`
- **Java/Kotlin**: Published to Maven Central — consumers now use `mavenCentral()` instead of `mavenLocal()`
- **All**: First stable release — version 1.0.0 across all ecosystems (Rust, Java, C#, Python, Erlang)
- **Docs**: Update all tutorials and guides to reference Maven Central and version 1.0.0
- **Dependencies**: Upgrade workspace dependencies (zip 8.2, tokio 1.50, libloading 0.9, minisign 0.9, others)

## [0.10.0] - 2026-02-14

### Added
- **Bundle**: v1.1 format with variant-level `build_info`, `sbom`, `schema_checksum`, and `schemas` for per-platform build provenance in combined bundles
- **Bundle**: `combine` CLI now propagates source bundle build metadata to each variant; `slim` preserves variant-level metadata
- **Bundle**: `list --build --variants` displays variant-level build info
- **Bundle**: Resolution methods (`get_effective_build_info`, etc.) in Java, C#, and Python loaders
- **Erlang**: Erlang/OTP consumer via Port-based architecture (`rustbridge-erlang/` OTP application + `rustbridge-port-driver` Rust binary)
- **Erlang**: JSON and binary transport, log forwarding to OTP `logger`, bundle loading with signature verification
- **Erlang**: EUnit and Common Test suites with full integration tests against hello-plugin
- **Erlang**: Rejected request count API (`get_rejected_count/1`) for concurrency limit monitoring
- **Erlang**: Benchmark suite (`rustbridge_bench_SUITE`) measuring JSON, binary, and concurrent call latency
- **Go**: Go consumer via CGo + dlopen for direct in-process FFI (`rustbridge-go/` Go module)
- **Go**: JSON transport via `Call()` and `CallTyped()`, binary transport via `CallRaw()`
- **Go**: Log callback integration with `log/slog` adapter (`SlogLogHandler`)
- **Go**: Functional options config pattern (`WithLogLevel`, `WithWorkerThreads`, etc.)
- **Go**: Bundle loading (`.rbp`) with SHA256 checksum verification and minisign signature verification
- **Go**: Integration tests and benchmarks against hello-plugin
- **Docs**: Erlang testing conventions (`docs/TESTING_ERLANG.md`)
- **Docs**: Go testing conventions (`docs/TESTING_GO.md`)
- **Docs**: Erlang and Go benchmark results added to `docs/BENCHMARK_RESULTS.md`
- **Docs**: Feature support matrix added to `docs/ARCHITECTURE.md` covering all 6 host languages
- **Tests**: Rust consumer integration tests for binary transport (`call_raw`), rejected request count stress, `load_by_name`, and bundle verification
- **Tests**: Go rejected request count stress test (`WithMaxConcurrentOps(1)`)
- **Tests**: Erlang bundle loading integration test (`load_bundle___hello_plugin___is_active`)

### Fixed
- **Go**: Error code constant names now match Rust `PluginError::error_code()` mapping (previously misnamed)
- **Go**: `BundleLoader.Load()` no longer leaks temp directory on success
- **Go**: TOCTOU race in `Call()`/`CallRaw()` — read lock now held across FFI calls
- **Go**: `HasBinaryTransport()` now holds mutex; `IsPluginError()` uses `errors.As` for wrapped errors
- **Go**: Added missing v1.1 manifest fields (`BuildInfo`, `Sbom`, `Schemas`, etc.) and `GetEffective*` methods
- **Erlang**: `decode_message/1` no longer crashes on malformed JSON — returns `{error, ...}` tuple
- **Erlang**: `get_state/1` uses `binary_to_atom/2` instead of `binary_to_existing_atom/2` to avoid `badarg`
- **Erlang**: `terminate/2` sends best-effort shutdown command before closing port
- **Erlang**: Added `handle_call` catch-all to prevent gen_server crash on unknown requests
- **Rust CLI**: `bundle combine`/`slim` now propagate errors from variant metadata setters instead of silently discarding with `.ok()`
- **Rust CLI**: Replace all production `unwrap()` calls in bundle operations with proper error handling
- **Java/C#/Python**: Added missing `BuildInfo.custom` field and `getEffectiveSchemaChecksum` method
- **Java/C#/Python**: Fixed stale `(v2.0+)` comments to `(v1.0+)`
- **Docs**: Updated Go error code tables in `ERROR_HANDLING.md` and `GO.md` to match corrected constants
- **Docs**: Added Go/Erlang sections to error handling ToC, `get_rejected_count` to Erlang monitoring docs
- **Docs**: Updated `BUNDLE_FORMAT.md` to reflect v1.1 support; added Go/Erlang pointer to `GETTING_STARTED.md`

### Changed
- **Pre-commit**: Added Erlang and Go change detection and test steps to `pre-commit.sh`

## [0.9.1] - 2026-02-06

### Added
- **Docs**: Added v0.9 release notes (`docs/RELEASE_NOTES_0.9.md`) for This Week in Rust announcement

### Fixed
- **Rust Consumer**: `NativePlugin::state()` now returns `Stopped` after shutdown instead of `Failed` (handle was removed from FFI manager, causing state query to return unknown)

### Changed
- **CI**: Added .NET 8.0, Python 3.12, and consumer integration tests to the `validate` job
- **CI**: Added clippy and consumer integration tests to the cross-platform `test-matrix` job
- **Pre-commit**: Added Python test section and consumer integration tests to `pre-commit.sh` and `pre-commit.bat`

## [0.9.0] - 2026-02-06

### Changed
- **CLI**: `rustbridge new` now prints `rustbridge pack` as the bundling step instead of `rustbridge bundle create`
- **Docs**: Tutorials and guides updated to recommend `rustbridge pack` for standard single-platform builds
- **Docs**: Added CLI reference (`docs/CLI.md`) covering all commands: `new`, `pack`, `promote`, `bundle`, `keygen`, `generate-header`
- **Docs**: Added C# and Python sections to `docs/ERROR_HANDLING.md` (PluginException, error codes, testing patterns)
- **Docs**: Added pack/promote workflow guide and command decision table to `docs/packaging/README.md`
- **Docs**: Added 1.0 release notes (`docs/RELEASE_NOTES_1.0.md`) with feature overview

### Added
- **CLI**: `rustbridge pack` now warns when libraries appear older than source files, helping catch forgotten rebuilds
- **CLI**: Colored warnings (yellow/bold) for all CLI commands via `yansi` — signing key missing, staleness, schema mismatch
- **CLI**: `rustbridge pack` command to auto-detect plugin project and create bundles from `Cargo.toml` metadata
- **CLI**: `rustbridge promote` command to slim a dev bundle to a signed release bundle
- **Rust Consumer**: `load_bundle_variant_with_config()` for loading specific variants (debug/release) from bundles
- **Python**: `BundleLoader.load_variant_with_config()` for loading specific variants from bundles
- **Example Plugin**: `variant-log-plugin` example that identifies its build variant for testing
- **Integration Tests**: Bundle variant loading tests (serial, parallel, unload/reload, log callback verification)
- **Build Script**: `scripts/build-variant-test-bundles.sh` for building 4 variant configurations into 2 bundles

### Fixed
- **CLI**: `rustbridge new` now prints correct bundle copy path (`target/bundle/`) in consumer next-steps output
- **Docs**: Fixed bundle copy paths in Getting Started guide, Kotlin consumer tutorial (Ch2), and Java consumer tutorial (Ch4)
- **Docs**: Fixed unused `PluginConfig` import in Getting Started Rust consumer example
- **Docs**: Replaced non-existent docs.rs link for `rustbridge-consumer` with local Rust language guide link
- **Docs**: Removed references to non-existent example implementations in tutorials README
- **Rust Consumer**: Fixed SIGBUS crash when multiple threads load plugins from bundles concurrently (extraction paths now unique per load)

### Added
- **Rust Consumer**: New `rustbridge-consumer` crate for loading plugins from Rust applications
  - Dynamic loading of shared libraries (.so, .dylib, .dll) and .rbp bundles
  - JSON transport via `call()` and `call_typed()` methods
  - Binary transport via `call_raw()` for high-performance paths
  - Full lifecycle management (Installed, Starting, Active, Stopping, Stopped, Failed)
  - Log callback integration for routing plugin logs to host
  - Bundle signature verification via minisign Ed25519
  - Automatic plugin cleanup on drop
- **Tutorial**: Rust consumer sections for Chapters 7 (Backpressure Queues) and 8 (Binary Transport)
- **Bundle Loader**: Added `verify_manifest_signature()` and `extract_library_verified()` methods to `BundleLoader`

## [0.8.1] - 2026-02-04

### Security
- Updated `bytes` crate to 1.11.1 to fix integer overflow vulnerability (RUSTSEC-2026-0007)

### Removed
- Removed unimplemented async FFI stubs (`plugin_call_async`, `plugin_cancel_async`) from C API

### Changed
- Documentation: Fixed Jackson dependency version inconsistencies (standardized to 2.18.2)
- Documentation: Added ERROR_HANDLING.md cross-references to language guides
- Documentation: Fixed formatting issues in KOTLIN.md

## [0.8.0] - 2026-02-01

### Removed
- **BREAKING: Removed JNI transport layer** - Java minimum version now 21+ (FFM only)
  - Removed `rustbridge-jni` Rust crate and all JNI-related code
  - Removed `rustbridge-jni` Java module and JNI implementation
  - Removed `install-jni-bridge` CLI command and `--jni-lib` bundle flags
  - Removed JNI templates from CLI (`java-jni/` template directory)
  - Updated all documentation to reflect FFM-only Java integration
  - Java users must now use Java 21+ with FFM (Foreign Function & Memory API)
  - Note: Java 21 requires `--enable-preview` in addition to `--enable-native-access=ALL-UNNAMED`

### Changed
- Java/Kotlin: Updated to Kotlin 2.3.0 and Gradle 9.3.1 for native Java 25 support
  - Templates: kotlin, java-ffm now use Gradle 9.3.1
  - rustbridge-kotlin uses Kotlin 2.3.0 (was 2.0.21)
  - Kotlin template uses Kotlin 2.3.0 (was 2.0.0)
- Java: Added automatic Java 21 preview feature detection in Gradle builds
  - Build files now conditionally add `--enable-preview` only when Java 21 is detected
  - Same build files work seamlessly on Java 21 and Java 22+ without modification
  - Updated FFM API calls to use Java 21-compatible methods (`allocateUtf8String`, `getUtf8String`)
- Docs: Updated all documentation to reflect Java 21+ support
  - FFM now works with Java 21+
  - Java 21 requires `--enable-preview` flag (automatically handled by Gradle)
  - Removed all JNI references from documentation
- Updated all version references from 0.7.0 to 0.8.0 across all languages and documentation

### Added
- Tutorial: Chapter 8 - Binary Transport with image thumbnail generator plugin
  - Java FFM, Kotlin, C#, and Python consumers demonstrating binary FFI
  - Variable-length binary response handling (header + payload pattern)
- Tutorial: Chapter 7 - Backpressure Queues for bounded queue flow control
  - C# and Python implementations demonstrating blocking producers when queues are full
- Docs: C# troubleshooting section for MSBuild metadata file errors
- Build: Added MPL-2.0 to allowed licenses in deny.toml (used by dirs crate)

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
- JNI implementation for Java 17+ (recommended, better binary performance)
- FFM implementation for Java 22+ (experimental)
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

[Unreleased]: https://github.com/jrobhoward/rustbridge/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/jrobhoward/rustbridge/compare/v0.10.0...v1.0.0
[0.10.0]: https://github.com/jrobhoward/rustbridge/compare/v0.9.1...v0.10.0
[0.9.1]: https://github.com/jrobhoward/rustbridge/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/jrobhoward/rustbridge/compare/v0.8.1...v0.9.0
[0.8.1]: https://github.com/jrobhoward/rustbridge/compare/v0.8.0...v0.8.1
[0.8.0]: https://github.com/jrobhoward/rustbridge/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/jrobhoward/rustbridge/compare/v0.6.2...v0.7.0
[0.6.2]: https://github.com/jrobhoward/rustbridge/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/jrobhoward/rustbridge/compare/v0.6.0...v0.6.1
[0.6.0]: https://github.com/jrobhoward/rustbridge/compare/v0.5.1...v0.6.0
[0.5.1]: https://github.com/jrobhoward/rustbridge/compare/v0.5.0...v0.5.1
[0.5.0]: https://github.com/jrobhoward/rustbridge/releases/tag/v0.5.0
