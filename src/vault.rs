//! Password vault. Each password is encrypted with Windows DPAPI (bound to the signed-in Windows
//! user) using an extra secret derived from the startup key, then stored as hex in vault.json.
//! A program running as you therefore also needs the startup key to read the passwords.
use crate::crypt::{hex, unhex};
use serde::{Deserialize, Serialize};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{LocalFree, HLOCAL};
use windows::Win32::Security::Cryptography::{CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Cred {
    pub host: String,
    pub user: String,
    pub pass: String, // hex(DPAPI(password, entropy))
    #[serde(default)]
    pub t: u64,
}

fn blob_to_vec(b: &CRYPT_INTEGER_BLOB) -> Vec<u8> {
    unsafe {
        let v = std::slice::from_raw_parts(b.pbData, b.cbData as usize).to_vec();
        let _ = LocalFree(Some(HLOCAL(b.pbData as *mut _)));
        v
    }
}

pub fn encrypt(plain: &str, entropy: &[u8]) -> Option<String> {
    unsafe {
        let inb = CRYPT_INTEGER_BLOB { cbData: plain.len() as u32, pbData: plain.as_ptr() as *mut u8 };
        let ent = CRYPT_INTEGER_BLOB { cbData: entropy.len() as u32, pbData: entropy.as_ptr() as *mut u8 };
        let mut out = CRYPT_INTEGER_BLOB::default();
        let e = if entropy.is_empty() { None } else { Some(&ent as *const CRYPT_INTEGER_BLOB) };
        CryptProtectData(&inb, PCWSTR::null(), e, None, None, 0, &mut out).ok()?;
        Some(hex(&blob_to_vec(&out)))
    }
}

fn try_decrypt(data: &[u8], entropy: &[u8]) -> Option<String> {
    unsafe {
        let inb = CRYPT_INTEGER_BLOB { cbData: data.len() as u32, pbData: data.as_ptr() as *mut u8 };
        let ent = CRYPT_INTEGER_BLOB { cbData: entropy.len() as u32, pbData: entropy.as_ptr() as *mut u8 };
        let mut out = CRYPT_INTEGER_BLOB::default();
        let e = if entropy.is_empty() { None } else { Some(&ent as *const CRYPT_INTEGER_BLOB) };
        CryptUnprotectData(&inb, None, e, None, None, 0, &mut out).ok()?;
        String::from_utf8(blob_to_vec(&out)).ok()
    }
}

/// Decrypts with the key-bound entropy; entries saved by older versions (no entropy) still open
/// and are re-encrypted after the next successful unlock.
pub fn decrypt(h: &str, entropy: &[u8]) -> Option<String> {
    let data = unhex(h)?;
    try_decrypt(&data, entropy).or_else(|| if entropy.is_empty() { None } else { try_decrypt(&data, &[]) })
}
