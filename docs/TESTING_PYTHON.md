# Python Testing Conventions

This document describes the testing conventions for the rustbridge Python bindings.

> **Note**: This guide follows the [shared testing conventions](./TESTING.md#cross-language-testing-conventions) used across all rustbridge languages, including the triple-underscore naming pattern and Arrange-Act-Assert structure.

## Quick Reference

```bash
# From rustbridge-python/
python -m venv .venv && source .venv/bin/activate  # Create virtual environment (first time)
pip install -e ".[dev]"                             # Install with dev dependencies

# Run tests
python -m pytest tests/ -v                          # Run all tests
python -m pytest tests/test_log_level.py -v         # Run specific test file
python -m pytest tests/ -k "test_from_code" -v      # Run tests matching pattern
python -m pytest tests/ --cov=rustbridge            # Run with coverage
```

## Test File Organization

```
rustbridge-python/
├── tests/
│   ├── __init__.py
│   ├── conftest.py                    # pytest fixtures
│   ├── test_log_level.py              # Unit tests for LogLevel
│   ├── test_lifecycle_state.py        # Unit tests for LifecycleState
│   ├── test_plugin_config.py          # Unit tests for PluginConfig
│   ├── test_minisign_verifier.py      # Unit tests for MinisignVerifier
│   ├── test_bundle_loader.py          # Unit tests for BundleLoader
│   └── test_hello_plugin_integration.py  # Integration tests
```

## Test Naming Convention

Tests use triple underscore naming to match the Rust/Java/C# conventions:

```
test_<subject>___<condition>___<expected_result>
```

Examples:
- `test_from_code___valid_codes___returns_correct_level`
- `test_call___echo_message___returns_response`
- `test_shutdown___explicit___state_becomes_stopped`

## Test Structure

Use the Arrange-Act-Assert pattern with blank lines separating sections:

```python
def test_from_code___valid_codes___returns_correct_level(self) -> None:
    # No "Arrange" comment - just blank line separation

    result = LogLevel.from_code(0)  # Act

    assert result == LogLevel.TRACE  # Assert
```

For simpler tests, sections can be combined:

```python
def test_from_code___invalid_code___raises_value_error(self) -> None:
    with pytest.raises(ValueError, match="Invalid log level code"):
        LogLevel.from_code(6)
```

## Test Classes

Group related tests in classes:

```python
class TestLogLevel:
    """Tests for LogLevel enum."""

    def test_from_code___valid_codes___returns_correct_level(self) -> None:
        assert LogLevel.from_code(0) == LogLevel.TRACE
        assert LogLevel.from_code(1) == LogLevel.DEBUG
        # ...

    def test_from_code___invalid_code___raises_value_error(self) -> None:
        with pytest.raises(ValueError):
            LogLevel.from_code(6)
```

## Fixtures

Common fixtures are defined in `conftest.py`:

```python
@pytest.fixture
def project_root() -> Path:
    """Return the project root directory."""
    return Path(__file__).parent.parent.parent

@pytest.fixture
def hello_plugin_path(project_root: Path) -> Path | None:
    """Return the path to the hello-plugin shared library."""
    # Returns None if not built
    ...

@pytest.fixture
def skip_if_no_plugin(hello_plugin_path: Path | None) -> None:
    """Skip test if hello-plugin is not built."""
    if hello_plugin_path is None:
        pytest.skip("hello-plugin not built")
```

## Integration Tests

Integration tests require the hello-plugin to be built:

```bash
# Build the plugin first
cargo build -p hello-plugin --release

# Run integration tests
python -m pytest tests/test_hello_plugin_integration.py -v
```

Integration tests use the `skip_if_no_plugin` fixture to skip gracefully if the plugin isn't available:

```python
class TestHelloPluginIntegration:
    def test_load___default_config___plugin_active(
        self, skip_if_no_plugin: None, hello_plugin_path: Path
    ) -> None:
        with NativePluginLoader.load(str(hello_plugin_path)) as plugin:
            assert plugin.state == LifecycleState.ACTIVE
```

## Mocking Platform Detection

Use `monkeypatch` for platform-dependent tests:

```python
def test_get_current_platform___linux_x86_64(
    self, monkeypatch: pytest.MonkeyPatch
) -> None:
    monkeypatch.setattr(platform, "system", lambda: "Linux")
    monkeypatch.setattr(platform, "machine", lambda: "x86_64")

    result = BundleLoader.get_current_platform()

    assert result == "linux-x86_64"
```

## Type Annotations

All test functions should have type annotations:

```python
def test_example___condition___result(self) -> None:
    ...
```

## Dependencies

Test dependencies are specified in `pyproject.toml`:

```toml
[project.optional-dependencies]
dev = [
    "pytest>=7.0",
    "pytest-cov>=4.0",
]
```

## Running Specific Tests

```bash
# By file
python -m pytest tests/test_log_level.py -v

# By class
python -m pytest tests/test_log_level.py::TestLogLevel -v

# By method
python -m pytest tests/test_log_level.py::TestLogLevel::test_from_code___valid_codes___returns_correct_level -v

# By keyword
python -m pytest tests/ -k "from_code" -v
```
