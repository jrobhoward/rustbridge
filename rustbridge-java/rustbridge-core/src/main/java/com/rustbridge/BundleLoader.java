package com.rustbridge;

import com.google.gson.Gson;
import com.google.gson.annotations.SerializedName;
import java.io.*;
import java.nio.file.*;
import java.security.InvalidKeyException;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.security.SignatureException;
import java.util.*;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;

/**
 * Loader for rustbridge plugin bundles (.rbp files).
 *
 * <p>Provides functionality to:
 * <ul>
 *   <li>Extract and parse bundle manifests</li>
 *   <li>Extract platform-specific libraries</li>
 *   <li>Verify SHA256 checksums</li>
 *   <li>Verify minisign signatures (optional)</li>
 * </ul>
 *
 * <h2>Usage</h2>
 * <pre>{@code
 * // Load with signature verification
 * BundleLoader loader = BundleLoader.builder()
 *     .bundlePath("my-plugin-1.0.0.rbp")
 *     .verifySignatures(true)
 *     .publicKey("RWS...") // Optional: override manifest key
 *     .build();
 *
 * Path libPath = loader.extractLibrary();
 *
 * // Load without signature verification (development only)
 * BundleLoader loader = BundleLoader.builder()
 *     .bundlePath("my-plugin-1.0.0.rbp")
 *     .verifySignatures(false)
 *     .build();
 * }</pre>
 */
public class BundleLoader implements AutoCloseable {
    private final Path bundlePath;
    private final boolean verifySignatures;
    private final String publicKeyOverride;
    private final ZipFile zipFile;
    private final BundleManifest manifest;

    private BundleLoader(Builder builder) throws IOException {
        this.bundlePath = builder.bundlePath;
        this.verifySignatures = builder.verifySignatures;
        this.publicKeyOverride = builder.publicKeyOverride;
        this.zipFile = new ZipFile(bundlePath.toFile());
        this.manifest = loadManifest();

        // Verify manifest signature if enabled
        if (verifySignatures) {
            verifyManifestSignature();
        }
    }

    /**
     * Create a new builder for constructing a BundleLoader.
     */
    public static Builder builder() {
        return new Builder();
    }

    /**
     * Get the bundle manifest.
     */
    public BundleManifest getManifest() {
        return manifest;
    }

    /**
     * Extract the library for the current platform to a temporary directory.
     *
     * @return path to the extracted library
     * @throws IOException if extraction fails
     * @throws SignatureException if signature verification fails (when enabled)
     */
    public Path extractLibrary() throws IOException, SignatureException {
        Path tempDir = Files.createTempDirectory("rustbridge-");
        return extractLibrary(tempDir);
    }

    /**
     * Extract the library for the current platform to the specified directory.
     *
     * @param outputDir directory to extract the library to
     * @return path to the extracted library
     * @throws IOException if extraction fails
     * @throws SignatureException if signature verification fails (when enabled)
     */
    public Path extractLibrary(Path outputDir) throws IOException, SignatureException {
        String platform = detectPlatform();
        return extractLibrary(platform, outputDir);
    }

    /**
     * Extract the library for a specific platform.
     *
     * @param platform platform string (e.g., "linux-x86_64")
     * @param outputDir directory to extract the library to
     * @return path to the extracted library
     * @throws IOException if extraction fails
     * @throws SignatureException if signature verification fails (when enabled)
     */
    public Path extractLibrary(String platform, Path outputDir)
            throws IOException, SignatureException {
        BundleManifest.PlatformInfo platformInfo = manifest.platforms.get(platform);
        if (platformInfo == null) {
            throw new IOException("Platform not supported: " + platform);
        }

        // Extract the library
        ZipEntry libEntry = zipFile.getEntry(platformInfo.library);
        if (libEntry == null) {
            throw new IOException("Library not found in bundle: " + platformInfo.library);
        }

        byte[] libData = readZipEntry(libEntry);

        // Verify checksum
        if (!verifyChecksum(libData, platformInfo.checksum)) {
            throw new IOException(
                "Checksum verification failed for " + platformInfo.library
            );
        }

        // Verify signature if enabled
        if (verifySignatures) {
            verifyLibrarySignature(platformInfo.library, libData);
        }

        // Write to output directory
        String fileName = Paths.get(platformInfo.library).getFileName().toString();
        Path outputPath = outputDir.resolve(fileName);
        Files.write(outputPath, libData);

        // Make executable on Unix
        if (!System.getProperty("os.name").toLowerCase().contains("win")) {
            outputPath.toFile().setExecutable(true);
        }

        return outputPath;
    }

