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
const MAX_INFLIGHT: usize = 8;
const MIN_BYTES: usize = 100_000;

/// Added to pages while compression is on: images below the fold load only when scrolled near.
pub const LAZY_JS: &str = "try{var _f=function(){document.querySelectorAll('img:not([loading])').forEach(function(i){if(i.getBoundingClientRect().top>innerHeight*1.5){i.loading='lazy'}i.decoding='async'})};var _o=new MutationObserver(_f);document.addEventListener('DOMContentLoaded',function(){_o.observe(document.documentElement,{childList:true,subtree:true});_f()})}catch(e){}";

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
    let bytes = crate::engine::fetch_bytes(url, referer, 25 * 1024 * 1024)?;
    shrink(&bytes, strength)
}

/// Re-encode image bytes at the given strength; None when it should be served unchanged.
pub fn shrink(bytes: &[u8], strength: u32) -> Option<(Vec<u8>, &'static str)> {
    if bytes.len() < MIN_BYTES {
        return None;
    }
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
    fn higher_strength_gives_smaller_files_and_tiny_images_are_skipped() {
        let big = jpeg(1800, 1200, 95);
        let (low, m1) = shrink(&big, 20).unwrap();
        let (high, m2) = shrink(&big, 90).unwrap();
        assert_eq!((m1, m2), ("image/jpeg", "image/jpeg"));
        assert!(low.len() < big.len(), "{} !< {}", low.len(), big.len());
        assert!(high.len() < low.len(), "{} !< {}", high.len(), low.len());
        let small = jpeg(40, 40, 90);
        assert!(shrink(&small, 90).is_none());
        // the result still decodes and respects the size cap
        let back = image::load_from_memory(&high).unwrap();
        assert!(back.width().max(back.height()) <= 2400 - 90 * 20);
    }
}
