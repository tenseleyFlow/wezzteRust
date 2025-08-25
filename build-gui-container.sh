#!/bin/bash
# Container-based Tauri GUI build script
set -e

echo "🚀 Starting container-based Tauri GUI build..."

# Build the container image
echo "📦 Building Fedora 40 container with GUI dependencies..."
podman build -f Dockerfile.build -t wezztershier-builder .

# Create container and extract built binaries
echo "🏗️ Building Tauri GUI in container..."
CONTAINER_ID=$(podman create wezztershier-builder)

# Extract the built binary
echo "📤 Extracting built binary from container..."
mkdir -p ./container-build-output

# Try multiple possible locations for the binary
podman cp "$CONTAINER_ID:/workspace/src-tauri/target/release/wezztershier-tauri" ./container-build-output/ 2>/dev/null || true
podman cp "$CONTAINER_ID:/workspace/src-tauri/target/release/bundle/" ./container-build-output/ 2>/dev/null || true

# Clean up container
podman rm "$CONTAINER_ID"

echo "✅ Container build complete!"
echo "📁 Built files are in: ./container-build-output/"

# List what we got
ls -la ./container-build-output/