# Changelog

All notable changes to Owl are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- **Segmented download engine**: files of 2 MB or more from servers that support ranges download over up to 8 connections at once, with pause, resume and retry with backoff, using the existing download UI. Page cookies are sent only to the host the download started from. Other downloads, and all downloads while a proxy is set, use the browser's normal path.
- **Lazy image compression**: Settings > Privacy slider (0 = off, 1-99). Images on web pages are shrunk and re-encoded before the page receives them; below-the-fold images load lazily. Small images, animations and images that would not get at least 10% smaller are left alone. Checked on a 9.2 MB 3840x5723 JPEG, which arrived as 537x800 at strength 80. Off while a proxy or secure DNS is on.
- **Save image as**: right-click an image to save it as PNG, JPEG, BMP, WebP or GIF into Downloads (also listed in the download list).
- **Secure DNS (Cloudflare 1.1.1.1)**: Settings > Privacy. Owl runs a loopback-only proxy that looks up site names over HTTPS at 1.1.1.1, so your network's DNS server cannot see them. Checked with Cloudflare's own test page (DoH: Yes). It is not a VPN and does not change your IP address; if 1.1.1.1 is blocked pages will not load. It is not used while a proxy is set, and the fast download engine stays off while it is on.
- Download engine extras: a speed limit and a "downloads at once" setting (extra fast downloads wait in a queue), and interrupted fast downloads continue where they stopped when you press Retry.
- **BitTorrent client**: magnet links and finished .torrent downloads open in Owl's built-in torrent engine (librqbit, Apache-2.0) and appear in the normal download list with progress, pause, resume and cancel. It only connects out (no listening port), stops seeding when complete, and is refused in private windows, while locked, and while a proxy is set, because peers would see your real address.
- **OwlKick Engine**: content blocking now runs on the adblock-rust engine used by Brave (MPL-2.0; not made or endorsed by Brave). It understands EasyList and uBlock-style rules, matches by request type and third-party status, and hides page clutter with cosmetic filters. Owl's built-in tracker list always applies; Settings > Update lists downloads EasyList and EasyPrivacy on request.

### Changed
- The executable is about 6 MB instead of 1 MB because of the filter and torrent engines.

## [0.1.2] - 2026-11-11

### Security
- Key material (startup key, vault, store and sealing keys) is overwritten in memory when it is dropped: on lock, key change and exit.
- Permission policy: sensors, local fonts, MIDI system access and window management are always refused. A new setting also refuses camera, microphone, location and notification requests without asking.
- Optional strict tracking prevention for third-party trackers and their stored cookies.
- Reputation (SmartScreen) checks are required explicitly; the engine's own password and autofill storage is switched off so secrets live only in Owl's vault.
- Opt-in, notify-only update check (Settings > Privacy) and a Check now button. It never downloads or installs anything and is skipped when a proxy is set.
- Release files carry GitHub build attestations: verify with `gh attestation verify <file> --repo fardinahosancse/owl-browser`.
- GitHub Actions are pinned to commit SHAs and a cargo-deny licence and sources check runs in CI.

### Changed
- Release notes now include a build log: commit, toolchain, file sizes, SHA-256 checksums and a link to the build run.

## [0.1.1] - 2026-11-11

### Fixed
- Installed app did not open when its folder was read-only (for example Program Files): the engine folder now lives under `%LOCALAPPDATA%\Owl`, and start-up errors show a message box.
- Private windows opened with no tab.
- A settings file saved with a byte-order mark was treated as damaged.
- Window controls hidden while a dialog was open; focus now returns to the page after a dialog.
- Contrast problems: new `--onred` and `--mute-on` theme tokens fix white-on-accent text, muted text on hovered or selected rows, and darker Sand and lighter Invert palettes.
- Narrow windows: row actions on History, Bookmarks and Passwords stay visible, Settings reflows, the tab strip scrolls, the toolbar hides low-value buttons, and the menu scrolls.

- Invert theme contrast, red-on-ink pairings (new `--rink` token), right-click on bookmark-bar items opening a tab, wrong history clear count, About header alignment, selected theme marker, sand swatch colour, undersized link targets.
- Bookmark notices are announced, time out, and Ctrl+D shows them with Undo; shortcuts are ignored while a dialog is open; reduced motion is honoured by smooth scroll and count-up.
- Bookmark-bar edit no longer deletes on an empty answer; Bookmark bar setting moved to Appearance.

### Added
- Command-line start URL (`Owl.exe https://example.com`).
- Keyboard access: tabs (arrow keys, Delete, Menu key), bookmark bar, combobox suggestions, `Ctrl+Tab` and `Ctrl+1-9`, dialog focus trap.
- Address-bar security indicator, find-in-page previous/next/close buttons, bookmark undo notice, recovery screen with Open folder and Quit.
- History: times, day headings, search, per-item remove and a confirmed Clear; bookmarks Move and Remove with confirmation.
- Settings grouped under Appearance, Security, Privacy, Downloads and data, Sync and Profile; Restart now button; About page.

## [0.1.0] - 2026-11-11

First public release.

### Added
- Rust browser on tao + wry using the system Edge WebView2 engine (about 1 MB executable).
- Swiss-style interface: footer toolbar with tabs, address bar and window controls; light, dark,
  black, navy, invert, sand and random themes; animations that respect reduced motion.
- Tabs with drag reorder, pin, mute, duplicate and right-click menu; bookmarks with groups and an
  optional bookmark bar; history; find in page; zoom; full screen; session restore.
- Landing-style home page with clock, search, quick access, time-online and blocking statistics.
- Download manager with progress, combined speed, pause, resume, cancel, retry, folder sorting,
  and an approval step for risky or unexpected downloads.
- Password manager with capture, click-to-fill and a generator; passwords bound to the passphrase.
- Now-playing widget for music and video playing in another tab.
- Built-in tracker and ad blocking, HTTPS-only mode with fallback page, Global Privacy Control,
  fingerprint noise (with a strict mode), popup blocker and optional proxy or Tor.
- First-run setup (name, email, passphrase) replacing the default key.
- Encryption at rest: Argon2id master key, encrypted store, passphrase-bound vault, and profile
  data sealed to an X25519 public key on lock, exit, crash recovery and Windows shutdown.
- Auto-lock, lock on Windows lock or sleep, and restore of typed text and scroll position.
- In-app centered dialog for alerts, confirms and prompts, with a close button.
- Inno Setup installer, portable zip and checksums built by GitHub Actions.

### Security
- Strict per-load nonce CSP on internal pages, no inline handlers, frame protection headers.
- Command allow-list for web pages; native (non-forgeable) fullscreen, popup and dialog events.
- Atomic settings and vault writes with backups and a recovery state.

[Unreleased]: https://github.com/fardinahosancse/owl-browser/compare/v0.1.2...HEAD
[0.1.2]: https://github.com/fardinahosancse/owl-browser/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/fardinahosancse/owl-browser/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/fardinahosancse/owl-browser/releases/tag/v0.1.0
