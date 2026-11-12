//! Secure DNS through Cloudflare 1.1.1.1.
//!
//! The page engine cannot be told to use DNS over HTTPS, so Owl runs a small SOCKS5 proxy that
//! listens on the loopback address only. The engine sends every connection through it, and the
//! proxy looks up names over HTTPS at 1.1.1.1 before connecting. This hides which sites you visit
//! from your network's DNS server. It is not a VPN: your IP address is unchanged, and the site
//! names still show in TLS handshakes.
//!
//! If 1.1.1.1 cannot be reached, lookups fail rather than falling back to the system resolver.
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{
        atomic::{AtomicU16, Ordering::Relaxed},
        Mutex, OnceLock,
    },
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    runtime::Runtime,
};

/// Port of the running proxy, 0 when off.
pub static PORT: AtomicU16 = AtomicU16::new(0);

static RT: OnceLock<Runtime> = OnceLock::new();
static CACHE: Mutex<Option<HashMap<String, (Instant, Vec<IpAddr>)>>> = Mutex::new(None);

pub fn active() -> bool {
    PORT.load(Relaxed) != 0
}

/// Start the loopback proxy; returns its port.
pub fn start() -> Option<u16> {
    let rt = RT.get_or_init(|| tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().thread_name("owl-doh").build().expect("runtime"));
    let std_l = std::net::TcpListener::bind("127.0.0.1:0").ok()?;
    std_l.set_nonblocking(true).ok()?;
    let port = std_l.local_addr().ok()?.port();
    rt.spawn(async move {
        let Ok(l) = TcpListener::from_std(std_l) else { return };
        loop {
            if let Ok((s, _)) = l.accept().await {
                tokio::spawn(async move {
                    let _ = handle(s).await;
                });
            }
        }
    });
    PORT.store(port, Relaxed);
    Some(port)
}

async fn handle(mut s: TcpStream) -> std::io::Result<()> {
    let bad = |m: &'static str| std::io::Error::new(std::io::ErrorKind::InvalidData, m);
    // greeting: version, method list; we only offer "no authentication" (loopback only)
    let mut h = [0u8; 2];
    s.read_exact(&mut h).await?;
    if h[0] != 5 {
        return Err(bad("socks version"));
    }
    let mut methods = vec![0u8; h[1] as usize];
    s.read_exact(&mut methods).await?;
    s.write_all(&[5, 0]).await?;
    // request
    let mut r = [0u8; 4];
    s.read_exact(&mut r).await?;
    if r[0] != 5 || r[1] != 1 {
        s.write_all(&[5, 7, 0, 1, 0, 0, 0, 0, 0, 0]).await?; // only CONNECT
        return Err(bad("command"));
    }
    let host = match r[3] {
        1 => {
            let mut a = [0u8; 4];
            s.read_exact(&mut a).await?;
            IpAddr::from(a).to_string()
        }
        3 => {
            let mut l = [0u8; 1];
            s.read_exact(&mut l).await?;
            let mut n = vec![0u8; l[0] as usize];
            s.read_exact(&mut n).await?;
            String::from_utf8(n).map_err(|_| bad("name"))?
        }
        4 => {
            let mut a = [0u8; 16];
            s.read_exact(&mut a).await?;
            IpAddr::from(a).to_string()
        }
        _ => return Err(bad("address type")),
    };
    let mut p = [0u8; 2];
    s.read_exact(&mut p).await?;
    let port = u16::from_be_bytes(p);
    let ips = resolve(&host).await;
    let mut up = None;
    for ip in ips {
        if let Ok(Ok(c)) = tokio::time::timeout(Duration::from_secs(10), TcpStream::connect((ip, port))).await {
            up = Some(c);
            break;
        }
    }
    let Some(mut up) = up else {
        s.write_all(&[5, 4, 0, 1, 0, 0, 0, 0, 0, 0]).await?; // host unreachable
        return Err(bad("connect"));
    };
    s.write_all(&[5, 0, 0, 1, 0, 0, 0, 0, 0, 0]).await?;
    let _ = tokio::io::copy_bidirectional(&mut s, &mut up).await;
    Ok(())
}

fn valid_name(h: &str) -> bool {
    !h.is_empty() && h.len() <= 253 && h.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_')
}

async fn resolve(host: &str) -> Vec<IpAddr> {
    if let Ok(ip) = host.trim_matches(|c| c == '[' || c == ']').parse::<IpAddr>() {
        return vec![ip];
    }
    let h = host.trim_end_matches('.').to_lowercase();
    // local names never leave the machine as DoH queries
    if h == "localhost" || h.ends_with(".localhost") || h.ends_with(".local") || !h.contains('.') {
        return tokio::net::lookup_host((h.as_str(), 0)).await.map(|i| i.map(|a| a.ip()).collect()).unwrap_or_default();
    }
    if !valid_name(&h) {
        return vec![];
    }
    if let Some((at, ips)) = CACHE.lock().unwrap().as_ref().and_then(|c| c.get(&h)).cloned() {
        if at.elapsed() < Duration::from_secs(300) {
            return ips;
        }
    }
    let q = h.clone();
    let ips = tokio::task::spawn_blocking(move || {
        let mut out: Vec<IpAddr> = Vec::new();
        for (ty, code) in [("A", 1u64), ("AAAA", 28)] {
            let path = format!("/dns-query?name={q}&type={ty}");
            if let Some(body) = crate::update::fetch_with("1.1.1.1", &path, 64 * 1024, "application/dns-json") {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
                    for a in v["Answer"].as_array().into_iter().flatten() {
                        if a["type"].as_u64() == Some(code) {
                            if let Some(ip) = a["data"].as_str().and_then(|d| d.parse::<IpAddr>().ok()) {
                                out.push(ip);
                            }
                        }
                    }
                }
            }
        }
        out
    })
    .await
    .unwrap_or_default();
    if !ips.is_empty() {
        let mut g = CACHE.lock().unwrap();
        let c = g.get_or_insert_with(HashMap::new);
        if c.len() > 2000 {
            c.clear();
        }
        c.insert(h, (Instant::now(), ips.clone()));
    }
    ips
}
