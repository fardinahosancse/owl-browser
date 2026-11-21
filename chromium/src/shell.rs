//! The Owl. window on Chromium: Owl.'s own interface, not Chrome's.
//!
//! One frameless window built from CEF Views:
//!   - a content panel that holds one Chromium BrowserView per tab (only the active one visible);
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
/// The home page is the bundled Owl. Calm Home extension (its fixed ID), the new tab page.
const HOME_EXT: &str = "chrome-extension://lejabghpgomlgigjhibhnocaddmlofid/newtab.html";
const NEW_TAB: &str = "chrome://newtab/";

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
    content: Option<Panel>,
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

fn with<R>(f: impl FnOnce(&mut Shell) -> R) -> R {
    SHELL.with(|s| f(&mut s.borrow_mut()))
}

fn s(x: &str) -> CefString {
    CefString::from(x)
}

/// What the address bar shows: Owl.'s own name for the home page, the real address otherwise.
fn display(url: &str) -> String {
    if url.starts_with(HOME_EXT) || url.starts_with("chrome://newtab") || url.starts_with("chrome://new-tab-page") {
        "owl://newtab".into()
    } else {
        url.to_string()
    }
}

/// Owl.'s own addresses mapped to what Chromium has.
fn internal(url: &str) -> String {
    match url.trim_end_matches('/') {
        "owl://newtab" | "owl://home" | "" => NEW_TAB.into(),
        "owl://downloads" => "chrome://downloads/".into(),
        "owl://history" => "chrome://history/".into(),
        "owl://bookmarks" => "chrome://bookmarks/".into(),
        "owl://passwords" => "chrome://password-manager/".into(),
        "owl://settings" | "owl://permissions" => "chrome://settings/".into(),
        "owl://about" => "chrome://version/".into(),
        "owl://report" => HOME_EXT.replace("newtab.html", "report.html"),
        _ => url.to_string(),
    }
}

/// Typed text to an address: as the WebView2 build does it.
fn normalize(input: &str) -> String {
    let t = input.trim();
    if t.is_empty() {
        return NEW_TAB.into();
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
    let (views, active, content) = with(|s| (s.tabs.iter().map(|t| t.view.clone()).collect::<Vec<_>>(), s.active, s.content.clone()));
    for (i, v) in views.iter().enumerate() {
        v.set_visible((i == active) as i32);
    }
    if let Some(c) = content {
        c.layout();
    }
    if let Some(v) = views.get(active) {
        v.request_focus();
    }
    push();
}

pub fn new_tab(url: &str) {
    let mut client = Some(TabClient::new());
    let mut delegate = TabViewDelegate::new(RuntimeStyle::DEFAULT);
    let settings = BrowserSettings { background_color: 0xFFFF_FFFF, ..Default::default() };
    let Some(view) = browser_view_create(client.as_mut(), Some(&s(url)), Some(&settings), None, None, Some(&mut delegate)) else { return };
    let content = with(|s| s.content.clone());
    if let Some(c) = content {
        let mut v = View::from(&view);
        c.add_child_view(Some(&mut v));
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
    let content = with(|s| s.content.clone());
    if let Some(c) = content {
        let mut v = View::from(&view);
        c.remove_child_view(Some(&mut v));
    }
    if last {
        // the last tab: Owl. opens home rather than an empty window, like the WebView2 build
        new_tab(NEW_TAB);
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
            new_tab(&if u.is_empty() { NEW_TAB.to_string() } else { normalize(&u) });
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
            "new" => new_tab(NEW_TAB),
            "close" => command(json!({"cmd": "close"})),
            "history" => new_tab("chrome://history/"),
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
                (true, 0x54) => { new_tab(NEW_TAB); true }                     // Ctrl+T
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
            let mut created = View::from(&*view);
            let views: Vec<(usize, BrowserView)> = with(|s| s.tabs.iter().enumerate().filter(|(_, t)| t.id == 0).map(|(i, t)| (i, t.view.clone())).collect());
            for (i, v) in views {
                if View::from(&v).is_same(Some(&mut created)) != 0 {
                    with(|s| {
                        if let Some(t) = s.tabs.get_mut(i) {
                            t.id = id;
                        }
                    });
                }
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
            let Some(window) = window else { return };
            window.set_title(Some(&s("Owl.")));
            if let Some(mut img) = image_create() {
                img.add_png(1.0, Some(ICON_PNG));
                window.set_window_app_icon(Some(&mut img));
                window.set_window_icon(Some(&mut img));
            }
            window.set_background_color(0xFFFF_FFFF);
            // page above, toolbar below
            let layout = window.set_to_box_layout(Some(&BoxLayoutSettings { horizontal: 0, ..Default::default() }));
            let content = panel_create(None);
            let tb = browser_view_create(
                Some(&mut ToolbarClient::new(MSG_PREFIX.to_string())),
                Some(&s(&toolbar_url())),
                Some(&BrowserSettings { background_color: 0xFFFF_FFFF, ..Default::default() }),
                None,
                None,
                Some(&mut ToolbarBrowserViewDelegate::new(RuntimeStyle::DEFAULT)),
            );
            if let Some(c) = &content {
                c.set_to_fill_layout();
                let mut v = View::from(c);
                window.add_child_view(Some(&mut v));
                if let Some(l) = &layout {
                    l.set_flex_for_view(Some(&mut v), 1);
                }
            }
            if let Some(t) = &tb {
                let mut v = View::from(t);
                window.add_child_view(Some(&mut v));
                if let Some(l) = &layout {
                    l.set_flex_for_view(Some(&mut v), 0);
                }
            }
            with(|s| {
                s.window = Some(window.clone());
                s.content = content;
                s.toolbar = tb;
            });
            new_tab(&self.start);
            window.show();
        }

        fn on_window_destroyed(&self, _window: Option<&mut Window>) {
            with(|s| {
                s.tabs.clear();
                s.toolbar = None;
                s.content = None;
                s.window = None;
            });
            quit_message_loop();
        }

        fn can_close(&self, _window: Option<&mut Window>) -> i32 {
            1
        }

        fn is_frameless(&self, _window: Option<&mut Window>) -> i32 {
            1
        }

        fn initial_show_state(&self, _window: Option<&mut Window>) -> ShowState {
            ShowState::NORMAL
        }

        fn window_runtime_style(&self) -> RuntimeStyle {
            RuntimeStyle::DEFAULT
        }
    }
}

/// Opens the Owl. window with its first tab.
pub fn open(start: &str) {
    let mut w = OwlWindow::new(start.to_string());
    window_create_top_level(Some(&mut w));
}
