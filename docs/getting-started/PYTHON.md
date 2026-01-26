# Getting Started: Python

This guide walks you through using rustbridge plugins from Python using ctypes.

## Prerequisites

- **Python 3.9 or later** - For type hints and modern features
  ```bash
  python --version  # Should be >= 3.9
  ```
- **A rustbridge plugin** - Either a `.rbp` bundle or native library

## Installation

### Using pip

```bash
pip install rustbridge
```

### From Source

```bash
cd rustbridge-python
pip install -e .
```

### With Development Dependencies

```bash
pip install -e ".[dev]"
```

## Loading a Plugin

### From Bundle (Recommended)

```python
from rustbridge.core import BundleLoader
from rustbridge.native import NativePluginLoader

# Load bundle and extract library for current platform
with BundleLoader("my-plugin-1.0.0.rbp", verify_signatures=False) as bundle:
    library_path = bundle.extract_library()

    # Load the plugin
    with NativePluginLoader.load(library_path) as plugin:
        response = plugin.call("echo", '{"message": "Hello"}')
        print(response)
```

### From Raw Library

```python
from rustbridge.native import NativePluginLoader

# Platform-specific path
plugin_path = "target/release/libmyplugin.so"  # Linux
# plugin_path = "target/release/libmyplugin.dylib"  # macOS
# plugin_path = "target/release/myplugin.dll"  # Windows

with NativePluginLoader.load(plugin_path) as plugin:
    response = plugin.call("echo", '{"message": "Hello"}')
    print(response)
```

## Making JSON Calls

```python
import json
from rustbridge.native import NativePluginLoader

with NativePluginLoader.load(plugin_path) as plugin:
    # Simple call
    response = plugin.call("echo", '{"message": "Hello, World!"}')
    print(response)

    # With Python dicts
    request = {"message": "Hello"}
    response = plugin.call("echo", json.dumps(request))
    result = json.loads(response)

    print(f"Message: {result['message']}")
    print(f"Length: {result['length']}")
```

## Type-Safe Calls with Dataclasses

```python
from dataclasses import dataclass
from typing import TypeVar, Type
import json

T = TypeVar('T')

@dataclass
class EchoRequest:
    message: str

@dataclass
class EchoResponse:
    message: str
    length: int

def call_typed(plugin, type_tag: str, request, response_type: Type[T]) -> T:
    """Make a type-safe plugin call."""
    if hasattr(request, '__dict__'):
        request_json = json.dumps(request.__dict__)
    else:
        request_json = json.dumps(request)

    response_json = plugin.call(type_tag, request_json)
    response_dict = json.loads(response_json)

    return response_type(**response_dict)

# Usage
with NativePluginLoader.load(plugin_path) as plugin:
    request = EchoRequest(message="Hello, Python!")
    response = call_typed(plugin, "echo", request, EchoResponse)

    print(f"Message: {response.message}")
    print(f"Length: {response.length}")
```

### Using Pydantic (Recommended)

```python
from pydantic import BaseModel

class EchoRequest(BaseModel):
    message: str

class EchoResponse(BaseModel):
    message: str
    length: int

def call_typed(plugin, type_tag: str, request: BaseModel, response_type):
    response_json = plugin.call(type_tag, request.model_dump_json())
    return response_type.model_validate_json(response_json)

# Usage
with NativePluginLoader.load(plugin_path) as plugin:
    request = EchoRequest(message="Hello!")
    response = call_typed(plugin, "echo", request, EchoResponse)
    print(response)
```

## Configuration

```python
from rustbridge.core import PluginConfig, LogLevel

config = PluginConfig(
    log_level=LogLevel.DEBUG,
    worker_threads=4,
    max_concurrent_ops=100,
    shutdown_timeout_ms=5000
)

with NativePluginLoader.load(plugin_path, config=config) as plugin:
    # Plugin configured...
    pass
```

## Logging

```python
from rustbridge.core import LogLevel

def log_callback(level: int, target: str, message: str) -> None:
    level_name = LogLevel(level).name
    print(f"[{level_name}] {target}: {message}")

with NativePluginLoader.load(plugin_path, log_callback=log_callback) as plugin:
    plugin.call("echo", '{"message": "test"}')

# Change log level dynamically
plugin.set_log_level(LogLevel.DEBUG)
```

## Binary Transport (Advanced)

For performance-critical paths, use binary transport with ctypes structures.

### Define Structures

```python
from ctypes import Structure, c_uint8, c_uint32, c_char, sizeof

MSG_ECHO = 1

class EchoRequestRaw(Structure):
    _pack_ = 1
    _fields_ = [
        ("version", c_uint8),
        ("_reserved", c_uint8 * 3),
        ("message", c_char * 256),
        ("message_len", c_uint32),
    ]

class EchoResponseRaw(Structure):
    _pack_ = 1
    _fields_ = [
        ("version", c_uint8),
        ("_reserved", c_uint8 * 3),
        ("message", c_char * 256),
        ("message_len", c_uint32),
        ("length", c_uint32),
    ]
```

