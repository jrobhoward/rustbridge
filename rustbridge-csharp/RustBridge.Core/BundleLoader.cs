using System.IO.Compression;
using System.Runtime.InteropServices;
using System.Security.Cryptography;
using System.Text.Json;

namespace RustBridge;

/// <summary>
/// Loader for RustBridge plugin bundles (.rbp files).
/// <para>
/// Provides functionality to:
/// <list type="bullet">
/// <item>Extract and parse bundle manifests</item>
/// <item>Extract platform-specific libraries</item>
/// <item>Verify SHA256 checksums</item>
/// <item>Verify minisign signatures (optional)</item>
/// </list>
/// </para>
/// <example>
/// <code>
/// // Load with signature verification
/// using var loader = BundleLoader.Create()
///     .WithBundlePath("my-plugin-1.0.0.rbp")
///     .WithSignatureVerification(true)
///     .WithPublicKey("RWS...") // Optional: override manifest key
///     .Build();
///
/// string libPath = loader.ExtractLibrary();
///
/// // Load without signature verification (development only)
/// using var loader = BundleLoader.Create()
///     .WithBundlePath("my-plugin-1.0.0.rbp")
///     .WithSignatureVerification(false)
///     .Build();
/// </code>
/// </example>
/// </summary>
public sealed class BundleLoader : IDisposable
{
    private readonly string _bundlePath;
    private readonly bool _verifySignatures;
    private readonly string? _publicKeyOverride;
    private readonly ZipArchive _zipArchive;
    private readonly FileStream _fileStream;
    private bool _disposed;

    /// <summary>
    /// The bundle manifest.
    /// </summary>
    public BundleManifest Manifest { get; }

    private BundleLoader(Builder builder)
    {
        _bundlePath = builder.BundlePath ?? throw new InvalidOperationException("bundlePath must be set");
        _verifySignatures = builder.VerifySignatures;
        _publicKeyOverride = builder.PublicKeyOverride;

        _fileStream = new FileStream(_bundlePath, FileMode.Open, FileAccess.Read, FileShare.Read);
        _zipArchive = new ZipArchive(_fileStream, ZipArchiveMode.Read);

        try
        {
            Manifest = LoadManifest();

            if (_verifySignatures)
            {
                VerifyManifestSignature();
            }
        }
        catch
        {
            _zipArchive.Dispose();
            _fileStream.Dispose();
            throw;
        }
    }

    /// <summary>
    /// Create a new builder for constructing a BundleLoader.
    /// </summary>
    public static Builder Create() => new();

    /// <summary>
    /// Extract the library for the current platform to a unique temporary directory.
    /// <para>
    /// The library is extracted to a unique subdirectory under the system temp directory,
    /// ensuring no conflicts with other extractions. The caller is responsible for cleaning
    /// up the temporary directory when done.
    /// </para>
    /// </summary>
    /// <returns>Path to the extracted library.</returns>
    /// <exception cref="IOException">If extraction fails.</exception>
    /// <exception cref="CryptographicException">If signature verification fails (when enabled).</exception>
    public string ExtractLibrary()
    {
        // Create unique temp directory under system temp path
        var tempDir = Path.Combine(Path.GetTempPath(), "rustbridge-" + Guid.NewGuid().ToString("N")[..8]);
        Directory.CreateDirectory(tempDir);
        return ExtractLibraryInternal(DetectPlatform(), tempDir, failIfExists: false);
    }

    /// <summary>
    /// Extract the library for the current platform to the specified directory.
    /// <para>
    /// This method will fail if the library file already exists at the target path.
    /// This prevents accidental overwrites and ensures the caller has explicit control
    /// over file lifecycle.
    /// </para>
    /// </summary>
    /// <param name="outputDir">Directory to extract the library to.</param>
    /// <returns>Path to the extracted library.</returns>
    /// <exception cref="IOException">If extraction fails or file already exists.</exception>
    /// <exception cref="CryptographicException">If signature verification fails (when enabled).</exception>
    public string ExtractLibrary(string outputDir)
    {
        var platform = DetectPlatform();
        return ExtractLibrary(platform, outputDir);
    }

    /// <summary>
    /// Extract the library for a specific platform to the specified directory.
    /// <para>
    /// This method will fail if the library file already exists at the target path.
    /// This prevents accidental overwrites and ensures the caller has explicit control
    /// over file lifecycle.
    /// </para>
    /// </summary>
    /// <param name="platform">Platform string (e.g., "linux-x86_64").</param>
    /// <param name="outputDir">Directory to extract the library to.</param>
    /// <returns>Path to the extracted library.</returns>
    /// <exception cref="IOException">If extraction fails or file already exists.</exception>
    /// <exception cref="CryptographicException">If signature verification fails (when enabled).</exception>
    public string ExtractLibrary(string platform, string outputDir)
    {
        return ExtractLibraryInternal(platform, outputDir, failIfExists: true);
    }

