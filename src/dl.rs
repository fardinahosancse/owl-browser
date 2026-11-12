//! Download manager hooks: takes over WebView2's download flow so Owl can show progress,
//! pause, resume and cancel.
use crate::Ev;
use std::{
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    sync::atomic::{AtomicU32, Ordering::Relaxed},
};
use tao::event_loop::EventLoopProxy;
use webview2_com::{
    take_pwstr,
    Microsoft::Web::WebView2::Win32::{
        ICoreWebView2Deferral, ICoreWebView2DownloadOperation, ICoreWebView2DownloadStartingEventArgs, ICoreWebView2Settings2,
        ICoreWebView2_2, ICoreWebView2_4,
    },
    BytesReceivedChangedEventHandler, DownloadStartingEventHandler, GetCookiesCompletedHandler, StateChangedEventHandler,
};
use windows::core::{Interface, HSTRING, PWSTR};
use wry::{WebView, WebViewExtWindows};

static NEXT: AtomicU32 = AtomicU32::new(1);

thread_local! {
    static OPS: RefCell<HashMap<u32, ICoreWebView2DownloadOperation>> = RefCell::new(HashMap::new());
    /// Downloads held open until Owl decides whether to allow them.
    /// What the segmented engine needs for each download: address, cookies, user agent, referrer.
    static CTX: RefCell<HashMap<u32, crate::engine::Ctx>> = RefCell::new(HashMap::new());
    static PROXY: RefCell<Option<EventLoopProxy<Ev>>> = RefCell::new(None);
    static HELD: RefCell<HashMap<u32, (ICoreWebView2DownloadStartingEventArgs, ICoreWebView2Deferral)>> = RefCell::new(HashMap::new());
}

/// Extensions that can run code or change the system when opened. Never opened directly.
pub fn risky(path: &str) -> bool {
    // Windows ignores trailing dots and spaces, so "x.exe." is still an executable
    let path = path.trim_end_matches(|c| c == '.' || c == ' ');
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    let ext = ext.split(':').next().unwrap_or("").to_string(); // alternate data stream suffixes
    [
        "exe", "msi", "msp", "msix", "msixbundle", "appx", "appxbundle", "bat", "cmd", "com", "scr", "pif", "cpl", "ps1",
        "psm1", "psd1", "vbs", "vbe", "js", "jse", "wsf", "wsh", "hta", "jar", "lnk", "url", "scf", "reg", "inf", "chm",
        "application", "gadget", "msc", "dll", "sys", "iso", "img", "vhd", "vhdx", "docm", "xlsm", "pptm", "dotm", "xlam",
        "ppam", "sldm", "xll", "wll", "msh", "ws", "theme", "settingcontent-ms", "appref-ms", "library-ms", "diagcab",
        "website", "svg", "html", "htm", "xhtml", "mht", "mhtml", "rdp", "py", "pyw", "pyc", "jnlp", "msu", "xbap", "cab",
        "iqy", "slk", "ps1xml", "psc1", "psc2", "wsc", "crx", "vsix", "doc", "xls", "ppt", "dot", "xlt", "pot", "pps",
        "rtf", "hlp", "mde", "mdb", "accde", "ade", "adp", "bas", "ins", "isp", "job", "lnk", "mad", "maf", "mag", "mam",
        "maq", "mar", "mas", "mat", "mau", "mav", "maw", "mda", "mdt", "mdw", "mdz", "ops", "prf", "prg", "pst", "scr",
        "sct", "shb", "shs", "tmp", "vb", "vsmacros", "vsw", "ws", "xnk", "appinstaller", "pdb",
    ]
    .contains(&ext.as_str())
}

/// The Downloads folder; Owl only opens files that live under it.
pub fn root() -> PathBuf {
    PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default()).join("Downloads")
}

pub fn inside_root(p: &str) -> bool {
    let (Ok(a), Ok(b)) = (PathBuf::from(p).canonicalize(), root().canonicalize()) else { return false };
    a.starts_with(b)
}

