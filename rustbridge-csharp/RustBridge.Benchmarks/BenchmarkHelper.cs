namespace RustBridge.Benchmarks;

/// <summary>
/// Helper class for finding the hello-plugin library.
/// </summary>
public static class BenchmarkHelper
{
    /// <summary>
    /// Find the hello-plugin library, searching in common locations.
    /// </summary>
    /// <returns>Full path to the library, or null if not found.</returns>
    public static string? FindHelloPlugin()
    {
        var libraryName = GetLibraryFileName("hello_plugin");
        var assemblyLocation = typeof(BenchmarkHelper).Assembly.Location;
        var assemblyDir = Path.GetDirectoryName(assemblyLocation) ?? ".";

        var searchBases = new[]
        {
            // From current directory
            Environment.CurrentDirectory,
            // From assembly location
            assemblyDir,
            // Walk up from assembly to find repo
            Path.Combine(assemblyDir, "..", "..", "..", ".."),
            Path.Combine(assemblyDir, "..", "..", "..", "..", ".."),
            Path.Combine(assemblyDir, "..", "..", "..", "..", "..", ".."),
        };

        foreach (var baseDir in searchBases)
        {
            // Prefer release build for benchmarks
            var releasePath = Path.Combine(baseDir, "target", "release", libraryName);
            if (File.Exists(releasePath))
            {
                return Path.GetFullPath(releasePath);
            }

            var debugPath = Path.Combine(baseDir, "target", "debug", libraryName);
            if (File.Exists(debugPath))
            {
                return Path.GetFullPath(debugPath);
            }
        }

        return null;
    }

    /// <summary>
    /// Get the hello-plugin library path or throw if not found.
    /// </summary>
    public static string GetHelloPluginOrThrow()
    {
        return FindHelloPlugin()
            ?? throw new InvalidOperationException(
                "hello-plugin not found. Run: cargo build --release -p hello-plugin");
    }

    private static string GetLibraryFileName(string name)
    {
        if (OperatingSystem.IsWindows()) return $"{name}.dll";
        if (OperatingSystem.IsMacOS()) return $"lib{name}.dylib";
        return $"lib{name}.so";
    }
}
