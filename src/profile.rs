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
    "Preferences", "Secure Preferences",
];
const MAX_ENTRIES: usize = 1_000_000;

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

/// Engine caches that hold copies of pages and are rebuilt on demand: deleted after every seal so
/// nothing readable is left behind while Owl is locked or closed.
const CACHES: &[&str] = &["Cache", "Code Cache", "GPUCache", "DawnGraphiteCache", "DawnWebGPUCache", "Download Service", "Shared Dictionary", "blob_storage"];
const ROOT_CACHES: &[&str] = &["GrShaderCache", "ShaderCache", "GraphiteDawnCache"];

/// Set when the last restore came from a file sealed without the startup key (after a crash), so
/// Owl could not prove it was not replaced.
pub static LAST_UNAUTHENTICATED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn wipe_caches(dir: &Path) {
    let (b, u) = (base(dir), user_data(dir));
    for c in CACHES {
        let _ = fs::remove_dir_all(b.join(c));
    }
    for c in ROOT_CACHES {
        let _ = fs::remove_dir_all(u.join(c));
    }
}

/// Writes plaintext into 1 MiB sealed chunks while hashing it.
struct ChunkWriter<'a> {
    f: &'a mut fs::File,
    st: crypt::Stream,
    buf: Vec<u8>,
    hash: sha2::Sha256,
}
impl ChunkWriter<'_> {
    fn put(&mut self, data: &[u8]) -> Result<(), String> {
        use sha2::Digest;
        self.hash.update(data);
        self.buf.extend_from_slice(data);
        while self.buf.len() >= crypt::STREAM_CHUNK {
            let rest = self.buf.split_off(crypt::STREAM_CHUNK);
            let c = self.st.seal_chunk(&self.buf, false);
            self.emit(&c, false)?;
            self.buf = rest;
        }
        Ok(())
    }
    fn emit(&mut self, c: &[u8], last: bool) -> Result<(), String> {
        let len = c.len() as u32 | if last { 0x8000_0000 } else { 0 };
        self.f.write_all(&len.to_le_bytes()).map_err(|e| e.to_string())?;
        self.f.write_all(c).map_err(|e| e.to_string())
    }
    fn finish(mut self) -> Result<[u8; 32], String> {
        use sha2::Digest;
        let c = self.st.seal_chunk(&self.buf, true);
        self.emit(&c, true)?;
        Ok(self.hash.finalize().into())
    }
}

/// Reads a format-3 sealed file as a plaintext stream. The trailer (MAC) is checked by `finish`.
struct ChunkReader {
    r: std::io::BufReader<fs::File>,
    st: crypt::Stream,
    cur: Vec<u8>,
    pos: usize,
    done: bool,
    hash: sha2::Sha256,
}
impl std::io::Read for ChunkReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        use sha2::Digest;
        use std::io::{Error, ErrorKind, Read as _};
        while self.pos >= self.cur.len() {
            if self.done {
                return Ok(0);
            }
            let mut l = [0u8; 4];
            self.r.read_exact(&mut l)?;
            let raw = u32::from_le_bytes(l);
            let last = raw & 0x8000_0000 != 0;
            let len = (raw & 0x7fff_ffff) as usize;
            if len > crypt::STREAM_CHUNK + 16 {
                return Err(Error::new(ErrorKind::InvalidData, "chunk too large"));
            }
            let mut c = vec![0u8; len];
            self.r.read_exact(&mut c)?;
            self.cur = self.st.open_chunk(&c, last).ok_or_else(|| Error::new(ErrorKind::InvalidData, "damaged or altered sealed data"))?;
            self.hash.update(&self.cur);
            self.pos = 0;
            self.done = last;
        }
        let n = out.len().min(self.cur.len() - self.pos);
        out[..n].copy_from_slice(&self.cur[self.pos..self.pos + n]);
        self.pos += n;
        Ok(n)
    }
}

fn open_reader(path: &Path, privk: &[u8; 32]) -> Result<(ChunkReader, Vec<u8>, bool), String> {
    use sha2::Digest;
    use std::io::Read;
    let f = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut r = std::io::BufReader::with_capacity(1 << 20, f);
    let mut h = vec![0u8; crypt::STREAM_HEADER];
    r.read_exact(&mut h).map_err(|e| e.to_string())?;
    let (st, authed) = crypt::stream_open(privk, &h).ok_or("could not open the sealed profile (wrong key or damaged file)")?;
    Ok((ChunkReader { r, st, cur: vec![], pos: 0, done: false, hash: sha2::Sha256::new() }, h, authed))
}

