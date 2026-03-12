# Modern TUI Redesign Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

## Goal

Redesign the Ayoru terminal UI so it feels like a modern media browser instead of a boxed utility dashboard. Keep the current color palette, but improve layout hierarchy, selection treatment, spacing, and the visual relationship between the main content pane and the supporting rail.

## Constraints

- Follow TDD: write the render tests first, verify they fail, then implement the minimal layout/style changes.
- Keep the existing TUI state model and interaction model unless a render requirement forces a small supporting change.
- Preserve current search, episode, favorites, recent, and history features.
- Keep the palette recognizable; the redesign should come from structure and typography, not a wholesale color change.

## Steps

### 1. Define the new render contract in tests

Files:
- `tests/tui_render.rs`
- `tests/tui_shell_render.rs`

Actions:
- Update render expectations to reflect a modern media-browser shell.
- Assert the presence of a compact top bar, stronger browsing copy, and shelf-style supporting sections.
- Assert the selected item uses a full-row highlight treatment instead of relying only on the `>` marker.

Run:

```bash
cargo test --test tui_render --test tui_shell_render
```

Expected:
- FAIL because the current render still uses the old boxed dashboard structure and copy.

### 2. Implement the redesigned layout

Files:
- `src/tui/ui.rs`

Actions:
- Replace the old “equal boxes” composition with:
  - a slim top bar
  - a dominant primary browsing pane
  - a quieter supporting rail
- Update section titles, spacing, and helper copy to feel like a media browser.
- Redesign selected rows to read as active media tiles/strips using background fill and accent styling.
- Reduce border noise; use spacing and surface contrast for hierarchy.

Run:

```bash
cargo test --test tui_render --test tui_shell_render
```

Expected:
- PASS with the new render structure.

### 3. Verify the redesign does not break the rest of the TUI

Files:
- `tests/tui_state.rs`
- `tests/tui_input.rs`
- `tests/tui_runtime.rs`
- `tests/tui_search_flow.rs`
- `tests/tui_episode_flow.rs`
- `tests/tui_playback_flow.rs`

Actions:
- Run the existing TUI behavior tests unchanged to confirm the redesign stayed presentational.

Run:

```bash
cargo test --test tui_state --test tui_input --test tui_runtime --test tui_search_flow --test tui_episode_flow --test tui_playback_flow
```

Expected:
- PASS, proving the redesign did not alter controller behavior.

### 4. Update docs if the visible shell language changed materially

Files:
- `README.md`

Actions:
- Adjust any user-facing description that now better fits the redesigned browsing shell.

Run:

```bash
rg -n "dashboard|boxed|utility" README.md src/tui/ui.rs tests/tui_render.rs tests/tui_shell_render.rs
```

Expected:
- No stale descriptions that conflict with the redesigned shell.

### 5. Final verification

Run:

```bash
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

Expected:
- All commands exit successfully.
