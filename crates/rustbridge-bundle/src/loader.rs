//! Bundle loading utilities.
//!
//! The [`BundleLoader`] provides functionality to load and extract plugin bundles.

use crate::builder::verify_sha256;
use crate::{BundleError, BundleResult, MANIFEST_FILE, Manifest, Platform, VariantInfo};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

/// Loader for plugin bundles.
///
/// # Example
///
/// ```no_run
/// use rustbridge_bundle::loader::BundleLoader;
///
/// let mut loader = BundleLoader::open("my-plugin-1.0.0.rbp")?;
/// let manifest = loader.manifest();
///
/// // Extract library for current platform to temp directory
/// let library_path = loader.extract_library_for_current_platform("/tmp/plugins")?;
/// # Ok::<(), rustbridge_bundle::BundleError>(())
/// ```
#[derive(Debug)]
pub struct BundleLoader {
    archive: ZipArchive<File>,
    manifest: Manifest,
}

impl BundleLoader {
    /// Open a bundle file for reading.
    pub fn open<P: AsRef<Path>>(path: P) -> BundleResult<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        // Read and parse manifest
        let manifest = {
            let mut manifest_file = archive.by_name(MANIFEST_FILE).map_err(|_| {
                BundleError::MissingFile(format!("{MANIFEST_FILE} not found in bundle"))
            })?;

            let mut manifest_json = String::new();
            manifest_file.read_to_string(&mut manifest_json)?;
            Manifest::from_json(&manifest_json)?
        };

        // Validate manifest
        manifest.validate()?;

