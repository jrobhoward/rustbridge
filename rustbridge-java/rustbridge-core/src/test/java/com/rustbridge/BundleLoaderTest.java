package com.rustbridge;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SignatureException;
import java.util.List;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import static org.junit.jupiter.api.Assertions.*;

class BundleLoaderTest {

    @TempDir
    Path tempDir;

    @Test
    void builder___no_bundle_path___throws_exception() {
        IllegalStateException exception = assertThrows(IllegalStateException.class, () -> {
            BundleLoader.builder().build();
        });

        assertTrue(exception.getMessage().contains("bundlePath"));
    }

    @Test
    void builder___nonexistent_path___throws_exception() {
        assertThrows(IOException.class, () -> {
            BundleLoader.builder()
                    .bundlePath("/nonexistent/path/bundle.rbp")
                    .build();
        });
    }

    @Test
    void builder___path_as_string___works() throws IOException {
        Path bundlePath = createMinimalBundle();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath.toString())
                .verifySignatures(false)
                .build()) {

            assertNotNull(loader);
            assertNotNull(loader.getManifest());
        }
    }

    @Test
    void builder___path_as_path___works() throws IOException {
        Path bundlePath = createMinimalBundle();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build()) {

            assertNotNull(loader);
        }
    }

    @Test
    void builder___verify_signatures_default___is_true() throws IOException {
        Path bundlePath = createMinimalBundle();

        IOException exception = assertThrows(IOException.class, () -> {
            BundleLoader.builder()
                    .bundlePath(bundlePath)
                    .build();
        });

        assertTrue(exception.getMessage().contains("public key") ||
                exception.getMessage().contains("signature"));
    }

    @Test
    void getManifest___returns_parsed_manifest() throws IOException {
        Path bundlePath = createMinimalBundle();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build()) {

            BundleLoader.BundleManifest manifest = loader.getManifest();

            assertNotNull(manifest);
            assertEquals("1.0", manifest.bundleVersion);
            assertNotNull(manifest.plugin);
            assertEquals("test-plugin", manifest.plugin.name());
            assertEquals("1.0.0", manifest.plugin.version());
        }
    }

    @Test
    void listFiles___returns_all_files() throws IOException {
        Path bundlePath = createMinimalBundle();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build()) {

            List<String> files = loader.listFiles();

            assertTrue(files.contains("manifest.json"));
        }
    }

    @Test
    void extractLibrary___unsupported_platform___throws_exception() throws IOException {
        Path bundlePath = createMinimalBundle();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build()) {

            IOException exception = assertThrows(IOException.class, () -> {
                loader.extractLibrary("unknown-platform", tempDir);
            });

            assertTrue(exception.getMessage().contains("not supported"));
        }
    }

    @Test
    void extractLibrary___missing_library_file___throws_exception() throws IOException {
        Path bundlePath = createBundleWithPlatformButNoLibrary();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build()) {

            IOException exception = assertThrows(IOException.class, () -> {
                loader.extractLibrary("linux-x86_64", tempDir);
            });

            assertTrue(exception.getMessage().contains("not found"));
        }
    }

    @Test
    void extractLibrary___checksum_mismatch___throws_exception() throws IOException {
        Path bundlePath = createBundleWithBadChecksum();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build()) {

            IOException exception = assertThrows(IOException.class, () -> {
                loader.extractLibrary("linux-x86_64", tempDir);
            });

            assertTrue(exception.getMessage().contains("Checksum"));
        }
    }

    @Test
    void extractLibrary___valid_checksum___succeeds() throws IOException, SignatureException {
        Path bundlePath = createBundleWithValidLibrary();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build()) {

            Path extractedLib = loader.extractLibrary("linux-x86_64", tempDir);

            assertTrue(Files.exists(extractedLib));
            assertTrue(extractedLib.toString().contains("libtest.so"));
        }
    }

    @Test
    void getSchemas___no_schemas___returns_empty_map() throws IOException {
        Path bundlePath = createMinimalBundle();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build()) {

            Map<String, BundleLoader.BundleManifest.SchemaInfo> schemas = loader.getSchemas();

            assertTrue(schemas.isEmpty());
        }
    }

    @Test
    void extractSchema___missing_schema___throws_exception() throws IOException {
        Path bundlePath = createMinimalBundle();

        try (BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build()) {

            IOException exception = assertThrows(IOException.class, () -> {
                loader.extractSchema("nonexistent.h", tempDir);
            });

            assertTrue(exception.getMessage().contains("not found"));
        }
    }

    @Test
    void close___can_be_called_multiple_times() throws IOException {
        Path bundlePath = createMinimalBundle();

        BundleLoader loader = BundleLoader.builder()
                .bundlePath(bundlePath)
                .verifySignatures(false)
                .build();

        assertDoesNotThrow(() -> {
            loader.close();
            loader.close();
        });
    }

    @Test
    void bundle___without_manifest___throws_exception() throws IOException {
        Path bundlePath = createBundleWithoutManifest();

        IOException exception = assertThrows(IOException.class, () -> {
            BundleLoader.builder()
                    .bundlePath(bundlePath)
                    .verifySignatures(false)
                    .build();
        });

        assertTrue(exception.getMessage().contains("manifest.json"));
    }

    // Helper methods to create test bundles

    private Path createMinimalBundle() throws IOException {
        Path bundlePath = tempDir.resolve("test.rbp");

        String manifest = """
                {
                    "bundle_version": "1.0",
                    "plugin": {
                        "name": "test-plugin",
                        "version": "1.0.0"
                    },
                    "platforms": {}
                }
                """;

        try (ZipOutputStream zos = new ZipOutputStream(new FileOutputStream(bundlePath.toFile()))) {
            ZipEntry entry = new ZipEntry("manifest.json");
            zos.putNextEntry(entry);
            zos.write(manifest.getBytes(StandardCharsets.UTF_8));
            zos.closeEntry();
        }

        return bundlePath;
    }

    private Path createBundleWithPlatformButNoLibrary() throws IOException {
        Path bundlePath = tempDir.resolve("test.rbp");

        String manifest = """
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

        try (ZipOutputStream zos = new ZipOutputStream(new FileOutputStream(bundlePath.toFile()))) {
            ZipEntry entry = new ZipEntry("manifest.json");
            zos.putNextEntry(entry);
            zos.write(manifest.getBytes(StandardCharsets.UTF_8));
            zos.closeEntry();
        }

        return bundlePath;
    }

    private Path createBundleWithBadChecksum() throws IOException {
        Path bundlePath = tempDir.resolve("test.rbp");

        String manifest = """
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

        try (ZipOutputStream zos = new ZipOutputStream(new FileOutputStream(bundlePath.toFile()))) {
            ZipEntry manifestEntry = new ZipEntry("manifest.json");
            zos.putNextEntry(manifestEntry);
            zos.write(manifest.getBytes(StandardCharsets.UTF_8));
            zos.closeEntry();

            ZipEntry libEntry = new ZipEntry("lib/linux-x86_64/libtest.so");
            zos.putNextEntry(libEntry);
            zos.write("fake library content".getBytes(StandardCharsets.UTF_8));
            zos.closeEntry();
        }

        return bundlePath;
    }

    private Path createBundleWithValidLibrary() throws IOException {
        Path bundlePath = tempDir.resolve("test.rbp");

        byte[] libContent = "fake library content for testing".getBytes(StandardCharsets.UTF_8);
        String checksum = sha256Hex(libContent);

        String manifest = """
                {
                    "bundle_version": "1.0",
                    "plugin": {
                        "name": "test-plugin",
                        "version": "1.0.0"
                    },
                    "platforms": {
                        "linux-x86_64": {
                            "library": "lib/linux-x86_64/libtest.so",
                            "checksum": "sha256:%s"
                        }
                    }
                }
                """.formatted(checksum);

        try (ZipOutputStream zos = new ZipOutputStream(new FileOutputStream(bundlePath.toFile()))) {
            ZipEntry manifestEntry = new ZipEntry("manifest.json");
            zos.putNextEntry(manifestEntry);
            zos.write(manifest.getBytes(StandardCharsets.UTF_8));
            zos.closeEntry();

            ZipEntry libEntry = new ZipEntry("lib/linux-x86_64/libtest.so");
            zos.putNextEntry(libEntry);
            zos.write(libContent);
            zos.closeEntry();
        }

        return bundlePath;
    }

    private Path createBundleWithoutManifest() throws IOException {
        Path bundlePath = tempDir.resolve("test.rbp");

        try (ZipOutputStream zos = new ZipOutputStream(new FileOutputStream(bundlePath.toFile()))) {
            ZipEntry entry = new ZipEntry("some-file.txt");
            zos.putNextEntry(entry);
            zos.write("content".getBytes(StandardCharsets.UTF_8));
            zos.closeEntry();
        }

        return bundlePath;
    }

    private String sha256Hex(byte[] data) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            byte[] hash = digest.digest(data);
            StringBuilder sb = new StringBuilder();
            for (byte b : hash) {
                sb.append(String.format("%02x", b));
            }
            return sb.toString();
        } catch (NoSuchAlgorithmException e) {
            throw new RuntimeException(e);
        }
    }
}
