//! Plugin loader for dynamically loading rustbridge plugins.

use crate::error::{ConsumerError, ConsumerResult};
use crate::ffi_bindings::{
    FfiPluginHandle, LogCallback, PluginCallFn, PluginCallRawFn, PluginCreateFn,
    PluginFreeBufferFn, PluginGetRejectedCountFn, PluginGetStateFn, PluginInitFn,
    PluginSetLogLevelFn, PluginShutdownFn, RbResponseFreeFn,
};
use crate::plugin::NativePlugin;
use libloading::Library;
use rustbridge_bundle::BundleLoader;
use rustbridge_core::{LogLevel, PluginConfig};
use std::ffi::c_char;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tracing::debug;

/// Monotonic counter to ensure each bundle extraction gets a unique directory.
/// This prevents SIGBUS when multiple threads extract to the same path concurrently
/// (one thread truncates the .so file while another has it mmap'd).
static EXTRACT_INSTANCE: AtomicU64 = AtomicU64::new(0);

/// Rust-friendly log callback type.
///
/// This callback receives log messages from the plugin.
pub type LogCallbackFn = Arc<dyn Fn(LogLevel, &str, &str) + Send + Sync>;

// Global log callback storage.
// Uses a RwLock so the FFI callback can read from any thread while
// set_log_callback writes only during plugin load/unload.
static LOG_CALLBACK: std::sync::RwLock<Option<LogCallbackFn>> = std::sync::RwLock::new(None);

/// Set the global log callback.
fn set_log_callback(callback: Option<LogCallbackFn>) {
    if let Ok(mut guard) = LOG_CALLBACK.write() {
        *guard = callback;
    }
}

/// FFI-compatible log callback that forwards to the Rust callback.
///
/// # Safety
/// - `target` must be a valid null-terminated C string or null
/// - `message` must be valid for `message_len` bytes or null
unsafe extern "C" fn ffi_log_callback(
    level: u8,
    target: *const c_char,
    message: *const u8,
    message_len: usize,
) {
    let callback = LOG_CALLBACK.read().ok().and_then(|guard| guard.clone());
    if let Some(callback) = callback {
        let log_level = LogLevel::from_u8(level);

        // SAFETY: target is a valid null-terminated C string
        let target_str = if target.is_null() {
            ""
        } else {
            unsafe { std::ffi::CStr::from_ptr(target) }
                .to_str()
                .unwrap_or("")
        };

        // SAFETY: message is valid for message_len bytes (NOT null-terminated)
        let message_str = if message.is_null() || message_len == 0 {
            ""
        } else {
            let bytes = unsafe { std::slice::from_raw_parts(message, message_len) };
            std::str::from_utf8(bytes).unwrap_or("")
        };

        callback(log_level, target_str, message_str);
    }
}

/// Loader for native plugins.
///
/// Provides methods to load plugins from shared libraries or bundles.
pub struct NativePluginLoader;

impl NativePluginLoader {
    /// Load a plugin from a shared library path.
    ///
    /// Uses default configuration and no log callback.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the shared library (.so, .dylib, or .dll)
    ///
    /// # Example
    ///
    /// ```ignore
    /// let plugin = NativePluginLoader::load("target/release/libmy_plugin.so")?;
    /// ```
    pub fn load<P: AsRef<Path>>(path: P) -> ConsumerResult<NativePlugin> {
        Self::load_with_config(path, &PluginConfig::default(), None)
    }

