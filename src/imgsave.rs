//! "Save image as" with a format choice. The image is fetched, decoded and written in the
//! chosen format (PNG, JPEG, BMP, GIF or WebP) to the Downloads folder.
use crate::Ev;
use image::ImageFormat;
use std::io::Cursor;

pub const FORMATS: &[(&str, &str)] = &[("png", "PNG"), ("jpg", "JPEG"), ("bmp", "BMP"), ("webp", "WebP"), ("gif", "GIF")];

fn fmt_of(ext: &str) -> Option<ImageFormat> {
    Some(match ext {
        "png" => ImageFormat::Png,
        "jpg" => ImageFormat::Jpeg,
        "bmp" => ImageFormat::Bmp,
        "webp" => ImageFormat::WebP,
        "gif" => ImageFormat::Gif,
        _ => return None,
    })
}

/// A readable file name taken from the address, without its old extension.
fn stem_of(url: &str) -> String {
    let last = url.split(['?', '#']).next().unwrap_or("").rsplit('/').next().unwrap_or("");
    let stem = last.rsplit_once('.').map(|x| x.0).unwrap_or(last);
    let clean: String = stem.chars().filter(|c| c.is_ascii_alphanumeric() || "-_ ".contains(*c)).take(80).collect();
    if clean.trim().is_empty() { "image".into() } else { clean }
}

fn load(url: &str, referer: &str) -> Option<Vec<u8>> {
    if let Some(rest) = url.strip_prefix("data:image/") {
        // data:image/png;base64,....
        let (meta, data) = rest.split_once(',')?;
        if !meta.ends_with(";base64") || data.len() > 40 * 1024 * 1024 {
            return None;
        }
        return b64(data);
    }
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return None;
    }
    crate::engine::fetch_bytes(url, referer, 40 * 1024 * 1024)
}

fn b64(s: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let (mut acc, mut bits) = (0u32, 0);
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\r' | b'\n' => continue,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(out)
}

/// Convert and save in the background; the result arrives as `Ev::ImageSaved(path or empty, name)`.
pub fn save(url: String, referer: String, ext: String, organize: bool, send: impl Fn(Ev) + Send + 'static) {
    std::thread::spawn(move || {
        let done = || -> Option<(String, u64)> {
            let fmt = fmt_of(&ext)?;
            let bytes = load(&url, &referer)?;
            let img = image::load_from_memory(&bytes).ok()?;
            let mut buf = Cursor::new(Vec::new());
            match fmt {
                // JPEG has no transparency: flatten to RGB
                ImageFormat::Jpeg => image::DynamicImage::ImageRgb8(img.to_rgb8()).write_to(&mut buf, fmt).ok()?,
                _ => img.write_to(&mut buf, fmt).ok()?,
            }
            let out = buf.into_inner();
            let path = crate::dl::target(&format!("{}.{ext}", stem_of(&url)), organize);
            std::fs::write(&path, &out).ok()?;
            Some((path, out.len() as u64))
        }();
        let (p, n) = done.unwrap_or_default();
        let p_is_empty = p.is_empty();
        send(Ev::ImageSaved(p, n, ext, if p_is_empty { url.chars().take(2000).collect() } else { String::new() }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_between_formats() {
        // a 2x2 PNG with transparency, built in memory
        let img = image::RgbaImage::from_fn(2, 2, |x, _| image::Rgba([200, 30 * x as u8, 10, if x == 0 { 255 } else { 128 }]));
        let mut png = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img).write_to(&mut png, ImageFormat::Png).unwrap();
        let png = png.into_inner();
        // base64 encode by hand for the data: URL path
        const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut enc = String::new();
        for c in png.chunks(3) {
            let n = (c[0] as u32) << 16 | (*c.get(1).unwrap_or(&0) as u32) << 8 | *c.get(2).unwrap_or(&0) as u32;
            for i in 0..4 {
                if i <= c.len() {
                    enc.push(T[(n >> (18 - 6 * i) & 63) as usize] as char);
                } else {
                    enc.push('=');
                }
            }
        }
        let url = format!("data:image/png;base64,{enc}");
        assert_eq!(load(&url, "").unwrap(), png);
        let dir = std::env::temp_dir().join(format!("owl-img-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Downloads")).unwrap();
        std::env::set_var("USERPROFILE", &dir);
        for (ext, _) in FORMATS {
            let (tx, rx) = std::sync::mpsc::channel();
            let tx = std::sync::Mutex::new(tx);
            save(url.clone(), String::new(), ext.to_string(), false, move |e| {
                if let Ev::ImageSaved(p, n, _, _) = e {
                    let _ = tx.lock().unwrap().send((p, n));
                }
            });
            let (p, n) = rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
            assert!(!p.is_empty() && n > 0, "{ext} failed");
            let back = image::open(&p).unwrap_or_else(|e| panic!("{ext} unreadable: {e}"));
            assert_eq!((back.width(), back.height()), (2, 2), "{ext}");
        }
    }
}
