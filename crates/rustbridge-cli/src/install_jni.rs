//! Install JNI bridge library to well-known location.
//!
//! The JNI bridge library is installed to `~/.rustbridge/lib/<version>/<platform>/`
//! where it can be found by `rustbridge bundle create --include-jni-bridge`.

use anyhow::{Context, Result};
use rustbridge_bundle::Platform;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Get the rustbridge library directory for the current version.
///
/// Returns `~/.rustbridge/lib/<version>/`
pub fn get_lib_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let version = env!("CARGO_PKG_VERSION");
    Ok(home.join(".rustbridge").join("lib").join(version))
}

/// Get the path where the JNI bridge library should be installed for a platform.
///
/// Returns `~/.rustbridge/lib/<version>/<platform>/<library_name>`
pub fn get_jni_bridge_path(platform: Platform) -> Result<PathBuf> {
    let lib_dir = get_lib_dir()?;
    let lib_name = platform.library_name("rustbridge_jni");
    Ok(lib_dir.join(platform.as_str()).join(lib_name))
}

/// Check if the JNI bridge is installed for the current platform and version.
#[allow(dead_code)] // Utility function for future use
pub fn is_installed() -> bool {
    if let Some(platform) = Platform::current() {
        if let Ok(path) = get_jni_bridge_path(platform) {
            return path.exists();
        }
    }
    false
}

/// Install the JNI bridge library.
///
/// If `from_path` is provided, copies from that path.
/// Otherwise, attempts to build from the rustbridge workspace.
pub fn run(from_path: Option<String>) -> Result<()> {
    let platform = Platform::current().context("Unsupported platform")?;
    let version = env!("CARGO_PKG_VERSION");

    println!(
        "Installing JNI bridge for {} (rustbridge v{})",
        platform, version
    );

    let source_path = if let Some(path) = from_path {
        // Use the provided path
        let path = PathBuf::from(&path);
        if !path.exists() {
            anyhow::bail!("File not found: {}", path.display());
        }
        println!("  Source: {}", path.display());
        path
    } else {
        // Try to build from workspace
        build_jni_bridge(&platform)?
    };

    // Create destination directory
    let dest_path = get_jni_bridge_path(platform)?;
    let dest_dir = dest_path.parent().unwrap();
    fs::create_dir_all(dest_dir)
        .with_context(|| format!("Failed to create directory: {}", dest_dir.display()))?;

    // Copy the library
    fs::copy(&source_path, &dest_path).with_context(|| {
        format!(
            "Failed to copy {} to {}",
            source_path.display(),
            dest_path.display()
        )
    })?;

    println!("  Installed: {}", dest_path.display());
    println!("\nJNI bridge installed successfully!");
    println!("You can now use: rustbridge bundle create --include-jni-bridge");

    Ok(())
}

/// Build the JNI bridge library from the rustbridge workspace.
fn build_jni_bridge(platform: &Platform) -> Result<PathBuf> {
    // Check if we're in a rustbridge workspace
    let workspace_root = find_workspace_root()?;

    println!("  Building from workspace: {}", workspace_root.display());
    println!("  Running: cargo build --release -p rustbridge-jni");

    // Build the JNI bridge
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "rustbridge-jni"])
        .current_dir(&workspace_root)
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("cargo build failed with exit code: {:?}", status.code());
    }

    // Find the built library
    let lib_name = platform.library_name("rustbridge_jni");
    let lib_path = workspace_root
        .join("target")
        .join("release")
        .join(&lib_name);

    if !lib_path.exists() {
        anyhow::bail!(
            "Build succeeded but library not found at: {}\n\
             This may indicate a cross-compilation issue.",
            lib_path.display()
        );
    }

    println!("  Built: {}", lib_path.display());
    Ok(lib_path)
}

/// Find the rustbridge workspace root.
///
/// Looks for Cargo.toml with `[workspace]` containing `rustbridge-jni`.
fn find_workspace_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir().context("Could not get current directory")?;

    // Walk up the directory tree looking for the workspace root
    let mut dir = current_dir.as_path();
    loop {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if is_rustbridge_workspace(&cargo_toml)? {
                return Ok(dir.to_path_buf());
            }
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => break,
        }
    }

    anyhow::bail!(
        "Not in a rustbridge workspace.\n\n\
         To install the JNI bridge, either:\n\
         1. Run this command from the rustbridge repository directory:\n\
            cd /path/to/rustbridge && rustbridge install-jni-bridge\n\n\
         2. Specify a pre-built library path:\n\
            rustbridge install-jni-bridge --from /path/to/librustbridge_jni.so"
    )
}

/// Check if a Cargo.toml is the rustbridge workspace root.
fn is_rustbridge_workspace(cargo_toml: &Path) -> Result<bool> {
    let content = fs::read_to_string(cargo_toml)
        .with_context(|| format!("Failed to read {}", cargo_toml.display()))?;

    // Simple check: look for workspace members containing rustbridge-jni
    Ok(content.contains("[workspace]") && content.contains("rustbridge-jni"))
}

/// Show the installation status.
pub fn show_status() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let lib_dir = get_lib_dir()?;

    println!("JNI Bridge Installation Status (rustbridge v{version})");
    println!("Library directory: {}", lib_dir.display());
    println!();

    let platforms = Platform::all();
    let mut any_installed = false;

    for platform in platforms {
        let path = get_jni_bridge_path(*platform)?;
        let status = if path.exists() {
            any_installed = true;
            "installed"
        } else {
            "not installed"
        };
        println!("  {}: {}", platform, status);
    }

    if !any_installed {
        println!();
        println!("No JNI bridges installed for this version.");
        println!("Run: rustbridge install-jni-bridge");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn get_lib_dir___returns_versioned_path() {
        let lib_dir = get_lib_dir().unwrap();
        let version = env!("CARGO_PKG_VERSION");

        assert!(lib_dir.to_string_lossy().contains(".rustbridge"));
        assert!(lib_dir.to_string_lossy().contains("lib"));
        assert!(lib_dir.to_string_lossy().contains(version));
    }

    #[test]
    fn get_jni_bridge_path___includes_platform_and_library_name() {
        let path = get_jni_bridge_path(Platform::LinuxX86_64).unwrap();

        assert!(path.to_string_lossy().contains("linux-x86_64"));
        assert!(path.to_string_lossy().contains("librustbridge_jni.so"));
    }
}
