#!/usr/bin/env bash
#
# Full release pipeline: build → sign → notarize → create .dmg → notarize .dmg
#
# Required environment variables:
#   APPLE_SIGNING_IDENTITY  - See build.sh
#   NOTARIZE_*              - See notarize.sh (either API key or Apple ID set)
#
# Optional environment variables:
#   SKIP_UNIVERSAL          - Set to "1" to build for current arch only
#
# Usage:
#   export APPLE_SIGNING_IDENTITY="Developer ID Application: ..."
#   export NOTARIZE_KEY_ID=... NOTARIZE_ISSUER=... NOTARIZE_KEY_PATH=...
#   ./scripts/release.sh
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Git History Editor"

cd "$PROJECT_DIR"

# Read version from tauri.conf.json
VERSION=$(grep '"version"' src-tauri/tauri.conf.json | head -1 | sed 's/.*"\([0-9.]*\)".*/\1/')
echo "==> Releasing ${APP_NAME} v${VERSION}"
echo ""

# --- Step 1: Build and sign ---
echo "=== STEP 1/4: Build and Sign ==="
"$SCRIPT_DIR/build.sh"

# Determine the .app path (same logic as build.sh)
if [ "${SKIP_UNIVERSAL:-}" = "1" ]; then
    APP_BUNDLE="$(find src-tauri/target/release/bundle/macos -name '*.app' -maxdepth 1 | head -1)"
else
    APP_BUNDLE="src-tauri/target/universal-apple-darwin/release/bundle/macos/${APP_NAME}.app"
fi

echo ""

# --- Step 2: Notarize .app ---
echo "=== STEP 2/4: Notarize App ==="
"$SCRIPT_DIR/notarize.sh" "$APP_BUNDLE"
echo ""

# --- Step 3: Create DMG ---
echo "=== STEP 3/4: Create DMG ==="
DMG_NAME="${APP_NAME}-${VERSION}.dmg"
DMG_PATH="${PROJECT_DIR}/dist/${DMG_NAME}"

mkdir -p "$(dirname "$DMG_PATH")"
rm -f "$DMG_PATH"

# Stage DMG contents: app + Applications symlink
DMG_STAGING="/tmp/${APP_NAME}-dmg-staging"
rm -rf "$DMG_STAGING"
mkdir -p "$DMG_STAGING"
cp -R "$APP_BUNDLE" "$DMG_STAGING/"
ln -s /Applications "$DMG_STAGING/Applications"

echo "==> Creating DMG..."
hdiutil create -volname "$APP_NAME" \
    -srcfolder "$DMG_STAGING" \
    -ov -format UDZO \
    "$DMG_PATH"

rm -rf "$DMG_STAGING"

# Sign the DMG
echo "==> Signing DMG..."
codesign --force --sign "$APPLE_SIGNING_IDENTITY" "$DMG_PATH"

echo ""

# --- Step 4: Notarize DMG ---
echo "=== STEP 4/4: Notarize DMG ==="
"$SCRIPT_DIR/notarize.sh" "$DMG_PATH"

echo ""
echo "========================================"
echo " Release complete!"
echo "========================================"
echo ""
echo "  App:     $APP_BUNDLE"
echo "  DMG:     $DMG_PATH"
echo "  Version: $VERSION"
echo ""
echo "  To distribute:"
echo "    - Upload to GitHub Releases"
echo "    - Or host on your website"
echo "========================================"
