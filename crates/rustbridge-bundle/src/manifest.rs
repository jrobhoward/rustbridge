//! Manifest schema for plugin bundles.
//!
//! The manifest describes the plugin metadata, supported platforms,
//! and available API messages. Supports multi-variant builds (release, debug, etc.)
//! with the `release` variant being mandatory and the implicit default.

use crate::{BUNDLE_VERSION, BundleError, BundleResult, Platform};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bundle manifest - the main descriptor for a plugin bundle.
///
/// This corresponds to the `manifest.json` file in the bundle root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Bundle format version (e.g., "1.0").
    pub bundle_version: String,

    /// Plugin metadata.
    pub plugin: PluginInfo,

    /// Platform-specific library information.
    /// Key is the platform string (e.g., "linux-x86_64").
    pub platforms: HashMap<String, PlatformInfo>,

    /// Build information (optional).
    /// Contains metadata about when/how the bundle was built.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build_info: Option<BuildInfo>,

    /// SBOM (Software Bill of Materials) paths.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sbom: Option<Sbom>,

    /// Combined checksum of all schema files.
    /// Used to verify schema compatibility when combining bundles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema_checksum: Option<String>,

    /// Path to license notices file within the bundle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notices: Option<String>,

    /// Path to the plugin's own license file within the bundle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_file: Option<String>,

    /// Minisign public key for signature verification (base64-encoded).
    /// Format: "RWS..." (standard minisign public key format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,

    /// Schema files embedded in the bundle.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub schemas: HashMap<String, SchemaInfo>,

    /// Bridge libraries bundled with the plugin (e.g., JNI bridge).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridges: Option<BridgeInfo>,
}

/// Plugin metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin name (e.g., "my-plugin").
    pub name: String,

    /// Plugin version (semver, e.g., "1.0.0").
    pub version: String,

    /// Short description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// List of authors.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,

    /// License identifier (e.g., "MIT").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Repository URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

/// Platform-specific library information with variant support.
///
/// Each platform must have at least a `release` variant.
/// The `release` variant is the implicit default when no variant is specified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Available variants for this platform.
    /// Must contain at least `release` (mandatory).
    pub variants: HashMap<String, VariantInfo>,
}

impl PlatformInfo {
    /// Create new platform info with a single release variant.
    pub fn new(library: String, checksum: String) -> Self {
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library,
                checksum,
                build: None,
            },
        );
        Self { variants }
    }

    /// Get the release variant (always present after validation).
    #[must_use]
    pub fn release(&self) -> Option<&VariantInfo> {
        self.variants.get("release")
    }

    /// Get a specific variant by name.
    #[must_use]
    pub fn variant(&self, name: &str) -> Option<&VariantInfo> {
        self.variants.get(name)
    }

    /// Get the default variant (release).
    #[must_use]
    pub fn default_variant(&self) -> Option<&VariantInfo> {
        self.release()
    }

    /// List all available variant names.
    #[must_use]
    pub fn variant_names(&self) -> Vec<&str> {
        self.variants.keys().map(String::as_str).collect()
    }

    /// Check if a variant exists.
    #[must_use]
    pub fn has_variant(&self, name: &str) -> bool {
        self.variants.contains_key(name)
    }

    /// Add a variant to this platform.
    pub fn add_variant(&mut self, name: String, info: VariantInfo) {
        self.variants.insert(name, info);
    }
}

/// Variant-specific library information.
///
/// Each variant represents a different build configuration (release, debug, etc.)
/// of the same plugin for a specific platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantInfo {
    /// Relative path to the library within the bundle.
    /// Example: "lib/linux-x86_64/release/libplugin.so"
    pub library: String,

    /// SHA256 checksum of the library file.
    /// Format: "sha256:hexstring"
    pub checksum: String,

    /// Flexible build metadata - any JSON object.
    /// This can contain toolchain-specific fields like:
    /// - `profile`: "release" or "debug"
    /// - `opt_level`: "0", "1", "2", "3", "s", "z"
    /// - `features`: ["json", "binary"]
    /// - `cflags`: "-O3 -march=native" (for C/C++)
    /// - `go_tags`: ["production"] (for Go)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<serde_json::Value>,
}

