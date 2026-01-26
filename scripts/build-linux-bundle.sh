#!/bin/bash
#
# Build a multi-architecture Linux plugin bundle (.rbp)
#
# This script builds a plugin for both x86_64 and ARM64 Linux, then packages
# them into a single .rbp bundle. It uses native cross-compilation with
# gcc-aarch64-linux-gnu.
#
# Usage:
#   ./scripts/build-linux-bundle.sh [options] <plugin-crate> [version]
#
# Options:
#   --sign [key]   Sign the bundle. Uses ~/.rustbridge/signing.key by default.
#                  Creates the key if it doesn't exist.
#   --no-sign      Don't sign (default behavior)
#   --help         Show this help message
#
# Examples:
#   ./scripts/build-linux-bundle.sh hello-plugin
#   ./scripts/build-linux-bundle.sh hello-plugin 1.0.0
#   ./scripts/build-linux-bundle.sh --sign my-plugin              # Sign with default key
#   ./scripts/build-linux-bundle.sh --sign /path/to/key my-plugin # Sign with custom key
#
# Requirements:
#   - Linux x86_64 host
#   - Rust toolchain (rustup)
#   - ARM64 cross-compiler: sudo apt install gcc-aarch64-linux-gnu
#   - rustbridge CLI (will be built if not found)
#
# Output:
#   target/bundle/<plugin>-<version>.rbp
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Parse arguments
SIGN_ENABLED=""
SIGN_KEY=""
PLUGIN_CRATE=""
VERSION=""
DEFAULT_SIGN_KEY="$HOME/.rustbridge/signing.key"

show_help() {
    echo "Usage: $0 [options] <plugin-crate> [version]"
    echo ""
    echo "Build a multi-architecture Linux plugin bundle (.rbp)"
    echo ""
    echo "Options:"
    echo "  --sign [key]   Sign the bundle. Uses ~/.rustbridge/signing.key by default."
    echo "                 Creates the key if it doesn't exist."
    echo "  --help         Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 hello-plugin"
    echo "  $0 hello-plugin 1.0.0"
    echo "  $0 --sign hello-plugin                  # Sign with default key"
    echo "  $0 --sign ~/.my-key.key hello-plugin    # Sign with custom key"
    echo ""
    echo "Requirements:"
    echo "  - Linux x86_64 host"
    echo "  - ARM64 cross-compiler: sudo apt install gcc-aarch64-linux-gnu"
    echo "  - Rust toolchain"
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --sign)
            SIGN_ENABLED="true"
            shift
            # Check if next arg is a key path (not another option or plugin name)
            if [[ $# -gt 0 && "$1" != -* && -f "$1" ]]; then
                SIGN_KEY="$1"
                shift
            fi
            ;;
        --help|-h)
            show_help
            exit 0
            ;;
        -*)
            print_error "Unknown option: $1"
            show_help
            exit 1
            ;;
        *)
            if [ -z "$PLUGIN_CRATE" ]; then
                PLUGIN_CRATE="$1"
            elif [ -z "$VERSION" ]; then
                VERSION="$1"
            else
                print_error "Unexpected argument: $1"
                show_help
                exit 1
            fi
            shift
            ;;
    esac
done

if [ -z "$PLUGIN_CRATE" ]; then
    print_error "Missing required argument: plugin-crate"
    show_help
    exit 1
fi

# Get script directory and repo root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$REPO_ROOT"

# ============================================================================
# Pre-flight checks
# ============================================================================

print_step "Running pre-flight checks..."

# Check we're on Linux
if [[ "$(uname -s)" != "Linux" ]]; then
    print_error "This script only runs on Linux. Detected: $(uname -s)"
    exit 1
fi

# Check we're on x86_64
ARCH="$(uname -m)"
if [[ "$ARCH" != "x86_64" ]]; then
    print_error "This script requires x86_64 host. Detected: $ARCH"
    exit 1
fi

print_success "Running on Linux x86_64"

# Check Rust toolchain
if ! command -v cargo &> /dev/null; then
    print_error "Rust toolchain not found"
    echo "Install Rust: https://rustup.rs/"
    exit 1
fi

print_success "Rust toolchain found"

