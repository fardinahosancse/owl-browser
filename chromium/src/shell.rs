//! The Owl. window on Chromium: Owl.'s own interface, not Chrome's.
//!
//! One frameless window built from CEF Views:
//!   - one Chromium BrowserView per tab, stacked in the window column (only the active one visible);
//!   - Owl.'s toolbar at the bottom: the same `ui.html` as the WebView2 build (tabs, address bar,
//!     buttons, window controls, menu), in its own BrowserView.
//! The toolbar talks to Rust the way it does in the WebView2 build (`window.ipc.postMessage`); here
//! that call is a console message with a private prefix, read in the toolbar's display handler.
//! Rust answers with `owl.state(...)`, the same state object the WebView2 build sends.
//!
//! Everything here runs on Chromium's UI thread, so the window state is a thread-local.

use cef::*;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::sync::atomic::Ordering::Relaxed;

const UI_HTML: &str = include_str!("../../src/ui.html");
const WORDMARK: &str = include_str!("../../assets/wordmark.d");
const ICON_PNG: &[u8] = include_bytes!("../../assets/icon.png");
/// Owl.'s one theme: its default (white paper, black ink, Owl. red), as in src/themes.rs
const OWL_THEME: &str = ":root{--bg:#fff;--ink:#111;--mute:#595959;--soft:#f2f2f0;--line:#d0d0d0;--red:#e30613;--onred:#fff;--on:#fff;--mute-on:#c4c4c4;--rink:#ff4d4d;color-scheme:light}";
/// The toolbar's height without the menu or a notice open (the WebView2 build's UI_BASE).
const TOOL_BASE: i32 = 72;
const MSG_PREFIX: &str = "\u{1}owl:";
const SHIM_JS: &str = include_str!("../home-shim.js");

/// Owl. Calm Home, Owl.'s home and new tab page. It ships in the "home" folder next to Owl.; at
/// start it is copied into the profile (which is writable), where Owl. also keeps owl-data.js:
/// the visits the page shows as recent pages, shortcuts and reports.
static HOME_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

fn home_dir() -> &'static std::path::Path {
    HOME_DIR.get_or_init(|| std::path::PathBuf::from(crate::profile_dir()).join("home"))
}

fn file_url(p: &std::path::Path) -> String {
    format!("file:///{}", p.to_string_lossy().replace('\\', "/"))
}

fn home_url() -> String {
    file_url(&home_dir().join("newtab.html"))
}

fn copy_dir(from: &std::path::Path, to: &std::path::Path) {
    let _ = std::fs::create_dir_all(to);
    for e in std::fs::read_dir(from).into_iter().flatten().flatten() {
        let (a, b) = (e.path(), to.join(e.file_name()));
        if a.is_dir() {
            copy_dir(&a, &b);
        } else {
            let _ = std::fs::copy(&a, &b);
        }
    }
}

/// Puts the home page in place: the bundled files, the stand-in for extension APIs, and the data.
pub fn prepare_home() {
    let src = std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.join("home")));
    let dst = home_dir();
    if let Some(src) = src.filter(|s| s.join("newtab.html").is_file()) {
        copy_dir(&src, dst);
    }
    let _ = std::fs::write(dst.join("owl-shim.js"), SHIM_JS);
    for page in ["newtab.html", "report.html"] {
        let f = dst.join(page);
        if let Ok(html) = std::fs::read_to_string(&f) {
            if !html.contains("owl-shim.js") {
                let html = html.replacen(r#"<script src="common.js"></script>"#, r#"<script src="owl-data.js"></script><script src="owl-shim.js"></script><script src="common.js"></script>"#, 1);
                let _ = std::fs::write(&f, html);
            }
        }
    }
    HISTORY.with(|h| *h.borrow_mut() = std::fs::read_to_string(history_file()).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default());
    write_data();
}

// ---------- the visits Owl. keeps for its home page ----------

thread_local! {
    static HISTORY: RefCell<Vec<Value>> = const { RefCell::new(Vec::new()) };
}

fn history_file() -> std::path::PathBuf {
    std::path::PathBuf::from(crate::profile_dir()).join("owl-history.json")
}