/// Build information (all fields optional).
///
/// Contains metadata about when and how the bundle was built.
/// Useful for traceability and debugging but not required.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildInfo {
    /// Who/what built this bundle (e.g., "GitHub Actions", "local")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub built_by: Option<String>,

    /// When the bundle was built (ISO 8601 timestamp).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub built_at: Option<String>,

    /// Host triple where the build ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,

    /// Compiler version used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiler: Option<String>,

    /// rustbridge version used to create the bundle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rustbridge_version: Option<String>,

    /// Git repository information (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git: Option<GitInfo>,

    /// Custom key/value metadata for informational purposes.
    /// Can include arbitrary data like repository URL, CI job ID, etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom: Option<HashMap<String, String>>,
}

/// Git repository information.
///
/// All fields except `commit` are optional. This section is only
/// present if the project uses git.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitInfo {
    /// Full commit hash (required if git section present).
    pub commit: String,

    /// Branch name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Git tag (if on a tagged commit).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Whether the working tree had uncommitted changes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dirty: Option<bool>,
}

/// SBOM (Software Bill of Materials) paths.
///
/// Points to SBOM files within the bundle. Both CycloneDX and SPDX
/// formats are supported and can be included simultaneously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    /// Path to CycloneDX SBOM file (e.g., "sbom/sbom.cdx.json").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cyclonedx: Option<String>,

    /// Path to SPDX SBOM file (e.g., "sbom/sbom.spdx.json").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spdx: Option<String>,
}

/// Check if a variant name is valid.
///
/// Valid variant names are lowercase alphanumeric with hyphens.
/// Examples: "release", "debug", "nightly", "opt-size"
fn is_valid_variant_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Must start and end with alphanumeric
    let chars: Vec<char> = name.chars().collect();
    if !chars[0].is_ascii_lowercase() && !chars[0].is_ascii_digit() {
        return false;
    }
    if !chars[chars.len() - 1].is_ascii_lowercase() && !chars[chars.len() - 1].is_ascii_digit() {
        return false;
    }

    // All characters must be lowercase alphanumeric or hyphen
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Schema file information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    /// Relative path to the schema file within the bundle.
    pub path: String,

    /// Schema format (e.g., "c-header", "json-schema").
    pub format: String,

    /// SHA256 checksum of the schema file.
    pub checksum: String,

    /// Optional description of what this schema describes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Bridge libraries bundled with the plugin.
///
/// This allows bundling bridge libraries (like the JNI bridge) alongside
/// the plugin for self-contained distribution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BridgeInfo {
    /// JNI bridge libraries by platform.
    /// Key is the platform string (e.g., "linux-x86_64").
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub jni: HashMap<String, PlatformInfo>,
}

