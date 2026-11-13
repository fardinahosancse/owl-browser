//! BitTorrent downloads (magnet links and .torrent files) using the librqbit engine.
//! Torrents appear in the normal download list. They stop seeding once complete, and are
//! refused while a proxy is set because peers and trackers would see the real address.
use crate::Ev;
use librqbit::{api::TorrentIdOrHash, AddTorrent, AddTorrentOptions, AddTorrentResponse, ManagedTorrent, Session, SessionOptions};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use tokio::runtime::Runtime;

/// Keep seeding after a download completes (off by default).
pub static SEED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

static RT: OnceLock<Runtime> = OnceLock::new();
/// The torrent session exists only while a torrent is active: it starts when a magnet link or
/// .torrent file arrives and is shut down (DHT, trackers, peers) when the last one ends.
static SESSION: tokio::sync::Mutex<Option<Arc<Session>>> = tokio::sync::Mutex::const_new(None);
/// Torrents being added right now, so an ending torrent does not stop their session.
static PENDING: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static JOBS: Mutex<Option<HashMap<u32, Arc<ManagedTorrent>>>> = Mutex::new(None);

fn rt() -> &'static Runtime {
    RT.get_or_init(|| tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().thread_name("owl-torrent").build().expect("runtime"))
}

fn jobs<R>(f: impl FnOnce(&mut HashMap<u32, Arc<ManagedTorrent>>) -> R) -> R {
    let mut g = JOBS.lock().unwrap();
    f(g.get_or_insert_with(HashMap::new))
}

pub fn has(id: u32) -> bool {
    jobs(|j| j.contains_key(&id))
}

pub fn is_magnet(u: &str) -> bool {
    u.len() < 4096 && u.starts_with("magnet:?") && u.contains("xt=urn:btih:")
}

async fn session() -> anyhow::Result<Arc<Session>> {
    let mut g = SESSION.lock().await;
    if let Some(s) = g.as_ref() {
        return Ok(s.clone());
    }
    let out: PathBuf = crate::dl::root();
    let _ = std::fs::create_dir_all(&out);
    // no persisted DHT state on disk and no listening port: Owl only connects out
    let opts = SessionOptions {
        dht: Some(librqbit::DhtSessionConfig { bootstrap_addrs: None, port: None, persistence: None }),
        ..Default::default()
    };
    let s = Session::new_with_opts(out, opts).await?;
    *g = Some(s.clone());
    Ok(s)
}

/// Shut the session down when no torrent is active or being added.
async fn stop_if_idle() {
    if PENDING.load(std::sync::atomic::Ordering::Relaxed) > 0 || jobs(|j| !j.is_empty()) {
        return;
    }
    let s = SESSION.lock().await.take();
    if let Some(s) = s {
        s.stop().await;
    }
}

/// True while the torrent engine is running.
pub fn running() -> bool {
    SESSION.try_lock().map_or(true, |g| g.is_some())
}

/// What to add: a magnet link or the bytes of a .torrent file.
pub enum Source {
    Magnet(String),
    File(Vec<u8>),
}

/// Add a torrent; progress and the final state arrive as ordinary download events.
/// `named` receives (name, total bytes) once the torrent's metadata is known.
pub fn add(id: u32, src: Source, send: impl Fn(Ev) + Send + Sync + 'static) {
    PENDING.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    rt().spawn(async move {
        let fail = || send(Ev::DlState(id, 1));
        let s = match session().await {
            Ok(s) => s,
            Err(e) => {
                crate::dl::dbg(&format!("torrent session: {e:#}"));
                #[cfg(test)]
                eprintln!("session error: {e:#}");
                PENDING.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                stop_if_idle().await;
                return fail();
            }
        };
        let add = match src {
            Source::Magnet(m) => AddTorrent::from_url(m),
            Source::File(b) => AddTorrent::from_bytes(b),
        };
        let opts = AddTorrentOptions { overwrite: false, ..Default::default() };
        let handle = match s.add_torrent(add, Some(opts)).await {
            Ok(AddTorrentResponse::Added(_, h)) | Ok(AddTorrentResponse::AlreadyManaged(_, h)) => h,
            other => {
                #[cfg(test)]
                eprintln!("add error: {:?}", other.err());
                PENDING.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
                stop_if_idle().await;
                return fail();
            }
        };
        jobs(|j| j.insert(id, handle.clone()));
        PENDING.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
        send(Ev::TorrentName(id, handle.name().unwrap_or_default()));
        let mut seeded_stop = false;
        loop {
            tokio::time::sleep(Duration::from_millis(700)).await;
            if !has(id) {
                stop_if_idle().await;
                return; // cancelled
            }
            let st = handle.stats();
            send(Ev::DlProg(id, st.progress_bytes as i64, st.total_bytes as i64));
            if st.error.is_some() {
                jobs(|j| j.remove(&id));
                fail();
                stop_if_idle().await;
                return;
            }
            if st.finished && !seeded_stop && SEED.load(std::sync::atomic::Ordering::Relaxed) {
                // the user chose to seed: report it as done but keep the torrent and engine running
                seeded_stop = true;
                send(Ev::DlState(id, 2));
                return;
            }
            if st.finished && !seeded_stop {
                seeded_stop = true;
                let _ = s.pause(&handle).await; // stop seeding when complete
                send(Ev::DlState(id, 2));
                jobs(|j| j.remove(&id));
                stop_if_idle().await; // nothing keeps seeding
                return;
            }
        }
    });
}

