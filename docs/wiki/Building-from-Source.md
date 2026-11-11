# Building from Source

## Prerequisites

| Requirement | Notes |
|---|---|
| Rust stable 1.80 or newer | `rustup default stable` |
| A Windows linker toolchain | Visual Studio Build Tools (MSVC, "Desktop development with C++") **or** MSYS2/MinGW-w64 with the `x86_64-pc-windows-gnu` target |
| Git | |
| Inno Setup 6 | Only to build the installer |

## Build

```powershell
git clone https://github.com/fardinahosancse/owl-browser.git
cd owl-browser
cargo build --release --locked
```

The output is `target\release\owl.exe`. With the GNU toolchain the WebView2 loader is a separate DLL under `target\release\build\webview2-com-sys-*\out\x64\WebView2Loader.dll`; keep it next to `owl.exe`. MSVC links it statically.

## Installer

```powershell
$env:OWL_VERSION = "0.1.2"
New-Item -ItemType Directory dist -Force | Out-Null
Copy-Item target\release\owl.exe dist\Owl.exe
iscc installer\owl.iss
```

## Contributing

Read [CONTRIBUTING.md](https://github.com/fardinahosancse/owl-browser/blob/main/CONTRIBUTING.md). Owl uses an issue-first workflow: file the issue with a priority label, work on it, then close it so it appears in the release notes.
