# RustBridge Benchmark Results

**Date:** 2026-01-25
**System:** Windows 11, AMD Ryzen Threadripper 1950X (16 cores), .NET 8.0.23, JDK 21.0.9, Python 3.13.9
**Plugin:** hello-plugin (release build)

## Summary

| Language | Transport | Latency | Throughput | Memory |
|----------|-----------|---------|------------|--------|
| **C#** | Binary | 360 ns | 3.2M ops/s | 40 B |
| **C#** | JSON | 2.72 μs | 385K ops/s | 688 B |
| **Java JNI** | Binary | 602 ns | 1.68M ops/s | - |
| **Java JNI** | JSON | 5.81 μs | 173K ops/s | - |
| **Java FFM** | Binary (bytes) | 728 ns | 1.35M ops/s | - |
| **Java FFM** | Binary (segment) | 1.16 μs | 1.35M ops/s | - |
| **Java FFM** | JSON | 3.11 μs | 328K ops/s | - |
| **Python** | Binary | 18.2 μs | 55K ops/s | - |
| **Python** | JSON | 26-31 μs | 32-38K ops/s | - |

**Key Findings:**
- Binary transport is **7-10x faster** than JSON across all languages
- C# achieves the lowest latency (360 ns binary, 2.72 μs JSON)
- Java JNI binary (602 ns) outperforms Java FFM binary (728 ns)
- Java FFM JSON (3.11 μs) outperforms Java JNI JSON (5.81 μs)
- Python has higher overhead due to ctypes FFI layer

---

## C# Benchmarks (.NET 8.0)

### Transport Latency

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON transport | 2,722.2 ns | 7.24 ns | 6.77 ns | 1.00 | 688 B |
| Binary transport | 359.6 ns | 0.64 ns | 0.60 ns | 0.13 | 40 B |

**Binary is 7.6x faster than JSON with 17x less memory allocation.**

### Throughput (1000 ops/invoke)

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON throughput | 2,597.4 ns | 10.03 ns | 9.39 ns | 1.00 | 696 B |
| Binary throughput | 312.9 ns | 0.43 ns | 0.40 ns | 0.12 | 40 B |

**Binary achieves 8.3x higher throughput.**

### Concurrent (100 parallel tasks)

| Method | Mean | Error | StdDev | Ratio | Allocated |
|--------|------|-------|--------|-------|-----------|
| JSON concurrent | 124.64 μs | 1.027 μs | 0.911 μs | 1.00 | 83.91 KB |
| Binary concurrent | 69.16 μs | 1.155 μs | 1.080 μs | 0.55 | 33.90 KB |

**Binary concurrent calls complete in 55% of the time with 40% memory.**

---

## Java JNI Benchmarks (JDK 21)

### Transport Latency

| Method | Mean | Error | StdDev |
|--------|------|-------|--------|
| jniJson | 5.806 μs | 2.570 μs | 0.141 μs |
| jniBinary | 0.602 μs | 0.044 μs | 0.002 μs |

**Binary is 9.6x faster than JSON.**

### Throughput

| Method | Throughput (ops/s) | Error |
|--------|-------------------|-------|
| jniBinaryThroughput | 1,682,356 | ±341,651 |
| jniJsonThroughput | 172,586 | ±10,359 |

**Binary achieves 9.7x higher throughput.**

### Concurrent Scaling

| Threads | Binary (ops/s) | JSON (ops/s) | Binary/JSON Ratio |
|---------|----------------|--------------|-------------------|
| 1 | 1,553,367 | 173,019 | 9.0x |
| 4 | 2,389,852 | 582,815 | 4.1x |
| 8 | 1,670,946 | 1,133,888 | 1.5x |

---

## Java FFM Benchmarks (JDK 21)

### Transport Latency

| Method | Mean | Error | StdDev |
|--------|------|-------|--------|
| ffmJson | 3.110 μs | 0.173 μs | 0.009 μs |
| ffmBinary (MemorySegment) | 1.157 μs | 0.121 μs | 0.007 μs |
| ffmBinaryBytes (byte[]) | 0.728 μs | 0.087 μs | 0.005 μs |

