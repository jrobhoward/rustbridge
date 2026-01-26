# RustBridge Benchmark Results

**Date:** 2026-01-25
**System:** Windows 11, AMD Ryzen Threadripper 1950X (16 cores), .NET 8.0.23, JDK 21.0.9, Python 3.13.9
**Plugin:** hello-plugin (release build)

## Summary

| Language | Transport | Latency | Throughput | Memory |
|----------|-----------|---------|------------|--------|
| **C#** | Binary | 326 ns | 3.2M ops/s | 40 B |
| **C#** | JSON | 2.55 μs | 393K ops/s | 688 B |
| **Java JNI** | Binary | 579 ns | 1.76M ops/s | - |
| **Java JNI** | JSON | 6.05 μs | 167K ops/s | - |
| **Java FFM** | Binary (zero-copy) | 667 ns | 1.50M ops/s | - |
| **Java FFM** | Binary (bytes) | 742 ns | 1.35M ops/s | - |
| **Java FFM** | Binary (segment) | 908 ns | 1.10M ops/s | - |
| **Java FFM** | JSON | 3.26 μs | 325K ops/s | - |
| **Python** | Binary | 5.73 μs | 175K ops/s | - |
| **Python** | JSON | 27-32 μs | 32-37K ops/s | - |

**Key Findings:**
- Binary transport is **5-10x faster** than JSON across all languages
- C# achieves the lowest latency (326 ns binary, 2.55 μs JSON)
- Java JNI binary (579 ns) outperforms Java FFM binary (667 ns zero-copy)
- Java FFM JSON (3.26 μs) outperforms Java JNI JSON (6.05 μs) by ~46%
- **Java FFM zero-copy** is the fastest FFM binary option (667 ns vs 742 ns bytes vs 908 ns segment)
- **Python binary improved 3.2x** after optimization (5.73 μs, was 18.2 μs)

---

## C# Benchmarks (.NET 8.0)

### Transport Latency

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON transport | 2,552.5 ns | 5.53 ns | 4.90 ns | 1.00 | 688 B |
| Binary transport | 326.0 ns | 0.50 ns | 0.44 ns | 0.13 | 40 B |

**Binary is 7.8x faster than JSON with 17x less memory allocation.**

### Throughput (1000 ops/invoke)

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON throughput | 2,544.0 ns | 7.56 ns | 7.07 ns | 1.00 | 696 B |
| Binary throughput | 314.6 ns | 0.20 ns | 0.18 ns | 0.12 | 40 B |

**Binary achieves 8.1x higher throughput.**

### Concurrent (100 parallel tasks)

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON concurrent | 122.63 μs | 0.749 μs | 0.664 μs | 1.00 | 83.91 KB |
| Binary concurrent | 70.94 μs | 0.507 μs | 0.450 μs | 0.58 | 33.90 KB |

**Binary concurrent calls complete in 58% of the time with 40% memory.**

---

## Java JNI Benchmarks (JDK 21)

### Transport Latency

| Method | Mean | Error | StdDev |
|--------|------|-------|--------|
| jniJson | 6.053 μs | 0.308 μs | 0.017 μs |
| jniBinary | 0.579 μs | 0.040 μs | 0.002 μs |

**Binary is 10.5x faster than JSON.**

### Throughput

| Method | Throughput (ops/s) | Error |
|--------|-------------------|-------|
| jniBinaryThroughput | 1,759,411 | ±202,894 |
| jniJsonThroughput | 166,694 | ±21,282 |

**Binary achieves 10.6x higher throughput.**

### Concurrent Scaling

| Threads | Binary (ops/s) | JSON (ops/s) | Binary/JSON Ratio |
|---------|----------------|--------------|-------------------|
| 1 | 1,572,661 | 168,506 | 9.3x |
| 4 | 2,420,161 | 592,009 | 4.1x |
| 8 | 1,534,061 | 1,119,235 | 1.4x |

---

## Java FFM Benchmarks (JDK 21)

### Transport Latency

| Method | Mean | Error | StdDev |
|--------|------|-------|--------|
| ffmJson | 3.264 μs | 0.125 μs | 0.007 μs |
| ffmBinary (MemorySegment) | 0.908 μs | 0.007 μs | 0.001 μs |
| ffmBinaryBytes (byte[]) | 0.742 μs | 0.085 μs | 0.005 μs |
| **ffmBinaryZeroCopy** | **0.667 μs** | 0.024 μs | 0.001 μs |

**Notes:**
- `ffmBinaryZeroCopy` is the **fastest** - returns direct native memory reference (no copy)
- `ffmBinaryBytes` returns a `byte[]` (copied to JVM heap)
- `ffmBinary` returns a `MemorySegment` (copied to Arena-managed memory)