/// Allow or refuse a download that is being held. Does nothing if it is not held.
pub fn dbg(m: &str) {
    use std::io::Write;
    if std::env::var_os("OWL_DEBUG").is_none() {
        return;
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(crate::data_dir().join("dl.log")) {
        let _ = writeln!(f, "{m}");
    }
}

pub fn decide(id: u32, allow: bool) {
    dbg(&format!("decide {id} {allow} direct={} ctx={}", crate::engine::DIRECT_OK.load(Relaxed), CTX.with(|c| c.borrow().contains_key(&id))));
    // allowed downloads of http(s) files are first probed for range support; the browser's own
    // download stays held until `probe_done` says whether the segmented engine takes over
    if allow && crate::engine::DIRECT_OK.load(Relaxed) {
        let ctx = CTX.with(|c| c.borrow().get(&id).cloned());
        if let Some(ctx) = ctx.filter(|c| c.url.starts_with("http://") || c.url.starts_with("https://")) {
            if HELD.with(|h| h.borrow().contains_key(&id)) {
                if let Some(p) = PROXY.with(|p| p.borrow().clone()) {
                    crate::engine::probe(id, ctx, p);
                    return;
                }
            }
        }
    }
    CTX.with(|c| {
        c.borrow_mut().remove(&id);
    });
    if let Some((args, def)) = HELD.with(|h| h.borrow_mut().remove(&id)) {
        unsafe {
            if !allow {
                let _ = args.SetCancel(true);
            }
            let _ = def.Complete();
        }
    }
    if !allow {
        OPS.with(|o| {
            o.borrow_mut().remove(&id);
        });
    }
}

/// The probe finished. If the file can be segmented, cancel the browser's download and hand it to
/// the engine; otherwise let the browser download it as usual.
pub fn probe_done(id: u32, probe: Option<crate::engine::Probe>, proxy: EventLoopProxy<Ev>) {
    let Some((args, def)) = HELD.with(|h| h.borrow_mut().remove(&id)) else {
        CTX.with(|c| c.borrow_mut().remove(&id));
        return; // cancelled while probing
    };
    let ctx = CTX.with(|c| c.borrow_mut().remove(&id));
    let take = match (&probe, &ctx) {
        (Some(p), Some(c)) if p.total >= crate::engine::MIN_SIZE => !std::path::Path::new(&c.path).exists(),
        _ => false,
    };
    unsafe {
        if take {
            let _ = args.SetCancel(true);
        }
        let _ = def.Complete();
    }
    if take {
        OPS.with(|o| {
            o.borrow_mut().remove(&id);
        });
        if !crate::engine::start(id, ctx.unwrap(), probe.unwrap(), proxy.clone()) {
            // could not prepare the file: report it so the user can retry
            let _ = proxy.send_event(Ev::DlState(id, 1));
        }
    }
}

pub fn next_id() -> u32 {
    NEXT.fetch_add(1, Relaxed)
}

pub fn set_next(n: u32) {
    NEXT.store(n, Relaxed);
}

fn category(ext: &str) -> &'static str {
    match ext {
        "pdf" | "doc" | "docx" | "xls" | "xlsx" | "ppt" | "pptx" | "txt" | "csv" | "epub" | "odt" | "rtf" => "Documents",
        "jpg" | "jpeg" | "png" | "gif" | "webp" | "svg" | "bmp" | "heic" | "avif" => "Images",
        "mp4" | "mkv" | "avi" | "mov" | "webm" | "wmv" => "Video",
        "mp3" | "wav" | "flac" | "m4a" | "ogg" | "aac" => "Music",
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" | "xz" => "Archives",
        "exe" | "msi" | "apk" | "dmg" | "iso" | "msix" => "Programs",
        _ => "",
    }
}

/// Where a download should be saved: Downloads\[Category]\name, never overwriting.
pub fn target(orig: &str, organize: bool) -> String {
    let p = PathBuf::from(orig);
    let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "download".into());
    let base = PathBuf::from(std::env::var("USERPROFILE").unwrap_or_default()).join("Downloads");
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) if !s.is_empty() => (s.to_string(), e.to_string()),
        _ => (name.clone(), String::new()),
    };
    let sub = if organize { category(&ext.to_lowercase()) } else { "" };
    let dir = if sub.is_empty() { base } else { base.join(sub) };
    let _ = std::fs::create_dir_all(&dir);
    let mut n = 0;
    loop {
        let f = match (n, ext.is_empty()) {
            (0, true) => stem.clone(),
            (0, false) => format!("{stem}.{ext}"),
            (_, true) => format!("{stem} ({n})"),
            (_, false) => format!("{stem} ({n}).{ext}"),
        };
        let full = dir.join(f);
        if !full.exists() {
            return full.to_string_lossy().to_string();
        }
        n += 1;
    }
}