        Ok(Self { archive, manifest })
    }

    /// Get the bundle manifest.
    #[must_use]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Check if the bundle supports the current platform.
    #[must_use]
    pub fn supports_current_platform(&self) -> bool {
        Platform::current()
            .map(|p| self.manifest.supports_platform(p))
            .unwrap_or(false)
    }

    /// Get the platform info for the current platform.
    #[must_use]
    pub fn current_platform_info(&self) -> Option<&crate::PlatformInfo> {
        Platform::current().and_then(|p| self.manifest.get_platform(p))
    }

    /// Extract the library for a specific platform to a directory.
    ///
    /// Extracts the release variant (default). For other variants,
    /// use `extract_library_variant` instead.
    ///
    /// Returns the path to the extracted library file.
    pub fn extract_library<P: AsRef<Path>>(
        &mut self,
        platform: Platform,
        output_dir: P,
    ) -> BundleResult<PathBuf> {
        self.extract_library_variant(platform, "release", output_dir)
    }

    /// Extract a specific variant of the library for a platform.
    ///
    /// Returns the path to the extracted library file.
    pub fn extract_library_variant<P: AsRef<Path>>(
        &mut self,
        platform: Platform,
        variant: &str,
        output_dir: P,
    ) -> BundleResult<PathBuf> {
        let library_path = self.variant_info(platform, variant)?.library.clone();
        let contents = self.read_variant(platform, variant)?;

        write_library(output_dir.as_ref(), &library_path, &contents)
    }

    /// Read a variant's library (or component) into memory.
    ///
    /// The bytes are checked against the checksum in the manifest. No
    /// signature is checked; use [`BundleLoader::read_variant_verified`] for
    /// that.
    pub fn read_variant(&mut self, platform: Platform, variant: &str) -> BundleResult<Vec<u8>> {
        let variant_info = self.variant_info(platform, variant)?;
        let library_path = variant_info.library.clone();
        let expected_checksum = variant_info.checksum.clone();

        let contents = self.read_file(&library_path).map_err(|_| {
            BundleError::MissingFile(format!("Library not found in bundle: {library_path}"))
        })?;

        if !verify_sha256(&contents, &expected_checksum) {
            let actual = crate::builder::compute_sha256(&contents);
            return Err(BundleError::ChecksumMismatch {
                path: library_path,
                expected: expected_checksum,
                actual: format!("sha256:{actual}"),
            });
        }

        Ok(contents)
    }

    /// Read a variant's library (or component) into memory, verified against
    /// operator-trusted keys.
    ///
    /// Checks, in order: the manifest signature against `trusted_keys`, the
    /// checksum, and the library signature against the key that verified the
    /// manifest. The manifest's own `public_key` is never used, so a bundle
    /// cannot vouch for itself.
    ///
    /// Returns [`BundleError::NoPublicKey`] when `trusted_keys` is empty and
    /// [`BundleError::SignatureVerificationFailed`] when the bundle is
    /// unsigned or no trusted key verifies it.
    pub fn read_variant_verified(
        &mut self,
        platform: Platform,
        variant: &str,
        trusted_keys: &[&str],
    ) -> BundleResult<Vec<u8>> {
        let key_index = self.verify_manifest_signature_trusted(trusted_keys)?;
        let library_path = self.variant_info(platform, variant)?.library.clone();
        let contents = self.read_variant(platform, variant)?;
        self.verify_library_signature_with_key(
            &library_path,
            &contents,
            Some(trusted_keys[key_index]),
        )?;
        Ok(contents)
    }

    /// Verify the manifest signature against operator-trusted keys.
    ///
    /// Tries each key in turn and returns the index of the first one that
    /// verifies the signature. The manifest's own `public_key` is ignored:
    /// a key carried inside the bundle proves integrity, not authorship.
    pub fn verify_manifest_signature_trusted(
        &mut self,
        trusted_keys: &[&str],
    ) -> BundleResult<usize> {
        if trusted_keys.is_empty() {
            return Err(BundleError::NoPublicKey);
        }

        let manifest_data = self.read_file(MANIFEST_FILE)?;
        let signature = self.read_signature(MANIFEST_FILE)?;

        trusted_keys
            .iter()
            .position(|key| verify_minisign_signature(key, &manifest_data, &signature).is_ok())
            .ok_or_else(|| {
                BundleError::SignatureVerificationFailed(
                    "manifest is not signed by any trusted key".to_string(),
                )
            })
    }

    /// Look up a variant in the manifest.
    fn variant_info(&self, platform: Platform, variant: &str) -> BundleResult<&VariantInfo> {
        self.manifest
            .get_platform(platform)
            .ok_or_else(|| {
                BundleError::UnsupportedPlatform(format!(
                    "Platform {} not found in bundle",
                    platform.as_str()
                ))
            })?
            .variant(variant)
            .ok_or_else(|| BundleError::VariantNotFound {
                platform: platform.as_str().to_string(),
                variant: variant.to_string(),
            })
    }

    /// Read the minisign signature stored next to `path`.
    fn read_signature(&mut self, path: &str) -> BundleResult<String> {
        let sig_path = format!("{path}.minisig");
        if !self.has_file(&sig_path) {
            return Err(BundleError::SignatureVerificationFailed(format!(
                "bundle is not signed: {sig_path} is missing"
            )));
        }
        let sig_data = self.read_file(&sig_path)?;
        String::from_utf8(sig_data).map_err(|e| {
            BundleError::SignatureVerificationFailed(format!("Invalid signature encoding: {e}"))
        })
    }
    /// List all available variants for a platform.
    #[must_use]
    pub fn list_variants(&self, platform: Platform) -> Vec<&str> {
        self.manifest.list_variants(platform)
    }

    /// Check if a specific variant exists for a platform.
    #[must_use]
    pub fn has_variant(&self, platform: Platform, variant: &str) -> bool {
        self.manifest
            .get_platform(platform)
            .map(|p| p.has_variant(variant))
            .unwrap_or(false)
    }

    /// Get the top-level build info from the manifest.
    #[must_use]
    pub fn build_info(&self) -> Option<&crate::BuildInfo> {
        self.manifest.get_build_info()
    }

    /// Get the top-level SBOM info from the manifest.
    #[must_use]
    pub fn sbom(&self) -> Option<&crate::Sbom> {
        self.manifest.get_sbom()
    }

    /// Get the effective build info for a platform/variant (variant-level overrides top-level).
    #[must_use]
    pub fn effective_build_info(&self, platform: &str, variant: &str) -> Option<&crate::BuildInfo> {
        self.manifest.get_effective_build_info(platform, variant)
    }

    /// Get the effective SBOM for a platform/variant (variant-level overrides top-level).
    #[must_use]
    pub fn effective_sbom(&self, platform: &str, variant: &str) -> Option<&crate::Sbom> {
        self.manifest.get_effective_sbom(platform, variant)
    }

    /// Extract the library for the current platform to a directory.
    ///
    /// Returns the path to the extracted library file.
    pub fn extract_library_for_current_platform<P: AsRef<Path>>(
        &mut self,
        output_dir: P,
    ) -> BundleResult<PathBuf> {
        let platform = Platform::current().ok_or_else(|| {
            BundleError::UnsupportedPlatform("Current platform is not supported".to_string())
        })?;

        self.extract_library(platform, output_dir)
    }

    /// Read a file from the bundle as bytes.
    pub fn read_file(&mut self, path: &str) -> BundleResult<Vec<u8>> {
        let mut file = self
            .archive
            .by_name(path)
            .map_err(|_| BundleError::MissingFile(format!("File not found in bundle: {path}")))?;

        let mut contents = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }

    /// Read a file listed in the manifest's `files` map (v1.2), checked
    /// against its recorded checksum. Verify the manifest's signature first
    /// for the checksum to mean anything.
    ///
    /// Returns [`BundleError::Unverified`] for a file the map does not list
    /// and [`BundleError::ChecksumMismatch`] for one whose contents differ.
    pub fn read_file_verified(&mut self, path: &str) -> BundleResult<Vec<u8>> {
        let expected = self
            .manifest
            .files
            .get(path)
            .cloned()
            .ok_or_else(|| BundleError::Unverified(path.to_string()))?;
        let contents = self.read_file(path)?;
        if !verify_sha256(&contents, &expected) {
            let actual = crate::builder::compute_sha256(&contents);
            return Err(BundleError::ChecksumMismatch {
                path: path.to_string(),
                expected,
                actual: format!("sha256:{actual}"),
            });
        }
        Ok(contents)
    }

    /// Read a file from the bundle as a string.
    pub fn read_file_string(&mut self, path: &str) -> BundleResult<String> {
        let mut file = self
            .archive
            .by_name(path)
            .map_err(|_| BundleError::MissingFile(format!("File not found in bundle: {path}")))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    /// List all files in the bundle. An entry whose name cannot be decoded
    /// is left out.
    #[must_use]
    pub fn list_files(&self) -> Vec<String> {
        (0..self.archive.len())
            .filter_map(|i| self.archive.name_for_index(i)?.ok())
            .map(std::borrow::Cow::into_owned)
            .collect()
    }

    /// Check if a file exists in the bundle.
    #[must_use]
    pub fn has_file(&self, path: &str) -> bool {
        self.archive.index_for_name(path).is_some()
    }

    /// Check if the bundle has signature files.
    #[must_use]
    pub fn has_signatures(&self) -> bool {
        self.has_file("manifest.json.minisig")
    }

    /// Get the public key from the manifest.
    #[must_use]
    pub fn public_key(&self) -> Option<&str> {
        self.manifest.public_key.as_deref()
    }

    /// Verify the manifest signature.
    ///
    /// Returns Ok(()) if verification succeeds, or an error if it fails.
    /// If no public key is available, returns `BundleError::NoPublicKey`.
    ///
    /// This uses the `public_key` inside the manifest, which proves the
    /// bundle was not altered after signing but not who signed it. To check
    /// authorship, use [`BundleLoader::verify_manifest_signature_trusted`].
    pub fn verify_manifest_signature(&mut self) -> BundleResult<()> {
        self.verify_manifest_signature_with_key(None)
    }

    /// Verify the manifest signature with an optional key override.
    ///
    /// # Arguments
    ///
    /// * `public_key_override` - If provided, use this key instead of the manifest's key
    pub fn verify_manifest_signature_with_key(
        &mut self,
        public_key_override: Option<&str>,
    ) -> BundleResult<()> {
        // Clone the public key to avoid borrow issues
        let public_key = public_key_override
            .map(String::from)
            .or_else(|| self.manifest.public_key.clone())
            .ok_or(BundleError::NoPublicKey)?;

        // Read manifest data
        let manifest_data = self.read_file(MANIFEST_FILE)?;

        // Read signature
        let sig_path = format!("{MANIFEST_FILE}.minisig");
        let sig_data = self.read_file(&sig_path)?;
        let signature = String::from_utf8(sig_data).map_err(|e| {
            BundleError::SignatureVerificationFailed(format!("Invalid signature encoding: {e}"))
        })?;

        // Verify
        verify_minisign_signature(&public_key, &manifest_data, &signature)
    }

    /// Verify a library signature.
    ///
    /// # Arguments
    ///
    /// * `library_path` - Path to the library within the bundle
    /// * `library_data` - The library file contents
    pub fn verify_library_signature(
        &mut self,
        library_path: &str,
        library_data: &[u8],
    ) -> BundleResult<()> {
        self.verify_library_signature_with_key(library_path, library_data, None)
    }

    /// Verify a library signature with an optional key override.
    pub fn verify_library_signature_with_key(
        &mut self,
        library_path: &str,
        library_data: &[u8],
        public_key_override: Option<&str>,
    ) -> BundleResult<()> {
        // Clone the public key to avoid borrow issues
        let public_key = public_key_override
            .map(String::from)
            .or_else(|| self.manifest.public_key.clone())
            .ok_or(BundleError::NoPublicKey)?;

        // Read signature
        let sig_path = format!("{library_path}.minisig");
        let sig_data = self.read_file(&sig_path)?;
        let signature = String::from_utf8(sig_data).map_err(|e| {
            BundleError::SignatureVerificationFailed(format!("Invalid signature encoding: {e}"))
        })?;

        // Verify
        verify_minisign_signature(&public_key, library_data, &signature)
    }

    /// Extract and verify a library for a platform.
    ///
    /// This method extracts the library and verifies both the checksum and signature.
    ///
    /// # Arguments
    ///
    /// * `platform` - Target platform
    /// * `output_dir` - Directory to extract to
    /// * `verify_signature` - Whether to verify the signature
    /// * `public_key_override` - Optional public key override
    pub fn extract_library_verified<P: AsRef<Path>>(
        &mut self,
        platform: Platform,
        output_dir: P,
        verify_signature: bool,
        public_key_override: Option<&str>,
    ) -> BundleResult<PathBuf> {
        // Verify manifest signature first if requested
        if verify_signature {
            self.verify_manifest_signature_with_key(public_key_override)?;
        }

        let library_path = self.variant_info(platform, "release")?.library.clone();
        let contents = self.read_variant(platform, "release")?;

        // Verify library signature if requested
        if verify_signature {
            self.verify_library_signature_with_key(&library_path, &contents, public_key_override)?;
        }

        write_library(output_dir.as_ref(), &library_path, &contents)
    }
}

