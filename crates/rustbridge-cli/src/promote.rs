//! Promote command - slim a dev bundle to a signed release bundle.
//!
//! Takes a dev bundle (containing both release and debug variants) and creates
//! a release-only bundle, optionally signing it.

use anyhow::{Context, Result};
use rustbridge_bundle::Platform;
use std::path::{Path, PathBuf};

/// Derive the output path for a promoted bundle.
///
/// If the input filename contains `-dev`, strips that suffix.
/// Otherwise appends `-release` before the extension.
fn derive_promote_output(input_path: &Path) -> PathBuf {
    let stem = input_path.file_stem().unwrap_or_default().to_string_lossy();
    let parent = input_path.parent().unwrap_or_else(|| Path::new("."));

    if let Some(stripped) = stem.strip_suffix("-dev") {
        parent.join(format!("{stripped}.rbp"))
    } else {
        parent.join(format!("{stem}-release.rbp"))
    }
}

/// Return the default signing key path (`~/.rustbridge/signing.key`).
fn default_signing_key_path() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Could not determine home directory")?;
    Ok(PathBuf::from(home).join(".rustbridge").join("signing.key"))
}

/// Run the promote command.
pub fn run_promote(
    input: String,
    output: Option<String>,
    sign_key: Option<String>,
    no_sign: bool,
) -> Result<()> {
    if sign_key.is_some() && no_sign {
        anyhow::bail!("Conflicting flags: --sign-key and --no-sign cannot be used together");
    }

    let input_path = Path::new(&input);
    if !input_path.exists() {
        anyhow::bail!("Input bundle not found: {input}");
    }

    let output_path = match output {
        Some(p) => PathBuf::from(p),
        None => derive_promote_output(input_path),
    };

    println!("Promoting bundle: {input}");
    println!("  Output: {}", output_path.display());

    // Detect current platform
    let platform = Platform::current().ok_or_else(|| {
        anyhow::anyhow!(
            "Unsupported platform: {}-{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )
    })?;
    let platform_str = platform.to_string();
    println!("  Platform: {platform_str}");

    // Resolve signing key
    let resolved_sign_key = if no_sign {
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

    // Delegate to bundle::slim with release variant only, current platform
    crate::bundle::slim(
        &input,
        &output_path.to_string_lossy(),
        Some(vec![platform_str]),
        vec!["release".to_string()],
        false, // keep docs
        resolved_sign_key,
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn derive_output___dev_bundle___strips_dev_suffix() {
        let input = Path::new("my-plugin-1.0.0-dev.rbp");

        let output = derive_promote_output(input);

        assert_eq!(output, PathBuf::from("my-plugin-1.0.0.rbp"));
    }

    #[test]
    fn derive_output___no_dev_suffix___appends_release() {
        let input = Path::new("my-plugin-1.0.0.rbp");

        let output = derive_promote_output(input);

        assert_eq!(output, PathBuf::from("my-plugin-1.0.0-release.rbp"));
    }

    #[test]
    fn derive_output___preserves_directory_path() {
        let input = Path::new("/some/dir/my-plugin-1.0.0-dev.rbp");

        let output = derive_promote_output(input);

        assert_eq!(output, PathBuf::from("/some/dir/my-plugin-1.0.0.rbp"));
    }
}