    /**
     * List all files in the bundle.
     */
    public List<String> listFiles() {
        List<String> files = new ArrayList<>();
        Enumeration<? extends ZipEntry> entries = zipFile.entries();
        while (entries.hasMoreElements()) {
            files.add(entries.nextElement().getName());
        }
        return files;
    }

    @Override
    public void close() throws IOException {
        zipFile.close();
    }

    /**
     * Load and parse the manifest.json from the bundle.
     */
    private BundleManifest loadManifest() throws IOException {
        ZipEntry manifestEntry = zipFile.getEntry("manifest.json");
        if (manifestEntry == null) {
            throw new IOException("manifest.json not found in bundle");
        }

        byte[] manifestData = readZipEntry(manifestEntry);
        Gson gson = new Gson();
        return gson.fromJson(new String(manifestData), BundleManifest.class);
    }

    /**
     * Verify the manifest signature.
     */
    private void verifyManifestSignature() throws IOException {
        String publicKey = publicKeyOverride != null ?
            publicKeyOverride : manifest.publicKey;

        if (publicKey == null) {
            throw new IOException(
                "Signature verification enabled but no public key available. " +
                "Bundle must include public_key in manifest, or provide via publicKey() builder method."
            );
        }

        // Read manifest data
        ZipEntry manifestEntry = zipFile.getEntry("manifest.json");
        byte[] manifestData = readZipEntry(manifestEntry);

        // Read signature
        ZipEntry sigEntry = zipFile.getEntry("manifest.json.minisig");
        if (sigEntry == null) {
            throw new IOException(
                "Signature verification enabled but manifest.json.minisig not found in bundle"
            );
        }
        String signature = new String(readZipEntry(sigEntry));

        // Verify
        try {
            MinisignVerifier verifier = new MinisignVerifier(publicKey);
            if (!verifier.verify(manifestData, signature)) {
                throw new IOException("Manifest signature verification failed");
            }
        } catch (InvalidKeyException | SignatureException e) {
            throw new IOException("Manifest signature verification failed", e);
        }
    }

    /**
     * Verify a library signature.
     */
    private void verifyLibrarySignature(String libraryPath, byte[] libraryData)
            throws IOException, SignatureException {
        String publicKey = publicKeyOverride != null ?
            publicKeyOverride : manifest.publicKey;

        if (publicKey == null) {
            throw new IOException("No public key available for signature verification");
        }

        // Read signature
        String sigPath = libraryPath + ".minisig";
        ZipEntry sigEntry = zipFile.getEntry(sigPath);
        if (sigEntry == null) {
            throw new IOException(
                "Signature verification enabled but " + sigPath + " not found in bundle"
            );
        }
        String signature = new String(readZipEntry(sigEntry));

        // Verify
        try {
            MinisignVerifier verifier = new MinisignVerifier(publicKey);
            if (!verifier.verify(libraryData, signature)) {
                throw new SignatureException("Library signature verification failed: " + libraryPath);
            }
        } catch (InvalidKeyException e) {
            throw new SignatureException("Library signature verification failed", e);
        }
    }

    /**
     * Read a zip entry into a byte array.
     */
    private byte[] readZipEntry(ZipEntry entry) throws IOException {
        try (InputStream is = zipFile.getInputStream(entry)) {
            ByteArrayOutputStream baos = new ByteArrayOutputStream();
            byte[] buffer = new byte[8192];
            int len;
            while ((len = is.read(buffer)) != -1) {
                baos.write(buffer, 0, len);
            }
            return baos.toByteArray();
        }
    }