### Throughput

| Method | Throughput (ops/s) | Error |
|--------|-------------------|-------|
| ffmBinaryThroughput | 1,349,179 | ±189,407 |
| ffmJsonThroughput | 325,231 | ±11,194 |

**Binary achieves 4.1x higher throughput.**

### Concurrent Scaling

| Threads | Binary (ops/s) | JSON (ops/s) | Binary/JSON Ratio |
|---------|----------------|--------------|-------------------|
| 1 | 1,340,305 | 326,371 | 4.1x |
| 4 | 1,817,613 | 835,024 | 2.2x |
| 8 | 2,289,383 | 1,398,446 | 1.6x |

---

## Java: JNI vs FFM Comparison

| Metric | JNI Binary | FFM Binary (zero-copy) | JNI JSON | FFM JSON |
|--------|------------|------------------------|----------|----------|
| Latency | 579 ns | 667 ns | 6.05 μs | 3.26 μs |
| Throughput | 1.76M ops/s | 1.35M ops/s | 167K ops/s | 325K ops/s |

**Analysis:**
- **Binary transport:** JNI is 13% faster than FFM zero-copy (579 ns vs 667 ns)
- **JSON transport:** FFM is 46% faster than JNI (3.26 μs vs 6.05 μs)
- JNI has lower overhead for raw byte operations
- FFM has better string handling, likely due to more efficient memory management

---

## Python Benchmarks (Python 3.13)

### Transport Latency

| Test | Mean | StdDev | Ops/s |
|------|------|--------|-------|
| Binary (small) | 5.73 μs | 0.77 μs | 174,545 |
| JSON (small math) | 26.70 μs | 7.58 μs | 37,450 |
| JSON (small greet) | 27.63 μs | 7.22 μs | 36,192 |
| JSON (small echo) | 31.59 μs | 35.96 μs | 31,655 |
| JSON (medium ~1KB) | 205.99 μs | 13.24 μs | 4,855 |
| JSON (large ~100KB) | 20,021 μs | 90.25 μs | 50 |

**Binary is 4.7-5.5x faster than JSON for small payloads.**

### Throughput & Lifecycle

| Test | Mean | Ops/s |
|------|------|-------|
| Sequential (100 calls) | 2,810 μs | 356 |
| Concurrent (100 tasks) | 12,391 μs | 81 |
| Load/unload cycle | 15,672 μs | 64 |

---

## Cross-Language Comparison

### Latency (Lower is Better)

```
Binary Transport:
C#              ████ 326 ns
Java JNI        ██████ 579 ns
Java FFM        ███████ 667 ns (zero-copy)
Python          ██████████████████████████████████████████████████ 5,729 ns

JSON Transport:
C#              █████████ 2,552 ns
Java FFM        ████████████ 3,264 ns
Java JNI        ██████████████████████ 6,053 ns
Python          ████████████████████████████████████████████████████████████████████████████████████████████████ 26,702 ns
```

### Why the Performance Differences?

1. **C# P/Invoke** - Direct native interop with minimal overhead, struct marshaling is highly optimized
2. **Java JNI** - Traditional native interface, excellent for binary data, higher overhead for strings
3. **Java FFM** - Modern foreign function API, better string handling, some overhead for memory management
4. **Python ctypes** - Interpreted language with dynamic typing adds significant overhead

### Binary vs JSON Speedup

| Language | Speedup Factor |
|----------|---------------|
| Java JNI | 10.5x |
| C# | 7.8x |
| Java FFM | 4.9x |
| Python | 4.7x |

---

## Recent Optimizations

### Python Binary Transport (v0.1.0)

**Before:** 18.2 μs per call
**After:** 5.73 μs per call
**Improvement:** 3.2x faster

**Change:** Eliminated double-copy in `call_raw()` by using `ctypes.addressof()` to pass the struct pointer directly instead of converting to bytes first.

### Java FFM Zero-Copy (v0.1.0)

**New method:** `callRawZeroCopy()` returns `RawResponse` wrapper with direct native memory access.

| Method | Latency | Improvement |
|--------|---------|-------------|
| ffmBinary (segment) | 908 ns | baseline |
| ffmBinaryBytes (byte[]) | 742 ns | 18% faster |
| **ffmBinaryZeroCopy** | **667 ns** | **27% faster** |

---

## Recommendations

### When to Use Binary Transport

- High-frequency calls (>10K ops/s needed)
- Latency-sensitive paths (<1μs required)
- Fixed-schema data with known layouts
- Memory-constrained environments

