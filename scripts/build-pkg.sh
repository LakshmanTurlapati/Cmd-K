#!/usr/bin/env bash
set -euo pipefail

# Build a universal .pkg installer for CMD+K (Intel + Apple Silicon)
# Usage: ./scripts/build-pkg.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="CMD+K"
IDENTIFIER="com.lakshmanturlapati.cmd-k"
VERSION="0.1.0"
TARGET="universal-apple-darwin"

APP_BUNDLE="$PROJECT_ROOT/src-tauri/target/$TARGET/release/bundle/macos/$APP_NAME.app"
DIST_DIR="$PROJECT_ROOT/dist"
PKG_NAME="$APP_NAME-$VERSION-universal.pkg"
PKG_PATH="$DIST_DIR/$PKG_NAME"

# --- Preflight checks ---

echo "[1/4] Checking prerequisites..."

if ! command -v rustup &>/dev/null; then
  echo "ERROR: rustup is not installed." >&2
  exit 1
fi

if ! command -v pnpm &>/dev/null; then
  echo "ERROR: pnpm is not installed." >&2
  exit 1
fi

if ! command -v pkgbuild &>/dev/null; then
  echo "ERROR: pkgbuild is not installed (requires Xcode command line tools)." >&2
  exit 1
fi

# Ensure both Rust targets are present
for arch_target in x86_64-apple-darwin aarch64-apple-darwin; do
  if ! rustup target list --installed | grep -q "$arch_target"; then
    echo "Adding missing Rust target: $arch_target"
    rustup target add "$arch_target"
  fi
done

echo "  All prerequisites met."

# --- Build universal binary ---

echo "[2/4] Building Tauri app for $TARGET..."
echo "  This compiles for both x86_64 and aarch64 -- it will take a while."

cd "$PROJECT_ROOT"
pnpm tauri build --target "$TARGET"

if [ ! -d "$APP_BUNDLE" ]; then
  echo "ERROR: Expected .app bundle not found at: $APP_BUNDLE" >&2
  echo "  Check the Tauri build output above for errors." >&2
  exit 1
fi

echo "  .app bundle created at: $APP_BUNDLE"

# --- Verify universal binary ---

echo "[3/4] Verifying universal binary..."

EXECUTABLE="$APP_BUNDLE/Contents/MacOS/$APP_NAME"
if [ ! -f "$EXECUTABLE" ]; then
  echo "ERROR: Executable not found at: $EXECUTABLE" >&2
  exit 1
fi

LIPO_OUTPUT=$(lipo -info "$EXECUTABLE")
echo "  $LIPO_OUTPUT"

if ! echo "$LIPO_OUTPUT" | grep -q "x86_64" || ! echo "$LIPO_OUTPUT" | grep -q "arm64"; then
  echo "WARNING: Binary may not be truly universal. Expected both x86_64 and arm64." >&2
fi

# --- Create .pkg installer ---

echo "[4/4] Creating .pkg installer..."

mkdir -p "$DIST_DIR"

# Create a temporary staging directory with the .app inside
STAGING_DIR=$(mktemp -d)
cp -R "$APP_BUNDLE" "$STAGING_DIR/$APP_NAME.app"

pkgbuild \
  --root "$STAGING_DIR" \
  --identifier "$IDENTIFIER" \
  --version "$VERSION" \
  --install-location /Applications \
  "$PKG_PATH"

rm -rf "$STAGING_DIR"

if [ ! -f "$PKG_PATH" ]; then
  echo "ERROR: .pkg file was not created." >&2
  exit 1
fi

PKG_SIZE=$(du -h "$PKG_PATH" | cut -f1)

echo ""
echo "========================================="
echo "  Build complete!"
echo "  Package: $PKG_PATH"
echo "  Size:    $PKG_SIZE"
echo "========================================="
echo ""
echo "NOTE: This package is unsigned. Users will need to"
echo "right-click > Open to bypass Gatekeeper on first launch."
