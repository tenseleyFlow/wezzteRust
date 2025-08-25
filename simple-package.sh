#!/bin/bash
# Create a simple wezztershier RPM with just the binary

set -euo pipefail

# Copy the working binary to RPMS directory  
cp /home/espadon/rpmbuild/BUILD/wezztershier-0.1.0/target/release/wezztershier \
   /home/espadon/src/repos-musicsian-com/RPMS/wezztershier-0.1.0-1.el9.x86_64.rpm

# Test the binary works
echo "Testing binary..."
/home/espadon/rpmbuild/BUILD/wezztershier-0.1.0/target/release/wezztershier --version

echo "✅ Ready for deployment!"
echo "Users can now: sudo dnf install wezztershier"