# Ayoru Release Workflow Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a tag-driven GitHub Actions workflow that builds the installer tarballs and publishes them to the matching GitHub Release.

**Architecture:** Create a single `.github/workflows/release.yml` workflow with an explicit native-runner matrix for Darwin and Linux targets. Package the built `ayoru` binary into installer-compatible tarball names, upload them as artifacts, then attach them to the tag’s GitHub Release.

**Tech Stack:** GitHub Actions, Rust/Cargo, tar, GitHub Releases

---

### Task 1: Add a release-matrix contract check

**Files:**
- Create: `.github/workflows/release.yml`
- Create: `tests/release_workflow.sh`
- Test: `tests/release_workflow.sh`

**Step 1: Write the failing test**

Create a shell test that asserts the workflow file contains the expected targets and asset names:

```sh
assert_contains ".github/workflows/release.yml" "aarch64-apple-darwin"
assert_contains ".github/workflows/release.yml" "ayoru-darwin-aarch64.tar.gz"
```

**Step 2: Run test to verify it fails**

Run: `sh tests/release_workflow.sh`
Expected: FAIL because the workflow file does not exist yet.

**Step 3: Write minimal implementation**

Add a first workflow skeleton with:

- `on.push.tags: ['v*']`
- matrix entries for all four targets
- exact asset names

**Step 4: Run test to verify it passes**

Run: `sh tests/release_workflow.sh`
Expected: PASS

**Step 5: Commit**

```bash
git add .github/workflows/release.yml tests/release_workflow.sh
git commit -m "test: add release workflow matrix contract"
```

### Task 2: Build and package the tarballs

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `tests/release_workflow.sh`
- Test: `tests/release_workflow.sh`

**Step 1: Write the failing test**

Extend the shell test to assert the workflow packages `ayoru` into tarballs and uploads job artifacts.

**Step 2: Run test to verify it fails**

Run: `sh tests/release_workflow.sh`
Expected: FAIL because packaging/upload steps are missing.

**Step 3: Write minimal implementation**

Add workflow steps to:

- install the Rust target
- `cargo build --release --target <target>`
- copy the built `ayoru` binary into a staging directory
- create the exact `.tar.gz`
- upload the tarball with `actions/upload-artifact`

**Step 4: Run test to verify it passes**

Run: `sh tests/release_workflow.sh`
Expected: PASS

**Step 5: Commit**

```bash
git add .github/workflows/release.yml tests/release_workflow.sh
git commit -m "feat: package release tarballs in workflow"
```

### Task 3: Publish GitHub Release assets

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `tests/release_workflow.sh`
- Test: `tests/release_workflow.sh`

**Step 1: Write the failing test**

Extend the shell test to assert the workflow has a release-publishing job that downloads artifacts and uses a release action to attach them to the pushed tag.

**Step 2: Run test to verify it fails**

Run: `sh tests/release_workflow.sh`
Expected: FAIL because release publication is missing.

**Step 3: Write minimal implementation**

Add a release job that:

- depends on all build jobs
- downloads the uploaded tarballs
- creates or updates the GitHub Release for the tag
- attaches all four tarballs

**Step 4: Run test to verify it passes**

Run: `sh tests/release_workflow.sh`
Expected: PASS

**Step 5: Commit**

```bash
git add .github/workflows/release.yml tests/release_workflow.sh
git commit -m "feat: publish release assets from workflow"
```

### Task 4: Document the maintainer release flow

**Files:**
- Modify: `README.md`

**Step 1: Write the failing test**

Check that README does not yet document the tag-triggered release flow.

**Step 2: Run test to verify it fails**

Run: `rg -n "git tag v0\\.1\\.0" README.md`
Expected: no matches

**Step 3: Write minimal implementation**

Add a short maintainer note with:

```bash
git tag v0.1.0
git push origin v0.1.0
```

and note that the workflow publishes the release assets automatically.

**Step 4: Run test to verify it passes**

Run: `rg -n "git tag v0\\.1\\.0" README.md`
Expected: one match

**Step 5: Commit**

```bash
git add README.md
git commit -m "docs: add tag-driven release notes"
```

### Task 5: Verify workflow structure locally

**Files:**
- Modify: `.github/workflows/release.yml` if needed
- Modify: `tests/release_workflow.sh` if needed

**Step 1: Write the failing test**

Add final checks in `tests/release_workflow.sh` that assert all four expected asset names appear exactly once and that the workflow triggers on `v*`.

**Step 2: Run test to verify it fails**

Run: `sh tests/release_workflow.sh`
Expected: FAIL until the workflow is complete.

**Step 3: Write minimal implementation**

Tighten the workflow YAML until the local contract checks pass.

**Step 4: Run test to verify it passes**

Run: `sh tests/release_workflow.sh`
Expected: PASS

**Step 5: Commit**

```bash
git add .github/workflows/release.yml tests/release_workflow.sh README.md
git commit -m "test: verify release workflow contract"
```
