//! Colour themes shared by the browser chrome and every owl:// page.
//! Every theme defines the same variables, so components never special-case a theme:
//!   --bg/--ink/--soft/--line   ground, text, panels, hairlines
//!   --mute                     secondary text on the ground
//!   --red                      the accent; --onred is text drawn ON the accent (AA contrast)
//!   --rink                     the accent drawn on an --ink background
//!   --on                       text drawn on an --ink background
//!   --mute-on                  secondary text drawn on an --ink background (hover, selected)
pub const THEMES: &str = r#"
:root{--bg:#fff;--ink:#111;--mute:#595959;--soft:#f2f2f0;--line:#d0d0d0;--red:#e30613;--onred:#fff;--on:#fff;--mute-on:#c4c4c4;--rink:#ff4d4d;color-scheme:light}
@media(prefers---rink:#d70015;color-scheme:dark){:root:not([data-theme]),:root[data-theme=system]{--bg:#101010;--ink:#f4f4f4;--mute:#9a9a9a;--soft:#1b1b1b;--line:#383838;--red:#ff3b30;--onred:#111;--on:#101010;--mute-on:#555;color-scheme:dark}}
:root[data-theme=dark]{--bg:#101010;--ink:#f4f4f4;--mute:#9a9a9a;--soft:#1b1b1b;--line:#383838;--red:#ff3b30;--onred:#111;--on:#101010;--mute-on:#555;--rink:#d70015;color-scheme:dark}
:root[data-theme=black]{--bg:#000;--ink:#fff;--mute:#9a9a9a;--soft:#121212;--line:#303030;--red:#ff2b1f;--onred:#000;--on:#000;--mute-on:#555;--rink:#d70015;color-scheme:dark}
:root[data-theme=navy]{--bg:#0b1a3a;--ink:#eef3ff;--mute:#9db1da;--soft:#13285a;--line:#2d4a8a;--red:#ff5a36;--onred:#0b1a3a;--on:#0b1a3a;--mute-on:#44557f;--rink:#d62a0b;color-scheme:dark}
:root[data-theme=invert]{--bg:#e30613;--ink:#fff;--mute:#fff;--soft:#c9050f;--line:#ff8f97;--red:#fff;--onred:#e30613;--on:#e30613;--mute-on:#7a1018;--rink:#e30613;color-scheme:light}
:root[data-theme=sand]{--bg:#efe9dc;--ink:#171717;--mute:#5d5748;--soft:#e4dcc9;--line:#bdb39b;--red:#b8290f;--onred:#fff;--on:#efe9dc;--mute-on:#d2cab6;--rink:#ff6a4d;color-scheme:light}
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
