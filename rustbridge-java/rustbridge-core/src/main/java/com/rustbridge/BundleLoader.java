package com.rustbridge;

import com.fasterxml.jackson.annotation.JsonProperty;
import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import org.jetbrains.annotations.NotNull;
import org.jetbrains.annotations.Nullable;

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

        try {
            this.manifest = loadManifest();

            // Verify manifest signature if enabled
            if (verifySignatures) {
                verifyManifestSignature();
            }
        } catch (IOException | RuntimeException e) {
            // Ensure ZipFile is closed if construction fails
            zipFile.close();
            throw e;
        }
    }

    /**
     * Create a new builder for constructing a BundleLoader.
     */
    public static @NotNull Builder builder() {
        return new Builder();
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
     * Get the bundle manifest.
     */
    public @NotNull BundleManifest getManifest() {
        return manifest;
    }

    /**
     * Extract the library for the current platform to a unique temporary directory.
     *
     * <p>The library is extracted to a unique subdirectory under the system temp directory,
     * ensuring no conflicts with other extractions. The caller is responsible for cleaning
     * up the temporary directory when done.
     *
     * <p>Uses the default variant (typically "release").
     *
     * @return path to the extracted library
     * @throws IOException        if extraction fails
     * @throws SignatureException if signature verification fails (when enabled)
     */
    public @NotNull Path extractLibrary() throws IOException, SignatureException {
        // Create unique temp directory under system temp path
        Path tempBase = Paths.get(System.getProperty("java.io.tmpdir"));
        Path tempDir = Files.createTempDirectory(tempBase, "rustbridge-");
        String platform = detectPlatform();
        String variant = getDefaultVariant(platform);
        return extractLibraryInternal(platform, variant, tempDir, false);
    }

    /**
     * Extract the library for the current platform to the specified directory.
     *
     * <p>This method will fail if the library file already exists at the target path.
     * This prevents accidental overwrites and ensures the caller has explicit control
     * over file lifecycle.
     *
     * <p>Uses the default variant (typically "release").
     *
     * @param outputDir directory to extract the library to
     * @return path to the extracted library
     * @throws IOException        if extraction fails or file already exists
     * @throws SignatureException if signature verification fails (when enabled)
     */
    public @NotNull Path extractLibrary(@NotNull Path outputDir) throws IOException, SignatureException {
        String platform = detectPlatform();
        return extractLibrary(platform, outputDir);
    }

    /**
     * Extract the library for a specific platform to the specified directory.
     *
     * <p>This method will fail if the library file already exists at the target path.
     * This prevents accidental overwrites and ensures the caller has explicit control
     * over file lifecycle.
     *
     * <p>Uses the default variant (typically "release").
     *
     * @param platform  platform string (e.g., "linux-x86_64")
     * @param outputDir directory to extract the library to
     * @return path to the extracted library
     * @throws IOException        if extraction fails or file already exists
     * @throws SignatureException if signature verification fails (when enabled)
     */
    public @NotNull Path extractLibrary(@NotNull String platform, @NotNull Path outputDir)
            throws IOException, SignatureException {
        String variant = getDefaultVariant(platform);
        return extractLibraryInternal(platform, variant, outputDir, true);
    }

    /**
     * Extract a specific variant of the library for a platform to the specified directory.
     *
     * <p>This method will fail if the library file already exists at the target path.
     *
     * @param platform  platform string (e.g., "linux-x86_64")
     * @param variant   variant name (e.g., "release", "debug")
     * @param outputDir directory to extract the library to
     * @return path to the extracted library
     * @throws IOException        if extraction fails, file already exists, or variant not found
     * @throws SignatureException if signature verification fails (when enabled)
     */
    public @NotNull Path extractLibrary(
            @NotNull String platform,
            @NotNull String variant,
            @NotNull Path outputDir
    ) throws IOException, SignatureException {
        return extractLibraryInternal(platform, variant, outputDir, true);
    }

    /**
     * Get the default variant for a platform.
     *
     * @param platform platform string (e.g., "linux-x86_64")
     * @return default variant name (typically "release")
     */
    public @NotNull String getDefaultVariant(@NotNull String platform) {
        BundleManifest.PlatformInfo platformInfo = manifest.platforms.get(platform);
        if (platformInfo == null) {
            return "release";
        }
        return platformInfo.getDefaultVariant();
    }

    /**
     * List available variants for a platform.
     *
     * @param platform platform string (e.g., "linux-x86_64")
     * @return list of available variant names
     * @throws IOException if platform is not supported
     */
    public @NotNull List<String> listVariants(@NotNull String platform) throws IOException {
        BundleManifest.PlatformInfo platformInfo = manifest.platforms.get(platform);
        if (platformInfo == null) {
            throw new IOException("Platform not supported: " + platform);
        }
        return platformInfo.listVariants();
    }

    /**
     * Get build info from the manifest (v2.0+ bundles only).
     *
     * @return build info, or null if not present
     */
    public @Nullable BundleManifest.BuildInfo getBuildInfo() {
        return manifest.buildInfo;
    }

    /**
     * Internal method to extract the library with configurable overwrite behavior.
     *
     * @param platform           platform string (e.g., "linux-x86_64")
     * @param variant            variant name (e.g., "release", "debug")
     * @param outputDir          directory to extract the library to
     * @param failIfExists       if true, fail when the target file already exists
     * @return path to the extracted library
     */
    private @NotNull Path extractLibraryInternal(
            @NotNull String platform,
            @NotNull String variant,
            @NotNull Path outputDir,
            boolean failIfExists
    ) throws IOException, SignatureException {
        BundleManifest.PlatformInfo platformInfo = manifest.platforms.get(platform);
        if (platformInfo == null) {
            throw new IOException("Platform not supported: " + platform);
        }

        // Get library path and checksum for the requested variant
        String libraryPath = platformInfo.getLibrary(variant);
        String checksum = platformInfo.getChecksum(variant);

        if (libraryPath == null || libraryPath.isEmpty()) {
            throw new IOException(
                    "Variant '" + variant + "' not found for platform '" + platform + "'"
            );
        }

        // Extract the library
        ZipEntry libEntry = zipFile.getEntry(libraryPath);
        if (libEntry == null) {
            throw new IOException("Library not found in bundle: " + libraryPath);
        }

        byte[] libData = readZipEntry(libEntry);

        // Verify checksum
        if (!verifyChecksum(libData, checksum)) {
            throw new IOException(
                    "Checksum verification failed for " + libraryPath
            );
        }

        // Verify signature if enabled
        if (verifySignatures) {
            verifyLibrarySignature(libraryPath, libData);
        }

        // Determine output path
        String fileName = Paths.get(libraryPath).getFileName().toString();
        Path outputPath = outputDir.resolve(fileName);

        // Check if file already exists when user specifies path
        if (failIfExists && Files.exists(outputPath)) {
            throw new IOException(
                    "Library already exists at target path: " + outputPath + ". " +
                    "Remove the existing file or use extractLibrary() for automatic temp directory."
            );
        }

        // Ensure output directory exists
        Files.createDirectories(outputDir);

        // Write the library
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
    public @NotNull List<String> listFiles() {
        List<String> files = new ArrayList<>();
        Enumeration<? extends ZipEntry> entries = zipFile.entries();
        while (entries.hasMoreElements()) {
            files.add(entries.nextElement().getName());
        }
        return files;
    }

    /**
     * Get all available schemas in the bundle.
     *
     * @return map of schema name to schema info
     */
    public @NotNull Map<String, BundleManifest.SchemaInfo> getSchemas() {
        if (manifest.schemas == null) {
            return new HashMap<>();
        }
        return manifest.schemas;
    }

    /**
     * Extract a schema file from the bundle.
     *
     * @param schemaName name of the schema (e.g., "messages.h")
     * @param outputDir  directory to extract the schema to
     * @return path to the extracted schema file
     * @throws IOException if extraction fails
     */
    public @NotNull Path extractSchema(@NotNull String schemaName, @NotNull Path outputDir) throws IOException {
        BundleManifest.SchemaInfo schemaInfo = manifest.schemas != null
                ? manifest.schemas.get(schemaName)
                : null;

        if (schemaInfo == null) {
            throw new IOException("Schema not found in bundle: " + schemaName);
        }

        // Extract the schema file
        ZipEntry schemaEntry = zipFile.getEntry(schemaInfo.path());
        if (schemaEntry == null) {
            throw new IOException("Schema file not found in bundle: " + schemaInfo.path());
        }

        byte[] schemaData = readZipEntry(schemaEntry);

        // Verify checksum
        if (!verifyChecksum(schemaData, schemaInfo.checksum())) {
            throw new IOException(
                    "Checksum verification failed for schema " + schemaName
            );
        }

        // Write to output directory
        Path outputPath = outputDir.resolve(schemaName);
        Files.write(outputPath, schemaData);

        return outputPath;
    }

    /**
     * Read a schema file content as string.
     *
     * @param schemaName name of the schema (e.g., "messages.h")
     * @return schema file content
     * @throws IOException if reading fails
     */
    public @NotNull String readSchema(@NotNull String schemaName) throws IOException {
        BundleManifest.SchemaInfo schemaInfo = manifest.schemas != null
                ? manifest.schemas.get(schemaName)
                : null;

        if (schemaInfo == null) {
            throw new IOException("Schema not found in bundle: " + schemaName);
        }

        // Extract the schema file
        ZipEntry schemaEntry = zipFile.getEntry(schemaInfo.path());
        if (schemaEntry == null) {
            throw new IOException("Schema file not found in bundle: " + schemaInfo.path());
        }

        byte[] schemaData = readZipEntry(schemaEntry);

        // Verify checksum
        if (!verifyChecksum(schemaData, schemaInfo.checksum())) {
            throw new IOException(
                    "Checksum verification failed for schema " + schemaName
            );
        }

        return new String(schemaData);
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
        ObjectMapper mapper = new ObjectMapper()
                .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
        try {
            return mapper.readValue(manifestData, BundleManifest.class);
        } catch (Exception e) {
            throw new RuntimeException("Failed to parse bundle manifest", e);
        }
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
     * Builder for BundleLoader.
     */
    public static class Builder {
        private Path bundlePath;
        private boolean verifySignatures = true; // Secure by default
        private String publicKeyOverride;

        /**
         * Set the path to the bundle file.
         */
        public @NotNull Builder bundlePath(@NotNull String path) {
            this.bundlePath = Paths.get(path);
            return this;
        }

        /**
         * Set the path to the bundle file.
         */
        public @NotNull Builder bundlePath(@NotNull Path path) {
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
        public @NotNull Builder verifySignatures(boolean verify) {
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
        public @NotNull Builder publicKey(@Nullable String publicKey) {
            this.publicKeyOverride = publicKey;
            return this;
        }

        /**
         * Build the BundleLoader.
         *
         * @throws IOException if the bundle cannot be opened or manifest is invalid
         */
        public @NotNull BundleLoader build() throws IOException {
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
        @JsonProperty("bundle_version")
        public String bundleVersion;

        public PluginInfo plugin;
        public Map<String, PlatformInfo> platforms;
        public ApiInfo api;

        @JsonProperty("public_key")
        public String publicKey; // Minisign public key (base64)

        public Map<String, SchemaInfo> schemas; // Schema files in the bundle

        @JsonProperty("build_info")
        public BuildInfo buildInfo; // Build metadata (v2.0+)

        public Sbom sbom; // SBOM information (v2.0+)

        @JsonProperty("schema_checksum")
        public String schemaChecksum; // Combined schema checksum for validation (v2.0+)

        public String notices; // Path to license notices file in bundle (v2.0+)

        /**
         * Plugin metadata information.
         *
         * @param name        plugin name
         * @param version     plugin version
         * @param description plugin description
         * @param authors     plugin authors
         * @param license     plugin license
         * @param repository  plugin repository URL
         */
        public record PluginInfo(
                String name,
                String version,
                String description,
                List<String> authors,
                String license,
                String repository
        ) {}

        /**
         * Variant-specific library information.
         *
         * @param library  path to the library file within the bundle
         * @param checksum SHA256 checksum of the library
         * @param build    optional build metadata (profile, opt_level, features, etc.)
         */
        public record VariantInfo(
                String library,
                String checksum,
                Object build
        ) {}

        /**
         * Platform-specific library information with variant support.
         *
         * <p>For v2.0 bundles, variants contain the actual library info.
         * For v1.0 bundles (and v2.0 backward compat), library/checksum
         * are populated directly.
         *
         * @param library        path to the library file (backward compat / default variant)
         * @param checksum       SHA256 checksum of the library (backward compat / default variant)
         * @param defaultVariant the default variant name (usually "release")
         * @param variants       map of variant name to VariantInfo
         */
        public record PlatformInfo(
                String library,
                String checksum,
                @JsonProperty("default_variant") String defaultVariant,
                Map<String, VariantInfo> variants
        ) {
            /**
             * Get the effective library path, preferring variant if available.
             *
             * @param variant variant name (e.g., "release", "debug")
             * @return library path
             */
            public String getLibrary(String variant) {
                if (variants != null && variants.containsKey(variant)) {
                    return variants.get(variant).library();
                }
                return library;
            }

            /**
             * Get the effective checksum, preferring variant if available.
             *
             * @param variant variant name (e.g., "release", "debug")
             * @return checksum
             */
            public String getChecksum(String variant) {
                if (variants != null && variants.containsKey(variant)) {
                    return variants.get(variant).checksum();
                }
                return checksum;
            }

            /**
             * Get the default variant name.
             *
             * @return default variant name, or "release" if not specified
             */
            public String getDefaultVariant() {
                return defaultVariant != null ? defaultVariant : "release";
            }

            /**
             * List available variants for this platform.
             *
             * @return list of variant names
             */
            public List<String> listVariants() {
                if (variants == null || variants.isEmpty()) {
                    return List.of("release");
                }
                return new ArrayList<>(variants.keySet());
            }
        }

        /**
         * API information for the plugin.
         *
         * @param minRustbridgeVersion minimum required rustbridge version
         * @param transports           supported transport types (e.g., "json", "cstruct")
         * @param messages             message type definitions
         */
        public record ApiInfo(
                @JsonProperty("min_rustbridge_version") String minRustbridgeVersion,
                List<String> transports,
                List<MessageInfo> messages
        ) {}

        /**
         * Message type information.
         *
         * @param typeTag         message type tag (e.g., "user.create")
         * @param description     message description
         * @param requestSchema   JSON Schema reference for the request type
         * @param responseSchema  JSON Schema reference for the response type
         * @param messageId       numeric message ID for binary transport
         * @param cstructRequest  C struct name for request (binary transport)
         * @param cstructResponse C struct name for response (binary transport)
         */
        public record MessageInfo(
                @JsonProperty("type_tag") String typeTag,
                String description,
                @JsonProperty("request_schema") String requestSchema,
                @JsonProperty("response_schema") String responseSchema,
                @JsonProperty("message_id") Integer messageId,
                @JsonProperty("cstruct_request") String cstructRequest,
                @JsonProperty("cstruct_response") String cstructResponse
        ) {}

        /**
         * Schema file information.
         *
         * @param path        path to the schema file within the bundle
         * @param format      schema format (e.g., "json-schema", "c-header")
         * @param checksum    SHA256 checksum of the schema file
         * @param description schema description
         */
        public record SchemaInfo(
                String path,
                String format,
                String checksum,
                String description
        ) {}

        /**
         * Git repository information.
         *
         * @param commitHash  full commit hash
         * @param commitShort short commit hash
         * @param branch      branch name
         * @param tag         tag name (if applicable)
         * @param dirty       whether working directory had uncommitted changes
         * @param repository  repository URL
         */
        public record GitInfo(
                @JsonProperty("commit_hash") String commitHash,
                @JsonProperty("commit_short") String commitShort,
                String branch,
                String tag,
                Boolean dirty,
                String repository
        ) {}

        /**
         * Build metadata information.
         *
         * @param builtBy           who/what built the bundle (e.g., "CI/CD", "developer")
         * @param builtAt           ISO 8601 timestamp
         * @param host              host triple (e.g., "x86_64-unknown-linux-gnu")
         * @param compiler          compiler version (e.g., "rustc 1.85.0")
         * @param rustbridgeVersion rustbridge CLI version
         * @param git               git repository info
         */
        public record BuildInfo(
                @JsonProperty("built_by") String builtBy,
                @JsonProperty("built_at") String builtAt,
                String host,
                String compiler,
                @JsonProperty("rustbridge_version") String rustbridgeVersion,
                GitInfo git
        ) {}

        /**
         * Dependency information for SBOM.
         *
         * @param name    dependency name
         * @param version dependency version
         * @param license license identifier
         * @param source  source URL
         */
        public record DependencyInfo(
                String name,
                String version,
                String license,
                String source
        ) {}

        /**
         * Software Bill of Materials (SBOM) information.
         *
         * @param format       SBOM format (e.g., "simplified", "cyclonedx-1.5")
         * @param path         path to full SBOM file in bundle (optional)
         * @param dependencies inline simplified dependency list
         */
        public record Sbom(
                String format,
                String path,
                List<DependencyInfo> dependencies
        ) {}
    }
}
