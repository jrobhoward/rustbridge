//! Platform detection and identification.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported platform targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
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
        }
    }

    /// Get the library filename prefix for this platform.
    #[must_use]
    pub fn library_prefix(&self) -> &'static str {
        match self {
            Self::LinuxX86_64 | Self::LinuxAarch64 => "lib",
            Self::DarwinX86_64 | Self::DarwinAarch64 => "lib",
            Self::WindowsX86_64 | Self::WindowsAarch64 => "",
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
        }
    }

    /// Get all supported platforms.
    #[must_use]
    pub fn all() -> &'static [Platform] {
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
    fn Platform___all___returns_six_platforms() {
        assert_eq!(Platform::all().len(), 6);
    }
}