impl Manifest {
    /// Create a new manifest with minimal required fields.
    #[must_use]
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            bundle_version: BUNDLE_VERSION.to_string(),
            plugin: PluginInfo {
                name: name.to_string(),
                version: version.to_string(),
                description: None,
                authors: Vec::new(),
                license: None,
                repository: None,
            },
            platforms: HashMap::new(),
            build_info: None,
            sbom: None,
            schema_checksum: None,
            notices: None,
            license_file: None,
            public_key: None,
            schemas: HashMap::new(),
            bridges: None,
        }
    }

    /// Add a platform with a release variant to the manifest.
    ///
    /// This is a convenience method that adds the library as the `release` variant.
    /// For multiple variants, use `add_platform_variant` instead.
    pub fn add_platform(&mut self, platform: Platform, library_path: &str, checksum: &str) {
        let platform_key = platform.as_str().to_string();

        if let Some(platform_info) = self.platforms.get_mut(&platform_key) {
            // Platform exists, add/update release variant
            platform_info.variants.insert(
                "release".to_string(),
                VariantInfo {
                    library: library_path.to_string(),
                    checksum: format!("sha256:{checksum}"),
                    build: None,
                },
            );
        } else {
            // New platform
            self.platforms.insert(
                platform_key,
                PlatformInfo::new(library_path.to_string(), format!("sha256:{checksum}")),
            );
        }
    }

    /// Add a specific variant to a platform.
    ///
    /// If the platform doesn't exist, it will be created.
    pub fn add_platform_variant(
        &mut self,
        platform: Platform,
        variant: &str,
        library_path: &str,
        checksum: &str,
        build: Option<serde_json::Value>,
    ) {
        let platform_key = platform.as_str().to_string();

        let platform_info = self
            .platforms
            .entry(platform_key)
            .or_insert_with(|| PlatformInfo {
                variants: HashMap::new(),
            });

        platform_info.variants.insert(
            variant.to_string(),
            VariantInfo {
                library: library_path.to_string(),
                checksum: format!("sha256:{checksum}"),
                build,
            },
        );
    }

    /// Set the public key for signature verification.
    pub fn set_public_key(&mut self, public_key: String) {
        self.public_key = Some(public_key);
    }

    /// Add a schema file to the manifest.
    pub fn add_schema(
        &mut self,
        name: String,
        path: String,
        format: String,
        checksum: String,
        description: Option<String>,
    ) {
        self.schemas.insert(
            name,
            SchemaInfo {
                path,
                format,
                checksum,
                description,
            },
        );
    }

    /// Set the build information.
    pub fn set_build_info(&mut self, build_info: BuildInfo) {
        self.build_info = Some(build_info);
    }

    /// Get the build information.
    #[must_use]
    pub fn get_build_info(&self) -> Option<&BuildInfo> {
        self.build_info.as_ref()
    }

    /// Set the SBOM paths.
    pub fn set_sbom(&mut self, sbom: Sbom) {
        self.sbom = Some(sbom);
    }

    /// Get the SBOM paths.
    #[must_use]
    pub fn get_sbom(&self) -> Option<&Sbom> {
        self.sbom.as_ref()
    }

    /// Set the schema checksum for bundle combining validation.
    pub fn set_schema_checksum(&mut self, checksum: String) {
        self.schema_checksum = Some(checksum);
    }

    /// Get the schema checksum.
    #[must_use]
    pub fn get_schema_checksum(&self) -> Option<&str> {
        self.schema_checksum.as_deref()
    }

    /// Set the notices file path.
    pub fn set_notices(&mut self, path: String) {
        self.notices = Some(path);
    }

    /// Get the notices file path.
    #[must_use]
    pub fn get_notices(&self) -> Option<&str> {
        self.notices.as_deref()
    }

    /// Set the license file path.
    pub fn set_license_file(&mut self, path: String) {
        self.license_file = Some(path);
    }

    /// Get the license file path.
    #[must_use]
    pub fn get_license_file(&self) -> Option<&str> {
        self.license_file.as_deref()
    }

    /// Add a JNI bridge library variant to the manifest.
    ///
    /// This is used for bundling the JNI bridge alongside the plugin
    /// for self-contained distribution to Java 17+ users.
    pub fn add_jni_bridge(
        &mut self,
        platform: Platform,
        variant: &str,
        library_path: &str,
        checksum: &str,
    ) {
        let bridges = self.bridges.get_or_insert_with(BridgeInfo::default);
        let platform_key = platform.as_str().to_string();

        let platform_info = bridges
            .jni
            .entry(platform_key)
            .or_insert_with(|| PlatformInfo {
                variants: HashMap::new(),
            });

        platform_info.variants.insert(
            variant.to_string(),
            VariantInfo {
                library: library_path.to_string(),
                checksum: format!("sha256:{checksum}"),
                build: None,
            },
        );
    }

    /// Check if the bundle includes a JNI bridge library.
    #[must_use]
    pub fn has_jni_bridge(&self) -> bool {
        self.bridges.as_ref().is_some_and(|b| !b.jni.is_empty())
    }

    /// Get JNI bridge info for a specific platform.
    #[must_use]
    pub fn get_jni_bridge(&self, platform: Platform) -> Option<&PlatformInfo> {
        self.bridges
            .as_ref()
            .and_then(|b| b.jni.get(platform.as_str()))
    }

    /// Get a specific variant for a platform.
    ///
    /// Returns the release variant if `variant` is None.
    #[must_use]
    pub fn get_variant(&self, platform: Platform, variant: Option<&str>) -> Option<&VariantInfo> {
        let platform_info = self.platforms.get(platform.as_str())?;
        let variant_name = variant.unwrap_or("release");
        platform_info.variants.get(variant_name)
    }

    /// Get the release variant for a platform (default).
    #[must_use]
    pub fn get_release_variant(&self, platform: Platform) -> Option<&VariantInfo> {
        self.get_variant(platform, Some("release"))
    }

    /// List all variants for a platform.
    #[must_use]
    pub fn list_variants(&self, platform: Platform) -> Vec<&str> {
        self.platforms
            .get(platform.as_str())
            .map(|p| p.variant_names())
            .unwrap_or_default()
    }

    /// Get platform info for a specific platform.
    #[must_use]
    pub fn get_platform(&self, platform: Platform) -> Option<&PlatformInfo> {
        self.platforms.get(platform.as_str())
    }

    /// Check if a platform is supported.
    #[must_use]
    pub fn supports_platform(&self, platform: Platform) -> bool {
        self.platforms.contains_key(platform.as_str())
    }

    /// Get all supported platforms.
    #[must_use]
    pub fn supported_platforms(&self) -> Vec<Platform> {
        self.platforms
            .keys()
            .filter_map(|k| Platform::parse(k))
            .collect()
    }

    /// Validate the manifest.
    pub fn validate(&self) -> BundleResult<()> {
        // Check bundle version
        if self.bundle_version.is_empty() {
            return Err(BundleError::InvalidManifest(
                "bundle_version is required".to_string(),
            ));
        }

        // Check plugin name
        if self.plugin.name.is_empty() {
            return Err(BundleError::InvalidManifest(
                "plugin.name is required".to_string(),
            ));
        }

        // Check plugin version
        if self.plugin.version.is_empty() {
            return Err(BundleError::InvalidManifest(
                "plugin.version is required".to_string(),
            ));
        }

        // Check at least one platform is defined
        if self.platforms.is_empty() {
            return Err(BundleError::InvalidManifest(
                "at least one platform must be defined".to_string(),
            ));
        }

        // Validate each platform
        for (key, info) in &self.platforms {
            if Platform::parse(key).is_none() {
                return Err(BundleError::InvalidManifest(format!(
                    "unknown platform: {key}"
                )));
            }

            // Each platform must have at least one variant
            if info.variants.is_empty() {
                return Err(BundleError::InvalidManifest(format!(
                    "platform {key}: at least one variant is required"
                )));
            }

            // Release variant is mandatory
            if !info.variants.contains_key("release") {
                return Err(BundleError::InvalidManifest(format!(
                    "platform {key}: 'release' variant is required"
                )));
            }

            // Validate each variant
            for (variant_name, variant_info) in &info.variants {
                // Validate variant name (lowercase alphanumeric + hyphens)
                if !is_valid_variant_name(variant_name) {
                    return Err(BundleError::InvalidManifest(format!(
                        "platform {key}: invalid variant name '{variant_name}' \
                         (must be lowercase alphanumeric with hyphens)"
                    )));
                }

                if variant_info.library.is_empty() {
                    return Err(BundleError::InvalidManifest(format!(
                        "platform {key}, variant {variant_name}: library path is required"
                    )));
                }

                if variant_info.checksum.is_empty() {
                    return Err(BundleError::InvalidManifest(format!(
                        "platform {key}, variant {variant_name}: checksum is required"
                    )));
                }

                if !variant_info.checksum.starts_with("sha256:") {
                    return Err(BundleError::InvalidManifest(format!(
                        "platform {key}, variant {variant_name}: checksum must start with 'sha256:'"
                    )));
                }
            }
        }

        Ok(())
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> BundleResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> BundleResult<Self> {
        Ok(serde_json::from_str(json)?)
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn Manifest___new___creates_valid_minimal_manifest() {
        let manifest = Manifest::new("test-plugin", "1.0.0");

        assert_eq!(manifest.plugin.name, "test-plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
        assert_eq!(manifest.bundle_version, BUNDLE_VERSION);
        assert!(manifest.platforms.is_empty());
    }

    #[test]
    fn Manifest___add_platform___adds_platform_info() {
        let mut manifest = Manifest::new("test-plugin", "1.0.0");
        manifest.add_platform(
            Platform::LinuxX86_64,
            "lib/linux-x86_64/libtest.so",
            "abc123",
        );

        assert!(manifest.supports_platform(Platform::LinuxX86_64));
        assert!(!manifest.supports_platform(Platform::WindowsX86_64));

        let info = manifest.get_platform(Platform::LinuxX86_64).unwrap();
        let release = info.release().unwrap();
        assert_eq!(release.library, "lib/linux-x86_64/libtest.so");
        assert_eq!(release.checksum, "sha256:abc123");
    }

    #[test]
    fn Manifest___add_platform___overwrites_existing() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/old.so", "old");
        manifest.add_platform(Platform::LinuxX86_64, "lib/new.so", "new");

        let info = manifest.get_platform(Platform::LinuxX86_64).unwrap();
        let release = info.release().unwrap();
        assert_eq!(release.library, "lib/new.so");
        assert_eq!(release.checksum, "sha256:new");
    }

    #[test]
    fn Manifest___validate___rejects_empty_name() {
        let manifest = Manifest::new("", "1.0.0");
        let result = manifest.validate();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("plugin.name"));
    }

    #[test]
    fn Manifest___validate___rejects_empty_version() {
        let manifest = Manifest::new("test", "");
        let result = manifest.validate();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("plugin.version"));
    }

    #[test]
    fn Manifest___validate___rejects_empty_platforms() {
        let manifest = Manifest::new("test", "1.0.0");
        let result = manifest.validate();

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("at least one platform")
        );
    }

    #[test]
    fn Manifest___validate___rejects_invalid_checksum_format() {
        let mut manifest = Manifest::new("test", "1.0.0");
        // Manually insert platform with wrong checksum format (no sha256: prefix)
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "abc123".to_string(), // Missing "sha256:" prefix
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
    fn Manifest___validate___rejects_unknown_platform() {
        let mut manifest = Manifest::new("test", "1.0.0");
        // Manually insert invalid platform key
        manifest.platforms.insert(
            "invalid-platform".to_string(),
            PlatformInfo::new("lib/test.so".to_string(), "sha256:abc123".to_string()),
        );

        let result = manifest.validate();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("unknown platform"));
    }

    #[test]
    fn Manifest___validate___rejects_empty_library_path() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "".to_string(),
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
                .contains("library path is required")
        );
    }

    #[test]
    fn Manifest___validate___rejects_empty_checksum() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let mut variants = HashMap::new();
        variants.insert(
            "release".to_string(),
            VariantInfo {
                library: "lib/test.so".to_string(),
                checksum: "".to_string(),
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
                .contains("checksum is required")
        );
    }

    #[test]
    fn Manifest___validate___rejects_missing_release_variant() {
        let mut manifest = Manifest::new("test", "1.0.0");
        let mut variants = HashMap::new();
        variants.insert(
            "debug".to_string(), // Only debug, no release
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
                .contains("'release' variant is required")
        );
    }

    #[test]
    fn Manifest___validate___rejects_invalid_variant_name() {
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
            "INVALID".to_string(), // Uppercase is invalid
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
    fn Manifest___validate___accepts_valid_manifest() {
        let mut manifest = Manifest::new("test-plugin", "1.0.0");
        manifest.add_platform(
            Platform::LinuxX86_64,
            "lib/linux-x86_64/libtest.so",
            "abc123",
        );

        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn Manifest___validate___accepts_all_platforms() {
        let mut manifest = Manifest::new("all-platforms", "1.0.0");
        for platform in Platform::all() {
            manifest.add_platform(
                *platform,
                &format!("lib/{}/libtest", platform.as_str()),
                "hash",
            );
        }

        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.supported_platforms().len(), 6);
    }

    #[test]
    fn Manifest___json_roundtrip___preserves_data() {
        let mut manifest = Manifest::new("test-plugin", "1.0.0");
        manifest.plugin.description = Some("A test plugin".to_string());
        manifest.add_platform(
            Platform::LinuxX86_64,
            "lib/linux-x86_64/libtest.so",
            "abc123",
        );
        manifest.add_platform(
            Platform::DarwinAarch64,
            "lib/darwin-aarch64/libtest.dylib",
            "def456",
        );

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert_eq!(parsed.plugin.name, manifest.plugin.name);
        assert_eq!(parsed.plugin.version, manifest.plugin.version);
        assert_eq!(parsed.plugin.description, manifest.plugin.description);
        assert_eq!(parsed.platforms.len(), 2);
    }

    #[test]
    fn Manifest___json_roundtrip___preserves_all_plugin_fields() {
        let mut manifest = Manifest::new("full-plugin", "2.3.4");
        manifest.plugin.description = Some("Full description".to_string());
        manifest.plugin.authors = vec!["Author 1".to_string(), "Author 2".to_string()];
        manifest.plugin.license = Some("Apache-2.0".to_string());
        manifest.plugin.repository = Some("https://github.com/test/repo".to_string());
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert_eq!(parsed.plugin.description, manifest.plugin.description);
        assert_eq!(parsed.plugin.authors, manifest.plugin.authors);
        assert_eq!(parsed.plugin.license, manifest.plugin.license);
        assert_eq!(parsed.plugin.repository, manifest.plugin.repository);
    }

    #[test]
    fn Manifest___json_roundtrip___preserves_schemas() {
        let mut manifest = Manifest::new("schema-plugin", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");
        manifest.add_schema(
            "messages.h".to_string(),
            "schema/messages.h".to_string(),
            "c-header".to_string(),
            "sha256:abc".to_string(),
            Some("C header for binary transport".to_string()),
        );

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert_eq!(parsed.schemas.len(), 1);
        let schema = parsed.schemas.get("messages.h").unwrap();
        assert_eq!(schema.path, "schema/messages.h");
        assert_eq!(schema.format, "c-header");
        assert_eq!(schema.checksum, "sha256:abc");
        assert_eq!(
            schema.description,
            Some("C header for binary transport".to_string())
        );
    }

    #[test]
    fn Manifest___json_roundtrip___preserves_public_key() {
        let mut manifest = Manifest::new("signed-plugin", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");
        manifest.set_public_key("RWSxxxxxxxxxxxxxxxx".to_string());

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert_eq!(parsed.public_key, Some("RWSxxxxxxxxxxxxxxxx".to_string()));
    }

    #[test]
    fn Manifest___from_json___invalid_json___returns_error() {
        let result = Manifest::from_json("{ invalid }");

        assert!(result.is_err());
    }

    #[test]
    fn Manifest___from_json___missing_required_fields___returns_error() {
        let result = Manifest::from_json(r#"{"bundle_version": "1.0"}"#);

        assert!(result.is_err());
    }

    #[test]
    fn Manifest___supported_platforms___returns_all_platforms() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/a.so", "a");
        manifest.add_platform(Platform::DarwinAarch64, "lib/b.dylib", "b");

        let platforms = manifest.supported_platforms();
        assert_eq!(platforms.len(), 2);
        assert!(platforms.contains(&Platform::LinuxX86_64));
        assert!(platforms.contains(&Platform::DarwinAarch64));
    }

    #[test]
    fn Manifest___get_platform___returns_none_for_unsupported() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");

        assert!(manifest.get_platform(Platform::LinuxX86_64).is_some());
        assert!(manifest.get_platform(Platform::WindowsX86_64).is_none());
    }

    #[test]
    fn BuildInfo___default___all_fields_none() {
        let build_info = BuildInfo::default();

        assert!(build_info.built_by.is_none());
        assert!(build_info.built_at.is_none());
        assert!(build_info.host.is_none());
        assert!(build_info.compiler.is_none());
        assert!(build_info.rustbridge_version.is_none());
        assert!(build_info.git.is_none());
    }

    #[test]
    fn Manifest___build_info___roundtrip() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");
        manifest.set_build_info(BuildInfo {
            built_by: Some("GitHub Actions".to_string()),
            built_at: Some("2025-01-26T10:30:00Z".to_string()),
            host: Some("x86_64-unknown-linux-gnu".to_string()),
            compiler: Some("rustc 1.90.0".to_string()),
            rustbridge_version: Some("0.2.0".to_string()),
            git: Some(GitInfo {
                commit: "abc123".to_string(),
                branch: Some("main".to_string()),
                tag: Some("v1.0.0".to_string()),
                dirty: Some(false),
            }),
            custom: None,
        });

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        let build_info = parsed.get_build_info().unwrap();
        assert_eq!(build_info.built_by, Some("GitHub Actions".to_string()));
        assert_eq!(build_info.compiler, Some("rustc 1.90.0".to_string()));

        let git = build_info.git.as_ref().unwrap();
        assert_eq!(git.commit, "abc123");
        assert_eq!(git.branch, Some("main".to_string()));
        assert_eq!(git.dirty, Some(false));
    }

    #[test]
    fn Manifest___sbom___roundtrip() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");
        manifest.set_sbom(Sbom {
            cyclonedx: Some("sbom/sbom.cdx.json".to_string()),
            spdx: Some("sbom/sbom.spdx.json".to_string()),
        });

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        let sbom = parsed.get_sbom().unwrap();
        assert_eq!(sbom.cyclonedx, Some("sbom/sbom.cdx.json".to_string()));
        assert_eq!(sbom.spdx, Some("sbom/sbom.spdx.json".to_string()));
    }

    #[test]
    fn Manifest___variants___roundtrip() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform_variant(
            Platform::LinuxX86_64,
            "release",
            "lib/linux-x86_64/release/libtest.so",
            "hash1",
            Some(serde_json::json!({
                "profile": "release",
                "opt_level": "3"
            })),
        );
        manifest.add_platform_variant(
            Platform::LinuxX86_64,
            "debug",
            "lib/linux-x86_64/debug/libtest.so",
            "hash2",
            Some(serde_json::json!({
                "profile": "debug",
                "opt_level": "0"
            })),
        );

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        let variants = parsed.list_variants(Platform::LinuxX86_64);
        assert_eq!(variants.len(), 2);
        assert!(variants.contains(&"release"));
        assert!(variants.contains(&"debug"));

        let release = parsed
            .get_variant(Platform::LinuxX86_64, Some("release"))
            .unwrap();
        assert_eq!(release.library, "lib/linux-x86_64/release/libtest.so");
        assert_eq!(
            release.build.as_ref().unwrap()["profile"],
            serde_json::json!("release")
        );
    }

    #[test]
    fn Manifest___schema_checksum___roundtrip() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");
        manifest.set_schema_checksum("sha256:abcdef123456".to_string());

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert_eq!(parsed.get_schema_checksum(), Some("sha256:abcdef123456"));
    }

    #[test]
    fn Manifest___notices___roundtrip() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");
        manifest.set_notices("docs/NOTICES.txt".to_string());

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert_eq!(parsed.get_notices(), Some("docs/NOTICES.txt"));
    }

    #[test]
    fn Manifest___license_file___roundtrip() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");
        manifest.set_license_file("legal/LICENSE".to_string());

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert_eq!(parsed.get_license_file(), Some("legal/LICENSE"));
    }

    #[test]
    fn is_valid_variant_name___accepts_valid_names() {
        assert!(is_valid_variant_name("release"));
        assert!(is_valid_variant_name("debug"));
        assert!(is_valid_variant_name("nightly"));
        assert!(is_valid_variant_name("opt-size"));
        assert!(is_valid_variant_name("v1"));
        assert!(is_valid_variant_name("build123"));
    }

    #[test]
    fn is_valid_variant_name___rejects_invalid_names() {
        assert!(!is_valid_variant_name("")); // Empty
        assert!(!is_valid_variant_name("RELEASE")); // Uppercase
        assert!(!is_valid_variant_name("Release")); // Mixed case
        assert!(!is_valid_variant_name("-debug")); // Starts with hyphen
        assert!(!is_valid_variant_name("debug-")); // Ends with hyphen
        assert!(!is_valid_variant_name("debug build")); // Contains space
        assert!(!is_valid_variant_name("debug_build")); // Contains underscore
    }

    #[test]
    fn PlatformInfo___new___creates_release_variant() {
        let platform_info =
            PlatformInfo::new("lib/test.so".to_string(), "sha256:abc123".to_string());

        assert!(platform_info.has_variant("release"));
        let release = platform_info.release().unwrap();
        assert_eq!(release.library, "lib/test.so");
        assert_eq!(release.checksum, "sha256:abc123");
    }

    #[test]
    fn PlatformInfo___variant_names___returns_all_variants() {
        let mut platform_info =
            PlatformInfo::new("lib/release.so".to_string(), "sha256:abc".to_string());
        platform_info.add_variant(
            "debug".to_string(),
            VariantInfo {
                library: "lib/debug.so".to_string(),
                checksum: "sha256:def".to_string(),
                build: None,
            },
        );

        let names = platform_info.variant_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"release"));
        assert!(names.contains(&"debug"));
    }

    #[test]
    fn Manifest___has_jni_bridge___returns_false_when_no_bridges() {
        let manifest = Manifest::new("test", "1.0.0");

        assert!(!manifest.has_jni_bridge());
    }

    #[test]
    fn Manifest___add_jni_bridge___adds_bridge_info() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_jni_bridge(
            Platform::LinuxX86_64,
            "release",
            "bridge/jni/linux-x86_64/release/librustbridge_jni.so",
            "abc123",
        );

        assert!(manifest.has_jni_bridge());

        let bridge = manifest.get_jni_bridge(Platform::LinuxX86_64).unwrap();
        let release = bridge.release().unwrap();
        assert_eq!(
            release.library,
            "bridge/jni/linux-x86_64/release/librustbridge_jni.so"
        );
        assert_eq!(release.checksum, "sha256:abc123");
    }

    #[test]
    fn Manifest___add_jni_bridge___multiple_platforms() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_jni_bridge(
            Platform::LinuxX86_64,
            "release",
            "bridge/jni/linux-x86_64/release/librustbridge_jni.so",
            "abc123",
        );
        manifest.add_jni_bridge(
            Platform::DarwinAarch64,
            "release",
            "bridge/jni/darwin-aarch64/release/librustbridge_jni.dylib",
            "def456",
        );

        assert!(manifest.get_jni_bridge(Platform::LinuxX86_64).is_some());
        assert!(manifest.get_jni_bridge(Platform::DarwinAarch64).is_some());
        assert!(manifest.get_jni_bridge(Platform::WindowsX86_64).is_none());
    }

    #[test]
    fn Manifest___jni_bridge___json_roundtrip() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/test.so", "hash");
        manifest.add_jni_bridge(
            Platform::LinuxX86_64,
            "release",
            "bridge/jni/linux-x86_64/release/librustbridge_jni.so",
            "abc123",
        );

        let json = manifest.to_json().unwrap();
        let parsed = Manifest::from_json(&json).unwrap();

        assert!(parsed.has_jni_bridge());
        let bridge = parsed.get_jni_bridge(Platform::LinuxX86_64).unwrap();
        let release = bridge.release().unwrap();
        assert_eq!(
            release.library,
            "bridge/jni/linux-x86_64/release/librustbridge_jni.so"
        );
    }

    #[test]
    fn BridgeInfo___default___empty_jni_map() {
        let bridge_info = BridgeInfo::default();

        assert!(bridge_info.jni.is_empty());
    }
}
