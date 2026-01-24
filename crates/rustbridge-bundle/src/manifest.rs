//! Manifest schema for plugin bundles.
//!
//! The manifest describes the plugin metadata, supported platforms,
//! and available API messages.

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

    /// API information (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api: Option<ApiInfo>,

    /// Minisign public key for signature verification (base64-encoded).
    /// Format: "RWS..." (standard minisign public key format).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,

    /// Schema files embedded in the bundle.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub schemas: HashMap<String, SchemaInfo>,
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

/// Platform-specific library information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Relative path to the library within the bundle.
    pub library: String,

    /// SHA256 checksum of the library file.
    pub checksum: String,
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

/// API information describing available messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfo {
    /// Minimum rustbridge version required.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_rustbridge_version: Option<String>,

    /// Supported transport types (e.g., ["json", "cstruct"]).
    #[serde(default)]
    pub transports: Vec<String>,

    /// Available messages.
    #[serde(default)]
    pub messages: Vec<MessageInfo>,
}

/// Information about a single message type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    /// Message type tag (e.g., "user.create").
    pub type_tag: String,

    /// Human-readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// JSON Schema reference for the request type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_schema: Option<String>,

    /// JSON Schema reference for the response type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_schema: Option<String>,

    /// Numeric message ID for binary transport.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message_id: Option<u32>,

    /// C struct name for request (binary transport).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cstruct_request: Option<String>,

    /// C struct name for response (binary transport).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cstruct_response: Option<String>,
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
            api: None,
            public_key: None,
            schemas: HashMap::new(),
        }
    }

    /// Add a platform to the manifest.
    pub fn add_platform(&mut self, platform: Platform, library_path: &str, checksum: &str) {
        self.platforms.insert(
            platform.as_str().to_string(),
            PlatformInfo {
                library: library_path.to_string(),
                checksum: format!("sha256:{checksum}"),
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

            if info.library.is_empty() {
                return Err(BundleError::InvalidManifest(format!(
                    "platform {key}: library path is required"
                )));
            }

            if info.checksum.is_empty() {
                return Err(BundleError::InvalidManifest(format!(
                    "platform {key}: checksum is required"
                )));
            }

            if !info.checksum.starts_with("sha256:") {
                return Err(BundleError::InvalidManifest(format!(
                    "platform {key}: checksum must start with 'sha256:'"
                )));
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

impl Default for ApiInfo {
    fn default() -> Self {
        Self {
            min_rustbridge_version: None,
            transports: vec!["json".to_string()],
            messages: Vec::new(),
        }
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
        assert_eq!(info.library, "lib/linux-x86_64/libtest.so");
        assert_eq!(info.checksum, "sha256:abc123");
    }

    #[test]
    fn Manifest___validate___rejects_empty_name() {
        let manifest = Manifest::new("", "1.0.0");
        let result = manifest.validate();

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("plugin.name"));
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
    fn Manifest___supported_platforms___returns_all_platforms() {
        let mut manifest = Manifest::new("test", "1.0.0");
        manifest.add_platform(Platform::LinuxX86_64, "lib/a.so", "a");
        manifest.add_platform(Platform::DarwinAarch64, "lib/b.dylib", "b");

        let platforms = manifest.supported_platforms();
        assert_eq!(platforms.len(), 2);
        assert!(platforms.contains(&Platform::LinuxX86_64));
        assert!(platforms.contains(&Platform::DarwinAarch64));
    }
}
