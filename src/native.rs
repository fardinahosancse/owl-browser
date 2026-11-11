//! Native WebView2 hooks that a web page cannot forge: real fullscreen requests, whether a popup
//! was user-initiated, navigation failures, and engine-update notices.
use crate::Ev;
use std::{collections::VecDeque, sync::Mutex};
use tao::event_loop::EventLoopProxy;
use webview2_com::{
    take_pwstr,
    Microsoft::Web::WebView2::Win32::{ICoreWebView2Deferral, ICoreWebView2Environment5, ICoreWebView2ScriptDialogOpeningEventArgs},
    ContainsFullScreenElementChangedEventHandler, NavigationCompletedEventHandler, NewBrowserVersionAvailableEventHandler,
    NewWindowRequestedEventHandler, PermissionRequestedEventHandler, ScriptDialogOpeningEventHandler,
};
use webview2_com::Microsoft::Web::WebView2::Win32::{
    COREWEBVIEW2_PERMISSION_KIND_CAMERA, COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION, COREWEBVIEW2_PERMISSION_KIND_LOCAL_FONTS,
    COREWEBVIEW2_PERMISSION_KIND_MICROPHONE, COREWEBVIEW2_PERMISSION_KIND_MIDI_SYSTEM_EXCLUSIVE_MESSAGES,
    COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS, COREWEBVIEW2_PERMISSION_KIND_OTHER_SENSORS,
    COREWEBVIEW2_PERMISSION_KIND_WINDOW_MANAGEMENT, COREWEBVIEW2_PERMISSION_STATE_DENY,
};
use windows::core::{Interface, BOOL};
use wry::{WebView, WebViewExtWindows};

/// When true, camera, microphone, location and notification requests are refused without a prompt.
pub static PERM_BLOCK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Strict tracking prevention (blocks third-party trackers and their storage; some embedded
/// logins and videos may break). Off means the engine's balanced level.
pub static TRACK_STRICT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Apply the tracking-prevention level to this webview's profile.
pub fn apply_tracking(wv: &WebView) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2Profile3, ICoreWebView2_13, COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_BALANCED,
        COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_STRICT,
    };
    unsafe {
        let level = if TRACK_STRICT.load(std::sync::atomic::Ordering::Relaxed) {
            COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_STRICT
        } else {
            COREWEBVIEW2_TRACKING_PREVENTION_LEVEL_BALANCED
        };
        if let Ok(w13) = wv.webview().cast::<ICoreWebView2_13>() {
            if let Ok(p) = w13.Profile() {
                if let Ok(p3) = p.cast::<ICoreWebView2Profile3>() {
                    let _ = p3.SetPreferredTrackingPreventionLevel(level);
                }
            }
        }
    }
}

/// One entry per popup request, pushed in the same order Owl's own popup handler queues them.
pub static POPUPS: Mutex<VecDeque<bool>> = Mutex::new(VecDeque::new());

thread_local! {
    /// JavaScript dialogs (alert, confirm, prompt) held open until the user answers Owl's modal.
    static DIALOGS: std::cell::RefCell<std::collections::HashMap<u32, (ICoreWebView2ScriptDialogOpeningEventArgs, ICoreWebView2Deferral)>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}
static NEXT_DLG: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(1);

/// Answer a held dialog: `ok` accepts it (with `text` for a prompt), otherwise it is cancelled.
pub fn dialog_answer(id: u32, ok: bool, text: &str) {
    if let Some((args, def)) = DIALOGS.with(|d| d.borrow_mut().remove(&id)) {
        unsafe {
            if ok {
                let _ = args.SetResultText(&windows::core::HSTRING::from(text));
                let _ = args.Accept();
            }
            let _ = def.Complete();
        }
    }
}

