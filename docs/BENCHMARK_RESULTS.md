# RustBridge Benchmark Results

**Version:** 0.8.0
**Updated:** 2026-02-01

## Executive Summary

RustBridge supports three host languages (C#, Java, Python) with two transport options (JSON and binary). This document provides benchmark data to help you make informed decisions, but **don't let raw numbers drive premature optimization**.

### The Bottom Line

| Transport | Best For | Trade-off |
|-----------|----------|-----------|
| **JSON** | Most applications | Easier debugging, flexible schemas, faster development |
| **Binary** | High-frequency hot paths | 3-9x faster, but harder to debug and maintain |

> **Recommendation:** Start with JSON. It's human-readable, easier to debug, and simpler to evolve. Only switch to binary for specific hot paths where profiling shows it matters. In most business applications, the difference between 300 ns and 2 μs is invisible to users but the difference in development time is very real.

---

## Quick Reference

### All Platforms Summary

| Platform | Language | Binary | JSON | Speedup |
|----------|----------|--------|------|---------|
| **macOS M1** | C# | 136 ns | 1.08 μs | 7.9x |
| **macOS M1** | Java FFM | 384 ns | 1.28 μs | 3.3x |
| **macOS M1** | Python | 1.94 μs | 8.7 μs | 4.5x |
| **Linux x86** | C# | 259 ns | 2.38 μs | 9.2x |
| **Linux x86** | Java FFM | 366 ns | 2.20 μs | 6.0x |
| **Linux x86** | Python | 4.83 μs | 24.8 μs | 5.1x |
| **Windows x86** | C# | 305 ns | 2.52 μs | 8.2x |
| **Windows x86** | Java FFM | 667 ns | 3.26 μs | 4.9x |
| **Windows x86** | Python | 5.66 μs | 27 μs | 4.8x |

### Key Observations

1. **Apple Silicon is fast** - M1 achieves the lowest latencies across all languages
2. **Linux outperforms Windows** on x86 by 15-45% depending on workload
3. **Binary is 3-9x faster** than JSON, but this rarely matters in practice
4. **C# achieves lowest latency**, Java FFM is competitive, Python is adequate for most uses

---

## A Word on Premature Optimization

Binary transport is faster. That's undeniable. But consider what "faster" means in context:

| Scenario | JSON Latency | Binary Latency | Actual Difference |
|----------|--------------|----------------|-------------------|
| Single API call | 2.5 μs | 300 ns | 2.2 μs saved |
| 100 calls/second | 250 μs total | 30 μs total | 220 μs saved |
| 10,000 calls/second | 25 ms total | 3 ms total | 22 ms saved |

**At 100 calls/second**, you save 220 microseconds. Your users will never notice.

**At 10,000 calls/second**, you save 22 milliseconds. This might matter if it's on a critical path.

Meanwhile, binary transport requires:
- Manually maintaining C struct layouts in multiple languages
- Careful attention to padding, alignment, and endianness
- More complex debugging (no human-readable payloads)
- Tighter coupling between Rust and host language code

**JSON gives you:**
- Human-readable payloads for logging and debugging
- Flexible schema evolution (add fields without breaking clients)
- Simpler code that's easier to maintain
- Faster development iteration

**Use binary transport when:**
- Profiling shows the FFI boundary is a bottleneck
- You're making >1,000 calls/second on a latency-critical path
- The data schema is stable and unlikely to change
- You have the engineering resources to maintain struct parity

**Use JSON transport when:**
- You're building a new feature (start simple, optimize later)
- Debugging or observability is important
- Schema flexibility matters
- Development velocity is more valuable than raw performance

---

## Platform Comparison

### By Operating System (x86-64)

Comparing Linux and Windows on identical hardware (AMD Ryzen Threadripper 1950X):

| Language | Transport | Linux | Windows | Linux Advantage |
|----------|-----------|-------|---------|-----------------|
| C# | Binary | 259 ns | 305 ns | **15% faster** |
| C# | JSON | 2.38 μs | 2.52 μs | **6% faster** |
| Java FFM | Binary | 366 ns | 667 ns | **45% faster** |
| Java FFM | JSON | 2.20 μs | 3.26 μs | **33% faster** |
| Python | Binary | 4.83 μs | 5.66 μs | **15% faster** |
| Python | JSON | 24.8 μs | 27 μs | **8% faster** |

**Analysis:**
- Linux consistently outperforms Windows across all languages
- Java FFM shows the largest Linux advantage (33-45%)
- For latency-sensitive deployments, prefer Linux

### By Architecture (x86 vs ARM)

Comparing x86-64 (Linux, Threadripper) vs ARM64 (macOS, M1):

| Language | Transport | x86 Linux | ARM macOS | ARM Advantage |
|----------|-----------|-----------|-----------|---------------|
| C# | Binary | 259 ns | 136 ns | **47% faster** |
| C# | JSON | 2.38 μs | 1.08 μs | **55% faster** |
| Java FFM | Binary | 366 ns | 384 ns | ~equal |
| Java FFM | JSON | 2.20 μs | 1.28 μs | **42% faster** |
| Python | Binary | 4.83 μs | 1.94 μs | **60% faster** |
| Python | JSON | 24.8 μs | 8.7 μs | **65% faster** |

**Analysis:**
- Apple M1 significantly outperforms x86 for C# and Python
- Java FFM binary is roughly equivalent; JSON is much faster on M1
- M1's unified memory architecture benefits FFI-heavy workloads
- Python sees the largest gains on ARM (60-65%)

---

## Language Comparison

### C# (.NET 8.0)

C# consistently achieves the lowest latencies via P/Invoke's optimized struct marshaling.

| Platform | Binary | JSON | Memory (Binary) |
|----------|--------|------|-----------------|
| macOS M1 | 136 ns | 1.08 μs | 40 B |
| Linux x86 | 259 ns | 2.38 μs | 40 B |
| Windows x86 | 305 ns | 2.52 μs | 40 B |

**Throughput (ops/s):**
| Platform | Binary | JSON |
|----------|--------|------|
| macOS M1 | 7.2M | 894K |
| Linux x86 | 4.15M | 440K |
| Windows x86 | 3.4M | 403K |

### Java FFM (Java 21+)

Java FFM provides excellent performance without JNI complexity. JDK version significantly impacts performance.

| Platform | JDK | Binary | JSON |
|----------|-----|--------|------|
| macOS M1 | 22 | 384 ns | 1.28 μs |
| Linux x86 | 25 | 366 ns | 2.20 μs |
| Windows x86 | 21 | 667 ns | 3.26 μs |

**JDK Version Impact (Linux x86):**
| JDK | Binary Latency | Notes |
|-----|----------------|-------|
| 21 (preview) | ~511 ns | Requires `--enable-preview` |
| 22-24 (stable) | ~450 ns | FFM APIs stable |
| 25 | 366 ns | Best performance |

**Recommendation:** Use JDK 22+ for stable FFM APIs. JDK 25 provides measurable performance improvements.

### Python (3.10+)

Python has the highest latency due to interpreter overhead, but remains practical for many use cases.

| Platform | Binary | JSON |
|----------|--------|------|
| macOS M1 | 1.94 μs | 8.7 μs |
| Linux x86 | 4.83 μs | 24.8 μs |
| Windows x86 | 5.66 μs | 27 μs |

**Throughput (ops/s):**
| Platform | Binary | JSON |
|----------|--------|------|
| macOS M1 | 515K | 115K |
| Linux x86 | 207K | 40K |
| Windows x86 | 177K | 38K |

**Note:** Python performance varies significantly by platform. macOS M1 achieves 2-3x better performance than x86 platforms.

---

## Transport Comparison

### Binary vs JSON Speedup by Language

| Language | macOS M1 | Linux x86 | Windows x86 |
|----------|----------|-----------|-------------|
| C# | 7.9x | 9.2x | 8.2x |
| Java FFM | 3.3x | 6.0x | 4.9x |
| Python | 4.5x | 5.1x | 4.8x |

### Memory Allocation (C#)

| Transport | Allocation per Call |
|-----------|---------------------|
| Binary | 40 B |
| JSON | 688 B |

Binary allocates **17x less memory**, which matters for GC-sensitive applications.

### When Binary Speedup Matters

| Calls/Second | Time Saved (JSON → Binary) | Verdict |
|--------------|----------------------------|---------|
| 10 | 22 μs/sec | Irrelevant |
| 100 | 220 μs/sec | Irrelevant |
| 1,000 | 2.2 ms/sec | Marginal |
| 10,000 | 22 ms/sec | Consider binary |
| 100,000 | 220 ms/sec | Use binary |

---

## Concurrent Scaling

### Java FFM (Linux x86, JDK 25)

| Threads | Binary (ops/s) | JSON (ops/s) |
|---------|----------------|--------------|
| 1 | 2.74M | 456K |
| 4 | 2.37M | 1.06M |
| 8 | 3.00M | 1.67M |

### C# Concurrent (100 parallel tasks)

| Platform | Binary | JSON | Binary Advantage |
|----------|--------|------|------------------|
| macOS M1 | 37.8 μs | 141.2 μs | 73% faster |
| Linux x86 | ~70 μs | ~125 μs | ~44% faster |
| Windows x86 | 71.3 μs | 128.1 μs | 44% faster |

---

## Recommendations

### Language Selection

| Use Case | Recommended | Why |
|----------|-------------|-----|
| Maximum performance | C# (.NET 8+) | Lowest latency, excellent ARM support |
| Enterprise/Server | Java FFM (JDK 22+) | Mature ecosystem, good performance |
| Scripting/Automation | Python 3.10+ | Rapid development, adequate performance |
| Cross-platform desktop | C# or Java | Both have excellent cross-platform support |

### Platform Selection

| Priority | Recommendation |
|----------|----------------|
| Lowest latency | macOS ARM64 (M1/M2/M3) |
| Best x86 performance | Linux |
| Windows required | Expect 15-45% higher latency than Linux |

### Transport Selection

| Scenario | Use JSON | Use Binary |
|----------|----------|------------|
| New feature development | Yes | |
| Debugging/troubleshooting | Yes | |
| Schema likely to change | Yes | |
| Calls < 1,000/sec | Yes | |
| Proven hot path > 10,000/sec | | Yes |
| Memory-constrained environment | | Yes |
| Stable, well-defined schema | | Yes |

---

## Reproducing These Benchmarks

### Prerequisites

```bash
cargo build --release -p hello-plugin
```

### C# Benchmarks

```bash
cd rustbridge-csharp
dotnet run -c Release --project RustBridge.Benchmarks -- --filter "*"
```

### Java Benchmarks

```bash
cd rustbridge-java
./gradlew :rustbridge-benchmarks:jmhJar

java --enable-native-access=ALL-UNNAMED \
  -jar rustbridge-benchmarks/build/libs/rustbridge-benchmarks-0.8.0-jmh.jar \
  -f 2 -wi 3 -i 5
```

Note: For Java 21, add `--enable-preview`.

### Python Benchmarks

```bash
cd rustbridge-python
pip install -e ".[dev]"
python -m pytest tests/test_benchmarks.py tests/test_binary_transport.py -v \
  --benchmark-only --benchmark-columns=mean,stddev,ops
```

---

## Test Environments

### Linux (x86-64)
- **OS:** Ubuntu 24.04 LTS (kernel 6.8.0-90-generic)
- **CPU:** AMD Ryzen Threadripper 1950X (16 cores)
- **Runtimes:** .NET 8.0.22, JDK 25.0.2 (Azul Zulu), Python 3.12.3

### Windows (x86-64)
- **OS:** Windows 11 (10.0.26100.7623)
- **CPU:** AMD Ryzen Threadripper 1950X (16 cores)
- **Runtimes:** .NET 8.0.23, JDK 21.0.9 (Eclipse Adoptium), Python 3.13.9

### macOS (ARM64)
- **OS:** macOS 26.2
- **CPU:** Apple M1 (8 cores: 4P + 4E)
- **Runtimes:** .NET 8.0.23, JDK 22.0.2 (Azul Zulu), Python 3.10.19

---

## Historical Notes

### Changes in v0.8.0
- **JNI Removed:** Java integration now uses FFM exclusively (Java 21+)
- **Simplified Setup:** No native library compilation required for Java
- **Performance Improved:** JDK 25 FFM is 28% faster than JDK 21 preview

### Archived Data
Previous benchmark data including JNI comparisons is available in the git history.
