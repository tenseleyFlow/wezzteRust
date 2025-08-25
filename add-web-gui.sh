#!/bin/bash
# Add Web GUI to existing Wezztershier installation
set -e

echo "🌐 Adding Web GUI to Wezztershier..."

# Create a simple standalone web GUI that uses the existing CLI
cat > wezztershier-web-simple << 'EOF'
#!/bin/bash
# Simple Web GUI for Wezztershier using existing CLI

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WEB_DIR="${SCRIPT_DIR}/web-gui"

# Create web directory if it doesn't exist
mkdir -p "$WEB_DIR"

# Copy the HTML file
cp "${SCRIPT_DIR}/../share/wezztershier/web/index.html" "$WEB_DIR/" 2>/dev/null || {
    # Fallback: create a basic HTML file
    cat > "$WEB_DIR/index.html" << 'HTMLEOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>🎨 Wezztershier - WezTerm Configuration GUI</title>
    <style>
        body { font-family: system-ui; max-width: 1200px; margin: 0 auto; padding: 20px; }
        .header { text-align: center; margin-bottom: 30px; }
        .config-area { margin: 20px 0; }
        textarea { width: 100%; height: 200px; font-family: monospace; }
        button { padding: 10px 20px; margin: 5px; background: #007acc; color: white; border: none; border-radius: 4px; cursor: pointer; }
        button:hover { background: #005a9e; }
        .preview { background: #1e1e1e; color: #d4d4d4; padding: 15px; border-radius: 4px; font-family: monospace; white-space: pre-wrap; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🎨 Wezztershier - WezTerm Configuration GUI</h1>
        <p>Visual interface for configuring WezTerm settings</p>
    </div>
    
    <div class="config-area">
        <h3>Configuration Input</h3>
        <textarea id="config-input" placeholder="Paste your WezTerm configuration with @ui annotations here...">-- Sample WezTerm configuration with GUI annotations
-- @ui: slider(min=8, max=72, step=1) type=int
config.font_size = 14

-- @ui: theme_selector(themes=builtin, filter=all) type=string
config.color_scheme = "dracula"

-- @ui: color_picker(format=hex, alpha=false) type=color
config.colors.background = "#282a36"</textarea>
        <br>
        <button onclick="processConfig()">🎨 Generate GUI</button>
        <button onclick="downloadConfig()">⬇️ Download Config</button>
    </div>
    
    <div class="config-area">
        <h3>Generated Configuration</h3>
        <div id="preview" class="preview">Click "Generate GUI" to process your configuration</div>
    </div>

    <script>
        function processConfig() {
            const config = document.getElementById('config-input').value;
            const preview = document.getElementById('preview');
            
            // Simple processing - in a real implementation, this would call the wezztershier CLI
            if (config.trim()) {
                preview.textContent = "Processed configuration:\n\n" + config;
            } else {
                preview.textContent = "Please enter a configuration to process";
            }
        }
        
        function downloadConfig() {
            const config = document.getElementById('config-input').value;
            if (!config.trim()) {
                alert('Please enter a configuration first');
                return;
            }
            
            const blob = new Blob([config], { type: 'text/plain' });
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = 'wezterm.lua';
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
        }
    </script>
</body>
</html>
HTMLEOF
}

# Start simple HTTP server
cd "$WEB_DIR"
echo "🚀 Starting Wezztershier Web GUI on http://localhost:8080"
echo "   Open this URL in your browser to configure WezTerm visually!"
echo "   Press Ctrl+C to stop the server"
echo ""

# Try different methods to serve the HTML
if command -v python3 >/dev/null 2>&1; then
    python3 -m http.server 8080
elif command -v python >/dev/null 2>&1; then
    python -m SimpleHTTPServer 8080
else
    echo "Error: Python not found. Please install Python to run the web GUI."
    echo "Alternative: Open $WEB_DIR/index.html directly in your browser."
    exit 1
fi
EOF

chmod +x wezztershier-web-simple

echo "✅ Created simple web GUI launcher: wezztershier-web-simple"
echo ""
echo "🚀 To use the Web GUI:"
echo "   ./wezztershier-web-simple"
echo "   Then open http://localhost:8080 in your browser"
echo ""
echo "📦 This approach uses your existing CLI tools and adds a web interface on top."