# Ayoru

`ayoru` is a terminal-first anime watcher built around a keyboard-driven TUI.

It lets you search a show, pick an episode, resolve a stream, and open playback in a local player without bouncing between browser tabs and ad-heavy sites.

## Why Use It

- bare `ayoru` opens the full-screen TUI
- full-screen TUI with search, favorites, history, and recently watched
- automatic stream ranking and playback fallback
- local player launch with `mpv`, `iina`, or `vlc`
- local-only saved state for the TUI

## Requirements

Before installing, make sure you have:

- Rust stable toolchain
- one supported player installed:
  - `mpv`
  - `iina`
  - `vlc`

## Install

### One-command install

```bash
curl -fsSL https://raw.githubusercontent.com/BrandonLee28/ayoru/main/scripts/install.sh | sh
```

What this does:

- installs `ayoru` to `~/.local/bin/ayoru`
- uses the rolling `alpha` GitHub Release built from `main`
- falls back to a source build if a release artifact is missing
- adds `~/.local/bin` to your shell `PATH` if needed

Verify the install:

```bash
command -v ayoru
ayoru --version
```

### Install from source with Cargo

```bash
git clone https://github.com/BrandonLee28/ayoru.git
cd ayoru
cargo install --path .
```

Cargo installs the binary to `~/.cargo/bin/ayoru`.

If `ayoru` is not found in a new terminal, add Cargo's bin directory to your `PATH`.

For `zsh` on macOS:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zprofile
source ~/.zprofile
```

## Quick Start

### Launch Ayoru

```bash
ayoru
```

The TUI includes:

- search
- favorites
- recently watched
- history
- keyboard-first navigation

## TUI Controls

- type to search when search is focused
- `/` focus search
- `Tab` move between `Media Browser`, `Favorites`, and `History`
- `h` / `l` move between tabs when search is not focused
- `j` / `k` or arrow keys move inside the active list
- `Enter` open a title or play an episode in the active flow
- `f` toggle favorite when search is not focused
- `d` remove the selected item from favorites or history
- `D` clear all history when the history tab is active
- `Esc` back out of detail or playback states
- `q` quit

## Troubleshooting

### `ayoru: command not found`

The install likely succeeded, but your shell has not picked up the bin directory yet.

If you used the one-command installer, open a new terminal or reload your shell profile:

```bash
source ~/.zprofile
```

Then check again:

```bash
command -v ayoru
```

### The installer falls back to building from source

That is expected when a matching `alpha` release artifact is unavailable for your platform. In that case the installer uses `cargo build --release`, so Rust and Cargo must be installed.

### Playback does not start

Make sure at least one supported player is installed:

- `mpv`
- `iina`
- `vlc`

## Local Data

The TUI stores local state at:

- `$XDG_STATE_HOME/ayoru/library.json`, or
- `~/.local/state/ayoru/library.json`

This file stores favorites, recently watched, and history in JSON.

## Current Scope

Ayoru currently focuses on:

- terminal-first playback
- one provider integration
- local player launch
- local-only saved state

It does not currently include:

- accounts
- sync
- recommendations
- collections
- a native desktop app

## Development

Useful commands:

```bash
cargo run -- --help
cargo test -q
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

## Repo Layout

```text
src/
  app.rs          provider and player runtime traits
  args.rs         command validation
  core/           models, playback policy, stream ranking
  player/         player detection and launch
  provider/       provider integrations
  tui/            TUI shell, renderer, runtime, persistence
tests/            integration and behavior tests
docs/             design and implementation docs
```

## Release Asset Naming

The installer expects release artifacts to use these names:

- `ayoru-darwin-aarch64.tar.gz`
- `ayoru-darwin-x86_64.tar.gz`
- `ayoru-linux-aarch64.tar.gz`
- `ayoru-linux-x86_64.tar.gz`

## Maintainer Release Flow

Pushes to `main` refresh the rolling `alpha` release used by the installer by default.

Push a version tag when you want a versioned release:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The GitHub Actions workflow publishes both:

- a rolling `alpha` prerelease from `main`
- versioned releases for `v0.1.x` tags

## Notes

- Favorites, history, and recently watched are local-only by design.
- Design and implementation notes live under `docs/plans/`.
