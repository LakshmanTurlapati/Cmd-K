#!/usr/bin/env bash
set -euo pipefail

# Build a production DMG for CMD+K (Intel + Apple Silicon) with proper code signing
# Ensures the binary is signed with the stable bundle identifier and embedded entitlements
# so macOS TCC can match AppleEvents permissions to the running binary.
#
# Usage: ./scripts/build-dmg.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="CMD+K"
IDENTIFIER="com.lakshmanturlapati.cmd-k"
VERSION="0.1.0"
TARGET="universal-apple-darwin"

APP_BUNDLE="$PROJECT_ROOT/src-tauri/target/$TARGET/release/bundle/macos/$APP_NAME.app"
ENTITLEMENTS="$PROJECT_ROOT/src-tauri/entitlements.plist"
DIST_DIR="$PROJECT_ROOT/dist"
DMG_NAME="$APP_NAME-$VERSION-universal.dmg"
DMG_PATH="$DIST_DIR/$DMG_NAME"

# --- Preflight checks ---

echo "[1/6] Checking prerequisites..."

if ! command -v rustup &>/dev/null; then
  echo "ERROR: rustup is not installed." >&2
  exit 1
fi

if ! command -v pnpm &>/dev/null; then
  echo "ERROR: pnpm is not installed." >&2
  exit 1
fi

if ! command -v hdiutil &>/dev/null; then
  echo "ERROR: hdiutil is not available (requires macOS)." >&2
  exit 1
fi

if [ ! -f "$ENTITLEMENTS" ]; then
  echo "ERROR: Entitlements file not found at: $ENTITLEMENTS" >&2
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

echo "[2/6] Building Tauri app for $TARGET..."
echo "  This compiles for both x86_64 and aarch64 -- it will take a while."

cd "$PROJECT_ROOT"
pnpm tauri build --target "$TARGET"

if [ ! -d "$APP_BUNDLE" ]; then
  echo "ERROR: Expected .app bundle not found at: $APP_BUNDLE" >&2
  echo "  Check the Tauri build output above for errors." >&2
  exit 1
fi

echo "  .app bundle created at: $APP_BUNDLE"

# --- Merge custom Info.plist keys ---
# Tauri auto-generates its own Info.plist and does NOT merge our custom
# src-tauri/Info.plist. We inject NSAppleEventsUsageDescription here so
# the production bundle has it before codesigning.

echo "  Merging custom Info.plist keys into production bundle..."
/usr/libexec/PlistBuddy -c "Add :NSAppleEventsUsageDescription string 'CMD+K needs to send commands to your terminal application.'" \
  "$APP_BUNDLE/Contents/Info.plist" 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :NSAppleEventsUsageDescription 'CMD+K needs to send commands to your terminal application.'" \
  "$APP_BUNDLE/Contents/Info.plist"
echo "  NSAppleEventsUsageDescription merged."

# --- Verify universal binary ---

echo "[3/6] Verifying universal binary..."

EXECUTABLE="$APP_BUNDLE/Contents/MacOS/cmd-k"
if [ ! -f "$EXECUTABLE" ]; then
  echo "ERROR: Executable not found at: $EXECUTABLE" >&2
  exit 1
fi

LIPO_OUTPUT=$(lipo -info "$EXECUTABLE")
echo "  $LIPO_OUTPUT"

if ! echo "$LIPO_OUTPUT" | grep -q "x86_64" || ! echo "$LIPO_OUTPUT" | grep -q "arm64"; then
  echo "WARNING: Binary may not be truly universal. Expected both x86_64 and arm64." >&2
fi

# --- Re-sign with explicit bundle identifier and entitlements ---

echo "[4/6] Re-signing .app bundle with explicit identifier and entitlements..."

codesign --force --deep --sign - \
  --identifier "$IDENTIFIER" \
  --entitlements "$ENTITLEMENTS" \
  "$APP_BUNDLE"

echo "  Signed with identifier: $IDENTIFIER"

# Verify the signing identity
echo "  Verifying code signature..."
CODESIGN_INFO=$(codesign -dvvv "$APP_BUNDLE" 2>&1)
ACTUAL_IDENTIFIER=$(echo "$CODESIGN_INFO" | grep "^Identifier=" | head -1 | cut -d= -f2)

if [ "$ACTUAL_IDENTIFIER" != "$IDENTIFIER" ]; then
  echo "ERROR: Identifier mismatch!" >&2
  echo "  Expected: $IDENTIFIER" >&2
  echo "  Got:      $ACTUAL_IDENTIFIER" >&2
  exit 1
fi

echo "  Identifier verified: $ACTUAL_IDENTIFIER"

# Verify entitlements are embedded
echo "  Verifying entitlements..."
ENTITLEMENTS_OUTPUT=$(codesign -d --entitlements - "$APP_BUNDLE" 2>&1)