fn now_ms() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn record_visit(url: &str, title: &str) {
    if !(url.starts_with("http://") || url.starts_with("https://")) || url.len() > 2048 {
        return;
    }
    let t = now_ms();
    let changed = HISTORY.with(|h| {
        let mut h = h.borrow_mut();
        // a page that changes its title, or a reload, within a minute is the same visit
        if let Some(last) = h.iter_mut().rev().find(|v| v["url"] == url) {
            if t.saturating_sub(last["t"].as_u64().unwrap_or(0)) < 60_000 {
                last["title"] = json!(title);
                return true;
            }
        }
        h.push(json!({"url": url, "title": title, "t": t}));
        let n = h.len();
        if n > 20_000 {
            h.drain(..n - 20_000);
        }
        true
    });
    if changed {
        let all = HISTORY.with(|h| h.borrow().clone());
        let _ = std::fs::write(history_file(), serde_json::to_string(&all).unwrap_or_default());
        write_data();
    }
}

/// owl-data.js: the visits and the most visited pages, read by the home page's stand-in.
fn write_data() {
    let all = HISTORY.with(|h| h.borrow().clone());
    let mut count: std::collections::HashMap<String, (u64, String)> = std::collections::HashMap::new();
    for v in all.iter().rev().take(5000) {
        let u = v["url"].as_str().unwrap_or("").to_string();
        let e = count.entry(u).or_insert((0, v["title"].as_str().unwrap_or("").to_string()));
        e.0 += 1;
    }
    let mut top: Vec<(String, (u64, String))> = count.into_iter().collect();
    top.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));
    let top: Vec<Value> = top.into_iter().take(8).map(|(u, (_, t))| json!({"url": u, "title": t})).collect();
    let data = json!({"history": all, "top": top});
    let _ = std::fs::write(home_dir().join("owl-data.js"), format!("window.OWL_DATA={data};"));
}

struct Tab {
    view: BrowserView,
    id: i32,
    title: String,
    url: String,
    loading: bool,
    back: bool,
    fwd: bool,
}

#[derive(Default)]
struct Shell {
    window: Option<Window>,
    /// the window's column: tab views (only the active one visible) above the toolbar
    layout: Option<BoxLayout>,
    toolbar: Option<BrowserView>,
    tool_h: i32,
    tabs: Vec<Tab>,
    active: usize,
    regions: Vec<DraggableRegion>,
    ready: bool,
}

thread_local! {
    static SHELL: RefCell<Shell> = RefCell::new(Shell { tool_h: TOOL_BASE, ..Default::default() });
}

/// Diagnosis switches (environment): OWL_FRAMELESS=0, OWL_BOX=0
fn flag(name: &str, default: bool) -> bool {
    std::env::var(name).map(|v| v != "0").unwrap_or(default)
}

fn with<R>(f: impl FnOnce(&mut Shell) -> R) -> R {
    SHELL.with(|s| f(&mut s.borrow_mut()))
}

fn s(x: &str) -> CefString {
    CefString::from(x)
}

/// What the address bar shows: Owl.'s own name for the home page, the real address otherwise.
fn display(url: &str) -> String {
    let home = file_url(home_dir());
    match url.strip_prefix(home.as_str()) {
        Some(rest) if rest.starts_with("/newtab.html") => "owl://newtab".into(),
        Some(rest) if rest.starts_with("/report.html") => "owl://report".into(),
        _ => url.to_string(),
    }
}

/// Owl.'s own addresses mapped to what Chromium has.
fn internal(url: &str) -> String {
    match url.trim_end_matches('/') {
        "owl://report" | "owl://history" => file_url(&home_dir().join("report.html")),
        // the rest of Owl.'s pages are not in this initial build yet: home for now
        u if u.starts_with("owl://") || u.is_empty() => home_url(),
        _ => url.to_string(),
    }
}

/// Typed text to an address: as the WebView2 build does it.
fn normalize(input: &str) -> String {
    let t = input.trim();
    if t.is_empty() {
        return home_url();
    }
    if t.starts_with("owl://") {
        return internal(t);
    }
    if t.contains("://") || t.starts_with("about:") || t.starts_with("data:") || t.starts_with("chrome:") {
        return t.to_string();
    }
    if !t.contains(' ') && (t.contains('.') || t.starts_with("localhost")) {
        return format!("https://{t}");
    }
    let q: String = t.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect();
    format!("https://duckduckgo.com/?q={q}")
}

// ---------- the toolbar page ----------

