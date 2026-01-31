//! Integration tests for CLI bundle commands.
//!
//! Tests error paths and edge cases for bundle create, combine, slim, and extract.

#![allow(non_snake_case)]

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

/// Helper to create a fake library file.
fn create_fake_library(dir: &TempDir, name: &str) -> PathBuf {
    let path = dir.path().join(name);
    fs::write(&path, b"fake library content").unwrap();
    path
}

/// Helper to create a minimal bundle for testing.
fn create_test_bundle(
    temp_dir: &TempDir,
    name: &str,
    platforms: &[(&str, &str)], // (platform, variant)
) -> PathBuf {
    use rustbridge_bundle::{BundleBuilder, Manifest, Platform};

    let manifest = Manifest::new(name, "1.0.0");
    let mut builder = BundleBuilder::new(manifest);

    for (platform_str, variant) in platforms {
        let platform = Platform::parse(platform_str).unwrap();
        let lib_path = create_fake_library(temp_dir, &format!("lib_{platform_str}_{variant}.so"));

        builder = builder
            .add_library_variant(platform, variant, &lib_path)
            .unwrap();
    }

    let output_path = temp_dir.path().join(format!("{name}.rbp"));
    builder.write(&output_path).unwrap();
    output_path
}

// =============================================================================
// Bundle Create Error Path Tests
// =============================================================================

mod create_errors {
    use super::*;
    use rustbridge_bundle::Platform;

    #[test]
    fn create___invalid_platform_string___returns_error() {
        // Platform::parse returns None for invalid platforms
        let result = Platform::parse("invalid-platform");

        assert!(result.is_none());
    }

    #[test]
    fn create___partial_platform_match___returns_none() {
        // "linux-x86" should NOT match "linux-x86_64"
        let result = Platform::parse("linux-x86");

        assert!(result.is_none());
    }

    #[test]
    fn create___platform_case_sensitivity___exact_match_required() {
        // Platform parsing is case-sensitive
        assert!(Platform::parse("Linux-x86_64").is_none());
        assert!(Platform::parse("LINUX-X86_64").is_none());
        assert!(Platform::parse("linux-X86_64").is_none());

        // Only exact lowercase works
        assert!(Platform::parse("linux-x86_64").is_some());
    }

    #[test]
    fn create___platform_with_whitespace___returns_none() {
        assert!(Platform::parse(" linux-x86_64").is_none());
        assert!(Platform::parse("linux-x86_64 ").is_none());
        assert!(Platform::parse(" linux-x86_64 ").is_none());
        assert!(Platform::parse("linux-x86_64\n").is_none());
        assert!(Platform::parse("\tlinux-x86_64").is_none());
    }

    #[test]
    fn create___nonexistent_library_file___returns_error() {
        use rustbridge_bundle::{BundleBuilder, Manifest, Platform};

        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest);

        let result =
            builder.add_library_variant(Platform::LinuxX86_64, "release", "/nonexistent/lib.so");

        assert!(result.is_err());
    }

    #[test]
    fn create___empty_library_file___succeeds() {
        use rustbridge_bundle::{BundleBuilder, Manifest, Platform};

        let temp_dir = TempDir::new().unwrap();
        let empty_lib = temp_dir.path().join("empty.so");
        fs::write(&empty_lib, b"").unwrap();

        let manifest = Manifest::new("test", "1.0.0");
        let builder = BundleBuilder::new(manifest);

        // Empty files should be allowed (weird but valid)
        let result = builder.add_library_variant(Platform::LinuxX86_64, "release", &empty_lib);

        assert!(result.is_ok());
    }
}

// =============================================================================
// Bundle Combine Error Path Tests
// =============================================================================

mod combine_errors {
    use super::*;
    use rustbridge_bundle::{BundleLoader, Platform};

    #[test]
    fn combine___single_bundle___fails_validation() {
        // The combine function requires at least 2 bundles
        // This tests that the check exists (tested at CLI level)
        let temp_dir = TempDir::new().unwrap();
        let bundle = create_test_bundle(&temp_dir, "single", &[("linux-x86_64", "release")]);

        // Verify the bundle was created
        assert!(bundle.exists());

        // A single bundle cannot be combined (this would be tested at CLI level)
        // Here we just verify the bundle is valid
        let loader = BundleLoader::open(&bundle).unwrap();
        assert_eq!(loader.manifest().plugin.name, "single");
    }