    /// Load a plugin with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the shared library
    /// * `config` - Plugin configuration
    /// * `log_callback` - Optional callback to receive log messages
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = PluginConfig::default();
    /// let log_callback: LogCallbackFn = Arc::new(|level, target, msg| {
    ///     println!("[{level}] {target}: {msg}");
    /// });
    ///
    /// let plugin = NativePluginLoader::load_with_config(
    ///     "target/release/libmy_plugin.so",
    ///     &config,
    ///     Some(log_callback),
    /// )?;
    /// ```
    pub fn load_with_config<P: AsRef<Path>>(
        path: P,
        config: &PluginConfig,
        log_callback: Option<LogCallbackFn>,
    ) -> ConsumerResult<NativePlugin> {
        let path = path.as_ref();
        debug!("Loading plugin from: {}", path.display());

        // Load the shared library
        // SAFETY: We're loading a shared library which requires unsafe
        let library = unsafe { Library::new(path) }?;

        // Load required symbols
        let plugin_create: PluginCreateFn = unsafe { *library.get(b"plugin_create\0")? };
        let plugin_init: PluginInitFn = unsafe { *library.get(b"plugin_init\0")? };
        let plugin_call: PluginCallFn = unsafe { *library.get(b"plugin_call\0")? };
        let plugin_shutdown: PluginShutdownFn = unsafe { *library.get(b"plugin_shutdown\0")? };
        let plugin_get_state: PluginGetStateFn = unsafe { *library.get(b"plugin_get_state\0")? };
        let plugin_get_rejected_count: PluginGetRejectedCountFn =
            unsafe { *library.get(b"plugin_get_rejected_count\0")? };
        let plugin_set_log_level: PluginSetLogLevelFn =
            unsafe { *library.get(b"plugin_set_log_level\0")? };
        let plugin_free_buffer: PluginFreeBufferFn =
            unsafe { *library.get(b"plugin_free_buffer\0")? };

        // Load optional binary transport symbols
        let plugin_call_raw: Option<PluginCallRawFn> =
            unsafe { library.get(b"plugin_call_raw\0").ok().map(|s| *s) };
        let rb_response_free: Option<RbResponseFreeFn> =
            unsafe { library.get(b"rb_response_free\0").ok().map(|s| *s) };

        // Set up log callback if provided.
        // Only update the global when a real callback is given — loading a
        // plugin without a callback must not clobber an existing one, since
        // the FFI layer uses a single global callback for all instances.
        if log_callback.is_some() {
            set_log_callback(log_callback);
        }
        let ffi_callback: LogCallback = Some(ffi_log_callback);

        // Create the plugin instance
        // SAFETY: plugin_create returns a valid pointer or null
        let plugin_ptr = unsafe { plugin_create() };
        if plugin_ptr.is_null() {
            return Err(ConsumerError::NullHandle);
        }

        // Serialize config to JSON
        let config_json = serde_json::to_vec(config)?;

        // Initialize the plugin
        // SAFETY: plugin_ptr is valid, config_json is valid for its length
        let handle: FfiPluginHandle = unsafe {
            plugin_init(
                plugin_ptr,
                config_json.as_ptr(),
                config_json.len(),
                ffi_callback,
            )
        };

        if handle.is_null() {
            return Err(ConsumerError::NullHandle);
        }

        debug!("Plugin initialized with handle: {:?}", handle);

        // SAFETY: All pointers are valid and came from the library
        Ok(unsafe {
            NativePlugin::new(
                library,
                handle,
                plugin_call,
                plugin_call_raw,
                plugin_shutdown,
                plugin_get_state,
                plugin_get_rejected_count,
                plugin_set_log_level,
                plugin_free_buffer,
                rb_response_free,
            )
        })
    }

    /// Load a plugin from a bundle file.
    ///
    /// Extracts the library for the current platform and loads it.
    ///
    /// # Arguments
    ///
    /// * `bundle_path` - Path to the .rbp bundle file
    ///
    /// # Example
    ///
    /// ```ignore
    /// let plugin = NativePluginLoader::load_bundle("my-plugin-1.0.0.rbp")?;
    /// ```
    pub fn load_bundle<P: AsRef<Path>>(bundle_path: P) -> ConsumerResult<NativePlugin> {
        Self::load_bundle_with_config(bundle_path, &PluginConfig::default(), None)
    }

