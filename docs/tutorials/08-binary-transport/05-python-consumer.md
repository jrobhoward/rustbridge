# Section 5: Python Consumer

In this section, you'll implement binary transport in Python using `ctypes` for struct definitions and direct memory manipulation. Python's ctypes module provides C-compatible data types that map directly to Rust's `#[repr(C)]` structs.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new thumbnail-plugin --all`
2. Replace `src/lib.rs` with the thumbnail plugin implementation
3. Add the `image` dependency to `Cargo.toml`
4. Build the plugin and create the bundle
5. Copy the bundle to `consumers/python/`
6. Copy a test image to `consumers/python/`

## Set Up the Python Environment

```bash
cd ~/rustbridge-workspace/thumbnail-plugin/consumers/python
python3 -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate
pip install -r requirements.txt
```

## Install rustbridge Python Package

```bash
pip install -e ~/rustbridge-workspace/rustbridge/rustbridge-python
```

## Verify the Generated Consumer

Copy the bundle and verify it works:

```bash
cp ../../thumbnail-plugin-1.0.0.rbp .
cp ../../consumers/test-image.jpg .
python main.py
```

You should see the basic echo response:

```
Response: Hello from Python!
Length: 18
```

## Define Struct Types

Create `thumbnail_structs.py`:

```python
"""Binary struct definitions for thumbnail plugin using ctypes."""

from __future__ import annotations

from ctypes import Structure, c_uint8, c_uint32, sizeof
from dataclasses import dataclass
from enum import IntEnum
from typing import ClassVar


# Message ID for thumbnail creation
MSG_THUMBNAIL_CREATE: int = 100


class OutputFormat(IntEnum):
    """Output format constants."""

    JPEG = 0
    PNG = 1
    WEBP = 2

    @property
    def extension(self) -> str:
        """Get file extension for this format."""
        return {
            OutputFormat.JPEG: ".jpg",
            OutputFormat.PNG: ".png",
            OutputFormat.WEBP: ".webp",
        }.get(self, ".bin")


class ThumbnailRequestHeader(Structure):
    """
    Request header for thumbnail creation (24 bytes).

    Layout:
      Offset 0:  version (u8)
      Offset 1:  _reserved (3 bytes)
      Offset 4:  target_width (u32, little-endian)
      Offset 8:  target_height (u32, little-endian)
      Offset 12: output_format (u32, little-endian)
      Offset 16: quality (u32, little-endian)
      Offset 20: payload_size (u32, little-endian)
      Total: 24 bytes
    """

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
        """Create a request header with the given parameters."""
        header = cls()
        header.version = cls.CURRENT_VERSION
        header._reserved0 = 0
        header._reserved1 = 0
        header._reserved2 = 0
        header.target_width = target_width
        header.target_height = target_height
        header.output_format = output_format.value
        header.quality = quality
        header.payload_size = payload_size
        return header


class ThumbnailResponseHeader(Structure):
    """
    Response header for thumbnail creation (20 bytes).

    Layout:
      Offset 0:  version (u8)
      Offset 1:  _reserved (3 bytes)
      Offset 4:  width (u32, little-endian)
      Offset 8:  height (u32, little-endian)
      Offset 12: format (u32, little-endian)
      Offset 16: payload_size (u32, little-endian)
      Total: 20 bytes
    """

    _pack_ = 1  # No padding
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

    @property
    def output_format(self) -> OutputFormat:
        """Get the output format as an enum."""
        return OutputFormat(self.format)


@dataclass
class ThumbnailResponse:
    """Parsed thumbnail response with image data."""

    width: int
    height: int
    format: OutputFormat
    thumbnail_data: bytes

    @property
    def dimensions(self) -> str:
        """Get dimensions as string (e.g., '100x75')."""
        return f"{self.width}x{self.height}"

    @property
    def size_kb(self) -> float:
        """Get thumbnail size in KB."""
        return len(self.thumbnail_data) / 1024

    @property
    def file_extension(self) -> str:
        """Get appropriate file extension."""
        return self.format.extension


# Verify struct sizes at import time
assert sizeof(ThumbnailRequestHeader) == ThumbnailRequestHeader.SIZE, (
    f"Request header size mismatch: {sizeof(ThumbnailRequestHeader)} != {ThumbnailRequestHeader.SIZE}"
)
assert sizeof(ThumbnailResponseHeader) == ThumbnailResponseHeader.SIZE, (
    f"Response header size mismatch: {sizeof(ThumbnailResponseHeader)} != {ThumbnailResponseHeader.SIZE}"
)
```

