# rustbridge C# Consumer Template

A minimal .NET project template for consuming rustbridge plugins.

## Prerequisites

- **.NET 6.0+** - SDK and runtime
- **A rustbridge plugin** - Your `.rbp` bundle file

## Quick Start

1. **Copy this template** to your project location
2. **Build rustbridge C# libraries**:
   ```bash
   cd /path/to/rustbridge/rustbridge-csharp
   dotnet build
   ```
3. **Update project reference** in `RustBridgeConsumer.csproj` to point to your rustbridge location
4. **Add your plugin bundle** - Copy your `.rbp` file to the project root
5. **Update Program.cs** - Set `bundlePath` to your `.rbp` file
6. **Run**:
   ```bash
   dotnet run
   ```

## Using NuGet Packages

When rustbridge is published to NuGet, replace the `<ProjectReference>` entries with:

```xml
<ItemGroup>
  <PackageReference Include="RustBridge.Core" Version="0.1.0" />
  <PackageReference Include="RustBridge.Native" Version="0.1.0" />
</ItemGroup>
```

## Documentation

- [rustbridge Documentation](https://github.com/jrobhoward/rustbridge)
- [C# Guide](https://github.com/jrobhoward/rustbridge/blob/main/docs/using-plugins/CSHARP.md)
