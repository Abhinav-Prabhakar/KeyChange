#!/usr/bin/env bash
# Build KeyChange.app — a double-clickable macOS bundle with the icon and fonts.
set -euo pipefail
cd "$(dirname "$0")/.."

VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
APP="dist/KeyChange.app"

cargo build --release

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/keychange-app "$APP/Contents/MacOS/KeyChange"
cp Info.plist "$APP/Contents/Info.plist"
cp assets/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"
printf 'APPL????' > "$APP/Contents/PkgInfo"

# Ad-hoc signature: enough for Gatekeeper to see a stable, unmodified bundle.
codesign --force --deep --sign - "$APP" >/dev/null 2>&1 || true

echo "Built $APP ($VERSION)"
