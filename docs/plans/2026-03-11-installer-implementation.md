# Ayoru Installer Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a one-command installer that installs `ayoru` into a standard user bin directory, fixes shell PATH exposure when needed, and falls back from release downloads to local/source builds.

**Architecture:** Implement a POSIX shell installer in `scripts/install.sh` with pure helper functions for target detection, asset naming, install directory choice, and shell profile updates. Keep the hard-to-test behavior in small functions and verify them with a shell test script, while using the existing Cargo build for source fallback.

**Tech Stack:** POSIX `sh`, `curl`/`wget`, `tar`, Rust/Cargo, existing README docs

---

### Task 1: Add a shell test harness for installer helpers

**Files:**
- Create: `tests/install_script.sh`
- Test: `tests/install_script.sh`

**Step 1: Write the failing test**

Create a shell test file that expects these helpers to exist after sourcing `scripts/install.sh`:

```sh
assert_eq "$(asset_name darwin arm64)" "ayoru-darwin-aarch64.tar.gz"
assert_eq "$(asset_name linux x86_64)" "ayoru-linux-x86_64.tar.gz"
assert_eq "$(shell_profile_for zsh /tmp/home)" "/tmp/home/.zprofile"
```

Also assert that PATH-update logic does not duplicate the export line.

**Step 2: Run test to verify it fails**

Run: `sh tests/install_script.sh`
Expected: FAIL because `scripts/install.sh` does not exist yet.

**Step 3: Write minimal implementation**

Create helper functions in `scripts/install.sh` that are safe to source in tests:

- `normalize_arch`
- `asset_name`
- `shell_profile_for`
- `path_export_line`
- `append_path_if_missing`

**Step 4: Run test to verify it passes**

Run: `sh tests/install_script.sh`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/install_script.sh scripts/install.sh
git commit -m "test: add installer helper coverage"
```

### Task 2: Implement release-download install flow

**Files:**
- Modify: `scripts/install.sh`
- Test: `tests/install_script.sh`

**Step 1: Write the failing test**

Add tests that assert:

```sh
assert_eq \
  "$(release_asset_url v0.1.0 darwin arm64)" \
  "https://github.com/BrandonLee28/ayoru/releases/download/v0.1.0/ayoru-darwin-aarch64.tar.gz"
```

and that unsupported platforms fail cleanly.

**Step 2: Run test to verify it fails**

Run: `sh tests/install_script.sh`
Expected: FAIL because release URL generation and platform validation are missing.

**Step 3: Write minimal implementation**

Add:

- `release_asset_url`
- platform/arch detection
- download helper using `curl` or `wget`
- extract/install helper for tarballs

**Step 4: Run test to verify it passes**

Run: `sh tests/install_script.sh`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/install_script.sh scripts/install.sh
git commit -m "feat: add release-based installer flow"
```

### Task 3: Implement source-build fallback

**Files:**
- Modify: `scripts/install.sh`
- Test: `tests/install_script.sh`

**Step 1: Write the failing test**

Add tests that prove source fallback chooses the local checkout when `.git` and `Cargo.toml` are present, and otherwise constructs the clone URL:

```sh
assert_eq "$(repo_source_dir /repo)" "/repo"
assert_eq "$(repo_archive_url)" "https://github.com/BrandonLee28/ayoru.git"
```

**Step 2: Run test to verify it fails**

Run: `sh tests/install_script.sh`
Expected: FAIL because source fallback helpers are missing.

**Step 3: Write minimal implementation**

Add:

- local-checkout detection
- temp-directory clone/build flow
- cargo presence check
- copy from build output into `~/.local/bin/ayoru`

**Step 4: Run test to verify it passes**

Run: `sh tests/install_script.sh`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/install_script.sh scripts/install.sh
git commit -m "feat: add installer source fallback"
```

### Task 4: Make install command the primary README path

**Files:**
- Modify: `README.md`

**Step 1: Write the failing test**

Manually check that README does not yet lead with the one-command installer.

**Step 2: Run test to verify it fails**

Run: `rg -n "curl -fsSL https://raw.githubusercontent.com/BrandonLee28/ayoru/main/scripts/install.sh \\| sh" README.md`
Expected: no matches

**Step 3: Write minimal implementation**

Update README to:

- lead with the `curl | sh` command
- explain that it installs into `~/.local/bin`
- keep Cargo under a developer-focused section
- document release asset naming expectations briefly

**Step 4: Run test to verify it passes**

Run: `rg -n "curl -fsSL https://raw.githubusercontent.com/BrandonLee28/ayoru/main/scripts/install.sh \\| sh" README.md`
Expected: one match

**Step 5: Commit**

```bash
git add README.md
git commit -m "docs: add one-command installer"
```

### Task 5: Verify end-to-end local installer behavior

**Files:**
- Modify: `scripts/install.sh` if needed
- Modify: `tests/install_script.sh` if needed

**Step 1: Write the failing test**

Add a smoke path in `tests/install_script.sh` or a manual verification step that installs into a temp HOME and temp bin dir, then confirms the binary exists.

**Step 2: Run test to verify it fails**

Run: `HOME="$(mktemp -d)" INSTALL_DIR="$(mktemp -d)" sh scripts/install.sh --from-source`
Expected: FAIL until full CLI flow and copy/install path are wired.

**Step 3: Write minimal implementation**

Support a local verification mode such as `--from-source` and optional `AYORU_INSTALL_DIR` for non-destructive test installs.

**Step 4: Run test to verify it passes**

Run: `HOME="$(mktemp -d)" AYORU_INSTALL_DIR="$(mktemp -d)" sh scripts/install.sh --from-source`
Expected: PASS and `ayoru` exists in the temp install dir.

**Step 5: Commit**

```bash
git add scripts/install.sh tests/install_script.sh README.md
git commit -m "feat: verify local installer flow"
```
