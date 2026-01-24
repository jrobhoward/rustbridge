//! Generate minisign key pairs for signing bundles.

use anyhow::{Context, Result};
use minisign::KeyPair;
use std::path::PathBuf;

/// Generate a new minisign key pair.
///
/// Creates a secret key and public key pair for signing plugin bundles.
/// The secret key is encrypted with a password provided by the user.
///
/// # Arguments
/// * `output` - Optional output path for the secret key. If None, uses ~/.rustbridge/signing.key
/// * `force` - If true, overwrites existing keys without prompting
pub fn run(output: Option<String>, force: bool) -> Result<()> {
    // Determine output paths
    let secret_key_path = match output {
        Some(path) => PathBuf::from(path),
        None => {
            let home = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .context("Could not determine home directory")?;
            let rustbridge_dir = PathBuf::from(home).join(".rustbridge");
            std::fs::create_dir_all(&rustbridge_dir)
                .context("Failed to create ~/.rustbridge directory")?;
            rustbridge_dir.join("signing.key")
        }
    };

    let public_key_path = secret_key_path.with_extension("pub");

    // Check if keys already exist
    if secret_key_path.exists() && !force {
        anyhow::bail!(
            "Secret key already exists at {}. Use --force to overwrite.",
            secret_key_path.display()
        );
    }

    println!("Generating new minisign key pair...");
    println!();

    // Get password from user
    println!("Enter password to encrypt secret key: ");
    let password = rpassword::read_password().context("Failed to read password")?;

    if password.is_empty() {
        anyhow::bail!("Password cannot be empty");
    }

    println!("Confirm password: ");
    let password_confirm =
        rpassword::read_password().context("Failed to read password confirmation")?;

    if password != password_confirm {
        anyhow::bail!("Passwords do not match");
    }

    // Generate key pair
    let KeyPair { pk, sk } = KeyPair::generate_encrypted_keypair(Some(password.clone()))
        .context("Failed to generate key pair")?;

    // Save secret key
    let secret_key_box = sk.to_box(None).context("Failed to encode secret key")?;
    std::fs::write(&secret_key_path, secret_key_box.to_string())
        .context("Failed to write secret key")?;

    // Save public key
    let public_key_box = pk.to_box().context("Failed to encode public key")?;
    std::fs::write(&public_key_path, public_key_box.to_string())
        .context("Failed to write public key")?;

    println!();
    println!("✓ Key pair generated successfully!");
    println!();
    println!("  Secret key: {}", secret_key_path.display());
    println!("  Public key: {}", public_key_path.display());
    println!();
    println!("Public key (for distribution):");
    println!("{}", pk.to_base64());
    println!();
    println!("Keep your secret key safe and never commit it to version control!");

    Ok(())
}