## Create Helper Functions

Create `thumbnail_helpers.py`:

```python
"""Helper functions for thumbnail binary transport."""

from __future__ import annotations

import time
from ctypes import sizeof
from typing import TYPE_CHECKING

from thumbnail_structs import (
    MSG_THUMBNAIL_CREATE,
    OutputFormat,
    ThumbnailRequestHeader,
    ThumbnailResponse,
    ThumbnailResponseHeader,
)

if TYPE_CHECKING:
    from rustbridge.native.native_plugin import NativePlugin


def create_request(
    target_width: int,
    target_height: int,
    output_format: OutputFormat,
    quality: int,
    image_data: bytes,
) -> bytes:
    """
    Create a thumbnail request buffer (header + image data).

    Args:
        target_width: Desired width (0 = proportional to height)
        target_height: Desired height (0 = proportional to width)
        output_format: Output format (JPEG, PNG, or WebP)
        quality: Quality 1-100 (for JPEG/WebP)
        image_data: Raw image bytes

    Returns:
        bytes: Request buffer containing header + image data
    """
    # Create header
    header = ThumbnailRequestHeader.create(
        target_width=target_width,
        target_height=target_height,
        output_format=output_format,
        quality=quality,
        payload_size=len(image_data),
    )

    # Combine header + image data
    return bytes(header) + image_data


def parse_response(response: bytes) -> ThumbnailResponse:
    """
    Parse a thumbnail response from bytes.

    Args:
        response: Response bytes from plugin

    Returns:
        ThumbnailResponse: Parsed response with thumbnail data

    Raises:
        ValueError: If response is invalid
    """
    # Validate minimum size
    if len(response) < ThumbnailResponseHeader.SIZE:
        raise ValueError(
            f"Response too small: {len(response)} bytes, "
            f"need at least {ThumbnailResponseHeader.SIZE}"
        )

    # Parse header
    header = ThumbnailResponseHeader.from_buffer_copy(
        response[: ThumbnailResponseHeader.SIZE]
    )

    # Validate version
    if header.version != ThumbnailResponseHeader.CURRENT_VERSION:
        raise ValueError(f"Unsupported version: {header.version}")

    # Validate total size
    expected_size = ThumbnailResponseHeader.SIZE + header.payload_size
    if len(response) < expected_size:
        raise ValueError(
            f"Response size mismatch: {len(response)} bytes, expected {expected_size}"
        )

    # Extract thumbnail data
    thumbnail_data = response[
        ThumbnailResponseHeader.SIZE : ThumbnailResponseHeader.SIZE + header.payload_size
    ]

    return ThumbnailResponse(
        width=header.width,
        height=header.height,
        format=header.output_format,
        thumbnail_data=thumbnail_data,
    )


def create_thumbnail(
    plugin: "NativePlugin",
    image_data: bytes,
    width: int = 100,
    height: int = 100,
    output_format: OutputFormat = OutputFormat.JPEG,
    quality: int = 85,
) -> ThumbnailResponse:
    """
    Create a thumbnail using the plugin.

    Args:
        plugin: The loaded native plugin
        image_data: Raw image bytes
        width: Target width (0 = proportional)
        height: Target height (0 = proportional)
        output_format: Output format
        quality: Quality 1-100

    Returns:
        ThumbnailResponse: The generated thumbnail
    """
    request = create_request(width, height, output_format, quality, image_data)
    response = plugin.call_raw(MSG_THUMBNAIL_CREATE, request)
    return parse_response(response)


def create_thumbnail_timed(
    plugin: "NativePlugin",
    image_data: bytes,
    width: int = 100,
    height: int = 100,
    output_format: OutputFormat = OutputFormat.JPEG,
    quality: int = 85,
) -> tuple[ThumbnailResponse, float]:
    """
    Create a thumbnail and measure processing time.

    Returns:
        tuple: (ThumbnailResponse, elapsed_seconds)
    """
    start = time.perf_counter()
    response = create_thumbnail(plugin, image_data, width, height, output_format, quality)
    elapsed = time.perf_counter() - start
    return response, elapsed


def create_thumbnail_sizes(
    plugin: "NativePlugin",
    image_data: bytes,
    sizes: list[tuple[int, int]],
    output_format: OutputFormat = OutputFormat.JPEG,
    quality: int = 85,
) -> list[ThumbnailResponse]:
    """
    Create thumbnails at multiple sizes.

    Args:
        plugin: The loaded native plugin
        image_data: Raw image bytes
        sizes: List of (width, height) tuples
        output_format: Output format
        quality: Quality 1-100

    Returns:
        list: List of ThumbnailResponse objects
    """
    return [
        create_thumbnail(plugin, image_data, w, h, output_format, quality)
        for w, h in sizes
    ]


def create_thumbnail_qualities(
    plugin: "NativePlugin",
    image_data: bytes,
    width: int,
    height: int,
    qualities: list[int],
) -> list[tuple[int, ThumbnailResponse]]:
    """
    Create thumbnails at multiple quality levels.

    Returns:
        list: List of (quality, ThumbnailResponse) tuples
    """
    return [
        (q, create_thumbnail(plugin, image_data, width, height, OutputFormat.JPEG, q))
        for q in qualities
    ]
```