    /// <summary>
    /// Internal method to extract the library with configurable overwrite behavior.
    /// </summary>
    private string ExtractLibraryInternal(string platform, string outputDir, bool failIfExists)
    {
        if (Manifest.Platforms == null || !Manifest.Platforms.TryGetValue(platform, out var platformInfo))
        {
            throw new IOException($"Platform not supported: {platform}");
        }

        var libEntry = _zipArchive.GetEntry(platformInfo.Library)
            ?? throw new IOException($"Library not found in bundle: {platformInfo.Library}");

        var libData = ReadZipEntry(libEntry);

        // Verify checksum
        if (!VerifyChecksum(libData, platformInfo.Checksum))
        {
            throw new IOException($"Checksum verification failed for {platformInfo.Library}");
        }

        // Verify signature if enabled
        if (_verifySignatures)
        {
            VerifyLibrarySignature(platformInfo.Library, libData);
        }

        // Determine output path
        var fileName = Path.GetFileName(platformInfo.Library);
        var outputPath = Path.Combine(outputDir, fileName);

        // Check if file already exists when user specifies path
        if (failIfExists && File.Exists(outputPath))
        {
            throw new IOException(
                $"Library already exists at target path: {outputPath}. " +
                "Remove the existing file or use ExtractLibrary() for automatic temp directory.");
        }

        // Ensure output directory exists
        Directory.CreateDirectory(outputDir);

        // Write the library
        File.WriteAllBytes(outputPath, libData);

        // Make executable on Unix
        if (!RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            File.SetUnixFileMode(outputPath,
                UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute |
                UnixFileMode.GroupRead | UnixFileMode.GroupExecute |
                UnixFileMode.OtherRead | UnixFileMode.OtherExecute);
        }

        return outputPath;
    }

    /// <summary>
    /// List all files in the bundle.
    /// </summary>
    public IReadOnlyList<string> ListFiles()
    {
        return _zipArchive.Entries.Select(e => e.FullName).ToList();
    }

    /// <summary>
    /// Get all available schemas in the bundle.
    /// </summary>
    public IReadOnlyDictionary<string, BundleManifest.SchemaInfo> GetSchemas()
    {
        return Manifest.Schemas ?? new Dictionary<string, BundleManifest.SchemaInfo>();
    }

    /// <summary>
    /// Extract a schema file from the bundle.
    /// </summary>
    /// <param name="schemaName">Name of the schema (e.g., "messages.h").</param>
    /// <param name="outputDir">Directory to extract the schema to.</param>
    /// <returns>Path to the extracted schema file.</returns>
    /// <exception cref="IOException">If extraction fails.</exception>
    public string ExtractSchema(string schemaName, string outputDir)
    {
        if (Manifest.Schemas == null || !Manifest.Schemas.TryGetValue(schemaName, out var schemaInfo))
        {
            throw new IOException($"Schema not found in bundle: {schemaName}");
        }

        var schemaEntry = _zipArchive.GetEntry(schemaInfo.Path)
            ?? throw new IOException($"Schema file not found in bundle: {schemaInfo.Path}");

        var schemaData = ReadZipEntry(schemaEntry);

        // Verify checksum
        if (!VerifyChecksum(schemaData, schemaInfo.Checksum))
        {
            throw new IOException($"Checksum verification failed for schema {schemaName}");
        }

        var outputPath = Path.Combine(outputDir, schemaName);
        File.WriteAllBytes(outputPath, schemaData);

        return outputPath;
    }

    /// <summary>
    /// Read a schema file content as string.
    /// </summary>
    /// <param name="schemaName">Name of the schema (e.g., "messages.h").</param>
    /// <returns>Schema file content.</returns>
    /// <exception cref="IOException">If reading fails.</exception>
    public string ReadSchema(string schemaName)
    {
        if (Manifest.Schemas == null || !Manifest.Schemas.TryGetValue(schemaName, out var schemaInfo))
        {
            throw new IOException($"Schema not found in bundle: {schemaName}");
        }

        var schemaEntry = _zipArchive.GetEntry(schemaInfo.Path)
            ?? throw new IOException($"Schema file not found in bundle: {schemaInfo.Path}");

        var schemaData = ReadZipEntry(schemaEntry);

        // Verify checksum
        if (!VerifyChecksum(schemaData, schemaInfo.Checksum))
        {
            throw new IOException($"Checksum verification failed for schema {schemaName}");
        }

        return System.Text.Encoding.UTF8.GetString(schemaData);
    }

    /// <inheritdoc/>
    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;

