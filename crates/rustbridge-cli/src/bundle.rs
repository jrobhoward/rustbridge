//! Bundle creation command.
//!
//! Creates `.rbp` bundles from plugin libraries and manifests.

use anyhow::{Context, Result};
use minisign::{SecretKey, SecretKeyBox};
use rustbridge_bundle::{BundleBuilder, Manifest, Platform};
use std::fs;
use std::path::Path;

/// Run the bundle command.
#[allow(clippy::too_many_arguments)]
pub fn run(
    name: &str,
    version: &str,
    libraries: &[(String, String)],
    output: Option<String>,
    schema_files: &[(String, String)],
    sign_key_path: Option<String>,
    generate_header: Option<String>,
    generate_schema: Option<String>,
) -> Result<()> {
    println!("Creating bundle: {name} v{version}");

    // Create manifest
    let manifest = Manifest::new(name, version);
    let mut builder = BundleBuilder::new(manifest);

    // Load signing key if provided
    if let Some(key_path) = sign_key_path {
        println!("  Loading signing key: {key_path}");

        let (public_key, secret_key) = load_signing_key(&key_path)
            .with_context(|| format!("Failed to load signing key from {key_path}"))?;

        builder = builder.with_signing_key(public_key, secret_key);
        println!("  Bundle will be signed");
    }

    // Add libraries
    for (platform_str, lib_path) in libraries {
        let platform = Platform::parse(platform_str)
            .with_context(|| format!("Unknown platform: {platform_str}"))?;

        println!("  Adding library: {lib_path} ({platform_str})");
        builder = builder
            .add_library(platform, lib_path)
            .with_context(|| format!("Failed to add library: {lib_path}"))?;
    }

    // Generate and add C header if requested
    if let Some(header_spec) = generate_header {
        let parts: Vec<&str> = header_spec.splitn(2, ':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid generate-header format: {header_spec}. Expected SOURCE:HEADER_NAME"
            );
        }
        let (source_file, header_name) = (parts[0], parts[1]);

        println!("  Generating C header: {source_file} -> schema/{header_name}");

        // Generate header to a temporary file
        let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
        let temp_header = temp_dir.path().join(header_name);

        crate::header_gen::run(source_file, temp_header.to_str().unwrap(), false)
            .with_context(|| format!("Failed to generate C header from {source_file}"))?;

        // Add the generated header to the bundle
        builder = builder
            .add_schema_file(&temp_header, header_name)
            .with_context(|| format!("Failed to add generated header: {header_name}"))?;
    }

    // Generate and add JSON Schema if requested
    if let Some(schema_spec) = generate_schema {
        let parts: Vec<&str> = schema_spec.splitn(2, ':').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "Invalid generate-schema format: {schema_spec}. Expected SOURCE:SCHEMA_NAME"
            );
        }
        let (source_file, schema_name) = (parts[0], parts[1]);

        println!("  Generating JSON Schema: {source_file} -> schema/{schema_name}");

        // Parse message types from Rust source
        use crate::codegen::{MessageType, generate_json_schema};
        let messages = MessageType::parse_file(Path::new(source_file))
            .with_context(|| format!("Failed to parse Rust source: {source_file}"))?;

        // Generate JSON Schema
        let schema = generate_json_schema(&messages)
            .with_context(|| format!("Failed to generate JSON Schema from {source_file}"))?;

        // Write to temporary file
        let temp_dir = tempfile::tempdir().context("Failed to create temp directory")?;
        let temp_schema = temp_dir.path().join(schema_name);
        fs::write(&temp_schema, serde_json::to_string_pretty(&schema)?)
            .context("Failed to write JSON Schema to temp file")?;

        // Add the generated schema to the bundle
        builder = builder
            .add_schema_file(&temp_schema, schema_name)
            .with_context(|| format!("Failed to add generated schema: {schema_name}"))?;
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
        for (variant_name, variant_info) in &info.variants {
            println!("    Variant: {variant_name}");
            println!("      Library: {}", variant_info.library);
            println!("      Checksum: {}", variant_info.checksum);
            if let Some(build) = &variant_info.build {
                println!("      Build: {}", build);
            }
        }
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

/// Load a signing key from a file.
///
/// Prompts the user for the password to decrypt the key.
/// Returns (public_key_base64, secret_key).
fn load_signing_key(key_path: &str) -> Result<(String, SecretKey)> {
    // Read the secret key file
    let key_str = fs::read_to_string(key_path).context("Failed to read key file")?;

    // Parse as secret key box
    let secret_key_box = SecretKeyBox::from_string(&key_str).context("Invalid key file format")?;

    // Read the public key file (same path with .pub extension)
    // Use with_extension to match keygen behavior: signing.key -> signing.pub
    let pub_key_path = std::path::Path::new(key_path).with_extension("pub");
    let pub_key_data = fs::read_to_string(&pub_key_path)
        .with_context(|| format!("Failed to read public key file: {}", pub_key_path.display()))?;
    let public_key = pub_key_data.trim().to_string();

    // Prompt for password
    println!("Enter password for signing key: ");
    let password = rpassword::read_password().context("Failed to read password")?;

    // Decrypt the key
    let secret_key = secret_key_box
        .into_secret_key(Some(password))
        .map_err(|_| anyhow::anyhow!("Invalid password or corrupted key file"))?;

    Ok((public_key, secret_key))
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
            None, // No signing
            None, // No header generation
            None, // No schema generation
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
            None, // No signing
            None, // No header generation
            None, // No schema generation
        )
        .unwrap();

        // List should succeed
        list(&output.to_string_lossy()).unwrap();
    }
}
