# Plan: Remove JNI Transport from rustbridge

## Summary

Remove the JNI transport layer entirely, keeping only FFM (Foreign Function & Memory API) for Java integration. This simplifies maintenance by eliminating a parallel implementation that has proven more problematic (e.g., log callbacks not working).

**Impact**: Java minimum version changes from 17 to 22+.

---

## Scope Analysis

### Files to Delete (Complete Removal)

**Rust crate:**
- `crates/rustbridge-jni/` (entire directory - 4 files, ~930 lines)
  - `src/lib.rs`, `src/loader.rs`, `src/ffi_types.rs`, `src/error.rs`, `Cargo.toml`

**Java module:**
- `rustbridge-java/rustbridge-jni/` (entire directory)
  - `src/main/java/com/rustbridge/jni/JniPluginLoader.java`
  - `src/main/java/com/rustbridge/jni/JniPlugin.java`
  - `src/test/java/com/rustbridge/jni/` (12 test files)
  - `build.gradle.kts`

**CLI templates:**
- `crates/rustbridge-cli/templates/java-jni/` (entire directory - 10 files)

**Documentation:**
- `docs/using-plugins/JAVA_JNI.md`
- `docs/tutorials/07-backpressure-queues/02-java-jni-consumer.md`
- `docs/tutorials/08-binary-transport/02-java-jni-consumer.md`
- `docs/tutorials_windows/07-backpressure-queues/02-java-jni-consumer.md`

---

### Files to Modify

#### 1. Workspace Configuration

**`Cargo.toml`** (line 13):
- Remove `"crates/rustbridge-jni"` from workspace members

**`rustbridge-java/settings.gradle.kts`** (line 5):
- Remove `include("rustbridge-jni")`

#### 2. CLI Code

**`crates/rustbridge-cli/src/main.rs`**:
- Remove `--java-jni` flag (line 47)
- Remove `--jni-lib` flag and related parsing (lines 115-122, 264-272, 293, 334-357, 405)
- Remove `java_jni` from `ConsumerOptions` struct

**`crates/rustbridge-cli/src/new.rs`**:
- Remove `JAVA_JNI_*` template constants (lines 50-62)
- Remove `java_jni` field from `ConsumerOptions` (line 120)
- Remove `create_java_jni_consumer` function (lines 303-350)
- Remove JNI from help output (lines 438-440)

**`crates/rustbridge-cli/src/bundle.rs`**:
- Remove `jni_libraries` parameter from `create_bundle` function (line 18)
- Remove JNI library addition loop (lines 58-66)
- Update test calls to remove JNI parameter (lines 875, 921, 962, 1009)

#### 3. Bundle Crate

**`crates/rustbridge-bundle/src/manifest.rs`**:
- Remove `jni` field from `BridgeInfo` struct (line 297)
- Remove `add_jni_bridge` method (lines 462-491)
- Remove `has_jni_bridge` method (lines 493-496)
- Remove `get_jni_bridge` method (lines 499-504)
- Remove JNI-related tests (lines 1214-1289)

**`crates/rustbridge-bundle/src/builder.rs`**:
- Remove `add_jni_library` method (lines 186-198)
- Remove `add_jni_library_variant` method (lines 201-240)
- Remove JNI-related tests (lines 959-1055)

#### 4. Java Benchmarks

**`rustbridge-java/rustbridge-benchmarks/build.gradle.kts`** (line 10):
- Remove `implementation(project(":rustbridge-jni"))`
- Update benchmark classes to remove JNI comparisons (separate files in `src/jmh/java/`)

#### 5. Documentation Updates

**`CLAUDE.md`**:
- Update Java version requirement from "17+ (JNI)" to "22+"
- Remove JNI references from FFI layer description (line 8)
- Remove `rustbridge-jni` from Tooling layer

**`README.md`**:
- Remove Java JNI row from version table (line 169)
- Remove JNI dependency example (line 202)
- Remove "Java JNI Bindings" from stability table (line 232)
- Update tutorial link text (line 104)

**`docs/using-plugins/README.md`**:
- Remove Java JNI row from table (line 11)

**`docs/tutorials/README.md`**:
- Remove JNI tutorial references (lines 30, 114, 119, 129)

**`docs/tutorials/04-java-consumer/README.md`**:
- Remove JNI fallback reference (lines 63-64)

**`docs/tutorials/06-cross-compilation/README.md`**:
- Remove JNI appendix reference (lines 69-71)

**`docs/tutorials/07-backpressure-queues/README.md`**:
- Remove `java-jni/` from directory structure (line 86)
- Remove JNI consumer section (lines 222, 240, 255)

**`docs/tutorials/08-binary-transport/README.md`**:
- Remove `java-jni/` from directory structure (line 113)
- Remove JNI copy commands (lines 541, 570)
- Remove JNI consumer section (lines 583-585, 607)

**Windows tutorial counterparts** (similar changes):
- `docs/tutorials_windows/README.md`
- `docs/tutorials_windows/04-java-consumer/README.md`
- `docs/tutorials_windows/07-backpressure-queues/README.md`
- `docs/tutorials_windows/08-binary-transport/README.md`

**Other docs with JNI mentions** (review and update):
- `docs/ARCHITECTURE.md` - Remove JniPluginLoader/JniPlugin from diagrams
- `docs/GETTING_STARTED.md` - Remove JNI references
- `docs/TESTING_JAVA.md` - Update to FFM-only
- `docs/BUNDLE_FORMAT.md` - Remove JNI bridge references
- `docs/MEMORY_MODEL.md` - Remove JNI references
- `docs/BENCHMARK_RESULTS.md` - Remove JNI benchmark data
- `docs/PRE_1.0_REFINEMENTS.md` - Update roadmap
- `docs/TASKS.md` - Remove JNI tasks

---

## Execution Order

### Phase 1: Rust Crate Removal
1. Remove `crates/rustbridge-jni` from `Cargo.toml` workspace members
2. Delete `crates/rustbridge-jni/` directory
3. Update `rustbridge-bundle` crate (remove JNI methods and tests)
4. Update `rustbridge-cli` crate (remove JNI flags, templates, code)
5. Delete `crates/rustbridge-cli/templates/java-jni/` directory
6. Run `cargo build --workspace` to verify

### Phase 2: Java Module Removal
1. Remove `rustbridge-jni` from `settings.gradle.kts`
2. Delete `rustbridge-java/rustbridge-jni/` directory
3. Update `rustbridge-benchmarks/build.gradle.kts` and benchmark code
4. Run `./gradlew build` from `rustbridge-java/` to verify

### Phase 3: Documentation
1. Delete JNI-specific documentation files
2. Update documentation with JNI references
3. Update `CLAUDE.md` and `README.md`

### Phase 4: Verification
1. Run `./scripts/pre-commit.sh` for full validation
2. Verify Java tests pass with `./gradlew test`
3. Search for any remaining "jni" references: `grep -ri "jni" --include="*.rs" --include="*.java" --include="*.md" --include="*.kts"`

---

## Verification Checklist

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy --workspace --examples --tests -- -D warnings` passes
- [ ] `./gradlew build` (from rustbridge-java/) succeeds
- [ ] `./gradlew test` (from rustbridge-java/) passes
- [ ] No "jni" references remain in codebase (except historical in CHANGELOG.md)
- [ ] `./scripts/pre-commit.sh` passes

---

## Notes

- **CHANGELOG.md**: Keep historical JNI entries but add a new entry documenting the removal
- **Git**: All changes should be in a single commit or logical commit sequence
- **Breaking Change**: This is a breaking change for Java 17-21 users; document in release notes
