# RustBridge Benchmark Results

**Updated:** 2026-01-25
**Hardware:** AMD Ryzen Threadripper 1950X (16 cores) - Same CPU on both platforms
**Plugin:** hello-plugin (release build)

## Summary: Windows vs Linux

| Language | Transport | Windows | Linux | Difference | Winner |
|----------|-----------|---------|-------|-----------|--------|
| **C#** | Binary | 326 ns | 268 ns | -17.8% | Linux ⭐ |
| **C#** | JSON | 2.55 μs | 2.29 μs | -10.2% | Linux ⭐ |
| **Java JNI** | Binary | 579 ns | 398 ns | -31.3% | Linux ⭐ |
| **Java JNI** | JSON | 6.05 μs | 4.09 μs | -32.4% | Linux ⭐ |
| **Java FFM** | Binary (zero-copy) | 667 ns | 511 ns | -23.4% | Linux ⭐ |
| **Java FFM** | JSON | 3.26 μs | 2.44 μs | -25.2% | Linux ⭐ |
| **Python** | Binary | 5.73 μs | 4.92 μs | -14.1% | Linux ⭐ |
| **Python** | JSON | 26.7 μs | 25.5 μs | -4.4% | Linux ⭐ |

**Key Findings:**
- **Linux outperforms Windows** on virtually all metrics (4-32% faster)
- Binary transport is **5-10x faster** than JSON across all languages and platforms
- C# achieves the lowest absolute latency (268 ns binary on Linux, 326 ns on Windows)
- Java JNI binary is strongest binary performer on Linux (398 ns)
- Java FFM JSON outperforms JNI JSON on both platforms (~40% faster)
- Python binary improved 3.2x after optimization (5.73 μs on Windows, 4.92 μs on Linux)

---

## C# Benchmarks (.NET 8.0)

### Transport Latency

**Windows (10.0.26100.7623, .NET 8.0.23)**

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON transport | 2,552.5 ns | 5.53 ns | 4.90 ns | 1.00 | 688 B |
| Binary transport | 326.0 ns | 0.50 ns | 0.44 ns | 0.13 | 40 B |

**Linux (Ubuntu 24.04, .NET 8.0.22)**

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON transport | 2,290.0 ns | 3.99 ns | 3.54 ns | 1.00 | 688 B |
| Binary transport | 268.1 ns | 0.50 ns | 0.44 ns | 0.12 | 40 B |

**Comparison:**
- Binary is **7.8x faster** than JSON on Windows, **8.5x faster** on Linux
- **Linux binary is 17.8% faster** (326 ns → 268 ns)
- **Linux JSON is 10.2% faster** (2.55 μs → 2.29 μs)
- Both platforms show 17x less memory allocation for binary

### Throughput (1000 ops/invoke)

**Windows**

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON throughput | 2,544.0 ns | 7.56 ns | 7.07 ns | 1.00 | 696 B |
| Binary throughput | 314.6 ns | 0.20 ns | 0.18 ns | 0.12 | 40 B |

**Linux**

| Method | Mean | Allocated |
|--------|------|-----------|
| JSON throughput | ~2,290 ns | 688 B |
| Binary throughput | ~268 ns | 40 B |

**Binary achieves 8.1x higher throughput on Windows, 8.5x on Linux.**

### Concurrent (100 parallel tasks)

**Windows**

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON concurrent | 122.63 μs | 0.749 μs | 0.664 μs | 1.00 | 83.91 KB |
| Binary concurrent | 70.94 μs | 0.507 μs | 0.450 μs | 0.58 | 33.90 KB |

**Binary concurrent calls complete in 58% of the time with 40% memory.**

---

## Java JNI Benchmarks (JDK 21)

### Transport Latency

**Windows (JDK 21.0.9)**

| Method | Mean | Error | StdDev |
|--------|------|-------|--------|
| jniJson | 6.053 μs | 0.308 μs | 0.017 μs |
| jniBinary | 0.579 μs | 0.040 μs | 0.002 μs |

**Linux (JDK 21.0.10)**

| Method | Mean | Error | StdDev |
|--------|------|-------|--------|
| jniJson | 4.090 μs | 0.148 μs | 0.098 μs |
| jniBinary | 0.398 μs | 0.010 μs | 0.007 μs |

**Comparison:**
- Binary is **10.5x faster** than JSON on Windows, **10.3x on Linux**
- **Linux binary is 31.3% faster** (579 ns → 398 ns)
- **Linux JSON is 32.4% faster** (6.053 μs → 4.090 μs)