    /// Load a plugin from a bundle with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `bundle_path` - Path to the .rbp bundle file
    /// * `config` - Plugin configuration
    /// * `log_callback` - Optional callback to receive log messages
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = PluginConfig::default();
    /// let plugin = NativePluginLoader::load_bundle_with_config(
    ///     "my-plugin-1.0.0.rbp",
    ///     &config,
    ///     None,
    /// )?;
    /// ```
    pub fn load_bundle_with_config<P: AsRef<Path>>(
        bundle_path: P,
        config: &PluginConfig,
        log_callback: Option<LogCallbackFn>,
    ) -> ConsumerResult<NativePlugin> {
        let bundle_path = bundle_path.as_ref();
        debug!("Loading bundle from: {}", bundle_path.display());

        // Open and validate the bundle
        let mut loader = BundleLoader::open(bundle_path)?;

        // Check platform support
        if !loader.supports_current_platform() {
            return Err(ConsumerError::Bundle(
                rustbridge_bundle::BundleError::UnsupportedPlatform(
                    "Current platform not supported by bundle".to_string(),
                ),
            ));
        }

        // Each load gets a unique extraction directory to prevent SIGBUS from
        // concurrent threads overwriting a file that another thread has mmap'd.
        let instance_id = EXTRACT_INSTANCE.fetch_add(1, Ordering::Relaxed);
        let extract_dir = bundle_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(".rustbridge-cache")
            .join(loader.manifest().plugin.name.as_str())
            .join(loader.manifest().plugin.version.as_str())
            .join(instance_id.to_string());

        // Extract the library for the current platform
        let lib_path = loader.extract_library_for_current_platform(&extract_dir)?;

        debug!("Extracted library to: {}", lib_path.display());

        // Load the extracted library
        Self::load_with_config(lib_path, config, log_callback)
    }

    /// Load a specific variant from a bundle with custom configuration.
    ///
    /// Unlike `load_bundle_with_config` which always extracts the default (release) variant,
    /// this method extracts the named variant (e.g., "debug", "release").
    ///
    /// # Arguments
    ///
    /// * `bundle_path` - Path to the .rbp bundle file
    /// * `variant` - Variant name (e.g., "release", "debug")
    /// * `config` - Plugin configuration
    /// * `log_callback` - Optional callback to receive log messages
    ///
    /// # Example
    ///
    /// ```ignore
    /// let config = PluginConfig::default();
    /// let plugin = NativePluginLoader::load_bundle_variant_with_config(
    ///     "my-plugin-1.0.0.rbp",
    ///     "debug",
    ///     &config,
    ///     None,
    /// )?;
    /// ```
    pub fn load_bundle_variant_with_config<P: AsRef<Path>>(
        bundle_path: P,
        variant: &str,
        config: &PluginConfig,
        log_callback: Option<LogCallbackFn>,
    ) -> ConsumerResult<NativePlugin> {
        let bundle_path = bundle_path.as_ref();
        debug!(
            "Loading bundle variant '{}' from: {}",
            variant,
            bundle_path.display()
        );

        // Open and validate the bundle
        let mut loader = BundleLoader::open(bundle_path)?;

        // Check platform support
        let platform = rustbridge_bundle::Platform::current().ok_or_else(|| {
            ConsumerError::Bundle(rustbridge_bundle::BundleError::UnsupportedPlatform(
                "Current platform is not supported".to_string(),
            ))
        })?;

        if !loader.supports_current_platform() {
            return Err(ConsumerError::Bundle(
                rustbridge_bundle::BundleError::UnsupportedPlatform(
                    "Current platform not supported by bundle".to_string(),
                ),
            ));
        }

        // Each load gets a unique extraction directory to prevent SIGBUS from
        // concurrent threads overwriting a file that another thread has mmap'd.
        let instance_id = EXTRACT_INSTANCE.fetch_add(1, Ordering::Relaxed);
        let extract_dir = bundle_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(".rustbridge-cache")
            .join(loader.manifest().plugin.name.as_str())
            .join(loader.manifest().plugin.version.as_str())
            .join(format!("{variant}-{instance_id}"));

        // Extract the specified variant
        let lib_path = loader.extract_library_variant(platform, variant, &extract_dir)?;

        debug!("Extracted variant library to: {}", lib_path.display());

        // Load the extracted library
        Self::load_with_config(lib_path, config, log_callback)
    }

    /// Load a plugin from a bundle to a specific extraction directory.
    ///
    /// This is useful when you want to control where the library is extracted.
    ///
    /// # Arguments
    ///
    /// * `bundle_path` - Path to the .rbp bundle file
    /// * `extract_dir` - Directory to extract the library to
    /// * `config` - Plugin configuration
    /// * `log_callback` - Optional callback to receive log messages
    pub fn load_bundle_to_dir<P: AsRef<Path>, Q: AsRef<Path>>(
        bundle_path: P,
        extract_dir: Q,
        config: &PluginConfig,
        log_callback: Option<LogCallbackFn>,
    ) -> ConsumerResult<NativePlugin> {
        Self::load_bundle_verified(
            bundle_path,
            Some(extract_dir),
            config,
            log_callback,
            false,
            None,
        )
    }

