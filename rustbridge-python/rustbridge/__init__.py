"""
rustbridge - Python bindings for Rust shared libraries.

This package provides Python bindings for rustbridge plugins using ctypes.
It supports JSON-based transport and optional bundle loading with minisign
signature verification.

Example:
    # Direct library loading
    from rustbridge import NativePluginLoader, PluginConfig, LogLevel

    config = PluginConfig.defaults().log_level(LogLevel.DEBUG)
    with NativePluginLoader.load_with_config("libmyplugin.so", config) as plugin:
        response = plugin.call("echo", '{"message": "hello"}')
        print(response)

    # Bundle loading (with signature verification)
    from rustbridge import BundleLoader

    loader = BundleLoader(verify_signatures=True)
    with loader.load("my-plugin-1.0.0.rbp") as plugin:
        response = plugin.call("echo", '{"message": "hello"}')
"""

from rustbridge.core.log_level import LogLevel
from rustbridge.core.lifecycle_state import LifecycleState
from rustbridge.core.plugin_exception import PluginException
from rustbridge.core.plugin_config import PluginConfig
from rustbridge.core.response_envelope import ResponseEnvelope
from rustbridge.core.bundle_manifest import BundleManifest, PlatformInfo
from rustbridge.core.bundle_loader import BundleLoader
from rustbridge.core.minisign_verifier import MinisignVerifier
from rustbridge.native.structures import FfiBuffer
from rustbridge.native.native_plugin import NativePlugin
from rustbridge.native.plugin_loader import NativePluginLoader

__version__ = "0.1.0"

__all__ = [
    # Core types
    "LogLevel",
    "LifecycleState",
    "PluginException",
    "PluginConfig",
    "ResponseEnvelope",
    # Bundle loading
    "BundleManifest",
    "PlatformInfo",
    "BundleLoader",
    "MinisignVerifier",
    # Native bindings
    "FfiBuffer",
    "NativePlugin",
    "NativePluginLoader",
]