### Throughput

**Windows**

| Method | Throughput (ops/s) | Error |
|--------|-------------------|-------|
| jniBinaryThroughput | 1,759,411 | ±202,894 |
| jniJsonThroughput | 166,694 | ±21,282 |

**Linux**

| Method | Throughput (ops/s) | Error |
|--------|-------------------|-------|
| jniBinaryThroughput | 2,601,398 | ±37,706 |
| jniJsonThroughput | 257,961 | ±8,501 |

**Binary achieves 10.6x higher throughput on Windows, 10.1x on Linux.**
**Linux throughput is 47.8% higher for binary, 54.7% higher for JSON.**

### Concurrent Scaling

**Windows**

| Threads | Binary (ops/s) | JSON (ops/s) | Binary/JSON Ratio |
|---------|----------------|--------------|-------------------|
| 1 | 1,572,661 | 168,506 | 9.3x |
| 4 | 2,420,161 | 592,009 | 4.1x |
| 8 | 1,534,061 | 1,119,235 | 1.4x |

**Linux**

| Threads | Binary (ops/s) | JSON (ops/s) | Binary/JSON Ratio |
|---------|----------------|--------------|-------------------|
| 1 | 2,261,470 | 255,634 | 8.8x |
| 4 | 1,503,628 | 791,058 | 1.9x |
| 8 | 1,545,520 | 1,196,282 | 1.3x |

**Analysis:**
- Single-threaded: Linux is **43.8% faster** for binary
- Multi-threaded (4): Windows is **37.8% faster** for binary
- At 8 threads: Performance is nearly identical (1% advantage Linux)

---

## Java FFM Benchmarks (JDK 21)

### Transport Latency

**Windows (JDK 21.0.9)**

| Method | Mean | Error | StdDev |
|--------|------|-------|--------|
| ffmJson | 3.264 μs | 0.125 μs | 0.007 μs |
| ffmBinary (MemorySegment) | 0.908 μs | 0.007 μs | 0.001 μs |
| ffmBinaryBytes (byte[]) | 0.742 μs | 0.085 μs | 0.005 μs |
| **ffmBinaryZeroCopy** | **0.667 μs** | 0.024 μs | 0.001 μs |

**Linux (JDK 21.0.10)**

| Method | Mean | Error | StdDev |
|--------|------|-------|--------|
| ffmJson | 2.442 μs | 0.040 μs | 0.027 μs |
| ffmBinary (MemorySegment) | 0.624 μs | 0.016 μs | - |
| ffmBinaryBytes (byte[]) | 0.551 μs | 0.005 μs | - |
| **ffmBinaryZeroCopy** | **0.511 μs** | 0.005 μs | - |

**Comparison:**
- **Linux binary zero-copy is 23.4% faster** (667 ns → 511 ns)
- **Linux JSON is 25.2% faster** (3.264 μs → 2.442 μs)
- Zero-copy is **fastest method** on both platforms

**Notes:**
- `ffmBinaryZeroCopy` returns direct native memory reference (no copy)
- `ffmBinaryBytes` returns a `byte[]` (copied to JVM heap)
- `ffmBinary` returns a `MemorySegment` (copied to Arena-managed memory)

### Throughput

**Windows**

| Method | Throughput (ops/s) | Error |
|--------|-------------------|-------|
| ffmBinaryThroughput | 1,349,179 | ±189,407 |
| ffmJsonThroughput | 325,231 | ±11,194 |

**Linux**

| Method | Throughput (ops/s) | Error |
|--------|-------------------|-------|
| ffmBinaryThroughput | 1,828,411 | ±7,722 |
| ffmJsonThroughput | 405,712 | ±4,895 |

**Binary achieves 4.1x higher throughput on Windows, 4.5x on Linux.**
**Linux throughput is 35.5% higher for binary, 24.7% higher for JSON.**

### Concurrent Scaling

**Windows**

| Threads | Binary (ops/s) | JSON (ops/s) | Binary/JSON Ratio |
|---------|----------------|--------------|-------------------|
| 1 | 1,340,305 | 326,371 | 4.1x |
| 4 | 1,817,613 | 835,024 | 2.2x |
| 8 | 2,289,383 | 1,398,446 | 1.6x |

**Linux**

| Threads | Binary (ops/s) | JSON (ops/s) | Binary/JSON Ratio |
|---------|----------------|--------------|-------------------|
| 1 | 1,824,764 | 406,816 | 4.5x |
| 4 | 1,930,779 | 958,152 | 2.0x |
| 8 | 2,656,271 | 1,530,037 | 1.7x |