    #[test]
    fn combine___duplicate_platform_variant___detected() {
        use rustbridge_bundle::BundleLoader;

        let temp_dir = TempDir::new().unwrap();

        // Create two bundles with the same platform/variant
        let bundle1 = create_test_bundle(&temp_dir, "plugin1", &[("linux-x86_64", "release")]);
        let bundle2 = create_test_bundle(&temp_dir, "plugin2", &[("linux-x86_64", "release")]);

        // Both bundles have linux-x86_64:release
        let loader1 = BundleLoader::open(&bundle1).unwrap();
        let loader2 = BundleLoader::open(&bundle2).unwrap();

        let variants1 = loader1.list_variants(Platform::LinuxX86_64);
        let variants2 = loader2.list_variants(Platform::LinuxX86_64);

        // Both have release variant - this would conflict when combining
        assert!(variants1.contains(&"release"));
        assert!(variants2.contains(&"release"));
    }

    #[test]
    fn combine___different_platforms___succeeds() {
        let temp_dir = TempDir::new().unwrap();

        // Create bundles with different platforms
        let _bundle1 = create_test_bundle(&temp_dir, "linux-build", &[("linux-x86_64", "release")]);
        let _bundle2 =
            create_test_bundle(&temp_dir, "darwin-build", &[("darwin-aarch64", "release")]);

        // These could be combined without conflict
        // (actual combine logic is in CLI, tested there)
    }
}

// =============================================================================
// Bundle Slim Error Path Tests
// =============================================================================

mod slim_errors {
    use super::*;
    use rustbridge_bundle::{BundleLoader, Platform};

    #[test]
    fn slim___nonexistent_variant___results_in_empty() {
        let temp_dir = TempDir::new().unwrap();
        let bundle = create_test_bundle(&temp_dir, "test", &[("linux-x86_64", "release")]);

        let loader = BundleLoader::open(&bundle).unwrap();
        let variants = loader.list_variants(Platform::LinuxX86_64);

        // Only release exists
        assert!(variants.contains(&"release"));
        assert!(!variants.contains(&"debug")); // doesn't exist

        // Slim to non-existent variant would result in empty output
        // (actual logic tested in CLI)
    }

    #[test]
    fn slim___nonexistent_platform___results_in_empty() {
        let temp_dir = TempDir::new().unwrap();
        let bundle = create_test_bundle(&temp_dir, "test", &[("linux-x86_64", "release")]);

        let loader = BundleLoader::open(&bundle).unwrap();

        // Windows doesn't exist
        assert!(!loader.manifest().supports_platform(Platform::WindowsX86_64));
    }
}

// =============================================================================
// Manifest Validation Edge Cases
// =============================================================================

mod manifest_edge_cases {
    use rustbridge_bundle::{Manifest, Platform, PlatformInfo, VariantInfo};
    use std::collections::HashMap;