if ! echo "$ENTITLEMENTS_OUTPUT" | grep -q "com.apple.security.automation.apple-events"; then
  echo "ERROR: Entitlements not embedded correctly!" >&2
  echo "  Expected com.apple.security.automation.apple-events in entitlements." >&2
  echo "  Output: $ENTITLEMENTS_OUTPUT" >&2
  exit 1
fi

echo "  Entitlements verified: com.apple.security.automation.apple-events is present"

# --- Create DMG ---

echo "[5/6] Creating DMG..."

mkdir -p "$DIST_DIR"

# Remove existing DMG if present
if [ -f "$DMG_PATH" ]; then
  rm -f "$DMG_PATH"
fi

# Create a temporary staging directory
STAGING_DIR=$(mktemp -d)
cp -R "$APP_BUNDLE" "$STAGING_DIR/$APP_NAME.app"

# Create a symlink to /Applications for drag-to-install
ln -s /Applications "$STAGING_DIR/Applications"

# Step 1: Create a read-write DMG so we can style the Finder window
RW_DMG_PATH="$DIST_DIR/${APP_NAME}-rw.dmg"
rm -f "$RW_DMG_PATH"

hdiutil create \
  -volname "$APP_NAME" \
  -srcfolder "$STAGING_DIR" \
  -ov \
  -format UDRW \
  "$RW_DMG_PATH"

rm -rf "$STAGING_DIR"

# Step 2: Mount the read-write DMG and style it with a background image
# First detach any stale mounts of this volume name
for vol in /Volumes/${APP_NAME}*; do
  [ -d "$vol" ] && hdiutil detach "$vol" -force 2>/dev/null || true
done
sleep 1

MOUNT_OUTPUT=$(hdiutil attach "$RW_DMG_PATH" -readwrite -noverify -noautoopen)
MOUNT_POINT=$(echo "$MOUNT_OUTPUT" | grep -o '/Volumes/.*' | head -1)
# Extract just the volume name (e.g. "CMD+K" from "/Volumes/CMD+K")
VOL_NAME=$(basename "$MOUNT_POINT")
echo "  Mounted DMG at: $MOUNT_POINT (volume: $VOL_NAME)"

# Copy background image into a hidden folder on the DMG volume
mkdir -p "$MOUNT_POINT/.background"
BG_IMG="$PROJECT_ROOT/dist/dmg-background.png"
if [ ! -f "$BG_IMG" ]; then
  echo "  Generating white background image..."
  python3 -c "
from PIL import Image
img = Image.new('RGB', (540, 300), color=(255, 255, 255))
img.save('$BG_IMG')
"
fi
cp "$BG_IMG" "$MOUNT_POINT/.background/background.png"

# Give Finder time to register the volume
sleep 2

osascript <<APPLESCRIPT
tell application "Finder"
    tell disk "$VOL_NAME"
        open
        set current view of container window to icon view
        set toolbar visible of container window to false
        set statusbar visible of container window to false
        set bounds of container window to {100, 100, 640, 400}
        set viewOptions to the icon view options of container window
        set arrangement of viewOptions to not arranged
        set icon size of viewOptions to 80
        set text size of viewOptions to 12
        set background picture of viewOptions to file ".background:background.png"
        set position of item "$APP_NAME.app" of container window to {150, 150}
        set position of item "Applications" of container window to {390, 150}
        close
        open
        update without registering applications
        delay 3
        close
    end tell
end tell
APPLESCRIPT

echo "  DMG window styled (white background image, black labels)."

# Step 3: Unmount, then convert to compressed read-only DMG
hdiutil detach "$MOUNT_POINT" -quiet
sleep 1

hdiutil convert "$RW_DMG_PATH" \
  -format UDZO \
  -o "$DMG_PATH"

rm -f "$RW_DMG_PATH"

if [ ! -f "$DMG_PATH" ]; then
  echo "ERROR: DMG file was not created." >&2
  exit 1
fi

DMG_SIZE=$(du -h "$DMG_PATH" | cut -f1)

# --- Done ---

echo "[6/6] Verifying final DMG..."

echo ""
echo "========================================="
echo "  Build complete!"
echo "  DMG:     $DMG_PATH"
echo "  Size:    $DMG_SIZE"
echo "========================================="
echo ""
echo "IMPORTANT: Before testing, reset TCC permissions:"
echo "  tccutil reset AppleEvents $IDENTIFIER"
echo ""
echo "Installation steps:"
echo "  1. Open the DMG: open $DMG_PATH"
echo "  2. Drag CMD+K to Applications"
echo "  3. Launch CMD+K from Applications"
echo "  4. Trigger a paste action -- macOS will show an AppleEvents"
echo "     permission prompt for each target terminal app"
echo "  5. Click 'Allow' when prompted"
echo ""
echo "Verify code signing:"
echo "  codesign -dvvv /Applications/$APP_NAME.app 2>&1 | grep Identifier"
echo "  codesign -d --entitlements - /Applications/$APP_NAME.app"
echo ""
echo "NOTE: This DMG is ad-hoc signed. Users will need to"
echo "right-click > Open to bypass Gatekeeper on first launch."
