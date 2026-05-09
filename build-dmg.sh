#!/bin/bash
set -euo pipefail

APP_NAME="Rustty"
DMG_NAME="Rustty.dmg"
BUNDLE_DIR="target/release/bundle/osx"
APP_PATH="$BUNDLE_DIR/$APP_NAME.app"

echo "==> Building release binary and .app bundle..."
cargo bundle --release

if [ ! -d "$APP_PATH" ]; then
    echo "ERROR: $APP_PATH not found"
    exit 1
fi

echo "==> Creating $DMG_NAME..."
rm -f "$DMG_NAME"

if command -v create-dmg &> /dev/null; then
    create-dmg \
        --volname "$APP_NAME" \
        --window-size 600 400 \
        --app-drop-link 400 200 \
        --icon "$APP_NAME.app" 200 200 \
        "$DMG_NAME" \
        "$APP_PATH"
else
    hdiutil create \
        -volname "$APP_NAME" \
        -srcfolder "$APP_PATH" \
        -ov \
        -format UDZO \
        "$DMG_NAME"
fi

echo "==> Done! Created $DMG_NAME ($(du -h "$DMG_NAME" | cut -f1))"
