# rustbridge Tasks & Roadmap

This document tracks incomplete tasks for the rustbridge project.

> **Note**: This file will eventually be replaced with GitHub Issues for better tracking.

---

## High Priority

| Task | Area | Effort | Notes |
|------|------|--------|-------|
| C# CI/CD pipeline | C# | 1 day | Add .NET build/test to GitHub Actions (ubuntu/windows/macos) |
| C# NuGet packaging | C# | 1-2 days | Publish .nupkg to NuGet.org (metadata already in .csproj) |

---

## Medium Priority

| Task | Area | Effort | Notes |
|------|------|--------|-------|
| Automatic handler dispatch macro | Rust | 3-5 days | Enhance `#[rustbridge_handler]` to auto-generate Plugin trait implementation |
| PluginConfig builder pattern | Rust | 1-2 days | Add fluent `PluginConfig::builder()` API for ergonomic testing |
| Gradle code generation integration | Java | 3-5 days | Create rustbridge-gradle-plugin with auto-generation task |
| Error message quality improvements | All | 1-2 days | Improve actionable error messages across all languages |

---

## Low Priority

| Task | Area | Effort | Notes |
|------|------|--------|-------|
| Memory consumption tracking | Rust | 3-5 days | Implement TrackingAllocator, expose via FFI (optional feature) |
| CPU/task metrics from Tokio | Rust | 1-2 days | Expose RuntimeMonitor stats via FFI (optional tokio-metrics) |
| Memory-based backpressure | Rust | 1-2 days | Reject requests when heap usage exceeds threshold |
| Clean up unused code warnings | Rust | 1 day | Dead code in runtime, logging crates |
| Async API (plugin_call_async) | Rust | 3-5 days | Not critical for current use cases |

---

## Deferred

These tasks are explicitly deferred pending user requirements:

| Task | Reason |
|------|--------|
| Java JMH benchmark harness | Rust benchmarks sufficient |
| Memory profiling setup | Not needed for current decision |
| Latency distribution analysis | Mean values sufficient |
| RbArray, RbOptional types | Not needed yet for binary transport |
| CStructCodec implementation | Direct handler approach used instead |

---

## Recently Completed

| Task | Area | Date | Notes |
|------|------|------|-------|
| Include JNI bridge in .rbp bundle | JNI | 2026-01-30 | `--jni-lib` CLI flag, `BundleLoader.extractJniBridge()`, `JniPluginLoader.loadFromBundle()` |
