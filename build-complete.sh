#!/bin/bash
# Complete build script for Wezztershier with Web GUI
set -e

echo "🚀 Building Complete Wezztershier with Web GUI..."

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean

# Build all components
echo "📦 Building CLI binary..."
cargo build -p wezztershier-cli --release

echo "🌐 Building Web GUI server..."
cargo build -p wezztershier-web --release

# Make GUI launcher executable
chmod +x wezztershier-gui

# Create package
echo "📦 Creating source package for RPM..."
mkdir -p target/package
tar --exclude='target' --exclude='.git' -czf target/package/wezztershier-0.1.0.tar.gz .

# Copy to RPM sources
echo "📦 Preparing RPM build..."
cp target/package/wezztershier-0.1.0.tar.gz /home/espadon/rpmbuild/SOURCES/

# Build RPM
echo "🎯 Building RPM package..."
rpmbuild -ba wezztershier.spec

echo "✅ Build complete!"
echo ""
echo "📋 Available packages:"
echo "   Base CLI: dnf install wezztershier"
echo "   With GUI: dnf install wezztershier wezztershier-gui"
echo ""
echo "🎨 To use the GUI:"
echo "   1. Install: dnf install wezztershier-gui"
echo "   2. Run: wezztershier-gui"
echo "   3. Open: http://localhost:8080 in your browser"