//! Encrypts the browser's own profile data (cookies, saved logins, history, site storage) while
//! Owl is closed or locked. Sensitive Chromium files are bundled and sealed to a public key (so
//! sealing needs no password); the password-wrapped private key opens them after a correct unlock.
//!
//! Data safety rules: nothing is deleted until the sealed copy is written, synced to disk and
//! (when the private key is at hand) opened and compared; any I/O problem aborts the seal and
//! leaves the originals alone; an existing sealed file that cannot be proven openable is kept,
//! never overwritten; a restore that fails keeps the sealed file and reports the error.
use crate::crypt;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const ITEMS: &[&str] = &[
    "Network", "Login Data", "Login Data-journal", "Login Data For Account", "Login Data For Account-journal",
    "Web Data", "Web Data-journal", "History", "History-journal", "Favicons", "Favicons-journal", "IndexedDB",
    "WebStorage", "Local Storage", "Session Storage", "Sessions", "Network Action Predictor",
    "Network Action Predictor-journal", "DIPS", "DIPS-wal", "DIPS-shm", "Service Worker", "File System",
    "Visited Links", "Top Sites", "Top Sites-journal", "Shortcuts", "Shortcuts-journal", "databases",
];
const MAX_BUNDLE: u64 = 700 * 1024 * 1024;
const MAX_ENTRIES: usize = 200_000;

fn user_data(dir: &Path) -> PathBuf {
    dir.join("profile").join("EBWebView")
}
fn base(dir: &Path) -> PathBuf {
    user_data(dir).join("Default")
}
fn sealed(dir: &Path) -> PathBuf {
    dir.join("profile").join("sealed.owl")
}

/// Collect files under `rel`. Any unreadable entry is an error: a partial bundle followed by a
/// delete would silently destroy data.
fn walk(root: &Path, rel: &str, out: &mut Vec<(String, PathBuf)>, total: &mut u64) -> Result<(), String> {
    let p = root.join(rel);
    if p.is_dir() {
        let rd = fs::read_dir(&p).map_err(|e| format!("list {rel}: {e}"))?;
        for e in rd {
            let e = e.map_err(|e| format!("list {rel}: {e}"))?;
            let r = format!("{rel}/{}", e.file_name().to_string_lossy());
            walk(root, &r, out, total)?;
        }
    } else {
        let m = fs::metadata(&p).map_err(|e| format!("stat {rel}: {e}"))?;
        *total += m.len();
        out.push((rel.to_string(), p));
    }
    Ok(())
}

