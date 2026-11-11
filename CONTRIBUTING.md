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

## Issue-first workflow

Every bug, weakness or task is tracked as a GitHub issue **before** work starts on it.

1. **File it first.** Give it a priority label and a clear description (what, impact, suggested fix):

   | Label | Meaning |
   |---|---|
   | `P0-critical` | Data loss, exploitable flaw or broken release: drop everything |
   | `P1-high` | Fix in the next release |
   | `P2-medium` | Should be fixed soon |
   | `P3-low` | Nice to have or minor |

   Add a type label too (`bug`, `security`, `privacy`, `robustness`, `tests`, `enhancement`,
   `documentation`, `chore`). Security problems that can be exploited today go through
   [SECURITY.md](SECURITY.md) (private report), not a public issue.
2. **Work on it** in a branch or pull request that references the issue.
3. **Resolve it.** Merge with `Fixes #123` in the commit or PR so GitHub closes it as completed,
   and note the fixing commit in the issue.
4. **Release notes pick it up automatically.** The release workflow lists every issue closed as
   completed since the previous release, grouped by priority, under **Resolved issues**. Also add a
   line to `CHANGELOG.md` for user-visible changes.

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
