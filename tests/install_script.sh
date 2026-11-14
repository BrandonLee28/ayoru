#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
TEMP=$(mktemp -d)
trap 'rm -rf "$TEMP"' EXIT HUP INT TERM
mkdir -p "$TEMP/bin" "$TEMP/release/Ayoru.app/Contents/MacOS"
printf '#!/bin/sh\nexit 0\n' > "$TEMP/release/Ayoru.app/Contents/MacOS/ayoru"
chmod +x "$TEMP/release/Ayoru.app/Contents/MacOS/ayoru"
tar -czf "$TEMP/release.tar.gz" -C "$TEMP/release" Ayoru.app
cat > "$TEMP/bin/uname" <<'STUB'
#!/bin/sh
case "$1" in -s) echo "${TEST_OS:-Darwin}" ;; -m) echo arm64 ;; esac
STUB
cat > "$TEMP/bin/curl" <<'STUB'
#!/bin/sh
printf '%s\n' "$*" > "$TEST_TEMP/curl-args"
while [ "$#" -gt 0 ]; do
    if [ "$1" = -o ]; then shift; cp "$TEST_TEMP/release.tar.gz" "$1"; exit 0; fi
    shift
done
exit 1
STUB
cat > "$TEMP/bin/ditto" <<'STUB'
#!/bin/sh
mkdir -p "$2"
cp -R "$1/." "$2/"
STUB
chmod +x "$TEMP/bin/"*
TEST_TEMP="$TEMP" PATH="$TEMP/bin:$PATH" AYORU_INSTALL_DIR="$TEMP/Applications" sh "$ROOT/scripts/install.sh" > "$TEMP/result"
test -x "$TEMP/Applications/Ayoru.app/Contents/MacOS/ayoru"
grep -q 'alpha/ayoru-darwin-aarch64.tar.gz' "$TEMP/curl-args"
if TEST_OS=Linux PATH="$TEMP/bin:$PATH" sh "$ROOT/scripts/install.sh" > "$TEMP/error" 2>&1; then
    echo 'Installer unexpectedly accepted Linux.' >&2; exit 1
fi
grep -q 'macOS only' "$TEMP/error"
echo 'Desktop installer checks passed.'
