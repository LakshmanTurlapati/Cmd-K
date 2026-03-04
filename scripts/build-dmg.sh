#!/usr/bin/env bash
set -euo pipefail

# Build a production DMG for CMD+K (Intel + Apple Silicon) with proper
# Developer ID code signing, Apple notarization, and ticket stapling.
#
# The resulting DMG passes Gatekeeper on any Mac without right-click workarounds.
#
# Prerequisites:
#   - "Developer ID Application" certificate installed in keychain
#   - Notarytool credentials stored:
#       xcrun notarytool store-credentials "CMD-K-NOTARIZE" \
#         --apple-id "lakshmantvnm@gmail.com" --team-id "36L722DZ7X" \
#         --password "<app-specific-password>"
#
# Usage: ./scripts/build-dmg.sh

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

APP_NAME="CMD+K"
IDENTIFIER="com.lakshmanturlapati.cmd-k"
VERSION="0.2.2"
TARGET="universal-apple-darwin"
SIGNING_IDENTITY="Developer ID Application: VENKAT LUKSSHMAN TURLAPATI (36L722DZ7X)"
KEYCHAIN_PROFILE="CMD-K-NOTARIZE"

APP_BUNDLE="$PROJECT_ROOT/src-tauri/target/$TARGET/release/bundle/macos/$APP_NAME.app"
ENTITLEMENTS="$PROJECT_ROOT/src-tauri/entitlements.plist"
DIST_DIR="$PROJECT_ROOT/release"
DMG_NAME="$APP_NAME-$VERSION-universal.dmg"
DMG_PATH="$DIST_DIR/$DMG_NAME"

# ============================================================
# [1/10] Preflight Checks
# ============================================================

echo "[1/10] Checking prerequisites..."

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

# Verify Developer ID Application certificate is in the keychain
if ! security find-identity -v -p codesigning | grep -q "Developer ID Application"; then
  echo "ERROR: No 'Developer ID Application' certificate found in keychain." >&2
  echo "  Install your Developer ID certificate before running this script." >&2
  exit 1
fi
echo "  Developer ID certificate found in keychain."

# Ensure both Rust targets are present
for arch_target in x86_64-apple-darwin aarch64-apple-darwin; do
  if ! rustup target list --installed | grep -q "$arch_target"; then
    echo "  Adding missing Rust target: $arch_target"
    rustup target add "$arch_target"
  fi
done

echo "  All prerequisites met."

# ============================================================
# [2/10] Build Universal Binary
# ============================================================

echo "[2/10] Building Tauri app for $TARGET..."
echo "  This compiles for both x86_64 and aarch64 -- it will take a while."

cd "$PROJECT_ROOT"
pnpm tauri build --target "$TARGET"

if [ ! -d "$APP_BUNDLE" ]; then
  echo "ERROR: Expected .app bundle not found at: $APP_BUNDLE" >&2
  echo "  Check the Tauri build output above for errors." >&2
  exit 1
fi

echo "  .app bundle created at: $APP_BUNDLE"

# ============================================================
# [3/10] Merge Info.plist + Verify Universal Binary
# ============================================================

echo "[3/10] Merging Info.plist keys and verifying universal binary..."

# Tauri auto-generates its own Info.plist and does NOT merge our custom
# src-tauri/Info.plist. Inject NSAppleEventsUsageDescription before codesigning.
/usr/libexec/PlistBuddy -c "Add :NSAppleEventsUsageDescription string 'CMD+K needs to send commands to your terminal application.'" \
  "$APP_BUNDLE/Contents/Info.plist" 2>/dev/null || \
/usr/libexec/PlistBuddy -c "Set :NSAppleEventsUsageDescription 'CMD+K needs to send commands to your terminal application.'" \
  "$APP_BUNDLE/Contents/Info.plist"
echo "  NSAppleEventsUsageDescription merged."

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

# ============================================================
# [4/10] Code Signing (Developer ID, inside-out)
# ============================================================

echo "[4/10] Code signing with Developer ID (inside-out)..."
echo "  Identity: $SIGNING_IDENTITY"

# Sign inside-out: nested code first, main bundle last.
# This is required for proper notarization — Apple rejects bundles
# where nested binaries are unsigned or ad-hoc signed.

