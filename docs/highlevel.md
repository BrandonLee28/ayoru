# ani-cli-clone Foundation (Pre-TUI)

This document captures the current CLI foundation so the next phase can build a TUI on top of stable core modules instead of reworking business logic.

## Product Slice Implemented

Current flow (working):
1. `ani <query>`
2. Interactive title selection
3. Interactive episode selection
4. Automatic stream resolution (AllAnime-based)
5. Automatic player detection/launch (`mpv -> iina -> vlc`)
6. Playback fallback with bounded retries

## Architecture

Code is organized by responsibility so UI layers can be swapped:

- `src/main.rs`
- Runtime wiring for provider, picker, and player runtimes.

- `src/app.rs`
- App orchestration and runtime traits:
  - `ProviderRuntime`
  - `PickerRuntime`
  - `PlayerRuntime`
- `run_with(...)` is the main use-case pipeline.

- `src/core/`
- Pure business logic:
  - `models.rs` (`Title`, `Episode`, `StreamCandidate`)
  - `stream_ranker.rs` (reliability > language/sub > resolution)
  - `playback.rs` (6s timeout + one-attempt-per-provider fallback)

- `src/provider/`
- Provider integrations:
  - `allanime.rs` (GraphQL calls, encoded source decode, clock expansion)

- `src/player/`
- Player detection and launch:
  - `detect.rs`
  - `launch.rs` (adds required referrer flags per player)

- `src/cli/`
- UI interaction layer:
  - `picker.rs` (input state machine)
  - `interactive.rs` (terminal picker implementation)

- `src/errors.rs`
- User-facing error model (`AppError`).

## Important Runtime Contracts

### ProviderRuntime
- Must return raw titles and episodes.
- Must return stream candidates with provider identity preserved.
- Must degrade gracefully when specific source endpoints fail.

### PickerRuntime
- Owns user selection UX only.
- Returns selected indexes or cancellation.
- `run_with` remains UI-framework-agnostic.

### PlayerRuntime
- Handles executable detection and launch semantics.
- Playback-related HTTP headers/referrers are player-specific and live here.

## Current Policies Locked In

- Language preference: sub-first.
- Provider order: `wixmp`, `youtube`, `sharepoint`, `hianime`.
- Fallback policy: one attempt per provider per run.
- Playback timeout: 6 seconds.
- Failure behavior: clear errors, no debug UI.

## What Is Ready For TUI

The TUI can replace only the picker runtime while reusing everything else.

Recommended path:
1. Add a new `TuiPickerRuntime` under `src/cli/` (or `src/tui/`).
2. Keep `run_with(...)` unchanged.
3. Keep provider/player/core modules unchanged.
4. Add TUI-specific tests around picker behavior and cancellation.

## Risks / Follow-up Items

- Provider API format may drift; keep fixture tests updated.
- Some upstream source endpoints intermittently 500; current logic skips failed expansions.
- Playback-start detection is currently process-based; deeper player event verification can be added later if needed.

## Test Coverage Snapshot

Existing tests cover:
- args parsing
- ranking policy
- provider parsing/decode
- picker state transitions
- player detection/launch command specs
- fallback timeout policy
- app-level failure paths

This foundation is intentionally modular so a richer TUI can be layered without rewriting provider/business logic.