pub fn install(wv: &WebView, proxy: EventLoopProxy<Ev>, tab: u32, organize: impl Fn() -> bool + 'static) {
    unsafe {
        PROXY.with(|p| *p.borrow_mut() = Some(proxy.clone()));
        let core = wv.webview();
        let Ok(core4) = core.cast::<ICoreWebView2_4>() else { return };
        let mut token = Default::default();
        let _ = core4.add_DownloadStarting(
            &DownloadStartingEventHandler::create(Box::new(move |_s, args| {
                let Some(args) = args else { return Ok(()) };
                let op = args.DownloadOperation()?;
                let mut p = PWSTR::null();
                op.Uri(&mut p)?;
                let url = take_pwstr(p);
                let mut p = PWSTR::null();
                args.ResultFilePath(&mut p)?;
                let orig = take_pwstr(p);
                let path = target(&orig, organize());
                args.SetResultFilePath(&HSTRING::from(path.as_str()))?;
                args.SetHandled(true)?; // no default Edge download bubble
                let id = NEXT.fetch_add(1, Relaxed);
                OPS.with(|o| o.borrow_mut().insert(id, op.clone()));
                // hold the download until the main loop decides; it may ask the user first
                let def = args.GetDeferral()?;
                HELD.with(|h| h.borrow_mut().insert(id, (args.clone(), def)));
                // gather what the segmented engine needs, then announce the download
                let ua = core
                    .Settings()
                    .ok()
                    .and_then(|s| s.cast::<ICoreWebView2Settings2>().ok())
                    .map(|s| {
                        let mut p = PWSTR::null();
                        if s.UserAgent(&mut p).is_ok() { take_pwstr(p) } else { String::new() }
                    })
                    .unwrap_or_default();
                let referer = {
                    let mut p = PWSTR::null();
                    if core.Source(&mut p).is_ok() { take_pwstr(p) } else { String::new() }
                };
                let announce = {
                    let (proxy, url, path) = (proxy.clone(), url.clone(), path.clone());
                    move || {
                        let _ = proxy.send_event(Ev::DlAsk(id, tab, url, path));
                    }
                };
                let mut ctx = crate::engine::Ctx { url: url.clone(), cookie: String::new(), ua, referer, path: path.clone() };
                let cm = core.cast::<ICoreWebView2_2>().ok().and_then(|c| c.CookieManager().ok());
                match cm {
                    Some(cm) if url.starts_with("http") => {
                        let announce2 = announce.clone();
                        let h = GetCookiesCompletedHandler::create(Box::new(move |_, list| {
                            dbg("cookies cb");
                            if let Some(list) = list {
                                let mut n = 0u32;
                                list.Count(&mut n)?;
                                let mut parts = Vec::new();
                                for i in 0..n.min(200) {
                                    let c = list.GetValueAtIndex(i)?;
                                    let (mut a, mut b) = (PWSTR::null(), PWSTR::null());
                                    c.Name(&mut a)?;
                                    c.Value(&mut b)?;
                                    parts.push(format!("{}={}", take_pwstr(a), take_pwstr(b)));
                                }
                                ctx.cookie = parts.join("; ");
                            }
                            CTX.with(|m| m.borrow_mut().insert(id, ctx.clone()));
                            announce2();
                            Ok(())
                        }));
                        if cm.GetCookies(&HSTRING::from(url.as_str()), &h).is_err() {
                            announce();
                        }
                    }
                    _ => {
                        CTX.with(|m| m.borrow_mut().insert(id, ctx));
                        announce();
                    }
                }

                let (p1, p2) = (proxy.clone(), proxy.clone());
                let mut t1 = Default::default();
                let _ = op.add_BytesReceivedChanged(
                    &BytesReceivedChangedEventHandler::create(Box::new(move |s, _| {
                        if let Some(s) = s {
                            let (mut r, mut t) = (0i64, 0i64);
                            s.BytesReceived(&mut r)?;
                            s.TotalBytesToReceive(&mut t)?;
                            let _ = p1.send_event(Ev::DlProg(id, r, t));
                        }
                        Ok(())
                    })),
                    &mut t1,
                );
                let mut t2 = Default::default();
                let _ = op.add_StateChanged(
                    &StateChangedEventHandler::create(Box::new(move |s, _| {
                        if let Some(s) = s {
                            let mut st = Default::default();
                            s.State(&mut st)?;
                            let _ = p2.send_event(Ev::DlState(id, st.0));
                            if st.0 == 2 {
                                let (mut r, mut t) = (0i64, 0i64);
                                s.BytesReceived(&mut r)?;
                                s.TotalBytesToReceive(&mut t)?;
                                let _ = p2.send_event(Ev::DlProg(id, r, t.max(r)));
                            }
                        }
                        Ok(())
                    })),
                    &mut t2,
                );
                Ok(())
            })),
            &mut token,
        );
    }
}

pub fn pause(id: u32) {
    if crate::engine::pause(id) || crate::torrent::pause(id) {
        return;
    }
    OPS.with(|o| {
        if let Some(op) = o.borrow().get(&id) {
            unsafe {
                let _ = op.Pause();
            }
        }
    });
}
pub fn resume(id: u32) -> bool {
    if crate::torrent::resume(id) {
        return true;
    }
    if let Some(p) = PROXY.with(|p| p.borrow().clone()) {
        if crate::engine::resume(id, p) {
            return true;
        }
    }
    OPS.with(|o| {
        o.borrow().get(&id).map_or(false, |op| unsafe { op.Resume().is_ok() })
    })
}
pub fn cancel(id: u32) {
    if crate::engine::cancel(id) || crate::torrent::cancel(id) {
        return;
    }
    OPS.with(|o| {
        if let Some(op) = o.borrow_mut().remove(&id) {
            unsafe {
                let _ = op.Cancel();
            }
        }
    });
}
pub fn forget(id: u32) {
    CTX.with(|c| {
        c.borrow_mut().remove(&id);
    });
    OPS.with(|o| {
        o.borrow_mut().remove(&id);
    });
}
