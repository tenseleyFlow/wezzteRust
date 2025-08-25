#!/bin/bash
# Build RPM packages for Wezzte

set -e

# Configuration
PACKAGE_NAME="wezzte"
VERSION="0.1.0"
RELEASE="1"
BUILD_DIR="/tmp/wezzte-rpm-build"
SPEC_FILE="packaging/wezzte.spec"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check if we're on an RPM-based system
if ! command -v rpmbuild &> /dev/null; then
    error "rpmbuild not found. Please install rpm-build package."
fi

# Check if we have the necessary build tools
if ! command -v cargo &> /dev/null; then
    error "cargo not found. Please install Rust."
fi

if ! command -v npm &> /dev/null; then
    error "npm not found. Please install Node.js."
fi

log "Starting RPM build process for $PACKAGE_NAME-$VERSION"

# Create build directory structure
log "Creating build directory structure"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

# Copy spec file
cp "$SPEC_FILE" "$BUILD_DIR/SPECS/"

# Create source tarball
log "Creating source tarball"
SOURCE_DIR="/tmp/wezzte-source"
rm -rf "$SOURCE_DIR"
mkdir -p "$SOURCE_DIR/$PACKAGE_NAME-$VERSION"

# Copy source files (exclude target and node_modules)
rsync -av \
    --exclude="target/" \
    --exclude="node_modules/" \
    --exclude=".git/" \
    --exclude="*.rpm" \
    --exclude="*.deb" \
    ./ "$SOURCE_DIR/$PACKAGE_NAME-$VERSION/"

# Create tarball
cd "$SOURCE_DIR"
tar -czf "$BUILD_DIR/SOURCES/$PACKAGE_NAME-$VERSION.tar.gz" "$PACKAGE_NAME-$VERSION"
cd - > /dev/null

# Build RPM packages
log "Building RPM packages"
rpmbuild --define "_topdir $BUILD_DIR" -ba "$BUILD_DIR/SPECS/wezzte.spec"

# Check build results
if [ $? -eq 0 ]; then
    log "RPM build successful!"
    
    # Copy RPMs to current directory
    find "$BUILD_DIR/RPMS" -name "*.rpm" -exec cp {} . \;
    find "$BUILD_DIR/SRPMS" -name "*.rpm" -exec cp {} . \;
    
    log "Created packages:"
    ls -la ./*.rpm 2>/dev/null || warn "No RPM files found in current directory"
    
else
    error "RPM build failed!"
fi

# Cleanup
log "Cleaning up build directory"
rm -rf "$BUILD_DIR" "$SOURCE_DIR"

log "RPM build process completed!"
log "Install with: sudo rpm -ivh wezzte-*.rpm"
log "Or with DNF: sudo dnf install ./wezzte-*.rpm"