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
pub struct BundleBuilder {
    manifest: Manifest,
    files: Vec<BundleFile>,
    signing_key: Option<(String, SecretKey)>, // (public_key_base64, secret_key)
}

/// A file to include in the bundle.
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

    /// Add a platform-specific library to the bundle.
    ///
    /// This reads the library file, computes its SHA256 checksum,
    /// and updates the manifest with the platform information.
    pub fn add_library<P: AsRef<Path>>(
        mut self,
        platform: Platform,
        library_path: P,
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
        let archive_path = format!("lib/{}/{}", platform.as_str(), file_name);

        // Update manifest
        self.manifest
            .add_platform(platform, &archive_path, &checksum);

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

    #[test]
    fn compute_sha256___returns_consistent_hash() {
        let data = b"hello world";
        let hash1 = compute_sha256(data);
        let hash2 = compute_sha256(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 is 32 bytes = 64 hex chars
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
    fn BundleBuilder___add_bytes___adds_file() {
        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest).add_bytes("test.txt", b"hello".to_vec());

        assert_eq!(builder.files.len(), 1);
        assert_eq!(builder.files[0].archive_path, "test.txt");
        assert_eq!(builder.files[0].contents, b"hello");
    }
}