# Check for ARM64 cross-compiler
if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
    print_error "ARM64 cross-compiler not found"
    echo ""
    echo "Install it with:"
    echo "  sudo apt install gcc-aarch64-linux-gnu"
    echo ""
    echo "On other distros:"
    echo "  Fedora: sudo dnf install gcc-aarch64-linux-gnu"
    echo "  Arch:   sudo pacman -S aarch64-linux-gnu-gcc"
    exit 1
fi

print_success "ARM64 cross-compiler found"

# ============================================================================
# Setup Rust targets
# ============================================================================

# Ensure x86_64 target is installed
if ! rustup target list --installed | grep -q "x86_64-unknown-linux-gnu"; then
    print_step "Adding x86_64-unknown-linux-gnu target..."
    rustup target add x86_64-unknown-linux-gnu
fi

# Ensure aarch64 target is installed
if ! rustup target list --installed | grep -q "aarch64-unknown-linux-gnu"; then
    print_step "Adding aarch64-unknown-linux-gnu target..."
    rustup target add aarch64-unknown-linux-gnu
fi

print_success "Rust targets configured"

# ============================================================================
# Configure cross-compilation linker
# ============================================================================

CARGO_CONFIG_DIR="$REPO_ROOT/.cargo"
CARGO_CONFIG_FILE="$CARGO_CONFIG_DIR/config.toml"
LINKER_CONFIG='[target.aarch64-unknown-linux-gnu]
linker = "aarch64-linux-gnu-gcc"'

# Check if config already has the linker setting
if [ -f "$CARGO_CONFIG_FILE" ]; then
    if ! grep -q "aarch64-unknown-linux-gnu" "$CARGO_CONFIG_FILE"; then
        print_step "Adding ARM64 linker to .cargo/config.toml..."
        echo "" >> "$CARGO_CONFIG_FILE"
        echo "$LINKER_CONFIG" >> "$CARGO_CONFIG_FILE"
        print_success "Linker configuration added"
    else
        print_success "Linker already configured"
    fi
else
    print_step "Creating .cargo/config.toml with ARM64 linker..."
    mkdir -p "$CARGO_CONFIG_DIR"
    echo "$LINKER_CONFIG" > "$CARGO_CONFIG_FILE"
    print_success "Linker configuration created"
fi

# ============================================================================
# Determine plugin info
# ============================================================================

# Check plugin crate exists
PLUGIN_CARGO_TOML=""
if [ -f "plugins/$PLUGIN_CRATE/Cargo.toml" ]; then
    PLUGIN_CARGO_TOML="plugins/$PLUGIN_CRATE/Cargo.toml"
elif [ -f "examples/$PLUGIN_CRATE/Cargo.toml" ]; then
    PLUGIN_CARGO_TOML="examples/$PLUGIN_CRATE/Cargo.toml"
elif [ -f "crates/$PLUGIN_CRATE/Cargo.toml" ]; then
    PLUGIN_CARGO_TOML="crates/$PLUGIN_CRATE/Cargo.toml"
elif [ -f "$PLUGIN_CRATE/Cargo.toml" ]; then
    PLUGIN_CARGO_TOML="$PLUGIN_CRATE/Cargo.toml"
else
    print_error "Plugin crate not found: $PLUGIN_CRATE"
    echo "Searched in: plugins/, examples/, crates/, and root"
    exit 1
fi

print_success "Found plugin: $PLUGIN_CARGO_TOML"

# Extract version from Cargo.toml if not provided
if [ -z "$VERSION" ]; then
    VERSION_LINE=$(grep -m1 '^version' "$PLUGIN_CARGO_TOML")

    # Check if using workspace version inheritance
    if echo "$VERSION_LINE" | grep -q "workspace.*=.*true"; then
        # Get version from workspace root Cargo.toml
        VERSION=$(grep -m1 '^version' "$REPO_ROOT/Cargo.toml" | sed 's/.*"\(.*\)".*/\1/')
    else
        VERSION=$(echo "$VERSION_LINE" | sed 's/.*"\(.*\)".*/\1/')
    fi

    if [ -z "$VERSION" ]; then
        print_error "Could not extract version from $PLUGIN_CARGO_TOML"
        exit 1
    fi
fi

print_success "Plugin: $PLUGIN_CRATE v$VERSION"

