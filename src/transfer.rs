//! Moving in and out of Owl: bookmarks straight from Chrome or Edge, bookmarks from any browser's
//! HTML export, passwords from Chrome's CSV export, and exports in the same two formats.

/// (url, title, folder)
pub type Mark = (String, String, String);
/// (url, user, password)
pub type Login = (String, String, String);

/// Chrome-family bookmark file (`User Data\Default\Bookmarks`, JSON) for "chrome" or "edge".
pub fn chromium_bookmarks(which: &str) -> Result<Vec<Mark>, String> {
    let base = std::env::var("LOCALAPPDATA").map_err(|_| "LOCALAPPDATA is not set".to_string())?;
    let (dir, name) = match which {
        "edge" => (r"Microsoft\Edge\User Data", "Microsoft Edge"),
        _ => (r"Google\Chrome\User Data", "Google Chrome"),
    };
    let root = std::path::Path::new(&base).join(dir);
    // the last used profile is in Local State; Default otherwise
    let prof = std::fs::read_to_string(root.join("Local State"))
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v["profile"]["last_used"].as_str().map(|s| s.to_string()))
        .filter(|p| !p.contains(['/', '\\', '.']))
        .unwrap_or_else(|| "Default".into());
    let file = root.join(&prof).join("Bookmarks");
    let text = std::fs::read_to_string(&file).map_err(|_| format!("No {name} bookmarks were found on this PC."))?;
    let v: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).map_err(|_| format!("{name}'s bookmark file could not be read."))?;
    let mut out = Vec::new();
    fn walk(n: &serde_json::Value, folder: &str, out: &mut Vec<Mark>) {
        match n["type"].as_str() {
            Some("url") => {
                let u = n["url"].as_str().unwrap_or("");
                if u.starts_with("http://") || u.starts_with("https://") {
                    out.push((u.to_string(), n["name"].as_str().unwrap_or("").to_string(), folder.to_string()));
                }
            }
            Some("folder") => {
                let f = n["name"].as_str().unwrap_or("");
                let sub = if folder.is_empty() || folder == "Bookmarks bar" { f.to_string() } else { folder.to_string() };
                for c in n["children"].as_array().into_iter().flatten() {
                    walk(c, &sub, out);
                }
            }
            _ => {}
        }
    }
    for (k, label) in [("bookmark_bar", "Bookmarks bar"), ("other", "Other"), ("synced", "Mobile")] {
        for c in v["roots"][k]["children"].as_array().into_iter().flatten() {
            walk(c, label, &mut out);
        }
    }
    Ok(out)
}

fn unescape(s: &str) -> String {
    s.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&#39;", "'")
}

/// Netscape bookmark HTML, the format every browser exports.
pub fn parse_html(h: &str) -> Vec<Mark> {
    let mut out = Vec::new();
    let mut folders: Vec<String> = Vec::new();
    let lower = h.to_lowercase();
    let mut i = 0;
    while i < h.len() {
        let next = ["<h3", "<a ", "</dl"].iter().filter_map(|t| lower[i..].find(t).map(|p| (p + i, *t))).min();
        let Some((p, tag)) = next else { break };
        let end = lower[p..].find('>').map(|e| p + e + 1).unwrap_or(h.len());
        match tag {
            "<h3" => {
                let close = lower[end..].find("</h3>").map(|e| end + e).unwrap_or(end);
                folders.push(unescape(h[end..close].trim()));
                i = close;
            }
            "<a " => {
                let attrs = &h[p..end];
                let href = attrs.to_lowercase().find("href=\"").map(|k| {
                    let s = &attrs[k + 6..];
                    s[..s.find('"').unwrap_or(s.len())].to_string()
                });
                let close = lower[end..].find("</a>").map(|e| end + e).unwrap_or(end);
                if let Some(u) = href.map(|u| unescape(&u)).filter(|u| u.starts_with("http://") || u.starts_with("https://")) {
                    let f = folders.last().cloned().unwrap_or_default();
                    out.push((u, unescape(h[end..close].trim()), f));
                }
                i = close;
            }
            _ => {
                folders.pop();
                i = end;
            }
        }
        if i <= p {
            i = p + 1;
        }
    }
    out
}

