using System.IO.Compression;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;

namespace RustBridge.Tests;

/// <summary>
/// Tests for <see cref="BundleLoader"/>.
/// </summary>
public class BundleLoaderTests : IDisposable
{
    private readonly string _tempDir;

    public BundleLoaderTests()
    {
        _tempDir = Path.Combine(Path.GetTempPath(), $"rustbridge-test-{Guid.NewGuid():N}");
        Directory.CreateDirectory(_tempDir);
    }

    public void Dispose()
    {
        if (Directory.Exists(_tempDir))
        {
            Directory.Delete(_tempDir, recursive: true);
        }
    }

    [Fact]
    public void Builder___NoBundlePath___ThrowsInvalidOperationException()
    {
        var exception = Assert.Throws<InvalidOperationException>(() =>
            BundleLoader.Create().Build());

        Assert.Contains("bundlePath", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void Builder___NonexistentPath___ThrowsFileNotFoundException()
    {
        Assert.Throws<FileNotFoundException>(() =>
            BundleLoader.Create()
                .WithBundlePath("/nonexistent/path/bundle.rbp")
                .Build());
    }

    [Fact]
    public void Builder___PathAsString___Works()
    {
        var bundlePath = CreateMinimalBundle();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        Assert.NotNull(loader);
        Assert.NotNull(loader.Manifest);
    }

    [Fact]
    public void Builder___VerifySignaturesDefault___IsTrue()
    {
        var bundlePath = CreateMinimalBundle();

        var exception = Assert.Throws<IOException>(() =>
            BundleLoader.Create()
                .WithBundlePath(bundlePath)
                .Build());

        Assert.True(
            exception.Message.Contains("public key", StringComparison.OrdinalIgnoreCase) ||
            exception.Message.Contains("signature", StringComparison.OrdinalIgnoreCase));
    }

    [Fact]
    public void GetManifest___ReturnsParsedManifest()
    {
        var bundlePath = CreateMinimalBundle();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var manifest = loader.Manifest;

        Assert.NotNull(manifest);
        Assert.Equal("1.0", manifest.BundleVersion);
        Assert.NotNull(manifest.Plugin);
        Assert.Equal("test-plugin", manifest.Plugin.Name);
        Assert.Equal("1.0.0", manifest.Plugin.Version);
    }

    [Fact]
    public void ListFiles___ReturnsAllFiles()
    {
        var bundlePath = CreateMinimalBundle();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var files = loader.ListFiles();

        Assert.Contains("manifest.json", files);
    }

    [Fact]
    public void ExtractLibrary___UnsupportedPlatform___ThrowsIOException()
    {
        var bundlePath = CreateMinimalBundle();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var exception = Assert.Throws<IOException>(() =>
            loader.ExtractLibrary("unknown-platform", _tempDir));

        Assert.Contains("not supported", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ExtractLibrary___MissingLibraryFile___ThrowsIOException()
    {
        var bundlePath = CreateBundleWithPlatformButNoLibrary();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var exception = Assert.Throws<IOException>(() =>
            loader.ExtractLibrary("linux-x86_64", _tempDir));

        Assert.Contains("not found", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ExtractLibrary___ChecksumMismatch___ThrowsIOException()
    {
        var bundlePath = CreateBundleWithBadChecksum();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var exception = Assert.Throws<IOException>(() =>
            loader.ExtractLibrary("linux-x86_64", _tempDir));

        Assert.Contains("Checksum", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ExtractLibrary___ValidChecksum___Succeeds()
    {
        var bundlePath = CreateBundleWithValidLibrary();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var extractedLib = loader.ExtractLibrary("linux-x86_64", _tempDir);

        Assert.True(File.Exists(extractedLib));
        Assert.Contains("libtest.so", extractedLib);
    }

    [Fact]
    public void ExtractLibrary___ChecksumWithoutPrefix___Succeeds()
    {
        var bundlePath = CreateBundleWithValidLibraryNoChecksumPrefix();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var extractedLib = loader.ExtractLibrary("linux-x86_64", _tempDir);

        Assert.True(File.Exists(extractedLib));
    }

    [Fact]
    public void GetSchemas___NoSchemas___ReturnsEmptyDictionary()
    {
        var bundlePath = CreateMinimalBundle();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var schemas = loader.GetSchemas();

        Assert.Empty(schemas);
    }

    [Fact]
    public void ExtractSchema___MissingSchema___ThrowsIOException()
    {
        var bundlePath = CreateMinimalBundle();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var exception = Assert.Throws<IOException>(() =>
            loader.ExtractSchema("nonexistent.h", _tempDir));

        Assert.Contains("not found", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    [Fact]
    public void ExtractSchema___ValidSchema___Succeeds()
    {
        var bundlePath = CreateBundleWithSchema();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var schemaPath = loader.ExtractSchema("messages.h", _tempDir);

        Assert.True(File.Exists(schemaPath));
        var content = File.ReadAllText(schemaPath);
        Assert.Contains("typedef struct", content);
    }

    [Fact]
    public void ReadSchema___ValidSchema___ReturnsContent()
    {
        var bundlePath = CreateBundleWithSchema();

        using var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var content = loader.ReadSchema("messages.h");

        Assert.Contains("typedef struct", content);
    }

    [Fact]
    public void Dispose___CanBeCalledMultipleTimes()
    {
        var bundlePath = CreateMinimalBundle();

        var loader = BundleLoader.Create()
            .WithBundlePath(bundlePath)
            .WithSignatureVerification(false)
            .Build();

        var exception = Record.Exception(() =>
        {
            loader.Dispose();
            loader.Dispose();
        });

        Assert.Null(exception);
    }

    [Fact]
    public void Bundle___WithoutManifest___ThrowsIOException()
    {
        var bundlePath = CreateBundleWithoutManifest();

        var exception = Assert.Throws<IOException>(() =>
            BundleLoader.Create()
                .WithBundlePath(bundlePath)
                .WithSignatureVerification(false)
                .Build());

        Assert.Contains("manifest.json", exception.Message);
    }

    [Fact]
    public void Bundle___InvalidManifestJson___ThrowsJsonException()
    {
        var bundlePath = CreateBundleWithInvalidManifest();

        Assert.Throws<JsonException>(() =>
            BundleLoader.Create()
                .WithBundlePath(bundlePath)
                .WithSignatureVerification(false)
                .Build());
    }

    [Fact]
    public void Builder___WithPublicKeyOverride___UsesOverride()
    {
        var bundlePath = CreateMinimalBundle();

        // Using a public key override should attempt to verify with that key
        var exception = Assert.Throws<IOException>(() =>
            BundleLoader.Create()
                .WithBundlePath(bundlePath)
                .WithSignatureVerification(true)
                .WithPublicKey("RWSinvalidkey")
                .Build());

        // Should fail because there's no signature file, not because of missing key
        Assert.Contains("minisig", exception.Message, StringComparison.OrdinalIgnoreCase);
    }

    // Helper methods to create test bundles

    private string CreateMinimalBundle()
    {
        var bundlePath = Path.Combine(_tempDir, "test.rbp");

        const string manifest = """
            {
                "bundle_version": "1.0",
                "plugin": {
                    "name": "test-plugin",
                    "version": "1.0.0"
                },
                "platforms": {}
            }
            """;

        using (var zipStream = new FileStream(bundlePath, FileMode.Create))
        using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create))
        {
            var entry = archive.CreateEntry("manifest.json");
            using var writer = new StreamWriter(entry.Open());
            writer.Write(manifest);
        }

        return bundlePath;
    }

    private string CreateBundleWithPlatformButNoLibrary()
    {
        var bundlePath = Path.Combine(_tempDir, "test-no-lib.rbp");

        const string manifest = """
            {
                "bundle_version": "1.0",
                "plugin": {
                    "name": "test-plugin",
                    "version": "1.0.0"
                },
                "platforms": {
                    "linux-x86_64": {
                        "library": "lib/linux-x86_64/libtest.so",
                        "checksum": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    }
                }
            }
            """;

        using (var zipStream = new FileStream(bundlePath, FileMode.Create))
        using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create))
        {
            var entry = archive.CreateEntry("manifest.json");
            using var writer = new StreamWriter(entry.Open());
            writer.Write(manifest);
        }

        return bundlePath;
    }

    private string CreateBundleWithBadChecksum()
    {
        var bundlePath = Path.Combine(_tempDir, "test-bad-checksum.rbp");

        const string manifest = """
            {
                "bundle_version": "1.0",
                "plugin": {
                    "name": "test-plugin",
                    "version": "1.0.0"
                },
                "platforms": {
                    "linux-x86_64": {
                        "library": "lib/linux-x86_64/libtest.so",
                        "checksum": "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    }
                }
            }
            """;

        using (var zipStream = new FileStream(bundlePath, FileMode.Create))
        using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create))
        {
            var manifestEntry = archive.CreateEntry("manifest.json");
            using (var writer = new StreamWriter(manifestEntry.Open()))
            {
                writer.Write(manifest);
            }

            var libEntry = archive.CreateEntry("lib/linux-x86_64/libtest.so");
            using (var writer = new StreamWriter(libEntry.Open()))
            {
                writer.Write("fake library content");
            }
        }

        return bundlePath;
    }

    private string CreateBundleWithValidLibrary()
    {
        var bundlePath = Path.Combine(_tempDir, "test-valid.rbp");

        var libContent = Encoding.UTF8.GetBytes("fake library content for testing");
        var checksum = Sha256Hex(libContent);

        var manifest = $$"""
            {
                "bundle_version": "1.0",
                "plugin": {
                    "name": "test-plugin",
                    "version": "1.0.0"
                },
                "platforms": {
                    "linux-x86_64": {
                        "library": "lib/linux-x86_64/libtest.so",
                        "checksum": "sha256:{{checksum}}"
                    }
                }
            }
            """;

        using (var zipStream = new FileStream(bundlePath, FileMode.Create))
        using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create))
        {
            var manifestEntry = archive.CreateEntry("manifest.json");
            using (var writer = new StreamWriter(manifestEntry.Open()))
            {
                writer.Write(manifest);
            }

            var libEntry = archive.CreateEntry("lib/linux-x86_64/libtest.so");
            using (var stream = libEntry.Open())
            {
                stream.Write(libContent);
            }
        }

        return bundlePath;
    }

    private string CreateBundleWithValidLibraryNoChecksumPrefix()
    {
        var bundlePath = Path.Combine(_tempDir, "test-valid-no-prefix.rbp");

        var libContent = Encoding.UTF8.GetBytes("fake library content for testing");
        var checksum = Sha256Hex(libContent);

        // Note: checksum without "sha256:" prefix
        var manifest = $$"""
            {
                "bundle_version": "1.0",
                "plugin": {
                    "name": "test-plugin",
                    "version": "1.0.0"
                },
                "platforms": {
                    "linux-x86_64": {
                        "library": "lib/linux-x86_64/libtest.so",
                        "checksum": "{{checksum}}"
                    }
                }
            }
            """;

        using (var zipStream = new FileStream(bundlePath, FileMode.Create))
        using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create))
        {
            var manifestEntry = archive.CreateEntry("manifest.json");
            using (var writer = new StreamWriter(manifestEntry.Open()))
            {
                writer.Write(manifest);
            }

            var libEntry = archive.CreateEntry("lib/linux-x86_64/libtest.so");
            using (var stream = libEntry.Open())
            {
                stream.Write(libContent);
            }
        }

        return bundlePath;
    }

