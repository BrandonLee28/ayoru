# Single-Session Playback Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

## Goal

Enforce strict single-session playback for supported players by preferring `IINA`, falling back to `mpv`, and reusing a single managed player session instead of launching uncontrolled new tabs/windows. Replace playback in the existing session whenever possible.

## Constraints

- Follow TDD: tests first, verify failure, then minimal implementation.
- Preferred player order becomes `IINA -> mpv`.
- `vlc` is removed from supported-player detection for this strict mode.
- Both `IINA` and `mpv` should be driven through a shared mpv-style IPC strategy.
- If a managed session exists, load the new URL with `replace` semantics instead of spawning a second playback session.

## Steps

### 1. Define the new player preference contract

Files:
- `tests/player_detection.rs`

Actions:
- Update detection expectations so `IINA` is preferred over `mpv`.
- Update the “no player found” guidance to mention only `iina` and `mpv`.

Run:

```bash
cargo test --test player_detection
```

Expected:
- FAIL because the current detection still prefers `mpv` and mentions `vlc`.

### 2. Define the new launch spec and IPC contract

Files:
- `tests/player_launch.rs`

Actions:
- Add expectations that both `IINA` and `mpv` launch with a stable IPC socket path.
- Add expectations that `IINA` uses `--keep-running`.
- Add a test for the IPC “replace current playback” payload using a temporary Unix socket listener.

Run:

```bash
cargo test --test player_launch
```

Expected:
- FAIL because the current launcher only shells out with direct URL args and has no IPC behavior.

### 3. Implement strict single-session playback

Files:
- `src/player/detect.rs`
- `src/player/launch.rs`
- `src/app.rs`

Actions:
- Change supported-player order to `IINA -> mpv`.
- Remove `vlc` from the strict-mode support list.
- Add a stable session socket path helper for each supported player.
- On playback:
  - if the session socket is live, send `loadfile ... replace` and related property updates
  - otherwise spawn the player with the IPC socket configured
- Ensure `IINA` launch includes `--keep-running` and mpv-compatible IPC options.

Run:

```bash
cargo test --test player_detection --test player_launch
```

Expected:
- PASS with the new launch and reuse behavior.

### 4. Verify playback flows still compile and behave

Files:
- `tests/tui_playback_flow.rs`
- `tests/playback_fallback.rs`
- `tests/player_detection.rs`

Actions:
- Run the existing playback-related behavior tests unchanged.

Run:

```bash
cargo test --test tui_playback_flow --test playback_fallback --test player_detection
```

Expected:
- PASS, proving the new player session management does not break playback orchestration.

### 5. Final verification

Run:

```bash
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

Expected:
- All commands exit successfully.
