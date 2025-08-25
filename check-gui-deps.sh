#!/bin/bash
# Check GUI development dependencies for Tauri

echo "🔍 Checking GUI Development Dependencies for Tauri..."
echo

# Function to check if a package is available
check_package() {
    local pkg=$1
    local desc=$2
    echo -n "Checking $pkg ($desc)... "
    if dnf list available "$pkg" &>/dev/null; then
        echo "✅ Available"
        return 0
    elif rpm -qa | grep -q "^${pkg%-devel}"; then
        echo "📦 Base package installed"
        return 1
    else
        echo "❌ Not found"
        return 2
    fi
}

# Check essential packages
echo "=== Essential GUI Packages ==="
check_package "gtk3-devel" "GTK 3 development files"
check_package "glib2-devel" "GLib 2 development files"  
check_package "webkit2gtk3-devel" "WebKit2GTK 3 development files"
check_package "librsvg2-devel" "SVG rendering library"

echo
echo "=== Alternative Package Names ==="
check_package "webkit2gtk4.0-devel" "WebKit2GTK 4.0 (alternative)"
check_package "libappindicator3-devel" "App indicator (Ubuntu style)"
check_package "libappindicator-gtk3-devel" "App indicator (RHEL style)"

echo
echo "=== Build Tools ==="
check_package "gcc-c++" "C++ compiler"
check_package "pkgconfig" "Package config tool"
check_package "openssl-devel" "OpenSSL development files"

echo
echo "=== Repository Status ==="
echo -n "EPEL repository... "
if dnf repolist enabled | grep -q epel; then
    echo "✅ Enabled"
else
    echo "❌ Disabled"
    echo "Run: sudo dnf install epel-release"
fi

echo -n "CRB/PowerTools repository... "
if dnf repolist enabled | grep -q "crb\|powertools"; then
    echo "✅ Enabled"
else
    echo "❌ Disabled"  
    echo "Run: sudo dnf config-manager --enable crb"
fi

echo
echo "=== PKG-CONFIG Status ==="
echo -n "Checking pkg-config paths... "
if pkg-config --exists glib-2.0 2>/dev/null; then
    echo "✅ glib-2.0 found"
else
    echo "❌ glib-2.0 missing"
    echo "PKG_CONFIG_PATH: ${PKG_CONFIG_PATH:-not set}"
    echo "Try: export PKG_CONFIG_PATH=/usr/lib64/pkgconfig:/usr/share/pkgconfig"
fi

echo
echo "🎯 Next Steps:"
echo "1. Fix any missing repositories (EPEL, CRB)"
echo "2. Install missing packages"
echo "3. Retry: npm run tauri build"