/// Wait until Chromium has released the profile (its `lockfile` can be opened for writing).
fn wait_released(dir: &Path, max: Duration) -> bool {
    let lock = user_data(dir).join("lockfile");
    let t = Instant::now();
    while t.elapsed() < max {
        if !lock.exists() || fs::OpenOptions::new().write(true).open(&lock).is_ok() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    false
}

/// True when sensitive Chromium files are sitting on disk unencrypted.
pub fn has_plaintext(dir: &Path) -> bool {
    let b = base(dir);
    ITEMS.iter().any(|it| b.join(it).exists())
}

/// True when a sealed file exists (used to detect a damaged settings file).
pub fn has_sealed(dir: &Path) -> bool {
    sealed(dir).is_file()
}

fn unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// Keep an existing sealed file that we cannot prove opens with the current key: never overwrite
/// it. At most three such orphans are kept.
fn park_orphan(dir: &Path) {
    let cur = sealed(dir);
    let dest = dir.join("profile").join(format!("sealed.{}.orphan", unix()));
    let _ = fs::rename(&cur, &dest);
    let mut orphans: Vec<PathBuf> = fs::read_dir(dir.join("profile"))
        .map(|rd| rd.flatten().map(|e| e.path()).filter(|p| p.extension().map_or(false, |x| x == "orphan")).collect())
        .unwrap_or_default();
    orphans.sort();
    while orphans.len() > 3 {
        let _ = fs::remove_file(orphans.remove(0));
    }
}

/// Seal the sensitive profile files to the public key (needs no password). When the private key
/// is available (`verify`) the sealed copy is opened and compared before any original is deleted.
pub fn seal_profile(dir: &Path, pubk: &[u8; 32], verify: Option<&[u8; 32]>) -> Result<(), String> {
    let b = base(dir);
    if !b.is_dir() {
        return Ok(());
    }
    if !crypt::box_selftest() {
        return Err("sealing self-test failed; data left as is".into());
    }
    if !wait_released(dir, Duration::from_secs(25)) {
        return Err("profile still in use; data left readable".into());
    }
    let mut files = vec![];
    let mut total = 0u64;
    for it in ITEMS {
        if b.join(it).exists() {
            walk(&b, it, &mut files, &mut total)?;
        }
    }
    if files.is_empty() {
        return Ok(());
    }
    if total > MAX_BUNDLE || files.len() > MAX_ENTRIES {
        return Err("profile too large to seal; data left readable".into());
    }
    let mut bundle: Vec<u8> = Vec::with_capacity(total as usize + 1024);
    bundle.extend_from_slice(&(files.len() as u32).to_le_bytes());
    for (rel, p) in &files {
        let data = fs::read(p).map_err(|e| format!("read {rel}: {e}"))?;
        bundle.extend_from_slice(&(rel.len() as u32).to_le_bytes());
        bundle.extend_from_slice(rel.as_bytes());
        bundle.extend_from_slice(&(data.len() as u64).to_le_bytes());
        bundle.extend_from_slice(&data);
    }
    let blob = crypt::seal_to(pubk, &bundle);
    let tmp = sealed(dir).with_extension("tmp");
    {
        let mut f = fs::File::create(&tmp).map_err(|e| e.to_string())?;
        f.write_all(&blob).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?; // on disk before any original is removed
    }
    // prove the sealed copy was written intact (and, when we can, that it opens and matches)
    let back = fs::read(&tmp).map_err(|e| e.to_string())?;
    let good = back == blob
        && match verify {
            Some(privk) => crypt::open_from(privk, &back).as_deref() == Some(bundle.as_slice()),
            None => true,
        };
    drop(back);
    if !good {
        let _ = fs::remove_file(&tmp);
        return Err("verification failed; originals kept".into());
    }
    // an existing sealed file is kept as the previous generation only if it provably opens;
    // otherwise it is parked, never overwritten
    let cur = sealed(dir);
    if cur.is_file() {
        let opens = match (verify, fs::read(&cur)) {
            (Some(p), Ok(d)) => crypt::open_from(p, &d).is_some(),
            _ => false,
        };
        if opens {
            let _ = fs::copy(&cur, cur.with_extension("prev"));
        } else {
            park_orphan(dir);
        }
    }
    fs::rename(&tmp, &cur).map_err(|e| e.to_string())?;
    for it in ITEMS {
        let p = b.join(it);
        if p.is_dir() {
            let _ = fs::remove_dir_all(&p);
        } else if p.exists() {
            let _ = fs::remove_file(&p);
        }
    }
    if has_plaintext(dir) {
        return Err("some files could not be removed after sealing".into());
    }
    Ok(())
}

/// Restore sealed files before any tab opens. Never overwrites plaintext that is already there.
/// On any failure the sealed file is kept and an error is returned: callers must not open tabs on
/// an empty profile.
pub fn unseal_profile(dir: &Path, legacy_key: &[u8; 32], privk: Option<&[u8; 32]>) -> Result<(), String> {
    let sp = sealed(dir);
    if !sp.is_file() {
        return Ok(());
    }
    let b = base(dir);
    if ITEMS.iter().any(|it| b.join(it).exists()) {
        return Ok(()); // plaintext is current (earlier crash or first run); the next seal parks the stale file
    }
    let data = fs::read(&sp).map_err(|e| e.to_string())?;
    // current files are sealed to the public key; files from earlier builds used a password key
    let bundle = match data.get(..8) {
        Some(m) if m == crypt::MAGIC_PROFILE2 => privk.and_then(|p| crypt::open_from(p, &data)),
        _ => crypt::open(legacy_key, &data),
    }
    .ok_or("could not open the sealed profile (wrong key or damaged file)")?;
    let rd_u32 = |p: &mut usize| -> Option<usize> {
        let end = p.checked_add(4)?;
        let v = u32::from_le_bytes(bundle.get(*p..end)?.try_into().ok()?) as usize;
        *p = end;
        Some(v)
    };
    let mut p = 0usize;
    let n = rd_u32(&mut p).ok_or("corrupt")?;
    if n > MAX_ENTRIES {
        return Err("sealed profile has too many entries".into());
    }
    let mut entries: Vec<(String, std::ops::Range<usize>)> = Vec::new();
    for _ in 0..n {
        let l = rd_u32(&mut p).ok_or("corrupt")?;
        let end = p.checked_add(l).ok_or("corrupt")?;
        let rel = String::from_utf8(bundle.get(p..end).ok_or("corrupt")?.to_vec()).map_err(|_| "corrupt")?;
        p = end;
        let end8 = p.checked_add(8).ok_or("corrupt")?;
        let len = u64::from_le_bytes(bundle.get(p..end8).ok_or("corrupt")?.try_into().unwrap()) as usize;
        p = end8;
        let end = p.checked_add(len).ok_or("corrupt")?;
        if end > bundle.len() {
            return Err("corrupt".into());
        }
        // only paths under the sealed item names, and nothing that can escape the profile folder
        let first = rel.split('/').next().unwrap_or("");
        if !ITEMS.contains(&first) || rel.contains("..") || rel.contains(':') || rel.contains('\\') || rel.starts_with('/') {
            return Err("unexpected path in sealed profile".into());
        }
        entries.push((rel, p..end));
        p = end;
    }
    let mut written: Vec<PathBuf> = vec![];
    for (rel, r) in &entries {
        let dest = b.join(rel);
        let res = (|| -> Result<(), String> {
            if let Some(par) = dest.parent() {
                fs::create_dir_all(par).map_err(|e| e.to_string())?;
            }
            fs::write(&dest, &bundle[r.clone()]).map_err(|e| format!("write {rel}: {e}"))
        })();
        if let Err(e) = res {
            // undo a half-restore so the sealed copy stays the only truth
            for w in written.iter().chain(std::iter::once(&dest)) {
                let _ = fs::remove_file(w);
            }
            return Err(e);
        }
        written.push(dest);
    }
    let _ = fs::remove_file(&sp);
    Ok(())
}
