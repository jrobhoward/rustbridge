#!/bin/bash
#
# Build variant-log-plugin in 4 configurations and create 2 bundles for testing.
#
# Builds:
#   1. release (no features)       -> staging/release-base/
#   2. debug (no features)         -> staging/debug-base/
#   3. release + extended-info     -> staging/release-extended/
#   4. debug + extended-info       -> staging/debug-extended/
#
# Bundles:
#   variant-log-plugin-base-0.9.1.rbp       (release + debug, no features)
#   variant-log-plugin-extended-0.9.1.rbp    (release + debug, extended-info)
#
# Output: target/variant-test-bundles/
#
# Usage:
#   ./scripts/build-variant-test-bundles.sh

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_step() { echo -e "${BLUE}==>${NC} $1"; }
print_success() { echo -e "${GREEN}✓${NC} $1"; }
print_error() { echo -e "${RED}✗${NC} $1"; }

# Navigate to repo root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

# Determine library name
LIB_NAME="libvariant_log_plugin.so"
if [[ "$(uname -s)" == "Darwin" ]]; then
    LIB_NAME="libvariant_log_plugin.dylib"
fi

VERSION="0.9.1"
STAGING="target/variant-test-staging"
OUTPUT="target/variant-test-bundles"
CRATE="variant-log-plugin"

rm -rf "$STAGING"
mkdir -p "$STAGING"/{release-base,debug-base,release-extended,debug-extended}
mkdir -p "$OUTPUT"

# ============================================================================
# Build 4 variants
# ============================================================================

print_step "Build 1/4: release (no features)..."
cargo build --release -p "$CRATE"
cp "target/release/$LIB_NAME" "$STAGING/release-base/"
print_success "release base"

print_step "Build 2/4: debug (no features)..."
cargo build -p "$CRATE"
cp "target/debug/$LIB_NAME" "$STAGING/debug-base/"
print_success "debug base"

print_step "Build 3/4: release + extended-info..."
cargo build --release -p "$CRATE" --features extended-info
cp "target/release/$LIB_NAME" "$STAGING/release-extended/"
print_success "release extended"

print_step "Build 4/4: debug + extended-info..."
cargo build -p "$CRATE" --features extended-info
cp "target/debug/$LIB_NAME" "$STAGING/debug-extended/"
print_success "debug extended"

# ============================================================================
# Build rustbridge CLI if needed
# ============================================================================

RUSTBRIDGE_CLI="target/release/rustbridge"
if [ ! -f "$RUSTBRIDGE_CLI" ]; then
    print_step "Building rustbridge CLI..."
    cargo build --release -p rustbridge-cli
fi

# ============================================================================
# Detect platform string
# ============================================================================

SYSTEM="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$SYSTEM" in
    linux)  PLATFORM_OS="linux" ;;
    darwin) PLATFORM_OS="darwin" ;;
    *)      print_error "Unsupported OS: $SYSTEM"; exit 1 ;;
esac

case "$ARCH" in
    x86_64)       PLATFORM_ARCH="x86_64" ;;
    aarch64|arm64) PLATFORM_ARCH="aarch64" ;;
    *)            print_error "Unsupported arch: $ARCH"; exit 1 ;;
esac

PLATFORM="${PLATFORM_OS}-${PLATFORM_ARCH}"

# ============================================================================
# Create bundles
# ============================================================================

BASE_BUNDLE="$OUTPUT/variant-log-plugin-base-$VERSION.rbp"
EXTENDED_BUNDLE="$OUTPUT/variant-log-plugin-extended-$VERSION.rbp"

print_step "Creating base bundle (release + debug, no features)..."
"$RUSTBRIDGE_CLI" bundle create \
    --name "$CRATE" \
    --version "$VERSION" \
    --lib "${PLATFORM}:release:${STAGING}/release-base/${LIB_NAME}" \
    --lib "${PLATFORM}:debug:${STAGING}/debug-base/${LIB_NAME}" \
    --output "$BASE_BUNDLE"
print_success "Base bundle: $BASE_BUNDLE"

print_step "Creating extended bundle (release + debug, extended-info)..."
"$RUSTBRIDGE_CLI" bundle create \
    --name "$CRATE" \
    --version "$VERSION" \
    --lib "${PLATFORM}:release:${STAGING}/release-extended/${LIB_NAME}" \
    --lib "${PLATFORM}:debug:${STAGING}/debug-extended/${LIB_NAME}" \
    --output "$EXTENDED_BUNDLE"
print_success "Extended bundle: $EXTENDED_BUNDLE"

# ============================================================================
# Summary
# ============================================================================

echo ""
print_success "All variant test bundles built successfully!"
echo ""
echo "  Base bundle:     $BASE_BUNDLE ($(du -h "$BASE_BUNDLE" | cut -f1))"
echo "  Extended bundle: $EXTENDED_BUNDLE ($(du -h "$EXTENDED_BUNDLE" | cut -f1))"
echo ""
echo "Run integration tests with:"
echo "  cargo test -p rustbridge-consumer -- --ignored bundle_variant"