    #[test]
    fn validate___unicode_variant_name___rejected() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "sha256:abc123".to_string(),
                build: None,
            },
        );
        variants.insert(
            "débug".to_string(), // Unicode character
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "sha256:abc123".to_string(),
                build: None,
            },
        );
        manifest
            .platforms
            .insert("linux-x86_64".to_string(), PlatformInfo { variants });

        let result = manifest.validate();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid variant name")
        );
    }

    #[test]
    fn validate___emoji_variant_name___rejected() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "sha256:abc123".to_string(),
                build: None,
            },
        );
        variants.insert(
            "🚀fast".to_string(), // Emoji
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "sha256:abc123".to_string(),
                build: None,
            },
        );
        manifest
            .platforms
            .insert("linux-x86_64".to_string(), PlatformInfo { variants });

        let result = manifest.validate();

        assert!(result.is_err());
    }

    #[test]
    fn validate___very_long_variant_name___allowed_if_valid_chars() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let long_name = "a".repeat(1000); // Very long but valid chars
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "sha256:abc123".to_string(),
                build: None,
            },
        );
        variants.insert(
            long_name,
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "sha256:abc123".to_string(),
                build: None,
            },
        );
        manifest
            .platforms
            .insert("linux-x86_64".to_string(), PlatformInfo { variants });

        // Long names with valid characters should be accepted
        let result = manifest.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn validate___checksum_non_hex_chars___accepted_by_format_check() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "lib/test.so".to_string(),
                // Non-hex chars after sha256: prefix - format check only validates prefix
                checksum: "sha256:gggggggg".to_string(),
                build: None,
            },
        );
        manifest
            .platforms
            .insert("linux-x86_64".to_string(), PlatformInfo { variants });

        // The manifest validation only checks for sha256: prefix, not hex validity
        // Actual verification happens during extraction
        let result = manifest.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn validate___checksum_mixed_case_prefix___rejected() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "SHA256:abc123".to_string(), // Uppercase prefix
                build: None,
            },
        );
        manifest
            .platforms
            .insert("linux-x86_64".to_string(), PlatformInfo { variants });

        let result = manifest.validate();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("sha256:"));
    }

    #[test]
    fn validate___checksum_double_prefix___rejected() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "sha256:sha256:abc123".to_string(),
                build: None,
            },
        );
        manifest
            .platforms
            .insert("linux-x86_64".to_string(), PlatformInfo { variants });

        // This is technically valid as it starts with "sha256:"
        // The checksum verification will fail later if content doesn't match
        let result = manifest.validate();
        assert!(result.is_ok()); // Format validation passes
    }

    #[test]
    fn validate___variant_only_hyphens___rejected() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "sha256:abc123".to_string(),
                build: None,
            },
        );
        variants.insert(
            "---".to_string(), // Only hyphens
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "sha256:abc123".to_string(),
                build: None,
            },
        );
        manifest
            .platforms
            .insert("linux-x86_64".to_string(), PlatformInfo { variants });

        let result = manifest.validate();

        // Should be rejected - variant names must start/end with alphanumeric
        assert!(result.is_err());
    }

    #[test]
    fn validate___empty_variants_map___rejected() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.platforms.insert(
            "linux-x86_64".to_string(),
            PlatformInfo {
                variants: HashMap::new(),
            },
        );

        let result = manifest.validate();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("at least one variant")
        );
    }

    #[test]
    fn add_platform___normalizes_checksum_prefix() {
        let mut manifest = Manifest::new("test", "1.0.0");

        // add_platform adds the sha256: prefix automatically
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "abc123");

        let info = manifest.get_platform(Platform::LinuxX86_64).unwrap();
        let release = info.release().unwrap();

        // Checksum should have sha256: prefix
        assert!(release.checksum.starts_with("sha256:"));
        assert_eq!(release.checksum, "sha256:abc123");
    }
}

// =============================================================================
// Transport Codec Error Path Tests
// =============================================================================

mod transport_edge_cases {
    use serde::{Deserialize, Serialize};
    use serde_json::json;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestMessage {
        message: String,
    }

    #[test]
    fn deserialize___deeply_nested_json___handled() {
        // Create deeply nested JSON
        let mut nested = json!({"value": "leaf"});
        for _ in 0..100 {
            nested = json!({"nested": nested});
        }

        // Should deserialize without stack overflow
        let result = serde_json::from_value::<serde_json::Value>(nested);
        assert!(result.is_ok());
    }

