//! Manifest parsing and validation

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// rustbridge.toml manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub plugin: PluginSection,

    #[serde(default)]
    pub messages: HashMap<String, MessageDefinition>,

    #[serde(default)]
    pub platforms: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSection {
    pub name: String,
    pub version: String,

    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub authors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDefinition {
    #[serde(default)]
    pub description: Option<String>,

    #[serde(default)]
    pub request_schema: Option<String>,

    #[serde(default)]
    pub response_schema: Option<String>,
}

impl Manifest {
    /// Load manifest from a file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("Failed to read manifest: {:?}", path.as_ref()))?;

        Self::from_str(&content)
    }

    /// Parse manifest from string
    pub fn from_str(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to parse manifest")
    }

    /// Validate the manifest
    pub fn validate(&self) -> Result<()> {
        // Check required fields
        if self.plugin.name.is_empty() {
            anyhow::bail!("Plugin name cannot be empty");
        }

        if self.plugin.version.is_empty() {
            anyhow::bail!("Plugin version cannot be empty");
        }

        // Validate version format (basic semver check)
        if !self.plugin.version.contains('.') {
            anyhow::bail!("Plugin version should be in semver format (e.g., 1.0.0)");
        }

        // Validate message definitions
        for (tag, def) in &self.messages {
            if tag.is_empty() {
                anyhow::bail!("Message type tag cannot be empty");
            }

            // Check schema files exist if specified
            if let Some(schema) = &def.request_schema {
                // Just validate the path looks reasonable for now
                if schema.is_empty() {
                    anyhow::bail!("Request schema path cannot be empty for message '{}'", tag);
                }
            }
        }

        // Validate platform keys
        for platform in self.platforms.keys() {
            if !is_valid_platform(platform) {
                anyhow::bail!("Invalid platform key: {}", platform);
            }
        }

        Ok(())
    }
}

/// Check if a platform string is valid
fn is_valid_platform(platform: &str) -> bool {
    let valid_platforms = [
        "linux-x86_64",
        "linux-aarch64",
        "darwin-x86_64",
        "darwin-aarch64",
        "windows-x86_64",
        "windows-aarch64",
    ];

    valid_platforms.contains(&platform)
}

/// Check command implementation
pub fn check(manifest_path: Option<String>) -> Result<()> {
    let path = manifest_path.unwrap_or_else(|| "rustbridge.toml".to_string());

    println!("Checking manifest: {}", path);

    let manifest = Manifest::from_file(&path)?;
    manifest.validate()?;

    println!(
        "✓ Plugin: {} v{}",
        manifest.plugin.name, manifest.plugin.version
    );
    println!("✓ Messages: {}", manifest.messages.len());
    println!("✓ Platforms: {}", manifest.platforms.len());
    println!("\nManifest is valid!");

    Ok(())
}

#[cfg(test)]
#[path = "manifest/manifest_tests.rs"]
mod manifest_tests;