pub fn install(wv: &WebView, proxy: EventLoopProxy<Ev>, id: u32) {
    unsafe {
        let core = wv.webview();
        let mut t = Default::default();
        let p = proxy.clone();
        let _ = core.add_ContainsFullScreenElementChanged(
            &ContainsFullScreenElementChangedEventHandler::create(Box::new(move |s, _| {
                if let Some(s) = s {
                    let mut b = BOOL(0);
                    s.ContainsFullScreenElement(&mut b)?;
                    let _ = p.send_event(Ev::Fullscreen(id, b.as_bool()));
                }
                Ok(())
            })),
            &mut t,
        );
        let _ = core.add_NewWindowRequested(
            &NewWindowRequestedEventHandler::create(Box::new(move |_, args| {
                let mut user = false;
                if let Some(a) = args {
                    let mut b = BOOL(0);
                    if a.IsUserInitiated(&mut b).is_ok() {
                        user = b.as_bool();
                    }
                }
                POPUPS.lock().unwrap().push_back(user);
                Ok(())
            })),
            &mut t,
        );
        // JavaScript dialogs: switch off the system ones (they appear outside the app and can
        // trap the user) and show Owl's own centered modal instead
        if let Ok(st) = core.Settings() {
            let _ = st.SetAreDefaultScriptDialogsEnabled(false);
        }
        let p = proxy.clone();
        let _ = core.add_ScriptDialogOpening(
            &ScriptDialogOpeningEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut kind = Default::default();
                args.Kind(&mut kind)?;
                let mut s = windows::core::PWSTR::null();
                args.Uri(&mut s)?;
                let uri = take_pwstr(s);
                let mut s = windows::core::PWSTR::null();
                args.Message(&mut s)?;
                let msg = take_pwstr(s);
                let mut s = windows::core::PWSTR::null();
                args.DefaultText(&mut s)?;
                let def = take_pwstr(s);
                let deferral = args.GetDeferral()?;
                let did = NEXT_DLG.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                DIALOGS.with(|d| d.borrow_mut().insert(did, (args.clone(), deferral)));
                let _ = p.send_event(Ev::Dialog(did, id, kind.0, uri, msg.chars().take(2000).collect(), def.chars().take(500).collect()));
                Ok(())
            })),
            &mut t,
        );
        // permission policy: fingerprinting-prone or rarely needed capabilities are always refused;
        // the sensitive ones are refused too when the user chose to block them. Everything else
        // keeps the engine's own prompt.
        let _ = core.add_PermissionRequested(
            &PermissionRequestedEventHandler::create(Box::new(move |_, args| {
                let Some(args) = args else { return Ok(()) };
                let mut kind = Default::default();
                args.PermissionKind(&mut kind)?;
                let always = kind == COREWEBVIEW2_PERMISSION_KIND_OTHER_SENSORS
                    || kind == COREWEBVIEW2_PERMISSION_KIND_LOCAL_FONTS
                    || kind == COREWEBVIEW2_PERMISSION_KIND_MIDI_SYSTEM_EXCLUSIVE_MESSAGES
                    || kind == COREWEBVIEW2_PERMISSION_KIND_WINDOW_MANAGEMENT;
                let sensitive = kind == COREWEBVIEW2_PERMISSION_KIND_CAMERA
                    || kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE
                    || kind == COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION
                    || kind == COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS;
                if always || (sensitive && PERM_BLOCK.load(std::sync::atomic::Ordering::Relaxed)) {
                    args.SetState(COREWEBVIEW2_PERMISSION_STATE_DENY)?;
                }
                Ok(())
            })),
            &mut t,
        );
        let p = proxy.clone();
        let _ = core.add_NavigationCompleted(
            &NavigationCompletedEventHandler::create(Box::new(move |_, args| {
                if let Some(a) = args {
                    let mut ok = BOOL(0);
                    a.IsSuccess(&mut ok)?;
                    if ok.as_bool() {
                        let _ = p.send_event(Ev::NavOk(id));
                    } else {
                        let mut st = Default::default();
                        a.WebErrorStatus(&mut st)?;
                        let _ = p.send_event(Ev::NavFail(id, st.0));
                    }
                }
                Ok(())
            })),
            &mut t,
        );
    }
}

/// Tells Owl when Windows has installed a newer WebView2 engine than the one this run uses.
pub fn watch_engine(wv: &WebView, proxy: EventLoopProxy<Ev>) {
    unsafe {
        let Ok(env5) = wv.environment().cast::<ICoreWebView2Environment5>() else { return };
        let mut t = Default::default();
        let _ = env5.add_NewBrowserVersionAvailable(
            &NewBrowserVersionAvailableEventHandler::create(Box::new(move |_, _| {
                let _ = proxy.send_event(Ev::EngineUpdate);
                Ok(())
            })),
            &mut t,
        );
    }
}
