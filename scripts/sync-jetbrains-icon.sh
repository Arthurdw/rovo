#!/bin/bash
set -e

ICON_SOURCE=".docs/rovo-icon.png"
ICON_DEST="jetbrains-plugin/src/main/resources/META-INF/pluginIcon.png"

# Check if source icon exists
if [ ! -f "$ICON_SOURCE" ]; then
    echo "Error: Source icon not found at $ICON_SOURCE"
    exit 1
fi

# Check if destination exists and if files are different
if [ ! -f "$ICON_DEST" ] || ! cmp -s "$ICON_SOURCE" "$ICON_DEST"; then
    echo "Syncing JetBrains plugin icon..."
    mkdir -p "$(dirname "$ICON_DEST")"
    cp "$ICON_SOURCE" "$ICON_DEST"
    git add "$ICON_DEST"
    echo "JetBrains plugin icon synced"
fi
