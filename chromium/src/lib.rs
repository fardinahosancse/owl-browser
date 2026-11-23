//! Owl. on Chromium (CEF): an experiment.
//!
//! The shipping Owl. renders with Microsoft Edge WebView2 and draws its own tabs, address bar,
//! download manager, password manager and dialogs. This build does the opposite: it opens
//! Chromium's own browser window (CEF "Chrome style"), so tabs, the address bar, downloads,
//! passwords, history, bookmarks, find in page, dialogs and settings are Chromium's built-in ones.
//! Owl. adds what is its own on top:
//!
//! - the Owl. look: an Owl. Chrome theme (white, paper grey, ink) with Owl.'s red as the accent,
//!   Chrome extras Owl. does not use hidden, and the Owl. icon on the window (see `chrome.rs`);
//! - Owl. Calm Home, the same extension as chrome-extension-homepage, as the new tab and home page;
//! - OwlKick: every request is checked with adblock-rust against Owl.'s tracker list and any
//!   EasyList/EasyPrivacy files in `%LOCALAPPDATA%\Owl\filters`, and blocked before it leaves;
//! - Secure DNS: Chromium's own DNS over HTTPS, pointed at Cloudflare 1.1.1.1 in secure mode.
//!
//! Chromium runs several processes (browser, renderer, GPU, network). They are all this same
//! executable started again with a `--type=...` switch; `execute_process` sends those to their job.
//!
//! With the `sandbox` feature (the default) this crate is a DLL: CEF's `bootstrap.exe`, shipped as
//! `owl-chromium.exe`, loads it and calls `RunWinMain` with the sandbox information, so renderer
//! and other child processes run inside Chromium's sandbox. Without the feature, `src/main.rs`
//! runs the same code unsandboxed, which is handy while developing.

mod rules;
mod chrome;
#[cfg(all(target_os = "windows", feature = "sandbox"))]
mod win;

use adblock::{lists::ParseOptions, request::Request as KickRequest, Engine, FilterSet};
use cef::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

/// Owl.'s brand red, the seed Chromium builds its colours from (Settings > Appearance > Theme).
const OWL_RED: u32 = 0xFFB8_290F;
const CLOUDFLARE_DOH: &str = "https://cloudflare-dns.com/dns-query";

/// Requests OwlKick blocked since start.
pub(crate) static BLOCKED: AtomicU64 = AtomicU64::new(0);

fn owl_dir() -> std::path::PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    std::path::Path::new(&base).join("Owl")
}

/// %LOCALAPPDATA%\Owl\chromium: its own Chromium profile, apart from the WebView2 build's data.
pub(crate) fn profile_dir() -> String {
    let dir = owl_dir().join("chromium");
    let _ = std::fs::create_dir_all(&dir);
    dir.to_string_lossy().into_owned()
}

// ---------- OwlKick ----------

thread_local! {
    // adblock-rust engines are single-threaded; Chromium asks from its IO thread, each thread gets one
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
}

fn build_engine() -> Engine {
    let mut base = String::new();
    for h in rules::BLOCKED_HOSTS {
        base.push_str(&format!("||{h}^\n"));
    }
    for p in rules::BLOCKED_PATHS {
        if p.starts_with('/') {
            base.push_str(&format!("{p}\n"));
        } else {
            base.push_str(&format!("||{p}\n"));
        }
    }
    let mut set = FilterSet::new(false);
    set.add_filter_list(base, ParseOptions::default());
    // the same downloaded lists the WebView2 build uses
    if let Ok(rd) = std::fs::read_dir(owl_dir().join("filters")) {
        for e in rd.flatten().filter(|e| e.path().extension().map_or(false, |x| x == "txt")) {
            if let Ok(t) = std::fs::read_to_string(e.path()) {
                set.add_filter_list(t, ParseOptions::default());
            }
        }
    }
    Engine::new_with_filter_set(set)
}

