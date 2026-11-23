//! Owl. in Chromium's own browser window (CEF "Chrome style").
//!
//! Tabs, the address bar, menus, downloads, passwords, history, bookmarks, settings and
//! extensions are Chromium's own. Owl. changes how they look and what they show:
//!
//! - the Owl. theme (`theme/` next to the exe, a Chrome theme in Owl.'s white, paper grey and ink),
//!   with Owl.'s red as the accent Chromium builds its own colours from;
//! - Owl. Calm Home (`home/` next to the exe, the same extension as in chrome-extension-homepage)
//!   as the new tab and home page, with its full access to history, bookmarks and downloads;
//! - Chrome extras Owl. does not use are hidden: the profile avatar, the media and battery saver
//!   buttons, and page actions such as Lens, Google AI, price tracking and translate offers.

use cef::*;
use std::sync::{Arc, Mutex};

/// A folder next to owl-chromium.exe.
pub(crate) fn beside_exe(name: &str) -> Option<std::path::PathBuf> {
    let dir = std::env::current_exe().ok()?.parent()?.join(name);
    dir.join("manifest.json").is_file().then_some(dir)
}

/// Owl. Calm Home's ID (fixed by the "key" in its manifest).
const HOME_ID: &str = "lejabghpgomlgigjhibhnocaddmlofid";

/// Chromium asks "Change back to Google?" the first time an extension takes over the new tab page.
/// Calm Home is Owl.'s own new tab page, so the profile records that question as answered
/// before Chromium starts.
pub(crate) fn accept_home_takeover(profile: &str) {
    let path = std::path::Path::new(profile).join("Default").join("Preferences");
    let _ = std::fs::create_dir_all(path.parent().unwrap());
    let mut prefs: serde_json::Value = std::fs::read_to_string(&path)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    let Some(root) = prefs.as_object_mut() else { return };
    let ext = root.entry("extensions").or_insert_with(|| serde_json::json!({}));
    let Some(ext) = ext.as_object_mut() else { return };
    let set = ext.entry("settings").or_insert_with(|| serde_json::json!({}));
    let Some(set) = set.as_object_mut() else { return };
    let home = set.entry(HOME_ID).or_insert_with(|| serde_json::json!({}));
    let Some(home) = home.as_object_mut() else { return };
    if home.get("ack_ntp_bubble") == Some(&serde_json::Value::Bool(true)) {
        return;
    }
    home.insert("ack_ntp_bubble".into(), serde_json::Value::Bool(true));
    home.insert("ack_settings_bubble".into(), serde_json::Value::Bool(true));
    let _ = std::fs::write(&path, prefs.to_string());
}

/// The extensions Owl. loads on every start: its home page and its theme.
pub(crate) fn owl_extensions() -> String {
    ["home", "theme"]
        .iter()
        .filter_map(|n| beside_exe(n))
        .map(|p| p.to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join(",")
}

struct Browsers {
    list: Vec<Browser>,
}

wrap_life_span_handler! {
    struct OwlLifeSpanHandler {
        browsers: Arc<Mutex<Browsers>>,
    }

    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            if let Some(b) = browser.cloned() {
                let url = b.main_frame().map(|f| CefString::from(&f.url()).to_string()).unwrap_or_default();
                crate::trace(&format!("browser created: id {} popup {} url {url}", b.identifier(), b.is_popup()));
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

wrap_command_handler! {
    struct OwlCommandHandler {
        tag: u8,
    }

    impl CommandHandler {
        fn is_chrome_toolbar_button_visible(&self, button_type: ChromeToolbarButtonType) -> i32 {
            let t = *button_type.as_ref() as i32;
            // hidden: the media controls, battery saver and the profile avatar (Owl. has no sign-in)
            let hide = [ChromeToolbarButtonType::MEDIA, ChromeToolbarButtonType::BATTERY_SAVER, ChromeToolbarButtonType::AVATAR]
                .iter()
                .any(|h| *h.as_ref() as i32 == t);
            (!hide) as i32
        }

        fn is_chrome_page_action_icon_visible(&self, icon_type: ChromePageActionIconType) -> i32 {
            let t = *icon_type.as_ref() as i32;
            // kept: the bookmark star, find, zoom, passwords, cookies and site permissions, and the
            // file and payment prompts a site asks for; Google's shopping, AI and Lens extras go
            let keep = [
                ChromePageActionIconType::BOOKMARK_STAR,
                ChromePageActionIconType::FIND,
                ChromePageActionIconType::ZOOM,
                ChromePageActionIconType::MANAGE_PASSWORDS,
                ChromePageActionIconType::CHANGE_PASSWORD,
                ChromePageActionIconType::COOKIE_CONTROLS,
                ChromePageActionIconType::FILE_SYSTEM_ACCESS,
                ChromePageActionIconType::INTENT_PICKER,
                ChromePageActionIconType::PWA_INSTALL,
                ChromePageActionIconType::SAVE_CARD,
                ChromePageActionIconType::SAVE_AUTOFILL_ADDRESS,
                ChromePageActionIconType::SAVE_IBAN,
                ChromePageActionIconType::MANDATORY_REAUTH,
                ChromePageActionIconType::SMS_REMOTE_FETCHER,
                ChromePageActionIconType::WEB_AUTHN_AMBIENT_SIGNIN,
                ChromePageActionIconType::AUTOFILL_PAYMENT,
            ];
            keep.iter().any(|k| *k.as_ref() as i32 == t) as i32
        }
    }
}

wrap_client! {
    struct OwlClient {
        browsers: Arc<Mutex<Browsers>>,
    }

    impl Client {
        fn life_span_handler(&self) -> Option<LifeSpanHandler> {
            Some(OwlLifeSpanHandler::new(self.browsers.clone()))
        }

        fn command_handler(&self) -> Option<CommandHandler> {
            Some(OwlCommandHandler::new(0))
        }

        fn request_handler(&self) -> Option<RequestHandler> {
            Some(crate::OwlRequestHandler::new(std::env::var("OWL_KICK_OFF").is_err()))
        }
    }
}

pub(crate) fn new_client() -> Client {
    OwlClient::new(Arc::new(Mutex::new(Browsers { list: vec![] })))
}

/// Opens the Owl. window: Chromium's tabbed browser, on `start` or on the new tab page.
pub(crate) fn open(client: &mut Option<Client>, start: &str) {
    let url = if start.is_empty() { "chrome://newtab/" } else { start };
    let window_info = WindowInfo { runtime_style: RuntimeStyle::CHROME, ..Default::default() }.set_as_popup(Default::default(), "Owl.");
    let settings = BrowserSettings::default();
    browser_host_create_browser(Some(&window_info), client.as_mut(), Some(&CefString::from(url)), Some(&settings), None, None);
}