# Determine library name (convert hyphens to underscores for .so name)
LIB_NAME="lib$(echo "$PLUGIN_CRATE" | tr '-' '_').so"

# ============================================================================
# Build for x86_64
# ============================================================================

print_step "Building for x86_64-unknown-linux-gnu..."

cargo build --release -p "$PLUGIN_CRATE" --target x86_64-unknown-linux-gnu

X86_LIB="target/x86_64-unknown-linux-gnu/release/$LIB_NAME"
if [ ! -f "$X86_LIB" ]; then
    print_error "x86_64 library not found: $X86_LIB"
    exit 1
fi

print_success "Built x86_64 library: $X86_LIB"

# ============================================================================
# Build for ARM64
# ============================================================================

print_step "Building for aarch64-unknown-linux-gnu..."

cargo build --release -p "$PLUGIN_CRATE" --target aarch64-unknown-linux-gnu

ARM64_LIB="target/aarch64-unknown-linux-gnu/release/$LIB_NAME"
if [ ! -f "$ARM64_LIB" ]; then
    print_error "ARM64 library not found: $ARM64_LIB"
    exit 1
fi

print_success "Built ARM64 library: $ARM64_LIB"

# ============================================================================
# Build rustbridge CLI if needed
# ============================================================================

RUSTBRIDGE_CLI="target/release/rustbridge"
if [ ! -f "$RUSTBRIDGE_CLI" ]; then
    print_step "Building rustbridge CLI..."
    cargo build --release -p rustbridge-cli
fi

if [ ! -f "$RUSTBRIDGE_CLI" ]; then
    print_error "Failed to build rustbridge CLI"
    exit 1
fi

print_success "rustbridge CLI ready"

# ============================================================================
# Create bundle
# ============================================================================

print_step "Creating bundle..."

BUNDLE_DIR="target/bundle"
mkdir -p "$BUNDLE_DIR"

BUNDLE_PATH="$BUNDLE_DIR/$PLUGIN_CRATE-$VERSION.rbp"

# Build the bundle create command
BUNDLE_CMD=("$RUSTBRIDGE_CLI" bundle create \
    --name "$PLUGIN_CRATE" \
    --version "$VERSION" \
    --lib "linux-x86_64:$X86_LIB" \
    --lib "linux-aarch64:$ARM64_LIB" \
    --output "$BUNDLE_PATH")

# Handle signing
if [ -n "$SIGN_ENABLED" ]; then
    # Use default key path if not specified
    if [ -z "$SIGN_KEY" ]; then
        SIGN_KEY="$DEFAULT_SIGN_KEY"
    fi

    # Create key if it doesn't exist
    if [ ! -f "$SIGN_KEY" ]; then
        # Create directory with secure permissions (like ~/.ssh/)
        SIGN_KEY_DIR="$(dirname "$SIGN_KEY")"
        if [ ! -d "$SIGN_KEY_DIR" ]; then
            print_step "Creating $SIGN_KEY_DIR with secure permissions..."
            mkdir -p "$SIGN_KEY_DIR"
            chmod 700 "$SIGN_KEY_DIR"
        fi

        print_step "Signing key not found, generating new key pair..."
        echo ""
        "$RUSTBRIDGE_CLI" keygen --output "$SIGN_KEY"
        echo ""
    fi

    if [ ! -f "$SIGN_KEY" ]; then
        print_error "Failed to create signing key"
        exit 1
    fi

    print_step "Bundle will be signed with: $SIGN_KEY"
    BUNDLE_CMD+=(--sign-key "$SIGN_KEY")
fi

"${BUNDLE_CMD[@]}"

if [ ! -f "$BUNDLE_PATH" ]; then
    print_error "Bundle creation failed"
    exit 1
fi

# ============================================================================
# Summary
# ============================================================================

echo ""
print_success "Bundle created successfully!"
echo ""
echo "  Bundle: $BUNDLE_PATH"
echo "  Size:   $(du -h "$BUNDLE_PATH" | cut -f1)"
echo ""
echo "Contents:"
"$RUSTBRIDGE_CLI" bundle list "$BUNDLE_PATH" | sed 's/^/  /'
echo ""
echo "To extract for current platform:"
echo "  $RUSTBRIDGE_CLI bundle extract $BUNDLE_PATH"
