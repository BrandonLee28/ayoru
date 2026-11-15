# Contributing to ayoru

`main` contains the native macOS desktop app. The [`cli`](https://github.com/BrandonLee28/ayoru/tree/cli) branch preserves the terminal app. Open fixes against the branch they affect; shared fixes may need to be applied to both.

## Desktop development

Install Apple's Command Line Tools and Rust. The repository pins the required Rust toolchain.

```sh
cargo run
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
sh tests/install_script.sh
sh tests/release_workflow.sh
sh tests/native_macos.sh
```

Native tests need access to macOS video services; `ffmpeg` enables the generated video decoding fixture. Live provider checks are optional and depend on the upstream service:

```sh
cargo test --test provider_live -- --ignored
```

See the [architecture](docs/highlevel.md) and [packaging guide](docs/guide.md#build-and-package) for implementation and distribution details.

## Releases

- Pushes to `main` publish desktop builds to `alpha`; `v*` tags publish versioned desktop releases.
- Pushes to `cli` publish terminal builds to `cli-alpha`; `cli-v*` tags publish versioned terminal releases.
- Keep these release channels separate: desktop archives contain an app bundle, while CLI archives contain a terminal executable.

## Reporting issues

Include your macOS or Linux version, CPU architecture, whether you use desktop or CLI, and steps to reproduce. For playback issues, include the title, episode, and displayed error. Avoid sharing your personal library file unless needed.
