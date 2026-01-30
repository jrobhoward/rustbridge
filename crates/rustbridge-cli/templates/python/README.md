# rustbridge Python Consumer Template

A minimal Python project template for consuming rustbridge plugins.

## Prerequisites

- **Python 3.9+**
- **A rustbridge plugin** - Your `.rbp` bundle file

## Quick Start

1. **Copy this template** to your project location

2. **Create and activate a virtual environment**:
   ```bash
   python -m venv .venv
   source .venv/bin/activate  # Linux/macOS
   # .venv\Scripts\activate   # Windows
   ```

3. **Install rustbridge Python library**:
   ```bash
   pip install -e /path/to/rustbridge/rustbridge-python
   ```

4. **Add your plugin bundle** - Copy your `.rbp` file to the project root

5. **Update main.py** - Set `bundle_path` to your `.rbp` file

6. **Run**:
   ```bash
   python main.py
   ```

## Documentation

- [rustbridge Documentation](https://github.com/jrobhoward/rustbridge)
- [Python Guide](https://github.com/jrobhoward/rustbridge/blob/main/docs/using-plugins/PYTHON.md)
