//! rustbridge CLI - Build tool and code generator
//!
//! Commands:
//! - `rustbridge build` - Build a plugin
//! - `rustbridge generate` - Generate host language bindings
//! - `rustbridge generate-header` - Generate C headers from Rust structs
//! - `rustbridge new` - Create a new plugin project
//! - `rustbridge check` - Validate a rustbridge.toml manifest

use clap::{Parser, Subcommand};

mod build;
mod generate;
mod header_gen;
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
    }

    Ok(())
}