## Update main.py

Replace `main.py`:

```python
#!/usr/bin/env python3
"""Binary transport demo for thumbnail plugin."""

from __future__ import annotations

import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from rustbridge.core import BundleLoader

from thumbnail_helpers import (
    create_thumbnail,
    create_thumbnail_qualities,
    create_thumbnail_sizes,
    create_thumbnail_timed,
)
from thumbnail_structs import OutputFormat


def main() -> None:
    print("=== Binary Transport Demo (Python) ===\n")

    bundle_path = "thumbnail-plugin-1.0.0.rbp"
    image_path = Path("test-image.jpg")

    # Load the test image
    if not image_path.exists():
        print(f"Error: Image not found: {image_path}")
        print("Please copy a test image to the current directory.")
        return

    image_data = image_path.read_bytes()
    print(f"Loaded image: {image_path} ({len(image_data)} bytes)\n")

    # Load the plugin from bundle
    loader = BundleLoader(verify_signatures=False)
    with loader.load(bundle_path) as plugin:

        # Demo 1: Basic thumbnail creation
        print("Demo 1: Create JPEG thumbnail (100x100)")
        response, elapsed = create_thumbnail_timed(
            plugin,
            image_data,
            width=100,
            height=100,
            output_format=OutputFormat.JPEG,
            quality=85,
        )
        print(f"  Thumbnail: {response.dimensions} {response.format.name} ({len(response.thumbnail_data)} bytes)")
        print(f"  Processing time: {elapsed * 1000:.2f} ms")

        Path("thumbnail-py-100x100.jpg").write_bytes(response.thumbnail_data)
        print("  Saved: thumbnail-py-100x100.jpg")

        # Demo 2: Proportional sizing
        print("\nDemo 2: Proportional sizing (width=200, height=0)")
        response = create_thumbnail(
            plugin,
            image_data,
            width=200,
            height=0,  # Proportional
            output_format=OutputFormat.PNG,
        )
        print(f"  Thumbnail: {response.dimensions} {response.format.name} ({len(response.thumbnail_data)} bytes)")

        Path("thumbnail-py-200xN.png").write_bytes(response.thumbnail_data)
        print("  Saved: thumbnail-py-200xN.png")

        # Demo 3: Multiple sizes
        print("\nDemo 3: Multiple sizes")
        sizes = [(50, 50), (100, 100), (150, 150), (200, 200)]
        thumbnails = create_thumbnail_sizes(plugin, image_data, sizes)
        for thumb in thumbnails:
            print(f"  {thumb.dimensions}: {len(thumb.thumbnail_data)} bytes")

        # Demo 4: Quality comparison
        print("\nDemo 4: Quality comparison (JPEG)")
        qualities = [20, 50, 80, 95]
        quality_results = create_thumbnail_qualities(plugin, image_data, 150, 150, qualities)
        for quality, thumb in quality_results:
            print(f"  Quality {quality}: {len(thumb.thumbnail_data)} bytes ({thumb.size_kb:.1f} KB)")
            Path(f"thumbnail-py-q{quality}.jpg").write_bytes(thumb.thumbnail_data)

        # Demo 5: Performance benchmark
        print("\nDemo 5: Performance benchmark (20 iterations)")
        iterations = 20

        # Warm up
        for _ in range(3):
            create_thumbnail(plugin, image_data, 100, 100)

        # Measure
        start = time.perf_counter()
        for _ in range(iterations):
            create_thumbnail(plugin, image_data, 100, 100)
        total_time = time.perf_counter() - start

        avg_ms = (total_time / iterations) * 1000
        print(f"  Total time: {total_time * 1000:.0f} ms")
        print(f"  Average per thumbnail: {avg_ms:.2f} ms")
        print(f"  Throughput: {1000 / avg_ms:.1f} thumbnails/sec")

        # Demo 6: Format comparison
        print("\nDemo 6: Format comparison (150x150)")
        for fmt in OutputFormat:
            quality = 0 if fmt == OutputFormat.PNG else 80
            thumb, elapsed = create_thumbnail_timed(
                plugin, image_data, 150, 150, fmt, quality
            )
            print(f"  {fmt.name}: {len(thumb.thumbnail_data)} bytes in {elapsed * 1000:.0f} ms")

        # Demo 7: Parallel processing with ThreadPoolExecutor
        print("\nDemo 7: Parallel thumbnail generation")
        with ThreadPoolExecutor(max_workers=4) as executor:
            futures = {
                executor.submit(
                    create_thumbnail_timed, plugin, image_data, w, h
                ): (w, h)
                for w, h in sizes
            }

            results = []
            for future in as_completed(futures):
                size = futures[future]
                thumb, elapsed = future.result()
                results.append((size, thumb, elapsed))

            # Sort by size and print
            for (w, h), thumb, elapsed in sorted(results, key=lambda x: x[0][0]):
                print(f"  {thumb.dimensions}: {elapsed * 1000:.0f} ms")

        # Demo 8: Struct size verification
        print("\nDemo 8: Struct size verification")
        from thumbnail_structs import ThumbnailRequestHeader, ThumbnailResponseHeader
        from ctypes import sizeof
        print(f"  Request header size: {sizeof(ThumbnailRequestHeader)} bytes (expected 24)")
        print(f"  Response header size: {sizeof(ThumbnailResponseHeader)} bytes (expected 20)")

    print("\n=== Demo Complete ===")


if __name__ == "__main__":
    main()
```

