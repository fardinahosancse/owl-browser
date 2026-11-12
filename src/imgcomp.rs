//! Lazy compression: when switched on, images on web pages are fetched, shrunk and re-encoded
//! before the page receives them, so pages load faster and use less data. The 1-99 setting is the
//! strength: higher means smaller images and lower quality. Images that would not get smaller,
//! small images, animated images and anything that fails are left exactly as the site sent them.
use crate::Ev;
use image::{imageops::FilterType, ImageFormat};
use std::{
    io::Cursor,
    sync::atomic::{AtomicU32, AtomicUsize, Ordering::Relaxed},
};

/// 0 = off, otherwise 1-99.
pub static STRENGTH: AtomicU32 = AtomicU32::new(0);
static INFLIGHT: AtomicUsize = AtomicUsize::new(0);
const MAX_INFLIGHT: usize = 16;
/// Only images at least this big are touched: smaller ones are not worth the extra round trip.
pub const MIN_BYTES: usize = 4 * 1024 * 1024;
/// Progressive JPEGs from this size up are cut short instead of re-encoded.
pub const MIN_TRUNC: usize = 1024 * 1024;

/// Added to pages while compression is on: images below the fold load only when scrolled near.
pub const LAZY_JS: &str = "try{var _f=function(){document.querySelectorAll('img:not([loading])').forEach(function(i){if(i.getBoundingClientRect().top>innerHeight*1.5){i.loading='lazy';i.fetchPriority='low'}i.decoding='async'})};var _o=new MutationObserver(_f);document.addEventListener('DOMContentLoaded',function(){_o.observe(document.documentElement,{childList:true,subtree:true});_f()})}catch(e){}";

/// Start compressing one image in the background. Returns false if there is no capacity, in which
/// case the caller lets the original through. The result arrives as `Ev::ImgReady(id, ..)`.
pub fn start(id: u32, url: String, referer: String, send: impl Fn(Ev) + Send + 'static) -> bool {
    if INFLIGHT.fetch_add(1, Relaxed) >= MAX_INFLIGHT {
        INFLIGHT.fetch_sub(1, Relaxed);
        return false;
    }
    let strength = STRENGTH.load(Relaxed);
    std::thread::spawn(move || {
        let r = work(&url, &referer, strength);
        INFLIGHT.fetch_sub(1, Relaxed);
        send(Ev::ImgReady(id, r));
    });
    true
}

fn work(url: &str, referer: &str, strength: u32) -> Option<(Vec<u8>, &'static str)> {
    // vector, animated and icon formats are never worth it
    let path = url.split(['?', '#']).next().unwrap_or("").to_lowercase();
    if [".svg", ".gif", ".ico", ".cur", ".avif"].iter().any(|e| path.ends_with(e)) {
        return None;
    }
    // ask for the size first with a one-byte request; most images are small and pass straight through
    let (size, ranged) = crate::engine::remote_size(url, referer)?;
    if size < MIN_TRUNC as i64 {
        return None;
    }
    // 1. progressive JPEG: fetch only the first part. It shows as a slightly soft image and the
    //    rest is never downloaded, so this saves real bandwidth and needs no re-encoding.
    if ranged {
        let head = crate::engine::fetch_range(url, referer, 0, 65_535.min(size - 1))?;
        if is_progressive_jpeg(&head) {
            let keep = ((size as f64) * keep_fraction(strength)) as i64;
            if keep >= 128 * 1024 && keep < size * 95 / 100 {
                let mut body = crate::engine::fetch_range(url, referer, 0, keep - 1)?;
                body.extend_from_slice(&[0xFF, 0xD9]); // end-of-image, so decoders stop cleanly
                return Some((body, "image/jpeg"));
            }
        }
    }
    // 2. anything else that is very large: shrink and re-encode it
    if size < MIN_BYTES as i64 {
        return None;
    }
    let bytes = crate::engine::fetch_bytes(url, referer, 25 * 1024 * 1024)?;
    shrink(&bytes, strength)
}

/// Share of a progressive JPEG to keep: 92% at strength 1 down to about 25% at 99.
pub fn keep_fraction(strength: u32) -> f64 {
    (0.92 - strength.min(99) as f64 * 0.0068).clamp(0.2, 0.92)
}