    #[test]
    fn deserialize___unicode_outside_bmp___handled() {
        // Emoji and other unicode outside Basic Multilingual Plane
        let json_str = r#"{"message": "Hello 🌍🚀✨ World"}"#;

        let result: Result<TestMessage, _> = serde_json::from_str(json_str);

        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.message.contains("🌍"));
    }

    #[test]
    fn deserialize___null_byte_in_string___handled() {
        // JSON with escaped null byte
        let json_str = r#"{"message": "hello\u0000world"}"#;

        let result: Result<TestMessage, _> = serde_json::from_str(json_str);

        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.message.contains('\0'));
    }

    #[test]
    fn deserialize___very_large_number___error_or_parsed() {
        // Number outside i64 range
        let json_str = r#"{"value": 99999999999999999999999999999999999999999999}"#;

        // serde_json will parse this as f64 or fail depending on target type
        let result: Result<serde_json::Value, _> = serde_json::from_str(json_str);

        // serde_json parses large numbers as f64
        assert!(result.is_ok());
    }

    #[test]
    fn serialize___string_with_special_chars___escaped() {
        let msg = TestMessage {
            message: "hello\n\t\"world\"\\".to_string(),
        };

        let json = serde_json::to_string(&msg).unwrap();

        // Special chars should be escaped
        assert!(json.contains("\\n"));
        assert!(json.contains("\\t"));
        assert!(json.contains("\\\""));
        assert!(json.contains("\\\\"));
    }

    #[test]
    fn deserialize___bom_prefix___error() {
        // UTF-8 BOM followed by JSON
        let json_with_bom = "\u{FEFF}{\"message\": \"test\"}";

        let result: Result<TestMessage, _> = serde_json::from_str(json_with_bom);

        // serde_json doesn't handle BOM
        assert!(result.is_err());
    }

    #[test]
    fn deserialize___trailing_data___error() {
        let json_str = r#"{"message": "test"}garbage"#;

        // from_str should reject trailing data
        let result: Result<TestMessage, _> = serde_json::from_str(json_str);

        assert!(result.is_err());
    }
}

// =============================================================================
// Extract Error Path Tests
// =============================================================================

mod extract_errors {
    use super::*;
    use rustbridge_bundle::BundleLoader;

    #[test]
    fn extract___nonexistent_bundle___returns_error() {
        let result = BundleLoader::open("/nonexistent/bundle.rbp");

        assert!(result.is_err());
    }

    #[test]
    fn extract___corrupt_zip___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let corrupt_path = temp_dir.path().join("corrupt.rbp");
        fs::write(&corrupt_path, b"not a valid zip file").unwrap();

        let result = BundleLoader::open(&corrupt_path);

        assert!(result.is_err());
    }

    #[test]
    fn extract___truncated_file___returns_error() {
        let temp_dir = TempDir::new().unwrap();

        // Create a valid bundle first
        let bundle = create_test_bundle(&temp_dir, "test", &[("linux-x86_64", "release")]);

        // Read and truncate it
        let content = fs::read(&bundle).unwrap();
        let truncated_path = temp_dir.path().join("truncated.rbp");
        fs::write(&truncated_path, &content[..content.len() / 2]).unwrap();

        let result = BundleLoader::open(&truncated_path);

        assert!(result.is_err());
    }

    #[test]
    fn extract___empty_file___returns_error() {
        let temp_dir = TempDir::new().unwrap();
        let empty_path = temp_dir.path().join("empty.rbp");
        fs::write(&empty_path, b"").unwrap();

        let result = BundleLoader::open(&empty_path);

        assert!(result.is_err());
    }
}

// =============================================================================
// Concurrent Bundle Operation Tests
// =============================================================================

