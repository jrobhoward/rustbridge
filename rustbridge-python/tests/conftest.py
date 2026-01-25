"""pytest fixtures for rustbridge tests."""

import os
import sys
from pathlib import Path

import pytest

# Add the parent directory to the path so we can import rustbridge
sys.path.insert(0, str(Path(__file__).parent.parent))


@pytest.fixture
def project_root() -> Path:
    """Return the project root directory (rust_lang_interop)."""
    return Path(__file__).parent.parent.parent


@pytest.fixture
def hello_plugin_path(project_root: Path) -> Path | None:
    """
    Return the path to the hello-plugin shared library.

    Returns None if the library hasn't been built.
    """
    import platform

    system = platform.system()
    if system == "Linux":
        lib_name = "libhello_plugin.so"
    elif system == "Darwin":
        lib_name = "libhello_plugin.dylib"
    elif system == "Windows":
        lib_name = "hello_plugin.dll"
    else:
        return None

    # Try release first, then debug
    for build_type in ["release", "debug"]:
        lib_path = project_root / "target" / build_type / lib_name
        if lib_path.exists():
            return lib_path

    return None


@pytest.fixture
def skip_if_no_plugin(hello_plugin_path: Path | None) -> None:
    """Skip test if hello-plugin is not built."""
    if hello_plugin_path is None:
        pytest.skip("hello-plugin not built. Run: cargo build -p hello-plugin --release")
