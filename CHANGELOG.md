# Changelog

All notable changes to Owl are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and this project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.3.0] - 2026-11-13

### Added
- **Download manager**: add a download by pasting a link or magnet link; live per-part progress and connection count for fast downloads; Start now for queued downloads; filters (active, completed, torrents), file-type filter and search; speed limit and downloads-at-once controls on the page; Copy link and Open folder; totals for active downloads, speed, connections and torrent upload.
- **Torrent rows**: their own icon and tag, state (fetching details, downloading, seeding), peers and seeders connected, upload speed and amount uploaded, Stop, Stop and delete, Stop seeding.
- **Toolbar quick toggles**: Saver (Data saver on/off), DNS (secure DNS, applies after restart) and Kick (OwlKick count; opens the activity page).
- **OwlKick activity page** (owl://owlkick): blocked this session and all time, most blocked hosts, pages with most blocking, blocks by kind and the latest blocked requests. Kept in memory only.
- **Settings sidebar**: sticky section list with the current section highlighted, and new groups: Appearance, Security, Privacy, OwlKick blocking, Network, Speed and data, Downloads, Sync, Profile and advanced.
- **Toolbar at the top** option (Settings > Appearance).
- **Calm home page** (Settings > Appearance): one screen built from your own day: a greeting, a large clock and a line showing how much of the day has passed, a plain sentence on time online, what OwlKick stopped and running downloads, the pages you were on with how long ago, where your time goes, and one useful hint. A soft light drifts across the page with the time of day (still with reduced motion).
- With the toolbar at the top, the address bar row with the window buttons is now the topmost row.
- **New clock**: Space Grotesk (SIL Open Font License, bundled; nothing is fetched at run time), a seconds line and the week number.
- **About, Privacy and Terms**: owl://about (menu > About Owl) with the version, the creator and credits; owl://privacy and owl://terms show PRIVACY.md and TERMS.md, which list everything Owl stores and every connection it makes.
- **Icons and tooltips**: every menu item and settings group has an icon; less obvious switches, download actions and settings groups explain themselves on hover. Unfinished features are marked Coming soon (choosing files in a torrent).
- **Native window dragging**: the tab strip and logo act as the title bar, handled by the engine. Dragging starts at once, Windows snap layouts work, double-click maximizes.
- **Landing page**: the Swiss redesign, with sections on the home themes, the download engine and BitTorrent, Data saver (with measured numbers), OwlKick and Secure DNS, and new full-screen screenshots. It deploys with every release and shows the latest version.

### Security
- **Passphrases**: existing 4-digit startup keys must be upgraded to a passphrase at unlock; new passphrases need 12 or more characters and are checked against common passwords.
- **Profile sealing**: sealed profiles are streamed in authenticated chunks with an HMAC trailer, so a forged, swapped or cut file is refused, and there is no size cap that could leave a large profile unsealed. Preferences are sealed too and caches are wiped.
- **Saved passwords**: capture and save use the address the page really shows, entries are bound to their site and user name, and passwords saved on https are never filled on http.
- **Storage**: settings, vault and store writes are crash-safe and a failed write is reported. A passphrase change is journaled, so a crash mid-change cannot lose saved passwords.
- **Sync**: an unencrypted file in the sync folder is refused when a sync passphrase is set.
- **Proxy**: stored site icons are not fetched around a proxy; socks4 proxies (which leak DNS) are refused.
- **Dependencies**: RUSTSEC-2026-0293 in ringbuf fixed with a vendored patch.

### Fixed
- A fast download showed "failed" while still downloading, because the browser's cancelled copy reported an interruption.
- Screen-reader-only labels were visible on some pages.
- First-run setup failed after the 4-digit key upgrade was added.
- A silent uninstall waited on the delete-your-data question; it now keeps the data without asking.
- The settings sidebar marked the wrong group when opened at a section.

### Changed
- Smoother toolbar: the bookmark bar is rebuilt only when bookmarks change, the loading bar no longer restarts on every update, and download rows update in place instead of redrawing the list.
- Dropdowns on the download list and in settings are flat, in the style of the buttons next to them.
- Data saver is a segmented Off / Light / Balanced / Ultra bar over a flat scale with a square thumb.
- Tabs reorder by dragging with a live preview: the others slide out of the way.
- Resizing the window resizes only the visible page, and pages and the toolbar take the theme's colour before they draw, so dark themes no longer flash white.
- Forms have visible labels and errors are tied to their fields; page headings are smaller.
- CI runs the unit tests, a smoke test that drives the real app (setup, fast download, Data saver, Save image as, the shutdown seal) and a silent install and uninstall.

### Known issues
- Releases are not code-signed yet, so Windows SmartScreen may warn on first download (#32). Verify the download with its build attestation or SHA256SUMS.
- Choosing files inside a multi-file torrent is not available yet; all files download (#104).
- Not every click path is covered by automated tests yet; the smoke test covers setup, downloads, Data saver, Save image and shutdown sealing (#107).
- The installer's wizard pages and the missing-WebView2 path are tested by hand only; silent install and uninstall run in CI (#31).
- Find in page has no match counter or previous and next buttons (#65, #83).
- Lock-time restore brings back form fields and scroll position only (#29).
- No macOS version yet (#91).

## [0.2.0] - 2026-11-13

### Added
- **OwlKick Engine**: content blocking now runs on the adblock-rust engine that Brave uses (MPL-2.0; Owl is not made or endorsed by Brave). It understands EasyList and uBlock-style rules, matches by request type and third-party status, and hides page clutter. Owl's built-in tracker list always applies. Settings > Privacy has Update lists (EasyList and EasyPrivacy, only when you press it, then refreshed weekly), and an "off for these sites" list.
- **Fast download engine**: files of 2 MB or more from servers that support ranges download over up to 8 connections, with pause, resume, retry with backoff, a speed limit, a "downloads at once" queue, and resume after Owl restarts (press Retry; cookies are kept encrypted for your Windows account). It uses the existing download list. Page cookies go only to the host the download started from.
- **BitTorrent**: magnet links (on pages, typed in the address bar, or passed by a second launch) and .torrent files open in a built-in client (librqbit, Apache-2.0) and appear in the download list. The torrent engine runs only while a torrent is active, then shuts down completely. It opens no listening port, is refused in private windows, while locked and while a proxy is set, and does not seed unless you turn on "Keep seeding finished torrents".
- **Data saver** (Settings > Privacy, slider 0-99 with Off, Light, Balanced and Ultra presets): sends the `Save-Data` header so cooperating sites send lighter pages and images; loads below-the-fold images lazily with low priority; fetches only the first part of big progressive JPEGs (1 MB or more), which shows as a slightly soft image and saves real bandwidth (checked: a 5.2 MB photo used 1.8 MB at strength 85 and rendered at full size); shrinks other huge images (4 MB or more). Small images are never touched. With a proxy or secure DNS on, the image fetching is off but the Save-Data request and lazy loading still apply. Applies to new pages.
- **Video and audio download button**: when a page plays a plain video or audio file, a toolbar button lists it and downloads it with the fast engine. Streaming playlists (HLS, DASH) and YouTube are not offered.
- **Save image as**: right-click an image to save it as PNG, JPEG, BMP, WebP or GIF. Works for blob: images and same-site cookie-protected images by asking the page to fetch them.
- **Secure DNS (Cloudflare 1.1.1.1)**: Owl runs a loopback-only proxy that looks up site names over HTTPS. Checked with Cloudflare's test page (DoH: Yes). It is not a VPN and does not hide your IP address. If 1.1.1.1 is blocked, pages will not load.
- **Cloudflare WARP switch**: connect, disconnect and check an installed WARP client (a real VPN) from Settings. Owl does not bundle WARP.
- **Site permissions page** (Settings > Site permissions): review and reset what you allowed or blocked for sites.
- A second launch of Owl now brings the running window forward and passes on the address instead of exiting silently.

### Changed
- The executable is about 7 MB (was 1 MB) because of the filter, torrent and image engines.
- Settings > Privacy and Downloads and data have new rows for the features above.

### Fixed
- The Downloads page now coerces every stored value and allow-lists the status, so a tampered store cannot inject markup (#17).
- Restart now waits for the old process to exit, so Owl no longer just closes (#21).
- Two request types were mapped the wrong way round in the blocker.

### Known issues
- Not yet tried in the running app: right-click Save image as, choosing a file from the video list, magnet links on a normal window, the download queue, and resume after restart. They are verified at engine or unit level only (#107).
- Torrents have no file selection; seeding only reaches peers Owl connects to (#104).
- The image fetching part of Data saver, fast downloads and Save image as in another format are off while a proxy or secure DNS is on (they would bypass them).
- Data saver's `Save-Data` header is not added to page navigations, only to sub-requests.
- The permissions page and the WARP connect path were not checked visually or with a real WARP client.
- Dependency advisory RUSTSEC-2026-0293 (ringbuf 0.4.8, pulled in by the torrent library's uTP transport) cannot be updated yet. The vulnerable code is not reachable because Owl starts the torrent engine with no listener, so no uTP socket exists. It is ignored in CI with this reason and tracked on GitHub.
- Files are not Authenticode-signed (SmartScreen may warn); verify with the build attestation (#32).
- macOS version is not started (#91). Open UI issues: form labels and errors (#51, #77), oversized headings (#56), find match counter (#65, #83). Many older P2 and P3 security and test issues remain open; see the issue list.

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

[Unreleased]: https://github.com/fardinahosancse/owl-browser/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/fardinahosancse/owl-browser/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/fardinahosancse/owl-browser/compare/v0.1.2...v0.2.0
[0.1.2]: https://github.com/fardinahosancse/owl-browser/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/fardinahosancse/owl-browser/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/fardinahosancse/owl-browser/releases/tag/v0.1.0