fn kind_of(t: ResourceType) -> &'static str {
    if t == ResourceType::MAIN_FRAME {
        "document"
    } else if t == ResourceType::SUB_FRAME {
        "subdocument"
    } else if t == ResourceType::STYLESHEET {
        "stylesheet"
    } else if t == ResourceType::SCRIPT {
        "script"
    } else if t == ResourceType::IMAGE || t == ResourceType::FAVICON {
        "image"
    } else if t == ResourceType::FONT_RESOURCE {
        "font"
    } else if t == ResourceType::MEDIA {
        "media"
    } else if t == ResourceType::XHR {
        "xmlhttprequest"
    } else if t == ResourceType::PING || t == ResourceType::CSP_REPORT {
        "ping"
    } else {
        "other"
    }
}

fn should_block(url: &str, source: &str, kind: &str) -> bool {
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return false;
    }
    ENGINE.with(|cell| {
        let mut e = cell.borrow_mut();
        let eng = e.get_or_insert_with(build_engine);
        match KickRequest::new(url, if source.is_empty() { url } else { source }, kind, "GET") {
            Ok(r) => {
                let res = eng.check_network_request(&r);
                res.important || (res.filter.is_some() && res.exception.is_none())
            }
            Err(_) => false,
        }
    })
}

wrap_resource_request_handler! {
    struct KickHandler {
        enabled: bool,
    }

    impl ResourceRequestHandler {
        fn on_before_resource_load(
            &self,
            _browser: Option<&mut Browser>,
            frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _callback: Option<&mut Callback>,
        ) -> ReturnValue {
            let Some(request) = request else { return ReturnValue::CONTINUE };
            if !self.enabled {
                return ReturnValue::CONTINUE;
            }
            let kind = kind_of(request.resource_type());
            if kind == "document" {
                return ReturnValue::CONTINUE; // never block the page itself
            }
            let url = CefString::from(&request.url()).to_string();
            let source = match frame {
                Some(f) => CefString::from(&f.url()).to_string(),
                None => CefString::from(&request.referrer_url()).to_string(),
            };
            if should_block(&url, &source, kind) {
                BLOCKED.fetch_add(1, Relaxed);
                return ReturnValue::CANCEL;
            }
            ReturnValue::CONTINUE
        }
    }
}

wrap_request_handler! {
    struct OwlRequestHandler {
        kick: bool,
    }

    impl RequestHandler {
        fn resource_request_handler(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _request: Option<&mut Request>,
            _is_navigation: ::std::os::raw::c_int,
            _is_download: ::std::os::raw::c_int,
            _request_initiator: Option<&CefString>,
            _disable_default_handling: Option<&mut ::std::os::raw::c_int>,
        ) -> Option<ResourceRequestHandler> {
            Some(KickHandler::new(self.kick))
        }
    }
}

fn value_of(set: impl FnOnce(&Value)) -> Value {
    let v = value_create().expect("value");
    set(&v);
    v
}

