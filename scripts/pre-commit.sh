#!/bin/bash
# Pre-commit validation script for rustbridge
#
# This script runs all validation checks before code is committed:
# - Code formatting (cargo fmt)
# - Security and license checks (cargo deny)
# - Rust unit tests
# - Java/Kotlin tests
# - C# tests
# - Integration tests (when available)
#
# Usage:
#   ./scripts/pre-commit.sh                # Run all checks
#   ./scripts/pre-commit.sh --fast         # Skip slower tests (integration, clippy)
#   ./scripts/pre-commit.sh --smart        # Only test what changed
#   ./scripts/pre-commit.sh --smart --fast # Smart + skip slow tests
#   CI=true ./scripts/pre-commit.sh        # CI mode (no color, verbose)

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
SMART_MODE=false

# What to test (set by smart detection or defaults)
RUN_RUST_TESTS=true
RUN_JAVA_TESTS=true
RUN_CSHARP_TESTS=true
RUN_RUST_FMT=true
RUN_CARGO_DENY=true

# Parse arguments
for arg in "$@"; do
    case $arg in
        --fast)
            FAST_MODE=true
            shift
            ;;
        --smart)
            SMART_MODE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [--fast] [--smart] [--help]"
            echo ""
            echo "Options:"
            echo "  --fast    Skip slower tests (integration tests, clippy)"
            echo "  --smart   Only run tests for components that changed"
            echo "  --help    Show this help message"
            echo ""
            echo "Examples:"
            echo "  $0                # Full validation"
            echo "  $0 --fast         # Quick validation (skip integration/clippy)"
            echo "  $0 --smart        # Test only what changed"
            echo "  $0 --smart --fast # Fastest: test only what changed, skip slow tests"
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
# Smart Mode: Detect What Changed
# ============================================================================
if [ "$SMART_MODE" = true ]; then
    print_header "Detecting Changes (Smart Mode)"

    # Get changed files (staged + unstaged, compared to HEAD)
    # Use --cached for only staged, or combine both
    CHANGED_FILES=$(git diff --name-only HEAD 2>/dev/null || true)
    STAGED_FILES=$(git diff --name-only --cached 2>/dev/null || true)
    ALL_CHANGES=$(echo -e "$CHANGED_FILES\n$STAGED_FILES" | sort -u | grep -v '^$' || true)

    if [ -z "$ALL_CHANGES" ]; then
        print_info "No changes detected. Running full validation."
        echo ""
    else
        # Categorize changes
        RUST_CHANGES=$(echo "$ALL_CHANGES" | grep -E '^(crates/|plugins/|Cargo\.(toml|lock)$|deny\.toml$|\.cargo/)' || true)
        JAVA_CHANGES=$(echo "$ALL_CHANGES" | grep -E '^rustbridge-java/' || true)
        CSHARP_CHANGES=$(echo "$ALL_CHANGES" | grep -E '^rustbridge-csharp/' || true)
        SCRIPT_CHANGES=$(echo "$ALL_CHANGES" | grep -E '^scripts/' || true)
        CONFIG_CHANGES=$(echo "$ALL_CHANGES" | grep -E '^(\.github/|rust-toolchain|clippy\.toml$)' || true)
        DOCS_ONLY=$(echo "$ALL_CHANGES" | grep -vE '\.(md|txt)$' | head -1 || true)

        # Determine what to run
        if [ -n "$SCRIPT_CHANGES" ] || [ -n "$CONFIG_CHANGES" ]; then
            # Config/script changes: run everything
            print_info "Config or script changes detected - running full validation"
            RUN_RUST_TESTS=true
            RUN_JAVA_TESTS=true
            RUN_CSHARP_TESTS=true
        elif [ -z "$DOCS_ONLY" ]; then
            # Only docs changed
            print_info "Only documentation changed - skipping tests"
            RUN_RUST_TESTS=false
            RUN_JAVA_TESTS=false
            RUN_CSHARP_TESTS=false
            RUN_CARGO_DENY=false
        else
            # Selective testing based on what changed
            if [ -z "$RUST_CHANGES" ]; then
                print_info "No Rust changes detected - skipping Rust tests"
                RUN_RUST_TESTS=false
                RUN_RUST_FMT=false
                RUN_CARGO_DENY=false
            else
                print_info "Rust changes detected:"
                echo "$RUST_CHANGES" | head -5 | sed 's/^/    /'
                [ $(echo "$RUST_CHANGES" | wc -l) -gt 5 ] && echo "    ... and more"
            fi

            if [ -z "$JAVA_CHANGES" ]; then
                print_info "No Java/Kotlin changes detected - skipping Java tests"
                RUN_JAVA_TESTS=false
            else
                print_info "Java/Kotlin changes detected:"
                echo "$JAVA_CHANGES" | head -5 | sed 's/^/    /'
                [ $(echo "$JAVA_CHANGES" | wc -l) -gt 5 ] && echo "    ... and more"
            fi

            if [ -z "$CSHARP_CHANGES" ]; then
                print_info "No C# changes detected - skipping C# tests"
                RUN_CSHARP_TESTS=false
            else
                print_info "C# changes detected:"
                echo "$CSHARP_CHANGES" | head -5 | sed 's/^/    /'
                [ $(echo "$CSHARP_CHANGES" | wc -l) -gt 5 ] && echo "    ... and more"
            fi

            # Special case: Rust FFI changes should trigger Java/C# tests
            # because Java/C# tests are integration tests that use the native lib
            if [ "$RUN_RUST_TESTS" = true ]; then
                FFI_CHANGES=$(echo "$RUST_CHANGES" | grep -E '(ffi|plugin_|FfiBuffer)' || true)
                if [ -n "$FFI_CHANGES" ]; then
                    if [ "$RUN_JAVA_TESTS" = false ]; then
                        print_warning "FFI changes detected - enabling Java tests for integration coverage"
                        RUN_JAVA_TESTS=true
                    fi
                    if [ "$RUN_CSHARP_TESTS" = false ]; then
                        print_warning "FFI changes detected - enabling C# tests for integration coverage"
                        RUN_CSHARP_TESTS=true
                    fi
                fi
            fi
        fi

        echo ""
        print_info "Test plan: Rust=$RUN_RUST_TESTS, Java=$RUN_JAVA_TESTS, C#=$RUN_CSHARP_TESTS, Fmt=$RUN_RUST_FMT, Deny=$RUN_CARGO_DENY"
        echo ""
    fi
