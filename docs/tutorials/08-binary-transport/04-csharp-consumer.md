# Section 4: C# Consumer

In this section, you'll implement binary transport in C# using `StructLayout` and unsafe code for direct memory manipulation. C#'s struct system maps naturally to Rust's `#[repr(C)]` structs.

## Prerequisites

Complete the [project setup](./README.md#project-setup) from the chapter introduction:

1. Scaffold the project with `rustbridge new thumbnail-plugin --all`
2. Replace `src/lib.rs` with the thumbnail plugin implementation
3. Add the `image` dependency to `Cargo.toml`
4. Build the plugin and create the bundle
5. Copy the bundle to `consumers/csharp/`
6. Copy a test image to `consumers/csharp/`

## Verify the Generated Consumer

```bash
cd ~/rustbridge-workspace/thumbnail-plugin/consumers/csharp
dotnet run
```

You should see the basic echo response:

```
Response: Hello from C#!
Length: 14
```

## Define Struct Types

Create `ThumbnailStructs.cs`:

```csharp
using System.Runtime.InteropServices;

namespace ThumbnailDemo;

/// <summary>
/// Message ID for thumbnail creation.
/// </summary>
public static class ThumbnailMessages
{
    public const uint MSG_THUMBNAIL_CREATE = 100;
}

/// <summary>
/// Output format constants.
/// </summary>
public enum OutputFormat : uint
{
    Jpeg = 0,
    Png = 1,
    WebP = 2
}

/// <summary>
/// Request header for thumbnail creation (24 bytes).
///
/// Layout:
///   Offset 0:  version (1 byte)
///   Offset 1:  _reserved (3 bytes)
///   Offset 4:  target_width (4 bytes, u32)
///   Offset 8:  target_height (4 bytes, u32)
///   Offset 12: output_format (4 bytes, u32)
///   Offset 16: quality (4 bytes, u32)
///   Offset 20: payload_size (4 bytes, u32)
///   Total: 24 bytes
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
            _reserved0 = 0,
            _reserved1 = 0,
            _reserved2 = 0,
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
///
/// Layout:
///   Offset 0:  version (1 byte)
///   Offset 1:  _reserved (3 bytes)
///   Offset 4:  width (4 bytes, u32)
///   Offset 8:  height (4 bytes, u32)
///   Offset 12: format (4 bytes, u32)
///   Offset 16: payload_size (4 bytes, u32)
///   Total: 20 bytes
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

    public OutputFormat OutputFormat => (OutputFormat)Format;
}

/// <summary>
/// Parsed thumbnail response with image data.
/// </summary>
public class ThumbnailResponse
{
    public uint Width { get; init; }
    public uint Height { get; init; }
    public OutputFormat Format { get; init; }
    public byte[] ThumbnailData { get; init; } = Array.Empty<byte>();

    public string Dimensions => $"{Width}x{Height}";
    public double SizeKb => ThumbnailData.Length / 1024.0;

    public string FileExtension => Format switch
    {
        OutputFormat.Jpeg => ".jpg",
        OutputFormat.Png => ".png",
        OutputFormat.WebP => ".webp",
        _ => ".bin"
    };
}
```

## Create Helper Methods

Create `ThumbnailHelpers.cs`:

```csharp
using System.Runtime.InteropServices;
using RustBridge;

namespace ThumbnailDemo;

/// <summary>
/// Helper methods for thumbnail binary transport.
/// </summary>
public static class ThumbnailHelpers
{
    /// <summary>
    /// Create a thumbnail request buffer (header + image data).
    /// </summary>
    public static byte[] CreateRequest(
        uint targetWidth,
        uint targetHeight,
        OutputFormat format,
        uint quality,
        byte[] imageData)
    {
        var header = ThumbnailRequestHeader.Create(
            targetWidth,
            targetHeight,
            format,
            quality,
            (uint)imageData.Length
        );

        // Allocate buffer for header + payload
        var request = new byte[ThumbnailRequestHeader.Size + imageData.Length];

        // Copy header to buffer
        unsafe
        {
            fixed (byte* ptr = request)
            {
                Marshal.StructureToPtr(header, (IntPtr)ptr, false);
            }
        }

        // Copy image data after header
        Array.Copy(imageData, 0, request, ThumbnailRequestHeader.Size, imageData.Length);

        return request;
    }

    /// <summary>
    /// Parse a thumbnail response from bytes.
    /// </summary>
    public static ThumbnailResponse ParseResponse(byte[] response)
    {
        if (response.Length < ThumbnailResponseHeader.Size)
        {
            throw new ArgumentException(
                $"Response too small: {response.Length} bytes, need at least {ThumbnailResponseHeader.Size}");
        }

        // Parse header
        ThumbnailResponseHeader header;
        unsafe
        {
            fixed (byte* ptr = response)
            {
                header = Marshal.PtrToStructure<ThumbnailResponseHeader>((IntPtr)ptr);
            }
        }

        // Validate version
        if (header.Version != ThumbnailResponseHeader.CurrentVersion)
        {
            throw new ArgumentException($"Unsupported version: {header.Version}");
        }

        // Validate total size
        var expectedSize = ThumbnailResponseHeader.Size + header.PayloadSize;
        if (response.Length < expectedSize)
        {
            throw new ArgumentException(
                $"Response size mismatch: {response.Length} bytes, expected {expectedSize}");
        }

        // Copy thumbnail data
        var thumbnailData = new byte[header.PayloadSize];
        Array.Copy(response, ThumbnailResponseHeader.Size, thumbnailData, 0, header.PayloadSize);

        return new ThumbnailResponse
        {
            Width = header.Width,
            Height = header.Height,
            Format = header.OutputFormat,
            ThumbnailData = thumbnailData
        };
    }

    /// <summary>
    /// Create a thumbnail using the plugin.
    /// </summary>
    public static ThumbnailResponse CreateThumbnail(
        IPlugin plugin,
        byte[] imageData,
        uint width = 100,
        uint height = 100,
        OutputFormat format = OutputFormat.Jpeg,
        uint quality = 85)
    {
        var request = CreateRequest(width, height, format, quality, imageData);
        var response = plugin.CallRaw(ThumbnailMessages.MSG_THUMBNAIL_CREATE, request);
        return ParseResponse(response);
    }

    /// <summary>
    /// Create a thumbnail and measure processing time.
    /// </summary>
    public static (ThumbnailResponse Response, TimeSpan Elapsed) CreateThumbnailTimed(
        IPlugin plugin,
        byte[] imageData,
        uint width = 100,
        uint height = 100,
        OutputFormat format = OutputFormat.Jpeg,
        uint quality = 85)
    {
        var stopwatch = System.Diagnostics.Stopwatch.StartNew();
        var response = CreateThumbnail(plugin, imageData, width, height, format, quality);
        stopwatch.Stop();
        return (response, stopwatch.Elapsed);
    }
}
```

## Update Program.cs

Replace `Program.cs`:

```csharp
using RustBridge;
using RustBridge.Native;
using ThumbnailDemo;

Console.WriteLine("=== Binary Transport Demo (C#) ===\n");

var bundlePath = "thumbnail-plugin-1.0.0.rbp";
var imagePath = "test-image.jpg";

// Load the test image
if (!File.Exists(imagePath))
{
    Console.Error.WriteLine($"Image not found: {imagePath}");
    Console.Error.WriteLine("Please copy a test image to the current directory.");
    return 1;
}
var imageData = File.ReadAllBytes(imagePath);
Console.WriteLine($"Loaded image: {imagePath} ({imageData.Length} bytes)\n");

using var bundleLoader = BundleLoader.Create()
    .WithBundlePath(bundlePath)
    .WithSignatureVerification(false)
    .Build();
var libraryPath = bundleLoader.ExtractLibrary();

using var plugin = NativePluginLoader.Load(libraryPath);

// Demo 1: Basic thumbnail creation
Console.WriteLine("Demo 1: Create JPEG thumbnail (100x100)");
{
    var (response, elapsed) = ThumbnailHelpers.CreateThumbnailTimed(
        plugin,
        imageData,
        width: 100,
        height: 100,
        format: OutputFormat.Jpeg,
        quality: 85
    );

    Console.WriteLine($"  Thumbnail: {response.Dimensions} {response.Format} ({response.ThumbnailData.Length} bytes)");
    Console.WriteLine($"  Processing time: {elapsed.TotalMilliseconds:F2} ms");

    File.WriteAllBytes("thumbnail-cs-100x100.jpg", response.ThumbnailData);
    Console.WriteLine("  Saved: thumbnail-cs-100x100.jpg");
}

// Demo 2: Proportional sizing
Console.WriteLine("\nDemo 2: Proportional sizing (width=200, height=0)");
{
    var response = ThumbnailHelpers.CreateThumbnail(
        plugin,
        imageData,
        width: 200,
        height: 0,  // Proportional
        format: OutputFormat.Png
    );

    Console.WriteLine($"  Thumbnail: {response.Dimensions} {response.Format} ({response.ThumbnailData.Length} bytes)");

    File.WriteAllBytes("thumbnail-cs-200xN.png", response.ThumbnailData);
    Console.WriteLine("  Saved: thumbnail-cs-200xN.png");
}

// Demo 3: Multiple sizes
Console.WriteLine("\nDemo 3: Multiple sizes");
var sizes = new (uint Width, uint Height)[] { (50, 50), (100, 100), (150, 150), (200, 200) };
foreach (var (w, h) in sizes)
{
    var response = ThumbnailHelpers.CreateThumbnail(plugin, imageData, w, h);
    Console.WriteLine($"  {response.Dimensions}: {response.ThumbnailData.Length} bytes");
}

// Demo 4: Quality comparison
Console.WriteLine("\nDemo 4: Quality comparison (JPEG)");
var qualities = new uint[] { 20, 50, 80, 95 };
foreach (var quality in qualities)
{
    var response = ThumbnailHelpers.CreateThumbnail(
        plugin, imageData, 150, 150, OutputFormat.Jpeg, quality);
    Console.WriteLine($"  Quality {quality}: {response.ThumbnailData.Length} bytes ({response.SizeKb:F1} KB)");

    File.WriteAllBytes($"thumbnail-cs-q{quality}.jpg", response.ThumbnailData);
}

// Demo 5: Performance benchmark
Console.WriteLine("\nDemo 5: Performance benchmark (20 iterations)");
{
    const int iterations = 20;

    // Warm up
    for (int i = 0; i < 3; i++)
    {
        ThumbnailHelpers.CreateThumbnail(plugin, imageData, 100, 100);
    }

    // Measure
    var stopwatch = System.Diagnostics.Stopwatch.StartNew();
    for (int i = 0; i < iterations; i++)
    {
        ThumbnailHelpers.CreateThumbnail(plugin, imageData, 100, 100);
    }
    stopwatch.Stop();

    var avgMs = stopwatch.Elapsed.TotalMilliseconds / iterations;
    Console.WriteLine($"  Total time: {stopwatch.Elapsed.TotalMilliseconds:F0} ms");
    Console.WriteLine($"  Average per thumbnail: {avgMs:F2} ms");
    Console.WriteLine($"  Throughput: {1000.0 / avgMs:F1} thumbnails/sec");
}

// Demo 6: Format comparison
Console.WriteLine("\nDemo 6: Format comparison (150x150)");
foreach (OutputFormat format in Enum.GetValues<OutputFormat>())
{
    var quality = format == OutputFormat.Png ? 0u : 80u;
    var (response, elapsed) = ThumbnailHelpers.CreateThumbnailTimed(
        plugin, imageData, 150, 150, format, quality);
    Console.WriteLine($"  {format}: {response.ThumbnailData.Length} bytes in {elapsed.TotalMilliseconds:F0} ms");
}

// Demo 7: Async processing
Console.WriteLine("\nDemo 7: Parallel thumbnail generation");
{
    var tasks = sizes.Select(async size =>
    {
        return await Task.Run(() =>
        {
            var stopwatch = System.Diagnostics.Stopwatch.StartNew();
            var response = ThumbnailHelpers.CreateThumbnail(plugin, imageData, size.Width, size.Height);
            stopwatch.Stop();
            return (Size: size, Response: response, Elapsed: stopwatch.Elapsed);
        });
    }).ToList();

    var results = await Task.WhenAll(tasks);
    foreach (var result in results.OrderBy(r => r.Size.Width))
    {
        Console.WriteLine($"  {result.Response.Dimensions}: {result.Elapsed.TotalMilliseconds:F0} ms");
    }
}

Console.WriteLine("\n=== Demo Complete ===");
return 0;
```

## Update the Project File

Update the `.csproj` file to enable unsafe code:

```xml
<Project Sdk="Microsoft.NET.Sdk">

  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net8.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <AllowUnsafeBlocks>true</AllowUnsafeBlocks>
  </PropertyGroup>

  <ItemGroup>
    <PackageReference Include="RustBridge.Core" Version="0.7.0" />
    <PackageReference Include="RustBridge.Native" Version="0.7.0" />
  </ItemGroup>

</Project>
```

## Run the Demo

```bash
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

Demo 2: Proportional sizing (width=200, height=0)
  Thumbnail: 200x150 Png (18234 bytes)
  Saved: thumbnail-cs-200xN.png

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
  Jpeg: 5234 bytes in 8 ms
  Png: 12456 bytes in 15 ms
  WebP: 5234 bytes in 8 ms

Demo 7: Parallel thumbnail generation
  50x37: 9 ms
  100x75: 8 ms
  150x112: 10 ms
  200x150: 12 ms

=== Demo Complete ===
```

## Key Observations

### StructLayout for Memory Layout

C#'s `StructLayout` attribute controls struct memory layout:

```csharp
[StructLayout(LayoutKind.Sequential, Pack = 1)]
public struct ThumbnailRequestHeader
{
    public byte Version;
    private byte _reserved0;
    private byte _reserved1;
    private byte _reserved2;
    public uint TargetWidth;
    // ...
}
```

Key points:
- `LayoutKind.Sequential`: Fields are laid out in declaration order
- `Pack = 1`: No padding between fields (matches Rust `#[repr(C)]`)
- Private reserved fields ensure alignment matches Rust

### Marshal for Struct Conversion

Use `Marshal` to convert between structs and byte arrays:

```csharp
// Struct to bytes
unsafe
{
    fixed (byte* ptr = request)
    {
        Marshal.StructureToPtr(header, (IntPtr)ptr, false);
    }
}

// Bytes to struct
ThumbnailResponseHeader header;
unsafe
{
    fixed (byte* ptr = response)
    {
        header = Marshal.PtrToStructure<ThumbnailResponseHeader>((IntPtr)ptr);
    }
}
```

### Unsafe Code Block

Binary transport requires unsafe code for pointer manipulation:

```xml
<AllowUnsafeBlocks>true</AllowUnsafeBlocks>
```

The `fixed` statement pins the array in memory:

```csharp
fixed (byte* ptr = response)
{
    // ptr is valid only within this block
    header = Marshal.PtrToStructure<ThumbnailResponseHeader>((IntPtr)ptr);
}
```

### Memory Safety

Validate all data before processing:

```csharp
// 1. Check minimum size
if (response.Length < ThumbnailResponseHeader.Size)
    throw new ArgumentException("Response too small");

// 2. Parse header
var header = Marshal.PtrToStructure<ThumbnailResponseHeader>(...);

// 3. Validate version
if (header.Version != ThumbnailResponseHeader.CurrentVersion)
    throw new ArgumentException($"Unsupported version: {header.Version}");

// 4. Validate total size
var expectedSize = ThumbnailResponseHeader.Size + header.PayloadSize;
if (response.Length < expectedSize)
    throw new ArgumentException("Response size mismatch");
```

### Alternative: Span&lt;T&gt; Approach

For more idiomatic C#, you can use `Span<T>`:

```csharp
public static ThumbnailResponse ParseResponseSpan(ReadOnlySpan<byte> response)
{
    if (response.Length < ThumbnailResponseHeader.Size)
        throw new ArgumentException("Response too small");

    var header = MemoryMarshal.Read<ThumbnailResponseHeader>(response);

    if (header.Version != ThumbnailResponseHeader.CurrentVersion)
        throw new ArgumentException($"Unsupported version: {header.Version}");

    var thumbnailData = response.Slice(
        ThumbnailResponseHeader.Size,
        (int)header.PayloadSize
    ).ToArray();

    return new ThumbnailResponse
    {
        Width = header.Width,
        Height = header.Height,
        Format = header.OutputFormat,
        ThumbnailData = thumbnailData
    };
}
```

## Error Handling

```csharp
try
{
    var response = ThumbnailHelpers.CreateThumbnail(plugin, imageData, 100, 100);
}
catch (PluginException ex)
{
    Console.Error.WriteLine($"Plugin error (code {ex.ErrorCode}): {ex.Message}");
}
catch (ArgumentException ex)
{
    Console.Error.WriteLine($"Invalid response: {ex.Message}");
}
```

## What's Next?

Continue to the Python implementation, which uses ctypes for struct handling.

[Continue to Section 5: Python Consumer](./05-python-consumer.md)