**Analysis:**
- **Linux consistently faster** across all thread counts (6-36%)
- **Linux shows stronger scaling**: continues improving to 8 threads
- Windows FFM: 1.3M → 2.3M ops/s (1→8 threads)
- Linux FFM: 1.8M → 2.6M ops/s (1→8 threads)

---

## Java: JNI vs FFM Comparison

**Windows**

| Metric | JNI Binary | FFM Binary (zero-copy) | JNI JSON | FFM JSON |
|--------|------------|------------------------|----------|----------|
| Latency | 579 ns | 667 ns | 6.05 μs | 3.26 μs |
| Throughput | 1.76M ops/s | 1.35M ops/s | 167K ops/s | 325K ops/s |

**Linux**

| Metric | JNI Binary | FFM Binary (zero-copy) | JNI JSON | FFM JSON |
|--------|------------|------------------------|----------|----------|
| Latency | 398 ns | 511 ns | 4.09 μs | 2.44 μs |
| Throughput | 2.60M ops/s | 1.83M ops/s | 258K ops/s | 406K ops/s |

**Analysis:**

**Windows:**
- **Binary transport:** JNI is 13% faster than FFM (579 ns vs 667 ns)
- **JSON transport:** FFM is 46% faster than JNI (3.26 μs vs 6.05 μs)

**Linux:**
- **Binary transport:** JNI is 22% faster than FFM (398 ns vs 511 ns) - stronger advantage
- **JSON transport:** FFM is 40% faster than JNI (2.44 μs vs 4.09 μs) - still strong

**Platform Insights:**
- JNI binary advantage is more pronounced on Linux (22% vs 13%)
- FFM JSON advantage is consistent on both platforms (~40-46%)
- JNI has lower overhead for raw byte operations
- FFM has better string handling, likely due to more efficient memory management

---

## Python Benchmarks

### Transport Latency

**Windows (Python 3.13.9)**

| Test | Mean | StdDev | Ops/s |
|------|------|--------|-------|
| Binary (small) | 5.73 μs | 0.77 μs | 174,545 |
| JSON (small math) | 26.70 μs | 7.58 μs | 37,450 |
| JSON (small greet) | 27.63 μs | 7.22 μs | 36,192 |
| JSON (small echo) | 31.59 μs | 35.96 μs | 31,655 |
| JSON (medium ~1KB) | 205.99 μs | 13.24 μs | 4,855 |
| JSON (large ~100KB) | 20,021 μs | 90.25 μs | 50 |

**Linux (Python 3.12.3)**

| Test | Mean | StdDev | Ops/s |
|------|------|--------|-------|
| Binary (small) | 4.92 μs | 0.47 μs | 203,083 |
| JSON (small math) | 25.51 μs | 125.08 μs | 39,207 |
| JSON (small greet) | 23.64 μs | 4.41 μs | 42,299 |
| JSON (small echo) | 26.84 μs | 11.41 μs | 37,261 |
| JSON (medium ~1KB) | 167.56 μs | 7.36 μs | 5,968 |
| JSON (large ~100KB) | 15,862 μs | 159.95 μs | 63 |

**Comparison:**
- **Linux binary is 14.1% faster** (5.73 μs → 4.92 μs)
- **Linux JSON is 4-21% faster** depending on payload
- **Larger payloads show more advantage** on Linux (20.8% faster for 100KB)
- Binary is **4.7-5.5x faster** than JSON on both platforms

### Throughput & Lifecycle

**Windows**

| Test | Mean | Ops/s |
|------|------|-------|
| Sequential (100 calls) | 2,810 μs | 356 |
| Concurrent (100 tasks) | 12,391 μs | 81 |
| Load/unload cycle | 15,672 μs | 64 |

**Linux**

| Test | Mean | Ops/s |
|------|------|-------|
| Sequential (100 calls) | 2,374.93 μs | 421 |
| Concurrent (100 tasks) | 8,465.83 μs | 118 |
| Load/unload cycle | 12,095.28 μs | 83 |

**Comparison:**
- **Linux sequential is 15.5% faster**
- **Linux concurrent is 31.7% faster** (largest advantage)
- **Linux lifecycle is 22.8% faster**

---

## Cross-Language Comparison

### Latency (Lower is Better)

**Windows**

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

**Linux**

