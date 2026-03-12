# Main Alpha Release Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make pushes to `main` publish a rolling `alpha` release and make the installer default to that alpha channel.

**Architecture:** Extend the release workflow so it triggers on both `main` and `v*` tags. Publish `main` builds to a fixed `alpha` prerelease and keep tag pushes publishing versioned releases. Update the installer to resolve `alpha` by default instead of GitHub's generic latest release path, and update the shell tests and README to match.

**Tech Stack:** GitHub Actions, POSIX shell, Markdown, existing shell contract tests

---

### Task 1: Add failing release workflow contract checks

**Files:**
- Modify: `tests/release_workflow.sh`
- Test: `tests/release_workflow.sh`

**Step 1: Write the failing test**

Add checks asserting the workflow:
- triggers on `main`
- has an `alpha` release path
- keeps the `v*` tag path

**Step 2: Run test to verify it fails**

Run: `sh tests/release_workflow.sh`
Expected: FAIL because the workflow is tag-only today.

**Step 3: Write minimal implementation**

Update `.github/workflows/release.yml` to publish both:
- rolling `alpha` release for `main`
- versioned release for `v*` tags

**Step 4: Run test to verify it passes**

Run: `sh tests/release_workflow.sh`
Expected: PASS

### Task 2: Change installer default release target

**Files:**
- Modify: `scripts/install.sh`
- Modify: `tests/install_script.sh`
- Test: `tests/install_script.sh`

**Step 1: Write the failing test**

Add a shell assertion that the default release URL resolves to the `alpha` tag download path.

**Step 2: Run test to verify it fails**

Run: `sh tests/install_script.sh`
Expected: FAIL because the default path still uses `releases/latest`.

**Step 3: Write minimal implementation**

Change the default release selector from `latest` to `alpha`, while preserving explicit version installs.

**Step 4: Run test to verify it passes**

Run: `sh tests/install_script.sh`
Expected: PASS

### Task 3: Update maintainer-facing docs

**Files:**
- Modify: `README.md`

**Step 1: Update release notes**

Document that:
- pushes to `main` refresh the rolling `alpha` release
- pushing `v0.1.x` still cuts a versioned release

**Step 2: Verify docs**

Run: `rg -n \"alpha|git tag v0\\.1\\.|main\" README.md`
Expected: README reflects both the rolling alpha channel and tagged version releases
