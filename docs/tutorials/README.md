# rustbridge Tutorials

Learn advanced rustbridge topics through hands-on tutorials.

## Prerequisites

Before starting these tutorials, complete the installation steps in the [README](../../README.md#install-from-source):

1. Clone the repository and install the CLI
2. Install host language libraries for your target language(s)

Verify your installation:

```bash
rustbridge --version
```

You should also complete the [Getting Started Guide](../GETTING_STARTED.md) to build your first plugin.

## Overview

These tutorials cover advanced topics for production-ready plugins. Each assumes you have the rustbridge CLI installed and your host language libraries set up.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          What You'll Build                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Production Bundles (Chapter 1)       Cross-Compilation (Chapter 2)        │
│  ──────────────────────────────       ──────────────────────────────       │
│  • Code signing with minisign         • Multi-platform bundles             │
│  • JSON schemas for validation        • Native and cross builds            │
│  • Build metadata and provenance      • Bundle combining                   │
│  • SBOM for compliance                                                     │
│                                                                             │
│  Backpressure Queues (Chapter 3)      Binary Transport (Chapter 4)         │
│  ───────────────────────────────      ────────────────────────────         │
│  • C# and Python consumers            • Image thumbnail generator          │
│  • Bounded queues for flow control    • C-compatible struct layouts        │
│  • Block producers when queue full    • 7x faster than JSON for binaries   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Tutorial Chapters

### [Chapter 1: Production Bundles](./01-production-bundles/README.md)

Create production-ready bundles with signing, schemas, and compliance features.

| Section | What You'll Learn |
|---------|-------------------|
| [01-code-signing.md](./01-production-bundles/01-code-signing.md) | Generate keys, sign bundles with minisign |
| [02-json-schemas.md](./01-production-bundles/02-json-schemas.md) | Embed schemas for validation |
| [03-build-metadata.md](./01-production-bundles/03-build-metadata.md) | Include provenance and git info |
| [04-sbom.md](./01-production-bundles/04-sbom.md) | Add Software Bill of Materials |

### [Chapter 2: Cross-Compilation](./02-cross-compilation/README.md)

Build multi-platform bundles for Linux, macOS, and Windows.

| Section | What You'll Learn |
|---------|-------------------|
| [01-platform-overview.md](./02-cross-compilation/01-platform-overview.md) | Platform identifiers and target triples |
| [02-native-toolchains.md](./02-cross-compilation/02-native-toolchains.md) | Build natively on each platform |
| [03-cross-compilation.md](./02-cross-compilation/03-cross-compilation.md) | Cross-compile with `cross` or cargo |

### [Chapter 3: Backpressure Queues](./03-backpressure-queues/README.md)

Implement bounded queues with backpressure for flow control in C# and Python.

| Section | What You'll Learn |
|---------|-------------------|
| [01-csharp-consumer.md](./03-backpressure-queues/01-csharp-consumer.md) | C# with BlockingCollection and Task |
| [02-python-consumer.md](./03-backpressure-queues/02-python-consumer.md) | Python with queue.Queue and concurrent.futures |

### [Chapter 4: Binary Transport](./04-binary-transport/README.md)

Build an image thumbnail generator using binary transport for efficient large payload handling.

| Section | What You'll Learn |
|---------|-------------------|
| [01-java-ffm-consumer.md](./04-binary-transport/01-java-ffm-consumer.md) | Java 21+ FFM with StructLayout and VarHandle |
| [02-kotlin-consumer.md](./04-binary-transport/02-kotlin-consumer.md) | Kotlin FFM with extension functions |
| [03-csharp-consumer.md](./04-binary-transport/03-csharp-consumer.md) | C# with StructLayout and Marshal |
| [04-python-consumer.md](./04-binary-transport/04-python-consumer.md) | Python with ctypes.Structure |

## Reference Implementations

Completed examples are available for reference. If you get stuck, compare your code against these working implementations:

- **Hello plugin**: [`examples/hello-plugin/`](../../examples/hello-plugin/)

## Choosing Your Path

**New to rustbridge?** Start with the [Getting Started Guide](../GETTING_STARTED.md), then return here for advanced topics.

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/jrobhoward/rustbridge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/jrobhoward/rustbridge/discussions)