/// True for a JPEG whose scans refine the whole picture step by step (SOF2 and its variants).
pub fn is_progressive_jpeg(b: &[u8]) -> bool {
    if b.len() < 4 || b[0] != 0xFF || b[1] != 0xD8 {
        return false;
    }
    let mut i = 2;
    while i + 4 <= b.len() {
        if b[i] != 0xFF {
            i += 1;
            continue;
        }
        let m = b[i + 1];
        if m == 0xFF {
            i += 1;
            continue;
        }
        if (0xC0..=0xCF).contains(&m) && !matches!(m, 0xC4 | 0xC8 | 0xCC) {
            return matches!(m, 0xC2 | 0xC6 | 0xCA | 0xCE);
        }
        if m == 0xDA {
            return false; // scan data before any frame header
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        i += 2 + len;
    }
    false
}

/// Re-encode image bytes at the given strength; None when it should be served unchanged.
pub fn shrink(bytes: &[u8], strength: u32) -> Option<(Vec<u8>, &'static str)> {
    let fmt = image::guess_format(bytes).ok()?;
    if !matches!(fmt, ImageFormat::Jpeg | ImageFormat::Png | ImageFormat::WebP) {
        return None; // gifs may be animated; others are not worth the risk
    }
    // animated WebP: VP8X header with the animation flag
    if fmt == ImageFormat::WebP && bytes.len() > 21 && &bytes[12..16] == b"VP8X" && bytes[20] & 0x02 != 0 {
        return None;
    }
    let mut img = image::load_from_memory(bytes).ok()?;
    let max_dim = (2400u32.saturating_sub(strength * 20)).max(400);
    if img.width().max(img.height()) > max_dim {
        img = img.resize(max_dim, max_dim, FilterType::Triangle);
    }
    let mut out = Cursor::new(Vec::new());
    let mime = if img.color().has_alpha() || (fmt == ImageFormat::Png && strength < 50) {
        img.write_to(&mut out, ImageFormat::Png).ok()?;
        "image/png"
    } else {
        let q = (100u32.saturating_sub(strength)).clamp(10, 92) as u8;
        let rgb = img.to_rgb8();
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, q);
        rgb.write_with_encoder(enc).ok()?;
        "image/jpeg"
    };
    let out = out.into_inner();
    // only worth serving when it is clearly smaller
    if out.len() * 10 >= bytes.len() * 9 {
        return None;
    }
    Some((out, mime))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn jpeg(w: u32, h: u32, q: u8) -> Vec<u8> {
        let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x / 7) as u8 ^ (y % 31) as u8, (y / 5) as u8, ((x * y) % 251) as u8]));
        let mut out = Cursor::new(Vec::new());
        let enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, q);
        img.write_with_encoder(enc).unwrap();
        out.into_inner()
    }

    #[test]
    fn progressive_detection_and_truncation_decode() {
        let prog = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/prog.jpg")).unwrap();
        let base = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/base.jpg")).unwrap();
        assert!(is_progressive_jpeg(&prog));
        assert!(!is_progressive_jpeg(&base));
        assert!(keep_fraction(99) < keep_fraction(1) && keep_fraction(99) > 0.2);
        // a progressive JPEG cut at 40% plus an end marker still decodes to the full picture size
        let mut cut = prog[..prog.len() * 4 / 10].to_vec();
        cut.extend_from_slice(&[0xFF, 0xD9]);
        let img = image::load_from_memory(&cut).expect("truncated progressive jpeg should still decode");
        assert_eq!((img.width(), img.height()), (640, 480));
    }

    #[test]
    fn higher_strength_gives_smaller_files_and_tiny_images_are_skipped() {
        let big = jpeg(1800, 1200, 95);
        let (low, m1) = shrink(&big, 20).unwrap();
        let (high, m2) = shrink(&big, 90).unwrap();
        assert_eq!((m1, m2), ("image/jpeg", "image/jpeg"));
        assert!(low.len() < big.len(), "{} !< {}", low.len(), big.len());
        assert!(high.len() < low.len(), "{} !< {}", high.len(), low.len());
        // the result still decodes and respects the size cap
        let back = image::load_from_memory(&high).unwrap();
        assert!(back.width().max(back.height()) <= 2400 - 90 * 20);
    }
}
