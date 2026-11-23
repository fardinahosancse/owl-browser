# Owl. on Chromium (CEF): experiment

This folder is an experiment on the `chromium-cef` branch: Owl. built on the **Chromium Embedded
Framework** (CEF, Chromium 156 through the [`cef`](https://crates.io/crates/cef) crate from the
Tauri project) instead of **Microsoft Edge WebView2**. The shipping browser is unchanged.

**Approach on this branch: Chromium's own interface, in Owl.'s skin.**

The window is Chromium's real tabbed browser (CEF "Chrome style"): its tabs, address bar, menus,
downloads, passwords, history, bookmarks, settings, `chrome://` pages and extensions. Owl. changes
how it looks and what it shows (`src/chrome.rs`):

- the **Owl. theme** (`theme/`, a Chrome theme): white toolbar and tabs on paper grey, ink text,
  Owl.'s red as the accent Chromium builds its own colours from; one theme, light;
- **Owl. Calm Home** (`chrome-extension-homepage/`, loaded as the real extension) as the new tab and
  home page, with its time online, reports, favourites, shortcuts and weather; Owl. keeps it light
  and drops its Theme row;
- Chrome extras hidden: the profile avatar (sign-in is off), media and battery saver buttons, and
  Google's Lens, AI, shopping and translate page actions;
- **OwlKick** on every request of every tab, **Secure DNS** (Cloudflare over HTTPS, secure mode), the
  renderer **sandbox**, and the Owl. icon and name.

The shapes of the tabs, toolbar and address bar stay Chromium's: changing those needs a Chromium
source fork. Wi-Fi and volume on Calm Home need the Windows helper registered for Chromium, not yet
done.

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

2. Serve Owl.'s pages through an `owl://` scheme handler and host the HTML toolbar.
3. Message bridge between the toolbar and Rust, then tabs.
4. Port the features in the table one by one, keeping `src/` shared where it does not touch the engine (crypto, vault, store, OwlKick lists, download engine, torrents).
5. An installer that carries the CEF runtime, and a release job that follows Chromium's security releases.