    /// Load a plugin from a bundle with signature verification.
    ///
    /// # Arguments
    ///
    /// * `bundle_path` - Path to the .rbp bundle file
    /// * `config` - Plugin configuration
    /// * `log_callback` - Optional callback to receive log messages
    /// * `verify_signatures` - Whether to verify minisign signatures
    /// * `public_key_override` - Optional public key to use instead of manifest's key
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Load with signature verification (recommended for production)
    /// let plugin = NativePluginLoader::load_bundle_with_verification(
    ///     "my-plugin-1.0.0.rbp",
    ///     &PluginConfig::default(),
    ///     None,
    ///     true,  // verify signatures
    ///     None,  // use manifest's public key
    /// )?;
    /// ```
    pub fn load_bundle_with_verification<P: AsRef<Path>>(
        bundle_path: P,
        config: &PluginConfig,
        log_callback: Option<LogCallbackFn>,
        verify_signatures: bool,
        public_key_override: Option<&str>,
    ) -> ConsumerResult<NativePlugin> {
        Self::load_bundle_verified(
            bundle_path,
            None::<&Path>,
            config,
            log_callback,
            verify_signatures,
            public_key_override,
        )
    }

    /// Internal method to load a bundle with all options.
    fn load_bundle_verified<P: AsRef<Path>, Q: AsRef<Path>>(
        bundle_path: P,
        extract_dir: Option<Q>,
        config: &PluginConfig,
        log_callback: Option<LogCallbackFn>,
        verify_signatures: bool,
        public_key_override: Option<&str>,
    ) -> ConsumerResult<NativePlugin> {
        let bundle_path = bundle_path.as_ref();
        debug!("Loading bundle from: {}", bundle_path.display());

        // Open and validate the bundle
        let mut loader = BundleLoader::open(bundle_path)?;

        // Check platform support
        let platform = rustbridge_bundle::Platform::current().ok_or_else(|| {
            ConsumerError::Bundle(rustbridge_bundle::BundleError::UnsupportedPlatform(
                "Current platform is not supported".to_string(),
            ))
        })?;

        if !loader.supports_current_platform() {
            return Err(ConsumerError::Bundle(
                rustbridge_bundle::BundleError::UnsupportedPlatform(
                    "Current platform not supported by bundle".to_string(),
                ),
            ));
        }

        // Determine extraction directory
        // When no explicit dir is provided, each load gets a unique directory to
        // prevent SIGBUS from concurrent threads overwriting a mmap'd file.
        let extract_dir_path: std::path::PathBuf = match extract_dir {
            Some(dir) => dir.as_ref().to_path_buf(),
            None => {
                let instance_id = EXTRACT_INSTANCE.fetch_add(1, Ordering::Relaxed);
                bundle_path
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(".rustbridge-cache")
                    .join(loader.manifest().plugin.name.as_str())
                    .join(loader.manifest().plugin.version.as_str())
                    .join(instance_id.to_string())
            }
        };

        // Extract with or without verification
        let lib_path = if verify_signatures {
            loader.extract_library_verified(
                platform,
                &extract_dir_path,
                true,
                public_key_override,
            )?
        } else {
            loader.extract_library_for_current_platform(&extract_dir_path)?
        };

        debug!("Extracted library to: {}", lib_path.display());

        // Load the extracted library
        Self::load_with_config(lib_path, config, log_callback)
    }

    /// Load a plugin by name, searching standard library paths.
    ///
    /// Searches for the library in:
    /// 1. Current directory
    /// 2. `./target/release`
    /// 3. `./target/debug`
    /// 4. System library paths (LD_LIBRARY_PATH on Linux, etc.)
    ///
    /// # Arguments
    ///
    /// * `name` - Library name without prefix/suffix (e.g., "myplugin" finds "libmyplugin.so")
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Searches for libmyplugin.so (Linux), libmyplugin.dylib (macOS), myplugin.dll (Windows)
    /// let plugin = NativePluginLoader::load_by_name("myplugin")?;
    /// ```
    pub fn load_by_name(name: &str) -> ConsumerResult<NativePlugin> {
        Self::load_by_name_with_config(name, &PluginConfig::default(), None)
    }

