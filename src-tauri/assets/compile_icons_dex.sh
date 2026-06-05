#!/usr/bin/env bash
# Compile AppIcons.java into app_icons.dex
# Requires: javac, and D8 (R8) which will be downloaded if missing.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

R8_JAR="$SCRIPT_DIR/r8.jar"
R8_URL="https://storage.googleapis.com/r8-releases/raw/8.12.28/r8.jar"

if [ ! -f "$R8_JAR" ]; then
    echo "Downloading D8/R8 compiler..."
    curl -fsSL -o "$R8_JAR" "$R8_URL" || { echo "Failed to download r8.jar"; exit 1; }
fi

echo "Compiling AppIcons.java..."
javac -source 8 -target 8 AppIcons.java

echo "Converting to DEX..."
java -cp "$R8_JAR" com.android.tools.r8.D8 --output . AppIcons.class
mv classes.dex app_icons.dex

echo "Cleaning up..."
rm -f AppIcons.class

echo "Done: app_icons.dex ($(wc -c < app_icons.dex) bytes)"