# TUI-Only Entrypoint Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

## Goal

Remove the separate CLI play mode so `ayoru` always launches the TUI. Delete the unused CLI interaction layer, simplify argument handling, and update tests and docs to reflect the product's TUI-only direction.

## Constraints

- Follow TDD: write a failing test first, verify it fails, then write minimal implementation.
- Do not keep `ayoru <query>` or `ayoru tui`.
- Keep the shared provider, playback, and player modules intact.
- Update public docs so they no longer describe a CLI mode.

## Steps

### 1. Define the new argument behavior in tests

Files:
- `tests/cli_args.rs`

Actions:
- Replace the existing mixed-mode parsing tests with TUI-only expectations.
- Add a passing case for `ayoru`.
- Add error cases for `ayoru frieren` and `ayoru tui`.

Run:

```bash
cargo test --test cli_args
```

Expected:
- FAIL because the current parser still accepts query mode and `tui`.

### 2. Implement the minimal parser change

Files:
- `src/args.rs`

Actions:
- Remove the `Command` enum and reduce `Args` to an empty marker or equivalent minimal type.
- Make `parse_from(...)` accept only the bare command invocation.
- Keep `--help` and `--version` behavior through Clap.
- Return a Clap error for any positional argument.

Run:

```bash
cargo test --test cli_args
```

Expected:
- PASS with the new TUI-only argument contract.

### 3. Remove the CLI entrypoint wiring

Files:
- `src/main.rs`
- `src/lib.rs`

Actions:
- Simplify `main` so it parses args for validation and always runs `ayoru::tui::run()`.
- Remove the exported `cli` module from `lib.rs`.

Run:

```bash
cargo test --test cli_args --test tui_runtime
```

Expected:
- PASS, proving the binary still boots the TUI path and parser changes did not break runtime compilation.

### 4. Delete the obsolete CLI interaction layer and tests

Files:
- `src/cli/mod.rs`
- `src/cli/interactive.rs`
- `src/cli/picker.rs`
- `tests/mvp_flow.rs`
- `tests/picker_navigation.rs`

Actions:
- Delete the old prompt-style picker implementation and tests that only exist for CLI mode.
- Keep shared playback/provider coverage in the remaining tests.

Run:

```bash
cargo test
```

Expected:
- PASS or expose any remaining references to the deleted CLI code.

### 5. Update product and developer documentation

Files:
- `README.md`
- `docs/highlevel.md`

Actions:
- Remove references to direct CLI playback and `ayoru tui`.
- Describe Ayoru as a TUI-only application launched with `ayoru`.
- Update repo layout notes to remove `src/cli/`.

Run:

```bash
rg -n "ayoru <query>|ayoru tui|direct CLI flow|src/cli" README.md docs/highlevel.md src tests
```

Expected:
- No matches that describe the removed CLI mode, except historical plan docs.

### 6. Final verification

Run:

```bash
cargo test
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
```

Expected:
- All commands exit successfully.
