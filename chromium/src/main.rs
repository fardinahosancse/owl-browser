//! Owl. on Chromium (CEF): an experiment.
//!
//! The shipping Owl. renders with Microsoft Edge WebView2 (itself Chromium, built and updated by
//! Microsoft). This program instead embeds Chromium directly through the Chromium Embedded
//! Framework, via the `cef` crate from the Tauri project. It opens one window with Owl.'s name,
//! icon and its own Chromium profile, and loads a page. It is the first step of the port mapped
//! out in README.md, not a replacement for the browser yet.
//!
//! Chromium runs several processes (browser, renderer, GPU, network). They are all this same
//! executable started again with a `--type=...` switch; `execute_process` sends those to their job.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use cef::*;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

const WORDMARK: &str = include_str!("../../assets/wordmark.d");

/// Owl.'s own start page, as a data: address (no network, nothing on disk).
fn start_page() -> String {
    let html = format!(
        r#"<!doctype html><meta charset=utf-8><title>Owl. on Chromium</title>
<style>html,body{{margin:0;height:100%;background:#efe9dc;color:#171717;font:16px/1.5 "Segoe UI",Helvetica,Arial,sans-serif}}
main{{display:grid;place-content:center;height:100%;gap:22px;padding:24px;text-align:center}}
svg{{width:min(70vw,520px);height:auto;fill:#B8290F}}
p{{margin:0;max-width:44ch}}b{{letter-spacing:.12em;text-transform:uppercase;font-size:11px;color:#B8290F}}
form{{display:flex;gap:8px;justify-content:center}}input{{width:min(60vw,440px);font:inherit;padding:10px 12px;border:1px solid #171717;background:#fff}}
button{{font:700 11px "Segoe UI",sans-serif;letter-spacing:.14em;text-transform:uppercase;border:0;background:#171717;color:#fff;padding:0 18px;cursor:pointer}}</style>
<main><svg viewBox="0 0 508.1 217" role="img" aria-label="Owl."><path d="{WORDMARK}"/></svg>
<b>Chromium preview</b><p>This window is Chromium itself, embedded through CEF, instead of Microsoft Edge WebView2.</p>
<form onsubmit="event.preventDefault();var v=this.u.value.trim();if(v)location.href=/^[a-z]+:/i.test(v)?v:(v.includes('.')&&!v.includes(' ')?'https://'+v:'https://duckduckgo.com/?q='+encodeURIComponent(v))">
<input name=u placeholder="Search or enter an address" autofocus><button>Go</button></form></main>"#,
        WORDMARK = WORDMARK.trim()
    );
    let data = CefString::from(&base64_encode(Some(html.as_bytes())));
    let uri = CefString::from(&uriencode(Some(&data), 0)).to_string();
    format!("data:text/html;base64,{uri}")
}

/// %LOCALAPPDATA%\Owl\chromium: its own profile, apart from the WebView2 build's data.
fn profile_dir() -> String {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    let dir = std::path::Path::new(&base).join("Owl").join("chromium");
    let _ = std::fs::create_dir_all(&dir);
    dir.to_string_lossy().into_owned()
}

/// Browsers that are open; when the last one closes the message loop ends.
struct Browsers {
    list: Vec<Browser>,
}

wrap_window_delegate! {
    struct OwlWindowDelegate {
        browser_view: RefCell<Option<BrowserView>>,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1280, height: 800 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            let browser_view = self.browser_view.borrow();
            let (Some(window), Some(browser_view)) = (window, browser_view.as_ref()) else {
                return;
            };
            let mut view = View::from(browser_view);
            window.add_child_view(Some(&mut view));
            window.set_title(Some(&CefString::from("Owl. (Chromium preview)")));
            window.show();
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            *self.browser_view.borrow_mut() = None;
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            let browser_view = self.browser_view.borrow();
            match browser_view.as_ref().and_then(|v| v.browser()) {
                Some(browser) => browser.host().map(|h| h.try_close_browser()).unwrap_or(1),
                None => 1,
            }
        }

        fn initial_show_state(&self, _window: Option<&mut Window>) -> ShowState {
            ShowState::NORMAL
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::DEFAULT
        }
    }
}

wrap_browser_view_delegate! {
    struct OwlBrowserViewDelegate {
        runtime_style: RuntimeStyle,
    }

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn on_popup_browser_view_created(
            &self,
            _browser_view: Option<&mut BrowserView>,
            popup_browser_view: Option<&mut BrowserView>,
            _is_devtools: i32,
        ) -> i32 {
            // a popup (window.open, target=_blank) gets its own Owl. window
            let mut delegate = OwlWindowDelegate::new(RefCell::new(popup_browser_view.cloned()));
            window_create_top_level(Some(&mut delegate));
            1
        }

        fn browser_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }
    }
}

wrap_client! {
    struct OwlClient {
        browsers: Arc<Mutex<Browsers>>,
    }

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(OwlDisplayHandler::new("Owl.".to_string()))
        }

        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(OwlLifeSpanHandler::new(self.browsers.clone()))
        }
    }
}

wrap_display_handler! {
    struct OwlDisplayHandler {
        // appended to every window title
        suffix: String,
    }

    impl DisplayHandler {
        fn on_title_change(&self, browser: Option<&mut Browser>, title: Option<&CefString>) {
            // "Page title - Owl." in the window title
            let mut browser = browser.cloned();
            if let Some(window) = browser_view_get_for_browser(browser.as_mut()).and_then(|v| v.window()) {
                let t = title.map(CefString::to_string).unwrap_or_default();
                let t = if t.is_empty() { self.suffix.clone() } else { format!("{t} - {}", self.suffix) };
                window.set_title(Some(&CefString::from(t.as_str())));
            }
        }
    }
}

wrap_life_span_handler! {
    struct OwlLifeSpanHandler {
        browsers: Arc<Mutex<Browsers>>,
    }

    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            if let Some(b) = browser.cloned() {
                self.browsers.lock().unwrap().list.push(b);
            }
        }

        fn do_close(&self, _browser: Option<&mut Browser>) -> i32 {
            0
        }

        fn on_before_close(&self, browser: Option<&mut Browser>) {
            let mut all = self.browsers.lock().unwrap();
            if let Some(mut b) = browser.cloned() {
                if let Some(i) = all.list.iter().position(|x| x.is_same(Some(&mut b)) != 0) {
                    all.list.remove(i);
                }
            }
            if all.list.is_empty() {
                quit_message_loop();
            }
        }
    }
}

wrap_browser_process_handler! {
    struct OwlBrowserProcessHandler {
        client: RefCell<Option<Client>>,
    }

    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            *self.client.borrow_mut() = Some(OwlClient::new(Arc::new(Mutex::new(Browsers { list: vec![] }))));

            // --url=... opens that address; otherwise Owl.'s start page
            let cmd = command_line_get_global().expect("no command line");
            let url = CefString::from(&cmd.switch_value(Some(&CefString::from("url")))).to_string();
            let url = if url.is_empty() { start_page() } else { url };
            let url = CefString::from(url.as_str());

            let settings = BrowserSettings::default();
            let mut client = self.default_client();
            let mut delegate = OwlBrowserViewDelegate::new(RuntimeStyle::DEFAULT);
            let browser_view = browser_view_create(client.as_mut(), Some(&url), Some(&settings), None, None, Some(&mut delegate));
            let mut window = OwlWindowDelegate::new(RefCell::new(browser_view));
            window_create_top_level(Some(&mut window));
        }

        fn default_client(&self) -> Option<Client> {
            self.client.borrow().clone()
        }
    }
}

wrap_app! {
    struct OwlApp;

    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(OwlBrowserProcessHandler::new(RefCell::new(None)))
        }
    }
}

fn main() {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();
    let Some(cmd_line) = args.as_cmd_line() else { return };

    // renderer, GPU and other helper processes run their job and return here
    let is_browser = cmd_line.has_switch(Some(&CefString::from("type"))) != 1;
    let code = execute_process(Some(args.as_main_args()), None, std::ptr::null_mut());
    if !is_browser {
        std::process::exit(code.max(0));
    }

    let root = profile_dir();
    let settings = Settings {
        no_sandbox: 1,
        root_cache_path: CefString::from(root.as_str()),
        cache_path: CefString::from(format!("{root}\\Default").as_str()),
        persist_session_cookies: 1,
        ..Default::default()
    };
    let mut app = OwlApp::new();
    if initialize(Some(args.as_main_args()), Some(&settings), Some(&mut app), std::ptr::null_mut()) != 1 {
        eprintln!("Chromium (CEF) could not start");
        std::process::exit(1);
    }
    run_message_loop();
    shutdown();
}
