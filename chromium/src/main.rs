//! Development build without the sandbox: `cargo run --no-default-features`. The real build is the
//! DLL in `lib.rs`, started by CEF's sandbox bootstrap (see README.md).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if cfg!(feature = "sandbox") {
        eprintln!("This build has the sandbox on: start owl-chromium.exe (CEF's bootstrap) next to owl-chromium.dll instead.");
        std::process::exit(2);
    }
    let _ = cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);
    let args = cef::args::Args::new();
    let Some(cmd_line) = args.as_cmd_line() else { return };
    std::process::exit(owl_chromium::run(args.as_main_args(), &cmd_line, std::ptr::null_mut()));
}