fi

# ============================================================================
# 1. Check Dependencies
# ============================================================================
print_header "Checking Dependencies"

if ! command_exists cargo; then
    print_error "cargo not found. Please install Rust."
    exit 1
fi
print_success "cargo found"

if [ "$RUN_CARGO_DENY" = true ]; then
    if ! command_exists cargo-deny; then
        print_warning "cargo-deny not found. Installing..."
        cargo install cargo-deny || {
            print_error "Failed to install cargo-deny"
            exit 1
        }
    fi
    print_success "cargo-deny found"
fi

if [ "$RUN_JAVA_TESTS" = true ] && [ -d "rustbridge-java" ]; then
    if ! command_exists java; then
        print_warning "java not found. Java tests will be skipped."
        RUN_JAVA_TESTS=false
    else
        print_success "java found"
    fi

    if [ ! -f "rustbridge-java/gradlew" ]; then
        print_warning "Gradle wrapper not found. Java tests will be skipped."
        RUN_JAVA_TESTS=false
    else
        print_success "gradle wrapper found"
    fi
fi

if [ "$RUN_CSHARP_TESTS" = true ] && [ -d "rustbridge-csharp" ]; then
    if ! command_exists dotnet; then
        print_warning "dotnet not found. C# tests will be skipped."
        RUN_CSHARP_TESTS=false
    else
        print_success "dotnet found"
    fi
fi

echo ""

# ============================================================================
# 2. Code Formatting
# ============================================================================
if [ "$RUN_RUST_FMT" = true ]; then
    print_header "Checking Code Formatting (cargo fmt)"

    if ! cargo fmt --all -- --check; then
        print_error "Code formatting check failed!"
        echo ""
        print_info "Run 'cargo fmt --all' to fix formatting issues."
        exit 1
    fi
    print_success "All Rust code is properly formatted"

    echo ""
