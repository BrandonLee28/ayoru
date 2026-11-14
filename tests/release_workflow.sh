#!/bin/sh
set -eu
WORKFLOW=.github/workflows/release.yml
grep -q 'aarch64-apple-darwin' "$WORKFLOW"
grep -q 'x86_64-apple-darwin' "$WORKFLOW"
grep -q 'scripts/package-macos.sh' "$WORKFLOW"
grep -q 'dist/\*.dmg' "$WORKFLOW"
if grep -q 'unknown-linux-gnu' "$WORKFLOW"; then echo 'Desktop releases must target macOS first.' >&2; exit 1; fi
echo 'Desktop release workflow checks passed.'
