//! Bundle creation and manipulation commands.
//!
//! Creates, combines, slims, and extracts `.rbp` bundles.

use anyhow::{Context, Result};
use minisign::{SecretKey, SecretKeyBox};
use rustbridge_bundle::{BundleBuilder, BundleLoader, Manifest, Platform};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Create a new bundle from libraries.
#[allow(clippy::too_many_arguments)]
pub fn create(
    name: &str,
    version: &str,
    libraries: &[(String, String, String)], // (platform, variant, path)
    output: Option<String>,
    schema_files: &[(String, String)],
    sign_key_path: Option<String>,
    generate_header: Option<String>,
    generate_schema: Option<String>,
    notices_path: Option<String>,
    license_path: Option<String>,
    no_metadata: bool,
    sbom_files: &[(String, String)],
    custom_metadata: &[(String, String)],
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

    // Add libraries with variant support
    for (platform_str, variant, lib_path) in libraries {
        let platform = Platform::parse(platform_str)
            .with_context(|| format!("Unknown platform: {platform_str}"))?;

        println!("  Adding library: {lib_path} ({platform_str}:{variant})");
        builder = builder
            .add_library_variant(platform, variant, lib_path)
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

    // Add notices file if provided
    if let Some(notices) = notices_path {
        println!("  Adding notices: {notices}");
        builder = builder
            .add_notices_file(&notices)
            .with_context(|| format!("Failed to add notices file: {notices}"))?;
    }

    // Add license file if provided
    if let Some(license) = license_path {
        println!("  Adding license: {license}");
        builder = builder
            .add_license_file(&license)
            .with_context(|| format!("Failed to add license file: {license}"))?;
    }

    // Add SBOM files
    if !sbom_files.is_empty() {
        let mut sbom = rustbridge_bundle::Sbom {
            cyclonedx: None,
            spdx: None,
        };

        for (source, archive_name) in sbom_files {
            println!("  Adding SBOM: {source} -> sbom/{archive_name}");
            builder = builder
                .add_sbom_file(source, archive_name)
                .with_context(|| format!("Failed to add SBOM file: {source}"))?;

            // Update SBOM metadata based on file name
            let archive_path = format!("sbom/{archive_name}");
            if archive_name.contains("cdx") || archive_name.contains("cyclonedx") {
                sbom.cyclonedx = Some(archive_path);
            } else if archive_name.contains("spdx") {
                sbom.spdx = Some(archive_path);
            }
        }

        builder = builder.with_sbom(sbom);
    }

    // Add build metadata if not disabled
    if !no_metadata {
        let mut build_info = collect_build_info();

        // Add custom metadata if provided
        if !custom_metadata.is_empty() {
            let custom: HashMap<String, String> = custom_metadata.iter().cloned().collect();
            build_info.custom = Some(custom);
            for (key, value) in custom_metadata {
                println!("  Custom metadata: {key}={value}");
            }
        }

        builder = builder.with_build_info(build_info);
        println!("  Build metadata collected");
    } else if !custom_metadata.is_empty() {
        // Even with --no-metadata, allow custom metadata
        let custom: HashMap<String, String> = custom_metadata.iter().cloned().collect();
        let build_info = rustbridge_bundle::BuildInfo {
            custom: Some(custom),
            ..Default::default()
        };
        builder = builder.with_build_info(build_info);
        for (key, value) in custom_metadata {
            println!("  Custom metadata: {key}={value}");
        }
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

/// Combine multiple bundles into one.
pub fn combine(
    bundle_paths: &[String],
    output_path: &str,
    sign_key_path: Option<String>,
    schema_mismatch_mode: &str,
) -> Result<()> {
    println!(
        "Combining {} bundles into {output_path}",
        bundle_paths.len()
    );

    if bundle_paths.len() < 2 {
        anyhow::bail!("At least 2 bundles are required for combining");
    }

    // Load all input bundles
    let mut loaders: Vec<BundleLoader> = Vec::new();
    for path in bundle_paths {
        let loader =
            BundleLoader::open(path).with_context(|| format!("Failed to open bundle: {path}"))?;
        loaders.push(loader);
    }

    // Get reference manifest from first bundle
    let first_manifest = loaders[0].manifest();

    // Check schema checksums
    let first_checksum = first_manifest.get_schema_checksum();
    for (i, loader) in loaders.iter().enumerate().skip(1) {
        let this_checksum = loader.manifest().get_schema_checksum();
        if first_checksum != this_checksum {
            let msg = format!(
                "Schema checksum mismatch between {} and {}",
                bundle_paths[0], bundle_paths[i]
            );
            match schema_mismatch_mode {
                "error" => anyhow::bail!("{msg}"),
                "warn" => eprintln!("Warning: {msg}"),
                "ignore" => {}
                _ => anyhow::bail!("Invalid schema-mismatch mode: {schema_mismatch_mode}"),
            }
        }
    }

    // Create new manifest based on first bundle
    let mut manifest = Manifest::new(&first_manifest.plugin.name, &first_manifest.plugin.version);
    manifest.plugin.description = first_manifest.plugin.description.clone();
    manifest.plugin.authors = first_manifest.plugin.authors.clone();
    manifest.plugin.license = first_manifest.plugin.license.clone();
    manifest.plugin.repository = first_manifest.plugin.repository.clone();

    // Copy API info from first bundle
    if let Some(api) = &first_manifest.api {
        manifest.api = Some(api.clone());
    }

    // Copy schema checksum from first bundle
    if let Some(checksum) = first_manifest.get_schema_checksum() {
        manifest.set_schema_checksum(checksum.to_string());
    }

    let mut builder = BundleBuilder::new(manifest);

    // Load signing key if provided
    if let Some(key_path) = &sign_key_path {
        println!("  Loading signing key: {key_path}");
        let (public_key, secret_key) = load_signing_key(key_path)
            .with_context(|| format!("Failed to load signing key from {key_path}"))?;
        builder = builder.with_signing_key(public_key, secret_key);
    }

    // Track platforms already added to detect conflicts
    let mut added_platforms: HashMap<String, HashMap<String, String>> = HashMap::new();

    // Collect platform info from all bundles first (to avoid borrow issues)
    struct LibraryEntry {
        platform_str: String,
        variant_name: String,
        library_path: String,
        checksum: String,
        build: Option<serde_json::Value>,
        bundle_idx: usize,
    }

    let mut library_entries: Vec<LibraryEntry> = Vec::new();

    for (bundle_idx, loader) in loaders.iter().enumerate() {
        let manifest = loader.manifest();

        for (platform_str, platform_info) in &manifest.platforms {
            for (variant_name, variant_info) in &platform_info.variants {
                library_entries.push(LibraryEntry {
                    platform_str: platform_str.clone(),
                    variant_name: variant_name.clone(),
                    library_path: variant_info.library.clone(),
                    checksum: variant_info.checksum.clone(),
                    build: variant_info.build.clone(),
                    bundle_idx,
                });
            }
        }
    }

    // Now merge platforms from all bundles
    for entry in library_entries {
        // Check for conflicts
        if let Some(variants) = added_platforms.get(&entry.platform_str)
            && variants.contains_key(&entry.variant_name)
        {
            anyhow::bail!(
                "Duplicate platform/variant: {}:{} (in {} and {})",
                entry.platform_str,
                entry.variant_name,
                variants.get(&entry.variant_name).unwrap(),
                bundle_paths[entry.bundle_idx]
            );
        }

        // Read the library file from this bundle
        let lib_contents = loaders[entry.bundle_idx].read_file(&entry.library_path)?;

        // Determine archive path in combined bundle
        let file_name = Path::new(&entry.library_path)
            .file_name()
            .unwrap()
            .to_string_lossy();
        let archive_path = format!(
            "lib/{}/{}/{}",
            entry.platform_str, entry.variant_name, file_name
        );

        // Add to builder
        builder = builder.add_bytes(&archive_path, lib_contents);

        // Update manifest
        builder.manifest_mut().add_platform_variant(
            Platform::parse(&entry.platform_str).unwrap(),
            &entry.variant_name,
            &archive_path,
            entry
                .checksum
                .strip_prefix("sha256:")
                .unwrap_or(&entry.checksum),
            entry.build,
        );

        // Track as added
        added_platforms
            .entry(entry.platform_str.clone())
            .or_default()
            .insert(
                entry.variant_name.clone(),
                bundle_paths[entry.bundle_idx].clone(),
            );

        println!(
            "  Added: {}:{} from {}",
            entry.platform_str, entry.variant_name, bundle_paths[entry.bundle_idx]
        );
    }

    // Copy schemas from first bundle
    let first_loader = &mut loaders[0];
    for file in first_loader.list_files() {
        if file.starts_with("schema/") {
            let contents = first_loader.read_file(&file)?;
            builder = builder.add_bytes(&file, contents);
        }
    }

    // Copy docs from first bundle
    for file in first_loader.list_files() {
        if file.starts_with("docs/") {
            let contents = first_loader.read_file(&file)?;
            builder = builder.add_bytes(&file, contents);
        }
    }

    // Copy license file from first bundle
    for file in first_loader.list_files() {
        if file.starts_with("legal/") {
            let contents = first_loader.read_file(&file)?;
            builder = builder.add_bytes(&file, contents);
        }
    }

    // Write the combined bundle
    builder
        .write(output_path)
        .with_context(|| format!("Failed to write combined bundle: {output_path}"))?;

    println!("Combined bundle created: {output_path}");
    Ok(())
}

/// Create a slimmed bundle with a subset of platforms/variants.
pub fn slim(
    input_path: &str,
    output_path: &str,
    platforms: Option<Vec<String>>,
    variants: Vec<String>,
    exclude_docs: bool,
    sign_key_path: Option<String>,
) -> Result<()> {
    println!("Slimming bundle: {input_path} -> {output_path}");

    let mut loader =
        BundleLoader::open(input_path).with_context(|| format!("Failed to open: {input_path}"))?;

    let source_manifest = loader.manifest().clone();

    // Create new manifest
    let mut manifest = Manifest::new(
        &source_manifest.plugin.name,
        &source_manifest.plugin.version,
    );
    manifest.plugin.description = source_manifest.plugin.description.clone();
    manifest.plugin.authors = source_manifest.plugin.authors.clone();
    manifest.plugin.license = source_manifest.plugin.license.clone();
    manifest.plugin.repository = source_manifest.plugin.repository.clone();
    manifest.api = source_manifest.api.clone();

    // Copy schema checksum if present
    if let Some(checksum) = source_manifest.get_schema_checksum() {
        manifest.set_schema_checksum(checksum.to_string());
    }

    let mut builder = BundleBuilder::new(manifest);

    // Load signing key if provided
    if let Some(key_path) = &sign_key_path {
        println!("  Loading signing key: {key_path}");
        let (public_key, secret_key) = load_signing_key(key_path)
            .with_context(|| format!("Failed to load signing key from {key_path}"))?;
        builder = builder.with_signing_key(public_key, secret_key);
    }

    // Filter and copy platforms/variants
    for (platform_str, platform_info) in &source_manifest.platforms {
        // Check if this platform should be included
        if let Some(ref allowed_platforms) = platforms
            && !allowed_platforms.contains(platform_str)
        {
            println!("  Skipping platform: {platform_str}");
            continue;
        }

        for (variant_name, variant_info) in &platform_info.variants {
            // Check if this variant should be included
            if !variants.contains(variant_name) {
                println!("  Skipping variant: {platform_str}:{variant_name}");
                continue;
            }

            // Read and copy the library
            let lib_contents = loader.read_file(&variant_info.library)?;
            let file_name = Path::new(&variant_info.library)
                .file_name()
                .unwrap()
                .to_string_lossy();
            let archive_path = format!("lib/{platform_str}/{variant_name}/{file_name}");

            builder = builder.add_bytes(&archive_path, lib_contents);

            // Update manifest
            builder.manifest_mut().add_platform_variant(
                Platform::parse(platform_str).unwrap(),
                variant_name,
                &archive_path,
                variant_info
                    .checksum
                    .strip_prefix("sha256:")
                    .unwrap_or(&variant_info.checksum),
                variant_info.build.clone(),
            );

            println!("  Included: {platform_str}:{variant_name}");
        }
    }

    // Copy schemas
    for file in loader.list_files() {
        if file.starts_with("schema/") {
            let contents = loader.read_file(&file)?;
            builder = builder.add_bytes(&file, contents);
        }
    }

    // Copy docs unless excluded
    if !exclude_docs {
        for file in loader.list_files() {
            if file.starts_with("docs/") {
                let contents = loader.read_file(&file)?;
                builder = builder.add_bytes(&file, contents);
            }
        }
    } else {
        println!("  Excluding documentation files");
    }

    // Copy SBOM if present
    for file in loader.list_files() {
        if file.starts_with("sbom/") {
            let contents = loader.read_file(&file)?;
            builder = builder.add_bytes(&file, contents);
        }
    }

    // Copy license file if present
    for file in loader.list_files() {
        if file.starts_with("legal/") {
            let contents = loader.read_file(&file)?;
            builder = builder.add_bytes(&file, contents);
        }
    }

    // Write the slimmed bundle
    builder
        .write(output_path)
        .with_context(|| format!("Failed to write slimmed bundle: {output_path}"))?;

    println!("Slimmed bundle created: {output_path}");
    Ok(())
}

/// List contents of a bundle.
pub fn list(bundle_path: &str, show_build: bool, show_variants: bool) -> Result<()> {
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

    // Show build info if requested
    if show_build && let Some(build_info) = manifest.get_build_info() {
        println!("\nBuild Info:");
        if let Some(built_by) = &build_info.built_by {
            println!("  Built by: {built_by}");
        }
        if let Some(built_at) = &build_info.built_at {
            println!("  Built at: {built_at}");
        }
        if let Some(host) = &build_info.host {
            println!("  Host: {host}");
        }
        if let Some(compiler) = &build_info.compiler {
            println!("  Compiler: {compiler}");
        }
        if let Some(git) = &build_info.git {
            println!("  Git commit: {}", git.commit);
            if let Some(branch) = &git.branch {
                println!("  Git branch: {branch}");
            }
            if let Some(tag) = &git.tag {
                println!("  Git tag: {tag}");
            }
            if let Some(dirty) = git.dirty {
                println!("  Dirty: {dirty}");
            }
        }
        if let Some(custom) = &build_info.custom {
            println!("  Custom metadata:");
            for (key, value) in custom {
                println!("    {key}: {value}");
            }
        }
    }

    println!("\nPlatforms:");
    for (platform, info) in &manifest.platforms {
        println!("  {platform}:");
        if show_variants {
            for (variant_name, variant_info) in &info.variants {
                println!("    Variant: {variant_name}");
                println!("      Library: {}", variant_info.library);
                println!("      Checksum: {}", variant_info.checksum);
                if let Some(build) = &variant_info.build {
                    println!("      Build: {}", build);
                }
            }
        } else {
            // Just show variant names
            let variant_names: Vec<&str> = info.variants.keys().map(|s| s.as_str()).collect();
            println!("    Variants: {}", variant_names.join(", "));
        }
    }

    if let Some(sbom) = manifest.get_sbom() {
        println!("\nSBOM:");
        if let Some(cdx) = &sbom.cyclonedx {
            println!("  CycloneDX: {cdx}");
        }
        if let Some(spdx) = &sbom.spdx {
            println!("  SPDX: {spdx}");
        }
    }

    if let Some(license_file) = manifest.get_license_file() {
        println!("\nLicense: {license_file}");
    }

    println!("\nFiles:");
    for file in loader.list_files() {
        println!("  {file}");
    }

    Ok(())
}

/// Extract a library from a bundle.
pub fn extract(
    bundle_path: &str,
    platform: Option<String>,
    variant: &str,
    output_dir: &str,
) -> Result<()> {
    let mut loader = BundleLoader::open(bundle_path)
        .with_context(|| format!("Failed to open: {bundle_path}"))?;

    let extracted_path = if let Some(platform_str) = platform {
        let platform = Platform::parse(&platform_str)
            .with_context(|| format!("Unknown platform: {platform_str}"))?;

        loader
            .extract_library_variant(platform, variant, output_dir)
            .with_context(|| format!("Failed to extract library for {platform_str}:{variant}"))?
    } else {
        let platform =
            Platform::current().with_context(|| "Current platform is not supported".to_string())?;

        loader
            .extract_library_variant(platform, variant, output_dir)
            .context("Failed to extract library for current platform")?
    };

    println!("Extracted: {}", extracted_path.display());
    Ok(())
}

/// Collect build information.
fn collect_build_info() -> rustbridge_bundle::BuildInfo {
    use rustbridge_bundle::BuildInfo;

    // Detect CI environment
    let built_by = if std::env::var("GITHUB_ACTIONS").is_ok() {
        Some("GitHub Actions".to_string())
    } else if std::env::var("GITLAB_CI").is_ok() {
        Some("GitLab CI".to_string())
    } else if std::env::var("CI").is_ok() {
        Some("CI".to_string())
    } else {
        Some("local".to_string())
    };

    // Get rustc version if available
    let compiler = std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|v| v.trim().to_string());

    BuildInfo {
        built_at: Some(chrono_lite_now()),
        built_by,
        host: None,
        compiler,
        rustbridge_version: Some(env!("CARGO_PKG_VERSION").to_string()),
        git: collect_git_info(),
        custom: None,
    }
}

/// Collect git information.
fn collect_git_info() -> Option<rustbridge_bundle::GitInfo> {
    use rustbridge_bundle::GitInfo;

    // Get commit hash
    let commit_output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;

    if !commit_output.status.success() {
        return None;
    }

    let commit = String::from_utf8(commit_output.stdout)
        .ok()?
        .trim()
        .to_string();

    let mut git_info = GitInfo {
        commit,
        branch: None,
        tag: None,
        dirty: None,
    };

    // Get branch name
    if let Ok(output) = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        && output.status.success()
        && let Ok(branch) = String::from_utf8(output.stdout)
    {
        let branch = branch.trim();
        if branch != "HEAD" {
            git_info.branch = Some(branch.to_string());
        }
    }

    // Get tag if on a tagged commit
    if let Ok(output) = std::process::Command::new("git")
        .args(["describe", "--tags", "--exact-match"])
        .output()
        && output.status.success()
        && let Ok(tag) = String::from_utf8(output.stdout)
    {
        git_info.tag = Some(tag.trim().to_string());
    }

    // Check if dirty
    if let Ok(output) = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        && output.status.success()
    {
        git_info.dirty = Some(!output.stdout.is_empty());
    }

    Some(git_info)
}

/// Get current timestamp in ISO 8601 format (simple implementation).
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Simple conversion - not perfectly accurate but good enough for timestamps
    let days = now / 86400;
    let remaining = now % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    // Approximate date calculation (simplified, ignores leap years properly)
    let mut year = 1970;
    let mut remaining_days = days;

    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
            366
        } else {
            365
        };

        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let is_leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for days_in_month in days_in_months.iter() {
        if remaining_days < *days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }

    let day = remaining_days + 1;

    format!("{year:04}-{month:02}-{day:02}T{hours:02}:{minutes:02}:{seconds:02}Z")
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
    // Minisign .pub format is two lines:
    //   untrusted comment: minisign public key <key_id>
    //   <base64_public_key>
    let pub_key_path = std::path::Path::new(key_path).with_extension("pub");
    let pub_key_data = fs::read_to_string(&pub_key_path)
        .with_context(|| format!("Failed to read public key file: {}", pub_key_path.display()))?;
    let public_key = pub_key_data
        .lines()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Invalid public key file format: expected 2 lines"))?
        .trim()
        .to_string();

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
    fn create___creates_bundle_with_library() {
        let temp_dir = TempDir::new().unwrap();

        // Create a fake library
        let lib_path = temp_dir.path().join("libtest.so");
        fs::write(&lib_path, b"fake library").unwrap();

        // Create bundle
        let output = temp_dir.path().join("test.rbp");
        let libs = vec![(
            "linux-x86_64".to_string(),
            "release".to_string(),
            lib_path.to_string_lossy().to_string(),
        )];

        create(
            "test-plugin",
            "1.0.0",
            &libs,
            Some(output.to_string_lossy().to_string()),
            &[],
            None,
            None,
            None,
            None,
            None, // No license file
            true, // Skip metadata for test
            &[],  // No SBOM files
            &[],  // No custom metadata
        )
        .unwrap();

        assert!(output.exists());
    }

    #[test]
    fn create___creates_bundle_with_multiple_variants() {
        let temp_dir = TempDir::new().unwrap();

        // Create fake libraries
        let release_lib = temp_dir.path().join("libtest_release.so");
        let debug_lib = temp_dir.path().join("libtest_debug.so");
        fs::write(&release_lib, b"release library").unwrap();
        fs::write(&debug_lib, b"debug library").unwrap();

        // Create bundle with multiple variants
        let output = temp_dir.path().join("test.rbp");
        let libs = vec![
            (
                "linux-x86_64".to_string(),
                "release".to_string(),
                release_lib.to_string_lossy().to_string(),
            ),
            (
                "linux-x86_64".to_string(),
                "debug".to_string(),
                debug_lib.to_string_lossy().to_string(),
            ),
        ];

        create(
            "test-plugin",
            "1.0.0",
            &libs,
            Some(output.to_string_lossy().to_string()),
            &[],
            None,
            None,
            None,
            None,
            None,
            true,
            &[],
            &[],
        )
        .unwrap();

        // Verify bundle has both variants
        let loader = BundleLoader::open(&output).unwrap();
        let variants = loader.list_variants(Platform::LinuxX86_64);
        assert!(variants.contains(&"release"));
        assert!(variants.contains(&"debug"));
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
            "release".to_string(),
            lib_path.to_string_lossy().to_string(),
        )];

        create(
            "test-plugin",
            "1.0.0",
            &libs,
            Some(output.to_string_lossy().to_string()),
            &[],
            None,
            None,
            None,
            None,
            None,
            true,
            &[],
            &[],
        )
        .unwrap();

        // List should succeed
        list(&output.to_string_lossy(), false, false).unwrap();
    }

    #[test]
    fn slim___extracts_subset_of_variants() {
        let temp_dir = TempDir::new().unwrap();

        // Create fake libraries
        let release_lib = temp_dir.path().join("libtest_release.so");
        let debug_lib = temp_dir.path().join("libtest_debug.so");
        fs::write(&release_lib, b"release library").unwrap();
        fs::write(&debug_lib, b"debug library").unwrap();

        // Create full bundle
        let full_bundle = temp_dir.path().join("full.rbp");
        let libs = vec![
            (
                "linux-x86_64".to_string(),
                "release".to_string(),
                release_lib.to_string_lossy().to_string(),
            ),
            (
                "linux-x86_64".to_string(),
                "debug".to_string(),
                debug_lib.to_string_lossy().to_string(),
            ),
        ];

        create(
            "test-plugin",
            "1.0.0",
            &libs,
            Some(full_bundle.to_string_lossy().to_string()),
            &[],
            None,
            None,
            None,
            None,
            None,
            true,
            &[],
            &[],
        )
        .unwrap();

        // Slim to release only
        let slim_bundle = temp_dir.path().join("slim.rbp");
        slim(
            &full_bundle.to_string_lossy(),
            &slim_bundle.to_string_lossy(),
            None,
            vec!["release".to_string()],
            false,
            None,
        )
        .unwrap();

        // Verify slim bundle only has release variant
        let loader = BundleLoader::open(&slim_bundle).unwrap();
        let variants = loader.list_variants(Platform::LinuxX86_64);
        assert_eq!(variants.len(), 1);
        assert!(variants.contains(&"release"));
    }
}
