# Scripts Directory

This directory contains automation scripts for development and CI workflows.

## pre-commit.sh

Comprehensive pre-commit validation script that runs all quality checks before code is committed.

### Usage

```bash
# Run all checks
./scripts/pre-commit.sh

# Skip slower tests (faster feedback)
./scripts/pre-commit.sh --fast

# Show help
./scripts/pre-commit.sh --help
```

### What It Checks

1. **Code Formatting** - `cargo fmt --all --check`
2. **Security & Licenses** - `cargo deny check`
   - Security advisories from RustSec
   - License compliance (blocks GPL/LGPL)
   - Dependency sources
3. **Rust Unit Tests** - `cargo test --workspace --lib`
4. **Rust Integration Tests** - `cargo test --workspace --test '*'` (skipped in --fast mode)
5. **Example Builds** - `cargo build -p hello-plugin --release`
6. **Java/Kotlin Tests** - `./gradlew test` (if Java is available)
7. **Clippy Lints** - `cargo clippy --workspace --examples --tests -- -D warnings` (skipped in --fast mode)

### CI Mode

The script automatically detects CI environments (via `CI` environment variable) and adjusts output:
- Disables color codes
- Provides verbose output
- Uses appropriate exit codes

```bash
# Run in CI mode
CI=true ./scripts/pre-commit.sh
```

### Exit Codes

- `0` - All checks passed
- `1` - One or more checks failed

## Recommendations

### Local Development

Add to your git hooks (optional):

```bash
# .git/hooks/pre-commit
#!/bin/bash
./scripts/pre-commit.sh --fast
```

Don't forget to make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

### CI Integration

See `.github/workflows/ci.yml` for an example of using this script in GitHub Actions.

The script is designed to be reusable in any CI system that supports bash scripts.