## Run the Demo

```bash
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

Demo 2: Proportional sizing (width=200, height=0)
  Thumbnail: 200x150 PNG (18234 bytes)
  Saved: thumbnail-py-200xN.png

Demo 3: Multiple sizes
  50x37: 987 bytes
  100x75: 2847 bytes
  150x112: 5234 bytes
  200x150: 8456 bytes

Demo 4: Quality comparison (JPEG)
  Quality 20: 1234 bytes (1.2 KB)
  Quality 50: 2567 bytes (2.5 KB)
  Quality 80: 4123 bytes (4.0 KB)
  Quality 95: 7890 bytes (7.7 KB)

Demo 5: Performance benchmark (20 iterations)
  Total time: 168 ms
  Average per thumbnail: 8.40 ms
  Throughput: 119.0 thumbnails/sec

Demo 6: Format comparison (150x150)
  JPEG: 5234 bytes in 8 ms
  PNG: 12456 bytes in 15 ms
  WEBP: 5234 bytes in 8 ms

Demo 7: Parallel thumbnail generation
  50x37: 9 ms
  100x75: 8 ms
  150x112: 10 ms
  200x150: 12 ms

Demo 8: Struct size verification
  Request header size: 24 bytes (expected 24)
  Response header size: 20 bytes (expected 20)

=== Demo Complete ===
```

