//! Bundle creation utilities.
//!
//! The [`BundleBuilder`] provides a fluent API for creating `.rbp` bundle archives.

use crate::{BundleError, BundleResult, MANIFEST_FILE, Manifest, Platform};
use minisign::SecretKey;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

/// Builder for creating plugin bundles.
///
/// # Example
///
/// ```no_run
/// use rustbridge_bundle::{BundleBuilder, Manifest, Platform};
///
/// let manifest = Manifest::new("my-plugin", "1.0.0");
/// let builder = BundleBuilder::new(manifest)
///     .add_library(Platform::LinuxX86_64, "target/release/libmyplugin.so")?
///     .add_schema_file("schema/messages.h", "include/messages.h")?;
///
/// builder.write("my-plugin-1.0.0.rbp")?;
/// # Ok::<(), rustbridge_bundle::BundleError>(())
/// ```
#[derive(Debug)]
pub struct BundleBuilder {
    manifest: Manifest,
    files: Vec<BundleFile>,
    signing_key: Option<(String, SecretKey)>, // (public_key_base64, secret_key)
}

/// A file to include in the bundle.
#[derive(Debug)]
struct BundleFile {
    /// Path within the bundle archive.
    archive_path: String,
    /// File contents.
    contents: Vec<u8>,
}

impl BundleBuilder {
    /// Create a new bundle builder with the given manifest.
    #[must_use]
    pub fn new(manifest: Manifest) -> Self {
        Self {
            manifest,
            files: Vec::new(),
            signing_key: None,
        }
    }

    /// Set the signing key for bundle signing.
    ///
    /// The secret key will be used to sign all library files and the manifest.
    /// The corresponding public key will be embedded in the manifest.
    ///
    /// # Arguments
    /// * `public_key_base64` - The public key in base64 format (from the .pub file)
    /// * `secret_key` - The secret key for signing
    pub fn with_signing_key(mut self, public_key_base64: String, secret_key: SecretKey) -> Self {
        self.manifest.set_public_key(public_key_base64.clone());
        self.signing_key = Some((public_key_base64, secret_key));
        self
    }

    /// Add a platform-specific library to the bundle as the release variant.
    ///
    /// This reads the library file, computes its SHA256 checksum,
    /// and updates the manifest with the platform information.
    ///
    /// This is a convenience method that adds the library as the `release` variant.
    /// For other variants, use `add_library_variant` instead.
    pub fn add_library<P: AsRef<Path>>(
        self,
        platform: Platform,
        library_path: P,
    ) -> BundleResult<Self> {
        self.add_library_variant(platform, "release", library_path)
    }

    /// Add a variant-specific library to the bundle.
    ///
    /// This reads the library file, computes its SHA256 checksum,
    /// and updates the manifest with the platform and variant information.
    ///
    /// # Arguments
    /// * `platform` - Target platform
    /// * `variant` - Variant name (e.g., "release", "debug")
    /// * `library_path` - Path to the library file
    pub fn add_library_variant<P: AsRef<Path>>(
        mut self,
        platform: Platform,
        variant: &str,
        library_path: P,
    ) -> BundleResult<Self> {
        let library_path = library_path.as_ref();

        // Read the library file
        let contents = fs::read(library_path).map_err(|e| {
            BundleError::LibraryNotFound(format!("{}: {}", library_path.display(), e))
        })?;

        // Compute SHA256 checksum
        let checksum = compute_sha256(&contents);

        // Determine the archive path (now includes variant)
        let file_name = library_path
            .file_name()
            .ok_or_else(|| {
                BundleError::InvalidManifest(format!(
                    "Invalid library path: {}",
                    library_path.display()
                ))
            })?
            .to_string_lossy();
        let archive_path = format!("lib/{}/{}/{}", platform.as_str(), variant, file_name);

        // Update manifest
        self.manifest
            .add_platform_variant(platform, variant, &archive_path, &checksum, None);

        // Add to files list
        self.files.push(BundleFile {
            archive_path,
            contents,
        });

        Ok(self)
    }

