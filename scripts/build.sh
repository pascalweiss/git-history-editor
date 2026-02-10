#!/usr/bin/env bash
#
# Build a signed universal macOS .app bundle.
#
# Prerequisites:
#   - Apple Developer Program membership
#   - "Developer ID Application" certificate installed in Keychain
#   - Rust targets: rustup target add aarch64-apple-darwin x86_64-apple-darwin
#
# Environment variables (required):
#   APPLE_SIGNING_IDENTITY  - Code signing identity, e.g.
#                             "Developer ID Application: Your Name (TEAMID)"
#
# Environment variables (optional):
#   SKIP_UNIVERSAL           - Set to "1" to build for current arch only (faster for testing)
#
# Usage:
#   APPLE_SIGNING_IDENTITY="Developer ID Application: ..." ./scripts/build.sh
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
APP_NAME="Git History Editor"

cd "$PROJECT_DIR"

# --- Validate environment ---
if [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
    echo "ERROR: APPLE_SIGNING_IDENTITY is not set."
    echo ""
    echo "Set it to your Developer ID Application certificate identity, e.g.:"
    echo '  export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAMID)"'
    echo ""
    echo "List available identities with:"
    echo "  security find-identity -v -p codesigning"
    exit 1
fi

# --- Build ---
if [ "${SKIP_UNIVERSAL:-}" = "1" ]; then
    echo "==> Building for current architecture only (SKIP_UNIVERSAL=1)..."
    npx tauri build --bundles app
    # Find the .app in the default target directory
    APP_BUNDLE="$(find src-tauri/target/release/bundle/macos -name '*.app' -maxdepth 1 | head -1)"
else
    echo "==> Building universal binary (aarch64 + x86_64)..."
    npx tauri build --bundles app --target universal-apple-darwin
    APP_BUNDLE="src-tauri/target/universal-apple-darwin/release/bundle/macos/${APP_NAME}.app"
fi

if [ ! -d "$APP_BUNDLE" ]; then
    echo "ERROR: Build output not found at $APP_BUNDLE"
    exit 1
fi

echo "==> Build complete: $APP_BUNDLE"

# --- Sign ---
echo "==> Signing with: $APPLE_SIGNING_IDENTITY"
codesign --force --deep --options runtime \
    --sign "$APPLE_SIGNING_IDENTITY" \
    "$APP_BUNDLE"

# --- Verify ---
echo "==> Verifying signature..."
codesign --verify --verbose=2 "$APP_BUNDLE"

echo ""
echo "Done! Signed app bundle at:"
echo "  $APP_BUNDLE"
