//! Owl. on Chromium (CEF): an experiment.
//!
//! The shipping Owl. renders with Microsoft Edge WebView2 and draws its own tabs, address bar,
//! download manager, password manager and dialogs. This build does the opposite: it opens
//! Chromium's own browser window (CEF "Chrome style"), so tabs, the address bar, downloads,
//! passwords, history, bookmarks, find in page, dialogs and settings are Chromium's built-in ones.
//! Owl. adds what is its own on top:
//!
//! - the Owl. look: Chromium's theme is seeded with Owl.'s red, and the window carries the Owl. icon;
//! - the same home page as Owl. (`home.html`, generated from Owl.'s page code by `make-home.py`);
//! - OwlKick: every request is checked with adblock-rust against Owl.'s tracker list and any
//!   EasyList/EasyPrivacy files in `%LOCALAPPDATA%\Owl\filters`, and blocked before it leaves;
//! - Secure DNS: Chromium's own DNS over HTTPS, pointed at Cloudflare 1.1.1.1 in secure mode.
//!
//! Chromium runs several processes (browser, renderer, GPU, network). They are all this same
//! executable started again with a `--type=...` switch; `execute_process` sends those to their job.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod rules;

use adblock::{lists::ParseOptions, request::Request as KickRequest, Engine, FilterSet};
use cef::*;
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::sync::{Arc, Mutex};

const HOME_HTML: &str = include_str!("../home.html");
/// Owl.'s brand red, the seed Chromium builds its colours from (Settings > Appearance > Theme).
const OWL_RED: u32 = 0xFFB8_290F;
const CLOUDFLARE_DOH: &str = "https://cloudflare-dns.com/dns-query";

/// Requests OwlKick blocked since start.
static BLOCKED: AtomicU64 = AtomicU64::new(0);

fn owl_dir() -> std::path::PathBuf {
    let base = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
    std::path::Path::new(&base).join("Owl")
}

/// %LOCALAPPDATA%\Owl\chromium: its own Chromium profile, apart from the WebView2 build's data.
fn profile_dir() -> String {
    let dir = owl_dir().join("chromium");
    let _ = std::fs::create_dir_all(&dir);
    dir.to_string_lossy().into_owned()
}

/// Owl.'s home page, written next to the profile and opened as a file address.
fn home_url() -> String {
    let path = owl_dir().join("chromium").join("home.html");
    let _ = std::fs::write(&path, HOME_HTML);
    format!("file:///{}", path.to_string_lossy().replace('\\', "/"))
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

// ---------- browser windows ----------

/// Browsers that are open; when the last one closes the message loop ends.
struct Browsers {
    list: Vec<Browser>,
}

wrap_client! {
    struct OwlClient {
        browsers: Arc<Mutex<Browsers>>,
    }

    impl Client {
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(OwlLifeSpanHandler::new(self.browsers.clone()))
        }

        fn request_handler(&self) -> Option<RequestHandler> {
            Some(OwlRequestHandler::new(std::env::var("OWL_KICK_OFF").is_err()))
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

fn value_of(set: impl FnOnce(&Value)) -> Value {
    let v = value_create().expect("value");
    set(&v);
    v
}

/// Owl.'s choices on top of Chromium's own settings: theme, home page, Secure DNS.
fn apply_owl_settings(home: &str) {
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
        let color = value_of(|v| {
            v.set_int(OWL_RED as i32);
        });
        for (name, mut value) in [
            ("browser.theme.user_color", color),
            ("homepage", s(home)),
            ("homepage_is_newtabpage", b(false)),
            ("browser.show_home_button", b(true)),
            ("signin.allowed", b(false)),
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
            *self.client.borrow_mut() = Some(OwlClient::new(Arc::new(Mutex::new(Browsers { list: vec![] }))));
            let home = home_url();
            apply_owl_settings(&home);

            // --url=... opens that address; otherwise Owl.'s home page
            let cmd = command_line_get_global().expect("no command line");
            let url = CefString::from(&cmd.switch_value(Some(&CefString::from("url")))).to_string();
            let url = if url.is_empty() { home } else { url };

            // Chrome style in its own window: Chromium's tab strip, address bar, menus and pages
            let window_info = WindowInfo { runtime_style: RuntimeStyle::CHROME, ..Default::default() }.set_as_popup(Default::default(), "Owl.");
            let settings = BrowserSettings::default();
            let mut client = self.default_client();
            browser_host_create_browser(Some(&window_info), client.as_mut(), Some(&CefString::from(url.as_str())), Some(&settings), None, None);
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
    eprintln!("OwlKick blocked {} requests", BLOCKED.load(Relaxed));
}
