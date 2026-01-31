//! rustbridge CLI - Build tool and code generator
//!
//! Commands:
//! - `rustbridge new` - Create a new plugin project
//! - `rustbridge generate-header` - Generate C headers from Rust structs
//! - `rustbridge keygen` - Generate signing keys for bundles
//! - `rustbridge bundle` - Create, inspect, or extract plugin bundles

use clap::{Parser, Subcommand};

mod bundle;
mod header_gen;
mod keygen;
mod new;

#[derive(Parser)]
#[command(name = "rustbridge")]
#[command(author, version, about = "Build tool for rustbridge plugins", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)] // BundleAction has many options; boxing would complicate clap usage
enum Commands {
    /// Create a new rustbridge plugin project
    New {
        /// Project name
        name: String,

        /// Project directory (default: ./<name>)
        #[arg(short, long)]
        path: Option<String>,

        /// Also generate Kotlin consumer project (requires Java 21+)
        #[arg(long)]
        kotlin: bool,

        /// Also generate Java FFM consumer project (requires Java 21+)
        #[arg(long)]
        java_ffm: bool,

        /// Also generate Java JNI consumer project (requires Java 17+)
        #[arg(long)]
        java_jni: bool,

        /// Also generate C# consumer project (requires .NET 8+)
        #[arg(long)]
        csharp: bool,

        /// Also generate Python consumer project
        #[arg(long)]
        python: bool,

        /// Generate all consumer projects
        #[arg(long)]
        all: bool,
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

        /// Library to include (can be repeated)
        /// Format: PLATFORM:PATH or PLATFORM:VARIANT:PATH
        /// Examples:
        ///   --lib linux-x86_64:target/release/libplugin.so (release variant)
        ///   --lib linux-x86_64:debug:target/debug/libplugin.so (debug variant)
        #[arg(short, long = "lib", value_name = "PLATFORM[:VARIANT]:PATH")]
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

        /// Auto-generate JSON Schema from Rust source file and embed in bundle
        /// Example: --generate-schema src/messages.rs:schema.json
        #[arg(long, value_name = "SOURCE:SCHEMA_NAME")]
        generate_schema: Option<String>,

        /// License notices file to include in the bundle
        #[arg(long, value_name = "PATH")]
        notices: Option<String>,

        /// Plugin's own license file to include in the bundle
        /// Example: --license LICENSE
        #[arg(long, value_name = "PATH")]
        license: Option<String>,

        /// Skip automatic build metadata collection
        #[arg(long)]
        no_metadata: bool,

        /// SBOM file to include: SOURCE:ARCHIVE_NAME (can be repeated)
        /// Example: --sbom sbom.cdx.json:sbom.cdx.json --sbom sbom.spdx.json:sbom.spdx.json
        #[arg(long, value_name = "SOURCE:ARCHIVE_NAME")]
        sbom: Vec<String>,

        /// Custom metadata as KEY=VALUE (can be repeated)
        /// Adds arbitrary key/value pairs to build_info.custom for informational purposes.
        /// Example: --metadata repository=https://github.com/user/project
        /// Example: --metadata ci_job_id=12345
        #[arg(long, value_name = "KEY=VALUE")]
        metadata: Vec<String>,
    },

    /// Combine multiple bundles into one
    Combine {
        /// Input bundle files (at least 2)
        #[arg(required = true, num_args = 2..)]
        bundles: Vec<String>,

        /// Output bundle path
        #[arg(short, long)]
        output: String,

        /// Path to signing key for re-signing the combined bundle
        #[arg(long, value_name = "KEY_PATH")]
        sign_key: Option<String>,

        /// Schema mismatch handling: error (default), warn, ignore
        #[arg(long, value_name = "MODE", default_value = "error")]
        schema_mismatch: String,
    },

    /// Create a slimmed bundle with subset of platforms/variants
    Slim {
        /// Input bundle path
        #[arg(short, long)]
        input: String,

        /// Output bundle path
        #[arg(short, long)]
        output: String,

        /// Platforms to keep (comma-separated)
        /// Example: --platforms linux-x86_64,darwin-aarch64
        #[arg(long, value_name = "PLATFORMS")]
        platforms: Option<String>,

        /// Variants to keep (comma-separated, default: release)
        /// Example: --variants release,debug
        #[arg(long, value_name = "VARIANTS", default_value = "release")]
        variants: String,

        /// Exclude documentation files
        #[arg(long)]
        exclude_docs: bool,

        /// Path to signing key for re-signing the slimmed bundle
        #[arg(long, value_name = "KEY_PATH")]
        sign_key: Option<String>,
    },

