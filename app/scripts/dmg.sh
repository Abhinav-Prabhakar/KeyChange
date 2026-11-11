#!/usr/bin/env bash
# Package KeyChange.app into a drag-to-Applications disk image.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
DMG="dist/KeyChange-$VERSION.dmg"
STAGE="dist/dmg"

scripts/bundle.sh

rm -rf "$STAGE" "$DMG"
mkdir -p "$STAGE"
cp -R dist/KeyChange.app "$STAGE/"
ln -s /Applications "$STAGE/Applications"

hdiutil create -quiet -volname "KeyChange" -srcfolder "$STAGE" \
  -fs HFS+ -format UDZO -imagekey zlib-level=9 "$DMG"

rm -rf "$STAGE"
echo "Wrote $DMG"
