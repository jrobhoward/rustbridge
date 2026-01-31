# Section 5: Python Consumer

In this section, you'll implement binary transport in Python using `ctypes`.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction.

## Set Up the Python Environment

```powershell
cd $env:USERPROFILE\rustbridge-workspace\thumbnail-plugin\consumers\python
python -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install -r requirements.txt
```

## Install rustbridge Python Package

```powershell
pip install -e $env:USERPROFILE\rustbridge-workspace\rustbridge\rustbridge-python
```

## Define Struct Types

Create `thumbnail_structs.py`:

```python
"""Binary struct definitions for thumbnail plugin using ctypes."""

from ctypes import Structure, c_uint8, c_uint32, sizeof
from dataclasses import dataclass
from enum import IntEnum
from typing import ClassVar


MSG_THUMBNAIL_CREATE: int = 100


class OutputFormat(IntEnum):
    JPEG = 0
    PNG = 1
    WEBP = 2


class ThumbnailRequestHeader(Structure):
    """Request header for thumbnail creation (24 bytes)."""

    _pack_ = 1  # No padding
    _fields_ = [
        ("version", c_uint8),
        ("_reserved0", c_uint8),
        ("_reserved1", c_uint8),
        ("_reserved2", c_uint8),
        ("target_width", c_uint32),
        ("target_height", c_uint32),
        ("output_format", c_uint32),
        ("quality", c_uint32),
        ("payload_size", c_uint32),
    ]

    SIZE: ClassVar[int] = 24
    CURRENT_VERSION: ClassVar[int] = 1

    @classmethod
    def create(
        cls,
        target_width: int,
        target_height: int,
        output_format: OutputFormat,
        quality: int,
        payload_size: int,
    ) -> "ThumbnailRequestHeader":
        header = cls()
        header.version = cls.CURRENT_VERSION
        header.target_width = target_width
        header.target_height = target_height
        header.output_format = output_format.value
        header.quality = quality
        header.payload_size = payload_size
        return header


class ThumbnailResponseHeader(Structure):
    """Response header for thumbnail creation (20 bytes)."""

    _pack_ = 1
    _fields_ = [
        ("version", c_uint8),
        ("_reserved0", c_uint8),
        ("_reserved1", c_uint8),
        ("_reserved2", c_uint8),
        ("width", c_uint32),
        ("height", c_uint32),
        ("format", c_uint32),
        ("payload_size", c_uint32),
    ]

    SIZE: ClassVar[int] = 20
    CURRENT_VERSION: ClassVar[int] = 1


@dataclass
class ThumbnailResponse:
    """Parsed thumbnail response with image data."""
    width: int
    height: int
    format: OutputFormat
    thumbnail_data: bytes

    @property
    def dimensions(self) -> str:
        return f"{self.width}x{self.height}"


# Verify struct sizes at import time
assert sizeof(ThumbnailRequestHeader) == ThumbnailRequestHeader.SIZE
assert sizeof(ThumbnailResponseHeader) == ThumbnailResponseHeader.SIZE
```

## Create Helper Functions

Create `thumbnail_helpers.py` with functions for:
- Creating request buffers
- Parsing response buffers
- Convenience wrappers

See the [Linux tutorial](../tutorials/08-binary-transport/05-python-consumer.md) for the complete implementation.

## Run the Demo

```powershell
python main.py
```

Expected output:

```
=== Binary Transport Demo (Python) ===

Loaded image: test-image.jpg (45678 bytes)

Demo 1: Create JPEG thumbnail (100x100)
  Thumbnail: 100x75 JPEG (2847 bytes)
  Processing time: 12.34 ms
  Saved: thumbnail-py-100x100.jpg
...
```

## Key Observations

### ctypes Structure Definition

```python
class ThumbnailRequestHeader(Structure):
    _pack_ = 1  # No padding, matches #[repr(C)]
    _fields_ = [
        ("version", c_uint8),
        # ...
    ]
```

### Struct Serialization

```python
# Struct to bytes
header = ThumbnailRequestHeader.create(...)
request = bytes(header) + image_data

# Bytes to struct
header = ThumbnailResponseHeader.from_buffer_copy(response[:20])
```

## Summary

You've implemented binary transport across all languages:

| Language | Struct System | Key Classes |
|----------|--------------|-------------|
| C# | StructLayout | Marshal.StructureToPtr |
| Python | ctypes.Structure | from_buffer_copy |

The pattern is consistent:
1. Define struct layouts matching Rust `#[repr(C)]`
2. Create request: header bytes + payload bytes
3. Call `call_raw()` with message ID and request
4. Parse response: read header, validate, extract payload

## Congratulations!

You've completed the Windows tutorials! You now know how to:

- Build rustbridge plugins with various features
- Consume plugins from Kotlin, Java, C#, and Python
- Prepare production bundles with signing and metadata
- Use both JSON and binary transport

For more advanced topics, see the [documentation](../../CLAUDE.md).
