//! Segmented download engine. Large files are split into parts that download over several
//! connections at once (like IDM or FDM), can be paused and resumed, and retry on errors.
//!
//! The browser's own download is held until `probe` shows the server supports ranges and the
//! file is big enough; otherwise the normal download simply continues. The page's cookies and
//! user agent are sent only to the host the download started from, never to a redirect target.
use crate::Ev;
use std::{
    collections::HashMap,
    ffi::c_void,
    fs::File,
    io::ErrorKind,
    os::windows::fs::FileExt,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicI64, AtomicU32, AtomicU64, AtomicU8, Ordering::Relaxed},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};
use tao::event_loop::EventLoopProxy;
use windows::core::{HSTRING, PCWSTR};
use windows::Win32::Networking::WinHttp::*;

/// Smallest file worth splitting.
pub const MIN_SIZE: i64 = 2 * 1024 * 1024;
const MAX_PARTS: i64 = 8;
const CHUNK: usize = 64 * 1024;

/// False while a proxy is configured: the engine connects directly, so it must stay out of the way
/// and let the browser (which honours the proxy) download.
pub static DIRECT_OK: AtomicBool = AtomicBool::new(true);

#[derive(Clone, Default)]
pub struct Ctx {
    pub url: String,
    pub cookie: String,
    pub ua: String,
    pub referer: String,
    pub path: String,
}

pub struct Probe {
    pub url: String,
    pub total: i64,
    pub with_cookie: bool,
}

struct Seg {
    start: i64,
    end: i64, // inclusive
    pos: AtomicI64,
}

/// Download speed limit in KB/s shared by every segment (0 = unlimited).
pub static LIMIT_KB: AtomicU64 = AtomicU64::new(0);
/// How many segmented downloads run at once; the rest wait in the queue.
pub static MAX_ACTIVE: AtomicU32 = AtomicU32::new(3);
static BUCKET: Mutex<Option<(Instant, f64)>> = Mutex::new(None);

/// Token bucket: blocks the calling segment long enough to keep the total under the limit.
fn throttle(n: usize) {
    let kb = LIMIT_KB.load(Relaxed);
    if kb == 0 {
        return;
    }
    let rate = kb as f64 * 1024.0;
    let wait = {
        let mut g = BUCKET.lock().unwrap();
        let (at, tokens) = g.get_or_insert((Instant::now(), rate));
        let now = Instant::now();
        *tokens = (*tokens + now.duration_since(*at).as_secs_f64() * rate).min(rate);
        *at = now;
        *tokens -= n as f64;
        if *tokens < 0.0 { -*tokens / rate } else { 0.0 }
    };
    if wait > 0.0 {
        std::thread::sleep(Duration::from_secs_f64(wait.min(2.0)));
    }
}

const QUEUE: u8 = 3;
const RUN: u8 = 0;
const PAUSE: u8 = 1;
const CANCEL: u8 = 2;

pub struct Job {
    ctx: Ctx,
    probe: Probe,
    segs: Vec<Seg>,
    state: AtomicU8,
    failed: AtomicBool,
    running: AtomicBool,
    file: File,
    part: PathBuf,
}

static JOBS: Mutex<Option<HashMap<u32, Arc<Job>>>> = Mutex::new(None);

fn jobs<R>(f: impl FnOnce(&mut HashMap<u32, Arc<Job>>) -> R) -> R {
    let mut g = JOBS.lock().unwrap();
    f(g.get_or_insert_with(HashMap::new))
}

pub fn has(id: u32) -> bool {
    jobs(|j| j.contains_key(&id))
}

// ---------- WinHTTP plumbing ----------

struct Handles(Vec<*mut c_void>);
impl Drop for Handles {
    fn drop(&mut self) {
        for h in self.0.drain(..).rev() {
            unsafe {
                let _ = WinHttpCloseHandle(h);
            }
        }
    }
}

struct Resp {
    h: Handles,
    status: u32,
}

