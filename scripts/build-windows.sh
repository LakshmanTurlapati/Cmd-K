#!/usr/bin/env bash
# Build Windows NSIS installer for CMD+K
# Requires: Windows with Rust toolchain, Node.js, pnpm
# Output: src-tauri/target/release/bundle/nsis/

set -euo pipefail

echo "=== CMD+K Windows Build ==="
echo "Building NSIS installer..."

# Install frontend dependencies
pnpm install

# Build the Tauri app (produces NSIS installer on Windows)
pnpm tauri build

echo ""
echo "=== Build Complete ==="
echo "Installer location: src-tauri/target/release/bundle/nsis/"
ls -la src-tauri/target/release/bundle/nsis/*.exe 2>/dev/null || echo "(Run on Windows to see .exe output)"
