#!/bin/sh
set -eu
[ "$(uname -s)" = Darwin ] || exit 0
TEMP=$(mktemp -d)
trap 'rm -rf "$TEMP"' EXIT HUP INT TERM
clang -fobjc-arc -framework AppKit -framework AVKit -framework AVFoundation -framework CoreMedia native/macos.m tests/native_macos.m -o "$TEMP/native-test"
if [ "$#" -gt 0 ]; then
    "$TEMP/native-test" "$1" "${2:-https://mkissa.to/}"
    exit 0
fi
if command -v ffmpeg >/dev/null 2>&1; then
    ffmpeg -hide_banner -loglevel error -f lavfi -i color=c=blue:s=320x180:r=30 -t 2 -c:v libx264 -pix_fmt yuv420p "$TEMP/test.mp4"
    "$TEMP/native-test" "$TEMP/test.mp4"
else
    "$TEMP/native-test"
    echo 'Video decode check skipped: install ffmpeg to generate the fixture.'
fi
