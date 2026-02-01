# Chapter 6: Cross-Compilation

In this chapter, you'll learn how to build multi-platform plugin bundles for Linux, macOS, and Windows.

## What You'll Learn

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                       Multi-Platform Bundle                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  my-plugin-0.1.0.rbp                                                        │
│  ────────────────────                                                       │
│  └── lib/                                                                   │
│      ├── linux-x86_64/                                                      │
│      │   └── release/                                                       │
│      │       └── libmy_plugin.so        ← Linux x86_64                      │
│      ├── linux-aarch64/                                                     │
│      │   └── release/                                                       │
│      │       └── libmy_plugin.so        ← Linux ARM64 (Graviton, RPi)       │
│      ├── darwin-x86_64/                                                     │
│      │   └── release/                                                       │
│      │       └── libmy_plugin.dylib     ← macOS Intel                       │
│      ├── darwin-aarch64/                                                    │
│      │   └── release/                                                       │
│      │       └── libmy_plugin.dylib     ← macOS Apple Silicon               │
│      └── windows-x86_64/                                                    │
│          └── release/                                                       │
│              └── my_plugin.dll          ← Windows x64                       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Build Strategies

| Strategy | Best For | Complexity |
|----------|----------|------------|
| **Native builds** | Most projects | Low |
| **Cross-compilation** | Pure Rust projects | Medium |

## Sections

### [01: Platform Overview](./01-platform-overview.md)
Understand platform identifiers, library naming, and target triples.

### [02: Native Toolchains](./02-native-toolchains.md)
Build natively on each platform (recommended approach).

### [03: Cross-Compilation](./03-cross-compilation.md)
Cross-compile from a single machine using `cross` or cargo targets.

## Prerequisites

- Completed Chapter 5 (Production Bundles)
- Access to target platforms (physical, VM, or CI)

## Recommended Approach

For most projects, **native builds** are simplest:

1. Build natively on each target platform
2. Create platform-specific bundles
3. Combine into a single multi-platform bundle using `rustbridge bundle combine`

This avoids cross-compilation complexity and uses native toolchains for best compatibility.

## Next Steps

After completing this chapter, continue to [Chapter 7: Backpressure Queues](../07-backpressure-queues/README.md) to learn how to implement bounded queues with flow control for C#, Java/JNI, and Python consumers.

For Java 17-20 support without FFM, see the [Appendix: Java JNI](../appendix-java-jni/README.md).