# 1) Sign all nested dylibs
echo "  Signing nested dylibs..."
if [ -d "$APP_BUNDLE/Contents/Frameworks" ]; then
  find "$APP_BUNDLE/Contents/Frameworks" -name '*.dylib' -type f | while read -r dylib; do
    echo "    $(basename "$dylib")"
    codesign --force --options runtime --timestamp \
      --sign "$SIGNING_IDENTITY" \
      "$dylib"
  done
else
  echo "    (no Frameworks directory — skipped)"
fi

# 2) Sign all nested frameworks
echo "  Signing nested frameworks..."
if [ -d "$APP_BUNDLE/Contents/Frameworks" ]; then
  find "$APP_BUNDLE/Contents/Frameworks" -name '*.framework' -type d | while read -r fw; do
    echo "    $(basename "$fw")"
    codesign --force --options runtime --timestamp \
      --sign "$SIGNING_IDENTITY" \
      "$fw"
  done
else
  echo "    (no Frameworks directory — skipped)"
fi

# 3) Sign any helper executables in MacOS/ (other than the main binary)
echo "  Signing helper executables..."
find "$APP_BUNDLE/Contents/MacOS" -type f -perm +111 ! -name "cmd-k" 2>/dev/null | while read -r helper; do
  echo "    $(basename "$helper")"
  codesign --force --options runtime --timestamp \
    --sign "$SIGNING_IDENTITY" \
    --entitlements "$ENTITLEMENTS" \
    "$helper"
done || true

# 4) Sign the main app bundle (outermost)
echo "  Signing main app bundle..."
codesign --force --options runtime --timestamp \
  --sign "$SIGNING_IDENTITY" \
  --identifier "$IDENTIFIER" \
  --entitlements "$ENTITLEMENTS" \
  "$APP_BUNDLE"

echo "  Code signing complete."

# ============================================================
# [5/10] Pre-Notarization Verification
# ============================================================

echo "[5/10] Verifying code signature before notarization..."

# Basic verification
codesign --verify --deep --strict "$APP_BUNDLE"
echo "  codesign --verify: PASSED"

# Check identifier
CODESIGN_INFO=$(codesign -dvvv "$APP_BUNDLE" 2>&1)
ACTUAL_IDENTIFIER=$(echo "$CODESIGN_INFO" | grep "^Identifier=" | head -1 | cut -d= -f2)

if [ "$ACTUAL_IDENTIFIER" != "$IDENTIFIER" ]; then
  echo "ERROR: Identifier mismatch!" >&2
  echo "  Expected: $IDENTIFIER" >&2
  echo "  Got:      $ACTUAL_IDENTIFIER" >&2
  exit 1
fi
echo "  Identifier: $ACTUAL_IDENTIFIER"

# Check hardened runtime flag
FLAGS_LINE=$(echo "$CODESIGN_INFO" | grep "CodeDirectory.*flags=" || true)
if ! echo "$FLAGS_LINE" | grep -q "runtime"; then
  echo "ERROR: Hardened runtime flag not set!" >&2
  echo "  The --options runtime flag is required for notarization." >&2
  echo "  Flags: $FLAGS_LINE" >&2
  exit 1
fi
echo "  Hardened runtime: enabled"

# Check timestamp
if ! echo "$CODESIGN_INFO" | grep -q "Timestamp="; then
  echo "ERROR: No secure timestamp found in signature!" >&2
  echo "  The --timestamp flag is required for notarization." >&2
  exit 1
fi
TIMESTAMP=$(echo "$CODESIGN_INFO" | grep "Timestamp=" | head -1)
echo "  $TIMESTAMP"

# Check signing authority
AUTHORITY=$(echo "$CODESIGN_INFO" | grep "^Authority=Developer ID Application" | head -1 || true)
if [ -z "$AUTHORITY" ]; then
  echo "ERROR: Not signed with Developer ID Application!" >&2
  echo "  Signature must use a Developer ID certificate for notarization." >&2
  echo "  Signing authorities:" >&2
  echo "$CODESIGN_INFO" | grep "^Authority=" >&2
  exit 1
fi
echo "  $AUTHORITY"

