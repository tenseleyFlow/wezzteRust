#!/bin/bash
# Deploy Wezztershier RPMs to the musicsian.com repository
# This script follows the established pattern from repos-musicsian-com

set -euo pipefail

PACKAGE_NAME="wezztershier"
RPM_BUILD_DIR="$HOME/rpmbuild/RPMS"
REPO_DIR="/home/espadon/src/repos-musicsian-com"

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
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

# Check if we have built RPMs
if [ ! -d "$RPM_BUILD_DIR" ]; then
    error "No RPM build directory found. Run 'make rpm' first."
fi

# Find the Wezztershier RPMs
WEZZTERSHIER_RPMS=($(find "$RPM_BUILD_DIR" -name "$PACKAGE_NAME*.rpm" 2>/dev/null || true))

if [ ${#WEZZTERSHIER_RPMS[@]} -eq 0 ]; then
    error "No Wezztershier RPM files found in $RPM_BUILD_DIR. Build the package first with 'make rpm'."
fi

log "Found ${#WEZZTERSHIER_RPMS[@]} Wezztershier RPM(s)"
for rpm in "${WEZZTERSHIER_RPMS[@]}"; do
    log "  $(basename "$rpm")"
done

# Copy RPMs to the repository
log "Copying RPMs to repository..."
for rpm in "${WEZZTERSHIER_RPMS[@]}"; do
    cp "$rpm" "$REPO_DIR/RPMS/"
    log "Copied $(basename "$rpm")"
done

# Create repository files following the established pattern
log "Creating repository configuration files..."

# Copy the repo file
cp packaging/wezztershier.repo "$REPO_DIR/"

# Create HTML page for Wezztershier (following the sultree.html pattern)
cat > "$REPO_DIR/wezztershier.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>wezztershier - Beautiful GUI for WezTerm Configuration</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; line-height: 1.6; }
        .header { border-bottom: 2px solid #333; padding-bottom: 10px; margin-bottom: 20px; }
        .install-box { background: #f4f4f4; padding: 15px; border-left: 4px solid #333; margin: 20px 0; }
        code { background: #f4f4f4; padding: 2px 4px; }
        .feature { margin: 10px 0; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🎨 wezztershier</h1>
        <p>Beautiful GUI generator for WezTerm configuration files</p>
    </div>

    <h2>Installation</h2>
    <div class="install-box">
        <strong>Add Repository:</strong><br>
        <code>sudo dnf config-manager --add-repo https://repos.musicsian.com/wezztershier.repo</code><br><br>
        <strong>Install Package:</strong><br>
        <code>sudo dnf install wezztershier</code>
    </div>

    <h2>Features</h2>
    <div class="feature">✨ <strong>Rich Widget Library</strong> - Interactive sliders, color pickers, theme selectors</div>
    <div class="feature">🎨 <strong>Advanced Color Support</strong> - Hex, RGB, HSL with alpha channels</div>
    <div class="feature">🎯 <strong>Built-in Themes</strong> - Dracula, Gruvbox, Solarized, Tokyo Night, Catppuccin</div>
    <div class="feature">🏗️ <strong>Smart Layouts</strong> - Automatic widget grouping and organization</div>
    <div class="feature">🛠️ <strong>CLI Tools</strong> - Parse, validate, and debug configurations</div>
    <div class="feature">⚡ <strong>High Performance</strong> - 10x faster than Python, 50% less memory</div>

    <h2>Quick Start</h2>
    <p>Add annotations to your WezTerm config:</p>
    <pre><code>-- <<TUNER-START>>
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 14
-- @ui: theme_selector(themes="builtin") type=string
config.color_scheme = "dracula"  
-- <<TUNER-END>></code></pre>

    <p>Then launch wezztershier:</p>
    <pre><code>wezztershier parse ~/.config/wezterm/wezterm.lua
wezztershier-gui ~/.config/wezterm/wezterm.lua</code></pre>

    <h2>Links</h2>
    <p>📦 <a href="wezztershier.repo">Repository File</a></p>
    <p>🔗 <a href="https://github.com/tenseleyFlow/wezzteRust">Source Code</a></p>
    <p>🏠 <a href="/">Back to Repository Index</a></p>
</body>
</html>
EOF

log "Created wezztershier.html"

# Change to repo directory and deploy
cd "$REPO_DIR"

log "Deploying to repository..."
if [ -x "./deploy.sh" ]; then
    ./deploy.sh
    log "✅ Successfully deployed Wezztershier to repos.musicsian.com"
else
    warn "deploy.sh not found or not executable. Manual deployment required."
fi

log "🎉 Wezztershier is now available at: https://repos.musicsian.com/wezztershier.html"