fn split_url(url: &str) -> Option<(bool, String, u16, String)> {
    let (secure, rest) = if let Some(r) = url.strip_prefix("https://") {
        (true, r)
    } else if let Some(r) = url.strip_prefix("http://") {
        (false, r)
    } else {
        return None;
    };
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (auth, tail) = rest.split_at(end);
    if auth.contains('@') || auth.is_empty() {
        return None; // no embedded credentials
    }
    let (host, port) = match auth.rsplit_once(':') {
        Some((h, p)) if !auth.ends_with(']') => (h.to_string(), p.parse().ok()?),
        _ => (auth.to_string(), if secure { 443 } else { 80 }),
    };
    let tail = tail.split('#').next().unwrap_or("");
    let path = if tail.starts_with('/') { tail.to_string() } else { format!("/{tail}") };
    Some((secure, host, port, path))
}

fn host_only(url: &str) -> String {
    split_url(url).map(|x| x.1.to_lowercase()).unwrap_or_default()
}

/// One GET; redirects are not followed here so credentials can be controlled.
fn get(url: &str, headers: &[(&str, String)]) -> Option<Resp> {
    let (secure, host, port, path) = split_url(url)?;
    unsafe {
        let s = WinHttpOpen(PCWSTR::null(), WINHTTP_ACCESS_TYPE_NO_PROXY, PCWSTR::null(), PCWSTR::null(), 0);
        if s.is_null() {
            return None;
        }
        let mut h = Handles(vec![s]);
        let hh = HSTRING::from(host.as_str());
        let c = WinHttpConnect(s, PCWSTR(hh.as_ptr()), port, 0);
        if c.is_null() {
            return None;
        }
        h.0.push(c);
        let pp = HSTRING::from(path.as_str());
        let r = WinHttpOpenRequest(c, windows::core::w!("GET"), PCWSTR(pp.as_ptr()), PCWSTR::null(), PCWSTR::null(), std::ptr::null(), if secure { WINHTTP_FLAG_SECURE } else { WINHTTP_OPEN_REQUEST_FLAGS(0) });
        if r.is_null() {
            return None;
        }
        h.0.push(r);
        // no automatic redirects and no automatic cookies: both are handled explicitly
        let never = WINHTTP_OPTION_REDIRECT_POLICY_NEVER.to_le_bytes();
        let _ = WinHttpSetOption(Some(r as *const c_void), WINHTTP_OPTION_REDIRECT_POLICY, Some(&never));
        let nocookie = WINHTTP_DISABLE_COOKIES.to_le_bytes();
        let _ = WinHttpSetOption(Some(r as *const c_void), WINHTTP_OPTION_DISABLE_FEATURE, Some(&nocookie));
        let _ = WinHttpSetTimeouts(r, 8000, 8000, 15000, 30000);
        let mut hdr = String::new();
        for (k, v) in headers {
            if !v.is_empty() && !v.contains(['\r', '\n']) {
                hdr.push_str(&format!("{k}: {v}\r\n"));
            }
        }
        if !hdr.is_empty() {
            let w: Vec<u16> = hdr.encode_utf16().collect();
            let _ = WinHttpAddRequestHeaders(r, &w, WINHTTP_ADDREQ_FLAG_ADD | WINHTTP_ADDREQ_FLAG_REPLACE);
        }
        if let Err(e) = WinHttpSendRequest(r, None, None, 0, 0, 0) {
            crate::dl::dbg(&format!("send failed {e}"));
            return None;
        }
        if let Err(e) = WinHttpReceiveResponse(r, std::ptr::null_mut()) {
            crate::dl::dbg(&format!("recv failed {e}"));
            return None;
        }
        let mut status = 0u32;
        let mut len = 4u32;
        WinHttpQueryHeaders(r, WINHTTP_QUERY_STATUS_CODE | WINHTTP_QUERY_FLAG_NUMBER, PCWSTR::null(), Some(&mut status as *mut u32 as *mut c_void), &mut len, std::ptr::null_mut()).ok()?;
        Some(Resp { h, status })
    }
}

fn header(r: &Resp, which: u32) -> String {
    unsafe {
        let req = *r.h.0.last().unwrap();
        let mut len = 0u32;
        let _ = WinHttpQueryHeaders(req, which, PCWSTR::null(), None, &mut len, std::ptr::null_mut());
        if len == 0 || len > 16384 {
            return String::new();
        }
        let mut buf = vec![0u16; (len as usize) / 2 + 1];
        if WinHttpQueryHeaders(req, which, PCWSTR::null(), Some(buf.as_mut_ptr() as *mut c_void), &mut len, std::ptr::null_mut()).is_err() {
            return String::new();
        }
        String::from_utf16_lossy(&buf[..(len as usize) / 2])
    }
}

