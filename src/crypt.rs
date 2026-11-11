//! Crypto helpers: secure randomness, Argon2id key derivation, sub-key derivation, and an
//! authenticated envelope (ChaCha20-Poly1305) used for the store, the sync file and the profile.
use zeroize::Zeroize;
use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{aead::{Aead, KeyInit}, ChaCha20Poly1305, Key, Nonce};
use sha2::{Digest, Sha256};
use windows::Win32::Security::Cryptography::{BCryptGenRandom, BCRYPT_USE_SYSTEM_PREFERRED_RNG};

pub const MAGIC_SYNC1: &[u8; 8] = b"OWLSYNC1"; // legacy: key from iterated SHA-256
pub const MAGIC_SYNC: &[u8; 8] = b"OWLSYNC2"; // key from Argon2id
pub const MAGIC_STORE: &[u8; 8] = b"OWLSTOR1";
pub const MAGIC_PROFILE: &[u8; 8] = b"OWLPROF1";
/// Rounds of the legacy hash (kept only to read data written by earlier builds).
pub const PIN_ROUNDS: u32 = 150_000;
pub const SYNC_ROUNDS: u32 = 1_000_000;

pub fn random(n: usize) -> Vec<u8> {
    let mut b = vec![0u8; n];
    unsafe {
        let _ = BCryptGenRandom(None, &mut b, BCRYPT_USE_SYSTEM_PREFERRED_RNG);
    }
    // never derive key material from a weak fallback: if the OS generator fails, stop
    assert!(n < 8 || b.iter().any(|x| *x != 0), "operating system random generator failed");
    b
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
pub fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

/// Legacy iterated SHA-256 stretch (builds before Argon2id).
pub fn kdf(salt: &str, secret: &str, rounds: u32) -> [u8; 32] {
    let mut h: [u8; 32] = Sha256::digest(format!("owl1|{salt}|{secret}")).into();
    for _ in 0..rounds {
        h = Sha256::digest(h).into();
    }
    h
}

/// Argon2id: memory-hard, so guessing needs 32 MiB of RAM per try and cannot be sped up much
/// with GPUs. `mem_kib` is 32768 for the startup key and 65536 for the sync passphrase.
pub fn argon(secret: &str, salt: &str, mem_kib: u32) -> [u8; 32] {
    let mut salt_bytes: [u8; 32] = Sha256::digest(format!("owl-argon|{salt}")).into();
    let mut out = [0u8; 32];
    let params = Params::new(mem_kib, 3, 1, Some(32)).expect("argon2 params");
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(secret.as_bytes(), &salt_bytes[..16], &mut out)
        .expect("argon2");
    salt_bytes.zeroize();
    out
}

/// Independent sub-keys from one master key, so one derivation covers everything.
pub fn sub(master: &[u8; 32], purpose: &str) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(master);
    h.update(b"|owl-sub|");
    h.update(purpose.as_bytes());
    h.finalize().into()
}

// ---- public-key sealing (a "sealed box"): anyone with the public key can seal, only the private
// key can open. Owl can therefore re-encrypt data at any moment (crash recovery, shutdown) without
// the password, and only the password-wrapped private key opens it again.
pub const MAGIC_PROFILE2: &[u8; 8] = b"OWLPROF2";
const BOX_HEADER: usize = 8 + 32 + 12 + 48 + 12;

pub fn x_public(privk: &[u8; 32]) -> [u8; 32] {
    x25519_dalek::x25519(*privk, x25519_dalek::X25519_BASEPOINT_BYTES)
}

pub fn new_keypair() -> ([u8; 32], [u8; 32]) {
    let p: [u8; 32] = random(32).try_into().unwrap();
    (p, x_public(&p))
}

fn box_wrap_key(shared: &[u8; 32], eph_pub: &[u8; 32], recipient: &[u8; 32]) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"owl-box-v1");
    h.update(shared);
    h.update(eph_pub);
    h.update(recipient);
    h.finalize().into()
}

