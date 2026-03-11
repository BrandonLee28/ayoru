# ani TUI Design

**Date:** 2026-03-11

## Goal

Build a standalone terminal UI launched with `ani tui` that feels keyboard-first and app-like, while leaving the current CLI behavior unchanged.

## Non-Goals

- Replacing or rewriting the existing prompt-style CLI flow.
- Forcing the TUI through the current `PickerRuntime` abstraction.
- Adding persistent history, library management, or a streaming activity log in v1.
- Refactoring the full application layer before proving the TUI shape.

## Product Direction

The TUI is a separate frontend built from scratch. It should reuse the existing provider, ranking, playback, and player-launch logic where that reuse is natural, but it must not alter the current `ani <query>` path.

The initial experience is search-first:

1. User runs `ani tui`.
2. Focus starts in a search-oriented view.
3. User types a query and submits search.
4. User selects a title.
5. User selects an episode.
6. The app resolves streams and launches playback.

The user experience should feel keyboard-centric in the same spirit as Codex: direct command feel, fast navigation, predictable modes, and minimal pointer-oriented chrome.

## Architecture

### Existing CLI

The current CLI path remains unchanged. `src/main.rs`, `run_with(...)`, and the current interactive picker continue to serve `ani <query>`.

### New TUI Frontend

Add a separate TUI module tree, preferably under `src/tui/`, with clear boundaries:

- `app` or `state`: authoritative app state and reducer-like transitions
- `actions` / `events`: user input and async completion messages
- `ui` / `render`: `ratatui` widget composition and layout
- `runtime`: terminal bootstrap, event loop, async task coordination

This frontend should launch from a new `ani tui` command path and own its full-screen lifecycle independently.

### Reuse Boundaries

Reuse existing modules where they already model the business logic well:

- provider search/episode/stream resolution
- stream ranking
- player detection
- player launch
- shared error types where practical

Do not route the TUI through `PickerRuntime`. That trait is intentionally narrow and built for prompt-style selection, not full-screen search, modal transitions, or background loading.

## Screen Model

The TUI should act like a single application with distinct modes rather than a sequence of isolated prompts.

### Core Modes

- `Search`
  - Query input
  - Search result list
  - Inline hints and loading state
- `Episodes`
  - Selected title context
  - Episode list
  - Back navigation to the preserved search state
- `Launching`
  - Focused transient state while streams resolve and playback starts
- `Error`
  - Recoverable inline or panel-based error presentation

### Layout

The initial layout should be shell-shaped but intentionally restrained:

- top bar: current mode, query, loading/status summary
- main pane: search results, episode list, or transient launch/error content
- bottom status bar: key hints and short feedback

V1 should not include persistent sidebars, activity logs, or multi-pane workspace complexity beyond what the interaction model needs.

## Interaction Model

The interface should prioritize fast keyboard use:

- immediate typing or `/` focuses search
- `j/k` and arrow keys move selection
- `Enter` advances into title selection, episode selection, or playback
- `Esc` navigates back one level
- `q` quits from stable screens

The interaction model matters more than visual mimicry. The goal is a clean command-driven feel, not a literal clone of another interface.

## Data Flow

The TUI owns its own state machine and dispatch loop.

### Inputs

User input becomes app actions such as:

- `UpdateQuery`
- `SubmitSearch`
- `MoveSelectionUp`
- `MoveSelectionDown`
- `OpenEpisodes`
- `PlayEpisode`
- `Back`
- `Quit`

### Async Work

All blocking work must run outside the render path:

- title search
- episode fetch
- stream fetch
- stream ranking
- player detection
- player launch

Async completions should return messages/events that update state and trigger redraws. Rendering should stay deterministic and non-blocking.

### Integration Strategy

The TUI should not call `run_with(...)` directly because that function assumes a pre-supplied query and prompt-oriented selection. The TUI needs fine-grained control over the search, title, episode, and launch stages.

If needed, small shared service helpers can be extracted later from the current orchestration path. That extraction should be driven by concrete duplication, not speculative cleanup.

## Error Handling

Recoverable errors should remain inside the TUI:

- provider search failures show an inline/panel error with retry
- episode/stream resolution failures return the user to a usable prior state
- playback launch failures surface clearly and preserve navigation context

Hard exits should be reserved for unrecoverable terminal setup or teardown failures.

## State Persistence

V1 should keep state in memory only.

When the user navigates back, preserve:

- current query text
- current search results if still valid
- current selected index where practical

Skip persistence, history, and library concepts until the core TUI workflow is proven.

## Testing Strategy

Implementation should follow strict TDD:

1. write a failing test
2. verify it fails for the expected reason
3. write the minimum code to pass
4. verify green
5. refactor without changing behavior

Testing emphasis:

- unit tests for the TUI state machine and reducer-style transitions
- targeted integration tests for async flows and screen transitions
- limited rendering tests only where layout or text output is worth locking down
- preserve all existing CLI tests as regression coverage proving the old path still works

### Key TUI Scenarios

- submit search and load results
- choose a title and load episodes
- navigate back while preserving query and selection
- handle recoverable provider errors
- recover from playback failure without crashing the TUI

## Delivery Strategy

Prefer a standalone `src/tui/` implementation behind `ani tui` with minimal intrusion into the existing CLI path. Reuse the core business logic aggressively, but keep the TUI UX and lifecycle independent.