## Key Observations

### ctypes Structure Definition

Python's `ctypes.Structure` maps directly to C/Rust structs:

```python
class ThumbnailRequestHeader(Structure):
    _pack_ = 1  # No padding, matches #[repr(C)]
    _fields_ = [
        ("version", c_uint8),
        ("_reserved0", c_uint8),
        ("_reserved1", c_uint8),
        ("_reserved2", c_uint8),
        ("target_width", c_uint32),
        # ...
    ]
```

Key points:
- `_pack_ = 1` disables padding (like `#[repr(C, packed)]`)
- Field types match Rust: `c_uint8` = `u8`, `c_uint32` = `u32`
- Use `sizeof()` to verify struct size

### Struct Serialization

Convert structs to/from bytes:

```python
# Struct to bytes
header = ThumbnailRequestHeader.create(...)
request = bytes(header) + image_data

# Bytes to struct
header = ThumbnailResponseHeader.from_buffer_copy(response[:20])
```

### Byte Order

Python's ctypes uses native byte order by default, which matches Rust on x86/ARM (little-endian). For explicit control:

```python
# For big-endian systems, you might need:
from ctypes import BigEndianStructure

class MyStructBE(BigEndianStructure):
    # ...
```

### Verification at Import Time

Verify struct sizes when the module loads:

```python
assert sizeof(ThumbnailRequestHeader) == ThumbnailRequestHeader.SIZE, (
    f"Request header size mismatch: {sizeof(ThumbnailRequestHeader)} != {ThumbnailRequestHeader.SIZE}"
)
```

This catches layout errors immediately rather than at runtime.

### Dataclass for Response

Use `@dataclass` for cleaner response handling:

```python
@dataclass
class ThumbnailResponse:
    width: int
    height: int
    format: OutputFormat
    thumbnail_data: bytes

    @property
    def dimensions(self) -> str:
        return f"{self.width}x{self.height}"
```

### IntEnum for Type Safety

Use `IntEnum` for constants with both numeric and named access:

```python
class OutputFormat(IntEnum):
    JPEG = 0
    PNG = 1
    WEBP = 2

# Usage:
format = OutputFormat.JPEG
print(format.value)  # 0
print(format.name)   # "JPEG"
```

## Error Handling

```python
from rustbridge.core.plugin_exception import PluginException

try:
    response = create_thumbnail(plugin, image_data, 100, 100)
except PluginException as e:
    print(f"Plugin error (code {e.error_code}): {e}")
except ValueError as e:
    print(f"Invalid response: {e}")
```

## Summary

You've now implemented binary transport in all five languages:

| Language | Struct System | Memory Management | Key Classes |
|----------|--------------|-------------------|-------------|
| Java FFM | StructLayout | Arena, freeBuffer() | MemorySegment, VarHandle |
| Java JNI | ByteBuffer | Automatic (JNI copies) | ByteBuffer, ByteOrder |
| Kotlin | StructLayout | Arena, use{} | MemorySegment, extension functions |
| C# | StructLayout | Marshal, fixed | Marshal.StructureToPtr |
| Python | ctypes.Structure | Automatic (bytes copy) | Structure, from_buffer_copy |

The pattern is consistent across languages:
1. Define struct layouts matching Rust `#[repr(C)]`
2. Create request: header bytes + payload bytes
3. Call `call_raw()` with message ID and request
4. Parse response: read header, validate, extract payload
5. Free native memory if required (FFM, Kotlin)

## Next Steps

You now have production-ready patterns for binary transport in all supported languages. Consider:

- Adding compression for large images before transport
- Implementing batch operations for multiple thumbnails
- Adding progress callbacks for large files
- Creating a unified API wrapper in each language
