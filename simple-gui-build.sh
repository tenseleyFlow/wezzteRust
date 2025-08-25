#!/bin/bash
# Simple GUI addition to existing Wezztershier package

echo "🎨 Adding Web GUI to existing Wezztershier package..."

# The strategy:
# 1. Your existing wezztershier CLI RPM already works
# 2. We just need to add the GUI launcher as a separate package
# 3. Keep it super simple - one script with embedded HTML

echo "✅ Files ready:"
echo "   📄 wezztershier-gui (launcher script with embedded HTML interface)"
echo "   📄 wezztershier.spec (updated with GUI package definition)"
echo ""

echo "🏗️  To build the complete package:"
echo ""
echo "1. Make sure your existing CLI build works:"
echo "   rpm -qi wezztershier  # Check if installed"
echo ""
echo "2. Create the source package:"
echo "   tar -czf wezztershier-0.1.0.tar.gz --exclude='.git' --exclude='target' ."
echo "   cp wezztershier-0.1.0.tar.gz /home/espadon/rpmbuild/SOURCES/"
echo ""
echo "3. Build both packages:"
echo "   rpmbuild -ba wezztershier.spec"
echo ""
echo "4. Install both packages:"
echo "   sudo dnf install /home/espadon/rpmbuild/RPMS/x86_64/wezztershier-0.1.0-*.rpm"
echo "   sudo dnf install /home/espadon/rpmbuild/RPMS/x86_64/wezztershier-gui-0.1.0-*.rpm"
echo ""
echo "5. Use the GUI:"
echo "   wezztershier-gui"
echo "   # Opens beautiful web interface at http://localhost:8080"
echo ""
echo "🎯 This approach:"
echo "   ✅ Uses your existing working CLI"
echo "   ✅ Adds GUI as separate optional package"
echo "   ✅ No complex build dependencies"
echo "   ✅ Beautiful responsive web interface"
echo "   ✅ Works with any browser"
echo ""
echo "Ready to build! Run the commands above to create your complete package."