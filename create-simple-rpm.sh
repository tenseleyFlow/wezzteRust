#!/bin/bash
# Create a simple RPM package with pre-built binary

set -euo pipefail

PACKAGE="wezztershier"
VERSION="0.1.0"
RELEASE="1"
ARCH="x86_64"

# Create temporary build structure
BUILD_ROOT=$(mktemp -d)
trap "rm -rf $BUILD_ROOT" EXIT

# Create directory structure
mkdir -p "$BUILD_ROOT/usr/bin"
mkdir -p "$BUILD_ROOT/usr/share/licenses/$PACKAGE"
mkdir -p "$BUILD_ROOT/usr/share/doc/$PACKAGE"
mkdir -p "$BUILD_ROOT/usr/share/$PACKAGE/templates"

# Copy files
cp /home/espadon/rpmbuild/BUILD/wezztershier-0.1.0/target/release/wezztershier "$BUILD_ROOT/usr/bin/"
cp LICENSE "$BUILD_ROOT/usr/share/licenses/$PACKAGE/"
cp README.md "$BUILD_ROOT/usr/share/doc/$PACKAGE/"
cp templates/*.lua "$BUILD_ROOT/usr/share/$PACKAGE/templates/" 2>/dev/null || true

# Create RPM using fpm
fpm -s dir -t rpm \
    --name "$PACKAGE" \
    --version "$VERSION" \
    --iteration "$RELEASE" \
    --architecture "$ARCH" \
    --description "Beautiful GUI generator for WezTerm configuration files" \
    --url "https://github.com/tenseleyFlow/wezzteRust" \
    --license "MIT" \
    --maintainer "espadonne (mfw) <espadonne@outlook.com>" \
    --depends "wezterm" \
    --suggests "lua" \
    --package ~/rpmbuild/RPMS/x86_64/ \
    -C "$BUILD_ROOT" \
    usr

echo "✅ RPM created successfully!"
ls -la ~/rpmbuild/RPMS/x86_64/${PACKAGE}-*.rpm