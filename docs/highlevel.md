# Ayoru desktop architecture

Ayoru is a macOS desktop application. Rust/GPUI owns the graphical shell; AppKit and AVKit supply native search input and embedded video controls. The previous terminal renderer, terminal runtime, external-player detection, and player launch code have been removed.

## User flow

1. Search titles using the native search field.
2. Browse a GPUI poster grid and open a title.
3. Load episodes and select one.
4. Resolve sub-first streams from AllAnime, rank them, and deduplicate providers.
5. Load a candidate into an AVPlayerView attached to the same GPUI window.
6. Confirm actual playback before recording history; try another provider after failure or six seconds without starting.

Watch Now shows recent episodes and favorites. Favorites can be toggled or removed. History can replay a saved episode, open its title, remove individual entries, or clear history and recents. Both pointer and focus-based keyboard activation are supported.

## Ownership and concurrency

- `src/desktop/mod.rs`: navigation, background request dispatch, artwork caching, and UI state. `src/desktop/ui.rs` owns the Monocode-adapted neutral charcoal sidebar, compact system typography, inset workspace, artwork/player canvas, poster grids, and episode selection below the player. Tokio runs network work away from the main thread. A channel delivers results to a 100 ms UI task. Request generations and cancellation prevent stale search/episode/stream responses from changing the active view.
- `src/desktop/native.rs`: a main-thread-only owner of the native controls. Canvas layout supplies logical-point bounds and the window's AppKit view. Video sits outside the scrolling content to keep the native view from covering scrolled controls.
- `native/macos.m`: ARC-managed NSSearchField and AVPlayerView, search submission, playback status/error reporting, and teardown. The player provides play/pause, seeking, volume, and fullscreen. Stopping or leaving playback releases the AVPlayer. The bridge passes each source’s referrer to media loading. On supported macOS versions it supplies an MP4 MIME hint for extensionless sources. AVPlayer’s automatic stalling wait is disabled so playback can start within the application’s six-second policy.
- `src/session.rs`: one attempt per ranked provider, startup timeout, and history confirmation exactly once per episode request. A user pause after playback starts does not trigger the startup timeout.
- `src/provider/allanime.rs`: POST metadata queries and authenticated GET episode queries against `api.mkissa.net`, optional thumbnail metadata, episode parsing, source decoding, MP4Upload resolution, and clock/Wix path expansion. Per-source referrers and resolution metadata survive expansion. A malformed source does not discard other providers.
- `src/provider/allanime_crypto.rs`: reads literal manifest data from the current `mkissa.to` frontend and its fixed CDN, without evaluating JavaScript. It obtains the provider’s current bootstrap key, signs five-minute AES-GCM request tokens, authenticates encrypted responses, and refreshes once if a deployment rotates between bootstrap and source retrieval. Frontend downloads and string-table parsing are bounded. There is no third-party key-feed dependency or hardcoded build ID/epoch.
- `src/library/`: the original JSON schema plus optional poster metadata. The storage path stays compatible. Writes use a sibling temporary file and rename. Read failures preserve the file and disable UI saves for that session.

Artwork downloads have time and size limits and live in the macOS user cache. Missing artwork falls back to title text. Search and playback errors are surfaced in the interface with retry available for playback.

## Build and distribution

`Cargo.toml` pins GPUI 0.2.2 and `rust-toolchain.toml` pins Rust 1.92. GPUI's `runtime_shaders` feature avoids the full-Xcode Metal compiler requirement. `build.rs` compiles the small Objective-C bridge and links system frameworks; there is no external player, bundled browser, or extra playback daemon.

`scripts/package-macos.sh` creates Ayoru.app with its icon and Info.plist, signs it, and produces app tarballs and DMGs. GitHub Actions builds Apple silicon and Intel versions. Developer ID signing and notarization are optional local packaging settings; CI uses ad hoc signing until credentials are configured.

## Validation and limits

Tests cover ranking, source parsing, fallback/timeouts, one-time history recording, legacy JSON loading, atomic save behavior, installer paths, and release assets. The native smoke test checks control attachment, Unicode input, decoding a generated local video, and cleanup. The live provider test is opt-in because upstream sources and network challenges can change independently of the app.

AVKit supports macOS media formats rather than every codec supported by mpv. Unsupported or unavailable sources trigger fallback. Accounts, sync, recommendations, collections, playback-position tracking, and other platforms remain outside the existing feature set.

## Live validation status (November 14, 2026)

The repaired flow passes search, poster metadata, episode listing, encrypted source retrieval, direct media byte access, and AVKit playback of a resolved episode. Regression fixtures cover literal manifest rotation, independent AES-GCM token bytes, tampered ciphertext, checked numeric expressions, clock-path idempotence, malformed-source isolation, media headers, and MP4Upload page expansion.

Public protocol behavior was investigated using the provider’s own frontend and bootstrap endpoint. The native MIME hint uses Apple’s documented [AVURLAssetOverrideMIMETypeKey](https://developer.apple.com/documentation/avfoundation/avurlassetoverridemimetypekey). macOS 12 remains the minimum deployment target; the public user-agent and MIME override options are guarded by their macOS 13/14 availability.

The provider can still change its frontend representation or endpoints. Unknown formats fail with a clear error rather than running downloaded code or falling back to stale tokens.
