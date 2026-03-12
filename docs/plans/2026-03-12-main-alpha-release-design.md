# Main Alpha Release Design

**Goal:** Make the current `main` branch automatically publish the installer assets used by the default install path while the project stays in alpha.

## Decision

Use a rolling `alpha` GitHub Release for pushes to `main`.

- pushes to `main` rebuild and republish installer tarballs
- the release tag for that channel is `alpha`
- version tags like `v0.1.1` still publish versioned releases
- the installer default release target becomes `alpha`

## Why

- keeps installs aligned with the newest `main`
- avoids rewriting semver tags
- keeps a path open for proper tagged releases later
- keeps installer logic simple and deterministic

## Operational Notes

- `alpha` should be treated as the moving prerelease channel
- `v*` tags remain the explicit release path
- docs should explain that `main` updates `alpha`, while tags publish versioned releases
