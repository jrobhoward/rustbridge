//! Platform detection and identification.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported platform targets.
///
/// Native targets carry a shared library for one OS and architecture.
/// `wasm32-wasip2` carries a WebAssembly component (bundle format 1.2) that
/// is not loaded through the C ABI; see [`Platform::is_native`].
///
/// The enum is `#[non_exhaustive]`: new targets (for example a later WASI
/// version) are added without a breaking change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Platform {
    /// Linux on x86_64.
    LinuxX86_64,
    /// Linux on ARM64.
    LinuxAarch64,
    /// macOS on x86_64 (Intel).
    DarwinX86_64,
    /// macOS on ARM64 (Apple Silicon).
    DarwinAarch64,
    /// Windows on x86_64.
    WindowsX86_64,
    /// Windows on ARM64.
    WindowsAarch64,
    /// WebAssembly component for WASI 0.2 (bundle format 1.2).
    #[serde(rename = "wasm32-wasip2")]
    Wasm32Wasip2,
}

impl Platform {
    /// Detect the current platform at runtime.
    #[must_use]
    pub fn current() -> Option<Self> {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;

        match (os, arch) {
            ("linux", "x86_64") => Some(Self::LinuxX86_64),
            ("linux", "aarch64") => Some(Self::LinuxAarch64),
            ("macos", "x86_64") => Some(Self::DarwinX86_64),
            ("macos", "aarch64") => Some(Self::DarwinAarch64),
            ("windows", "x86_64") => Some(Self::WindowsX86_64),
            ("windows", "aarch64") => Some(Self::WindowsAarch64),
            _ => None,
        }
    }