/// One CSV record per line, with quoted fields and doubled quotes (RFC 4180).
fn csv_rows(t: &str) -> Vec<Vec<String>> {
    let (mut rows, mut row, mut f, mut q) = (Vec::new(), Vec::new(), String::new(), false);
    let mut it = t.chars().peekable();
    while let Some(c) = it.next() {
        match (c, q) {
            ('"', true) if it.peek() == Some(&'"') => {
                f.push('"');
                it.next();
            }
            ('"', _) => q = !q,
            (',', false) => row.push(std::mem::take(&mut f)),
            ('\r', false) => {}
            ('\n', false) => {
                row.push(std::mem::take(&mut f));
                rows.push(std::mem::take(&mut row));
            }
            _ => f.push(c),
        }
    }
    if !f.is_empty() || !row.is_empty() {
        row.push(f);
        rows.push(row);
    }
    rows
}

/// Chrome, Edge and Firefox password exports: a header with url, username and password columns.
pub fn parse_passwords(t: &str) -> Result<Vec<Login>, String> {
    let rows = csv_rows(t.trim_start_matches('\u{feff}'));
    let head: Vec<String> = rows.first().ok_or("The file is empty.")?.iter().map(|h| h.trim().to_lowercase()).collect();
    let col = |names: &[&str]| head.iter().position(|h| names.contains(&h.as_str()));
    let (u, n, p) = (
        col(&["url", "origin", "login_uri", "website"]).ok_or("This file has no url column. Export passwords from Chrome as CSV.")?,
        col(&["username", "login_username", "user"]).ok_or("This file has no username column.")?,
        col(&["password", "login_password"]).ok_or("This file has no password column.")?,
    );
    Ok(rows
        .iter()
        .skip(1)
        .filter_map(|r| Some((r.get(u)?.trim().to_string(), r.get(n)?.to_string(), r.get(p)?.to_string())))
        .filter(|(u, _, p)| (u.starts_with("http://") || u.starts_with("https://")) && !p.is_empty() && p.len() < 512)
        .take(5000)
        .collect())
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Bookmarks as Netscape HTML, grouped in folders, importable by every browser.
pub fn export_html(marks: &[Mark]) -> String {
    let mut groups: Vec<(String, Vec<&Mark>)> = Vec::new();
    for m in marks {
        let g = if m.2.is_empty() { "Owl." } else { m.2.as_str() };
        match groups.iter_mut().find(|(n, _)| n == g) {
            Some((_, v)) => v.push(m),
            None => groups.push((g.to_string(), vec![m])),
        }
    }
    let mut h = String::from("<!DOCTYPE NETSCAPE-Bookmark-file-1>\n<!-- Exported from Owl. -->\n<META HTTP-EQUIV=\"Content-Type\" CONTENT=\"text/html; charset=UTF-8\">\n<TITLE>Bookmarks</TITLE>\n<H1>Bookmarks</H1>\n<DL><p>\n");
    for (g, v) in groups {
        h.push_str(&format!("    <DT><H3>{}</H3>\n    <DL><p>\n", esc(&g)));
        for m in v {
            h.push_str(&format!("        <DT><A HREF=\"{}\">{}</A>\n", esc(&m.0), esc(if m.1.is_empty() { &m.0 } else { &m.1 })));
        }
        h.push_str("    </DL><p>\n");
    }
    h.push_str("</DL><p>\n");
    h
}

fn csv_field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// Passwords in Chrome's CSV layout (name,url,username,password,note).
pub fn export_csv(logins: &[Login]) -> String {
    let mut o = String::from("name,url,username,password,note\n");
    for (u, n, p) in logins {
        let host = u.split("://").nth(1).unwrap_or(u).split('/').next().unwrap_or("");
        o.push_str(&format!("{},{},{},{},\n", csv_field(host), csv_field(u), csv_field(n), csv_field(p)));
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trips() {
        let h = export_html(&[("https://a.example/x?y=1&z=2".into(), "A \"q\"".into(), "Work".into()), ("https://b.example/".into(), "".into(), "".into())]);
        let back = parse_html(&h);
        assert_eq!(back.len(), 2);
        assert_eq!(back[0], ("https://a.example/x?y=1&z=2".into(), "A \"q\"".into(), "Work".into()));
        let c = export_csv(&[("https://s.example/login".into(), "me, myself".into(), "p\"w,1".into())]);
        let l = parse_passwords(&c).unwrap();
        assert_eq!(l, vec![("https://s.example/login".into(), "me, myself".into(), "p\"w,1".into())]);
    }
}
