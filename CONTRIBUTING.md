# Contributing to Owl

Thanks for helping. Owl is small on purpose, so changes that keep it light, private and simple
are the easiest to accept.

## Getting set up

See [Prerequisites](README.md#prerequisites) and [Build from source](README.md#build-from-source).

```powershell
cargo build --release --locked
cargo check --locked
```

Test changes with a throwaway profile so you never touch your real data. Owl reads its folder from
`%LOCALAPPDATA%`, so you can point it elsewhere for a session:

```powershell
$env:LOCALAPPDATA = "$env:TEMP\owl-dev"
.\target\release\owl.exe
```

## Pull requests

- One focused change per pull request, with a short description of what and why.
- Keep the executable small: avoid new dependencies unless they are clearly worth it.
- Anything that handles untrusted input (pages, downloads, files read from disk) needs care:
  validate, escape, and prefer allow-lists. Internal pages must not use inline event handlers.
- Update `CHANGELOG.md` under **Unreleased** for user-visible changes.
- Never commit secrets, profile folders, `config.json`, `vault.json` or `store.json`.

## Reporting bugs

Open an issue with your Windows version, the Owl version (from the installer name or release you used) and steps to reproduce.
For security problems, follow [SECURITY.md](SECURITY.md) instead.

## Code style

Standard Rust formatting (`cargo fmt`). Comments explain why, not what.
