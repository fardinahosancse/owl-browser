# Data and Files

Everything is local, under `%LOCALAPPDATA%\Owl`. Open it from **Settings > Open Owl folder**.

| Path | Contents |
|---|---|
| `config.json` (+ `config.bak`) | Settings, wrapped keys, key-derivation salt. Contains no passwords. |
| `store.json` | History, bookmarks, session, downloads and statistics. **Encrypted.** |
| `vault.json` (+ `vault.bak`) | Saved passwords, each encrypted with Windows DPAPI plus a secret derived from your passphrase. |
| `profile\` | The WebView2 profile. While Owl is locked or closed, sensitive files are moved into `profile\sealed.owl`. |
| `chrome\` | Engine data for Owl's own interface. |
| `extensions\` | Optional user scripts (`*.js`). |

## Backups

Copy the whole `Owl` folder while Owl is closed. You also need your passphrase.

## Removing everything

Close Owl and delete the `Owl` folder. The uninstaller can do this for you.
