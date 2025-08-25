#!/bin/bash
# Create GUI package with proper structure for RPM build
set -e

echo "🎨 Creating Wezztershier GUI Package..."

# Create temporary build directory
BUILD_DIR="wezztershier-0.1.0"
rm -rf "$BUILD_DIR"
mkdir "$BUILD_DIR"

echo "📂 Copying necessary files..."

# Copy essential files for the package
cp wezztershier-gui "$BUILD_DIR/"
cp wezztershier.spec "$BUILD_DIR/"
cp LICENSE "$BUILD_DIR/"

# Copy templates and examples if they exist
cp -r templates "$BUILD_DIR/" 2>/dev/null || {
    echo "   Note: No templates directory found, creating basic ones..."
    mkdir -p "$BUILD_DIR/templates"
    cat > "$BUILD_DIR/templates/basic.lua" << 'EOF'
-- Basic WezTerm configuration template
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 14

-- @ui: select(options=Block,Underline,Bar) type=string
config.default_cursor_style = "Block"
EOF
}

cp -r examples "$BUILD_DIR/" 2>/dev/null || {
    echo "   Note: No examples directory found, creating basic one..."
    mkdir -p "$BUILD_DIR/examples"
    cat > "$BUILD_DIR/examples/test-config.lua" << 'EOF'
-- Example WezTerm configuration with UI annotations
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 16

-- @ui: color_picker(format=hex, alpha=false) type=color
config.colors = { background = "#282a36" }
EOF
}

# Create target/release structure for the spec
mkdir -p "$BUILD_DIR/target/release"

# Check if we have an existing CLI binary to copy
if [ -f "target/release/wezztershier" ]; then
    cp "target/release/wezztershier" "$BUILD_DIR/target/release/"
    echo "   ✅ Copied existing CLI binary"
else
    echo "   ⚠️  No existing CLI binary found"
    echo "      Creating placeholder - make sure you have a working wezztershier binary"
    echo '#!/bin/bash
echo "Wezztershier CLI placeholder"
echo "Install the actual binary or build from source"' > "$BUILD_DIR/target/release/wezztershier"
    chmod +x "$BUILD_DIR/target/release/wezztershier"
fi

echo "📦 Creating source tarball..."
tar -czf wezztershier-0.1.0.tar.gz "$BUILD_DIR"

echo "📁 Copying to RPM sources..."
cp wezztershier-0.1.0.tar.gz /home/espadon/rpmbuild/SOURCES/

echo "🏗️  Building RPM packages..."
rpmbuild -ba "$BUILD_DIR/wezztershier.spec"

echo ""
echo "✅ Build complete!"
echo ""
echo "📦 Check your packages:"
ls -la /home/espadon/rpmbuild/RPMS/x86_64/wezztershier*

echo ""
echo "🎯 To install:"
echo "   sudo rpm -Uvh /home/espadon/rpmbuild/RPMS/x86_64/wezztershier-0.1.0-*.rpm"
echo "   sudo rpm -Uvh /home/espadon/rpmbuild/RPMS/x86_64/wezztershier-gui-0.1.0-*.rpm"
echo ""
echo "🎨 To use the GUI:"
echo "   wezztershier-gui"
echo "   # Opens at http://localhost:8080"

# Clean up
rm -rf "$BUILD_DIR"