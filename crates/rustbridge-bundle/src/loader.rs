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
            assert!(info.unwrap().library.contains("libtest.so"));
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
        let expected_checksum = format!("sha256:{}", compute_sha256(lib_contents));
        assert_eq!(platform_info.checksum, expected_checksum);

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
}
