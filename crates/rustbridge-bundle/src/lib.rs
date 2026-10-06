//! Plugin bundle format for rustbridge
//!
//! This crate provides types and utilities for creating and loading `.rbp`
//! (rustbridge plugin) bundles - standardized archives containing multi-platform
//! plugin libraries and metadata.
//!
//! # Bundle Structure
//!
//! ```text
//! my-plugin-1.0.0.rbp
//! ├── manifest.json
//! ├── schema/
//! │   ├── messages.json          # JSON Schema for message types
//! │   └── messages.h             # C header with struct definitions
//! ├── lib/
//! │   ├── linux-x86_64/
//! │   │   └── libmyplugin.so
//! │   ├── darwin-aarch64/
//! │   │   └── libmyplugin.dylib
//! │   └── windows-x86_64/
//! │       └── myplugin.dll
//! └── docs/
//!     └── README.md
//! ```
//!
//! # Example
//!
//! ```no_run
//! use rustbridge_bundle::{BundleBuilder, Manifest, Platform};
//!
//! // Create a bundle
//! let manifest = Manifest::new("my-plugin", "1.0.0");
//! let builder = BundleBuilder::new(manifest)
//!     .add_library(Platform::LinuxX86_64, "target/release/libmyplugin.so")?
//!     .add_library(Platform::DarwinAarch64, "target/release/libmyplugin.dylib")?;
//!
//! builder.write("my-plugin-1.0.0.rbp")?;
//! # Ok::<(), rustbridge_bundle::BundleError>(())
//! ```

mod error;
mod manifest;
mod platform;

pub mod builder;
pub mod loader;

pub use builder::BundleBuilder;
pub use error::BundleError;
pub use loader::BundleLoader;
pub use manifest::{
    BuildInfo, ComponentInfo, GitInfo, Manifest, PlatformInfo, PluginInfo, Sbom, SchemaInfo,
    VariantInfo,
};
pub use platform::Platform;

/// Result type for bundle operations.
pub type BundleResult<T> = Result<T, BundleError>;

/// Bundle file extension.
pub const BUNDLE_EXTENSION: &str = "rbp";

/// Current bundle format version.
pub const BUNDLE_VERSION: &str = "1.0";

/// Bundle format version 1.1 (variant-level metadata).
pub const BUNDLE_VERSION_1_1: &str = "1.1";

/// Bundle format version 1.2 (WebAssembly components and manifest extensions).
///
/// A 1.2 manifest may carry the `wasm32-wasip2` platform, a `component`
/// entry on a variant, and top-level extension sections that 1.2 readers keep
/// as they are.
pub const BUNDLE_VERSION_1_2: &str = "1.2";

/// Manifest file name within the bundle.
pub const MANIFEST_FILE: &str = "manifest.json";
