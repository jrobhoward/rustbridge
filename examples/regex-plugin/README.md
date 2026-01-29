# Regex Plugin Example

A complete rustbridge plugin demonstrating regex pattern matching with LRU caching, configuration, and structured logging.

This example serves as the reference implementation for the [tutorials](../../docs/tutorials/README.md).

## Features

- **Regex Pattern Matching**: Test text against regex patterns
- **LRU Cache**: Compiled patterns are cached to avoid recompilation
- **Configuration**: Cache size is configurable via plugin config
- **Structured Logging**: Uses tracing for debug and info level logs

## Quick Start

```bash
# Build the plugin
cargo build --release

# Run tests
cargo test

# Run the cache effectiveness benchmark
cargo test bench___cache_effectiveness -- --nocapture
```

## Message Types

### match

Test if a regex pattern matches some text.

**Request:**
```json
{
  "pattern": "\\d+",
  "text": "test123"
}
```

**Response:**
```json
{
  "matches": true,
  "cached": false
}
```

## Configuration

The plugin accepts configuration via `config.data`:

```json
{
  "cache_size": 100
}
```

- `cache_size`: Maximum number of compiled regex patterns to keep in the LRU cache (default: 100)

## Creating a Bundle

```bash
# Linux
rustbridge bundle create \
  --name regex-plugin \
  --version 0.6.0 \
  --lib linux-x86_64:target/release/libregex_plugin.so \
  --output regex-plugin-0.6.0.rbp

# macOS (Apple Silicon)
rustbridge bundle create \
  --name regex-plugin \
  --version 0.6.0 \
  --lib darwin-aarch64:target/release/libregex_plugin.dylib \
  --output regex-plugin-0.6.0.rbp

# Windows
rustbridge bundle create \
  --name regex-plugin \
  --version 0.6.0 \
  --lib windows-x86_64:target/release/regex_plugin.dll \
  --output regex-plugin-0.6.0.rbp
```

## Using from Kotlin

```kotlin
data class MatchRequest(val pattern: String, val text: String)
data class MatchResponse(val matches: Boolean, val cached: Boolean)

// Using the type-safe extension
val response = plugin.callTyped<MatchResponse>(
    "match",
    MatchRequest("\\d+", "test123")
)

println("Matches: ${response.matches}")
println("From cache: ${response.cached}")
```

## License

MIT
