# Privacy policy

Owl is a desktop browser that runs on your PC. It has no account, no server of its own and no
telemetry. This page says what Owl stores, what it sends and to whom. It is also shown inside the
browser at `owl://privacy`.

## What Owl collects about you

Nothing. The author of Owl receives no data from your copy of Owl: no usage statistics, no crash
reports, no identifiers, no analytics.

## What Owl stores on your PC

Everything is kept in `%LOCALAPPDATA%\Owl` on your own PC:

- **Your profile**: the name and email address you type at first run. They never leave your PC and
  are only used to greet you.
- **History, bookmarks, open tabs, download list and time per site**: encrypted with a key derived
  from your passphrase.
- **Saved passwords**: encrypted and bound to your passphrase, the site and the user name.
- **Cookies, logins, site storage and cache** (kept by the WebView2 engine): sealed into one
  encrypted file when Owl locks, closes, crashes or Windows shuts down.
- **OwlKick activity** (what was blocked): kept in memory only and cleared when Owl closes.
- **Settings**: a plain `config.json` file, with no browsing data in it.

Private windows save none of the above.

## What Owl sends, and only when

Owl talks to the sites you open, like any browser. Beyond that, these are the only connections it
makes:

| Feature | Goes to | When |
|---|---|---|
| Update check | GitHub (`api.github.com`) | Once a day, only if you turn it on |
| OwlKick filter lists | The EasyList project's servers | Only when you download or refresh the lists |
| Secure DNS | Cloudflare (`1.1.1.1`) | Every site-name lookup, only while Secure DNS is on |
| Search | DuckDuckGo | When you search from the address bar or home page |
| BitTorrent | Other peers in the torrent swarm | Only while a torrent you added is active |
| Cloudflare WARP | Cloudflare, through its own installed app | Only if you installed WARP and connect it |
| Proxy | The proxy you entered | All page traffic, only if you set one |

Pages are rendered by Microsoft Edge WebView2. Microsoft's WebView2 runtime updates itself and
keeps Microsoft Defender SmartScreen on, which checks the reputation of sites and downloads with
Microsoft. Owl keeps SmartScreen on deliberately, because it protects against phishing and malware.
Microsoft's privacy terms apply to those checks.

## Third parties

When you use Secure DNS, Cloudflare's resolver privacy policy applies to your lookups. When you
download filter lists, the list host sees your IP address. When you use a torrent, the other peers
see your IP address. Owl does not share anything with any of them beyond what the connection itself
needs.

## Your control

- Delete `%LOCALAPPDATA%\Owl` to remove everything Owl stored. The uninstaller offers to do this.
- Clear history, bookmarks or passwords from their pages inside Owl at any time.
- Every network feature in the table above has its own switch in Settings.

## Changes

Changes to this policy are listed in the [changelog](CHANGELOG.md) of the release that makes them.

## Contact

Questions go to the [issue tracker](https://github.com/fardinahosancse/owl-browser/issues). For a
security problem, follow [SECURITY.md](SECURITY.md) instead of opening a public issue.

_Last updated: 13 November 2026._
