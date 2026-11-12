//! OwlKick Engine: Owl's content blocker, built on the open source adblock-rust engine that
//! Brave Browser uses (https://github.com/brave/adblock-rust, MPL-2.0). It is not made or
//! endorsed by Brave. It understands Adblock Plus / uBlock Origin filter syntax, so EasyList and
//! EasyPrivacy work as they do in other blockers.
//!
//! Rules come from a built-in base set (Owl's tracker host list) plus any filter lists saved in
//! `%LOCALAPPDATA%\Owl\filters\*.txt`. Downloading lists is opt-in (Settings > Privacy).
use adblock::{lists::ParseOptions, request::Request, Engine, FilterSet};
use std::{cell::RefCell, path::PathBuf, sync::Mutex};

/// Filter lists the optional updater fetches: (file name, host, path).
pub const LISTS: &[(&str, &str, &str)] = &[
    ("easylist.txt", "easylist.to", "/easylist/easylist.txt"),
    ("easyprivacy.txt", "easylist.to", "/easylist/easyprivacy.txt"),
];

thread_local! {
    // the engine is single-threaded by design; WebView2 callbacks all run on the UI thread
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
}

pub fn dir() -> PathBuf {
    crate::data_dir().join("filters")
}

fn build(hosts: &[&str], paths: &[&str]) -> Engine {
    let mut set = FilterSet::new(false);
    let mut base = String::new();
    for h in hosts {
        base.push_str(&format!("||{h}^\n"));
    }
    for p in paths {
        if p.starts_with('/') {
            base.push_str(&format!("{p}\n"));
        } else {
            base.push_str(&format!("||{p}\n"));
        }
    }
    set.add_filter_list(base, ParseOptions::default());
    if let Ok(rd) = std::fs::read_dir(dir()) {
        for e in rd.flatten() {
            if e.path().extension().map_or(false, |x| x == "txt") {
                if let Ok(t) = std::fs::read_to_string(e.path()) {
                    set.add_filter_list(t, ParseOptions::default());
                }
            }
        }
    }
    Engine::new_with_filter_set(set)
}

/// Download the filter lists in the background; `done(n)` gets the number saved.
pub fn update_lists(done: impl FnOnce(usize) + Send + 'static) {
    std::thread::spawn(move || {
        let _ = std::fs::create_dir_all(dir());
        let mut ok = 0;
        for (name, host, path) in LISTS {
            if let Some(t) = crate::update::fetch_url(host, path, 12 * 1024 * 1024) {
                // sanity check: a real list starts with an Adblock header or has many rules
                if t.starts_with("[Adblock") || t.lines().count() > 500 {
                    let tmp = dir().join(format!("{name}.tmp"));
                    if std::fs::write(&tmp, t).is_ok() && std::fs::rename(&tmp, dir().join(name)).is_ok() {
                        ok += 1;
                    }
                }
            }
        }
        done(ok);
    });
}

/// Rebuild after lists were downloaded.
pub fn reload() {
    ENGINE.with(|e| *e.borrow_mut() = None);
}

/// Sites where the engine is switched off (the user's choice), kept lower-case without "www.".
pub static OFF_SITES: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub fn is_off(page_url: &str) -> bool {
    let h = page_url.split("://").nth(1).unwrap_or("").split(['/', '?', '#', ':']).next().unwrap_or("").to_lowercase();
    let h = h.trim_start_matches("www.");
    !h.is_empty() && OFF_SITES.lock().unwrap().iter().any(|s| h == s || h.ends_with(&format!(".{s}")))
}

/// Age in days of the newest downloaded list, None if there is none.
pub fn lists_age_days() -> Option<u64> {
    let newest = std::fs::read_dir(dir()).ok()?.flatten().filter(|e| e.path().extension().map_or(false, |x| x == "txt")).filter_map(|e| e.metadata().ok()?.modified().ok()).max()?;
    Some(std::time::SystemTime::now().duration_since(newest).map(|d| d.as_secs() / 86_400).unwrap_or(0))
}

/// Number of lists currently on disk.
pub fn list_count() -> usize {
    std::fs::read_dir(dir()).map(|r| r.flatten().filter(|e| e.path().extension().map_or(false, |x| x == "txt")).count()).unwrap_or(0)
}

/// `kind` is the adblock request type: document, stylesheet, image, media, font, script, xhr,
/// websocket, ping or other.
pub fn should_block(url: &str, source: &str, kind: &str, hosts: &[&str], paths: &[&str]) -> bool {
    ENGINE.with(|cell| {
        let mut b = cell.borrow_mut();
        let eng = b.get_or_insert_with(|| build(hosts, paths));
        match Request::new(url, if source.is_empty() { url } else { source }, kind, "GET") {
            Ok(r) => { let res = eng.check_network_request(&r); res.important || (res.filter.is_some() && res.exception.is_none()) },
            Err(_) => false,
        }
    })
}

/// Element-hiding CSS for a page address (cosmetic filters from the loaded lists).
pub fn cosmetic_css(url: &str, hosts: &[&str], paths: &[&str]) -> String {
    ENGINE.with(|cell| {
        let mut b = cell.borrow_mut();
        let eng = b.get_or_insert_with(|| build(hosts, paths));
        let res = eng.url_cosmetic_resources(url);
        let mut css = String::new();
        for s in res.hide_selectors.iter().take(4000) {
            css.push_str(s);
            css.push_str("{display:none!important}");
        }
        css
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_rules_block_and_allow() {
        let hosts = ["tracker.example"];
        let paths = ["/adserver/"];
        assert!(should_block("https://tracker.example/p.js", "https://site.org/", "script", &hosts, &paths));
        assert!(should_block("https://sub.tracker.example/p.js", "https://site.org/", "image", &hosts, &paths));
        assert!(should_block("https://site.org/adserver/x.gif", "https://site.org/", "image", &hosts, &paths));
        assert!(!should_block("https://site.org/app.js", "https://site.org/", "script", &hosts, &paths));
        assert!(!should_block("https://nottracker.example/p.js", "https://site.org/", "script", &hosts, &paths));
    }

    #[test]
    fn per_site_switch_off() {
        *OFF_SITES.lock().unwrap() = vec!["bank.example".to_string()];
        assert!(is_off("https://www.bank.example/login"));
        assert!(is_off("https://pay.bank.example:8443/x"));
        assert!(!is_off("https://notbank.example/"));
        assert!(!is_off("https://other.org/"));
        OFF_SITES.lock().unwrap().clear();
    }
}