/// Owl.'s choices on top of Chromium's own settings: one light colour scheme, Secure DNS.
fn apply_owl_settings() {
    let s = |x: &str| value_of(|v| {
        v.set_string(Some(&CefString::from(x)));
    });
    let b = |x: bool| value_of(|v| {
        v.set_bool(x as i32);
    });
    // Secure DNS: Chromium's built-in DNS over HTTPS, Cloudflare, secure mode (no fallback to the
    // network's resolver). These are browser-wide (local state) preferences.
    if let Some(global) = preference_manager_get_global() {
        for (name, mut value) in [("dns_over_https.mode", s("secure")), ("dns_over_https.templates", s(CLOUDFLARE_DOH))] {
            let mut err = CefString::from("");
            if global.set_preference(Some(&CefString::from(name)), Some(&mut value), Some(&mut err)) != 1 {
                eprintln!("preference {name}: {err}");
            }
        }
    }
    // the rest belong to the profile
    if let Some(ctx) = request_context_get_global_context() {
        // Owl. has one theme, light: Chromium's own pages and controls follow it
        ctx.set_chrome_color_scheme(ColorVariant::LIGHT, OWL_RED);
        for (name, mut value) in [
            ("signin.allowed", b(false)),
            // the Home button opens Owl. Calm Home (the new tab page)
            ("browser.show_home_button", b(true)),
            ("homepage_is_newtabpage", b(true)),
        ] {
            let mut err = CefString::from("");
            if ctx.set_preference(Some(&CefString::from(name)), Some(&mut value), Some(&mut err)) != 1 {
                eprintln!("preference {name}: {err}");
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
            apply_owl_settings();
            chrome::accept_home_takeover();
            *self.client.borrow_mut() = Some(chrome::new_client());
            // --url=... opens that address; otherwise the new tab page, Owl. Calm Home
            let cmd = command_line_get_global().expect("no command line");
            let url = CefString::from(&cmd.switch_value(Some(&CefString::from("url")))).to_string();
            chrome::open(&mut self.client.borrow().clone(), &url);
        }

        fn default_client(&self) -> Option<Client> {
            // tabs and windows Chromium opens itself use this client too, so OwlKick covers them
            self.client.borrow().clone()
        }
    }
}

wrap_app! {
    struct OwlApp;

    impl App {
        fn on_before_command_line_processing(&self, process_type: Option<&CefString>, command_line: Option<&mut CommandLine>) {
            if process_type.map(|p| p.to_string()).unwrap_or_default().is_empty() {
                if let Some(cl) = command_line {
                    cl.append_switch(Some(&CefString::from("hide-crash-restore-bubble")));
                    // Owl. Calm Home and the Owl. theme, from the folders next to the exe
                    let ext = chrome::owl_extensions();
                    if !ext.is_empty() {
                        cl.append_switch_with_value(Some(&CefString::from("load-extension")), Some(&CefString::from(ext.as_str())));
                        cl.append_switch_with_value(
                            Some(&CefString::from("disable-features")),
                            Some(&CefString::from("DisableLoadExtensionCommandLineSwitch")),
                        );
                    }
                    trace(&format!("extensions: {ext}"));
                }
            }
        }

        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            Some(OwlBrowserProcessHandler::new(RefCell::new(None)))
        }
    }
}

/// Start-up trace for diagnosing, only when OWL_CEF_TRACE names a file.
pub fn trace(msg: &str) {
    if let Ok(p) = std::env::var("OWL_CEF_TRACE") {
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "[{}] {msg}", std::process::id());
        }
    }
}

/// Starts the browser process, or runs a child process's job. `sandbox_info` comes from
/// `bootstrap.exe` (sandboxed) or is null (development build without the sandbox).
pub fn run(main_args: &MainArgs, cmd_line: &CommandLine, sandbox_info: *mut u8) -> i32 {
    trace("run: start");
    // a Rust panic anywhere is written to the trace (the process aborts right after)
    std::panic::set_hook(Box::new(|info| trace(&format!("PANIC: {info}"))));
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    trace("run: api_hash done");
    // renderer, GPU and other child processes run their job and return here
    let is_browser = cmd_line.has_switch(Some(&CefString::from("type"))) != 1;
    trace(&format!("run: execute_process (browser={is_browser}, sandbox={})", !sandbox_info.is_null()));
    let code = execute_process(Some(main_args), None, sandbox_info);
    trace(&format!("run: execute_process returned {code}"));
    if !is_browser {
        return code.max(0);
    }

    let root = profile_dir();
    let settings = Settings {
        no_sandbox: sandbox_info.is_null() as _,
        root_cache_path: CefString::from(root.as_str()),
        cache_path: CefString::from(format!(r"{root}\Default").as_str()),
        persist_session_cookies: 1,
        ..Default::default()
    };
    let mut app = OwlApp::new();
    trace("run: initialize");
    if initialize(Some(main_args), Some(&settings), Some(&mut app), sandbox_info) != 1 {
        eprintln!("Chromium (CEF) could not start");
        return 1;
    }
    run_message_loop();
    shutdown();
    eprintln!("OwlKick blocked {} requests", BLOCKED.load(Relaxed));
    0
}
