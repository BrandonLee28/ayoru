# History Play vs Open Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

## Goal

Make the history tab treat a history row as a saved watch entry instead of just a title. `Enter` should immediately play the saved episode, while a separate action should open the show into the episode picker for browsing.

## Constraints

- Follow TDD: tests first, verify failure, then minimal implementation.
- Keep favorites behavior unchanged.
- History rows must still support delete and clear-history actions.
- Footer hints must reflect the new history-specific controls.

## Steps

### 1. Define the new history input contract

Files:
- `tests/tui_input.rs`
- `tests/tui_render.rs`

Actions:
- Add expectations that `Enter` on history means direct play.
- Add expectations that `o` on history opens the show.
- Update footer render expectations to show both actions.

Run:

```bash
cargo test --test tui_input --test tui_render
```

Expected:
- FAIL because history currently only supports opening the selected title.

### 2. Define history playback/open behavior in controller tests

Files:
- `tests/tui_episode_flow.rs`
- `tests/tui_playback_flow.rs`

Actions:
- Add a test proving direct play from history resolves the saved episode immediately.
- Add a test proving `o` from history still opens the show’s episode list.

Run:

```bash
cargo test --test tui_episode_flow --test tui_playback_flow
```

Expected:
- FAIL because the controller does not distinguish between “play entry” and “open show.”

### 3. Implement the new history actions

Files:
- `src/tui/action.rs`
- `src/tui/state.rs`
- `src/tui/controller.rs`
- `src/tui/runtime.rs`
- `src/tui/ui.rs`

Actions:
- Add a dedicated history-play action/effect.
- Map `Enter` on the history tab to direct playback of the saved watch episode.
- Map `o` on the history tab to opening the show into the episode picker.
- Keep playback success/failure messaging coherent when launched from history.
- Update footer hints and row metadata to reflect the new controls.

Run:

```bash
cargo test --test tui_input --test tui_render --test tui_episode_flow --test tui_playback_flow
```

Expected:
- PASS with the new interaction model.

### 4. Final verification

Run:

```bash
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

Expected:
- All commands exit successfully.