else
    print_info "Skipping Rust formatting check (no Rust changes)"
    echo ""
fi

# ============================================================================
# 3. Security and License Checks
# ============================================================================
if [ "$RUN_CARGO_DENY" = true ]; then
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
else
    print_info "Skipping cargo deny checks (no Rust changes)"
    echo ""
fi

# ============================================================================
# 4. Rust Unit Tests
# ============================================================================
if [ "$RUN_RUST_TESTS" = true ]; then
    print_header "Running Rust Unit Tests"

    if ! cargo test --workspace --lib; then
        print_error "Rust unit tests failed!"
        exit 1
    fi
    print_success "All Rust unit tests passed"

    echo ""
else
    print_info "Skipping Rust unit tests (no Rust changes)"
    echo ""
fi

# ============================================================================
# 5. Rust Integration Tests
# ============================================================================
if [ "$RUN_RUST_TESTS" = true ] && [ "$FAST_MODE" = false ]; then
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
elif [ "$RUN_RUST_TESTS" = false ]; then
    print_info "Skipping Rust integration tests (no Rust changes)"
    echo ""
else
    print_info "Skipping Rust integration tests (--fast mode)"
    echo ""
fi

# ============================================================================
# 6. Build Example Plugins
# ============================================================================
if [ "$RUN_RUST_TESTS" = true ]; then
    print_header "Building Example Plugins"

    if ! cargo build -p hello-plugin --release; then
        print_error "Failed to build hello-plugin"
        exit 1
    fi
    print_success "hello-plugin built successfully"

    echo ""
else
    print_info "Skipping example plugin build (no Rust changes)"
    echo ""
fi

# ============================================================================
# 7. Java/Kotlin Tests
# ============================================================================
if [ "$RUN_JAVA_TESTS" = true ] && [ -d "rustbridge-java" ] && [ -f "rustbridge-java/gradlew" ] && command_exists java; then
    print_header "Running Java/Kotlin Tests"

    cd rustbridge-java

    if ! ./gradlew test; then
        print_error "Java/Kotlin tests failed!"
        exit 1
    fi
    print_success "All Java/Kotlin tests passed"

    cd "$PROJECT_ROOT"
    echo ""
elif [ "$RUN_JAVA_TESTS" = false ]; then
    print_info "Skipping Java/Kotlin tests (no Java changes)"
    echo ""
else
    print_info "Skipping Java/Kotlin tests (not available)"
    echo ""
fi

# ============================================================================
# 8. C# Tests
# ============================================================================
if [ "$RUN_CSHARP_TESTS" = true ] && [ -d "rustbridge-csharp" ] && command_exists dotnet; then
    print_header "Running C# Tests"

    cd rustbridge-csharp

    if ! dotnet build; then
        print_error "C# build failed!"
        exit 1
    fi
    print_success "C# build succeeded"

    if ! dotnet test; then
        print_error "C# tests failed!"
        exit 1
    fi
    print_success "All C# tests passed"

    cd "$PROJECT_ROOT"
    echo ""
elif [ "$RUN_CSHARP_TESTS" = false ]; then
    print_info "Skipping C# tests (no C# changes)"
    echo ""
else
    print_info "Skipping C# tests (not available)"
    echo ""
fi

# ============================================================================
# 10. Integration/E2E Tests (Future)
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
# 11. Clippy (Optional but Recommended)
# ============================================================================
if [ "$RUN_RUST_TESTS" = true ] && [ "$FAST_MODE" = false ]; then
    print_header "Running Clippy (Lints)"

    if ! cargo clippy --workspace --examples --tests -- -D warnings; then
        print_error "Clippy checks failed!"
        echo ""
        print_info "Fix the warnings above or adjust clippy configuration."
        exit 1
    fi
    print_success "All clippy checks passed"

    echo ""
elif [ "$RUN_RUST_TESTS" = false ]; then
    print_info "Skipping clippy checks (no Rust changes)"
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
if [ "$SMART_MODE" = true ]; then
    print_success "All relevant checks passed! ✨"
    print_info "(Smart mode: only tested changed components)"
else
    print_success "All checks passed! ✨"
fi
echo ""
print_info "Your code is ready to commit."
echo ""

exit 0