```
Binary Transport:
C#              ████ 268 ns
Java JNI        █████ 398 ns
Java FFM        ██████ 511 ns (zero-copy)
Python          █████████████████████████████████████████████ 4,920 ns

JSON Transport:
C#              ██████████ 2,290 ns
Java FFM        ████████████ 2,442 ns
Java JNI        ████████████████ 4,090 ns
Python          ██████████████████████████████████████████████████████████████ 25,510 ns
```

**Summary:**
- **Linux achieves better latency across all languages** (4-32% faster)
- **C# remains fastest** on both platforms
- **Java JNI binary is 2nd fastest** on both platforms
- **Ranking is consistent across platforms**, but Linux amplifies advantages

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
- **Linux deployment preferred** (4-35% faster than Windows)

### When to Use JSON Transport

- Flexibility over raw performance
- Debugging/logging needs (human-readable)
- Variable-schema data
- Interoperability with external systems
- **Performance is consistent across platforms**

### Language Selection

| Use Case | Recommended | Windows Performance | Linux Performance |
|----------|-------------|-------|----------|
| Maximum binary performance | C# | 326 ns | 268 ns ⭐ |
| Maximum JSON performance | C# or Java FFM | 2.55 μs / 3.26 μs | 2.29 μs / 2.44 μs ⭐ |
| Cross-platform desktop | C# (.NET) | Good | Better ⭐ |
| Enterprise/Android | Java FFM (Java 21+) | Good | Better ⭐ |
| Legacy Java (8-20) | Java JNI | Baseline | 31% faster ⭐ |
| Scripting/prototyping | Python | Baseline | 15-32% faster ⭐ |

### Java: JNI vs FFM Decision Guide

| Scenario | Recommendation | Windows | Linux |
|----------|----------------|---------|-------|
| Binary-heavy workload | JNI | 13% faster | **22% faster** ⭐ |
| JSON-heavy workload | FFM | 46% faster | 40% faster |
| Mixed workload | FFM | Better balance | Better balance |
| Java 8-20 compatibility | JNI | Only option | Only option |
| Future-proofing | FFM | Modern API | Modern API, Linux faster ⭐ |

### Java FFM: Which Binary Method?

| Scenario | Recommended Method | Windows | Linux |
|----------|-------------------|---------|-------|
| Maximum performance | `callRawZeroCopy()` | 667 ns | 511 ns ⭐ |
| Need byte[] for existing APIs | `callRawBytes()` | 742 ns | 551 ns ⭐ |
| Need Arena lifetime management | `callRaw()` | 908 ns | 624 ns ⭐ |

### Platform-Specific Guidance

**Linux Deployment:**
- ✅ **Expect 10-35% performance improvement** over Windows
- Concurrent workloads show the largest advantage (~32%)
- Java JNI binary is particularly strong (31% faster than Windows)
- Worth considering for latency-sensitive applications

**Windows Deployment:**
- ✅ **Performance is acceptable** but expect 10-35% slower
- Multi-threaded JNI binary scaling is slightly better (4-thread workloads)
- All languages perform predictably
- Suitable for business applications where platform diversity is required

**Cross-Platform Applications:**
- **Target Linux for deployment** if performance is critical
- **Expect consistent ranking** (C# > Java JNI > Java FFM > Python) on both platforms
- **Binary transport advantage** is consistent (5-10x faster than JSON)

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

### Windows
- **OS:** Windows 11 (10.0.26100.7623)
- **CPU:** AMD Ryzen Threadripper 1950X (16 logical cores)
- **RAM:** Available
- **Runtime Versions:**
  - .NET SDK 8.0.417, Runtime 8.0.23
  - JDK 21.0.9 (Eclipse Adoptium)
  - Python 3.13.9
- **Benchmark Tools:**
  - BenchmarkDotNet v0.14.0
  - JMH 1.37
  - pytest-benchmark 5.2.3

### Linux
- **OS:** Linux (Ubuntu 24.04 LTS, kernel 6.8.0-90-generic)
- **CPU:** AMD Ryzen Threadripper 1950X (16 logical cores) - **Same hardware as Windows**
- **RAM:** 62 GB available
- **Runtime Versions:**
  - .NET SDK 8.0.122, Runtime 8.0.22
  - JDK 21.0.10 (Eclipse Temurin)
  - Python 3.12.3
- **Benchmark Tools:**
  - BenchmarkDotNet v0.14.0
  - JMH 1.37
  - pytest-benchmark 5.2.3

### Both Platforms
- **Rust Plugin:** hello-plugin built with `--release`
- **Hardware:** Identical CPU (AMD Ryzen Threadripper 1950X) enables fair platform comparison
- **Note:** Linux JDK versions are newer (21.0.10 vs 21.0.9 on Windows)
