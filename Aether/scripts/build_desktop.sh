#!/bin/bash
set -e

# Navigation to Project Root
# Assuming script is run from project root or scripts/
# We need to find the root.
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$DIR/.."

echo "Project Root: $PROJECT_ROOT"
cd "$PROJECT_ROOT"

# 1. Build Backend
echo "🔨 Building Backend (Release Mode)..."
echo "---------------------------------------------------"
cd backend
cargo build --release
cd ..

# 2. Prepare Sidecar
echo "📦 Preparing Sidecar Binary..."
echo "---------------------------------------------------"
mkdir -p frontend/src-tauri/binaries

# Detect Architecture
ARCH=$(uname -m)
if [ "$ARCH" == "arm64" ]; then
  TRIPLE="aarch64-apple-darwin"
else
  TRIPLE="x86_64-apple-darwin"
fi

echo "Detected Architecture: $ARCH ($TRIPLE)"

cp backend/target/release/aether_backend "frontend/src-tauri/binaries/aether_backend-$TRIPLE"
chmod +x "frontend/src-tauri/binaries/aether_backend-$TRIPLE"

echo "✅ Sidecar copied to frontend/src-tauri/binaries/aether_backend-$TRIPLE"

# 3. Build Frontend
echo "🎨 Building Frontend..."
echo "---------------------------------------------------"
cd frontend
npm install # Ensure dependencies
npm run build
cd ..

# 4. Build Tauri App
echo "🚀 Building Desktop App..."
echo "---------------------------------------------------"
cd frontend

if ! command -v cargo-tauri &> /dev/null || ! cargo-tauri --version | grep -q "tauri-cli 1"; then
    echo "⚠️ cargo-tauri v1 not found. Attempting to install..."
    cargo install tauri-cli --version "^1.5" --force
fi

cargo tauri build

echo "✅ Build Complete! Check frontend/src-tauri/target/release/bundle/"
