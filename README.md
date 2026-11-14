# ayoru

A quieter way to watch anime. ayoru is a native macOS desktop app built in Rust with [GPUI](https://gpui.rs/), with a Monocode-adapted charcoal interface, compact system-type sidebar, poster shelves, and video playback inside the window.

- Search titles and browse their episodes with mouse or keyboard.
- Save favorites and replay episodes from history or Recently Watched.
- Watch inside the app with native playback controls, seeking, volume, and fullscreen.
- Automatically try ranked stream providers when playback fails.
- Keep your library locally, without accounts or sync.
- Check API availability in the sidebar footer; it refreshes on launch and every minute after each check. The indicator checks the GraphQL API, while individual video hosts may still be unavailable.

The desktop app replaces the terminal interface. macOS is the first supported platform; Linux and Windows builds are not included in this migration.

## Current provider status

The AllAnime integration uses the service's current `mkissa.to` / `api.mkissa.net` endpoints. It reads public bootstrap parameters from the live frontend, obtains the current playback key, generates episode request tokens, and authenticates encrypted source responses. No downloaded JavaScript is executed, and no third-party key feed is required.

Live validation on November 14, 2026 passed search, artwork metadata, episode listing, source resolution, and embedded playback of a resolved episode. The sidebar uses native macOS background blur with a charcoal tint; the content pane uses an opaque inset surface.

## Install

Download the `.dmg` for your Mac from [Releases](https://github.com/BrandonLee28/ayoru/releases):

- Apple silicon: `ayoru-darwin-aarch64.dmg`
- Intel: `ayoru-darwin-x86_64.dmg`

Open the disk image and drag **ayoru** to **Applications**. macOS 12 or later is required. No external video player or Rust installation is needed to run the app.

Desktop assets become available after this branch is merged and the release workflow runs. Builds are ad hoc signed unless a Developer ID identity is supplied during packaging. For downloaded builds that macOS blocks, use **System Settings → Privacy & Security → Open Anyway** after attempting to open the app. Official Developer ID signing and notarization require the maintainer's Apple credentials.

The shell installer installs the app to `~/Applications`:

```sh
curl -fsSL https://raw.githubusercontent.com/BrandonLee28/ayoru/main/scripts/install.sh | sh
open ~/Applications/ayoru.app
```

Set `AYORU_INSTALL_DIR` to choose another application directory. The installer no longer changes your shell profile or installs a terminal executable.

## Use

Type a title in the search field and press **Return**. Click a poster to see episodes, then select an episode and press **Play**. During playback, click an episode number to switch episodes. **Watch Now** contains your recent episodes and favorites. Favorites and History have removal controls; **Clear History** also clears Recently Watched.

- **⌘B**: collapse or expand the sidebar
- **⌘K / ⌘F**: expand the sidebar and focus search
- **Escape**: stop playback and return to episodes, or leave title details
- **Tab / Shift-Tab**: move between graphical controls
- **Return / Space**: activate a focused control
- **⌘W / ⌘Q**: close / quit

Video uses macOS AVFoundation/AVKit and supports the media formats available to the system, including compatible MP4 and HLS streams. On macOS 14 and later, ayoru supplies the MIME type for the provider’s extensionless MP4 links. Older systems rely on format detection and fall back when a source cannot be identified. It retains the sub-first stream ranking and six-second startup timeout, trying each provider once. History is recorded only after playback starts. Some sources may be unavailable or use codecs unsupported by macOS; failures move to the next provider and eventually show an error with a Retry control.

Provider availability depends on the upstream service and your network. Frontend format changes can still require an integration update, but rotating playback epochs and build-specific mask values are read at runtime. A blocked request, missing episodes, missing artwork, or exhausted stream candidates appears in the interface. Titles without artwork use a text card.

## Your existing library

The desktop app reads the same JSON file as the terminal version, including favorites, history, and recently watched:

- `$XDG_STATE_HOME/ayoru/library.json`, when set
- otherwise `~/.local/state/ayoru/library.json`

Existing entries load without migration; poster URLs are optional. Saves replace the file atomically. An unreadable library is preserved and saving stays disabled until you repair it and restart the app. Artwork is cached in `~/Library/Caches/ayoru/artwork` and can be deleted without losing your library.

## Build and package

Install Apple's Command Line Tools (`xcode-select --install`) and [Rust](https://rustup.rs/). The repository pins Rust 1.92. GPUI compiles its Metal shaders at runtime, so full Xcode is not required.

```sh
cargo run
cargo build --release --locked
sh scripts/package-macos.sh
open dist/ayoru.app
```

Packaging creates `dist/ayoru.app`, an architecture-specific app tarball, and a drag-to-Applications DMG. To package a cross-target build, pass its binary, output directory, and architecture:

```sh
sh scripts/package-macos.sh target/aarch64-apple-darwin/release/ayoru dist aarch64
```

`AYORU_SKIP_DMG=1` builds the app and tarball only. `AYORU_SIGNING_IDENTITY` selects a configured code-signing identity (default: ad hoc). `AYORU_NOTARY_PROFILE` optionally submits and staples the DMG using an existing `notarytool` keychain profile. The workflow builds separate Apple silicon and Intel artifacts; pushes to `main` update `alpha`, while version tags publish versioned releases.

## Development checks

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
sh tests/install_script.sh
sh tests/release_workflow.sh
sh tests/native_macos.sh
```

The native smoke test checks Japanese search text, view placement, playback cleanup, and actual video decoding when `ffmpeg` is available to generate a local fixture. It requires access to macOS video services; filesystem sandboxing can prevent decode. The opt-in live check exercises search, artwork metadata, episodes, and stream resolution:

```sh
cargo test --test provider_live -- --ignored
```

## Source layout

```text
src/desktop/       GPUI interface, background requests, native control bridge
src/library/       compatible saved library and storage
src/session.rs     embedded playback fallback and history confirmation
src/provider/      AllAnime requests, bootstrap/token protocol, and source resolution
src/core/          models and stream ranking
native/            AppKit search field, AVKit player, app icon drawing
scripts/           desktop installation and app/DMG packaging
```

`docs/highlevel.md` describes the desktop architecture. Older documents under `docs/plans/` describe the retired terminal application.

The desktop visual style is adapted from Monocode Native. See [design attribution](THIRD_PARTY_NOTICES.md).
