//! Bundle creation command.
//!
//! Creates `.rbp` bundles from plugin libraries and manifests.

use anyhow::{Context, Result};
use rustbridge_bundle::{BundleBuilder, Manifest, Platform};
use std::path::Path;

/// Run the bundle command.
pub fn run(
    name: &str,
    version: &str,
    libraries: &[(String, String)],
    output: Option<String>,
    schema_files: &[(String, String)],
) -> Result<()> {
    println!("Creating bundle: {name} v{version}");

    // Create manifest
    let manifest = Manifest::new(name, version);
    let mut builder = BundleBuilder::new(manifest);

    // Add libraries
    for (platform_str, lib_path) in libraries {
        let platform = Platform::parse(platform_str)
            .with_context(|| format!("Unknown platform: {platform_str}"))?;

        println!("  Adding library: {lib_path} ({platform_str})");
        builder = builder
            .add_library(platform, lib_path)
            .with_context(|| format!("Failed to add library: {lib_path}"))?;
    }

    // Add schema files
    for (source, archive_name) in schema_files {
        println!("  Adding schema: {source} -> schema/{archive_name}");
        builder = builder
            .add_schema_file(source, archive_name)
            .with_context(|| format!("Failed to add schema file: {source}"))?;
    }

    // Determine output path
    let output_path = output.unwrap_or_else(|| format!("{name}-{version}.rbp"));
    let output_path = Path::new(&output_path);

    // Write the bundle
    builder
        .write(output_path)
        .with_context(|| format!("Failed to write bundle: {}", output_path.display()))?;

    println!("Bundle created: {}", output_path.display());
    Ok(())
}

/// List contents of a bundle.
pub fn list(bundle_path: &str) -> Result<()> {
    use rustbridge_bundle::BundleLoader;

    let loader = BundleLoader::open(bundle_path)
        .with_context(|| format!("Failed to open: {bundle_path}"))?;

    let manifest = loader.manifest();
    println!(
        "Bundle: {} v{}",
        manifest.plugin.name, manifest.plugin.version
    );
    println!("Bundle format: v{}", manifest.bundle_version);

    if let Some(desc) = &manifest.plugin.description {
        println!("Description: {desc}");
    }

    println!("\nPlatforms:");
    for (platform, info) in &manifest.platforms {
        println!("  {platform}:");
        println!("    Library: {}", info.library);
        println!("    Checksum: {}", info.checksum);
    }

    println!("\nFiles:");
    for file in loader.list_files() {
        println!("  {file}");
    }

    Ok(())
}

/// Extract a library from a bundle.
pub fn extract(bundle_path: &str, platform: Option<String>, output_dir: &str) -> Result<()> {
    use rustbridge_bundle::BundleLoader;

    let mut loader = BundleLoader::open(bundle_path)
        .with_context(|| format!("Failed to open: {bundle_path}"))?;

    let extracted_path = if let Some(platform_str) = platform {
        let platform = Platform::parse(&platform_str)
            .with_context(|| format!("Unknown platform: {platform_str}"))?;

        loader
            .extract_library(platform, output_dir)
            .with_context(|| format!("Failed to extract library for {platform_str}"))?
    } else {
        loader
            .extract_library_for_current_platform(output_dir)
            .context("Failed to extract library for current platform")?
    };

    println!("Extracted: {}", extracted_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn run___creates_bundle_with_library() {
        let temp_dir = TempDir::new().unwrap();

        // Create a fake library
        let lib_path = temp_dir.path().join("libtest.so");
        fs::write(&lib_path, b"fake library").unwrap();

        // Create bundle
        let output = temp_dir.path().join("test.rbp");
        let libs = vec![(
            "linux-x86_64".to_string(),
            lib_path.to_string_lossy().to_string(),
        )];

        run(
            "test-plugin",
            "1.0.0",
            &libs,
            Some(output.to_string_lossy().to_string()),
            &[],
        )
        .unwrap();

        assert!(output.exists());
    }

    #[test]
    fn list___shows_bundle_contents() {
        let temp_dir = TempDir::new().unwrap();

        // Create a fake library
        let lib_path = temp_dir.path().join("libtest.so");
        fs::write(&lib_path, b"fake library").unwrap();

        // Create bundle
        let output = temp_dir.path().join("test.rbp");
        let libs = vec![(
            "linux-x86_64".to_string(),
            lib_path.to_string_lossy().to_string(),
        )];

        run(
            "test-plugin",
            "1.0.0",
            &libs,
            Some(output.to_string_lossy().to_string()),
            &[],
        )
        .unwrap();

        // List should succeed
        list(&output.to_string_lossy()).unwrap();
    }
}
