# rustbridge Tutorials

Learn to build custom plugins with rustbridge through hands-on tutorials.

## Overview

These tutorials guide you through building Rust plugins and calling them from multiple languages. Start with the regex plugin to learn core concepts, then explore JSON processing and language-specific consumers.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          What You'll Build                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Rust Plugins (Chapters 1, 3)         Language Consumers (Chapters 2, 4)   │
│  ─────────────────────────────        ──────────────────────────────────   │
│  • Regex pattern matching             • Kotlin: Load bundles, logging      │
│  • LRU cache for patterns             • Java: FFM API, type-safe calls     │
│  • JSON validation/prettify           • Error handling patterns            │
│  • Configurable behavior              • Performance benchmarking           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Prerequisites

Before starting, ensure you have:

- **Rust 1.90+** installed
- **Java 21+** (for FFM-based Kotlin consumer)
- **rustbridge CLI** installed ([Getting Started](../GETTING_STARTED.md))

## Tutorial Chapters

### [Chapter 1: Building a Regex Plugin](./01-regex-plugin/README.md)

Build a production-quality Rust plugin from scratch.

| Section | What You'll Learn |
|---------|-------------------|
| [01-scaffold.md](./01-regex-plugin/01-scaffold.md) | Generate project with `rustbridge new` |
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

### [Chapter 3: Building a JSON Plugin](./03-json-plugin/README.md)

Build a JSON validation and prettification plugin to learn message handling patterns.

| Section | What You'll Learn |
|---------|-------------------|
| [01-scaffold.md](./03-json-plugin/01-scaffold.md) | Generate project structure |
| [02-validate-message.md](./03-json-plugin/02-validate-message.md) | Implement JSON validation endpoint |
| [03-prettify-message.md](./03-json-plugin/03-prettify-message.md) | Add JSON prettification with custom indent |
| [04-error-handling.md](./03-json-plugin/04-error-handling.md) | Error patterns, build bundle |

### [Chapter 4: Calling from Java](./04-java-consumer/README.md)

Load your plugin from Java using the Foreign Function & Memory (FFM) API.

| Section | What You'll Learn |
|---------|-------------------|
| [01-project-setup.md](./04-java-consumer/01-project-setup.md) | Set up Java FFM consumer project |
| [02-calling-plugin.md](./04-java-consumer/02-calling-plugin.md) | Load bundle, type-safe calls with records |
| [03-error-handling.md](./04-java-consumer/03-error-handling.md) | Handle plugin errors gracefully |

## Reference Implementations

Completed examples are available for reference. If you get stuck, compare your code against these working implementations:

- **Regex plugin** (Chapters 1-2): [`examples/regex-plugin/`](../../examples/regex-plugin/)
- **JSON plugin** (Chapters 3-4): [`examples/json-plugin/`](../../examples/json-plugin/)

## Choosing Your Path

**New to rustbridge?** Start with the [Getting Started Guide](../GETTING_STARTED.md), then return here for the deeper dive.

**Want to skip the tutorial?** Use the completed examples as a reference:

```bash
# Copy the regex plugin example
cp -r ~/rustbridge-workspace/rustbridge/examples/regex-plugin ~/rustbridge-workspace/my-plugin

# Or the JSON plugin example
cp -r ~/rustbridge-workspace/rustbridge/examples/json-plugin ~/rustbridge-workspace/my-plugin
```

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/jrobhoward/rustbridge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/jrobhoward/rustbridge/discussions)