### Make Binary Calls

```python
# Create request
request = EchoRequestRaw()
request.version = 1
msg = b"Hello"
request.message = msg.ljust(256, b'\x00')
request.message_len = len(msg)

# Call binary transport
response_bytes = plugin.call_raw(MSG_ECHO, request, sizeof(EchoResponseRaw))
response = EchoResponseRaw.from_buffer_copy(response_bytes)

print(f"Length: {response.length}")
```

## Error Handling

```python
from rustbridge.core import PluginException

try:
    response = plugin.call("invalid.type", "{}")
except PluginException as e:
    print(f"Error code: {e.error_code}")
    print(f"Message: {e.message}")

    match e.error_code:
        case 6:
            print("Unknown message type")
        case 7:
            print("Handler error")
        case 13:
            print("Too many concurrent requests")
        case _:
            print("Unexpected error")
```

## Async Usage with asyncio

```python
import asyncio
from concurrent.futures import ThreadPoolExecutor

executor = ThreadPoolExecutor(max_workers=4)

async def call_async(plugin, type_tag: str, request: str) -> str:
    loop = asyncio.get_event_loop()
    return await loop.run_in_executor(
        executor,
        plugin.call,
        type_tag,
        request
    )

# Usage
async def main():
    with NativePluginLoader.load(plugin_path) as plugin:
        response = await call_async(plugin, "echo", '{"message": "Hello"}')
        print(response)

asyncio.run(main())
```

### Concurrent Calls

```python
async def main():
    with NativePluginLoader.load(plugin_path) as plugin:
        tasks = [
            call_async(plugin, "echo", json.dumps({"message": f"Message {i}"}))
            for i in range(100)
        ]
        responses = await asyncio.gather(*tasks)
        print(f"Completed {len(responses)} calls")

asyncio.run(main())
```

## Monitoring

```python
from rustbridge.core import LifecycleState

# Check plugin state
state = plugin.state
print(f"State: {state}")  # LifecycleState.ACTIVE

# Monitor rejected requests
rejected_count = plugin.rejected_request_count
if rejected_count > 0:
    print(f"Rejected: {rejected_count} requests")
```

## Context Manager Pattern

The plugin implements the context manager protocol for automatic cleanup:

```python
# Recommended: use context manager
with NativePluginLoader.load(plugin_path) as plugin:
    plugin.call("echo", '{"message": "Hello"}')
# Plugin automatically shut down here

# Alternative: manual management
plugin = NativePluginLoader.load(plugin_path)
try:
    plugin.call("echo", '{"message": "Hello"}')
finally:
    plugin.shutdown()
```

## Complete Example

```python
import json
from dataclasses import dataclass
from rustbridge.core import PluginConfig, LogLevel
from rustbridge.native import NativePluginLoader

@dataclass
class AddRequest:
    a: int
    b: int

@dataclass
class AddResponse:
    result: int

def main():
    config = PluginConfig(log_level=LogLevel.INFO)

    def log_callback(level: int, target: str, message: str) -> None:
        print(f"[{LogLevel(level).name}] {message}")

    with NativePluginLoader.load(
        "target/release/libcalculator_plugin.so",
        config=config,
        log_callback=log_callback
    ) as plugin:
        # Make typed call
        request = AddRequest(a=42, b=58)
        request_json = json.dumps(request.__dict__)

        response_json = plugin.call("math.add", request_json)
        response_dict = json.loads(response_json)
        response = AddResponse(**response_dict)

        print(f"42 + 58 = {response.result}")

if __name__ == "__main__":
    main()
```

## Performance Notes

Python is the slowest among supported languages due to interpreter overhead:

| Transport | Latency (Linux) | Latency (Windows) |
|-----------|-----------------|-------------------|
| Binary | 4.92 μs | 5.73 μs |
| JSON | 25.5 μs | 26.7 μs |

Binary transport is **5.2x faster** than JSON.

For performance-critical applications, consider:
- Using binary transport for hot paths
- Batching multiple operations
- Using asyncio with ThreadPoolExecutor for concurrency

## Testing

```bash
# Run all tests
python -m pytest tests/ -v

# Run with coverage
python -m pytest tests/ --cov=rustbridge

# Run specific test
python -m pytest tests/test_hello_plugin_integration.py -v
```

## Related Documentation

- [../TRANSPORT.md](../TRANSPORT.md) - Transport layer details
- [../MEMORY_MODEL.md](../MEMORY_MODEL.md) - Memory ownership
- [../TESTING_PYTHON.md](../TESTING_PYTHON.md) - Python testing conventions
