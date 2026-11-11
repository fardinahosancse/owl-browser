# Security Policy

## Supported versions

Only the latest release receives security fixes.

## Reporting a vulnerability

Please **do not open a public issue** for security problems.

Report privately through GitHub: **Security > Report a vulnerability** on this repository
(private vulnerability reporting). Include:

- what you found and where (file and line if possible),
- steps to reproduce or a proof of concept,
- the impact you expect.

You will get an acknowledgement as soon as possible, and a fix or a clear explanation afterwards.
Please give a reasonable time to fix a problem before disclosing it publicly.

## Scope

In scope: the Owl executable and its installer, the internal `owl://` pages, the command channel
between pages and the browser, local encryption (vault, store, sealed profile), lock and recovery
logic, download handling, and the release workflow.

Out of scope: vulnerabilities in Microsoft Edge WebView2 itself (report to Microsoft), malware that
already runs as your Windows user and reads process memory, physical attacks on an unlocked and
running session, and social-engineering of the user.

## Design notes and known limits

Owl is a small hobby-grade project and has not been professionally audited. Be aware of:

- While Owl is **open and unlocked**, other programs running as the same Windows user can read its
  files and memory. Encryption at rest protects data while Owl is locked, closed or after a crash.
- Data is sealed to a public key stored in `config.json`; anyone who can write to your profile
  folder can also forge a sealed file. Sealing provides confidentiality, not tamper resistance.
- The passphrase is the root of trust. Offline guessing is slowed by Argon2id but not prevented.
- The WebView2 cache and some Chromium metadata files are not part of the sealed set.
- Fingerprint protection is best-effort and can be detected.

## Secrets in this repository

The repository contains no credentials. A secret scan runs on every push. If you find a secret,
report it as above.