/// End seeding of a finished torrent without deleting its files.
pub fn stop(id: u32) {
    let Some(h) = jobs(|j| j.remove(&id)) else { return };
    rt().spawn(async move {
        if let Ok(s) = session().await {
            let _ = s.pause(&h).await;
        }
        stop_if_idle().await;
    });
}

/// Live details for the download list: peers, seeders, speeds, state.
pub fn live(id: u32) -> Option<serde_json::Value> {
    let h = jobs(|j| j.get(&id).cloned())?;
    let st = serde_json::to_value(h.stats()).ok()?;
    let l = &st["live"];
    let ps = &l["snapshot"]["peer_stats"];
    let mbps = |v: &serde_json::Value| (v["mbps"].as_f64().unwrap_or(0.0) * 1024.0 * 1024.0) as u64;
    let state = if st["finished"].as_bool().unwrap_or(false) {
        "seeding"
    } else if st["state"].as_str() == Some("paused") {
        "paused"
    } else if st["state"].as_str() == Some("initializing") || st["total_bytes"].as_u64().unwrap_or(0) == 0 {
        "fetching details"
    } else {
        "downloading"
    };
    Some(serde_json::json!({
        "peers": ps["live"].as_u64().unwrap_or(0),
        "seeders": ps["live_seeders"].as_u64().unwrap_or(0),
        "seen": ps["seen"].as_u64().unwrap_or(0),
        "down": mbps(&l["download_speed"]),
        "up": mbps(&l["upload_speed"]),
        "uploaded": st["uploaded_bytes"].as_u64().unwrap_or(0),
        "state": state,
    }))
}

pub fn pause(id: u32) -> bool {
    let Some(h) = jobs(|j| j.get(&id).cloned()) else { return false };
    rt().spawn(async move {
        if let Ok(s) = session().await {
            let _ = s.pause(&h).await;
        }
    });
    true
}

pub fn resume(id: u32) -> bool {
    let Some(h) = jobs(|j| j.get(&id).cloned()) else { return false };
    rt().spawn(async move {
        if let Ok(s) = session().await {
            let _ = s.unpause(&h).await;
        }
    });
    true
}

pub fn cancel(id: u32) -> bool {
    let Some(h) = jobs(|j| j.remove(&id)) else { return false };
    rt().spawn(async move {
        if let Ok(s) = session().await {
            let _ = s.delete(TorrentIdOrHash::Id(h.id()), true).await;
        }
        stop_if_idle().await;
    });
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Needs the internet: adds a public-domain test torrent and expects metadata and progress.
    #[test]
    #[ignore]
    fn magnet_downloads_something() {
        let dir = std::env::temp_dir().join(format!("owl-tor-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("USERPROFILE", &dir);
        let (tx, rx) = std::sync::mpsc::channel();
        let m = "magnet:?xt=urn:btih:dd8255ecdc7ca55fb0bbf81323d87062db1f6d1c&dn=Big+Buck+Bunny&tr=udp%3A%2F%2Ftracker.opentrackr.org%3A1337&tr=udp%3A%2F%2Fexplodie.org%3A6969";
        let tx = Mutex::new(tx);
        add(1, Source::Magnet(m.into()), move |e| {
            let _ = tx.lock().unwrap().send(match e {
                Ev::TorrentName(_, n) => format!("name {n}"),
                Ev::DlProg(_, r, t) => format!("prog {r} {t}"),
                Ev::DlState(_, s) => format!("state {s}"),
                _ => String::new(),
            });
        });
        let end = std::time::Instant::now() + Duration::from_secs(90);
        let mut got = 0i64;
        while std::time::Instant::now() < end {
            if let Ok(m) = rx.recv_timeout(Duration::from_secs(2)) {
                println!("{m}");
                if let Some(r) = m.strip_prefix("prog ") {
                    got = got.max(r.split(' ').next().unwrap().parse().unwrap_or(0));
                }
                if got > 1_000_000 {
                    break;
                }
            }
        }
        assert!(got > 1_000_000, "no data received");
        assert!(running(), "engine should be on while a torrent is active");
        assert!(cancel(1));
        let end = std::time::Instant::now() + Duration::from_secs(20);
        while running() && std::time::Instant::now() < end {
            std::thread::sleep(Duration::from_millis(250));
        }
        assert!(!running(), "engine should shut down when the last torrent ends");
    }
}
