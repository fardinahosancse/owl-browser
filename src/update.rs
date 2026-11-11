//! Notify-only update check. It asks GitHub for the latest release tag (no identifiers are sent,
//! only a generic user agent), never downloads or installs anything, and is skipped when a proxy
//! is set so a Tor or VPN user's address is not revealed.
use crate::Ev;
use std::ffi::c_void;
use tao::event_loop::EventLoopProxy;
use windows::core::{w, PCWSTR};
use windows::Win32::Networking::WinHttp::*;

const HOST: PCWSTR = w!("api.github.com");
const PATH: PCWSTR = w!("/repos/fardinahosancse/owl-browser/releases/latest");

fn fetch() -> Option<String> {
    unsafe {
        let s = WinHttpOpen(w!("Owl-update-check"), WINHTTP_ACCESS_TYPE_AUTOMATIC_PROXY, PCWSTR::null(), PCWSTR::null(), 0);
        if s.is_null() {
            return None;
        }
        let mut out = None;
        let c = WinHttpConnect(s, HOST, INTERNET_DEFAULT_HTTPS_PORT, 0);
        if !c.is_null() {
            let r = WinHttpOpenRequest(c, w!("GET"), PATH, PCWSTR::null(), PCWSTR::null(), std::ptr::null(), WINHTTP_FLAG_SECURE);
            if !r.is_null() {
                let _ = WinHttpSetTimeouts(r, 5000, 5000, 5000, 8000);
                if WinHttpSendRequest(r, None, None, 0, 0, 0).is_ok() && WinHttpReceiveResponse(r, std::ptr::null_mut()).is_ok() {
                    let mut buf: Vec<u8> = Vec::new();
                    loop {
                        let mut chunk = [0u8; 4096];
                        let mut n = 0u32;
                        if WinHttpReadData(r, chunk.as_mut_ptr() as *mut c_void, chunk.len() as u32, &mut n).is_err() || n == 0 {
                            break;
                        }
                        buf.extend_from_slice(&chunk[..n as usize]);
                        if buf.len() > 256 * 1024 {
                            break;
                        }
                    }
                    out = String::from_utf8(buf).ok();
                }
                let _ = WinHttpCloseHandle(r);
            }
            let _ = WinHttpCloseHandle(c);
        }
        let _ = WinHttpCloseHandle(s);
        out
    }
}

fn parse(v: &str) -> Option<(u32, u32, u32)> {
    let mut it = v.trim_start_matches('v').split(['.', '-']).map(|p| p.parse::<u32>().ok());
    Some((it.next()??, it.next()??, it.next()??))
}

/// Runs in the background; sends `Ev::Update(latest, manual)` with an empty string when current.
pub fn check(proxy: EventLoopProxy<Ev>, manual: bool) {
    std::thread::spawn(move || {
        let latest = fetch()
            .and_then(|b| serde_json::from_str::<serde_json::Value>(&b).ok())
            .and_then(|j| j["tag_name"].as_str().map(String::from));
        let newer = latest
            .as_deref()
            .and_then(|l| Some((parse(l)?, parse(env!("CARGO_PKG_VERSION"))?)))
            .map_or(false, |(l, c)| l > c);
        let reached = latest.is_some();
        let _ = proxy.send_event(Ev::Update(if newer { latest.unwrap_or_default() } else { String::new() }, manual, reached));
    });
}
