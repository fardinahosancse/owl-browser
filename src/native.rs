//! Native WebView2 hooks that a web page cannot forge: real fullscreen requests, whether a popup
//! was user-initiated, navigation failures, and engine-update notices.
use crate::Ev;
use std::{collections::VecDeque, sync::Mutex};
use tao::event_loop::EventLoopProxy;
use webview2_com::{
    take_pwstr,
    Microsoft::Web::WebView2::Win32::{ICoreWebView2Deferral, ICoreWebView2Environment5, ICoreWebView2ScriptDialogOpeningEventArgs},
    ContainsFullScreenElementChangedEventHandler, NavigationCompletedEventHandler, NewBrowserVersionAvailableEventHandler,
    NavigationStartingEventHandler, NewWindowRequestedEventHandler, PermissionRequestedEventHandler, ScriptDialogOpeningEventHandler,
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
        // right-click on an image: add "Save image as <format>" entries to the menu
        if let (Ok(core11), Ok(env9)) = (core.cast::<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_11>(), wv.environment().cast::<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Environment9>()) {
            use webview2_com::Microsoft::Web::WebView2::Win32::{COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND, COREWEBVIEW2_CONTEXT_MENU_TARGET_KIND_IMAGE};
            let pc = proxy.clone();
            let _ = core11.add_ContextMenuRequested(
                &webview2_com::ContextMenuRequestedEventHandler::create(Box::new(move |_, args| {
                    let Some(args) = args else { return Ok(()) };
                    let target = args.ContextMenuTarget()?;
                    let mut kind = Default::default();
                    target.Kind(&mut kind)?;
                    if kind != COREWEBVIEW2_CONTEXT_MENU_TARGET_KIND_IMAGE {
                        return Ok(());
                    }
                    let mut s = windows::core::PWSTR::null();
                    target.SourceUri(&mut s)?;
                    let uri = take_pwstr(s);
                    let items = args.MenuItems()?;
                    let mut at = 0u32;
                    items.Count(&mut at)?;
                    for (ext, label) in crate::imgsave::FORMATS {
                        let item = env9.CreateContextMenuItem(&windows::core::HSTRING::from(format!("Save image as {label}")), None, COREWEBVIEW2_CONTEXT_MENU_ITEM_KIND_COMMAND)?;
                        let (p, u, e) = (pc.clone(), uri.clone(), ext.to_string());
                        let mut tk = Default::default();
                        item.add_CustomItemSelected(
                            &webview2_com::CustomItemSelectedEventHandler::create(Box::new(move |_, _| {
                                let _ = p.send_event(Ev::SaveImage(u.clone(), e.clone()));
                                Ok(())
                            })),
                            &mut tk,
                        )?;
                        items.InsertValueAtIndex(at, &item)?;
                        at += 1;
                    }
                    Ok(())
                })),
                &mut t,
            );
        }
        // magnet links open in Owl's torrent client instead of an external program
        let pm = proxy.clone();
        let _ = core.add_NavigationStarting(
            &NavigationStartingEventHandler::create(Box::new(move |_, args| {
                if let Some(a) = args {
                    let mut s = windows::core::PWSTR::null();
                    a.Uri(&mut s)?;
                    let uri = take_pwstr(s);
                    if uri.len() >= 8 && uri[..8].eq_ignore_ascii_case("magnet:?") {
                        a.SetCancel(true)?;
                        let _ = pm.send_event(Ev::Magnet(uri));
                    }
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
            // reputation checks (SmartScreen) for sites and downloads must stay on
            if let Ok(s8) = st.cast::<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings8>() {
                let _ = s8.SetIsReputationCheckingRequired(true);
            }
            // Owl has its own password manager; the engine must not keep a second copy
            if let Ok(s4) = st.cast::<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings4>() {
                let _ = s4.SetIsPasswordAutosaveEnabled(false);
                let _ = s4.SetIsGeneralAutofillEnabled(false);
            }
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

/// Permissions the user has changed from the default (allowed or blocked), for the permissions page.
/// Calls `send(json)` with `[{"k":kind,"o":origin,"s":state}]`.
pub fn list_permissions(wv: &WebView, send: impl Fn(String) + 'static) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{ICoreWebView2Profile4, ICoreWebView2_13};
    unsafe {
        let Ok(w13) = wv.webview().cast::<ICoreWebView2_13>() else { return send("[]".into()) };
        let Ok(p) = w13.Profile() else { return send("[]".into()) };
        let Ok(p4) = p.cast::<ICoreWebView2Profile4>() else { return send("[]".into()) };
        let h = webview2_com::GetNonDefaultPermissionSettingsCompletedHandler::create(Box::new(move |_, list| {
            let mut out = Vec::new();
            if let Some(list) = list {
                let mut n = 0u32;
                list.Count(&mut n)?;
                for i in 0..n.min(500) {
                    let item = list.GetValueAtIndex(i)?;
                    let (mut k, mut st) = (Default::default(), Default::default());
                    item.PermissionKind(&mut k)?;
                    item.PermissionState(&mut st)?;
                    let mut o = windows::core::PWSTR::null();
                    item.PermissionOrigin(&mut o)?;
                    out.push(serde_json::json!({"k": k.0, "o": take_pwstr(o), "s": st.0}));
                }
            }
            send(serde_json::Value::Array(out).to_string());
            Ok(())
        }));
        let _ = p4.GetNonDefaultPermissionSettings(&h);
    }
}

/// Put one site permission back to "ask".
pub fn reset_permission(wv: &WebView, kind: i32, origin: &str) {
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2Profile4, ICoreWebView2_13, COREWEBVIEW2_PERMISSION_KIND, COREWEBVIEW2_PERMISSION_STATE,
    };
    unsafe {
        let Ok(w13) = wv.webview().cast::<ICoreWebView2_13>() else { return };
        let Ok(p) = w13.Profile() else { return };
        let Ok(p4) = p.cast::<ICoreWebView2Profile4>() else { return };
        let h = webview2_com::SetPermissionStateCompletedHandler::create(Box::new(|_| Ok(())));
        let _ = p4.SetPermissionState(COREWEBVIEW2_PERMISSION_KIND(kind), &windows::core::HSTRING::from(origin), COREWEBVIEW2_PERMISSION_STATE(0), &h);
    }
}

/// Let the chrome's CSS `app-region: drag` areas act as the window's title bar, handled by the
/// engine itself: dragging starts at once (no message round trip), Windows snap layouts and
/// double-click to maximize work. Needs WebView2 runtime 1.0.2420 or newer; returns false on older
/// runtimes, where the script-driven drag keeps working.
pub fn native_drag(wv: &WebView) -> bool {
    unsafe {
        let Ok(st) = wv.webview().Settings() else { return false };
        match st.cast::<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Settings9>() {
            Ok(s9) => s9.SetIsNonClientRegionSupportEnabled(true).is_ok(),
            Err(_) => false,
        }
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

/// Call one Chrome DevTools Protocol method on a page; `done` gets the JSON result ("" on error).
pub fn cdp(wv: &WebView, method: &str, params: &str, done: impl FnOnce(String) + 'static) {
    unsafe {
        let core = wv.webview();
        let h = webview2_com::CallDevToolsProtocolMethodCompletedHandler::create(Box::new(move |r, json| {
            done(if r.is_ok() { json } else { String::new() });
            Ok(())
        }));
        let _ = core.CallDevToolsProtocolMethod(&windows::core::HSTRING::from(method), &windows::core::HSTRING::from(params), &h);
    }
}

/// How many cookies the page's profile holds for this address.
pub fn cookie_count(wv: &WebView, url: &str, done: impl FnOnce(u32) + 'static) {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2_2;
    unsafe {
        let Some(cm) = wv.webview().cast::<ICoreWebView2_2>().ok().and_then(|c| c.CookieManager().ok()) else { return done(0) };
        let h = webview2_com::GetCookiesCompletedHandler::create(Box::new(move |_, list| {
            let mut n = 0u32;
            if let Some(list) = list {
                list.Count(&mut n)?;
            }
            done(n);
            Ok(())
        }));
        let _ = cm.GetCookies(&windows::core::HSTRING::from(url), &h);
    }
}

fn b64_decode(s: &str) -> Vec<u8> {
    let val = |c: u8| match c {
        b'A'..=b'Z' => Some(c - b'A'),
        b'a'..=b'z' => Some(c - b'a' + 26),
        b'0'..=b'9' => Some(c - b'0' + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    };
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0);
    for c in s.bytes().filter_map(val) {
        acc = acc << 6 | c as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    out
}

/// Days since 1970-01-01 to a calendar date (Howard Hinnant's civil_from_days).
fn civil(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Subject, issuer and expiry of the site's certificate, from Network.getCertificate's base64
/// DER chain (the first entry is the site's own certificate). Read with Windows' crypt32.
pub fn cert_info(cdp_json: &str) -> Option<(String, String, String)> {
    use windows::Win32::Security::Cryptography::{
        CertCreateCertificateContext, CertFreeCertificateContext, CertGetNameStringW, CERT_NAME_ISSUER_FLAG, CERT_NAME_SIMPLE_DISPLAY_TYPE,
        PKCS_7_ASN_ENCODING, X509_ASN_ENCODING,
    };
    let v: serde_json::Value = serde_json::from_str(cdp_json).ok()?;
    let der = b64_decode(v["tableNames"].get(0)?.as_str()?);
    if der.len() < 64 {
        return None;
    }
    unsafe {
        let ctx = CertCreateCertificateContext(X509_ASN_ENCODING | PKCS_7_ASN_ENCODING, &der);
        if ctx.is_null() {
            return None;
        }
        let name = |flags: u32| {
            let mut buf = [0u16; 256];
            let n = CertGetNameStringW(ctx, CERT_NAME_SIMPLE_DISPLAY_TYPE, flags, None, Some(&mut buf));
            String::from_utf16_lossy(&buf[..(n as usize).saturating_sub(1)])
        };
        let (subject, issuer) = (name(0), name(CERT_NAME_ISSUER_FLAG));
        let ft = (*(*ctx).pCertInfo).NotAfter;
        let _ = CertFreeCertificateContext(Some(ctx));
        let t = ((ft.dwHighDateTime as u64) << 32 | ft.dwLowDateTime as u64) / 10_000_000;
        let unix = t as i64 - 11_644_473_600;
        let (y, m, d) = civil(unix.div_euclid(86_400));
        const MON: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
        Some((subject, issuer, format!("{d} {} {y}", MON[(m - 1) as usize])))
    }
}
