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

static RT: OnceLock<Runtime> = OnceLock::new();
static SESSION: tokio::sync::OnceCell<Arc<Session>> = tokio::sync::OnceCell::const_new();
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
    SESSION
        .get_or_try_init(|| async {
            let out: PathBuf = crate::dl::root();
            let _ = std::fs::create_dir_all(&out);
            // no persisted DHT state on disk and no listening port: Owl only connects out
            let opts = SessionOptions {
                dht: Some(librqbit::DhtSessionConfig { bootstrap_addrs: None, port: None, persistence: None }),
                ..Default::default()
            };
            Session::new_with_opts(out, opts).await
        })
        .await
        .cloned()
}

/// What to add: a magnet link or the bytes of a .torrent file.
pub enum Source {
    Magnet(String),
    File(Vec<u8>),
}

/// Add a torrent; progress and the final state arrive as ordinary download events.
/// `named` receives (name, total bytes) once the torrent's metadata is known.
pub fn add(id: u32, src: Source, send: impl Fn(Ev) + Send + Sync + 'static) {
    rt().spawn(async move {
        let fail = || send(Ev::DlState(id, 1));
        let s = match session().await {
            Ok(s) => s,
            Err(e) => {
                crate::dl::dbg(&format!("torrent session: {e:#}"));
                #[cfg(test)]
                eprintln!("session error: {e:#}");
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
                return fail();
            }
        };
        jobs(|j| j.insert(id, handle.clone()));
        send(Ev::TorrentName(id, handle.name().unwrap_or_default()));
        let mut seeded_stop = false;
        loop {
            tokio::time::sleep(Duration::from_millis(700)).await;
            if !has(id) {
                return; // cancelled
            }
            let st = handle.stats();
            send(Ev::DlProg(id, st.progress_bytes as i64, st.total_bytes as i64));
            if st.error.is_some() {
                return fail();
            }
            if st.finished && !seeded_stop {
                seeded_stop = true;
                let _ = s.pause(&handle).await; // stop seeding when complete
                send(Ev::DlState(id, 2));
                jobs(|j| j.remove(&id));
                return;
            }
        }
    });
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
        assert!(cancel(1));
    }
}
