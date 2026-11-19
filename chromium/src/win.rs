//! The entry point CEF's `bootstrap.exe` calls when the sandbox is on. The bootstrap executable
//! (shipped as `owl-chromium.exe`) sets up Chromium's sandbox, then loads `owl-chromium.dll` (this
//! crate) and calls `RunWinMain` for the browser process and for every child process.

use cef::*;

#[unsafe(no_mangle)]
unsafe extern "C" fn RunWinMain(instance: sys::HINSTANCE, _command_line: *const u8, _command_show: i32, sandbox_info: *mut u8) -> i32 {
    crate::trace("RunWinMain");
    // CEF needs its API version set before any other CEF call
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let main_args = MainArgs { instance };
    let args = args::Args::from(main_args);
    let Some(cmd_line) = args.as_cmd_line() else {
        return 1;
    };
    crate::run(args.as_main_args(), &cmd_line, sandbox_info)
}

/// The same for CEF's console bootstrap (`bootstrapc.exe`), which prints errors to the console.
#[unsafe(no_mangle)]
unsafe extern "C" fn RunConsoleMain(_argc: i32, _argv: *mut *mut u8, sandbox_info: *mut u8) -> i32 {
    crate::trace("RunConsoleMain");
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let args = args::Args::new();
    crate::trace("RunConsoleMain: args read");
    let Some(cmd_line) = args.as_cmd_line() else {
        return 1;
    };
    crate::run(args.as_main_args(), &cmd_line, sandbox_info)
}