fn toolbar_url() -> String {
    let shim = format!(
        "<script>window.ipc={{postMessage:function(m){{console.log({p}+m)}}}};</script>\
<style>#qsav,#qdns,#inc,#mdb,#zc{{display:none!important}}</style>",
        p = serde_json::to_string(MSG_PREFIX).unwrap()
    );
    let html = UI_HTML
        .replace("/*THEMES*/", OWL_THEME)
        .replace("__WM__", WORDMARK.trim())
        .replacen("<head>", &format!("<head>{shim}"), 1);
    let b64 = CefString::from(&base64_encode(Some(html.as_bytes()))).to_string();
    format!("data:text/html;base64,{b64}")
}

fn tool_js(code: &str) {
    let tb = with(|s| s.toolbar.clone());
    if let Some(f) = tb.and_then(|v| v.browser()).and_then(|b| b.main_frame()) {
        f.execute_java_script(Some(&s(code)), Some(&s("owl://toolbar")), 0);
    }
}

/// The state object the toolbar expects (the WebView2 build's push()).
fn push() {
    let st = with(|sh| {
        if !sh.ready {
            return None;
        }
        let tabs: Vec<Value> = sh
            .tabs
            .iter()
            .map(|t| json!({"id": t.id, "title": if t.title.is_empty() { display(&t.url) } else { t.title.clone() }, "url": display(&t.url), "icon": "", "loading": t.loading, "back": t.back, "fwd": t.fwd, "zoom": 1.0, "pinned": false, "muted": false, "audible": false}))
            .collect();
        let active = sh.tabs.get(sh.active).map(|t| t.id).unwrap_or(0);
        Some(json!({
            "tabs": tabs, "active": active, "bookmarks": [], "bookmarked": false, "locked": false, "private": false,
            "blocked": crate::BLOCKED.load(Relaxed), "media": 0, "dl": {"active": 0},
            "cfg": {"theme": "owl", "eff": "owl", "block": true, "bar": false, "setup": false, "lockmode": "pin", "top": false, "saver": 0, "doh": true, "doh_on": true, "proxied": false}
        }))
    });
    if let Some(st) = st {
        tool_js(&format!("owl.state({st})"));
    }
}

// ---------- tabs ----------

fn tab_index(id: i32) -> Option<usize> {
    with(|s| s.tabs.iter().position(|t| t.id == id))
}

fn active_browser() -> Option<Browser> {
    with(|s| s.tabs.get(s.active).and_then(|t| t.view.browser()))
}

/// Show only the active tab's view.
fn show_active() {
    // copied out first: CEF calls below can run layout callbacks that read the shared state
    let (views, active, win) = with(|s| (s.tabs.iter().map(|t| t.view.clone()).collect::<Vec<_>>(), s.active, s.window.clone()));
    for (i, v) in views.iter().enumerate() {
        v.set_visible((i == active) as i32);
    }
    if let Some(w) = win {
        w.layout();
    }
    if let Some(v) = views.get(active) {
        v.request_focus();
    }
    push();
}

pub fn new_tab(url: &str) {
    crate::trace(&format!("shell: new_tab {url}"));
    let mut client = Some(TabClient::new());
    let mut delegate = TabViewDelegate::new(RuntimeStyle::ALLOY);
    let settings = BrowserSettings { background_color: 0xFFFF_FFFF, ..Default::default() };
    let Some(view) = browser_view_create(client.as_mut(), Some(&s(url)), Some(&settings), None, None, Some(&mut delegate)) else { return };
    let (win, layout, at) = with(|s| (s.window.clone(), s.layout.clone(), s.tabs.len() as i32));
    if let Some(w) = win {
        let mut v = View::from(&view);
        // above the toolbar, which stays the last child
        w.add_child_view_at(Some(&mut v), at);
        if let Some(l) = layout {
            l.set_flex_for_view(Some(&mut v), 1);
        }
    }
    with(|s| {
        // id 0 until Chromium has created the browser (see on_browser_created)
        s.tabs.push(Tab { view, id: 0, title: String::new(), url: url.to_string(), loading: true, back: false, fwd: false });
        s.active = s.tabs.len() - 1;
    });
    show_active();
}

