# rustbridge Tutorials

Learn to build production-quality plugins with rustbridge through hands-on tutorials.

## Overview

These tutorials guide you through building a **regex matching plugin** with LRU caching, configuration, and structured logging. You'll then call it from Kotlin with type-safe wrappers.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          What You'll Build                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Rust Plugin (Chapter 1)              Kotlin Consumer (Chapter 2)           │
│  ─────────────────────────            ────────────────────────────          │
│  • Regex pattern matching             • Load plugin from .rbp bundle        │
│  • LRU cache for patterns             • Type-safe JSON calls                │
│  • Configurable cache size            • Logging callbacks                   │
│  • Structured logging                 • Performance benchmarking            │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Prerequisites

Before starting, ensure you have:

- **Rust 1.90+** with `cargo-generate` installed
- **Java 21+** (for FFM-based Kotlin consumer)
- **rustbridge CLI** installed ([Getting Started](../GETTING_STARTED.md))

```bash
# Install cargo-generate if you haven't already
cargo install cargo-generate
```

## Tutorial Chapters

### [Chapter 1: Building a Regex Plugin](./01-regex-plugin/README.md)

Build a production-quality Rust plugin from scratch.

| Section | What You'll Learn |
|---------|-------------------|
| [01-scaffold.md](./01-regex-plugin/01-scaffold.md) | Generate project with cargo-generate |
| [02-basic-matching.md](./01-regex-plugin/02-basic-matching.md) | Define messages, implement matching |
| [03-lru-cache.md](./01-regex-plugin/03-lru-cache.md) | Add LRU cache, measure performance |
| [04-configuration.md](./01-regex-plugin/04-configuration.md) | Make cache size configurable |

### [Chapter 2: Calling from Kotlin](./02-kotlin-consumer/README.md)

Call your plugin from Kotlin with type safety and logging.

| Section | What You'll Learn |
|---------|-------------------|
| [01-project-setup.md](./02-kotlin-consumer/01-project-setup.md) | Set up Gradle project, build bundle |
| [02-calling-plugin.md](./02-kotlin-consumer/02-calling-plugin.md) | Load bundle, make JSON calls |
| [03-logging-callbacks.md](./02-kotlin-consumer/03-logging-callbacks.md) | Capture plugin logs in Kotlin |
| [04-type-safe-calls.md](./02-kotlin-consumer/04-type-safe-calls.md) | Data classes, extension functions |
| [05-benchmarking.md](./02-kotlin-consumer/05-benchmarking.md) | Debug vs release, cache effectiveness |

## Reference Implementation

The completed plugin is available at [`examples/regex-plugin/`](../../examples/regex-plugin/) for reference. If you get stuck, compare your code against the working example.

## Choosing Your Path

**New to rustbridge?** Start with the [Getting Started Guide](../GETTING_STARTED.md), then return here for the deeper dive.

**Want to skip the tutorial?** Use the completed [regex-plugin example](../../examples/regex-plugin/) or generate a plugin with all features:

```bash
cargo generate --git https://github.com/jrobhoward/rustbridge --path templates/tutorial-plugin
```

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/jrobhoward/rustbridge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/jrobhoward/rustbridge/discussions)
