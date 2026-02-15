#!/usr/bin/env bash
#
# Launch Git History Editor from ~/Applications
#
# Usage:
#   ./run/launch.sh
#

APP_PATH="$HOME/Applications/Git History Editor.app"

if [ ! -d "$APP_PATH" ]; then
    echo "ERROR: App not found at $APP_PATH"
    echo ""
    echo "Build and install it first:"
    echo "  INSTALL=1 ./run/build.sh"
    exit 1
fi

echo "Launching Git History Editor..."
open "$APP_PATH"