fn close_tab(id: i32) {
    let Some(i) = tab_index(id) else { return };
    let (view, last) = with(|s| {
        let t = s.tabs.remove(i);
        if s.active >= s.tabs.len() && !s.tabs.is_empty() {
            s.active = s.tabs.len() - 1;
        } else if i < s.active {
            s.active -= 1;
        }
        (t.view, s.tabs.is_empty())
    });
    if let Some(b) = view.browser() {
        if let Some(h) = b.host() {
            h.close_browser(1);
        }
    }
    let win = with(|s| s.window.clone());
    if let Some(w) = win {
        let mut v = View::from(&view);
        w.remove_child_view(Some(&mut v));
    }
    if last {
        // the last tab: Owl. opens home rather than an empty window, like the WebView2 build
        new_tab(&home_url());
    } else {
        show_active();
    }
}

fn switch_to(id: i32) {
    if let Some(i) = tab_index(id) {
        with(|s| s.active = i);
        show_active();
    }
}

// ---------- the toolbar's commands ----------

fn command(v: Value) {
    let cmd = v["cmd"].as_str().unwrap_or("");
    let st = |k: &str| v[k].as_str().unwrap_or("").to_string();
    let win = with(|s| s.window.clone());
    match cmd {
        "ready" => {
            with(|s| s.ready = true);
            push();
        }
        "navigate" => {
            let u = normalize(&st("url"));
            match active_browser().and_then(|b| b.main_frame()) {
                Some(f) => f.load_url(Some(&s(&u))),
                None => new_tab(&u),
            }
        }
        "open_owl" => new_tab(&internal(&st("url"))),
        "new_tab" => {
            let u = st("url");
            new_tab(&if u.is_empty() { home_url() } else { normalize(&u) });
        }
        "close" => {
            let id = v["id"].as_i64().map(|x| x as i32).unwrap_or_else(|| with(|s| s.tabs.get(s.active).map(|t| t.id).unwrap_or(0)));
            close_tab(id);
        }
        "close_others" => {
            let keep = v["id"].as_i64().unwrap_or(0) as i32;
            let ids: Vec<i32> = with(|s| s.tabs.iter().map(|t| t.id).filter(|&i| i != keep).collect());
            for i in ids {
                close_tab(i);
            }
        }
        "switch" => switch_to(v["id"].as_i64().unwrap_or(0) as i32),
        "move" => {
            let (f, t) = (v["from"].as_i64().unwrap_or(0) as i32, v["to"].as_i64().unwrap_or(0) as i32);
            with(|s| {
                if let (Some(a), Some(b)) = (s.tabs.iter().position(|x| x.id == f), s.tabs.iter().position(|x| x.id == t)) {
                    let active_id = s.tabs.get(s.active).map(|x| x.id);
                    let tab = s.tabs.remove(a);
                    s.tabs.insert(b, tab);
                    if let Some(id) = active_id {
                        s.active = s.tabs.iter().position(|x| x.id == id).unwrap_or(0);
                    }
                }
            });
            push();
        }
        "back" => {
            if let Some(b) = active_browser() {
                b.go_back();
            }
        }
        "forward" => {
            if let Some(b) = active_browser() {
                b.go_forward();
            }
        }
        "reload" | "tab_reload" => {
            if let Some(b) = active_browser() {
                b.reload();
            }
        }
        "stop" => {
            if let Some(b) = active_browser() {
                b.stop_load();
            }
        }
        "key" => match v["k"].as_str().unwrap_or("") {
            "reload" => command(json!({"cmd": "reload"})),
            "back" => command(json!({"cmd": "back"})),
            "forward" => command(json!({"cmd": "forward"})),
            "new" => new_tab(&home_url()),
            "close" => command(json!({"cmd": "close"})),
            "history" => new_tab(&internal("owl://history")),
            _ => {}
        },
        "devtools" => {
            if let Some(h) = active_browser().and_then(|b| b.host()) {
                h.show_dev_tools(Some(&WindowInfo::default()), None, Some(&BrowserSettings::default()), None);
            }
        }
        "print" => {
            if let Some(h) = active_browser().and_then(|b| b.host()) {
                h.print();
            }
        }
        "win_min" => {
            if let Some(w) = win {
                w.minimize();
            }
        }
        "win_max" => {
            if let Some(w) = win {
                if w.is_maximized() != 0 {
                    w.restore();
                } else {
                    w.maximize();
                }
            }
        }
        "win_close" => {
            if let Some(w) = win {
                w.close();
            }
        }
        // the menu and notices open upwards: the toolbar grows, the page shrinks
        "ui_extra" => {
            let h = v["h"].as_f64().unwrap_or(0.0).max(0.0) as i32;
            with(|s| s.tool_h = TOOL_BASE + h);
            if let Some(w) = with(|s| s.window.clone()) {
                if let Some(t) = with(|s| s.toolbar.clone()) {
                    t.invalidate_layout();
                }
                w.layout();
            }
        }
        _ => {} // the WebView2 build's other features are not ported to this initial version yet
    }
}

