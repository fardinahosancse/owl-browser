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
        ICoreWebView2Deferral, ICoreWebView2DownloadOperation, ICoreWebView2DownloadStartingEventArgs, ICoreWebView2_4,
    },
    BytesReceivedChangedEventHandler, DownloadStartingEventHandler, StateChangedEventHandler,
};
use windows::core::{Interface, HSTRING, PWSTR};
use wry::{WebView, WebViewExtWindows};

static NEXT: AtomicU32 = AtomicU32::new(1);

thread_local! {
    static OPS: RefCell<HashMap<u32, ICoreWebView2DownloadOperation>> = RefCell::new(HashMap::new());
    /// Downloads held open until Owl decides whether to allow them.
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
pub fn decide(id: u32, allow: bool) {
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
        let Ok(core4) = wv.webview().cast::<ICoreWebView2_4>() else { return };
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
                let _ = proxy.send_event(Ev::DlAsk(id, tab, url, path));

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
    OPS.with(|o| {
        if let Some(op) = o.borrow().get(&id) {
            unsafe {
                let _ = op.Pause();
            }
        }
    });
}
pub fn resume(id: u32) -> bool {
    OPS.with(|o| {
        o.borrow().get(&id).map_or(false, |op| unsafe { op.Resume().is_ok() })
    })
}
pub fn cancel(id: u32) {
    OPS.with(|o| {
        if let Some(op) = o.borrow_mut().remove(&id) {
            unsafe {
                let _ = op.Cancel();
            }
        }
    });
}
pub fn forget(id: u32) {
    OPS.with(|o| {
        o.borrow_mut().remove(&id);
    });
}
