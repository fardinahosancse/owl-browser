# Architecture

```
owl.exe  (Rust, ~1 MB)
├── tao            window and event loop (borderless, custom controls)
├── wry            WebView2 hosting: one UI webview + one webview per tab
├── WebView2       Edge's engine (system runtime)
└── src/
    ├── main.rs      app state, commands, trust model, locking, tabs
    ├── ui.html      browser chrome (footer toolbar, tabs, menu, modal, lock screen)
    ├── pages.rs     owl:// pages (home, history, bookmarks, downloads, passwords, settings, about)
    ├── themes.rs    shared colour themes and tokens
    ├── crypt.rs     Argon2id, sub-keys, ChaCha20-Poly1305, X25519 sealed box
    ├── vault.rs     password vault (DPAPI + passphrase-bound entropy)
    ├── profile.rs   seal and restore of cookies, logins and site data
    ├── dl.rs        download manager hooks
    ├── native.rs    non-forgeable WebView2 events, permission policy, tracking level
    ├── update.rs    opt-in, notify-only release check
    └── media.rs     now-playing widget and per-tab audio
```

## Trust model

The chrome is a single trusted WebView that talks to Rust over a JSON command channel. Page webviews are untrusted and get an explicit allow-list of commands. Internal pages are served from `owl://` with a per-response nonce CSP and anti-framing headers.

## Key hierarchy

```
passphrase --Argon2id--> master key --> vault key / store key / legacy profile key
                                    \-> unwraps the X25519 private key (opens sealed profile)
public key (in config.json) --> seals profile data without the passphrase
```

## Themes

Every theme defines the same CSS variables (`--bg --ink --mute --soft --line --red --onred --on --mute-on --rink`), so components never special-case a theme.

## Platform notes

Owl currently targets Windows only. A macOS port is tracked as an enhancement issue; it would need the WebView2-specific modules (`native.rs`, `dl.rs`, `media.rs`, `profile.rs`, `vault.rs`) behind a platform layer.