// ---------- CEF handlers ----------

wrap_display_handler! {
    struct ToolbarDisplay {
        prefix: String,
    }

    impl DisplayHandler {
        fn on_console_message(
            &self,
            _browser: Option<&mut Browser>,
            _level: LogSeverity,
            message: Option<&CefString>,
            _source: Option<&CefString>,
            _line: ::std::os::raw::c_int,
        ) -> ::std::os::raw::c_int {
            let m = message.map(CefString::to_string).unwrap_or_default();
            crate::trace(&format!("toolbar console: {}", m.chars().take(120).collect::<String>()));
            match m.strip_prefix(self.prefix.as_str()) {
                Some(body) => {
                    if let Ok(v) = serde_json::from_str::<Value>(body) {
                        command(v);
                    }
                    1
                }
                None => 0,
            }
        }
    }
}

wrap_drag_handler! {
    struct ToolbarDrag {
        tag: u8,
    }

    impl DragHandler {
        fn on_draggable_regions_changed(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            regions: Option<&[DraggableRegion]>,
        ) {
            with(|s| s.regions = regions.map(|r| r.to_vec()).unwrap_or_default());
            apply_regions();
        }
    }
}

/// The toolbar's `app-region: drag` areas become the window's title bar (it has no frame).
fn apply_regions() {
    let (win, tb, regions) = with(|s| (s.window.clone(), s.toolbar.clone(), s.regions.clone()));
    let (Some(w), Some(t)) = (win, tb) else { return };
    let at = t.bounds();
    let moved: Vec<DraggableRegion> = regions
        .into_iter()
        .map(|mut r| {
            r.bounds.x += at.x;
            r.bounds.y += at.y;
            r
        })
        .collect();
    w.set_draggable_regions(Some(&moved));
}

wrap_client! {
    struct ToolbarClient {
        prefix: String,
    }

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(ToolbarDisplay::new(self.prefix.clone()))
        }

        fn drag_handler(&self) -> Option<DragHandler> {
            Some(ToolbarDrag::new(0))
        }

        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            Some(Keys::new(0))
        }
    }
}

wrap_browser_view_delegate! {
    struct ToolbarBrowserViewDelegate {
        runtime_style: RuntimeStyle,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 800, height: with(|s| s.tool_h) }
        }

        fn on_layout_changed(&self, _view: Option<&mut View>, _new_bounds: Option<&Rect>) {
            apply_regions();
        }
    }

    impl BrowserViewDelegate {
        fn browser_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }
    }
}

wrap_display_handler! {
    struct TabDisplay {
        tag: u8,
    }

    impl DisplayHandler {
        fn on_title_change(&self, browser: Option<&mut Browser>, title: Option<&CefString>) {
            let Some(id) = browser.map(|b| b.identifier()) else { return };
            let t = title.map(CefString::to_string).unwrap_or_default();
            crate::trace(&format!("tab {id} title {t}"));
            let url = with(|s| s.tabs.iter().find(|x| x.id == id).map(|x| x.url.clone()).unwrap_or_default());
            if !t.is_empty() {
                record_visit(&url, &t);
            }
            let active = with(|s| {
                if let Some(tab) = s.tabs.iter_mut().find(|x| x.id == id) {
                    tab.title = t.clone();
                }
                s.tabs.get(s.active).map(|x| x.id) == Some(id)
            });
            if active {
                if let Some(w) = with(|s| s.window.clone()) {
                    w.set_title(Some(&s(&if t.is_empty() { "Owl.".to_string() } else { format!("{t} - Owl.") })));
                }
            }
            push();
        }

        fn on_address_change(&self, browser: Option<&mut Browser>, frame: Option<&mut Frame>, url: Option<&CefString>) {
            if frame.map(|f| f.is_main()) != Some(1) {
                return;
            }
            let Some(id) = browser.map(|b| b.identifier()) else { return };
            let u = url.map(CefString::to_string).unwrap_or_default();
            crate::trace(&format!("tab {id} address {u}"));
            with(|s| {
                if let Some(tab) = s.tabs.iter_mut().find(|x| x.id == id) {
                    tab.url = u;
                }
            });
            push();
        }
    }
}

