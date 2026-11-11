# Security Model

Owl is built around a few simple rules: no account, no telemetry, and nothing leaves your PC unless a page you open asks for it. The one optional exception is the update check.

## What is protected, and how

| Area | What Owl does |
|---|---|
| Encryption at rest | History, bookmarks, session and statistics are encrypted (ChaCha20-Poly1305). Cookies, logins and site storage are sealed to an X25519 public key on lock, exit, crash recovery and Windows shutdown. |
| Passphrase | One Argon2id derivation (64 MiB) produces every key. Wrong guesses are rate-limited with a persisted, escalating lock-out. |
| Password vault | Windows DPAPI plus a secret derived from your passphrase. The engine's own password and autofill storage is off. |
| Keys in memory | Overwritten when Owl locks, the key changes or Owl exits. |
| Auto-lock | After idle time, and when Windows locks, sleeps or shuts down. |
| Safe failure | Atomic writes with backups; a damaged settings file enters a recovery state; a failed restore never deletes the sealed file. |
| Internal pages | Per-load nonce CSP, no inline handlers, no framing, a command allow-list for web pages, native (non-forgeable) fullscreen, popup and dialog events. |
| Tracking | Host blocklist plus optional strict tracking prevention. |
| HTTPS | Optional HTTPS-only mode with a warning page; lock or "Not secure" indicator. |
| Permissions | Sensors, local fonts, MIDI and window management always refused; optional block for camera, mic, location, notifications. |
| Fingerprinting | Noise on canvas and audio, rounded hardware hints, strict mode. |
| Downloads | Risky types and bursts held for approval; open without a shell, only from Downloads; reputation checks required. |
| Dialogs | JavaScript dialogs shown in Owl's own modal. |

## Sealed to a public key

Profile data is encrypted to a public key, so Owl can lock it at any moment (including after a crash or at Windows shutdown) without asking for your password. Only your passphrase unwraps the private key.

## Honest limits

- Hobby-grade project, **not** professionally audited.
- While Owl is open and unlocked, other programs running as you can read its files and memory.
- Sealing gives confidentiality, not tamper resistance.
- Engine security fixes come through Windows and Edge updates.
- Fingerprint protection does not make you anonymous.
- A weak passphrase weakens everything.

## Reporting a vulnerability

Use **Security > Report a vulnerability** on the repository. Do not open a public issue. See [SECURITY.md](https://github.com/fardinahosancse/owl-browser/blob/main/SECURITY.md).
