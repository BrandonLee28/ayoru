# Ayoru Installer Design

**Date:** 2026-03-11

## Goal

Provide a first-class one-command install path for Ayoru that feels like a normal CLI tool, without requiring users to understand Cargo or Rust toolchain setup.

## Public Install Entry Point

The documented installer command will be:

```bash
curl -fsSL https://raw.githubusercontent.com/BrandonLee28/ayoru/main/scripts/install.sh | sh
```

The URL is derived from the repository remote:

- `https://github.com/BrandonLee28/ayoru.git`

## Product Direction

Cargo remains the developer install path, but it should no longer be the primary user-facing install story.

The primary install UX should:

- work as a single shell command
- place `ayoru` in a standard user bin directory
- make the binary available in future terminal sessions
- explain any shell changes clearly
- avoid surprising destructive behavior

## Recommended Approach

Use a hybrid installer script:

1. Prefer downloading a prebuilt release artifact from GitHub Releases.
2. Fall back to building from source when a matching release artifact does not exist.

This gives the best long-term UX while still working immediately in the repository today.

## Installer Behavior

The installer will:

- live at `scripts/install.sh`
- support being run via `curl | sh`
- support being run locally from a git checkout
- detect OS and architecture
- choose a release artifact name from that target
- install `ayoru` into `~/.local/bin` by default
- create the install directory if needed
- only modify shell startup files when the install dir is not already on `PATH`
- verify the installed binary with `command -v ayoru` and `ayoru --version`

## Release Path

When a tagged GitHub release contains a matching asset, the installer should:

- compute the release asset URL
- download the tarball
- extract the `ayoru` binary
- install it into `~/.local/bin/ayoru`

Expected release asset naming:

- `ayoru-darwin-aarch64.tar.gz`
- `ayoru-darwin-x86_64.tar.gz`
- `ayoru-linux-x86_64.tar.gz`
- `ayoru-linux-aarch64.tar.gz`

## Source Fallback

If a release asset is unavailable, the installer should fall back to a source build.

Fallback behavior:

- require `cargo`
- if running inside a local repo, build from that checkout
- otherwise clone the repo into a temp directory and run `cargo install --path . --root <temp-root>`
- copy the built `ayoru` binary into `~/.local/bin`

This avoids forcing users to understand Cargo's install root or PATH conventions.

## PATH Handling

The installer should treat `~/.local/bin` as the install target and ensure it is visible to new shells.

For shell startup updates:

- prefer `~/.zprofile` for `zsh`
- prefer `~/.bash_profile` or `~/.profile` for `bash`
- append only the single export line needed
- avoid duplicate entries

The installer should print exactly what profile file it changed.

## Error Handling

The installer should fail with clear messages when:

- the platform is unsupported
- neither `curl` nor `wget` is available for release downloads
- release download fails and source fallback is impossible
- `cargo` is required but missing
- the install step cannot write to the target directory

## Verification

Verification should cover:

- target triple detection
- release asset URL generation
- install directory and profile file selection
- PATH mutation idempotence
- local source-build success path

## Repo Changes

The implementation should include:

- `scripts/install.sh`
- a small test around deterministic installer helpers
- README updates that make the installer primary
- a note documenting expected release asset names so future release automation stays aligned