# Check entitlements
ENTITLEMENTS_OUTPUT=$(codesign -d --entitlements - "$APP_BUNDLE" 2>&1)
if ! echo "$ENTITLEMENTS_OUTPUT" | grep -q "com.apple.security.automation.apple-events"; then
  echo "ERROR: Entitlements not embedded correctly!" >&2
  echo "  Expected com.apple.security.automation.apple-events." >&2
  exit 1
fi
echo "  Entitlements: com.apple.security.automation.apple-events present"

echo "  All pre-notarization checks PASSED."

# ============================================================
# [6/10] Create DMG
# ============================================================

echo "[6/10] Creating DMG..."

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
VOL_NAME=$(basename "$MOUNT_POINT")
echo "  Mounted DMG at: $MOUNT_POINT (volume: $VOL_NAME)"

# Copy background image into a hidden folder on the DMG volume
mkdir -p "$MOUNT_POINT/.background"
BG_IMG="$PROJECT_ROOT/scripts/dmg-background.png"
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

echo "  DMG window styled."

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

echo "  DMG created at: $DMG_PATH"

# ============================================================
# [7/10] Sign DMG
# ============================================================

echo "[7/10] Signing DMG..."

codesign --force --timestamp \
  --sign "$SIGNING_IDENTITY" \
  "$DMG_PATH"

# Verify DMG signature
codesign --verify "$DMG_PATH"
echo "  DMG signed and verified."

# ============================================================
# [8/10] Submit to Notarization
# ============================================================

echo "[8/10] Submitting DMG to Apple notarization service..."
echo "  This may take several minutes. Waiting for result..."

NOTARIZE_OUTPUT=$(xcrun notarytool submit "$DMG_PATH" \
  --keychain-profile "$KEYCHAIN_PROFILE" \
  --wait 2>&1) || true

echo "$NOTARIZE_OUTPUT"

# Extract the submission ID for potential log retrieval
SUBMISSION_ID=$(echo "$NOTARIZE_OUTPUT" | grep -oE '[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}' | head -1 || true)

# Check if notarization succeeded
if echo "$NOTARIZE_OUTPUT" | grep -qi "status: Accepted"; then
  echo "  Notarization ACCEPTED."
else
  echo "ERROR: Notarization failed!" >&2
  if [ -n "$SUBMISSION_ID" ]; then
    echo "  Fetching notarization log for submission: $SUBMISSION_ID" >&2
    xcrun notarytool log "$SUBMISSION_ID" \
      --keychain-profile "$KEYCHAIN_PROFILE" 2>&1 || true
  fi
  exit 1
fi

# ============================================================
# [9/10] Staple Notarization Ticket
# ============================================================

echo "[9/10] Stapling notarization ticket to DMG..."

xcrun stapler staple "$DMG_PATH"
echo "  Ticket stapled."

# ============================================================
# [10/10] Final Verification
# ============================================================

echo "[10/10] Final verification..."

# Verify stapled ticket
xcrun stapler validate "$DMG_PATH"
echo "  Stapler validate: PASSED"

# Gatekeeper assessment
SPCTL_OUTPUT=$(spctl --assess --type install --verbose=2 "$DMG_PATH" 2>&1) || true
echo "  spctl: $SPCTL_OUTPUT"

if echo "$SPCTL_OUTPUT" | grep -qi "accepted"; then
  echo "  Gatekeeper: ACCEPTED"
else
  echo "  WARNING: spctl did not return 'accepted'. This may be normal on the" >&2
  echo "  build machine. Test on a separate Mac to confirm Gatekeeper passes." >&2
fi

DMG_SIZE=$(du -h "$DMG_PATH" | cut -f1)

echo ""
echo "========================================="
echo "  Build complete!"
echo "  DMG:          $DMG_PATH"
echo "  Size:         $DMG_SIZE"
echo "  Signed:       $SIGNING_IDENTITY"
echo "  Notarized:    YES"
echo "  Stapled:      YES"
echo "========================================="
echo ""
echo "Installation steps:"
echo "  1. Open the DMG: open \"$DMG_PATH\""
echo "  2. Drag CMD+K to Applications"
echo "  3. Launch CMD+K from Applications — no Gatekeeper warnings"
echo ""
