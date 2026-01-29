# Tutorial Plugin Template

A cargo-generate template that creates a complete regex-matching plugin with LRU caching, configuration, and logging.

This is the "finished product" from the [rustbridge tutorials](../../docs/tutorials/README.md).

## Usage

```bash
cargo generate --git https://github.com/jrobhoward/rustbridge templates/tutorial-plugin
```

Or from a local clone:

```bash
cargo generate --path ~/rustbridge-workspace/rustbridge/templates/tutorial-plugin
```

## After Generating

```bash
cd your-plugin-name
cargo fmt
cargo test
cargo build --release
```

## What's Included

The generated plugin includes:

- **Regex pattern matching** with the `regex` crate
- **LRU cache** for compiled patterns (configurable size)
- **Configuration support** via `PluginFactory`
- **Structured logging** with `tracing`
- **Tests** for basic functionality

## Starting Simpler?

If you want to start with a minimal echo plugin and build up incrementally (as the tutorial does), use the basic template instead:

```bash
cp -r ~/rustbridge-workspace/rustbridge/templates/plugin ~/my-plugin
```

Then follow the [tutorials](../../docs/tutorials/README.md) to add features step by step.

## License

MIT