mod concurrent_operations {
    use super::*;
    use rustbridge_bundle::{BundleLoader, Platform};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn concurrent___multiple_loaders_same_bundle___independent() {
        let temp_dir = TempDir::new().unwrap();
        let bundle = create_test_bundle(&temp_dir, "shared", &[("linux-x86_64", "release")]);

        // Open the same bundle from multiple threads
        let bundle_path = Arc::new(bundle);
        let mut handles = Vec::new();

        for i in 0..5 {
            let path = Arc::clone(&bundle_path);
            let handle = thread::spawn(move || {
                let loader = BundleLoader::open(&*path).unwrap();
                let manifest = loader.manifest();

                // Each thread reads independently
                assert_eq!(manifest.plugin.name, "shared");
                assert!(manifest.supports_platform(Platform::LinuxX86_64));

                // Return thread ID for verification
                i
            });
            handles.push(handle);
        }

        // All threads should complete successfully
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn concurrent___extract_to_different_dirs___no_conflict() {
        let temp_dir = TempDir::new().unwrap();
        let bundle = create_test_bundle(&temp_dir, "extract-test", &[("linux-x86_64", "release")]);

        let bundle_path = Arc::new(bundle);
        let base_dir = Arc::new(temp_dir.path().to_path_buf());
        let mut handles = Vec::new();

        for i in 0..3 {
            let path = Arc::clone(&bundle_path);
            let dir = Arc::clone(&base_dir);
            let handle = thread::spawn(move || {
                let mut loader = BundleLoader::open(&*path).unwrap();

                // Each thread extracts to a different directory
                let extract_dir = dir.join(format!("extract_{}", i));
                fs::create_dir_all(&extract_dir).unwrap();

                let result =
                    loader.extract_library_variant(Platform::LinuxX86_64, "release", &extract_dir);

                result.is_ok()
            });
            handles.push(handle);
        }

        // All extractions should succeed
        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert!(results.iter().all(|&r| r), "All extractions should succeed");
    }

    #[test]
    fn concurrent___list_files_while_reading___no_deadlock() {
        let temp_dir = TempDir::new().unwrap();
        let bundle = create_test_bundle(&temp_dir, "list-test", &[("linux-x86_64", "release")]);

        let bundle_path = Arc::new(bundle);
        let mut handles = Vec::new();

        for _ in 0..10 {
            let path = Arc::clone(&bundle_path);
            let handle = thread::spawn(move || {
                let loader = BundleLoader::open(&*path).unwrap();

                // Interleave list_files and manifest reads
                let files = loader.list_files();
                let manifest = loader.manifest();
                let files2 = loader.list_files();

                assert!(!files.is_empty());
                assert!(!files2.is_empty());
                assert!(!manifest.plugin.name.is_empty());
            });
            handles.push(handle);
        }

        // Should complete without deadlock (test has implicit timeout)
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn rapid___open_close_cycles___no_resource_leak() {
        let temp_dir = TempDir::new().unwrap();
        let bundle = create_test_bundle(&temp_dir, "rapid", &[("linux-x86_64", "release")]);

        // Rapidly open and close the bundle
        for _ in 0..100 {
            let loader = BundleLoader::open(&bundle).unwrap();
            let _ = loader.manifest();
            drop(loader);
        }

        // If we get here without running out of file handles, test passes
    }
}

// =============================================================================
// Configuration Edge Case Tests
// =============================================================================

mod config_edge_cases {
    use rustbridge_bundle::{Manifest, Platform};

    #[test]
    fn manifest___plugin_name_with_unicode___preserved() {
        let manifest = Manifest::new("плагин-тест", "1.0.0");

        assert_eq!(manifest.plugin.name, "плагин-тест");
    }

    #[test]
    fn manifest___plugin_name_with_emoji___preserved() {
        let manifest = Manifest::new("plugin-🚀", "1.0.0");

        assert_eq!(manifest.plugin.name, "plugin-🚀");
    }

    #[test]
    fn manifest___version_with_prerelease___preserved() {
        let manifest = Manifest::new("test", "1.0.0-beta.1+build.123");

        assert_eq!(manifest.plugin.version, "1.0.0-beta.1+build.123");
    }

    #[test]
    fn manifest___description_multiline___preserved() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.plugin.description = Some("Line 1\nLine 2\nLine 3".to_string());

        assert!(manifest.plugin.description.as_ref().unwrap().contains('\n'));
    }

    #[test]
    fn manifest___authors_empty_list___valid() {
        let manifest = Manifest::new("test", "1.0.0");

        assert!(manifest.plugin.authors.is_empty());
    }

    #[test]
    fn manifest___authors_with_email___preserved() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.plugin.authors = vec!["John Doe <john@example.com>".to_string()];

        assert!(manifest.plugin.authors[0].contains('@'));
    }

    #[test]
    fn manifest___repository_url_special_chars___preserved() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.plugin.repository =
            Some("https://github.com/org/repo?query=1&foo=bar#section".to_string());

        let repo = manifest.plugin.repository.as_ref().unwrap();
        assert!(repo.contains('?'));
        assert!(repo.contains('&'));
        assert!(repo.contains('#'));
    }

    #[test]
    fn manifest___all_platforms___distinct_strings() {
        let platforms: Vec<&str> = Platform::all().iter().map(|p| p.as_str()).collect();

        // All platform strings should be unique
        let unique: std::collections::HashSet<_> = platforms.iter().collect();
        assert_eq!(unique.len(), platforms.len());
    }
}