**Notes:**
- `ffmBinary` returns a `MemorySegment` (zero-copy, stays in native memory)
- `ffmBinaryBytes` returns a `byte[]` (copied to JVM heap)
- `ffmBinaryBytes` is faster because it avoids Arena allocation overhead in the hot path

### Throughput

| Method | Throughput (ops/s) | Error |
|--------|-------------------|-------|
| ffmBinaryThroughput | 1,348,265 | ±35,528 |
| ffmJsonThroughput | 328,142 | ±2,064 |

**Binary achieves 4.1x higher throughput.**

### Concurrent Scaling

| Threads | Binary (ops/s) | JSON (ops/s) | Binary/JSON Ratio |
|---------|----------------|--------------|-------------------|
| 1 | 1,335,285 | 289,475 | 4.6x |
| 4 | 1,530,894 | 822,239 | 1.9x |
| 8 | 2,126,683 | 1,305,481 | 1.6x |

---

## Java: JNI vs FFM Comparison

| Metric | JNI Binary | FFM Binary | JNI JSON | FFM JSON |
|--------|------------|------------|----------|----------|
| Latency | 602 ns | 728 ns | 5.81 μs | 3.11 μs |
| Throughput | 1.68M ops/s | 1.35M ops/s | 173K ops/s | 328K ops/s |

**Analysis:**
- **Binary transport:** JNI is 17% faster than FFM (602 ns vs 728 ns)
- **JSON transport:** FFM is 47% faster than JNI (3.11 μs vs 5.81 μs)
- JNI has lower overhead for raw byte operations
- FFM has better string handling, likely due to more efficient memory management

---

## Python Benchmarks (Python 3.13)

### Transport Latency

| Test | Mean | StdDev | Ops/s |
|------|------|--------|-------|
| Binary (small) | 18.21 μs | 8.48 μs | 54,907 |
| JSON (small math) | 26.33 μs | 8.32 μs | 37,981 |
| JSON (small greet) | 28.08 μs | 8.21 μs | 35,607 |
| JSON (small echo) | 30.69 μs | 32.94 μs | 32,580 |
| JSON (medium ~1KB) | 207.63 μs | 12.65 μs | 4,816 |
| JSON (large ~100KB) | 19,834 μs | 223.02 μs | 50 |

**Binary is 1.5-1.7x faster than JSON for small payloads.**

### Throughput & Lifecycle

| Test | Mean | Ops/s |
|------|------|-------|
| Sequential (100 calls) | 2,809 μs | 356 |
| Concurrent (100 tasks) | 12,510 μs | 80 |
| Load/unload cycle | 15,456 μs | 65 |

---

## Cross-Language Comparison

### Latency (Lower is Better)

```
Binary Transport:
C#          ████ 360 ns
Java JNI    ██████ 602 ns
Java FFM    ███████ 728 ns
Python      ██████████████████████████████████████████████████████████████████████████ 18,213 ns

JSON Transport:
C#          █████████ 2,722 ns
Java FFM    ██████████ 3,110 ns
Java JNI    ███████████████████ 5,806 ns
Python      █████████████████████████████████████████████████████████████████████████████ 26,329 ns
```

### Why the Performance Differences?

1. **C# P/Invoke** - Direct native interop with minimal overhead, struct marshaling is highly optimized
2. **Java JNI** - Traditional native interface, excellent for binary data, higher overhead for strings
3. **Java FFM** - Modern foreign function API, better string handling, some overhead for memory management
4. **Python ctypes** - Interpreted language with dynamic typing adds significant overhead

### Binary vs JSON Speedup

| Language | Speedup Factor |
|----------|---------------|
| Java JNI | 9.6x |
| C# | 7.6x |
| Java FFM | 4.3x |
| Python | 1.5x |

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
| Binary-heavy workload | JNI (17% faster) |
| JSON-heavy workload | FFM (47% faster) |
| Mixed workload | FFM (better overall balance) |
| Java 8-20 compatibility | JNI (only option) |
| Future-proofing | FFM (modern API, will improve) |

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
