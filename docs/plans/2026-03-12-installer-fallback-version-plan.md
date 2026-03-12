# Installer Fallback And Version Alignment Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make installer release-download failures fall back cleanly without misleading extraction errors, and align the built binary version with the `v0.1.1` release tag.

**Architecture:** Extend the shell regression test to simulate a failed release asset download while keeping source fallback available from the local repo. Update the installer with explicit failure checks around release download and extraction, then bump crate metadata so source builds and `--version` output match the published tag.

**Tech Stack:** POSIX shell, Cargo, shell test script

---

### Task 1: Add regression coverage for failed release installs

**Files:**
- Modify: `tests/install_script.sh`
- Test: `tests/install_script.sh`

**Step 1: Write the failing test**

Add a shell scenario that:
- stubs `curl` to fail for `releases/download/*`
- stubs `cargo` to produce a local `target/release/ayoru` binary reporting `0.1.1`
- asserts installer output includes the fallback message
- asserts installer output does not include `tar:` or `cp:`
- asserts the installed binary reports `ayoru 0.1.1`

**Step 2: Run test to verify it fails**

Run: `sh tests/install_script.sh`
Expected: FAIL because the installer still runs `tar` and `cp` after a failed download, and the fallback build still reports `0.1.0`.

### Task 2: Fix installer release fallback behavior

**Files:**
- Modify: `scripts/install.sh`
- Test: `tests/install_script.sh`

**Step 1: Write minimal implementation**

Update `install_from_release` so each step explicitly returns on failure:
- release download
- tar extraction
- binary presence / install copy

Keep the fallback in `main`, but make its message explicit about the release version that failed.

**Step 2: Run test to verify it passes**

Run: `sh tests/install_script.sh`
Expected: PASS for the new fallback regression and existing helper checks.

### Task 3: Align crate version with release tag

**Files:**
- Modify: `Cargo.toml`
- Modify: `Cargo.lock`
- Test: `cargo check`

**Step 1: Write minimal implementation**

Bump the root package version from `0.1.0` to `0.1.1` in `Cargo.toml` and the root `ayoru` package entry in `Cargo.lock`.

**Step 2: Run verification**

Run: `cargo check`
Expected: PASS

Run: `sh tests/install_script.sh`
Expected: PASS
