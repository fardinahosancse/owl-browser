# Installation

## Requirements

- Windows 10 (1809 or later) or Windows 11, 64-bit.
- Microsoft Edge WebView2 Runtime. It is included with Windows 11 and current Windows 10. The installer checks for it and links to the official download if it is missing.

## Installer (recommended)

1. Open the [latest release](https://github.com/fardinahosancse/owl-browser/releases/latest).
2. Download `Owl-Setup-<version>.exe` and run it.
3. Start **Owl** from the Start menu.

The installer is per-user (no administrator rights needed), adds Start menu and optional desktop shortcuts, and offers to remove your data on uninstall (it keeps it by default).

Windows SmartScreen may warn on first run because the files are not yet Authenticode-signed. See [Releases and Verification](Releases-and-Verification) to check that a download really came from this repository.

## Portable

Download `Owl-<version>-portable.zip`, extract it anywhere and run `Owl.exe`.

## Updating

Owl never updates itself. Turn on **Settings > Privacy > Check for new versions** to be told when a newer release exists, or use **Check now** in Settings. Then download and run the new installer over the old one; your data is kept.

## Uninstalling

Use Windows **Apps > Installed apps**. Choose whether to keep or delete your data when asked.