wrap_load_handler! {
    struct TabLoad {
        tag: u8,
    }

    impl LoadHandler {
        fn on_loading_state_change(
            &self,
            browser: Option<&mut Browser>,
            is_loading: ::std::os::raw::c_int,
            can_go_back: ::std::os::raw::c_int,
            can_go_forward: ::std::os::raw::c_int,
        ) {
            let Some(id) = browser.map(|b| b.identifier()) else { return };
            with(|s| {
                if let Some(tab) = s.tabs.iter_mut().find(|x| x.id == id) {
                    tab.loading = is_loading != 0;
                    tab.back = can_go_back != 0;
                    tab.fwd = can_go_forward != 0;
                }
            });
            push();
        }
    }
}

wrap_life_span_handler! {
    struct TabLife {
        tag: u8,
    }

    impl LifeSpanHandler {
        fn on_before_popup(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _popup_id: ::std::os::raw::c_int,
            target_url: Option<&CefString>,
            _target_frame_name: Option<&CefString>,
            _target_disposition: WindowOpenDisposition,
            _user_gesture: ::std::os::raw::c_int,
            _popup_features: Option<&PopupFeatures>,
            _window_info: Option<&mut WindowInfo>,
            _client: Option<&mut Option<Client>>,
            _settings: Option<&mut BrowserSettings>,
            _extra_info: Option<&mut Option<DictionaryValue>>,
            _no_javascript_access: Option<&mut ::std::os::raw::c_int>,
        ) -> ::std::os::raw::c_int {
            // pop-ups and target=_blank links open as Owl. tabs, not as new windows
            let u = target_url.map(CefString::to_string).unwrap_or_default();
            if !u.is_empty() {
                new_tab(&u);
            }
            1
        }
    }
}

wrap_keyboard_handler! {
    struct Keys {
        tag: u8,
    }

    impl KeyboardHandler {
        fn on_pre_key_event(
            &self,
            _browser: Option<&mut Browser>,
            event: Option<&KeyEvent>,
            _os_event: Option<&mut cef::sys::MSG>,
            _is_keyboard_shortcut: Option<&mut ::std::os::raw::c_int>,
        ) -> ::std::os::raw::c_int {
            let Some(e) = event else { return 0 };
            if e.type_ != KeyEventType::RAWKEYDOWN {
                return 0;
            }
            let ctrl = e.modifiers & 4 != 0; // EVENTFLAG_CONTROL_DOWN
            let k = e.windows_key_code;
            let hit = match (ctrl, k) {
                (true, 0x54) => { new_tab(&home_url()); true }                     // Ctrl+T
                (true, 0x57) => { command(json!({"cmd": "close"})); true }     // Ctrl+W
                (true, 0x52) | (false, 0x74) => { command(json!({"cmd": "reload"})); true } // Ctrl+R, F5
                (true, 0x4C) => { tool_js("owl.focusAddr()"); if let Some(t) = with(|s| s.toolbar.clone()) { t.request_focus(); } true } // Ctrl+L
                (true, 0x09) => {                                                // Ctrl+Tab
                    with(|s| if !s.tabs.is_empty() { s.active = (s.active + 1) % s.tabs.len() });
                    show_active();
                    true
                }
                _ => false,
            };
            hit as i32
        }
    }
}

wrap_client! {
    struct TabClient {}

    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> {
            Some(TabDisplay::new(0))
        }

        fn load_handler(&self) -> Option<LoadHandler> {
            Some(TabLoad::new(0))
        }

        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(TabLife::new(0))
        }

        fn keyboard_handler(&self) -> Option<KeyboardHandler> {
            Some(Keys::new(0))
        }

        fn request_handler(&self) -> Option<RequestHandler> {
            Some(crate::OwlRequestHandler::new(std::env::var("OWL_KICK_OFF").is_err()))
        }
    }
}

