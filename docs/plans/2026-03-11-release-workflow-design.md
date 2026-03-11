# Ayoru Release Workflow Design

**Date:** 2026-03-11

## Goal

Automate GitHub Releases for Ayoru so pushing a version tag publishes the release assets expected by the installer.

## Trigger

The workflow should run on pushed git tags matching:

- `v*`

This keeps releases explicit and aligns with installer URLs such as:

- `https://github.com/BrandonLee28/ayoru/releases/download/v0.1.0/...`

## Product Direction

The installer already expects GitHub release artifacts. The release workflow should be the missing operational piece that makes those downloads real.

This v1 should stay narrow:

- no general CI workflow expansion
- no Homebrew publishing
- no nightly or prerelease channels

## Recommended Approach

Use one tag-driven GitHub Actions workflow with an explicit build matrix.

The matrix should define:

- runner OS
- Rust target triple
- output asset name

This is the simplest way to keep release asset names aligned with the installer contract.

## Build Targets

The workflow should publish exactly these assets:

- `ayoru-darwin-aarch64.tar.gz`
- `ayoru-darwin-x86_64.tar.gz`
- `ayoru-linux-aarch64.tar.gz`
- `ayoru-linux-x86_64.tar.gz`

## Runner Strategy

Use native GitHub-hosted runners:

- `macos-latest` for Darwin artifacts
- `ubuntu-latest` for Linux artifacts

Avoid cross-platform packaging in the first version. Native runner builds are more reliable and easier to debug.

## Packaging Contract

Each tarball should contain only:

- `ayoru`

The workflow must fail if any expected tarball is not produced.

## Release Publishing

The workflow should:

1. build each target
2. package the `ayoru` binary into the exact tarball name
3. upload the tarballs as job artifacts
4. create or update the GitHub Release for the tag
5. attach all four tarballs to that release

## Verification

Verification should cover:

- workflow YAML syntax sanity
- exact asset file names
- documented tag flow for maintainers
- alignment with the installer naming contract

## Documentation

README should include a short maintainer note describing the release flow:

```bash
git tag v0.1.0
git push origin v0.1.0
```

That is enough to trigger asset publication.