    /**
     * Verify SHA256 checksum.
     */
    private boolean verifyChecksum(byte[] data, String expectedChecksum) {
        try {
            MessageDigest digest = MessageDigest.getInstance("SHA-256");
            byte[] hash = digest.digest(data);
            String actualChecksum = bytesToHex(hash);

            // Handle both "sha256:xxx" and raw "xxx" formats
            String expected = expectedChecksum.startsWith("sha256:")
                ? expectedChecksum.substring(7)
                : expectedChecksum;

            return actualChecksum.equalsIgnoreCase(expected);
        } catch (NoSuchAlgorithmException e) {
            return false;
        }
    }

    /**
     * Detect the current platform string.
     */
    private String detectPlatform() {
        String os = System.getProperty("os.name").toLowerCase();
        String arch = System.getProperty("os.arch").toLowerCase();

        String osName;
        if (os.contains("linux")) {
            osName = "linux";
        } else if (os.contains("mac") || os.contains("darwin")) {
            osName = "darwin";
        } else if (os.contains("win")) {
            osName = "windows";
        } else {
            osName = "unknown";
        }

        String archName;
        if (arch.contains("amd64") || arch.contains("x86_64")) {
            archName = "x86_64";
        } else if (arch.contains("aarch64") || arch.contains("arm64")) {
            archName = "aarch64";
        } else {
            archName = arch;
        }

        return osName + "-" + archName;
    }

    /**
     * Convert bytes to hex string.
     */
    private static String bytesToHex(byte[] bytes) {
        StringBuilder sb = new StringBuilder();
        for (byte b : bytes) {
            sb.append(String.format("%02x", b));
        }
        return sb.toString();
    }

    /**
     * Builder for BundleLoader.
     */
    public static class Builder {
        private Path bundlePath;
        private boolean verifySignatures = true; // Secure by default
        private String publicKeyOverride;

        /**
         * Set the path to the bundle file.
         */
        public Builder bundlePath(String path) {
            this.bundlePath = Paths.get(path);
            return this;
        }

        /**
         * Set the path to the bundle file.
         */
        public Builder bundlePath(Path path) {
            this.bundlePath = path;
            return this;
        }

        /**
         * Enable or disable signature verification.
         *
         * <p>Default: true (verification enabled)
         *
         * <p><strong>WARNING:</strong> Disabling signature verification means
         * the bundle can contain malicious code. Only disable for development/testing.
         */
        public Builder verifySignatures(boolean verify) {
            this.verifySignatures = verify;
            return this;
        }

        /**
         * Override the public key from the manifest.
         *
         * <p>This allows you to provide a trusted public key instead of using
         * the key embedded in the manifest. Useful for defense-in-depth.
         *
         * @param publicKey minisign public key in base64 format (e.g., "RWS...")
         */
        public Builder publicKey(String publicKey) {
            this.publicKeyOverride = publicKey;
            return this;
        }

        /**
         * Build the BundleLoader.
         *
         * @throws IOException if the bundle cannot be opened or manifest is invalid
         */
        public BundleLoader build() throws IOException {
            if (bundlePath == null) {
                throw new IllegalStateException("bundlePath must be set");
            }
            if (!Files.exists(bundlePath)) {
                throw new FileNotFoundException("Bundle not found: " + bundlePath);
            }
            return new BundleLoader(this);
        }
    }

    /**
     * Bundle manifest structure.
     */
    public static class BundleManifest {
        @SerializedName("bundle_version")
        public String bundleVersion;

        public PluginInfo plugin;
        public Map<String, PlatformInfo> platforms;
        public ApiInfo api;

        @SerializedName("public_key")
        public String publicKey; // Minisign public key (base64)

        public static class PluginInfo {
            public String name;
            public String version;
            public String description;
            public List<String> authors;
            public String license;
            public String repository;
        }

        public static class PlatformInfo {
            public String library;
            public String checksum;
        }

        public static class ApiInfo {
            @SerializedName("min_rustbridge_version")
            public String minRustbridgeVersion;

            public List<String> transports;
            public List<MessageInfo> messages;
        }

        public static class MessageInfo {
            @SerializedName("type_tag")
            public String typeTag;

            public String description;

            @SerializedName("request_schema")
            public String requestSchema;

            @SerializedName("response_schema")
            public String responseSchema;

            @SerializedName("message_id")
            public Integer messageId;

            @SerializedName("cstruct_request")
            public String cstructRequest;

            @SerializedName("cstruct_response")
            public String cstructResponse;
        }
    }
}
