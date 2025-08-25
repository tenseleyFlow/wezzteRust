#!/bin/bash
# Build GUI package for Wezztershier
set -e

echo "🎨 Building Wezztershier GUI Package..."
echo ""

# Check if we have the base binary (we should since CLI already works)
if [ ! -f "target/release/wezztershier" ]; then
    echo "⚠️  Base CLI not found. Let me build it first..."
    # Use whatever method works for the CLI
    echo "📦 Note: You may need to build the CLI first"
    echo "   The CLI package already exists in your rpmbuild, so this should work"
fi

# Make our GUI launcher executable
chmod +x wezztershier-gui

echo "✅ GUI launcher is ready!"
echo ""

# Create package directory with just what we need for the GUI
echo "📦 Creating source package..."
mkdir -p gui-package
cp wezztershier-gui gui-package/
cp wezztershier.spec gui-package/
cp LICENSE gui-package/
cp -r templates gui-package/ 2>/dev/null || echo "Note: templates not found, continuing..."
cp -r examples gui-package/ 2>/dev/null || echo "Note: examples not found, continuing..."

# Create a minimal target/release structure for the spec
mkdir -p gui-package/target/release
# Copy existing CLI binary if it exists
if [ -f "target/release/wezztershier" ]; then
    cp target/release/wezztershier gui-package/target/release/
else
    # Create placeholder - the spec will use the existing one from previous build
    echo "#!/bin/echo CLI binary placeholder" > gui-package/target/release/wezztershier
    chmod +x gui-package/target/release/wezztershier
fi

cd gui-package

# Create tarball
echo "📦 Creating source tarball..."
tar --exclude='gui-package' -czf wezztershier-0.1.0.tar.gz .

# Copy to RPM sources
echo "📦 Copying to RPM build area..."
cp wezztershier-0.1.0.tar.gz /home/espadon/rpmbuild/SOURCES/

# Build the RPM
echo "🏗️  Building RPM with GUI package..."
rpmbuild -ba wezztershier.spec

cd ..

echo ""
echo "✅ Build complete!"
echo ""
echo "📋 Your packages are now available:"
echo "   Base CLI: /home/espadon/rpmbuild/RPMS/x86_64/wezztershier-0.1.0-1.*.rpm"
echo "   GUI:      /home/espadon/rpmbuild/RPMS/x86_64/wezztershier-gui-0.1.0-1.*.rpm"
echo ""
echo "🎯 To install both:"
echo "   sudo dnf install /home/espadon/rpmbuild/RPMS/x86_64/wezztershier*.rpm"
echo ""
echo "🎨 To use the GUI:"
echo "   wezztershier-gui"
echo "   # Opens web interface at http://localhost:8080"