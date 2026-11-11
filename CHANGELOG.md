# Changelog

All notable changes to Owl are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/fardinahosancse/owl-browser/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/fardinahosancse/owl-browser/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/fardinahosancse/owl-browser/releases/tag/v0.1.0
