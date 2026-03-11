#!/bin/sh

set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)

. "$REPO_ROOT/scripts/install.sh"

assert_eq() {
    actual=$1
    expected=$2

    if [ "$actual" != "$expected" ]; then
        printf 'expected %s, got %s\n' "$expected" "$actual" >&2
        exit 1
    fi
}

assert_eq "$(normalize_arch arm64)" "aarch64"
assert_eq "$(normalize_arch x86_64)" "x86_64"
assert_eq "$(asset_name darwin arm64)" "ayoru-darwin-aarch64.tar.gz"
assert_eq "$(asset_name linux x86_64)" "ayoru-linux-x86_64.tar.gz"
assert_eq \
    "$(release_asset_url v0.1.0 darwin arm64)" \
    "https://github.com/BrandonLee28/ayoru/releases/download/v0.1.0/ayoru-darwin-aarch64.tar.gz"
assert_eq "$(shell_profile_for zsh /tmp/test-home)" "/tmp/test-home/.zprofile"
assert_eq "$(shell_profile_for bash /tmp/test-home)" "/tmp/test-home/.bash_profile"
assert_eq "$(path_export_line /tmp/bin)" 'export PATH="/tmp/bin:$PATH"'

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM
profile="$tmpdir/.zprofile"

append_path_if_missing "$profile" "/tmp/bin"
append_path_if_missing "$profile" "/tmp/bin"

assert_eq "$(wc -l < "$profile" | tr -d ' ')" "1"
assert_eq "$(cat "$profile")" 'export PATH="/tmp/bin:$PATH"'

assert_eq "$(require_supported_target Darwin arm64)" "darwin:aarch64"

unsupported_output=$(
    (
        require_supported_target FreeBSD amd64
    ) 2>&1 || true
)

case "$unsupported_output" in
    *"unsupported platform: FreeBSD amd64"*)
        ;;
    *)
        printf 'unexpected unsupported target output: %s\n' "$unsupported_output" >&2
        exit 1
        ;;
esac

local_repo=$(mktemp -d)
mkdir -p "$local_repo/.git"
touch "$local_repo/Cargo.toml"
assert_eq "$(repo_source_dir "$local_repo")" "$local_repo"
worktree_repo=$(mktemp -d)
touch "$worktree_repo/.git"
touch "$worktree_repo/Cargo.toml"
assert_eq "$(repo_source_dir "$worktree_repo")" "$worktree_repo"
assert_eq "$(script_dir_for_path ./scripts/install.sh /worktree)" "/worktree/scripts"
assert_eq "$(repo_clone_url)" "https://github.com/BrandonLee28/ayoru.git"
rm -rf "$local_repo"
rm -rf "$worktree_repo"

printf 'install helper tests passed\n'