### When to Use JSON Transport

- Flexibility over raw performance
- Debugging/logging needs (human-readable)
- Variable-schema data
- Interoperability with external systems

### Language Selection

| Use Case | Recommended |
|----------|-------------|
| Maximum binary performance | Java JNI or C# |
| Maximum JSON performance | Java FFM or C# |
| Cross-platform desktop | C# (.NET) |
| Enterprise/Android | Java FFM (Java 21+) |
| Legacy Java (8-20) | Java JNI |
| Scripting/prototyping | Python |

### Java: JNI vs FFM Decision Guide

| Scenario | Recommendation |
|----------|---------------|
| Binary-heavy workload | JNI (13% faster) |
| JSON-heavy workload | FFM (46% faster) |
| Mixed workload | FFM (better overall balance) |
| Java 8-20 compatibility | JNI (only option) |
| Future-proofing | FFM (modern API, will improve) |

### Java FFM: Which Binary Method?

| Scenario | Recommended Method |
|----------|-------------------|
| Maximum performance | `callRawZeroCopy()` - 667 ns |
| Need byte[] for existing APIs | `callRawBytes()` - 742 ns |
| Need Arena lifetime management | `callRaw()` - 908 ns |

---

## Reproducing These Benchmarks

### Prerequisites

Build the Rust plugins in release mode:

```bash
# Build the hello-plugin (required for all benchmarks)
cargo build --release -p hello-plugin

# Build the JNI library (required for Java JNI benchmarks)
cargo build --release -p rustbridge-jni
```

### C# Benchmarks

```bash
cd rustbridge-csharp

# Transport latency benchmark
dotnet run -c Release --project RustBridge.Benchmarks -- --filter "*TransportBenchmark*"

# Throughput benchmark
dotnet run -c Release --project RustBridge.Benchmarks -- --filter "*ThroughputBenchmark*"

# Concurrent benchmark
dotnet run -c Release --project RustBridge.Benchmarks -- --filter "*ConcurrentBenchmark*"

# Run all benchmarks
dotnet run -c Release --project RustBridge.Benchmarks
```

### Java Benchmarks (FFM + JNI)

First, build the JMH benchmark jar:

```bash
cd rustbridge-java
./gradlew :rustbridge-benchmarks:jmhJar
```

Run benchmarks with the JNI library path. The key is passing `-jvmArgs` so forked JVM processes also get the library path:

```bash
cd rustbridge-java

# Full benchmarks (recommended settings from build.gradle.kts)
java --enable-preview --enable-native-access=ALL-UNNAMED \
  -jar rustbridge-benchmarks/build/libs/rustbridge-benchmarks-0.1.0-jmh.jar \
  -jvmArgs "--enable-preview --enable-native-access=ALL-UNNAMED -Djava.library.path=/path/to/rustbridge/target/release"

# Quick run with fewer iterations (for testing)
java --enable-preview --enable-native-access=ALL-UNNAMED \
  -jar rustbridge-benchmarks/build/libs/rustbridge-benchmarks-0.1.0-jmh.jar \
  -f 1 -wi 2 -i 3 \
  -jvmArgs "--enable-preview --enable-native-access=ALL-UNNAMED -Djava.library.path=/path/to/rustbridge/target/release"
```

**Important:** The `-jvmArgs` flag is critical for JNI benchmarks. JMH forks new JVM processes for each benchmark, and without `-jvmArgs`, those forked processes won't have the `java.library.path` set, causing JNI to fail silently (returning no-ops).

On Windows, use forward slashes in the path:
```bash
-Djava.library.path=C:/Users/yourname/git/rustbridge/target/release
```

### Python Benchmarks

```bash
cd rustbridge-python

# Install dependencies (first time only)
pip install -e ".[dev]"

# Run all benchmarks
python -m pytest tests/test_benchmarks.py tests/test_binary_transport.py -v --benchmark-only

# Run with specific columns
python -m pytest tests/test_benchmarks.py tests/test_binary_transport.py -v \
  --benchmark-only --benchmark-columns=mean,stddev,ops
```

---

## Test Environment

- **OS:** Windows 11 (10.0.26100.7623)
- **CPU:** AMD Ryzen Threadripper 1950X (16 logical cores)
- **Runtime Versions:**
  - .NET SDK 8.0.417, Runtime 8.0.23
  - JDK 21.0.9 (Eclipse Adoptium)
  - Python 3.13.9
- **Benchmark Tools:**
  - BenchmarkDotNet v0.14.0
  - JMH 1.37
  - pytest-benchmark 5.2.3
- **Rust Plugin:** hello-plugin built with `--release`
