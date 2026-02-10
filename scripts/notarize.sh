#!/usr/bin/env bash
#
# Notarize a signed .app bundle or .dmg with Apple.
#
# This submits the artifact to Apple's notarization service, waits for
# the result, and staples the notarization ticket.
#
# Prerequisites:
#   - Artifact must already be signed (run build.sh first for .app)
#   - App Store Connect API key, OR Apple ID with app-specific password
#
# Environment variables (API key method — recommended):
#   NOTARIZE_KEY_ID      - App Store Connect API key ID
#   NOTARIZE_ISSUER      - App Store Connect API issuer ID
#   NOTARIZE_KEY_PATH    - Path to the .p8 private key file
#
# Environment variables (Apple ID method — alternative):
#   NOTARIZE_APPLE_ID    - Your Apple ID email
#   NOTARIZE_TEAM_ID     - Your team ID
#   NOTARIZE_PASSWORD    - App-specific password (generate at appleid.apple.com)
#
# Usage:
#   ./scripts/notarize.sh path/to/Git\ History\ Editor.app
#   ./scripts/notarize.sh path/to/Git\ History\ Editor.dmg
#
set -euo pipefail

TARGET="${1:-}"

if [ -z "$TARGET" ]; then
    echo "Usage: $0 <path-to-.app-or-.dmg>"
    echo ""
    echo "Examples:"
    echo "  $0 'src-tauri/target/universal-apple-darwin/release/bundle/macos/Git History Editor.app'"
    echo "  $0 'dist/Git History Editor-0.1.0.dmg'"
    exit 1
fi

if [ ! -e "$TARGET" ]; then
    echo "ERROR: Not found: $TARGET"
    exit 1
fi

# --- Determine submission file ---
# .app bundles need to be zipped first; .dmg can be submitted directly
if [ -d "$TARGET" ]; then
    # It's a .app bundle — zip it
    APP_NAME="$(basename "$TARGET" .app)"
    SUBMIT_FILE="/tmp/${APP_NAME}-notarize.zip"
    echo "==> Creating zip for notarization..."
    ditto -c -k --keepParent "$TARGET" "$SUBMIT_FILE"
    CLEANUP_FILE="$SUBMIT_FILE"
else
    # It's a file (.dmg, .pkg, etc.) — submit directly
    SUBMIT_FILE="$TARGET"
    CLEANUP_FILE=""
fi

# --- Validate credentials ---
submit_args=()

if [ -n "${NOTARIZE_KEY_ID:-}" ] && [ -n "${NOTARIZE_ISSUER:-}" ] && [ -n "${NOTARIZE_KEY_PATH:-}" ]; then
    submit_args+=(--key "$NOTARIZE_KEY_PATH" --key-id "$NOTARIZE_KEY_ID" --issuer "$NOTARIZE_ISSUER")
elif [ -n "${NOTARIZE_APPLE_ID:-}" ] && [ -n "${NOTARIZE_TEAM_ID:-}" ] && [ -n "${NOTARIZE_PASSWORD:-}" ]; then
    submit_args+=(--apple-id "$NOTARIZE_APPLE_ID" --team-id "$NOTARIZE_TEAM_ID" --password "$NOTARIZE_PASSWORD")
else
    echo "ERROR: No notarization credentials provided."
    echo ""
    echo "Set one of these credential sets:"
    echo ""
    echo "  API Key (recommended):"
    echo "    export NOTARIZE_KEY_ID=<key-id>"
    echo "    export NOTARIZE_ISSUER=<issuer-id>"
    echo "    export NOTARIZE_KEY_PATH=~/.private_keys/AuthKey_XXXX.p8"
    echo ""
    echo "  Apple ID:"
    echo "    export NOTARIZE_APPLE_ID=you@example.com"
    echo "    export NOTARIZE_TEAM_ID=XXXXXXXXXX"
    echo "    export NOTARIZE_PASSWORD=xxxx-xxxx-xxxx-xxxx"
    [ -n "$CLEANUP_FILE" ] && rm -f "$CLEANUP_FILE"
    exit 1
fi

# --- Submit ---
echo "==> Submitting to Apple notarization service..."
xcrun notarytool submit "$SUBMIT_FILE" "${submit_args[@]}" --wait

# --- Staple ---
echo "==> Stapling notarization ticket..."
xcrun stapler staple "$TARGET"

# --- Clean up ---
[ -n "$CLEANUP_FILE" ] && rm -f "$CLEANUP_FILE"

echo ""
echo "Done! Notarized and stapled:"
echo "  $TARGET"
