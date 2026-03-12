# Ayoru High-Level Architecture

This document describes the current TUI-first architecture for Ayoru.

## Product Slice Implemented

Current flow (working):
1. Launch `ayoru`
2. Search for a title inside the TUI
3. Pick an episode
4. Resolve streams from AllAnime
5. Detect and launch a local player (`mpv -> iina -> vlc`)
6. Retry playback across ranked stream providers when needed

## Architecture

Code is organized so the TUI owns interaction while shared modules handle provider access and playback:

- `src/main.rs`
- Validates the bare `ayoru` invocation and launches the TUI runtime.

- `src/tui/`
- TUI shell, controller, state machine, renderer, storage, and runtime loop.

- `src/app.rs`
- Shared runtime traits and system player implementation:
  - `ProviderRuntime`
  - `PlayerRuntime`
  - `SystemPlayerRuntime`

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

- `src/errors.rs`
- User-facing error model (`AppError`).

## Important Runtime Contracts

### ProviderRuntime
- Must return raw titles and episodes.
- Must return stream candidates with provider identity preserved.
- Must degrade gracefully when specific source endpoints fail.

### PlayerRuntime
- Handles executable detection and launch semantics.
- Playback-related HTTP headers and referrers are player-specific and live here.

## Current Policies Locked In

- Language preference: sub-first.
- Provider order: `wixmp`, `youtube`, `sharepoint`, `hianime`.
- Fallback policy: one attempt per provider per run.
- Playback timeout: 6 seconds.
- Failure behavior: clear errors, no debug UI.

## Risks / Follow-up Items

- Provider API format may drift; keep fixture tests updated.
- Some upstream source endpoints intermittently 500; current logic skips failed expansions.
- Playback-start detection is currently process-based; deeper player event verification can be added later if needed.

## Test Coverage Snapshot

Existing tests cover:
- args parsing for the TUI-only entrypoint
- ranking policy
- provider parsing and decode
- player detection and launch command specs
- TUI controller, state, rendering, storage, and runtime behavior
- fallback timeout policy

This foundation is intentionally centered on a single interaction model so the product surface stays narrow and coherent.
