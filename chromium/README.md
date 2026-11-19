# Owl. on Chromium (CEF): experiment

This folder is an experiment on the `chromium-cef` branch: Owl. built on the **Chromium Embedded
Framework** (CEF, Chromium 156 through the [`cef`](https://crates.io/crates/cef) crate from the
Tauri project) instead of **Microsoft Edge WebView2**. The shipping browser is unchanged.

**Approach on this branch: use what Chromium already has, keep what is Owl.'s own.**

Owl. on WebView2 rebuilt many browser features itself because WebView2 is only an engine. Chromium
(CEF "Chrome style") comes with the full browser UI, so this build uses Chromium's built-in tabs,
address bar, downloads, passwords, history, bookmarks, find in page, dialogs, permissions and
settings, and does not port Owl.'s versions of them. Owl. keeps:

- **The Owl. look**: Chromium's theme is seeded with Owl.'s red (#B8290F), the window has the Owl. icon.
- **The same home page**: `home.html` is Owl.'s classic home page, generated from Owl.'s own page
  code (`dump_home_for_chromium` test, then `make-home.py`), set as Chromium's home page and start page.
- **OwlKick**: adblock-rust checks every request against Owl.'s tracker list and any lists in
  `%LOCALAPPDATA%\Owlilters`, in Chromium's request handler, for every tab including ones the
  user opens. `OWL_KICK_OFF=1` turns it off for testing.
- **Secure DNS**: Chromium's own DNS over HTTPS, set to Cloudflare (`cloudflare-dns.com/dns-query`)
  in secure mode, so lookups never fall back to the network's resolver.

Its profile lives in `%LOCALAPPDATA%\Owl\chromium`. CI builds it, launches it, takes a screenshot
and uploads both as artifacts.

## Good to know first

WebView2 is already Chromium: it is Microsoft Edge's build of Chromium, shipped and patched by
Windows Update. So pages already render exactly as in Chrome. What a CEF build changes:

| | WebView2 (today) | CEF (this experiment) |
|---|---|---|
| Engine | Chromium, Edge build | Chromium, CEF build |
| Download size | Owl.exe about 7 MB | about 150 MB unpacked, roughly 70 MB installer |
| Security updates | Microsoft, automatic, every few days | **Owl. must ship a new build for every Chromium security release** (about every 1 to 4 weeks) |
| Renderer sandbox | on, managed by Edge | needs CEF's `bootstrap.exe` setup; **off in this first step** |
| Google/Edge services | Edge's SmartScreen | none (no Safe Browsing unless added) |
| Runs on | Windows 10 and 11 | Windows 10 and 11, and later macOS and Linux from the same code |
| Custom `owl://` scheme | emulated over `http://owl.localhost` | real scheme through `CefSchemeHandlerFactory` |

The main reasons to switch would be one engine on Windows, macOS and Linux (#91) and full control
over the engine. The main costs are size and the duty to ship Chromium security fixes on time.

## Building

Needs the MSVC Rust toolchain, Visual Studio Build Tools (C++), and CMake. The first build
downloads the CEF binary distribution for the pinned version.

```bash
cd chromium
cargo build --release
```

Copy `libcef.dll` and the other files from the CEF `Release` and `Resources` folders next to
`target\release\owl-chromium.exe` (the CI workflow `.github/workflows/chromium.yml` does this).

## Port map: what each Owl. feature uses now, and its CEF counterpart

| Owl. feature | WebView2 / wry today | CEF |
|---|---|---|
| Window, tabs, toolbar | tao window, one wry webview per tab plus the chrome webview | CEF Views: `Window`, one `BrowserView` per tab; the chrome stays HTML in its own `BrowserView` |
| Owl.'s pages (`owl://`) | custom protocol on `http://owl.localhost` | `register_scheme_handler_factory("owl", ...)` |
| Chrome to app messages | `window.ipc.postMessage` | `CefMessageRouter` or process messages (`send_process_message`) |
| OwlKick (adblock-rust) | `WebResourceRequested` filter | `ResourceRequestHandler::on_before_resource_load` (cancel) |
| Downloads, fast engine, torrents | `DownloadStarting` event | `DownloadHandler::on_before_download` / `can_download` |
| Data saver (Save-Data, image shrink) | request headers, `WebResourceRequested` | `on_before_resource_load` headers, `ResponseFilter` |
| Secure DNS | loopback DoH proxy via `--proxy-server` | same proxy, or Chromium's own DoH through preferences |
| Site information, certificate | DevTools `Network.getCertificate` | `NavigationEntry::ssl_status` then `X509Certificate` directly |
| Cookies, Clear site data | `CookieManager`, DevTools `Storage.clearDataForOrigin` | `CookieManager`, `execute_dev_tools_method` |
| Permissions | `PermissionRequested`, profile permission API | `PermissionHandler` |
| Dialogs (alert, confirm, prompt) | `ScriptDialogOpening` | `JsDialogHandler` |
| Passwords capture and fill | injected scripts plus IPC | same scripts through `on_context_created` / process messages |
| Lock, sealing the profile | WebView2 user data folder | CEF `root_cache_path`, same sealing after `shutdown()` |
| Theme colour before paint | `DefaultBackgroundColor` | `BrowserSettings::background_color` |
| Window dragging | `app-region: drag` (non-client regions) | `DragHandler::on_draggable_regions_changed` |

## Next steps, in order

1. Turn on the renderer sandbox (`bootstrap.exe` layout from CEF).
2. Serve Owl.'s pages through an `owl://` scheme handler and host the HTML toolbar.
3. Message bridge between the toolbar and Rust, then tabs.
4. Port the features in the table one by one, keeping `src/` shared where it does not touch the engine (crypto, vault, store, OwlKick lists, download engine, torrents).
5. An installer that carries the CEF runtime, and a release job that follows Chromium's security releases.
