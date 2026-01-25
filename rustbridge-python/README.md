# rustbridge-python

Python bindings for rustbridge - call Rust shared libraries from Python using ctypes.

## Installation

```bash
pip install .

# Or for development
pip install -e ".[dev]"
```

## Quick Start

### Direct Library Loading

```python
from rustbridge import NativePluginLoader, PluginConfig, LogLevel

# Simple loading
with NativePluginLoader.load("libmyplugin.so") as plugin:
    response = plugin.call("echo", '{"message": "hello"}')
    print(response)

# With configuration
config = PluginConfig.defaults().log_level(LogLevel.DEBUG)
with NativePluginLoader.load_with_config("libmyplugin.so", config) as plugin:
    response = plugin.call("echo", '{"message": "hello"}')
```

### Bundle Loading (with signature verification)

```python
from rustbridge import BundleLoader, PluginConfig

# Load with signature verification (default)
loader = BundleLoader(verify_signatures=True)
with loader.load("my-plugin-1.0.0.rbp") as plugin:
    response = plugin.call("echo", '{"message": "hello"}')

# Skip verification for development
loader = BundleLoader(verify_signatures=False)
with loader.load("my-plugin-1.0.0.rbp") as plugin:
    ...
```

### With Log Callback

```python
from rustbridge import NativePluginLoader, PluginConfig, LogLevel

def on_log(level: LogLevel, target: str, message: str):
    print(f"[{level.name}] {target}: {message}")

config = PluginConfig.defaults().log_level(LogLevel.DEBUG)
with NativePluginLoader.load_with_config("libmyplugin.so", config, log_callback=on_log) as plugin:
    response = plugin.call("echo", '{"message": "hello"}')
```

## Configuration

```python
from rustbridge import PluginConfig, LogLevel

config = (PluginConfig.defaults()
    .log_level(LogLevel.DEBUG)      # Set log level
    .worker_threads(4)               # Set worker threads
    .max_concurrent_ops(500)         # Max concurrent operations
    .shutdown_timeout_ms(10000)      # Shutdown timeout
    .set("custom_key", "value")      # Custom configuration
    .init_param("db_url", "..."))    # Initialization parameter
```

## API Reference

### Core Types

- `LogLevel` - Log level enum (TRACE, DEBUG, INFO, WARN, ERROR, OFF)
- `LifecycleState` - Plugin lifecycle state (INSTALLED, STARTING, ACTIVE, STOPPING, STOPPED, FAILED)
- `PluginConfig` - Configuration builder
- `PluginException` - Exception with error code
- `ResponseEnvelope` - JSON response wrapper

### Native Plugin

- `NativePluginLoader.load(path)` - Load a plugin
- `NativePluginLoader.load_with_config(path, config, callback)` - Load with configuration
- `plugin.call(type_tag, request)` - Make a JSON call
- `plugin.call_typed(type_tag, request)` - Make a typed call (auto JSON serialization)
- `plugin.state` - Get lifecycle state
- `plugin.set_log_level(level)` - Set log level
- `plugin.shutdown()` - Shutdown the plugin

### Bundle Loading

- `BundleLoader(verify_signatures=True)` - Create a loader
- `loader.load(path)` - Load plugin from bundle
- `loader.load_with_config(path, config, callback)` - Load with configuration
- `loader.get_manifest(path)` - Read bundle manifest
- `BundleLoader.get_current_platform()` - Get current platform string

## Development

```bash
# Install development dependencies
pip install -e ".[dev]"

# Run tests
python -m pytest tests/ -v

# Run specific test
python -m pytest tests/test_log_level.py -v

# Build hello-plugin for integration tests
cargo build -p hello-plugin --release
```

## Requirements

- Python 3.10+
- PyNaCl (for Ed25519 signature verification)

## License

MIT
