//! Pack command - auto-detect plugin project and create a bundle.
//!
//! Reads name/version from `Cargo.toml`, detects the platform, finds built libraries,
//! and delegates to `bundle::create()`.

use anyhow::{Context, Result};
use rustbridge_bundle::Platform;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Minimal Cargo.toml representation for plugin detection.
#[derive(Deserialize)]
struct CargoToml {
    package: Option<PackageInfo>,
    lib: Option<LibInfo>,
}

#[derive(Deserialize)]
struct PackageInfo {
    name: Option<String>,
    version: Option<toml::Value>,
}

#[derive(Deserialize)]
struct LibInfo {
    name: Option<String>,
    #[serde(rename = "crate-type")]
    crate_type: Option<Vec<String>>,
}

/// Minimal workspace Cargo.toml representation for version resolution.
#[derive(Deserialize)]
struct WorkspaceCargoToml {
    workspace: Option<WorkspaceInfo>,
}

#[derive(Deserialize)]
struct WorkspaceInfo {
    package: Option<WorkspacePackageInfo>,
}

#[derive(Deserialize)]
struct WorkspacePackageInfo {
    version: Option<String>,
}

/// Detected plugin project metadata.
#[derive(Debug)]
pub struct PluginProject {
    /// Package name from Cargo.toml
    pub name: String,
    /// Resolved version string
    pub version: String,
    /// Library base name (underscored, used for filename generation)
    pub lib_name: String,
}

impl PluginProject {
    /// Detect plugin project from a directory containing Cargo.toml.
    pub fn detect(project_dir: &Path) -> Result<Self> {
        let cargo_toml_path = project_dir.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            anyhow::bail!(
                "No Cargo.toml found in {}. Run this command from a plugin project directory.",
                project_dir.display()
            );
        }

        let contents = std::fs::read_to_string(&cargo_toml_path)
            .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;

        let cargo_toml: CargoToml =
            toml::from_str(&contents).context("Failed to parse Cargo.toml")?;

        // Validate crate-type includes cdylib
        let lib_info = cargo_toml.lib.as_ref();
        let has_cdylib = lib_info
            .and_then(|l| l.crate_type.as_ref())
            .is_some_and(|types| types.iter().any(|t| t == "cdylib"));

        if !has_cdylib {
            anyhow::bail!(
                "Expected [lib] crate-type to include \"cdylib\" in {}. \
                 This command only works with plugin projects.",
                cargo_toml_path.display()
            );
        }

        // Extract package name
        let package = cargo_toml
            .package
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing [package] section in Cargo.toml"))?;

        let name = package
            .name
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Missing package.name in Cargo.toml"))?;

        // Resolve version
        let version = resolve_version(package, project_dir)?;

        // Determine library base name: [lib] name or derive from package name
        let lib_name = lib_info
            .and_then(|l| l.name.clone())
            .unwrap_or_else(|| name.replace('-', "_"));

        Ok(Self {
            name,
            version,
            lib_name,
        })
    }
}

/// Resolve version from package info, following workspace inheritance.
fn resolve_version(package: &PackageInfo, project_dir: &Path) -> Result<String> {
    let version_value = package
        .version
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing package.version in Cargo.toml"))?;

    match version_value {
        toml::Value::String(v) => Ok(v.clone()),
        toml::Value::Table(t) => {
            if t.get("workspace").and_then(|v| v.as_bool()) == Some(true) {
                find_workspace_version(project_dir)
            } else {
                anyhow::bail!("Unsupported package.version format in Cargo.toml")
            }
        }
        _ => anyhow::bail!("Unsupported package.version format in Cargo.toml"),
    }
}

/// Walk up parent directories to find the workspace root Cargo.toml and read its version.
fn find_workspace_version(start_dir: &Path) -> Result<String> {
    let mut dir = start_dir.to_path_buf();

    loop {
        // Check parent directory
        if !dir.pop() {
            anyhow::bail!(
                "Could not find workspace root with [workspace.package] version. \
                 Searched from {}",
                start_dir.display()
            );
        }

        let candidate = dir.join("Cargo.toml");
        if !candidate.exists() {
            continue;
        }

        let contents = std::fs::read_to_string(&candidate)
            .with_context(|| format!("Failed to read {}", candidate.display()))?;

        let workspace_toml: WorkspaceCargoToml = match toml::from_str(&contents) {
            Ok(parsed) => parsed,
            Err(_) => continue,
        };

        if let Some(workspace) = workspace_toml.workspace
            && let Some(pkg) = workspace.package
            && let Some(version) = pkg.version
        {
            return Ok(version);
        }
    }
}