fn read(r: &Resp, buf: &mut [u8]) -> std::io::Result<usize> {
    unsafe {
        let req = *r.h.0.last().unwrap();
        let mut n = 0u32;
        WinHttpReadData(req, buf.as_mut_ptr() as *mut c_void, buf.len() as u32, &mut n).map_err(|_| std::io::Error::new(ErrorKind::Other, "read"))?;
        Ok(n as usize)
    }
}

fn resolve(base: &str, loc: &str) -> Option<String> {
    let (secure, host, port, path) = split_url(base)?;
    let scheme = if secure { "https" } else { "http" };
    let default = if secure { 443 } else { 80 };
    let origin = if port == default { format!("{scheme}://{host}") } else { format!("{scheme}://{host}:{port}") };
    let loc = loc.trim();
    Some(if loc.starts_with("http://") || loc.starts_with("https://") {
        loc.to_string()
    } else if let Some(r) = loc.strip_prefix("//") {
        format!("{scheme}://{r}")
    } else if loc.starts_with('/') {
        format!("{origin}{loc}")
    } else {
        let dir = path.rsplit_once('/').map(|x| x.0).unwrap_or("");
        format!("{origin}{dir}/{loc}")
    })
}

fn base_headers(ctx: &Ctx, with_cookie: bool, range: &str) -> Vec<(&'static str, String)> {
    let mut v = vec![("Range", range.to_string()), ("Accept", "*/*".to_string()), ("Accept-Encoding", "identity".to_string())];
    if !ctx.ua.is_empty() {
        v.push(("User-Agent", ctx.ua.clone()));
    }
    if with_cookie {
        v.push(("Cookie", ctx.cookie.clone()));
        v.push(("Referer", ctx.referer.clone()));
    }
    v
}

/// Ask the server about the file. Runs on a worker thread and reports with `Ev::DlProbe`.
pub fn probe(id: u32, ctx: Ctx, proxy: EventLoopProxy<Ev>) {
    std::thread::spawn(move || {
        let r = probe_blocking(&ctx);
        crate::dl::dbg(&format!("probe {id} -> {:?}", r.as_ref().map(|p| (p.total, p.with_cookie))));
        let _ = proxy.send_event(Ev::DlProbe(id, r));
    });
}

fn probe_blocking(ctx: &Ctx) -> Option<Probe> {
    let origin_host = host_only(&ctx.url);
    let mut url = ctx.url.clone();
    for _ in 0..8 {
        let with_cookie = host_only(&url) == origin_host;
        let r = get(&url, &base_headers(ctx, with_cookie, "bytes=0-0"))?;
        match r.status {
            301 | 302 | 303 | 307 | 308 => {
                let loc = header(&r, WINHTTP_QUERY_LOCATION);
                url = resolve(&url, &loc)?;
            }
            206 => {
                // "bytes 0-0/12345"
                let cr = header(&r, WINHTTP_QUERY_CONTENT_RANGE);
                let total: i64 = cr.rsplit('/').next()?.trim().parse().ok()?;
                return Some(Probe { url, total, with_cookie });
            }
            _ => return None, // no range support, auth needed or an error: let the browser handle it
        }
    }
    None
}

// ---------- jobs ----------

/// Begin a segmented download. `None` if the file could not be prepared, `Some(true)` if it was
/// queued behind other downloads.
pub fn start(id: u32, ctx: Ctx, probe: Probe, proxy: EventLoopProxy<Ev>) -> Option<bool> {
    let part = PathBuf::from(format!("{}.owlpart", ctx.path));
    let Ok(file) = std::fs::OpenOptions::new().write(true).create_new(true).open(&part) else { return None };
    if file.set_len(probe.total as u64).is_err() {
        let _ = std::fs::remove_file(&part);
        return None;
    }
    let n = (probe.total / MIN_SIZE).clamp(1, MAX_PARTS);
    let size = (probe.total + n - 1) / n;
    let segs: Vec<Seg> = (0..n)
        .map(|i| {
            let s = i * size;
            Seg { start: s, end: ((i + 1) * size - 1).min(probe.total - 1), pos: AtomicI64::new(s) }
        })
        .filter(|s| s.start <= s.end)
        .collect();
    let job = Arc::new(Job { ctx, probe, segs, state: AtomicU8::new(RUN), failed: AtomicBool::new(false), running: AtomicBool::new(false), file, part });
    Some(launch(id, job, proxy))
}