    /// Load a plugin by name with custom configuration.
    pub fn load_by_name_with_config(
        name: &str,
        config: &PluginConfig,
        log_callback: Option<LogCallbackFn>,
    ) -> ConsumerResult<NativePlugin> {
        let lib_name = library_filename(name);

        // Search paths
        let search_paths = [
            std::path::PathBuf::from("."),
            std::path::PathBuf::from("./target/release"),
            std::path::PathBuf::from("./target/debug"),
        ];

        for search_path in &search_paths {
            let full_path = search_path.join(&lib_name);
            if full_path.exists() {
                debug!("Found library at: {}", full_path.display());
                return Self::load_with_config(full_path, config, log_callback);
            }
        }

        // Try loading directly (system library paths)
        debug!("Attempting to load '{}' from system paths", lib_name);
        Self::load_with_config(&lib_name, config, log_callback)
    }
}

/// Get the platform-specific library filename for a given name.
///
/// - Linux: `lib{name}.so`
/// - macOS: `lib{name}.dylib`
/// - Windows: `{name}.dll`
fn library_filename(name: &str) -> String {
    #[cfg(target_os = "linux")]
    {
        format!("lib{name}.so")
    }
    #[cfg(target_os = "macos")]
    {
        format!("lib{name}.dylib")
    }
    #[cfg(target_os = "windows")]
    {
        format!("{name}.dll")
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        format!("lib{name}.so")
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    #![allow(clippy::unwrap_used)]

    use super::*;
    use std::ffi::CString;

    #[test]
    fn NativePluginLoader___load___nonexistent_library___returns_error() {
        let result = NativePluginLoader::load("/nonexistent/library.so");

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, ConsumerError::LibraryLoad(_)));
    }

    #[test]
    fn NativePluginLoader___load_bundle___nonexistent_bundle___returns_error() {
        let result = NativePluginLoader::load_bundle("/nonexistent/bundle.rbp");

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, ConsumerError::Bundle(_)));
    }

    #[test]
    fn NativePluginLoader___load_bundle_variant___nonexistent_bundle___returns_error() {
        let result = NativePluginLoader::load_bundle_variant_with_config(
            "/nonexistent/bundle.rbp",
            "debug",
            &PluginConfig::default(),
            None,
        );

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(matches!(err, ConsumerError::Bundle(_)));
    }

    #[test]
    fn ffi_log_callback___no_callback_set___does_not_panic() {
        // Clear any existing callback
        set_log_callback(None);

        // Create target as null-terminated C string, message as bytes with length
        let target = CString::new("test").unwrap();
        let message = b"test message";

        // This should not panic
        unsafe {
            ffi_log_callback(2, target.as_ptr(), message.as_ptr(), message.len());
        }
    }

    #[test]
    fn ffi_log_callback___with_callback___invokes_callback() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let callback: LogCallbackFn = Arc::new(move |level, target, message| {
            assert_eq!(level, LogLevel::Info);
            assert_eq!(target, "test");
            assert_eq!(message, "test message");
            called_clone.store(true, Ordering::SeqCst);
        });

        set_log_callback(Some(callback));

        let target = CString::new("test").unwrap();
        let message = b"test message";

        unsafe {
            ffi_log_callback(2, target.as_ptr(), message.as_ptr(), message.len());
        }

        assert!(called.load(Ordering::SeqCst));

        // Clean up
        set_log_callback(None);
    }

    #[test]
    fn ffi_log_callback___null_pointers___uses_empty_strings() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let callback: LogCallbackFn = Arc::new(move |_level, target, message| {
            assert_eq!(target, "");
            assert_eq!(message, "");
            called_clone.store(true, Ordering::SeqCst);
        });

        set_log_callback(Some(callback));

        unsafe {
            ffi_log_callback(2, std::ptr::null(), std::ptr::null(), 0);
        }

        assert!(called.load(Ordering::SeqCst));

        // Clean up
        set_log_callback(None);
    }
}