/// Return the default signing key path (`~/.rustbridge/signing.key`).
fn default_signing_key_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Could not determine home directory")?;
    Ok(PathBuf::from(home).join(".rustbridge").join("signing.key"))
}

/// Derive the output bundle path.
fn derive_pack_output(project_dir: &Path, name: &str, version: &str, dev: bool) -> PathBuf {
    let suffix = if dev { "-dev" } else { "" };
    project_dir
        .join("target")
        .join("bundle")
        .join(format!("{name}-{version}{suffix}.rbp"))
}

/// Find the cargo target directory by walking up to the workspace root.
///
/// Cargo puts build artifacts in the workspace root's `target/` directory,
/// not in the member crate's directory.
fn find_target_dir(project_dir: &Path) -> PathBuf {
    // Check if there's a workspace root above us
    let mut dir = project_dir.to_path_buf();
    loop {
        if !dir.pop() {
            // No workspace root found, use project_dir
            return project_dir.join("target");
        }

        let candidate = dir.join("Cargo.toml");
        if !candidate.exists() {
            continue;
        }

        // Check if it has a [workspace] section
        if let Ok(contents) = std::fs::read_to_string(&candidate)
            && let Ok(parsed) = toml::from_str::<WorkspaceCargoToml>(&contents)
            && parsed.workspace.is_some()
        {
            return dir.join("target");
        }
    }
}

