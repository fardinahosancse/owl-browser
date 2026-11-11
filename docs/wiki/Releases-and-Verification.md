# Releases and Verification

## How a release is made

1. Bump `version` in `Cargo.toml` (and `Cargo.lock`) and add a `## [x.y.z]` section to `CHANGELOG.md`.
2. Commit, tag `vx.y.z` and push the tag.
3. The Release workflow builds with `cargo build --release --locked`, compiles the installer with Inno Setup, makes the portable zip, checksums, a build log and build attestations, and publishes notes from the changelog plus an automatic **Resolved issues** list.

The workflow refuses to publish if the tag and the `Cargo.toml` version disagree.

## Each release contains

- `Owl-Setup-<version>.exe`
- `Owl-<version>-portable.zip`
- `SHA256SUMS.txt`
- `BUILD-LOG.txt` (commit, toolchain, sizes, hashes, link to the build run)

## Verify a download

Checksum:

```powershell
Get-FileHash .\Owl-Setup-<version>.exe -Algorithm SHA256
```

Build attestation (proves it was built from this repository by the release workflow):

```powershell
gh attestation verify .\Owl-Setup-<version>.exe --repo fardinahosancse/owl-browser
```

## Supply-chain controls

- GitHub Actions pinned to commit SHAs, updated by Dependabot.
- CI: secret scan, dependency advisory audit, licence and sources check (`cargo-deny`).
- `Cargo.lock` is committed and every build uses `--locked`.

All releases: https://github.com/fardinahosancse/owl-browser/releases