/// After the plaintext is read to its end: check the trailer. With `auth` it must carry a valid MAC.
fn check_trailer(rd: ChunkReader, header: &[u8], authed: bool, auth: Option<&[u8; 32]>) -> Result<[u8; 32], String> {
    use sha2::Digest;
    use std::io::Read;
    if !rd.done || rd.pos < rd.cur.len() {
        return Err("sealed profile ended early".into());
    }
    let plain: [u8; 32] = rd.hash.clone().finalize().into();
    let mut r = rd.r;
    let mut tr = [0u8; 36];
    r.read_exact(&mut tr).map_err(|_| "sealed profile has no trailer".to_string())?;
    if u32::from_le_bytes(tr[..4].try_into().unwrap()) != u32::MAX {
        return Err("sealed profile trailer is damaged".into());
    }
    if authed {
        let Some(k) = auth else { return Err("sealed profile needs the startup key to verify".into()) };
        let mut msg = header.to_vec();
        msg.extend_from_slice(&plain);
        if !crypt::ct_eq_bytes(&crypt::hmac(k, &msg), &tr[4..]) {
            return Err("sealed profile failed its integrity check; it was not restored".into());
        }
    }
    let mut extra = [0u8; 1];
    if r.read(&mut extra).map_err(|e| e.to_string())? != 0 {
        return Err("unexpected data after the sealed profile".into());
    }
    Ok(plain)
}

