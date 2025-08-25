# Build Instructions for wezztershier

## System Requirements

### RHEL/Rocky/AlmaLinux/CentOS Stream
```bash
# Enable development tools
sudo dnf groupinstall "Development Tools"
sudo dnf config-manager --set-enabled crb

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install system dependencies
sudo dnf install -y \
    gtk3-devel \
    webkit2gtk3-devel \
    librsvg2-devel \
    openssl-devel \
    nodejs npm

# Note: libayatana-appindicator3-devel might not be available on all RHEL variants
# In that case, try: libappindicator-gtk3-devel or skip GUI build
```

### Ubuntu/Debian
```bash
sudo apt update
sudo apt install -y \
    build-essential \
    curl \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    nodejs npm

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

## Build Commands

### CLI Only (Minimal Dependencies)
```bash
make build          # Build CLI only
make install        # Install CLI
```

### Full Build (CLI + GUI)
```bash
make build-gui      # Build CLI + GUI
```

### RPM Package
```bash
make rpm           # Build RPM (requires rpmbuild)
```

### Distribution Package
```bash
make dist          # Create source tarball
```

## Troubleshooting

### Missing libappindicator
If you get errors about libappindicator, try:
1. Skip GUI build: `cargo build --release -p wezztershier-cli`
2. Or install alternative packages:
   - RHEL: `libappindicator-gtk3-devel`
   - Some systems: `libappindicator3-1-dev`

### Missing webkit2gtk
On older systems, try: `webkitgtk4-devel`

### Node.js Issues
If npm fails, install Node.js from NodeSource:
```bash
curl -fsSL https://rpm.nodesource.com/setup_18.x | sudo bash -
sudo dnf install nodejs
```