/// Write library bytes into `output_dir`, named after the file in the bundle.
fn write_library(output_dir: &Path, library_path: &str, contents: &[u8]) -> BundleResult<PathBuf> {
    fs::create_dir_all(output_dir)?;

    let file_name = Path::new(library_path)
        .file_name()
        .ok_or_else(|| BundleError::InvalidManifest("Invalid library path".to_string()))?;

    let output_path = output_dir.join(file_name);

    fs::write(&output_path, contents)?;

    // Set executable permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&output_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&output_path, perms)?;
    }

    Ok(output_path)
}

/// Verify a minisign signature.
///
/// Minisign format:
/// - Public key: 2 bytes algorithm ID ("Ed") + 8 bytes key ID + 32 bytes Ed25519 key
/// - Signature: 2 bytes algorithm ID + 8 bytes key ID + 64 bytes signature
///   - "ED" (0x45, 0x44) = prehashed with BLAKE2b-512
///   - "Ed" (0x45, 0x64) = legacy non-prehashed
fn verify_minisign_signature(
    public_key_base64: &str,
    data: &[u8],
    signature_content: &str,
) -> BundleResult<()> {
    use minisign::{PublicKey, SignatureBox};

    // Parse public key
    let public_key = PublicKey::from_base64(public_key_base64).map_err(|e| {
        BundleError::SignatureVerificationFailed(format!("Invalid public key: {e}"))
    })?;

    // Parse signature (second line of the minisig file)
    let signature_box = SignatureBox::from_string(signature_content).map_err(|e| {
        BundleError::SignatureVerificationFailed(format!("Invalid signature format: {e}"))
    })?;

    // Verify - minisign::verify handles prehashing automatically
    let mut data_reader = std::io::Cursor::new(data);
    minisign::verify(
        &public_key,
        &signature_box,
        &mut data_reader,
        true,
        false,
        false,
    )
    .map_err(|e| {
        BundleError::SignatureVerificationFailed(format!("Signature verification failed: {e}"))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use crate::builder::{BundleBuilder, compute_sha256};
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_bundle(temp_dir: &TempDir) -> PathBuf {
        let bundle_path = temp_dir.path().join("test.rbp");

        // Create a fake library file
        let lib_path = temp_dir.path().join("libtest.so");
        fs::write(&lib_path, b"fake library contents").unwrap();

        // Build the bundle
        let manifest = Manifest::new("test-plugin", "1.0.0");
        BundleBuilder::new(manifest)
            .add_library(Platform::LinuxX86_64, &lib_path)
            .unwrap()
            .add_bytes("schema/messages.h", b"// header".to_vec())
            .write(&bundle_path)
            .unwrap();

        bundle_path
    }

    fn create_multi_platform_bundle(temp_dir: &TempDir) -> PathBuf {
        let bundle_path = temp_dir.path().join("multi.rbp");

        let linux_lib = temp_dir.path().join("libtest.so");
        let macos_lib = temp_dir.path().join("libtest.dylib");
        let windows_lib = temp_dir.path().join("test.dll");
        fs::write(&linux_lib, b"linux library").unwrap();
        fs::write(&macos_lib, b"macos library").unwrap();
        fs::write(&windows_lib, b"windows library").unwrap();

        let manifest = Manifest::new("multi-platform", "2.0.0");
        BundleBuilder::new(manifest)
            .add_library(Platform::LinuxX86_64, &linux_lib)
            .unwrap()
            .add_library(Platform::DarwinAarch64, &macos_lib)
            .unwrap()
            .add_library(Platform::WindowsX86_64, &windows_lib)
            .unwrap()
            .write(&bundle_path)
            .unwrap();

        bundle_path
    }

    #[test]
    fn BundleLoader___open___reads_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);

        let loader = BundleLoader::open(&bundle_path).unwrap();

        assert_eq!(loader.manifest().plugin.name, "test-plugin");
        assert_eq!(loader.manifest().plugin.version, "1.0.0");
    }

    #[test]
    fn BundleLoader___open___nonexistent_file___returns_error() {
        let result = BundleLoader::open("/nonexistent/bundle.rbp");

        assert!(result.is_err());
    }

    #[test]
    fn BundleLoader___open___not_a_zip___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let fake_bundle = temp_dir.path().join("fake.rbp");
        fs::write(&fake_bundle, b"not a zip file").unwrap();

        let result = BundleLoader::open(&fake_bundle);

        assert!(result.is_err());
    }

    #[test]
    fn BundleLoader___open___missing_manifest___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path().join("no-manifest.rbp");

        // Create a ZIP without manifest.json
        let file = File::create(&bundle_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("some-file.txt", options).unwrap();
        zip.write_all(b"content").unwrap();
        zip.finish().unwrap();

        let result = BundleLoader::open(&bundle_path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BundleError::MissingFile(_)));
        assert!(err.to_string().contains("manifest.json"));
    }

    #[test]
    fn BundleLoader___open___invalid_manifest_json___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path().join("bad-manifest.rbp");

        // Create a ZIP with invalid JSON in manifest
        let file = File::create(&bundle_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(b"{ invalid json }").unwrap();
        zip.finish().unwrap();

        let result = BundleLoader::open(&bundle_path);

        assert!(result.is_err());
    }

    #[test]
    fn BundleLoader___list_files___returns_all_files() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);

        let loader = BundleLoader::open(&bundle_path).unwrap();
        let files = loader.list_files();

        assert!(files.contains(&"manifest.json".to_string()));
        assert!(files.contains(&"schema/messages.h".to_string()));
    }

    #[test]
    fn BundleLoader___has_file___returns_true_for_existing() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);

        let loader = BundleLoader::open(&bundle_path).unwrap();

        assert!(loader.has_file("manifest.json"));
        assert!(loader.has_file("schema/messages.h"));
        assert!(!loader.has_file("nonexistent.txt"));
    }

    #[test]
    fn BundleLoader___read_file___returns_contents() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);

        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let contents = loader.read_file_string("schema/messages.h").unwrap();

        assert_eq!(contents, "// header");
    }

    #[test]
    fn BundleLoader___read_file___missing_file___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);

        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let result = loader.read_file("nonexistent.txt");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BundleError::MissingFile(_)));
    }

    #[test]
    fn BundleLoader___read_file___returns_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);

        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let contents = loader.read_file("schema/messages.h").unwrap();

        assert_eq!(contents, b"// header");
    }

    #[test]
    fn BundleLoader___extract_library___verifies_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);
        let extract_dir = temp_dir.path().join("extracted");

        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let lib_path = loader
            .extract_library(Platform::LinuxX86_64, &extract_dir)
            .unwrap();

        assert!(lib_path.exists());
        let contents = fs::read(&lib_path).unwrap();
        assert_eq!(contents, b"fake library contents");
    }

    #[test]
    fn BundleLoader___extract_library___unsupported_platform___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);
        let extract_dir = temp_dir.path().join("extracted");

        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let result = loader.extract_library(Platform::WindowsX86_64, &extract_dir);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BundleError::UnsupportedPlatform(_)));
    }

    #[test]
    fn BundleLoader___extract_library___creates_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);
        let extract_dir = temp_dir.path().join("deep").join("nested").join("dir");

        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let lib_path = loader
            .extract_library(Platform::LinuxX86_64, &extract_dir)
            .unwrap();

        assert!(extract_dir.exists());
        assert!(lib_path.exists());
    }

    #[test]
    fn BundleLoader___multi_platform___extract_each_platform() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_multi_platform_bundle(&temp_dir);

        let mut loader = BundleLoader::open(&bundle_path).unwrap();

        // Verify all three platforms are supported
        assert!(loader.manifest().supports_platform(Platform::LinuxX86_64));
        assert!(loader.manifest().supports_platform(Platform::DarwinAarch64));
        assert!(loader.manifest().supports_platform(Platform::WindowsX86_64));

        // Extract Linux
        let linux_dir = temp_dir.path().join("linux");
        let linux_lib = loader
            .extract_library(Platform::LinuxX86_64, &linux_dir)
            .unwrap();
        assert_eq!(fs::read(&linux_lib).unwrap(), b"linux library");

        // Extract macOS
        let macos_dir = temp_dir.path().join("macos");
        let macos_lib = loader
            .extract_library(Platform::DarwinAarch64, &macos_dir)
            .unwrap();
        assert_eq!(fs::read(&macos_lib).unwrap(), b"macos library");

        // Extract Windows
        let windows_dir = temp_dir.path().join("windows");
        let windows_lib = loader
            .extract_library(Platform::WindowsX86_64, &windows_dir)
            .unwrap();
        assert_eq!(fs::read(&windows_lib).unwrap(), b"windows library");
    }

    #[test]
    fn BundleLoader___supports_current_platform___returns_correct_value() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);

        let loader = BundleLoader::open(&bundle_path).unwrap();

        // This test will pass on Linux x86_64, fail on other platforms
        // which is expected behavior
        if Platform::current() == Some(Platform::LinuxX86_64) {
            assert!(loader.supports_current_platform());
        }
    }

    #[test]
    fn BundleLoader___current_platform_info___returns_info_when_supported() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);

        let loader = BundleLoader::open(&bundle_path).unwrap();

        if Platform::current() == Some(Platform::LinuxX86_64) {
            let info = loader.current_platform_info();
            assert!(info.is_some());
            let release = info.unwrap().release().unwrap();
            assert!(release.library.contains("libtest.so"));
        }
    }

    #[test]
    fn roundtrip___create_and_load___preserves_all_data() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path().join("roundtrip.rbp");

        // Create library
        let lib_path = temp_dir.path().join("libplugin.so");
        let lib_contents = b"roundtrip test library";
        fs::write(&lib_path, lib_contents).unwrap();

        // Create bundle with metadata
        let mut manifest = Manifest::new("roundtrip-plugin", "3.2.1");
        manifest.plugin.description = Some("A test plugin for roundtrip".to_string());
        manifest.plugin.authors = vec!["Author One".to_string(), "Author Two".to_string()];
        manifest.plugin.license = Some("MIT".to_string());

        BundleBuilder::new(manifest)
            .add_library(Platform::LinuxX86_64, &lib_path)
            .unwrap()
            .add_bytes("docs/README.md", b"# Documentation".to_vec())
            .write(&bundle_path)
            .unwrap();

        // Load and verify
        let mut loader = BundleLoader::open(&bundle_path).unwrap();

        assert_eq!(loader.manifest().plugin.name, "roundtrip-plugin");
        assert_eq!(loader.manifest().plugin.version, "3.2.1");
        assert_eq!(
            loader.manifest().plugin.description,
            Some("A test plugin for roundtrip".to_string())
        );
        assert_eq!(loader.manifest().plugin.authors.len(), 2);
        assert_eq!(loader.manifest().plugin.license, Some("MIT".to_string()));

        // Verify checksum in manifest matches actual content
        let platform_info = loader
            .manifest()
            .get_platform(Platform::LinuxX86_64)
            .unwrap();
        let release = platform_info.release().unwrap();
        let expected_checksum = format!("sha256:{}", compute_sha256(lib_contents));
        assert_eq!(release.checksum, expected_checksum);

        // Verify library extraction
        let extract_dir = temp_dir.path().join("extract");
        let extracted = loader
            .extract_library(Platform::LinuxX86_64, &extract_dir)
            .unwrap();
        assert_eq!(fs::read(&extracted).unwrap(), lib_contents);
    }

    #[test]
    fn roundtrip___bundle_with_schemas___preserves_schema_info() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path().join("schema-bundle.rbp");

        let lib_path = temp_dir.path().join("libtest.so");
        let header_path = temp_dir.path().join("messages.h");
        let json_path = temp_dir.path().join("schema.json");
        fs::write(&lib_path, b"lib").unwrap();
        fs::write(&header_path, b"#pragma once").unwrap();
        fs::write(&json_path, b"{}").unwrap();

        let manifest = Manifest::new("schema-plugin", "1.0.0");
        BundleBuilder::new(manifest)
            .add_library(Platform::LinuxX86_64, &lib_path)
            .unwrap()
            .add_schema_file(&header_path, "messages.h")
            .unwrap()
            .add_schema_file(&json_path, "schema.json")
            .unwrap()
            .write(&bundle_path)
            .unwrap();

        let mut loader = BundleLoader::open(&bundle_path).unwrap();

        // Verify schemas are in manifest
        assert_eq!(loader.manifest().schemas.len(), 2);
        assert!(loader.manifest().schemas.contains_key("messages.h"));
        assert!(loader.manifest().schemas.contains_key("schema.json"));

        // Verify schema files can be read
        assert!(loader.has_file("schema/messages.h"));
        assert!(loader.has_file("schema/schema.json"));
        assert_eq!(
            loader.read_file_string("schema/messages.h").unwrap(),
            "#pragma once"
        );
    }

    // --- v1.2: components, in-memory reads, trusted keys ---

    const WORLD: &str = "paddock:runtime/app@0.1.0";

    fn create_component_bundle(temp_dir: &TempDir, signing: Option<&minisign::KeyPair>) -> PathBuf {
        let bundle_path = temp_dir.path().join("component.rbp");
        let component_path = temp_dir.path().join("app.wasm");
        fs::write(&component_path, b"\0asm fake component").unwrap();

        let mut builder = BundleBuilder::new(Manifest::new("app", "1.0.0"))
            .add_component(Platform::Wasm32Wasip2, &component_path, WORLD)
            .unwrap()
            .with_extension("appserver", serde_json::json!({ "kind": "app" }));
        if let Some(keypair) = signing {
            builder = builder.with_signing_key(keypair.pk.to_base64(), keypair.sk.clone());
        }
        builder.write(&bundle_path).unwrap();

        bundle_path
    }

    #[test]
    fn BundleLoader___open___component_bundle___keeps_component_and_extension() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_component_bundle(&temp_dir, None);

        let loader = BundleLoader::open(&bundle_path).unwrap();

        let manifest = loader.manifest();
        assert_eq!(manifest.bundle_version, crate::BUNDLE_VERSION_1_2);
        let variant = manifest
            .get_release_variant(Platform::Wasm32Wasip2)
            .unwrap();
        assert_eq!(variant.component.as_ref().unwrap().world, WORLD);
        assert_eq!(variant.library, "lib/wasm32-wasip2/release/app.wasm");
        assert_eq!(
            manifest.extension("appserver"),
            Some(&serde_json::json!({ "kind": "app" }))
        );
    }

    #[test]
    fn BundleLoader___read_variant___valid_checksum___returns_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_component_bundle(&temp_dir, None);
        let mut loader = BundleLoader::open(&bundle_path).unwrap();

        let bytes = loader
            .read_variant(Platform::Wasm32Wasip2, "release")
            .unwrap();

        assert_eq!(bytes, b"\0asm fake component");
    }

    #[test]
    fn BundleLoader___read_variant___checksum_mismatch___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = temp_dir.path().join("bad.rbp");
        let path = "lib/wasm32-wasip2/release/app.wasm";
        let mut manifest = Manifest::new("app", "1.0.0");
        manifest.add_component_variant(
            Platform::Wasm32Wasip2,
            "release",
            path,
            &compute_sha256(b"original"),
            WORLD,
        );
        BundleBuilder::new(manifest)
            .add_bytes(path, b"tampered".to_vec())
            .write(&bundle_path)
            .unwrap();
        let mut loader = BundleLoader::open(&bundle_path).unwrap();

        let result = loader.read_variant(Platform::Wasm32Wasip2, "release");

        assert!(matches!(result, Err(BundleError::ChecksumMismatch { .. })));
    }

    #[test]
    fn BundleLoader___read_variant___unknown_variant___returns_variant_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_component_bundle(&temp_dir, None);
        let mut loader = BundleLoader::open(&bundle_path).unwrap();

        let result = loader.read_variant(Platform::Wasm32Wasip2, "debug");

        assert!(matches!(result, Err(BundleError::VariantNotFound { .. })));
    }

    #[test]
    fn BundleLoader___read_variant_verified___trusted_key___returns_bytes() {
        let temp_dir = TempDir::new().unwrap();
        let keypair = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let bundle_path = create_component_bundle(&temp_dir, Some(&keypair));
        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let trusted = keypair.pk.to_base64();

        let bytes = loader
            .read_variant_verified(Platform::Wasm32Wasip2, "release", &[&trusted])
            .unwrap();

        assert_eq!(bytes, b"\0asm fake component");
    }

    #[test]
    fn BundleLoader___read_variant_verified___signed_by_untrusted_key___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let signer = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let operator = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let bundle_path = create_component_bundle(&temp_dir, Some(&signer));
        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let trusted = operator.pk.to_base64();

        let result = loader.read_variant_verified(Platform::Wasm32Wasip2, "release", &[&trusted]);

        assert!(matches!(
            result,
            Err(BundleError::SignatureVerificationFailed(_))
        ));
    }

    #[test]
    fn BundleLoader___read_variant_verified___unsigned_bundle___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let operator = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let bundle_path = create_component_bundle(&temp_dir, None);
        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let trusted = operator.pk.to_base64();

        let result = loader.read_variant_verified(Platform::Wasm32Wasip2, "release", &[&trusted]);

        match result {
            Err(BundleError::SignatureVerificationFailed(msg)) => {
                assert!(msg.contains("not signed"));
            }
            other => panic!("expected SignatureVerificationFailed, got {other:?}"),
        }
    }

    #[test]
    fn BundleLoader___read_variant_verified___no_trusted_keys___returns_no_public_key() {
        let temp_dir = TempDir::new().unwrap();
        let keypair = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let bundle_path = create_component_bundle(&temp_dir, Some(&keypair));
        let mut loader = BundleLoader::open(&bundle_path).unwrap();

        let result = loader.read_variant_verified(Platform::Wasm32Wasip2, "release", &[]);

        assert!(matches!(result, Err(BundleError::NoPublicKey)));
    }

    #[test]
    fn BundleLoader___verify_manifest_signature_trusted___second_key_matches___returns_its_index() {
        let temp_dir = TempDir::new().unwrap();
        let signer = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let other = minisign::KeyPair::generate_unencrypted_keypair().unwrap();
        let bundle_path = create_component_bundle(&temp_dir, Some(&signer));
        let mut loader = BundleLoader::open(&bundle_path).unwrap();
        let keys = [other.pk.to_base64(), signer.pk.to_base64()];

        let index = loader
            .verify_manifest_signature_trusted(&[&keys[0], &keys[1]])
            .unwrap();

        assert_eq!(index, 1);
    }

    #[test]
    fn BundleLoader___read_file_verified___listed_file___returns_contents() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("files.rbp");
        BundleBuilder::new(Manifest::new("app", "1.0.0"))
            .add_component(
                Platform::Wasm32Wasip2,
                {
                    let p = temp_dir.path().join("a.wasm");
                    fs::write(&p, b"\0asm").unwrap();
                    p
                },
                "w:w/w@0.1.0",
            )
            .unwrap()
            .add_checksummed_bytes("priv/data.txt", b"hello".to_vec())
            .write(&path)
            .unwrap();
        let mut loader = BundleLoader::open(&path).unwrap();

        let contents = loader.read_file_verified("priv/data.txt").unwrap();

        assert_eq!(contents, b"hello");
        assert!(loader.manifest().files["priv/data.txt"].starts_with("sha256:"));
    }

    #[test]
    fn BundleLoader___read_file_verified___unlisted_file___unverified() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);
        let mut loader = BundleLoader::open(&bundle_path).unwrap();

        let result = loader.read_file_verified("schema/messages.h");

        assert!(matches!(result, Err(BundleError::Unverified(_))));
    }

    #[test]
    fn BundleLoader___read_file_verified___contents_differ___checksum_mismatch() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("tampered.rbp");
        let mut manifest = Manifest::new("app", "1.0.0");
        manifest.add_component_variant(
            Platform::Wasm32Wasip2,
            "release",
            "lib/a.wasm",
            &compute_sha256(b"\0asm"),
            "w:w/w@0.1.0",
        );
        manifest.files.insert(
            "priv/x".into(),
            format!("sha256:{}", compute_sha256(b"original")),
        );
        BundleBuilder::new(manifest)
            .add_bytes("lib/a.wasm", b"\0asm".to_vec())
            .add_bytes("priv/x", b"changed".to_vec())
            .write(&path)
            .unwrap();
        let mut loader = BundleLoader::open(&path).unwrap();

        let result = loader.read_file_verified("priv/x");

        assert!(matches!(result, Err(BundleError::ChecksumMismatch { .. })));
    }
}
