# Tabbed Library Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

## Goal

Replace the passive right rail with three first-class tabs: `Media Browser`, `Favorites`, and `History`. Each tab should be directly selectable with `Tab`, render as the primary interactive list, open the selected title with `Enter`, and expose context-aware footer hints. Add editing operations so favorites/history items can be removed and history can be cleared entirely.

## Constraints

- Follow TDD: write failing tests first and verify they fail before implementation.
- Keep `Media Browser` search flow intact.
- `Favorites` and `History` must open episodes with `Enter`, not just act as static shelves.
- Keep favorite toggling on `f`.
- Add a delete action for selected favorites/history items and a clear-history action.

## Steps

### 1. Define the tabbed input behavior in tests

Files:
- `tests/tui_input.rs`
- `tests/tui_shell_state.rs`

Actions:
- Add expectations for `Tab` cycling between the three top-level tabs.
- Add expectations for delete and clear-history keybindings in the relevant tabs.
- Assert tab-specific movement still maps to list navigation.

Run:

```bash
cargo test --test tui_input --test tui_shell_state
```

Expected:
- FAIL because the current state/input model still uses the old panel-based shell.

### 2. Define controller behavior for favorites/history editing

Files:
- `tests/tui_shell_controller.rs`
- `tests/tui_episode_flow.rs`

Actions:
- Add tests proving favorites/history tabs open selected titles into episode flow with `Enter`.
- Add tests proving delete removes the selected favorite/history item and persists it.
- Add a test proving clear-history removes all history entries and persists the empty history.

Run:

```bash
cargo test --test tui_shell_controller --test tui_episode_flow
```

Expected:
- FAIL because the controller has no tab-aware removal or clear-history actions yet.

### 3. Define the new render contract

Files:
- `tests/tui_render.rs`
- `tests/tui_shell_render.rs`

Actions:
- Update render tests to expect a tab bar for `Media Browser`, `Favorites`, and `History`.
- Assert the footer hints change per active tab.
- Assert `Up Next` no longer appears.

Run:

```bash
cargo test --test tui_render --test tui_shell_render
```

Expected:
- FAIL because the current render still uses the shelf/rail model.

### 4. Implement the new tab and edit model

Files:
- `src/tui/state.rs`
- `src/tui/action.rs`
- `src/tui/controller.rs`
- `src/tui/runtime.rs`
- `src/tui/ui.rs`

Actions:
- Replace the panel-oriented shell state with a top-level tab model.
- Add actions/effects for tab switching, removing the selected item, and clearing history.
- Make `Enter` load episodes from any tab using the selected title in that tab.
- Update input mapping and footer hints to be tab-aware.
- Remove obsolete rail-specific logic.

Run:

```bash
cargo test --test tui_input --test tui_shell_state --test tui_shell_controller --test tui_episode_flow --test tui_render --test tui_shell_render
```

Expected:
- PASS with the new interaction model.

### 5. Verify broader TUI behavior still holds

Files:
- `tests/tui_search_flow.rs`
- `tests/tui_playback_flow.rs`
- `tests/tui_runtime.rs`
- `tests/tui_state.rs`
- `tests/tui_storage.rs`

Actions:
- Run the unaffected TUI behavior suites unchanged.

Run:

```bash
cargo test --test tui_search_flow --test tui_playback_flow --test tui_runtime --test tui_state --test tui_storage
```

Expected:
- PASS, proving the tabbed redesign did not regress search, playback, runtime, or storage behavior.

### 6. Final verification

Run:

```bash
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

Expected:
- All commands exit successfully.
