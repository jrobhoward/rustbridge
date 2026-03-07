# rustbridge C# Consumer Template

A minimal .NET project template for consuming rustbridge plugins.

## Prerequisites

- **.NET 8.0+** - SDK and runtime
- **A rustbridge plugin** - Your `.rbp` bundle file

## Quick Start

1. **Copy this template** to your project location
2. **Add your plugin bundle** - Copy your `.rbp` file to the project root
3. **Update Program.cs** - Set `bundlePath` to your `.rbp` file
4. **Run**:
   ```bash
   dotnet run
   ```

Dependencies are resolved automatically from NuGet.

## Local Development

For building against rustbridge source instead of NuGet packages, see the
[Development Guide](https://github.com/jrobhoward/rustbridge/blob/main/docs/DEVELOPMENT.md#c).

## Documentation

- [rustbridge Documentation](https://github.com/jrobhoward/rustbridge)
- [C# Guide](https://github.com/jrobhoward/rustbridge/blob/main/docs/using-plugins/CSHARP.md)