    /// Add a variant-specific library with build metadata.
    ///
    /// Similar to `add_library_variant` but also attaches build metadata
    /// to the variant (e.g., compiler flags, features, etc.).
    pub fn add_library_variant_with_build<P: AsRef<Path>>(
        mut self,
        platform: Platform,
        variant: &str,
        library_path: P,
        build: serde_json::Value,
    ) -> BundleResult<Self> {
        let library_path = library_path.as_ref();

        // Read the library file
        let contents = fs::read(library_path).map_err(|e| {
            BundleError::LibraryNotFound(format!("{}: {}", library_path.display(), e))
        })?;

        // Compute SHA256 checksum
        let checksum = compute_sha256(&contents);

        // Determine the archive path
        let file_name = library_path
            .file_name()
            .ok_or_else(|| {
                BundleError::InvalidManifest(format!(
                    "Invalid library path: {}",
                    library_path.display()
                ))
            })?
            .to_string_lossy();
        let archive_path = format!("lib/{}/{}/{}", platform.as_str(), variant, file_name);

        // Update manifest with build metadata
        self.manifest.add_platform_variant(
            platform,
            variant,
            &archive_path,
            &checksum,
            Some(build),
        );

        // Add to files list
        self.files.push(BundleFile {
            archive_path,
            contents,
        });

        Ok(self)
    }

    /// Add a schema file to the bundle.
    ///
    /// Schema files are stored in the `schema/` directory within the bundle.
    ///
    /// The schema format is automatically detected from the file extension:
    /// - `.h` -> "c-header"
    /// - `.json` -> "json-schema"
    /// - Others -> "unknown"
    pub fn add_schema_file<P: AsRef<Path>>(
        mut self,
        source_path: P,
        archive_name: &str,
    ) -> BundleResult<Self> {
        let source_path = source_path.as_ref();

        let contents = fs::read(source_path).map_err(|e| {
            BundleError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to read schema file {}: {}",
                    source_path.display(),
                    e
                ),
            ))
        })?;

        // Compute checksum
        let checksum = compute_sha256(&contents);

        // Detect format from extension
        let format = detect_schema_format(archive_name);

        let archive_path = format!("schema/{archive_name}");

        // Add to manifest
        self.manifest.add_schema(
            archive_name.to_string(),
            archive_path.clone(),
            format,
            checksum,
            None, // No description by default
        );

        self.files.push(BundleFile {
            archive_path,
            contents,
        });

        Ok(self)
    }

    /// Add a documentation file to the bundle.
    ///
    /// Documentation files are stored in the `docs/` directory within the bundle.
    pub fn add_doc_file<P: AsRef<Path>>(
        mut self,
        source_path: P,
        archive_name: &str,
    ) -> BundleResult<Self> {
        let source_path = source_path.as_ref();

        let contents = fs::read(source_path).map_err(|e| {
            BundleError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read doc file {}: {}", source_path.display(), e),
            ))
        })?;

        let archive_path = format!("docs/{archive_name}");

        self.files.push(BundleFile {
            archive_path,
            contents,
        });

        Ok(self)
    }

    /// Add raw bytes as a file in the bundle.
    pub fn add_bytes(mut self, archive_path: &str, contents: Vec<u8>) -> Self {
        self.files.push(BundleFile {
            archive_path: archive_path.to_string(),
            contents,
        });
        self
    }

    /// Set the build information for the bundle.
    pub fn with_build_info(mut self, build_info: crate::BuildInfo) -> Self {
        self.manifest.set_build_info(build_info);
        self
    }

    /// Set the SBOM paths.
    pub fn with_sbom(mut self, sbom: crate::Sbom) -> Self {
        self.manifest.set_sbom(sbom);
        self
    }

    /// Add a notices file to the bundle.
    ///
    /// The file will be stored in the `docs/` directory.
    pub fn add_notices_file<P: AsRef<Path>>(mut self, source_path: P) -> BundleResult<Self> {
        let source_path = source_path.as_ref();

        let contents = fs::read(source_path).map_err(|e| {
            BundleError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to read notices file {}: {}",
                    source_path.display(),
                    e
                ),
            ))
        })?;

        let archive_path = "docs/NOTICES.txt".to_string();
        self.manifest.set_notices(archive_path.clone());

        self.files.push(BundleFile {
            archive_path,
            contents,
        });

        Ok(self)
    }

    /// Add the plugin's license file to the bundle.
    ///
    /// The file will be stored in the `legal/` directory as `LICENSE`.
    /// This is for the plugin's own license, not third-party notices.
    pub fn add_license_file<P: AsRef<Path>>(mut self, source_path: P) -> BundleResult<Self> {
        let source_path = source_path.as_ref();

        let contents = fs::read(source_path).map_err(|e| {
            BundleError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to read license file {}: {}",
                    source_path.display(),
                    e
                ),
            ))
        })?;

        let archive_path = "legal/LICENSE".to_string();
        self.manifest.set_license_file(archive_path.clone());

        self.files.push(BundleFile {
            archive_path,
            contents,
        });

        Ok(self)
    }

    /// Add an SBOM file to the bundle.
    ///
    /// The file will be stored in the `sbom/` directory.
    pub fn add_sbom_file<P: AsRef<Path>>(
        mut self,
        source_path: P,
        archive_name: &str,
    ) -> BundleResult<Self> {
        let source_path = source_path.as_ref();

        let contents = fs::read(source_path).map_err(|e| {
            BundleError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read SBOM file {}: {}", source_path.display(), e),
            ))
        })?;

        let archive_path = format!("sbom/{archive_name}");

        self.files.push(BundleFile {
            archive_path,
            contents,
        });

        Ok(self)
    }

    /// Write the bundle to a file.
    pub fn write<P: AsRef<Path>>(self, output_path: P) -> BundleResult<()> {
        let output_path = output_path.as_ref();

        // Validate the manifest
        self.manifest.validate()?;

        // Create the ZIP file
        let file = File::create(output_path)?;
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        // Write manifest.json
        let manifest_json = self.manifest.to_json()?;
        zip.start_file(MANIFEST_FILE, options)?;
        zip.write_all(manifest_json.as_bytes())?;

        // Sign and write manifest.json.minisig if signing is enabled
        if let Some((ref _public_key, ref secret_key)) = self.signing_key {
            let signature = sign_data(secret_key, manifest_json.as_bytes())?;
            zip.start_file(format!("{MANIFEST_FILE}.minisig"), options)?;
            zip.write_all(signature.as_bytes())?;
        }

        // Write all other files
        for bundle_file in &self.files {
            zip.start_file(&bundle_file.archive_path, options)?;
            zip.write_all(&bundle_file.contents)?;

            // Sign library files if signing is enabled
            if let Some((ref _public_key, ref secret_key)) = self.signing_key {
                // Only sign library files (in lib/ directory)
                if bundle_file.archive_path.starts_with("lib/") {
                    let signature = sign_data(secret_key, &bundle_file.contents)?;
                    let sig_path = format!("{}.minisig", bundle_file.archive_path);
                    zip.start_file(&sig_path, options)?;
                    zip.write_all(signature.as_bytes())?;
                }
            }
        }

        zip.finish()?;

        Ok(())
    }

    /// Get the current manifest (for inspection).
    #[must_use]
    pub fn manifest(&self) -> &Manifest {
        &self.manifest
    }

    /// Get a mutable reference to the manifest (for modification).
    pub fn manifest_mut(&mut self) -> &mut Manifest {
        &mut self.manifest
    }
}

