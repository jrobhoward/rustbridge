//! rustbridge CLI - Build tool and code generator
//!
//! Commands:
//! - `rustbridge build` - Build a plugin
//! - `rustbridge generate` - Generate host language bindings
//! - `rustbridge generate-header` - Generate C headers from Rust structs
//! - `rustbridge new` - Create a new plugin project
//! - `rustbridge check` - Validate a rustbridge.toml manifest
//! - `rustbridge bundle` - Create, inspect, or extract plugin bundles

use clap::{Parser, Subcommand};

mod build;
mod bundle;
mod generate;
mod header_gen;
mod keygen;
mod manifest;
mod new;

#[derive(Parser)]
#[command(name = "rustbridge")]
#[command(author, version, about = "Build tool for rustbridge plugins", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a rustbridge plugin
    Build {
        /// Path to the plugin project (default: current directory)
        #[arg(short, long)]
        path: Option<String>,

        /// Build in release mode
        #[arg(short, long)]
        release: bool,

        /// Target platform (e.g., x86_64-unknown-linux-gnu)
        #[arg(short, long)]
        target: Option<String>,
    },

    /// Generate host language bindings
    Generate {
        /// Target language (java, csharp, python)
        #[arg(short, long)]
        lang: String,

        /// Output directory for generated code
        #[arg(short, long)]
        output: String,

        /// Path to rustbridge.toml manifest
        #[arg(short, long)]
        manifest: Option<String>,
    },

    /// Create a new rustbridge plugin project
    New {
        /// Project name
        name: String,

        /// Project directory (default: ./<name>)
        #[arg(short, long)]
        path: Option<String>,
    },

    /// Validate a rustbridge.toml manifest
    Check {
        /// Path to rustbridge.toml (default: ./rustbridge.toml)
        #[arg(short, long)]
        manifest: Option<String>,
    },

    /// Generate C header from Rust #[repr(C)] structs
    GenerateHeader {
        /// Path to Rust source file containing #[repr(C)] structs
        #[arg(short, long)]
        source: String,

        /// Output path for generated C header (default: messages.h)
        #[arg(short, long, default_value = "messages.h")]
        output: String,

        /// Verify the generated header compiles with a C compiler
        #[arg(short, long)]
        verify: bool,
    },

    /// Generate a new minisign key pair for signing bundles
    Keygen {
        /// Output path for secret key (default: ~/.rustbridge/signing.key)
        #[arg(short, long)]
        output: Option<String>,

        /// Force overwrite if key already exists
        #[arg(short, long)]
        force: bool,
    },

    /// Create, inspect, or extract plugin bundles
    Bundle {
        #[command(subcommand)]
        action: BundleAction,
    },
}

#[derive(Subcommand)]
enum BundleAction {
    /// Create a new bundle from libraries
    Create {
        /// Plugin name
        #[arg(short, long)]
        name: String,

        /// Plugin version (semver)
        #[arg(short, long)]
        version: String,

        /// Library to include: PLATFORM:PATH (can be repeated)
        /// Example: --lib linux-x86_64:target/release/libmyplugin.so
        #[arg(short, long = "lib", value_name = "PLATFORM:PATH")]
        libraries: Vec<String>,

        /// Output bundle path (default: <name>-<version>.rbp)
        #[arg(short, long)]
        output: Option<String>,

        /// Schema file to include: SOURCE:ARCHIVE_NAME (can be repeated)
        /// Example: --schema messages.h:messages.h
        #[arg(short, long, value_name = "SOURCE:ARCHIVE_NAME")]
        schema: Vec<String>,

        /// Path to signing key for code signing (optional)
        /// Example: --sign-key ~/.rustbridge/signing.key
        #[arg(long, value_name = "KEY_PATH")]
        sign_key: Option<String>,

        /// Auto-generate C header from Rust source file and embed in bundle
        /// Example: --generate-header src/binary_messages.rs:messages.h
        #[arg(long, value_name = "SOURCE:HEADER_NAME")]
        generate_header: Option<String>,
    },

    /// List contents of a bundle
    List {
        /// Path to the bundle file
        bundle: String,
    },

    /// Extract library from a bundle
    Extract {
        /// Path to the bundle file
        bundle: String,

        /// Target platform (default: current platform)
        #[arg(short, long)]
        platform: Option<String>,

        /// Output directory for extracted library
        #[arg(short, long, default_value = ".")]
        output: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            path,
            release,
            target,
        } => {
            build::run(path, release, target)?;
        }
        Commands::Generate {
            lang,
            output,
            manifest,
        } => {
            generate::run(&lang, &output, manifest)?;
        }
        Commands::New { name, path } => {
            new::run(&name, path)?;
        }
        Commands::Check { manifest } => {
            manifest::check(manifest)?;
        }
        Commands::GenerateHeader {
            source,
            output,
            verify,
        } => {
            header_gen::run(&source, &output, verify)?;
        }
        Commands::Keygen { output, force } => {
            keygen::run(output, force)?;
        }
        Commands::Bundle { action } => match action {
            BundleAction::Create {
                name,
                version,
                libraries,
                output,
                schema,
                sign_key,
                generate_header,
            } => {
                // Parse library arguments (PLATFORM:PATH)
                let libs: Vec<(String, String)> = libraries
                    .iter()
                    .map(|s| {
                        let parts: Vec<&str> = s.splitn(2, ':').collect();
                        if parts.len() != 2 {
                            anyhow::bail!("Invalid library format: {s}. Expected PLATFORM:PATH");
                        }
                        Ok((parts[0].to_string(), parts[1].to_string()))
                    })
                    .collect::<anyhow::Result<_>>()?;

                // Parse schema arguments (SOURCE:ARCHIVE_NAME)
                let schemas: Vec<(String, String)> = schema
                    .iter()
                    .map(|s| {
                        let parts: Vec<&str> = s.splitn(2, ':').collect();
                        if parts.len() != 2 {
                            anyhow::bail!(
                                "Invalid schema format: {s}. Expected SOURCE:ARCHIVE_NAME"
                            );
                        }
                        Ok((parts[0].to_string(), parts[1].to_string()))
                    })
                    .collect::<anyhow::Result<_>>()?;

                bundle::run(
                    &name,
                    &version,
                    &libs,
                    output,
                    &schemas,
                    sign_key,
                    generate_header,
                )?;
            }
            BundleAction::List {
                bundle: bundle_path,
            } => {
                bundle::list(&bundle_path)?;
            }
            BundleAction::Extract {
                bundle: bundle_path,
                platform,
                output: output_dir,
            } => {
                bundle::extract(&bundle_path, platform, &output_dir)?;
            }
        },
    }

    Ok(())
}
