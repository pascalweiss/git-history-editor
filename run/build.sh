#!/usr/bin/env bash
#
# Build Git History Editor macOS app
#
# Builds an unsigned .app bundle that you can place in ~/Applications
# or code-sign yourself.
#
# Environment variables (optional):
#   UNIVERSAL       - Set to "1" to build universal binary (aarch64 + x86_64)
#                     Default: build for current architecture (faster)
#   INSTALL         - Set to "1" to automatically copy app to ~/Applications
#   SIGN_IDENTITY   - Developer ID to sign with (optional, e.g.
#                     "Developer ID Application: Your Name (TEAMID)")
#
# Usage:
#   ./run/build.sh                              # Build for current arch
#   UNIVERSAL=1 ./run/build.sh                  # Build universal
#   INSTALL=1 ./run/build.sh                    # Build and install to ~/Applications
#   SIGN_IDENTITY="..." INSTALL=1 ./run/build.sh  # Build, sign, and install
#

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Git History Editor"
BUILD_UNIVERSAL="${UNIVERSAL:-0}"
AUTO_INSTALL="${INSTALL:-0}"
SIGN_IDENTITY="${SIGN_IDENTITY:-}"

cd "$PROJECT_DIR"

echo "==> Building Git History Editor..."

if [ "$BUILD_UNIVERSAL" = "1" ]; then
    echo "   Target: Universal binary (aarch64 + x86_64)"
    npx tauri build --bundles app --target universal-apple-darwin
    APP_BUNDLE="src-tauri/target/universal-apple-darwin/release/bundle/macos/${APP_NAME}.app"
else
    ARCH=$(uname -m)
    echo "   Target: Current architecture ($ARCH)"
    npx tauri build --bundles app
    APP_BUNDLE="$(find src-tauri/target/release/bundle/macos -name '*.app' -maxdepth 1 | head -1)"
fi

if [ ! -d "$APP_BUNDLE" ]; then
    echo "ERROR: Build failed. App bundle not found at $APP_BUNDLE"
    exit 1
fi

echo "==> Build complete!"
echo "    App: $APP_BUNDLE"

# --- Optional: Sign ---
if [ -n "$SIGN_IDENTITY" ]; then
    echo "==> Signing with: $SIGN_IDENTITY"
    codesign --force --deep --options runtime \
        --sign "$SIGN_IDENTITY" \
        "$APP_BUNDLE"

    echo "==> Verifying signature..."
    codesign --verify --verbose=2 "$APP_BUNDLE" || {
        echo "WARNING: Signature verification failed"
    }
fi

# --- Optional: Install to ~/Applications ---
if [ "$AUTO_INSTALL" = "1" ]; then
    DEST_APP="$HOME/Applications/${APP_NAME}.app"
    echo ""
    echo "==> Installing to ~/Applications..."

    # Remove old version if it exists
    if [ -d "$DEST_APP" ]; then
        echo "   Removing old app..."
        rm -rf "$DEST_APP"
    fi

    cp -r "$APP_BUNDLE" "$DEST_APP"
    echo "    Installed: $DEST_APP"
    echo ""
    echo "==> To launch:"
    echo "   open \"$DEST_APP\""
    echo "   or"
    echo "   ./run/launch.sh"
else
    echo ""
    echo "==> To install to ~/Applications:"
    echo "   INSTALL=1 ./run/build.sh"
    echo ""
    echo "==> Or manually copy:"
    echo "   cp -r \"$APP_BUNDLE\" ~/Applications/"
fi
