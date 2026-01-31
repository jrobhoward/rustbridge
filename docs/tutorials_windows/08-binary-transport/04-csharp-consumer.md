# Section 4: C# Consumer

In this section, you'll implement binary transport in C# using `StructLayout` and unsafe code.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction.

## Verify the Generated Consumer

```powershell
cd $env:USERPROFILE\rustbridge-workspace\thumbnail-plugin\consumers\csharp
dotnet run
```

## Define Struct Types

Create `ThumbnailStructs.cs`:

```csharp
using System.Runtime.InteropServices;

namespace ThumbnailDemo;

public static class ThumbnailMessages
{
    public const uint MSG_THUMBNAIL_CREATE = 100;
}

public enum OutputFormat : uint
{
    Jpeg = 0,
    Png = 1,
    WebP = 2
}

/// <summary>
/// Request header for thumbnail creation (24 bytes).
/// </summary>
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct ThumbnailRequestHeader
{
    public byte Version;
    private byte _reserved0;
    private byte _reserved1;
    private byte _reserved2;
    public uint TargetWidth;
    public uint TargetHeight;
    public uint OutputFormat;
    public uint Quality;
    public uint PayloadSize;

    public const int Size = 24;
    public const byte CurrentVersion = 1;

    public static ThumbnailRequestHeader Create(
        uint targetWidth,
        uint targetHeight,
        OutputFormat format,
        uint quality,
        uint payloadSize)
    {
        return new ThumbnailRequestHeader
        {
            Version = CurrentVersion,
            TargetWidth = targetWidth,
            TargetHeight = targetHeight,
            OutputFormat = (uint)format,
            Quality = quality,
            PayloadSize = payloadSize
        };
    }
}

/// <summary>
/// Response header for thumbnail creation (20 bytes).
/// </summary>
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct ThumbnailResponseHeader
{
    public byte Version;
    private byte _reserved0;
    private byte _reserved1;
    private byte _reserved2;
    public uint Width;
    public uint Height;
    public uint Format;
    public uint PayloadSize;

    public const int Size = 20;
    public const byte CurrentVersion = 1;
}
```

## Create Helper Methods

Create `ThumbnailHelpers.cs` with methods for:
- Creating request buffers (header + image data)
- Parsing response buffers (header + thumbnail data)
- Calling the plugin with proper error handling

See the [Linux tutorial](../tutorials/08-binary-transport/04-csharp-consumer.md) for the complete implementation.

## Update the Project File

Enable unsafe code in the `.csproj`:

```xml
<AllowUnsafeBlocks>true</AllowUnsafeBlocks>
```

## Run the Demo

```powershell
dotnet run
```

Expected output:

```
=== Binary Transport Demo (C#) ===

Loaded image: test-image.jpg (45678 bytes)

Demo 1: Create JPEG thumbnail (100x100)
  Thumbnail: 100x75 Jpeg (2847 bytes)
  Processing time: 12.34 ms
  Saved: thumbnail-cs-100x100.jpg
...
```

## Key Observations

### StructLayout for Memory Layout

```csharp
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct ThumbnailRequestHeader
{
    // Fields laid out in declaration order, no padding
}
```

### Marshal for Struct Conversion

```csharp
// Struct to bytes
unsafe
{
    fixed (byte* ptr = request)
    {
        Marshal.StructureToPtr(header, (IntPtr)ptr, false);
    }
}
```

## What's Next?

Continue to the Python implementation.

[Continue to Section 5: Python Consumer →](./05-python-consumer.md)
