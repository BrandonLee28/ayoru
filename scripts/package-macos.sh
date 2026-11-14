#!/bin/sh
set -eu

# Usage: package-macos.sh [binary] [output-directory] [aarch64|x86_64]
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
BINARY=${1:-"$ROOT/target/release/ayoru"}
OUTPUT=${2:-"$ROOT/dist"}
ARCH=${3:-$(uname -m)}
case "$ARCH" in arm64) ARCH=aarch64 ;; aarch64|x86_64) ;; *) echo "Unsupported architecture: $ARCH" >&2; exit 1 ;; esac
[ "$(uname -s)" = Darwin ] || { echo 'Packaging requires macOS.' >&2; exit 1; }
[ -f "$BINARY" ] || { echo "Build the release binary first: $BINARY" >&2; exit 1; }
mkdir -p "$OUTPUT"
OUTPUT=$(CDPATH= cd -- "$OUTPUT" && pwd)
STAGING=$(mktemp -d)
trap 'rm -rf "$STAGING"' EXIT HUP INT TERM
APP="$STAGING/ayoru.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BINARY" "$APP/Contents/MacOS/ayoru"
chmod 755 "$APP/Contents/MacOS/ayoru"
cp "$ROOT/THIRD_PARTY_NOTICES.md" "$APP/Contents/Resources/THIRD_PARTY_NOTICES.md"
VERSION=$(sed -n 's/^version = "\([^"]*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleName</key><string>ayoru</string>
<key>CFBundleDisplayName</key><string>ayoru</string>
<key>CFBundleIdentifier</key><string>com.brandonlee.ayoru</string>
<key>CFBundleExecutable</key><string>ayoru</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>$VERSION</string>
<key>CFBundleVersion</key><string>$VERSION</string>
<key>CFBundleIconFile</key><string>ayoru</string>
<key>LSMinimumSystemVersion</key><string>12.0</string>
<key>NSHighResolutionCapable</key><true/>
<key>NSPrincipalClass</key><string>NSApplication</string>
<key>NSAppTransportSecurity</key><dict><key>NSAllowsArbitraryLoadsForMedia</key><true/></dict>
</dict></plist>
PLIST
# Package the same artwork embedded for the running Dock icon.
cp "$ROOT/assets/app-icon.png" "$STAGING/icon.png"
mkdir -p "$STAGING/ayoru.iconset"
for SIZE in 16 32 128 256 512; do
    sips -z "$SIZE" "$SIZE" "$STAGING/icon.png" --out "$STAGING/ayoru.iconset/icon_${SIZE}x${SIZE}.png" >/dev/null
    DOUBLE=$((SIZE * 2))
    sips -z "$DOUBLE" "$DOUBLE" "$STAGING/icon.png" --out "$STAGING/ayoru.iconset/icon_${SIZE}x${SIZE}@2x.png" >/dev/null
done
iconutil -c icns "$STAGING/ayoru.iconset" -o "$APP/Contents/Resources/ayoru.icns"
plutil -lint "$APP/Contents/Info.plist"
codesign --force --options runtime --sign "${AYORU_SIGNING_IDENTITY:--}" "$APP"
codesign --verify --strict "$APP"
# ditto preserves the bundle structure and signature.
if [ -e "$OUTPUT/ayoru.app" ]; then rm -rf "$OUTPUT/ayoru.app"; fi
ditto "$APP" "$OUTPUT/ayoru.app"
tar -czf "$OUTPUT/ayoru-darwin-$ARCH.tar.gz" -C "$STAGING" ayoru.app
if [ "${AYORU_SKIP_DMG:-0}" != 1 ]; then
    # Keep build intermediates out of the disk image.
    DMG_ROOT="$STAGING/image"
    mkdir -p "$DMG_ROOT"
    ditto "$APP" "$DMG_ROOT/ayoru.app"
    ln -s /Applications "$DMG_ROOT/Applications"
    hdiutil create -volname ayoru -srcfolder "$DMG_ROOT" -ov -format UDZO "$OUTPUT/ayoru-darwin-$ARCH.dmg"
    if [ -n "${AYORU_NOTARY_PROFILE:-}" ]; then
        xcrun notarytool submit "$OUTPUT/ayoru-darwin-$ARCH.dmg" --keychain-profile "$AYORU_NOTARY_PROFILE" --wait
        xcrun stapler staple "$OUTPUT/ayoru-darwin-$ARCH.dmg"
    fi
fi
printf 'Built %s/ayoru.app\n' "$OUTPUT"
