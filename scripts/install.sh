#!/bin/sh
set -eu

REPO_SLUG=${AYORU_REPO_SLUG:-BrandonLee28/ayoru}
VERSION=${AYORU_RELEASE_VERSION:-alpha}
INSTALL_DIR=${AYORU_INSTALL_DIR:-"$HOME/Applications"}
MODE=release
while [ "$#" -gt 0 ]; do
    case "$1" in
        --from-source) MODE=source ;;
        --release-version) shift; VERSION=${1:?Missing release version} ;;
        --install-dir) shift; INSTALL_DIR=${1:?Missing install directory} ;;
        *) echo "Unknown argument: $1" >&2; exit 1 ;;
    esac
    shift
done
[ "$(uname -s)" = Darwin ] || { echo 'ayoru Desktop currently supports macOS only.' >&2; exit 1; }
case "$(uname -m)" in arm64|aarch64) ARCH=aarch64 ;; x86_64) ARCH=x86_64 ;; *) echo 'Unsupported architecture.' >&2; exit 1 ;; esac
TEMP=$(mktemp -d)
trap 'rm -rf "$TEMP"' EXIT HUP INT TERM
if [ "$MODE" = release ]; then
    URL="https://github.com/$REPO_SLUG/releases/download/$VERSION/ayoru-darwin-$ARCH.tar.gz"
    curl -fL "$URL" -o "$TEMP/ayoru.tar.gz"
    tar -xzf "$TEMP/ayoru.tar.gz" -C "$TEMP"
    APP="$TEMP/ayoru.app"
    [ -x "$APP/Contents/MacOS/ayoru" ] || { echo 'Release archive has no desktop app. Use --from-source until the desktop release is published.' >&2; exit 1; }
else
    if [ -f Cargo.toml ] && [ -f scripts/package-macos.sh ]; then
        REPO=$(pwd)
    else
        git clone --depth 1 "https://github.com/$REPO_SLUG.git" "$TEMP/repo"
        REPO="$TEMP/repo"
    fi
    (cd "$REPO" && cargo build --release --locked)
    AYORU_SKIP_DMG=1 sh "$REPO/scripts/package-macos.sh" "$REPO/target/release/ayoru" "$TEMP/package" "$ARCH"
    APP="$TEMP/package/ayoru.app"
fi
mkdir -p "$INSTALL_DIR"
ditto "$APP" "$INSTALL_DIR/ayoru.app"
printf 'Installed %s/ayoru.app\nLaunch with: open "%s/ayoru.app"\n' "$INSTALL_DIR" "$INSTALL_DIR"
