#!/usr/bin/env bash
# Render assets/icon.svg into assets/AppIcon.icns (and a 1024 preview).
set -euo pipefail
cd "$(dirname "$0")/.."

CHROME=${CHROME:-"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"}
if [[ ! -x "$CHROME" ]]; then
  echo "Chrome not found. Set CHROME=/path/to/chrome and re-run." >&2
  exit 1
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

"$CHROME" --headless=new --disable-gpu --hide-scrollbars \
  --force-device-scale-factor=1 --default-background-color=00000000 \
  --window-size=1024,1024 --screenshot="$TMP/icon-1024.png" \
  "file://$PWD/assets/icon.svg" >/dev/null 2>&1

cp "$TMP/icon-1024.png" assets/icon-1024.png

ICONSET="$TMP/AppIcon.iconset"
mkdir -p "$ICONSET"
for size in 16 32 128 256 512; do
  sips -z "$size" "$size" "$TMP/icon-1024.png" --out "$ICONSET/icon_${size}x${size}.png" >/dev/null
  double=$((size * 2))
  sips -z "$double" "$double" "$TMP/icon-1024.png" --out "$ICONSET/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$ICONSET" -o assets/AppIcon.icns
echo "Wrote assets/AppIcon.icns and assets/icon-1024.png"
