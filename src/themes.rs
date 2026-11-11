//! Colour themes shared by the browser chrome and every owl:// page.
//! Every theme defines the same seven variables, so components never special-case a theme.
pub const THEMES: &str = r#"
:root{--bg:#fff;--ink:#111;--mute:#6b6b6b;--soft:#f2f2f0;--line:#d8d8d8;--red:#e30613;--on:#fff;color-scheme:light}
@media(prefers-color-scheme:dark){:root:not([data-theme]),:root[data-theme=system]{--bg:#101010;--ink:#f4f4f4;--mute:#8f8f8f;--soft:#1b1b1b;--line:#303030;--red:#ff3b30;--on:#101010;color-scheme:dark}}
:root[data-theme=dark]{--bg:#101010;--ink:#f4f4f4;--mute:#8f8f8f;--soft:#1b1b1b;--line:#303030;--red:#ff3b30;--on:#101010;color-scheme:dark}
:root[data-theme=black]{--bg:#000;--ink:#fff;--mute:#8c8c8c;--soft:#121212;--line:#2a2a2a;--red:#ff2b1f;--on:#000;color-scheme:dark}
:root[data-theme=navy]{--bg:#0b1a3a;--ink:#eef3ff;--mute:#8da2d0;--soft:#13285a;--line:#264078;--red:#ff5a36;--on:#0b1a3a;color-scheme:dark}
:root[data-theme=invert]{--bg:#e30613;--ink:#fff;--mute:#ffd3d6;--soft:#c9050f;--line:#f25a63;--red:#111;--on:#e30613;color-scheme:light}
:root[data-theme=sand]{--bg:#efe9dc;--ink:#171717;--mute:#766f60;--soft:#e4dcc9;--line:#cdc4ae;--red:#d6341a;--on:#efe9dc;color-scheme:light}
"#;

/// Themes picked at random when the user chooses "random".
pub const POOL: &[&str] = &["dark", "black", "navy", "invert", "sand", "light"];

pub fn is_dark(t: &str) -> Option<bool> {
    match t {
        "light" | "invert" | "sand" => Some(false),
        "dark" | "black" | "navy" => Some(true),
        _ => None,
    }
}
