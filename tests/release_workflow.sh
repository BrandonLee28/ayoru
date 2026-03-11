#!/bin/sh

set -eu

WORKFLOW=.github/workflows/release.yml

assert_contains() {
    file=$1
    needle=$2

    if ! grep -Fq -- "$needle" "$file"; then
        printf 'expected %s to contain %s\n' "$file" "$needle" >&2
        exit 1
    fi
}

assert_count() {
    file=$1
    needle=$2
    expected=$3
    actual=$(grep -F -- "$needle" "$file" | wc -l | tr -d ' ')

    if [ "$actual" != "$expected" ]; then
        printf 'expected %s occurrences of %s in %s, got %s\n' "$expected" "$needle" "$file" "$actual" >&2
        exit 1
    fi
}

assert_contains "$WORKFLOW" "tags:"
assert_contains "$WORKFLOW" "- 'v*'"
assert_contains "$WORKFLOW" "aarch64-apple-darwin"
assert_contains "$WORKFLOW" "x86_64-apple-darwin"
assert_contains "$WORKFLOW" "aarch64-unknown-linux-gnu"
assert_contains "$WORKFLOW" "x86_64-unknown-linux-gnu"
assert_contains "$WORKFLOW" 'rustup target add ${{ matrix.target }}'
assert_contains "$WORKFLOW" 'cargo build --release --target ${{ matrix.target }}'
assert_contains "$WORKFLOW" "tar -czf"
assert_contains "$WORKFLOW" "actions/upload-artifact@v4"
assert_contains "$WORKFLOW" "publish:"
assert_contains "$WORKFLOW" "needs: build"
assert_contains "$WORKFLOW" "actions/download-artifact@v4"
assert_contains "$WORKFLOW" "softprops/action-gh-release@v2"
assert_contains "$WORKFLOW" "ayoru-darwin-aarch64.tar.gz"
assert_contains "$WORKFLOW" "ayoru-darwin-x86_64.tar.gz"
assert_contains "$WORKFLOW" "ayoru-linux-aarch64.tar.gz"
assert_contains "$WORKFLOW" "ayoru-linux-x86_64.tar.gz"
assert_count "$WORKFLOW" "ayoru-darwin-aarch64.tar.gz" "1"
assert_count "$WORKFLOW" "ayoru-darwin-x86_64.tar.gz" "1"
assert_count "$WORKFLOW" "ayoru-linux-aarch64.tar.gz" "1"
assert_count "$WORKFLOW" "ayoru-linux-x86_64.tar.gz" "1"

printf 'release workflow contract tests passed\n'