/// Compute SHA256 hash of data and return as hex string.
pub fn compute_sha256(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

/// Verify SHA256 checksum of data.
pub fn verify_sha256(data: &[u8], expected: &str) -> bool {
    let actual = compute_sha256(data);

    // Handle both "sha256:xxx" and raw "xxx" formats
    let expected_hex = expected.strip_prefix("sha256:").unwrap_or(expected);

    actual == expected_hex
}

/// Detect schema format from file extension.
fn detect_schema_format(filename: &str) -> String {
    if filename.ends_with(".h") || filename.ends_with(".hpp") {
        "c-header".to_string()
    } else if filename.ends_with(".json") {
        "json-schema".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Sign data using a minisign secret key.
///
/// Returns the signature in minisign format (base64-encoded).
fn sign_data(secret_key: &SecretKey, data: &[u8]) -> BundleResult<String> {
    let signature_box = minisign::sign(
        None, // No public key needed for signing
        secret_key, data, None, // No trusted comment
        None, // No untrusted comment
    )
    .map_err(|e| BundleError::Io(std::io::Error::other(format!("Failed to sign data: {e}"))))?;

    Ok(signature_box.to_string())
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use tempfile::TempDir;

    #[test]
    fn compute_sha256___returns_consistent_hash() {
        let data = b"hello world";
        let hash1 = compute_sha256(data);
        let hash2 = compute_sha256(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 is 32 bytes = 64 hex chars
    }

    #[test]
    fn compute_sha256___different_data___different_hash() {
        let hash1 = compute_sha256(b"hello");
        let hash2 = compute_sha256(b"world");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn compute_sha256___empty_data___returns_valid_hash() {
        let hash = compute_sha256(b"");

        assert_eq!(hash.len(), 64);
        // Known SHA256 of empty string
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn verify_sha256___accepts_valid_checksum() {
        let data = b"hello world";
        let checksum = compute_sha256(data);

        assert!(verify_sha256(data, &checksum));
        assert!(verify_sha256(data, &format!("sha256:{checksum}")));
    }

    #[test]
    fn verify_sha256___rejects_invalid_checksum() {
        let data = b"hello world";

        assert!(!verify_sha256(data, "invalid"));
        assert!(!verify_sha256(data, "sha256:invalid"));
    }

    #[test]
    fn verify_sha256___case_sensitive___rejects_uppercase() {
        let data = b"hello world";
        let checksum = compute_sha256(data).to_uppercase();

        assert!(!verify_sha256(data, &checksum));
    }

    #[test]
    fn BundleBuilder___add_bytes___adds_file() {
        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest).add_bytes("test.txt", b"hello".to_vec());

        assert_eq!(builder.files.len(), 1);
        assert_eq!(builder.files[0].archive_path, "test.txt");
        assert_eq!(builder.files[0].contents, b"hello");
    }

    #[test]
    fn BundleBuilder___add_bytes___multiple_files() {
        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest)
            .add_bytes("file1.txt", b"content1".to_vec())
            .add_bytes("file2.txt", b"content2".to_vec())
            .add_bytes("dir/file3.txt", b"content3".to_vec());

        assert_eq!(builder.files.len(), 3);
    }

    #[test]
    fn BundleBuilder___add_library___nonexistent_file___returns_error() {
        let manifest = Manifest::new("test", "1.0.0");
        let result = BundleBuilder::new(manifest)
            .add_library(Platform::LinuxX86_64, "/nonexistent/path/libtest.so");

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BundleError::LibraryNotFound(_)));
        assert!(err.to_string().contains("/nonexistent/path/libtest.so"));
    }

    #[test]
    fn BundleBuilder___add_library___valid_file___computes_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let lib_path = temp_dir.path().join("libtest.so");
        fs::write(&lib_path, b"fake library").unwrap();

        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest)
            .add_library(Platform::LinuxX86_64, &lib_path)
            .unwrap();

        let platform_info = builder
            .manifest
            .get_platform(Platform::LinuxX86_64)
            .unwrap();
        let release = platform_info.release().unwrap();
        assert!(release.checksum.starts_with("sha256:"));
        assert_eq!(release.library, "lib/linux-x86_64/release/libtest.so");
    }

    #[test]
    fn BundleBuilder___add_library_variant___adds_multiple_variants() {
        let temp_dir = TempDir::new().unwrap();
        let release_lib = temp_dir.path().join("libtest_release.so");
        let debug_lib = temp_dir.path().join("libtest_debug.so");
        fs::write(&release_lib, b"release library").unwrap();
        fs::write(&debug_lib, b"debug library").unwrap();

        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest)
            .add_library_variant(Platform::LinuxX86_64, "release", &release_lib)
            .unwrap()
            .add_library_variant(Platform::LinuxX86_64, "debug", &debug_lib)
            .unwrap();

        let platform_info = builder
            .manifest
            .get_platform(Platform::LinuxX86_64)
            .unwrap();

        assert!(platform_info.has_variant("release"));
        assert!(platform_info.has_variant("debug"));

        let release = platform_info.variant("release").unwrap();
        let debug = platform_info.variant("debug").unwrap();

        assert_eq!(
            release.library,
            "lib/linux-x86_64/release/libtest_release.so"
        );
        assert_eq!(debug.library, "lib/linux-x86_64/debug/libtest_debug.so");
    }

    #[test]
    fn BundleBuilder___add_library___multiple_platforms() {
        let temp_dir = TempDir::new().unwrap();

        let linux_lib = temp_dir.path().join("libtest.so");
        let macos_lib = temp_dir.path().join("libtest.dylib");
        fs::write(&linux_lib, b"linux lib").unwrap();
        fs::write(&macos_lib, b"macos lib").unwrap();

        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest)
            .add_library(Platform::LinuxX86_64, &linux_lib)
            .unwrap()
            .add_library(Platform::DarwinAarch64, &macos_lib)
            .unwrap();

        assert!(builder.manifest.supports_platform(Platform::LinuxX86_64));
        assert!(builder.manifest.supports_platform(Platform::DarwinAarch64));
        assert!(!builder.manifest.supports_platform(Platform::WindowsX86_64));
    }

    #[test]
    fn BundleBuilder___add_schema_file___nonexistent___returns_error() {
        let manifest = Manifest::new("test", "1.0.0");
        let result =
            BundleBuilder::new(manifest).add_schema_file("/nonexistent/schema.h", "schema.h");

        assert!(result.is_err());
    }

    #[test]
    fn BundleBuilder___add_schema_file___detects_c_header_format() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("messages.h");
        fs::write(&schema_path, b"#include <stdint.h>").unwrap();

        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest)
            .add_schema_file(&schema_path, "messages.h")
            .unwrap();

        let schema_info = builder.manifest.schemas.get("messages.h").unwrap();
        assert_eq!(schema_info.format, "c-header");
    }

    #[test]
    fn BundleBuilder___add_schema_file___detects_json_schema_format() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("schema.json");
        fs::write(&schema_path, b"{}").unwrap();

        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest)
            .add_schema_file(&schema_path, "schema.json")
            .unwrap();

        let schema_info = builder.manifest.schemas.get("schema.json").unwrap();
        assert_eq!(schema_info.format, "json-schema");
    }

    #[test]
    fn BundleBuilder___add_schema_file___unknown_format() {
        let temp_dir = TempDir::new().unwrap();
        let schema_path = temp_dir.path().join("schema.xyz");
        fs::write(&schema_path, b"content").unwrap();

        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest)
            .add_schema_file(&schema_path, "schema.xyz")
            .unwrap();

        let schema_info = builder.manifest.schemas.get("schema.xyz").unwrap();
        assert_eq!(schema_info.format, "unknown");
    }

    #[test]
    fn BundleBuilder___write___invalid_manifest___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test.rbp");

        // Manifest without any platforms is invalid
        let manifest = Manifest::new("test", "1.0.0");
        let result = BundleBuilder::new(manifest).write(&output_path);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, BundleError::InvalidManifest(_)));
    }

    #[test]
    fn BundleBuilder___write___creates_valid_bundle() {
        let temp_dir = TempDir::new().unwrap();
        let lib_path = temp_dir.path().join("libtest.so");
        let output_path = temp_dir.path().join("test.rbp");
        fs::write(&lib_path, b"fake library").unwrap();

        let manifest = Manifest::new("test", "1.0.0");
        BundleBuilder::new(manifest)
            .add_library(Platform::LinuxX86_64, &lib_path)
            .unwrap()
            .write(&output_path)
            .unwrap();

        assert!(output_path.exists());

        // Verify it's a valid ZIP
        let file = File::open(&output_path).unwrap();
        let archive = zip::ZipArchive::new(file).unwrap();
        assert!(archive.len() >= 2); // manifest + library
    }

    #[test]
    fn BundleBuilder___manifest_mut___allows_modification() {
        let manifest = Manifest::new("test", "1.0.0");
        let mut builder = BundleBuilder::new(manifest);

        builder.manifest_mut().plugin.description = Some("Modified".to_string());

        assert_eq!(
            builder.manifest().plugin.description,
            Some("Modified".to_string())
        );
    }

    #[test]
    fn detect_schema_format___hpp_extension___returns_c_header() {
        assert_eq!(detect_schema_format("types.hpp"), "c-header");
    }

    // ========================================================================
    // Minisign Signature Tests
    // ========================================================================
    // These tests verify that minisign signature generation and verification
    // work correctly. The test vectors are used as reference for consumer
    // language implementations (Java, C#, Python).

    #[test]
    fn sign_data___generates_verifiable_signature() {
        use minisign::{KeyPair, PublicKey};
        use std::io::Cursor;

        // Generate a test keypair
        let keypair = KeyPair::generate_unencrypted_keypair().unwrap();

        // Test data
        let test_data = b"Hello, rustbridge!";

        // Sign the data
        let mut reader = Cursor::new(test_data.as_slice());
        let signature_box = minisign::sign(
            Some(&keypair.pk),
            &keypair.sk,
            &mut reader,
            Some("trusted comment"),
            Some("untrusted comment"),
        )
        .unwrap();

        // Verify the signature
        let pk = PublicKey::from_base64(&keypair.pk.to_base64()).unwrap();
        let mut verify_reader = Cursor::new(test_data.as_slice());
        let result = minisign::verify(&pk, &signature_box, &mut verify_reader, true, false, false);

        assert!(result.is_ok(), "Signature verification should succeed");
    }

    #[test]
    fn sign_data___wrong_data___verification_fails() {
        use minisign::{KeyPair, PublicKey};
        use std::io::Cursor;

        let keypair = KeyPair::generate_unencrypted_keypair().unwrap();
        let test_data = b"Hello, rustbridge!";
        let wrong_data = b"Hello, rustbridge?"; // Changed ! to ?

        // Sign the original data
        let mut reader = Cursor::new(test_data.as_slice());
        let signature_box =
            minisign::sign(Some(&keypair.pk), &keypair.sk, &mut reader, None, None).unwrap();

        // Try to verify with wrong data
        let pk = PublicKey::from_base64(&keypair.pk.to_base64()).unwrap();
        let mut verify_reader = Cursor::new(wrong_data.as_slice());
        let result = minisign::verify(&pk, &signature_box, &mut verify_reader, true, false, false);

        assert!(result.is_err(), "Verification should fail with wrong data");
    }

    #[test]
    fn sign_data___signature_format___has_prehash_algorithm_id() {
        use base64::Engine;
        use minisign::KeyPair;
        use std::io::Cursor;

        let keypair = KeyPair::generate_unencrypted_keypair().unwrap();
        let test_data = b"test";

        let mut reader = Cursor::new(test_data.as_slice());
        let signature_box = minisign::sign(
            Some(&keypair.pk),
            &keypair.sk,
            &mut reader,
            Some("trusted"),
            Some("untrusted"),
        )
        .unwrap();

        let sig_string = signature_box.into_string();
        let lines: Vec<&str> = sig_string.lines().collect();

        // The signature line is the second line
        let sig_base64 = lines[1];
        let sig_bytes = base64::engine::general_purpose::STANDARD
            .decode(sig_base64)
            .unwrap();

        // First two bytes should be "ED" (0x45, 0x44) for prehashed signatures
        assert_eq!(sig_bytes[0], 0x45, "First byte should be 'E'");
        assert_eq!(
            sig_bytes[1], 0x44,
            "Second byte should be 'D' for prehashed"
        );
        assert_eq!(sig_bytes.len(), 74, "Signature should be 74 bytes");
    }

    #[test]
    fn sign_data___public_key_format___has_ed_algorithm_id() {
        use base64::Engine;
        use minisign::KeyPair;

        let keypair = KeyPair::generate_unencrypted_keypair().unwrap();
        let pk_base64 = keypair.pk.to_base64();
        let pk_bytes = base64::engine::general_purpose::STANDARD
            .decode(&pk_base64)
            .unwrap();

        // First two bytes should be "Ed" (0x45, 0x64) for public keys
        assert_eq!(pk_bytes[0], 0x45, "First byte should be 'E'");
        assert_eq!(
            pk_bytes[1], 0x64,
            "Second byte should be 'd' for public key"
        );
        assert_eq!(pk_bytes.len(), 42, "Public key should be 42 bytes");
    }

    #[test]
    fn BundleBuilder___add_license_file___adds_file_to_legal_dir() {
        let temp_dir = TempDir::new().unwrap();
        let license_path = temp_dir.path().join("LICENSE");
        fs::write(&license_path, b"MIT License\n\nCopyright...").unwrap();

        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest)
            .add_license_file(&license_path)
            .unwrap();

        // Check file was added
        assert_eq!(builder.files.len(), 1);
        assert_eq!(builder.files[0].archive_path, "legal/LICENSE");

        // Check manifest was updated
        assert_eq!(builder.manifest.get_license_file(), Some("legal/LICENSE"));
    }

    #[test]
    fn BundleBuilder___add_license_file___nonexistent___returns_error() {
        let manifest = Manifest::new("test", "1.0.0");
        let result = BundleBuilder::new(manifest).add_license_file("/nonexistent/LICENSE");

        assert!(result.is_err());
    }
}