    private string CreateBundleWithSchema()
    {
        var bundlePath = Path.Combine(_tempDir, "test-schema.rbp");

        var schemaContent = Encoding.UTF8.GetBytes("""
            // Auto-generated header
            typedef struct {
                uint32_t version;
                uint8_t key[64];
            } SmallRequest;
            """);
        var schemaChecksum = Sha256Hex(schemaContent);

        var manifest = $$"""
            {
                "bundle_version": "1.0",
                "plugin": {
                    "name": "test-plugin",
                    "version": "1.0.0"
                },
                "platforms": {},
                "schemas": {
                    "messages.h": {
                        "path": "schemas/messages.h",
                        "format": "c-header",
                        "checksum": "sha256:{{schemaChecksum}}",
                        "description": "C struct definitions"
                    }
                }
            }
            """;

        using (var zipStream = new FileStream(bundlePath, FileMode.Create))
        using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create))
        {
            var manifestEntry = archive.CreateEntry("manifest.json");
            using (var writer = new StreamWriter(manifestEntry.Open()))
            {
                writer.Write(manifest);
            }

            var schemaEntry = archive.CreateEntry("schemas/messages.h");
            using (var stream = schemaEntry.Open())
            {
                stream.Write(schemaContent);
            }
        }

        return bundlePath;
    }

    private string CreateBundleWithoutManifest()
    {
        var bundlePath = Path.Combine(_tempDir, "test-no-manifest.rbp");

        using (var zipStream = new FileStream(bundlePath, FileMode.Create))
        using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create))
        {
            var entry = archive.CreateEntry("some-file.txt");
            using var writer = new StreamWriter(entry.Open());
            writer.Write("content");
        }

        return bundlePath;
    }

    private string CreateBundleWithInvalidManifest()
    {
        var bundlePath = Path.Combine(_tempDir, "test-invalid-manifest.rbp");

        using (var zipStream = new FileStream(bundlePath, FileMode.Create))
        using (var archive = new ZipArchive(zipStream, ZipArchiveMode.Create))
        {
            var entry = archive.CreateEntry("manifest.json");
            using var writer = new StreamWriter(entry.Open());
            writer.Write("{ invalid json }}}");
        }

        return bundlePath;
    }

    private static string Sha256Hex(byte[] data)
    {
        var hash = SHA256.HashData(data);
        return Convert.ToHexString(hash).ToLowerInvariant();
    }
}