pub fn seal_to(recipient: &[u8; 32], plain: &[u8]) -> Vec<u8> {
    let (eph, eph_pub) = new_keypair();
    let shared = x25519_dalek::x25519(eph, *recipient);
    let mut shared = shared;
    let mut wrap = box_wrap_key(&shared, &eph_pub, recipient);
    shared.zeroize();
    let mut eph = eph;
    let mut k: [u8; 32] = random(32).try_into().unwrap();
    let (n1, n2) = (random(12), random(12));
    let wk = ChaCha20Poly1305::new(Key::from_slice(&wrap)).encrypt(Nonce::from_slice(&n1), k.as_slice()).expect("encrypt");
    let ct = ChaCha20Poly1305::new(Key::from_slice(&k)).encrypt(Nonce::from_slice(&n2), plain).expect("encrypt");
    wrap.zeroize();
    k.zeroize();
    eph.zeroize();
    let mut out = MAGIC_PROFILE2.to_vec();
    out.extend_from_slice(&eph_pub);
    out.extend_from_slice(&n1);
    out.extend_from_slice(&wk);
    out.extend_from_slice(&n2);
    out.extend_from_slice(&ct);
    out
}

pub fn open_from(privk: &[u8; 32], data: &[u8]) -> Option<Vec<u8>> {
    if data.len() < BOX_HEADER + 16 || &data[..8] != MAGIC_PROFILE2 {
        return None;
    }
    let eph_pub: [u8; 32] = data[8..40].try_into().ok()?;
    let recipient = x_public(privk);
    let mut shared = x25519_dalek::x25519(*privk, eph_pub);
    let mut wrap = box_wrap_key(&shared, &eph_pub, &recipient);
    shared.zeroize();
    let kv = ChaCha20Poly1305::new(Key::from_slice(&wrap)).decrypt(Nonce::from_slice(&data[40..52]), &data[52..100]);
    wrap.zeroize();
    let mut k: [u8; 32] = kv.ok()?.try_into().ok()?;
    let out = ChaCha20Poly1305::new(Key::from_slice(&k)).decrypt(Nonce::from_slice(&data[100..112]), &data[112..]).ok();
    k.zeroize();
    out
}

/// Round-trip self-test, run once at start: if public-key sealing were ever broken, Owl would
/// fall back to keeping plaintext rather than risk destroying data.
pub fn box_selftest() -> bool {
    let (p, q) = new_keypair();
    let msg = b"owl self test";
    open_from(&p, &seal_to(&q, msg)).as_deref() == Some(msg.as_slice())
}

pub fn ct_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

pub fn nonce_hex() -> String {
    hex(&random(12))
}

/// magic(8) + salt(16) + nonce(12) + ciphertext.
pub fn seal(magic: &[u8; 8], key: &[u8; 32], salt: &[u8; 16], plain: &[u8]) -> Vec<u8> {
    let nonce = random(12);
    let ct = ChaCha20Poly1305::new(Key::from_slice(key)).encrypt(Nonce::from_slice(&nonce), plain).expect("encrypt");
    let mut out = magic.to_vec();
    out.extend_from_slice(salt);
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ct);
    out
}

pub fn magic_of(data: &[u8]) -> Option<[u8; 8]> {
    let m: [u8; 8] = data.get(..8)?.try_into().ok()?;
    [*MAGIC_SYNC1, *MAGIC_SYNC, *MAGIC_STORE, *MAGIC_PROFILE].contains(&m).then_some(m)
}

pub fn is_sealed(data: &[u8]) -> bool {
    magic_of(data).is_some()
}

pub fn header_salt(data: &[u8]) -> Option<[u8; 16]> {
    if data.len() < 8 + 16 + 12 + 16 || !is_sealed(data) {
        return None;
    }
    data[8..24].try_into().ok()
}

pub fn open(key: &[u8; 32], data: &[u8]) -> Option<Vec<u8>> {
    header_salt(data)?;
    ChaCha20Poly1305::new(Key::from_slice(key)).decrypt(Nonce::from_slice(&data[24..36]), &data[36..]).ok()
}
