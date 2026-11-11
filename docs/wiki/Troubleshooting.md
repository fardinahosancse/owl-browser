# Troubleshooting

## Owl does not open

- Make sure the **Edge WebView2 Runtime** is installed (it ships with Windows 11 and current Windows 10). Download it from Microsoft if missing.
- If Owl shows a message box at start-up, read it: it names the problem.
- Versions before 0.1.1 failed silently when installed in a read-only folder such as Program Files. Update to the latest release.
- Run `Owl.exe --private` to test with a throwaway profile.

## I forgot my passphrase

There is no reset, by design: your data is encrypted with it. You can start fresh by closing Owl and deleting `%LOCALAPPDATA%\Owl` (this erases history, bookmarks and passwords).

## Recovery screen ("settings could not be read")

Owl found a damaged `config.json` and refused to start over. Choose **Open Owl folder**, copy `config.bak` over `config.json` while Owl is closed, and start again.

## A site does not work

- Turn off **Strict tracking prevention** or **Block trackers** for testing.
- Turn off **Strict fingerprint protection** if a site shows the wrong language or layout.
- Turn off the permission block if a site needs camera, microphone or location.

## Windows SmartScreen warning

The files are not yet Authenticode-signed. Verify the download as described in [Releases and Verification](Releases-and-Verification), then choose **More info > Run anyway**.

## Report a bug

Search the [issues](https://github.com/fardinahosancse/owl-browser/issues) first, then open a new one with your Owl version and steps to reproduce. For security problems, do not open a public issue; see [Security Model](Security-Model).