fn active_count() -> usize {
    jobs(|j| j.values().filter(|x| x.running.load(Relaxed)).count())
}

/// Register the job and run it now, or queue it when enough downloads are already running.
fn launch(id: u32, job: Arc<Job>, proxy: EventLoopProxy<Ev>) -> bool {
    let queued = active_count() >= MAX_ACTIVE.load(Relaxed).max(1) as usize;
    jobs(|j| j.insert(id, job.clone()));
    save_state(&job);
    if queued {
        job.state.store(QUEUE, Relaxed);
    } else {
        run(id, job, proxy);
    }
    queued
}

/// Start the oldest queued job if there is room.
fn kick(proxy: &EventLoopProxy<Ev>) {
    if active_count() >= MAX_ACTIVE.load(Relaxed).max(1) as usize {
        return;
    }
    let next = jobs(|j| j.iter().filter(|(_, x)| x.state.load(Relaxed) == QUEUE && !x.running.load(Relaxed)).map(|(i, x)| (*i, x.clone())).min_by_key(|(i, _)| *i));
    if let Some((id, job)) = next {
        let _ = proxy.send_event(Ev::DlQueue(id, false));
        run(id, job, proxy.clone());
    }
}

fn state_path(job: &Job) -> PathBuf {
    PathBuf::from(format!("{}.json", job.part.to_string_lossy()))
}

/// Remember how far each part got, so a download can continue after Owl restarts.
fn save_state(job: &Job) {
    let segs: Vec<[i64; 3]> = job.segs.iter().map(|s| [s.start, s.end, s.pos.load(Relaxed)]).collect();
    let v = serde_json::json!({"url": job.probe.url, "total": job.probe.total, "ua": job.ctx.ua, "referer": job.ctx.referer, "path": job.ctx.path, "segs": segs});
    let _ = std::fs::write(state_path(job), v.to_string());
}

/// Continue an interrupted download from its saved state. Cookies are not kept, so downloads
/// that need a signed-in session may fail and must be started again from the page.
pub fn restore(id: u32, path: &str, proxy: EventLoopProxy<Ev>) -> bool {
    if !DIRECT_OK.load(Relaxed) || has(id) {
        return false;
    }
    let part = PathBuf::from(format!("{path}.owlpart"));
    let Ok(txt) = std::fs::read_to_string(format!("{}.json", part.to_string_lossy())) else { return false };
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else { return false };
    let (Some(url), Some(total)) = (v["url"].as_str(), v["total"].as_i64()) else { return false };
    if !(url.starts_with("http://") || url.starts_with("https://")) || v["path"].as_str() != Some(path) {
        return false;
    }
    let Ok(file) = std::fs::OpenOptions::new().write(true).open(&part) else { return false };
    if file.metadata().map_or(true, |m| m.len() != total as u64) {
        return false;
    }
    let mut segs = Vec::new();
    for s in v["segs"].as_array().into_iter().flatten() {
        let (Some(a), Some(b), Some(p)) = (s[0].as_i64(), s[1].as_i64(), s[2].as_i64()) else { return false };
        if a < 0 || b >= total || p < a || p > b + 1 {
            return false;
        }
        segs.push(Seg { start: a, end: b, pos: AtomicI64::new(p) });
    }
    if segs.is_empty() {
        return false;
    }
    let ctx = Ctx { url: url.to_string(), cookie: String::new(), ua: v["ua"].as_str().unwrap_or("").to_string(), referer: v["referer"].as_str().unwrap_or("").to_string(), path: path.to_string() };
    let probe = Probe { url: url.to_string(), total, with_cookie: false };
    let job = Arc::new(Job { ctx, probe, segs, state: AtomicU8::new(PAUSE), failed: AtomicBool::new(false), running: AtomicBool::new(false), file, part });
    launch(id, job, proxy);
    true
}


fn received(job: &Job) -> i64 {
    job.segs.iter().map(|s| s.pos.load(Relaxed) - s.start).sum()
}

