# rustbridge Tutorials

Learn rustbridge through hands-on tutorials, from building your first plugin to advanced production topics.

## Prerequisites

Before starting these tutorials, complete the installation steps in the [README](../../README.md#install-from-source):

1. Clone the repository
2. Install the rustbridge CLI
3. Install host language libraries for your target language(s)

Verify your installation:

```bash
rustbridge --version
```

You should also complete the [Getting Started Guide](../GETTING_STARTED.md) to build your first plugin.

## Environment Setup

To make commands easy to copy-paste, set up a workspace directory for rustbridge and your tutorial projects.

**Linux/macOS** - Add to your `~/.bashrc`, `~/.zshrc`, or `~/.profile`:

```bash
export RUSTBRIDGE_WORKSPACE="$HOME/rustbridge-workspace"
```

Then reload your shell (`source ~/.bashrc` or open a new terminal).

**Windows** - Set in PowerShell (one-time setup):

```powershell
[Environment]::SetEnvironmentVariable("RUSTBRIDGE_WORKSPACE", "$HOME\rustbridge-workspace", "User")
```

> **Note**: Windows users should translate bash commands (e.g., `cp` → `copy`, `./gradlew` → `gradlew.bat`). The tutorial uses bash syntax throughout.

### Workspace Structure

Your workspace will contain the rustbridge repository and tutorial projects:

```
$RUSTBRIDGE_WORKSPACE/
├── rustbridge/              # Clone of the rustbridge repo (from README installation)
├── regex-plugin/            # Chapter 1: Your regex plugin
├── regex-kotlin-app/        # Chapter 2: Kotlin consumer project
├── json-plugin/             # Chapter 3: Your JSON plugin
├── json-java-app/           # Chapter 4: Java consumer project
└── ...                      # Other tutorial projects
```

Create the workspace and verify:

```bash
mkdir -p $RUSTBRIDGE_WORKSPACE
echo $RUSTBRIDGE_WORKSPACE   # Should print your workspace path
```

> **Safety note**: Some tutorials include cleanup commands with `rm -rf`. Always verify `$RUSTBRIDGE_WORKSPACE` is set before running destructive commands:
> ```bash
> # Safe pattern - only runs if variable is set
> [ -n "$RUSTBRIDGE_WORKSPACE" ] && rm -rf "$RUSTBRIDGE_WORKSPACE/some-project"
> ```

## Overview

These tutorials progress from basic plugin development to advanced production topics.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                          What You'll Build                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  PART 1: BUILDING PLUGINS                                                   │
│  ────────────────────────                                                   │
│                                                                             │
│  Regex Plugin (Chapter 1)              JSON Plugin (Chapter 3)              │
│  ─────────────────────────             ─────────────────────────            │
│  • Project scaffolding                 • JSON validation handler            │
│  • Basic regex matching                • Pretty-print handler               │
│  • LRU caching for performance         • Structured error handling          │
│  • Runtime configuration                                                    │
│                                                                             │
│  PART 2: CONSUMING PLUGINS                                                  │
│  ─────────────────────────                                                  │
│                                                                             │
│  Kotlin Consumer (Chapter 2)           Java Consumer (Chapter 4)            │
│  ────────────────────────────          ──────────────────────────           │
│  • FFM project setup                   • FFM project setup                  │
│  • Calling plugin methods              • Calling plugin methods             │
│  • Logging callbacks                   • Error handling patterns            │
│  • Type-safe wrappers                                                       │
│  • JMH benchmarking                                                         │
│                                                                             │
│  PART 3: ADVANCED TOPICS                                                    │
│  ───────────────────────                                                    │
│                                                                             │
│  Production Bundles (Chapter 5)        Cross-Compilation (Chapter 6)        │
│  ──────────────────────────────        ──────────────────────────────       │
│  • Code signing with minisign          • Multi-platform bundles             │
│  • JSON schemas for validation         • Native and cross builds            │
│  • Build metadata and provenance       • Bundle combining                   │
│  • SBOM for compliance                                                      │
│                                                                             │
│  Backpressure Queues (Chapter 7)       Binary Transport (Chapter 8)         │
│  ───────────────────────────────       ────────────────────────────         │
│  • C# and Python consumers             • Image thumbnail generator          │
│  • Bounded queues for flow control     • C-compatible struct layouts        │
│  • Block producers when queue full     • 7x faster than JSON for binaries   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Tutorial Chapters

### Part 1: Building Plugins

#### [Chapter 1: Regex Plugin](./01-regex-plugin/README.md)

Build a regex matching plugin with caching and runtime configuration.

| Section | What You'll Learn |
|---------|-------------------|
| [01-scaffold.md](./01-regex-plugin/01-scaffold.md) | Create project structure and basic setup |
| [02-basic-matching.md](./01-regex-plugin/02-basic-matching.md) | Implement regex matching handler |
| [03-lru-cache.md](./01-regex-plugin/03-lru-cache.md) | Add LRU caching for compiled regexes |
| [04-configuration.md](./01-regex-plugin/04-configuration.md) | Support runtime configuration |

#### [Chapter 3: JSON Plugin](./03-json-plugin/README.md)

Build a JSON validation and formatting plugin with structured error handling.

| Section | What You'll Learn |
|---------|-------------------|
| [01-scaffold.md](./03-json-plugin/01-scaffold.md) | Create project structure |
| [02-validate-message.md](./03-json-plugin/02-validate-message.md) | Implement JSON validation |
| [03-prettify-message.md](./03-json-plugin/03-prettify-message.md) | Add pretty-print formatting |
| [04-error-handling.md](./03-json-plugin/04-error-handling.md) | Structured error responses |

### Part 2: Consuming Plugins

#### [Chapter 2: Kotlin Consumer](./02-kotlin-consumer/README.md)

Consume plugins from Kotlin using FFM (Foreign Function & Memory API).

| Section | What You'll Learn |
|---------|-------------------|
| [01-project-setup.md](./02-kotlin-consumer/01-project-setup.md) | Set up Kotlin/Gradle project with FFM |
| [02-calling-plugin.md](./02-kotlin-consumer/02-calling-plugin.md) | Load and call plugin methods |
| [03-logging-callbacks.md](./02-kotlin-consumer/03-logging-callbacks.md) | Handle log messages from Rust |
| [04-type-safe-calls.md](./02-kotlin-consumer/04-type-safe-calls.md) | Create type-safe wrapper functions |
| [05-benchmarking.md](./02-kotlin-consumer/05-benchmarking.md) | JMH benchmarking setup |

#### [Chapter 4: Java Consumer](./04-java-consumer/README.md)

Consume plugins from Java using FFM (Foreign Function & Memory API).

| Section | What You'll Learn |
|---------|-------------------|
| [01-project-setup.md](./04-java-consumer/01-project-setup.md) | Set up Java/Gradle project with FFM |
| [02-calling-plugin.md](./04-java-consumer/02-calling-plugin.md) | Load and call plugin methods |
| [03-error-handling.md](./04-java-consumer/03-error-handling.md) | Handle errors from Rust plugins |

### Part 3: Advanced Topics

#### [Chapter 5: Production Bundles](./05-production-bundles/README.md)

Create production-ready bundles with signing, schemas, and compliance features.

| Section | What You'll Learn |
|---------|-------------------|
| [01-code-signing.md](./05-production-bundles/01-code-signing.md) | Generate keys, sign bundles with minisign |
| [02-json-schemas.md](./05-production-bundles/02-json-schemas.md) | Embed schemas for validation |
| [03-build-metadata.md](./05-production-bundles/03-build-metadata.md) | Include provenance and git info |
| [04-sbom.md](./05-production-bundles/04-sbom.md) | Add Software Bill of Materials |

#### [Chapter 6: Cross-Compilation](./06-cross-compilation/README.md)

Build multi-platform bundles for Linux, macOS, and Windows.

| Section | What You'll Learn |
|---------|-------------------|
| [01-platform-overview.md](./06-cross-compilation/01-platform-overview.md) | Platform identifiers and target triples |
| [02-native-toolchains.md](./06-cross-compilation/02-native-toolchains.md) | Build natively on each platform |
| [03-cross-compilation.md](./06-cross-compilation/03-cross-compilation.md) | Cross-compile with `cross` or cargo |

#### [Chapter 7: Backpressure Queues](./07-backpressure-queues/README.md)

Implement bounded queues with backpressure for flow control in C# and Python.

| Section | What You'll Learn |
|---------|-------------------|
| [01-csharp-consumer.md](./07-backpressure-queues/01-csharp-consumer.md) | C# with BlockingCollection and Task |
| [02-python-consumer.md](./07-backpressure-queues/02-python-consumer.md) | Python with queue.Queue and concurrent.futures |

#### [Chapter 8: Binary Transport](./08-binary-transport/README.md)

Build an image thumbnail generator using binary transport for efficient large payload handling.

| Section | What You'll Learn |
|---------|-------------------|
| [01-java-ffm-consumer.md](./08-binary-transport/01-java-ffm-consumer.md) | Java 21+ FFM with StructLayout and VarHandle |
| [02-kotlin-consumer.md](./08-binary-transport/02-kotlin-consumer.md) | Kotlin FFM with extension functions |
| [03-csharp-consumer.md](./08-binary-transport/03-csharp-consumer.md) | C# with StructLayout and Marshal |
| [04-python-consumer.md](./08-binary-transport/04-python-consumer.md) | Python with ctypes.Structure |

## Reference Implementations

Completed examples are available for reference. If you get stuck, compare your code against these working implementations:

- **Hello plugin**: [`examples/hello-plugin/`](../../examples/hello-plugin/)
- **Regex plugin**: [`examples/regex-plugin/`](../../examples/regex-plugin/) *(if available)*
- **JSON plugin**: [`examples/json-plugin/`](../../examples/json-plugin/) *(if available)*

## Choosing Your Path

**New to rustbridge?** Start with the [Getting Started Guide](../GETTING_STARTED.md), then work through Chapters 1-4 in order.

**Building production plugins?** Jump to Chapters 5-6 for bundling and cross-compilation.

**Need high-performance binary I/O?** See Chapter 8 for binary transport.

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/jrobhoward/rustbridge/issues)
- **Discussions**: [GitHub Discussions](https://github.com/jrobhoward/rustbridge/discussions)