/// Read and check one bundle stream; `sink` receives each file (relative path, reader of its bytes).
fn walk_bundle(
    rd: &mut ChunkReader,
    mut sink: impl FnMut(&str, &mut dyn std::io::Read, u64) -> Result<(), String>,
) -> Result<(), String> {
    use std::io::Read;
    let mut b4 = [0u8; 4];
    let mut b8 = [0u8; 8];
    rd.read_exact(&mut b4).map_err(|e| e.to_string())?;
    let n = u32::from_le_bytes(b4) as usize;
    if n > MAX_ENTRIES {
        return Err("sealed profile has too many entries".into());
    }
    for _ in 0..n {
        rd.read_exact(&mut b4).map_err(|e| e.to_string())?;
        let l = u32::from_le_bytes(b4) as usize;
        if l > 4096 {
            return Err("corrupt".into());
        }
        let mut name = vec![0u8; l];
        rd.read_exact(&mut name).map_err(|e| e.to_string())?;
        let rel = String::from_utf8(name).map_err(|_| "corrupt".to_string())?;
        let first = rel.split('/').next().unwrap_or("");
        if !ITEMS.contains(&first) || rel.contains("..") || rel.contains(':') || rel.contains('\\') || rel.starts_with('/') {
            return Err("unexpected path in sealed profile".into());
        }
        rd.read_exact(&mut b8).map_err(|e| e.to_string())?;
        let len = u64::from_le_bytes(b8);
        let mut take = (&mut *rd).take(len);
        sink(&rel, &mut take, len)?;
        // the sink must consume the file fully
        let mut rest = std::io::sink();
        std::io::copy(&mut take, &mut rest).map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Seal the sensitive profile files to the public key (needs no password), in a streamed format
/// with no size limit. With the private key (`verify`) the sealed copy is read back and compared
/// before any original is deleted; with the startup-derived `auth` key it is also authenticated.
pub fn seal_profile(dir: &Path, pubk: &[u8; 32], verify: Option<&[u8; 32]>, auth: Option<&[u8; 32]>) -> Result<(), String> {
    use sha2::Digest;
    use std::io::Read;
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
        wipe_caches(dir);
        return Ok(());
    }
    if files.len() > MAX_ENTRIES {
        return Err(format!("profile has {} files, more than Owl can seal; data left readable", files.len()));
    }
    let tmp = sealed(dir).with_extension("tmp");
    let (header, st) = crypt::stream_seal(pubk, auth.is_some());
    let plain_hash = {
        let mut f = fs::File::create(&tmp).map_err(|e| e.to_string())?;
        f.write_all(&header).map_err(|e| e.to_string())?;
        let mut w = ChunkWriter { f: &mut f, st, buf: Vec::with_capacity(crypt::STREAM_CHUNK + 4096), hash: sha2::Sha256::new() };
        w.put(&(files.len() as u32).to_le_bytes())?;
        let mut piece = vec![0u8; 256 * 1024];
        for (rel, p) in &files {
            let mut src = fs::File::open(p).map_err(|e| format!("read {rel}: {e}"))?;
            let len = src.metadata().map_err(|e| format!("read {rel}: {e}"))?.len();
            w.put(&(rel.len() as u32).to_le_bytes())?;
            w.put(rel.as_bytes())?;
            w.put(&len.to_le_bytes())?;
            let mut left = len;
            while left > 0 {
                let want = piece.len().min(left as usize);
                src.read_exact(&mut piece[..want]).map_err(|e| format!("read {rel}: {e}"))?;
                w.put(&piece[..want])?;
                left -= want as u64;
            }
        }
        let h = w.finish()?;
        let mut tr = u32::MAX.to_le_bytes().to_vec();
        match auth {
            Some(k) => {
                let mut msg = header.clone();
                msg.extend_from_slice(&h);
                tr.extend_from_slice(&crypt::hmac(k, &msg));
            }
            None => tr.extend_from_slice(&[0u8; 32]),
        }
        f.write_all(&tr).map_err(|e| e.to_string())?;
        f.sync_all().map_err(|e| e.to_string())?; // on disk before any original is removed
        h
    };
    // prove the sealed copy opens and holds exactly what was read (when the private key is at hand)
    if let Some(privk) = verify {
        let check = (|| -> Result<(), String> {
            let (mut rd, h, authed) = open_reader(&tmp, privk)?;
            walk_bundle(&mut rd, |_, _, _| Ok(()))?;
            let got = check_trailer(rd, &h, authed, auth)?;
            if got != plain_hash {
                return Err("sealed copy differs".into());
            }
            Ok(())
        })();
        if let Err(e) = check {
            let _ = fs::remove_file(&tmp);
            return Err(format!("verification failed ({e}); originals kept"));
        }
    }
    // an existing sealed file is kept as the previous generation only if it provably opens;
    // otherwise it is parked, never overwritten
    let cur = sealed(dir);
    if cur.is_file() {
        let opens = match verify {
            Some(p) => opens_with(&cur, p),
            None => false,
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
    wipe_caches(dir);
    if has_plaintext(dir) {
        return Err("some files could not be removed after sealing".into());
    }
    Ok(())
}

/// True if a sealed file (any format) opens with this private key.
fn opens_with(path: &Path, privk: &[u8; 32]) -> bool {
    let Ok(mut f) = fs::File::open(path) else { return false };
    let mut m = [0u8; 8];
    use std::io::Read;
    if f.read_exact(&mut m).is_err() {
        return false;
    }
    if &m == crypt::MAGIC_PROFILE3 {
        return open_reader(path, privk).is_ok();
    }
    fs::read(path).ok().and_then(|d| crypt::open_from(privk, &d)).is_some()
}

/// Restore a format-3 sealed profile: a first pass checks every chunk and the MAC without writing
/// anything; only then are the files written.
fn unseal_v3(dir: &Path, sp: &Path, privk: &[u8; 32], auth: Option<&[u8; 32]>) -> Result<(), String> {
    let (mut rd, h, authed) = open_reader(sp, privk)?;
    walk_bundle(&mut rd, |_, _, _| Ok(()))?;
    check_trailer(rd, &h, authed, auth)?;
    LAST_UNAUTHENTICATED.store(!authed, std::sync::atomic::Ordering::Relaxed);
    let b = base(dir);
    let (mut rd, h, authed) = open_reader(sp, privk)?;
    let mut written: Vec<PathBuf> = vec![];
    let res = walk_bundle(&mut rd, |rel, src, _| {
        let dest = b.join(rel);
        if let Some(par) = dest.parent() {
            fs::create_dir_all(par).map_err(|e| e.to_string())?;
        }
        let mut f = fs::File::create(&dest).map_err(|e| format!("write {rel}: {e}"))?;
        written.push(dest.clone());
        std::io::copy(src, &mut f).map_err(|e| format!("write {rel}: {e}"))?;
        Ok(())
    })
    .and_then(|_| check_trailer(rd, &h, authed, auth).map(|_| ()));
    if let Err(e) = res {
        // undo a half-restore so the sealed copy stays the only truth
        for w in &written {
            let _ = fs::remove_file(w);
        }
        return Err(e);
    }
    let _ = fs::remove_file(sp);
    Ok(())
}

/// Restore sealed files before any tab opens. Never overwrites plaintext that is already there.
/// On any failure the sealed file is kept and an error is returned: callers must not open tabs on
/// an empty profile.
pub fn unseal_profile(dir: &Path, legacy_key: &[u8; 32], privk: Option<&[u8; 32]>, auth: Option<&[u8; 32]>) -> Result<(), String> {
    let sp = sealed(dir);
    if !sp.is_file() {
        return Ok(());
    }
    let b = base(dir);
    if ITEMS.iter().any(|it| b.join(it).exists()) {
        return Ok(()); // plaintext is current (earlier crash or first run); the next seal parks the stale file
    }
    if opens_v3_magic(&sp) {
        let privk = privk.ok_or("could not open the sealed profile (no key)")?;
        return unseal_v3(dir, &sp, privk, auth);
    }
    // older formats were sealed without the startup key, so they cannot be authenticated
    LAST_UNAUTHENTICATED.store(true, std::sync::atomic::Ordering::Relaxed);
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

fn opens_v3_magic(p: &Path) -> bool {
    use std::io::Read;
    let mut m = [0u8; 8];
    fs::File::open(p).and_then(|mut f| f.read_exact(&mut m)).is_ok() && &m == crypt::MAGIC_PROFILE3
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree(dir: &Path, big: usize) {
        let b = base(dir);
        fs::create_dir_all(b.join("Network")).unwrap();
        fs::write(b.join("Network").join("Cookies"), b"cookie-data-123").unwrap();
        fs::create_dir_all(b.join("IndexedDB").join("site")).unwrap();
        let data: Vec<u8> = (0..big).map(|i| (i * 31 % 251) as u8).collect();
        fs::write(b.join("IndexedDB").join("site").join("blob"), &data).unwrap();
        fs::write(b.join("Preferences"), b"{}").unwrap();
        fs::create_dir_all(b.join("Cache").join("Cache_Data")).unwrap();
        fs::write(b.join("Cache").join("Cache_Data").join("f_0001"), b"cached page").unwrap();
    }

    fn tmpdir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("owl-seal-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn seal_and_restore_large_profile_with_auth() {
        let d = tmpdir("ok");
        tree(&d, 3 * crypt::STREAM_CHUNK + 777);
        let (privk, pubk) = crypt::new_keypair();
        let auth = [7u8; 32];
        seal_profile(&d, &pubk, Some(&privk), Some(&auth)).unwrap();
        assert!(!has_plaintext(&d));
        assert!(!base(&d).join("Cache").exists(), "caches are wiped");
        assert!(has_sealed(&d));
        unseal_profile(&d, &[0u8; 32], Some(&privk), Some(&auth)).unwrap();
        assert_eq!(fs::read(base(&d).join("Network").join("Cookies")).unwrap(), b"cookie-data-123");
        let blob = fs::read(base(&d).join("IndexedDB").join("site").join("blob")).unwrap();
        assert_eq!(blob.len(), 3 * crypt::STREAM_CHUNK + 777);
        assert_eq!(blob[12345], (12345 * 31 % 251) as u8);
        assert_eq!(fs::read(base(&d).join("Preferences")).unwrap(), b"{}");
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn forged_or_tampered_files_are_refused() {
        let d = tmpdir("forge");
        tree(&d, 1000);
        let (privk, pubk) = crypt::new_keypair();
        // a forger knows the public key but not the startup key
        seal_profile(&d, &pubk, Some(&privk), Some(&[1u8; 32])).unwrap();
        assert!(unseal_profile(&d, &[0u8; 32], Some(&privk), Some(&[9u8; 32])).is_err());
        assert!(has_sealed(&d) && !has_plaintext(&d), "nothing written on failure");
        let sp = sealed(&d);
        let mut bytes = fs::read(&sp).unwrap();
        let mid = bytes.len() / 2;
        bytes[mid] ^= 1;
        fs::write(&sp, &bytes).unwrap();
        assert!(unseal_profile(&d, &[0u8; 32], Some(&privk), Some(&[1u8; 32])).is_err());
        bytes[mid] ^= 1;
        fs::write(&sp, &bytes[..bytes.len() - 40]).unwrap();
        assert!(unseal_profile(&d, &[0u8; 32], Some(&privk), Some(&[1u8; 32])).is_err());
        assert!(!has_plaintext(&d));
        let _ = fs::remove_dir_all(&d);
    }

    #[test]
    fn seal_without_startup_key_restores_but_is_marked() {
        let d = tmpdir("crash");
        tree(&d, 10);
        let (privk, pubk) = crypt::new_keypair();
        seal_profile(&d, &pubk, None, None).unwrap();
        unseal_profile(&d, &[0u8; 32], Some(&privk), Some(&[3u8; 32])).unwrap();
        assert!(LAST_UNAUTHENTICATED.load(std::sync::atomic::Ordering::Relaxed));
        assert_eq!(fs::read(base(&d).join("Network").join("Cookies")).unwrap(), b"cookie-data-123");
        let _ = fs::remove_dir_all(&d);
    }
}
