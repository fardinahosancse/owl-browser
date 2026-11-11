<p align="center">
  <img src="assets/banner.svg" alt="Owl. Ultra-light private browser" width="100%">
</p>

<p align="center">
  <a href="https://github.com/fardinahosancse/owl-browser/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/fardinahosancse/owl-browser/actions/workflows/ci.yml/badge.svg"></a>
  <a href="https://github.com/fardinahosancse/owl-browser/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/fardinahosancse/owl-browser?color=E30613&label=release"></a>
  <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-111111"></a>
  <img alt="Platform: Windows 10/11" src="https://img.shields.io/badge/platform-Windows%2010%20%7C%2011-111111">
  <img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-Rust-E30613">
</p>

<p align="center">
  <b>Owl</b> is a small, fast and private desktop browser for Windows. It is written in Rust,
  renders with the system's Edge WebView2 engine, and wraps it in a Swiss-style interface that
  puts the address bar and tabs at the bottom and gives the page every other pixel.
</p>

---

## Contents

- [Highlights](#highlights)
- [Install](#install)
- [Prerequisites](#prerequisites)
- [Build from source](#build-from-source)
- [First run](#first-run)
- [Keyboard shortcuts](#keyboard-shortcuts)
- [Where your data lives](#where-your-data-lives)
- [Privacy and security](#privacy-and-security)
- [Architecture](#architecture)
- [Releases and CI/CD](#releases-and-cicd)
- [Contributing](#contributing)
- [Credits](#credits)
- [License](#license)

## Highlights

| | |
|---|---|
| **Tiny** | The whole browser is a single executable of about 1 MB. The page engine is the Edge WebView2 runtime already on your PC, so YouTube, Facebook, logins, caching and 1080p60 video work as in Edge. |
| **Swiss design** | A footer toolbar (tabs, address bar, window controls), generous type, a red accent, and light, dark, black, navy, invert, sand and random themes. Motion respects the system "reduce motion" setting. |
| **Private by default** | Built-in tracker and ad blocking, HTTPS-only mode, Global Privacy Control, fingerprint noise, an optional proxy or Tor, and private windows that never save anything. |
| **Encrypted at rest** | Cookies, saved logins, history and site storage are sealed on lock, on exit, after a crash and at Windows shutdown. Saved passwords are bound to your passphrase. |
| **Start-up passphrase** | A first-run profile replaces the default key with your own passphrase. Wrong guesses are rate-limited and locked out with growing delays. |
| **Real browser features** | Tabs with drag reorder, pin, mute and duplicate; bookmarks with groups; history; a download manager with pause, resume, speed and folders; find in page; zoom; a password manager; session restore; a now-playing widget for music and video. |

## Install

### Installer (recommended)

1. Open the [latest release](https://github.com/fardinahosancse/owl-browser/releases/latest).
2. Download `Owl-Setup-<version>.exe` and run it.
3. Start **Owl** from the Start menu.

The installer is per-user by default (no administrator rights needed), adds Start menu and
optional desktop shortcuts, and offers to remove your data when you uninstall (it keeps it by
default).

### Portable

Download `Owl-<version>-portable.zip`, extract it anywhere and run `Owl.exe`.

Every release publishes a `SHA256SUMS.txt`. Verify your download:

```powershell
Get-FileHash .\Owl-Setup-<version>.exe -Algorithm SHA256
```

## Prerequisites

To **run** Owl:

| Requirement | Notes |
|---|---|
| Windows 10 (1809+) or Windows 11, 64-bit | |
| Microsoft Edge WebView2 Runtime | Already included with Windows 11 and current Windows 10. The installer checks for it and links to the official download if it is missing. |

To **build** Owl from source:

| Requirement | Notes |
|---|---|
| [Rust](https://rustup.rs) stable, 1.80 or newer | `rustup default stable` |
| A Windows linker toolchain | Either [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) (MSVC, "Desktop development with C++") **or** MSYS2/MinGW-w64 with the `x86_64-pc-windows-gnu` Rust target |
| Git | |
| [Inno Setup 6](https://jrsoftware.org/isinfo.php) | Only if you want to build the installer locally |

## Build from source

```powershell
git clone https://github.com/fardinahosancse/owl-browser.git
cd owl-browser

# optimised, size-tuned build (output: target\release\owl.exe)
cargo build --release --locked
```

With the GNU toolchain the WebView2 loader is a separate DLL. It is produced during the build
under `target\release\build\webview2-com-sys-*\out\x64\WebView2Loader.dll`; keep it next to
`owl.exe`. The MSVC toolchain links the loader statically and needs no extra file.

Build the installer (optional):

```powershell
$env:OWL_VERSION = "0.1.0"
New-Item -ItemType Directory dist -Force | Out-Null
Copy-Item target\release\owl.exe dist\Owl.exe
iscc installer\owl.iss        # writes installer\Output\Owl-Setup-0.1.0.exe
```

## First run

1. A **Welcome** screen asks for your name, email and a new passphrase (10 or more characters).
   Your name and email stay on this PC inside the encrypted store and are never sent anywhere.
2. Owl opens to the home page: clock, search, quick access, bookmark groups, time-online and
   privacy statistics, downloads and shortcuts.
3. Open **Menu > Settings** to choose a theme, set auto-lock, a proxy, sync and more.

## Keyboard shortcuts

| Shortcut | Action | Shortcut | Action |
|---|---|---|---|
| `Ctrl+T` | New tab | `Ctrl+L` | Address bar |
| `Ctrl+W` | Close tab | `Ctrl+F` | Find in page |
| `Ctrl+D` | Bookmark | `Ctrl+B` | Bookmark bar |
| `Ctrl+H` | History | `Ctrl+R` / `F5` | Reload |
| `Alt+Left` / `Alt+Right` | Back / forward | `F11` | Full screen |
| `Ctrl+` `+` `-` `0` | Zoom | `Ctrl+Shift+L` | Lock Owl |
| `F12` | Developer tools (page tabs) | | |

## Where your data lives

Everything is local, under `%LOCALAPPDATA%\Owl`:

| Path | Contents |
|---|---|
| `config.json` (+ `config.bak`) | Settings, wrapped keys, key-derivation salt. Contains no passwords. |
| `store.json` | History, bookmarks, session, downloads and statistics. **Encrypted.** |
| `vault.json` (+ `vault.bak`) | Saved passwords, each encrypted with Windows DPAPI plus a secret derived from your passphrase. |
| `profile\` | The WebView2 profile. While Owl is locked or closed, sensitive files are moved into `profile\sealed.owl`. |
| `extensions\` | Optional user scripts (`*.js`) that run on every web page. |

## Privacy and security

Owl is built around a few simple rules:

- **No account, no telemetry, no analytics.** The only network traffic is what pages you open make.
- **Strict internal pages.** `owl://` pages use a per-load nonce Content-Security-Policy, no inline
  handlers, and cannot be framed. Web pages can reach only a short allow-list of commands.
- **Downloads are held for approval** when they are risky types, start from a background tab or
  arrive in a burst. Files open without a shell and only from the Downloads folder.
- **Passphrase, not a PIN.** Keys are derived with Argon2id (64 MiB) and a persisted, escalating
  lock-out. All sub-keys (vault, store, sealing) come from one derivation.
- **Sealed to a public key.** Profile data is encrypted to an X25519 public key, so Owl can lock it
  at any moment (including after a crash or at Windows shutdown) without asking for your password.
  Only your passphrase unwraps the private key.
- **Safe failure.** Settings and vault writes are atomic with backups; a damaged settings file puts
  Owl into a recovery state instead of starting over; a failed restore never deletes the sealed file.

Honest limits:

- Owl is a hobby-grade project and has **not** had a professional audit. See [SECURITY.md](SECURITY.md).
- While Owl is open and unlocked, other programs running as you can read its files and memory.
- Fingerprint protection reduces common signals; it does not make you anonymous.
- A weak passphrase weakens everything derived from it. Use a long one.

Found a vulnerability? Please follow [SECURITY.md](SECURITY.md) and do not open a public issue.

## Architecture

```
owl.exe  (Rust, ~1 MB)
├── tao            window and event loop (borderless, custom controls)
├── wry            WebView2 hosting: one UI webview + one webview per tab
├── WebView2       Edge's engine (system runtime)
└── src/
    ├── main.rs      app state, commands, trust model, locking, tabs
    ├── ui.html      browser chrome (footer toolbar, tabs, menu, modal, lock screen)
    ├── pages.rs     owl:// pages (home, history, bookmarks, downloads, passwords, settings)
    ├── themes.rs    shared colour themes
    ├── crypt.rs     Argon2id, sub-keys, ChaCha20-Poly1305, X25519 sealed box
    ├── vault.rs     password vault (DPAPI + passphrase-bound entropy)
    ├── profile.rs   seal and restore of cookies, logins and site data
    ├── dl.rs        download manager hooks
    ├── native.rs    non-forgeable WebView2 events (fullscreen, popups, dialogs, updates)
    └── media.rs     now-playing widget and per-tab audio
```

The chrome is a single trusted WebView that talks to Rust over a JSON command channel. Page
webviews are untrusted and get an explicit allow-list of commands.

## Releases and CI/CD

- **CI** (`.github/workflows/ci.yml`) builds on every push and pull request, runs a secret scan and
  a dependency audit, and uploads the executable as a workflow artifact.
- **Release** (`.github/workflows/release.yml`) runs when a version tag is pushed. It builds the
  optimised executable, compiles the **installer** with Inno Setup, creates a portable zip and
  checksums, takes the release notes from [CHANGELOG.md](CHANGELOG.md), and publishes a GitHub
  Release.

To cut a release:

```powershell
# 1. bump `version` in Cargo.toml and add a "## [x.y.z] - YYYY-MM-DD" section to CHANGELOG.md
# 2. commit, then tag and push
git commit -am "Release v0.2.0"
git tag v0.2.0
git push origin main --tags
```

The workflow refuses to publish if the tag and `Cargo.toml` version disagree.

## Contributing

Issues and pull requests are welcome. Please read [CONTRIBUTING.md](CONTRIBUTING.md) first.

## Credits

- [Lucide](https://lucide.dev) icons (ISC licence), drawn inline.
- [tao](https://github.com/tauri-apps/tao) and [wry](https://github.com/tauri-apps/wry) from the Tauri project.
- [webview2-com](https://github.com/wravery/webview2-rs) and the [windows](https://github.com/microsoft/windows-rs) crate.
- [Argon2](https://github.com/RustCrypto/password-hashes), [ChaCha20-Poly1305](https://github.com/RustCrypto/AEADs)
  and [x25519-dalek](https://github.com/dalek-cryptography/curve25519-dalek).
- Microsoft Edge WebView2 for the rendering engine.

## License

[MIT](LICENSE) © 2026 fardinahosancse