fn run(id: u32, job: Arc<Job>, proxy: EventLoopProxy<Ev>) {
    if job.running.swap(true, Relaxed) {
        return;
    }
    job.state.store(RUN, Relaxed);
    job.failed.store(false, Relaxed);
    std::thread::spawn(move || {
        let handles: Vec<_> = (0..job.segs.len())
            .map(|i| {
                let j = job.clone();
                std::thread::spawn(move || segment(&j, i))
            })
            .collect();
        let mut ticks = 0u32;
        while handles.iter().any(|h| !h.is_finished()) {
            std::thread::sleep(Duration::from_millis(250));
            ticks += 1;
            if ticks % 8 == 0 {
                save_state(&job);
            }
            let _ = proxy.send_event(Ev::DlProg(id, received(&job), job.probe.total));
        }
        for h in handles {
            let _ = h.join();
        }
        job.running.store(false, Relaxed);
        let _ = proxy.send_event(Ev::DlProg(id, received(&job), job.probe.total));
        match job.state.load(Relaxed) {
            CANCEL => {
                let _ = std::fs::remove_file(&job.part);
                let _ = std::fs::remove_file(state_path(&job));
                jobs(|j| j.remove(&id));
            }
            PAUSE => save_state(&job),
            _ => {
                let complete = job.segs.iter().all(|s| s.pos.load(Relaxed) > s.end);
                if complete && !job.failed.load(Relaxed) {
                    let _ = job.file.sync_all();
                    if std::fs::rename(&job.part, &job.ctx.path).is_ok() {
                        let _ = std::fs::remove_file(state_path(&job));
                        jobs(|j| j.remove(&id));
                        let _ = proxy.send_event(Ev::DlState(id, 2));
                        return;
                    }
                }
                // keep the job so Resume can try again
                save_state(&job);
                let _ = proxy.send_event(Ev::DlState(id, 1));
            }
        }
        kick(&proxy);
    });
}

fn segment(job: &Job, i: usize) {
    let seg = &job.segs[i];
    let mut tries = 0;
    let mut buf = vec![0u8; CHUNK];
    while seg.pos.load(Relaxed) <= seg.end {
        if job.state.load(Relaxed) != RUN {
            return;
        }
        let pos = seg.pos.load(Relaxed);
        let range = format!("bytes={}-{}", pos, seg.end);
        let ok = (|| {
            let r = get(&job.probe.url, &base_headers(&job.ctx, job.probe.with_cookie, &range))?;
            if r.status != 206 {
                return None;
            }
            loop {
                if job.state.load(Relaxed) != RUN {
                    return Some(());
                }
                let n = read(&r, &mut buf).ok()?;
                if n == 0 {
                    return Some(());
                }
                let p = seg.pos.load(Relaxed);
                let n = n.min((seg.end - p + 1) as usize);
                throttle(n);
                job.file.seek_write(&buf[..n], p as u64).ok()?;
                seg.pos.store(p + n as i64, Relaxed);
                if p + n as i64 > seg.end {
                    return Some(());
                }
            }
        })();
        if ok.is_some() {
            if seg.pos.load(Relaxed) <= seg.end && job.state.load(Relaxed) == RUN {
                // the server closed early; count it as a failed attempt
                tries += 1;
            } else {
                tries = 0;
            }
        } else {
            tries += 1;
        }
        if tries > 5 {
            job.failed.store(true, Relaxed);
            return;
        }
        if tries > 0 {
            std::thread::sleep(Duration::from_secs(1 << (tries - 1).min(4)));
        }
    }
}

pub fn pause(id: u32) -> bool {
    jobs(|j| j.get(&id).map(|x| x.state.store(PAUSE, Relaxed)).is_some())
}

pub fn resume(id: u32, proxy: EventLoopProxy<Ev>) -> bool {
    match jobs(|j| j.get(&id).cloned()) {
        Some(job) => {
            // wait for the previous supervisor to have stopped
            if job.state.load(Relaxed) == PAUSE || job.failed.load(Relaxed) {
                run(id, job, proxy);
            }
            true
        }
        None => false,
    }
}

pub fn cancel(id: u32) -> bool {
    match jobs(|j| j.get(&id).cloned()) {
        Some(job) => {
            job.state.store(CANCEL, Relaxed);
            // a paused or failed job has no supervisor running: clean up here
            let _ = std::fs::remove_file(&job.part);
            let _ = std::fs::remove_file(state_path(&job));
            jobs(|j| j.remove(&id));
            true
        }
        None => false,
    }
}
