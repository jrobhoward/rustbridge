//! Bundle loading utilities.
//!
//! The [`BundleLoader`] provides functionality to load and extract plugin bundles.

use crate::builder::verify_sha256;
use crate::{BundleError, BundleResult, MANIFEST_FILE, Manifest, Platform};
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
    /// Returns the path to the extracted library file.
    pub fn extract_library<P: AsRef<Path>>(
        &mut self,
        platform: Platform,
        output_dir: P,
    ) -> BundleResult<PathBuf> {
        let output_dir = output_dir.as_ref();

        // Get platform info from manifest
        let platform_info = self.manifest.get_platform(platform).ok_or_else(|| {
            BundleError::UnsupportedPlatform(format!(
                "Platform {} not found in bundle",
                platform.as_str()
            ))
        })?;

        let library_path = platform_info.library.clone();
        let expected_checksum = platform_info.checksum.clone();

        // Read the library from the archive
        let contents = {
            let mut library_file = self.archive.by_name(&library_path).map_err(|_| {
                BundleError::MissingFile(format!("Library not found in bundle: {library_path}"))
            })?;

            let mut contents = Vec::new();
            library_file.read_to_end(&mut contents)?;
            contents
        };

        // Verify checksum
        if !verify_sha256(&contents, &expected_checksum) {
            let actual = crate::builder::compute_sha256(&contents);
            return Err(BundleError::ChecksumMismatch {
                path: library_path,
                expected: expected_checksum,
                actual: format!("sha256:{actual}"),
            });
        }

        // Create output directory
        fs::create_dir_all(output_dir)?;

        // Determine output filename
        let file_name = Path::new(&library_path)
            .file_name()
            .ok_or_else(|| BundleError::InvalidManifest("Invalid library path".to_string()))?;

        let output_path = output_dir.join(file_name);

        // Write the library file
        fs::write(&output_path, &contents)?;

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

    /// List all files in the bundle.
    #[must_use]
    pub fn list_files(&self) -> Vec<String> {
        (0..self.archive.len())
            .filter_map(|i| self.archive.name_for_index(i).map(String::from))
            .collect()
    }

    /// Check if a file exists in the bundle.
    #[must_use]
    pub fn has_file(&self, path: &str) -> bool {
        self.archive.index_for_name(path).is_some()
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use crate::builder::BundleBuilder;
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

    #[test]
    fn BundleLoader___open___reads_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let bundle_path = create_test_bundle(&temp_dir);

        let loader = BundleLoader::open(&bundle_path).unwrap();

        assert_eq!(loader.manifest().plugin.name, "test-plugin");
        assert_eq!(loader.manifest().plugin.version, "1.0.0");
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
}