    /// Get the platform key string (e.g., "linux-x86_64").
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "linux-x86_64",
            Self::LinuxAarch64 => "linux-aarch64",
            Self::DarwinX86_64 => "darwin-x86_64",
            Self::DarwinAarch64 => "darwin-aarch64",
            Self::WindowsX86_64 => "windows-x86_64",
            Self::WindowsAarch64 => "windows-aarch64",
            Self::Wasm32Wasip2 => "wasm32-wasip2",
        }
    }

    /// Parse a platform from its string representation.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "linux-x86_64" => Some(Self::LinuxX86_64),
            "linux-aarch64" => Some(Self::LinuxAarch64),
            "darwin-x86_64" => Some(Self::DarwinX86_64),
            "darwin-aarch64" => Some(Self::DarwinAarch64),
            "windows-x86_64" => Some(Self::WindowsX86_64),
            "windows-aarch64" => Some(Self::WindowsAarch64),
            "wasm32-wasip2" => Some(Self::Wasm32Wasip2),
            _ => None,
        }
    }

    /// Get the expected library file extension for this platform.
    #[must_use]
    pub fn library_extension(&self) -> &'static str {
        match self {
            Self::LinuxX86_64 | Self::LinuxAarch64 => "so",
            Self::DarwinX86_64 | Self::DarwinAarch64 => "dylib",
            Self::WindowsX86_64 | Self::WindowsAarch64 => "dll",
            Self::Wasm32Wasip2 => "wasm",
        }
    }

    /// Get the library filename prefix for this platform.
    #[must_use]
    pub fn library_prefix(&self) -> &'static str {
        match self {
            Self::LinuxX86_64 | Self::LinuxAarch64 => "lib",
            Self::DarwinX86_64 | Self::DarwinAarch64 => "lib",
            Self::WindowsX86_64 | Self::WindowsAarch64 | Self::Wasm32Wasip2 => "",
        }
    }

    /// Format a library name for this platform.
    ///
    /// # Example
    ///
    /// ```
    /// use rustbridge_bundle::Platform;
    ///
    /// assert_eq!(Platform::LinuxX86_64.library_name("myplugin"), "libmyplugin.so");
    /// assert_eq!(Platform::WindowsX86_64.library_name("myplugin"), "myplugin.dll");
    /// ```
    #[must_use]
    pub fn library_name(&self, base_name: &str) -> String {
        format!(
            "{}{}.{}",
            self.library_prefix(),
            base_name,
            self.library_extension()
        )
    }

    /// Get the Rust target triple for this platform.
    #[must_use]
    pub fn rust_target(&self) -> &'static str {
        match self {
            Self::LinuxX86_64 => "x86_64-unknown-linux-gnu",
            Self::LinuxAarch64 => "aarch64-unknown-linux-gnu",
            Self::DarwinX86_64 => "x86_64-apple-darwin",
            Self::DarwinAarch64 => "aarch64-apple-darwin",
            Self::WindowsX86_64 => "x86_64-pc-windows-msvc",
            Self::WindowsAarch64 => "aarch64-pc-windows-msvc",
            Self::Wasm32Wasip2 => "wasm32-wasip2",
        }
    }

    /// Whether this platform carries a native shared library loaded through
    /// the C ABI. `false` for WebAssembly targets.
    #[must_use]
    pub fn is_native(&self) -> bool {
        !matches!(self, Self::Wasm32Wasip2)
    }

    /// Get all supported platforms, native and WebAssembly.
    #[must_use]
    pub fn all() -> &'static [Platform] {
        &[
            Self::LinuxX86_64,
            Self::LinuxAarch64,
            Self::DarwinX86_64,
            Self::DarwinAarch64,
            Self::WindowsX86_64,
            Self::WindowsAarch64,
            Self::Wasm32Wasip2,
        ]
    }

    /// Get the native platforms only (those [`Platform::current`] can return).
    #[must_use]
    pub fn native() -> &'static [Platform] {
        &[
            Self::LinuxX86_64,
            Self::LinuxAarch64,
            Self::DarwinX86_64,
            Self::DarwinAarch64,
            Self::WindowsX86_64,
            Self::WindowsAarch64,
        ]
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    fn Platform___current___returns_some_on_supported_platform() {
        // This test will pass on any supported platform
        let platform = Platform::current();
        // We can't assert a specific value since it depends on the host
        // But we can verify the round-trip works if we got a value
        if let Some(p) = platform {
            assert_eq!(Platform::parse(p.as_str()), Some(p));
        }
    }

    #[test]
    fn Platform___from_str___parses_valid_platforms() {
        assert_eq!(Platform::parse("linux-x86_64"), Some(Platform::LinuxX86_64));
        assert_eq!(
            Platform::parse("darwin-aarch64"),
            Some(Platform::DarwinAarch64)
        );
        assert_eq!(
            Platform::parse("windows-x86_64"),
            Some(Platform::WindowsX86_64)
        );
    }

    #[test]
    fn Platform___from_str___returns_none_for_invalid() {
        assert_eq!(Platform::parse("invalid"), None);
        assert_eq!(Platform::parse("linux-arm"), None);
    }

    #[test]
    fn Platform___library_name___formats_correctly() {
        assert_eq!(
            Platform::LinuxX86_64.library_name("myplugin"),
            "libmyplugin.so"
        );
        assert_eq!(
            Platform::DarwinAarch64.library_name("myplugin"),
            "libmyplugin.dylib"
        );
        assert_eq!(
            Platform::WindowsX86_64.library_name("myplugin"),
            "myplugin.dll"
        );
    }

    #[test]
    fn Platform___all___returns_seven_platforms() {
        assert_eq!(Platform::all().len(), 7);
    }

    #[test]
    fn Platform___native___excludes_wasm() {
        assert_eq!(Platform::native().len(), 6);
        assert!(Platform::native().iter().all(Platform::is_native));
    }

    #[test]
    fn Platform___wasm32_wasip2___round_trips_and_formats() {
        let platform = Platform::parse("wasm32-wasip2");

        assert_eq!(platform, Some(Platform::Wasm32Wasip2));
        assert_eq!(Platform::Wasm32Wasip2.as_str(), "wasm32-wasip2");
        assert_eq!(Platform::Wasm32Wasip2.library_name("app"), "app.wasm");
        assert_eq!(Platform::Wasm32Wasip2.rust_target(), "wasm32-wasip2");
        assert!(!Platform::Wasm32Wasip2.is_native());
    }

    #[test]
    fn Platform___serde___uses_platform_key() {
        let json = serde_json::to_string(&Platform::Wasm32Wasip2).unwrap();

        assert_eq!(json, "\"wasm32-wasip2\"");
        assert_eq!(
            serde_json::from_str::<Platform>(&json).unwrap(),
            Platform::Wasm32Wasip2
        );
    }
}