wrap_browser_view_delegate! {
    struct TabViewDelegate {
        runtime_style: RuntimeStyle,
    }

    impl ViewDelegate {}

    impl BrowserViewDelegate {
        fn on_browser_created(&self, browser_view: Option<&mut BrowserView>, browser: Option<&mut Browser>) {
            let (Some(view), Some(b)) = (browser_view, browser) else { return };
            let id = b.identifier();
            crate::trace(&format!("tab browser created {id}"));
            let mut created = View::from(&*view);
            let views: Vec<(usize, BrowserView)> = with(|s| s.tabs.iter().enumerate().filter(|(_, t)| t.id == 0).map(|(i, t)| (i, t.view.clone())).collect());
            // the tab whose view this is; tabs are created one at a time, so the first one still
            // waiting for its browser is the fallback
            let i = views.iter().find(|(_, v)| View::from(v).is_same(Some(&mut created)) != 0).or(views.first()).map(|(i, _)| *i);
            if let Some(i) = i {
                with(|s| {
                    if let Some(t) = s.tabs.get_mut(i) {
                        t.id = id;
                    }
                });
            }
            push();
        }

        fn browser_runtime_style(&self) -> RuntimeStyle {
            self.runtime_style
        }
    }
}

wrap_window_delegate! {
    struct OwlWindow {
        start: String,
    }

    impl ViewDelegate {
        fn preferred_size(&self, _view: Option<&mut View>) -> Size {
            Size { width: 1280, height: 800 }
        }
    }

    impl PanelDelegate {}

    impl WindowDelegate {
        fn on_window_created(&self, window: Option<&mut Window>) {
            crate::trace("shell: window created");
            let Some(window) = window else { return };
            window.set_title(Some(&s("Owl.")));
            if let Some(mut img) = image_create() {
                img.add_png(1.0, Some(ICON_PNG));
                window.set_window_app_icon(Some(&mut img));
                window.set_window_icon(Some(&mut img));
            }
            crate::trace("shell: icon set");
            window.set_background_color(0xFFFF_FFFF);
            crate::trace("shell: background set");
            // page above, toolbar below
            let layout = if flag("OWL_BOX", true) { window.set_to_box_layout(Some(&BoxLayoutSettings { horizontal: 0, cross_axis_alignment: AxisAlignment::STRETCH, ..Default::default() })) } else { None };
            crate::trace(&format!("shell: box layout {}", layout.is_some()));
            let tb = browser_view_create(
                Some(&mut ToolbarClient::new(MSG_PREFIX.to_string())),
                Some(&s(&toolbar_url())),
                Some(&BrowserSettings { background_color: 0xFFFF_FFFF, ..Default::default() }),
                None,
                None,
                Some(&mut ToolbarBrowserViewDelegate::new(RuntimeStyle::ALLOY)),
            );
            crate::trace(&format!("shell: toolbar view {}", tb.is_some()));
            if let Some(t) = &tb {
                let mut v = View::from(t);
                crate::trace("shell: toolbar as view");
                window.add_child_view(Some(&mut v));
                crate::trace("shell: toolbar child added");
                if let Some(l) = &layout {
                    l.set_flex_for_view(Some(&mut v), 0);
                }
            }
            crate::trace("shell: toolbar added");
            with(|s| {
                s.window = Some(window.clone());
                s.layout = layout;
                s.toolbar = tb;
            });
            crate::trace("shell: views added");
            new_tab(&self.start);
            crate::trace("shell: first tab");
            window.show();
            crate::trace("shell: shown");
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            with(|s| {
                s.tabs.clear();
                s.toolbar = None;
                s.layout = None;
                s.window = None;
            });
            quit_message_loop();
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            1
        }

        fn is_frameless(&self, _window: Option<&mut Window>) -> i32 {
            flag("OWL_FRAMELESS", true) as i32
        }

        fn initial_show_state(&self, _window: Option<&mut Window>) -> ShowState {
            ShowState::NORMAL
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            // the window itself must be Alloy style: a Chrome-style Views window crashes when a
            // BrowserView is added to it (CEF 156); the tabs inside stay Chrome style
            RuntimeStyle::ALLOY
        }
    }
}

/// Opens the Owl. window with its first tab.
pub fn open(start: &str) {
    crate::trace("shell: open");
    prepare_home();
    let start = if start.is_empty() { home_url() } else { start.to_string() };
    let start = start.as_str();
    let mut w = OwlWindow::new(start.to_string());
    window_create_top_level(Some(&mut w));
}
