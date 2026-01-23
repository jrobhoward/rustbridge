//! Build command implementation

use anyhow::{Context, Result};
use std::process::Command;

/// Run the build command
pub fn run(path: Option<String>, release: bool, target: Option<String>) -> Result<()> {
    let project_path = path.unwrap_or_else(|| ".".to_string());

    println!("Building rustbridge plugin in: {}", project_path);

    // Build cargo command
    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if release {
        cmd.arg("--release");
        println!("Mode: release");
    } else {
        println!("Mode: debug");
    }

    if let Some(t) = &target {
        cmd.arg("--target").arg(t);
        println!("Target: {}", t);
    }

    cmd.current_dir(&project_path);

    // Execute
    let status = cmd.status().context("Failed to execute cargo build")?;

    if status.success() {
        println!("\n✓ Build successful!");

        // Show output location
        let output_dir = if release { "release" } else { "debug" };
        let target_prefix = target.map(|t| format!("{}/", t)).unwrap_or_default();
        println!(
            "Output: {}/target/{}{}",
            project_path, target_prefix, output_dir
        );
    } else {
        anyhow::bail!("Build failed with exit code: {:?}", status.code());
    }

    Ok(())
}
