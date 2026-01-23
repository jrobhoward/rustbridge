#!/bin/bash
# Pre-commit validation script for rustbridge
#
# This script runs all validation checks before code is committed:
# - Code formatting (cargo fmt)
# - Security and license checks (cargo deny)
# - Rust unit tests
# - Java/Kotlin tests
# - Integration tests (when available)
#
# Usage:
#   ./scripts/pre-commit.sh           # Run all checks
#   ./scripts/pre-commit.sh --fast    # Skip slower tests
#   CI=true ./scripts/pre-commit.sh   # CI mode (no color, verbose)

set -e  # Exit on first error

# Colors for output (disabled in CI)
if [ -z "$CI" ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
FAST_MODE=false

# Parse arguments
for arg in "$@"; do
    case $arg in
        --fast)
            FAST_MODE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [--fast] [--help]"
            echo ""
            echo "Options:"
            echo "  --fast    Skip slower integration tests"
            echo "  --help    Show this help message"
            exit 0
            ;;
    esac
done

# Helper functions
print_header() {
    echo -e "${BLUE}===================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}===================================================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

# Check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Change to project root
cd "$PROJECT_ROOT"

echo ""
print_header "rustbridge Pre-Commit Validation"
echo ""

# ============================================================================
# 1. Check Dependencies
# ============================================================================
print_header "Checking Dependencies"

if ! command_exists cargo; then
    print_error "cargo not found. Please install Rust."
    exit 1
fi
print_success "cargo found"

if ! command_exists cargo-deny; then
    print_warning "cargo-deny not found. Installing..."
    cargo install cargo-deny || {
        print_error "Failed to install cargo-deny"
        exit 1
    }
fi
print_success "cargo-deny found"

if [ -d "rustbridge-java" ]; then
    if ! command_exists java; then
        print_warning "java not found. Java tests will be skipped."
    else
        print_success "java found"
    fi

    if [ ! -f "rustbridge-java/gradlew" ]; then
        print_warning "Gradle wrapper not found. Java tests will be skipped."
    else
        print_success "gradle wrapper found"
    fi
fi

echo ""

# ============================================================================
# 2. Code Formatting
# ============================================================================
print_header "Checking Code Formatting (cargo fmt)"

if ! cargo fmt --all -- --check; then
    print_error "Code formatting check failed!"
    echo ""
    print_info "Run 'cargo fmt --all' to fix formatting issues."
    exit 1
fi
print_success "All Rust code is properly formatted"

echo ""

# ============================================================================
# 3. Security and License Checks
# ============================================================================
print_header "Running Security and License Checks (cargo deny)"

if [ ! -f "deny.toml" ]; then
    print_warning "deny.toml not found. Skipping cargo deny checks."
else
    if ! cargo deny check; then
        print_error "cargo deny checks failed!"
        echo ""
        print_info "Review the issues above and update dependencies or deny.toml as needed."
        exit 1
    fi
    print_success "All cargo deny checks passed"
fi

echo ""

# ============================================================================
# 4. Rust Unit Tests
# ============================================================================
print_header "Running Rust Unit Tests"

if ! cargo test --workspace --lib; then
    print_error "Rust unit tests failed!"
    exit 1
fi
print_success "All Rust unit tests passed"

echo ""

# ============================================================================
# 5. Rust Integration Tests
# ============================================================================
if [ "$FAST_MODE" = false ]; then
    print_header "Running Rust Integration Tests"

    # Check if there are any integration tests
    # Integration tests are in crates/*/tests/*.rs directories
    if find . -path "*/tests/*.rs" -type f 2>/dev/null | grep -q .; then
        if ! cargo test --workspace --test '*'; then
            print_error "Rust integration tests failed!"
            exit 1
        fi
        print_success "All Rust integration tests passed"
    else
        print_info "No integration tests found (define tests in crates/*/tests/*.rs)"
    fi

    echo ""
else
    print_info "Skipping Rust integration tests (--fast mode)"
    echo ""
fi

# ============================================================================
# 6. Build Example Plugins
# ============================================================================
print_header "Building Example Plugins"

if ! cargo build -p hello-plugin --release; then
    print_error "Failed to build hello-plugin"
    exit 1
fi
print_success "hello-plugin built successfully"

echo ""

# ============================================================================
# 7. Java/Kotlin Tests
# ============================================================================
if [ -d "rustbridge-java" ] && [ -f "rustbridge-java/gradlew" ] && command_exists java; then
    print_header "Running Java/Kotlin Tests"

    cd rustbridge-java

    if ! ./gradlew test; then
        print_error "Java/Kotlin tests failed!"
        exit 1
    fi
    print_success "All Java/Kotlin tests passed"

    cd "$PROJECT_ROOT"
    echo ""
else
    print_info "Skipping Java/Kotlin tests (not available)"
    echo ""
fi

# ============================================================================
# 8. Integration/E2E Tests (Future)
# ============================================================================
if [ "$FAST_MODE" = false ]; then
    if [ -d "tests/integration" ]; then
        print_header "Running Integration/E2E Tests"

        # Add integration test commands here when available
        # Example:
        # if ! ./scripts/run-integration-tests.sh; then
        #     print_error "Integration tests failed!"
        #     exit 1
        # fi

        print_info "No integration tests defined yet"
        echo ""
    fi
fi

# ============================================================================
# 9. Clippy (Optional but Recommended)
# ============================================================================
if [ "$FAST_MODE" = false ]; then
    print_header "Running Clippy (Lints)"

    if ! cargo clippy --workspace --examples --tests -- -D warnings; then
        print_error "Clippy checks failed!"
        echo ""
        print_info "Fix the warnings above or adjust clippy configuration."
        exit 1
    fi
    print_success "All clippy checks passed"

    echo ""
else
    print_info "Skipping clippy checks (--fast mode)"
    echo ""
fi

# ============================================================================
# Summary
# ============================================================================
print_header "Pre-Commit Validation Complete"
echo ""
print_success "All checks passed! ✨"
echo ""
print_info "Your code is ready to commit."
echo ""

exit 0
