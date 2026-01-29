# Tutorial Plugin Template

A cargo-generate template for creating rustbridge plugins with configurable features.

## Usage

```bash
cargo generate --git https://github.com/jrobhoward/rustbridge --path templates/tutorial-plugin
```

Or from a local clone:

```bash
cargo generate --path ~/rustbridge-workspace/rustbridge/templates/tutorial-plugin
```

## Features

When generating a project, you'll be prompted for:

| Feature | Default | Description |
|---------|---------|-------------|
| `include_regex` | true | Regex pattern matching with the `regex` crate |
| `include_cache` | true | LRU cache for compiled patterns |
| `include_config` | true | Plugin configuration via `PluginFactory` |
| `include_logging` | true | Structured logging with `tracing` |

## Generated Project

The generated project includes:

- **Message types**: Either echo or regex matching
- **Plugin struct**: With optional cache storage
- **Configuration**: Optional `PluginConfigData` struct
- **Tests**: Basic test coverage

## Tutorial

This template is used in the [rustbridge tutorials](../../docs/tutorials/README.md) to guide you through building a production-quality plugin step by step.

## License

MIT