/// Run the pack command.
#[allow(clippy::too_many_arguments)]
pub fn run_pack(
    dev: bool,
    sign_key: Option<String>,
    no_sign: bool,
    schema_source: Option<String>,
    header_source: Option<String>,
) -> Result<()> {
    // Validate flag combinations
    if sign_key.is_some() && no_sign {
        anyhow::bail!("Conflicting flags: --sign-key and --no-sign cannot be used together");
    }
    if schema_source.is_some() && !dev {
        anyhow::bail!("--schema-source is only available in dev mode (use --dev)");
    }
    if header_source.is_some() && !dev {
        anyhow::bail!("--header-source is only available in dev mode (use --dev)");
    }

    let cwd = std::env::current_dir().context("Failed to get current directory")?;
    let project = PluginProject::detect(&cwd)?;

    let platform = Platform::current().ok_or_else(|| {
        anyhow::anyhow!(
            "Unsupported platform: {}-{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let platform_str = platform.to_string();

    println!("Packing plugin: {} v{}", project.name, project.version);
    println!("  Platform: {platform_str}");
    println!(
        "  Library name: {}",
        platform.library_name(&project.lib_name)
    );

    let target_dir = find_target_dir(&cwd);
    let lib_filename = platform.library_name(&project.lib_name);

    // Build library list
    let mut libraries: Vec<(String, String, String)> = Vec::new();

    // Always include release library
    let release_lib = target_dir.join("release").join(&lib_filename);
    if !release_lib.exists() {
        anyhow::bail!(
            "Release library not found: {}\n\
             Run: cargo build --release",
            release_lib.display()
        );
    }
    libraries.push((
        platform_str.clone(),
        "release".to_string(),
        release_lib.to_string_lossy().to_string(),
    ));
    println!("  Release library: {}", release_lib.display());

    // Include debug library if dev mode
    if dev {
        let debug_lib = target_dir.join("debug").join(&lib_filename);
        if !debug_lib.exists() {
            anyhow::bail!(
                "Debug library not found: {}\n\
                 Run: cargo build",
                debug_lib.display()
            );
        }
        libraries.push((
            platform_str,
            "debug".to_string(),
            debug_lib.to_string_lossy().to_string(),
        ));
        println!("  Debug library: {}", debug_lib.display());
    }

    // Find SBOM files
    let mut sbom_files: Vec<(String, String)> = Vec::new();
    for sbom_name in &["sbom.cdx.json", "sbom.spdx.json"] {
        let sbom_path = cwd.join(sbom_name);
        if sbom_path.exists() {
            println!("  SBOM: {sbom_name}");
            sbom_files.push((
                sbom_path.to_string_lossy().to_string(),
                (*sbom_name).to_string(),
            ));
        }
    }

    // Find LICENSE file
    let license_path = find_license_file(&cwd);
    if let Some(ref lp) = license_path {
        println!("  License: {}", lp.display());
    }

    // Resolve signing key for release mode
    let resolved_sign_key = if dev || no_sign {
        None
    } else {
        match sign_key {
            Some(path) => Some(path),
            None => {
                let default_key = default_signing_key_path()?;
                if default_key.exists() {
                    println!("  Signing with: {}", default_key.display());
                    Some(default_key.to_string_lossy().to_string())
                } else {
                    eprintln!(
                        "Warning: No signing key found at {}. Bundle will not be signed. \
                         Use 'rustbridge keygen' to generate a key.",
                        default_key.display()
                    );
                    None
                }
            }
        }
    };

    // Create output directory
    let output = derive_pack_output(&cwd, &project.name, &project.version, dev);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    println!("  Output: {}", output.display());

    // Delegate to bundle::create
    crate::bundle::create(
        &project.name,
        &project.version,
        &libraries,
        Some(output.to_string_lossy().to_string()),
        &[], // schema_files (handled via generate flags)
        resolved_sign_key,
        header_source, // generate_header
        schema_source, // generate_schema
        None,          // notices
        license_path.map(|p| p.to_string_lossy().to_string()),
        false, // no_metadata
        &sbom_files,
        &[], // custom_metadata
    )?;

    Ok(())
}

/// Find a LICENSE file in the project directory.
fn find_license_file(project_dir: &Path) -> Option<PathBuf> {
    for name in &[
        "LICENSE",
        "LICENSE.md",
        "LICENSE.txt",
        "LICENSE-MIT",
        "LICENSE-APACHE",
    ] {
        let path = project_dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_cargo_toml(dir: &Path, content: &str) {
        fs::write(dir.join("Cargo.toml"), content).unwrap();
    }

    #[test]
    fn detect___standalone_project___extracts_name_and_version() {
        let temp = TempDir::new().unwrap();
        write_cargo_toml(
            temp.path(),
            r#"
[package]
name = "my-plugin"
version = "2.1.0"

[lib]
crate-type = ["cdylib"]
"#,
        );

        let project = PluginProject::detect(temp.path()).unwrap();

        assert_eq!(project.name, "my-plugin");
        assert_eq!(project.version, "2.1.0");
    }

    #[test]
    fn detect___custom_lib_name___uses_lib_name() {
        let temp = TempDir::new().unwrap();
        write_cargo_toml(
            temp.path(),
            r#"
[package]
name = "my-plugin"
version = "1.0.0"

[lib]
name = "custom_name"
crate-type = ["cdylib"]
"#,
        );

        let project = PluginProject::detect(temp.path()).unwrap();

        assert_eq!(project.lib_name, "custom_name");
    }

    #[test]
    fn detect___workspace_version___resolves_from_workspace_root() {
        let temp = TempDir::new().unwrap();

        // Create workspace root
        write_cargo_toml(
            temp.path(),
            r#"
[workspace]
members = ["crates/my-plugin"]

[workspace.package]
version = "3.0.0"
"#,
        );

        // Create member directory
        let member_dir = temp.path().join("crates").join("my-plugin");
        fs::create_dir_all(&member_dir).unwrap();
        write_cargo_toml(
            &member_dir,
            r#"
[package]
name = "my-plugin"
version.workspace = true

[lib]
crate-type = ["cdylib"]
"#,
        );

        let project = PluginProject::detect(&member_dir).unwrap();

        assert_eq!(project.version, "3.0.0");
    }

    #[test]
    fn detect___no_cargo_toml___returns_error() {
        let temp = TempDir::new().unwrap();

        let result = PluginProject::detect(temp.path());

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("No Cargo.toml"), "Error was: {err}");
    }

    #[test]
    fn detect___no_cdylib_crate_type___returns_error() {
        let temp = TempDir::new().unwrap();
        write_cargo_toml(
            temp.path(),
            r#"
[package]
name = "my-lib"
version = "1.0.0"

[lib]
crate-type = ["rlib"]
"#,
        );

        let result = PluginProject::detect(temp.path());

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("cdylib"), "Error was: {err}");
    }

    #[test]
    fn detect___hyphenated_name___converts_to_underscores() {
        let temp = TempDir::new().unwrap();
        write_cargo_toml(
            temp.path(),
            r#"
[package]
name = "my-cool-plugin"
version = "1.0.0"

[lib]
crate-type = ["cdylib"]
"#,
        );

        let project = PluginProject::detect(temp.path()).unwrap();

        assert_eq!(project.lib_name, "my_cool_plugin");
    }

    #[test]
    fn output_path___release_mode___no_dev_suffix() {
        let dir = Path::new("/project");

        let path = derive_pack_output(dir, "my-plugin", "1.0.0", false);

        assert_eq!(
            path,
            PathBuf::from("/project/target/bundle/my-plugin-1.0.0.rbp")
        );
    }

    #[test]
    fn output_path___dev_mode___has_dev_suffix() {
        let dir = Path::new("/project");

        let path = derive_pack_output(dir, "my-plugin", "1.0.0", true);

        assert_eq!(
            path,
            PathBuf::from("/project/target/bundle/my-plugin-1.0.0-dev.rbp")
        );
    }
}