        _zipArchive.Dispose();
        _fileStream.Dispose();
    }

    private BundleManifest LoadManifest()
    {
        var manifestEntry = _zipArchive.GetEntry("manifest.json")
            ?? throw new IOException("manifest.json not found in bundle");

        var manifestData = ReadZipEntry(manifestEntry);

        var options = new JsonSerializerOptions
        {
            PropertyNameCaseInsensitive = true
        };

        return JsonSerializer.Deserialize<BundleManifest>(manifestData, options)
            ?? throw new IOException("Failed to parse bundle manifest");
    }

    private void VerifyManifestSignature()
    {
        var publicKey = _publicKeyOverride ?? Manifest.PublicKey;

        if (string.IsNullOrEmpty(publicKey))
        {
            throw new IOException(
                "Signature verification enabled but no public key available. " +
                "Bundle must include public_key in manifest, or provide via WithPublicKey() builder method.");
        }

        // Read manifest data
        var manifestEntry = _zipArchive.GetEntry("manifest.json")!;
        var manifestData = ReadZipEntry(manifestEntry);

        // Read signature
        var sigEntry = _zipArchive.GetEntry("manifest.json.minisig")
            ?? throw new IOException(
                "Signature verification enabled but manifest.json.minisig not found in bundle");
        var signature = System.Text.Encoding.UTF8.GetString(ReadZipEntry(sigEntry));

        // Verify
        var verifier = new MinisignVerifier(publicKey);
        if (!verifier.Verify(manifestData, signature))
        {
            throw new IOException("Manifest signature verification failed");
        }
    }

    private void VerifyLibrarySignature(string libraryPath, byte[] libraryData)
    {
        var publicKey = _publicKeyOverride ?? Manifest.PublicKey;

        if (string.IsNullOrEmpty(publicKey))
        {
            throw new IOException("No public key available for signature verification");
        }

        // Read signature
        var sigPath = libraryPath + ".minisig";
        var sigEntry = _zipArchive.GetEntry(sigPath)
            ?? throw new IOException(
                $"Signature verification enabled but {sigPath} not found in bundle");
        var signature = System.Text.Encoding.UTF8.GetString(ReadZipEntry(sigEntry));

        // Verify
        var verifier = new MinisignVerifier(publicKey);
        if (!verifier.Verify(libraryData, signature))
        {
            throw new CryptographicException($"Library signature verification failed: {libraryPath}");
        }
    }

    private static byte[] ReadZipEntry(ZipArchiveEntry entry)
    {
        using var stream = entry.Open();
        using var memoryStream = new MemoryStream();
        stream.CopyTo(memoryStream);
        return memoryStream.ToArray();
    }

    private static bool VerifyChecksum(byte[] data, string expectedChecksum)
    {
        var hash = SHA256.HashData(data);
        var actualChecksum = Convert.ToHexString(hash).ToLowerInvariant();

        // Handle both "sha256:xxx" and raw "xxx" formats
        var expected = expectedChecksum.StartsWith("sha256:", StringComparison.OrdinalIgnoreCase)
            ? expectedChecksum[7..]
            : expectedChecksum;

        return actualChecksum.Equals(expected, StringComparison.OrdinalIgnoreCase);
    }

    private static string DetectPlatform()
    {
        string osName;
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
            osName = "linux";
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.OSX))
            osName = "darwin";
        else if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
            osName = "windows";
        else
            osName = "unknown";

        var arch = RuntimeInformation.OSArchitecture;
        var archName = arch switch
        {
            Architecture.X64 => "x86_64",
            Architecture.Arm64 => "aarch64",
            Architecture.X86 => "x86",
            Architecture.Arm => "arm",
            _ => arch.ToString().ToLowerInvariant()
        };

        return $"{osName}-{archName}";
    }

    /// <summary>
    /// Builder for BundleLoader.
    /// </summary>
    public sealed class Builder
    {
        internal string? BundlePath { get; private set; }
        internal bool VerifySignatures { get; private set; } = true; // Secure by default
        internal string? PublicKeyOverride { get; private set; }

        /// <summary>
        /// Set the path to the bundle file.
        /// </summary>
        public Builder WithBundlePath(string path)
        {
            BundlePath = path;
            return this;
        }

        /// <summary>
        /// Enable or disable signature verification.
        /// <para>
        /// Default: true (verification enabled)
        /// </para>
        /// <para>
        /// <b>WARNING:</b> Disabling signature verification means
        /// the bundle can contain malicious code. Only disable for development/testing.
        /// </para>
        /// </summary>
        public Builder WithSignatureVerification(bool verify)
        {
            VerifySignatures = verify;
            return this;
        }

        /// <summary>
        /// Override the public key from the manifest.
        /// <para>
        /// This allows you to provide a trusted public key instead of using
        /// the key embedded in the manifest. Useful for defense-in-depth.
        /// </para>
        /// </summary>
        /// <param name="publicKey">Minisign public key in base64 format (e.g., "RWS...").</param>
        public Builder WithPublicKey(string? publicKey)
        {
            PublicKeyOverride = publicKey;
            return this;
        }

        /// <summary>
        /// Build the BundleLoader.
        /// </summary>
        /// <exception cref="InvalidOperationException">If bundlePath is not set.</exception>
        /// <exception cref="FileNotFoundException">If the bundle file doesn't exist.</exception>
        /// <exception cref="IOException">If the bundle cannot be opened or manifest is invalid.</exception>
        public BundleLoader Build()
        {
            if (string.IsNullOrEmpty(BundlePath))
            {
                throw new InvalidOperationException("bundlePath must be set");
            }
            if (!File.Exists(BundlePath))
            {
                throw new FileNotFoundException($"Bundle not found: {BundlePath}", BundlePath);
            }
            return new BundleLoader(this);
        }
    }
}