    /// List contents of a bundle
    List {
        /// Path to the bundle file
        bundle: String,

        /// Show build info
        #[arg(long)]
        show_build: bool,

        /// Show all variants
        #[arg(long)]
        show_variants: bool,
    },

    /// Extract library from a bundle
    Extract {
        /// Path to the bundle file
        bundle: String,

        /// Target platform (default: current platform)
        #[arg(short, long)]
        platform: Option<String>,

        /// Variant to extract (default: release)
        #[arg(long, default_value = "release")]
        variant: String,

        /// Output directory for extracted library
        #[arg(short, long, default_value = ".")]
        output: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::New {
            name,
            path,
            kotlin,
            java_ffm,
            java_jni,
            csharp,
            python,
            all,
        } => {
            let options = new::NewOptions {
                kotlin: kotlin || all,
                java_ffm: java_ffm || all,
                java_jni: java_jni || all,
                csharp: csharp || all,
                python: python || all,
            };
            new::run(&name, path, options)?;
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
                generate_schema,
                notices,
                license,
                no_metadata,
                sbom,
                metadata,
            } => {
                // Parse library arguments (PLATFORM:PATH or PLATFORM:VARIANT:PATH)
                let libs: Vec<(String, String, String)> = libraries
                    .iter()
                    .map(|s| {
                        let parts: Vec<&str> = s.splitn(3, ':').collect();
                        match parts.len() {
                            2 => {
                                // PLATFORM:PATH -> (platform, "release", path)
                                Ok((
                                    parts[0].to_string(),
                                    "release".to_string(),
                                    parts[1].to_string(),
                                ))
                            }
                            3 => {
                                // PLATFORM:VARIANT:PATH -> (platform, variant, path)
                                Ok((
                                    parts[0].to_string(),
                                    parts[1].to_string(),
                                    parts[2].to_string(),
                                ))
                            }
                            _ => anyhow::bail!(
                                "Invalid library format: {s}. Expected PLATFORM:PATH or PLATFORM:VARIANT:PATH"
                            ),
                        }
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

                // Parse SBOM arguments (SOURCE:ARCHIVE_NAME)
                let sbom_files: Vec<(String, String)> = sbom
                    .iter()
                    .map(|s| {
                        let parts: Vec<&str> = s.splitn(2, ':').collect();
                        if parts.len() != 2 {
                            anyhow::bail!("Invalid sbom format: {s}. Expected SOURCE:ARCHIVE_NAME");
                        }
                        Ok((parts[0].to_string(), parts[1].to_string()))
                    })
                    .collect::<anyhow::Result<_>>()?;

                // Parse custom metadata arguments (KEY=VALUE)
                let custom_metadata: Vec<(String, String)> = metadata
                    .iter()
                    .map(|s| {
                        let parts: Vec<&str> = s.splitn(2, '=').collect();
                        if parts.len() != 2 {
                            anyhow::bail!("Invalid metadata format: {s}. Expected KEY=VALUE");
                        }
                        Ok((parts[0].to_string(), parts[1].to_string()))
                    })
                    .collect::<anyhow::Result<_>>()?;

                bundle::create(
                    &name,
                    &version,
                    &libs,
                    output,
                    &schemas,
                    sign_key,
                    generate_header,
                    generate_schema,
                    notices,
                    license,
                    no_metadata,
                    &sbom_files,
                    &custom_metadata,
                )?;
            }
            BundleAction::Combine {
                bundles,
                output,
                sign_key,
                schema_mismatch,
            } => {
                bundle::combine(&bundles, &output, sign_key, &schema_mismatch)?;
            }
            BundleAction::Slim {
                input,
                output,
                platforms,
                variants,
                exclude_docs,
                sign_key,
            } => {
                let platform_list: Option<Vec<String>> =
                    platforms.map(|p| p.split(',').map(|s| s.trim().to_string()).collect());
                let variant_list: Vec<String> =
                    variants.split(',').map(|s| s.trim().to_string()).collect();

                bundle::slim(
                    &input,
                    &output,
                    platform_list,
                    variant_list,
                    exclude_docs,
                    sign_key,
                )?;
            }
            BundleAction::List {
                bundle: bundle_path,
                show_build,
                show_variants,
            } => {
                bundle::list(&bundle_path, show_build, show_variants)?;
            }
            BundleAction::Extract {
                bundle: bundle_path,
                platform,
                variant,
                output: output_dir,
            } => {
                bundle::extract(&bundle_path, platform, &variant, &output_dir)?;
            }
        },
    }

    Ok(())
}
