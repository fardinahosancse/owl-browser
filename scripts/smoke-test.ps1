# End-to-end smoke test of the real Owl executable. It uses a throwaway profile and Downloads
# folder, drives Owl with the OWL_TEST_CMDS hook (the same commands the browser UI sends when you
# click), and checks the results on disk:
#   first-run setup, a fast (segmented) download, the Data saver toggle, Save image as, and the
#   seal that runs when Windows shuts down.
# Usage: scripts/smoke-test.ps1 -Exe path\to\owl.exe   (needs Python 3 for the test server)
param([Parameter(Mandatory = $true)][string]$Exe, [int]$Port = 8799, [int]$Seconds = 60)
$ErrorActionPreference = "Stop"
$root = Join-Path ([IO.Path]::GetTempPath()) ("owl-smoke-" + [guid]::NewGuid().ToString("N").Substring(0, 8))
$data = Join-Path $root "appdata"; $userp = Join-Path $root "user"
New-Item -ItemType Directory -Force (Join-Path $data "Owl"), (Join-Path $userp "Downloads") | Out-Null
'{"theme":"light","autolock":0,"organize":true}' | Set-Content (Join-Path $data "Owl\config.json") -Encoding ASCII

$server = Start-Process python -ArgumentList (Join-Path $PSScriptRoot "test-server.py"), $Port -PassThru -NoNewWindow -RedirectStandardOutput (Join-Path $root "server.txt")
Start-Sleep 2
$bigHash = (Get-Content (Join-Path $root "server.txt") -TotalCount 1).Trim()

$base = "http://127.0.0.1:$Port"
$cmds = @(
  @(3, @{ cmd = "setup"; name = "Smoke Test"; email = "smoke@example.com"; pass = "quiet owl reads maps" }),
  @(8, @{ cmd = "navigate"; url = "$base/page.html" }),
  @(11, @{ cmd = "dl_add"; url = "$base/big.bin" }),
  @(13, @{ cmd = "quick_saver" }),
  @(15, @{ cmd = "_save_image"; url = "$base/pic.png"; ext = "jpg" }),
  @(26, @{ cmd = "_end_session" })
)
$env:OWL_TEST_CMDS = (ConvertTo-Json -InputObject $cmds -Depth 5 -Compress)
$env:LOCALAPPDATA = $data; $env:USERPROFILE = $userp
$owl = Start-Process $Exe -PassThru
$fail = @()
try {
  $deadline = (Get-Date).AddSeconds($Seconds)
  $sealed = Join-Path $data "Owl\profile\sealed.owl"
  while ((Get-Date) -lt $deadline -and -not (Test-Path $sealed)) { Start-Sleep 1 }
  Start-Sleep 3
  $cfg = Get-Content (Join-Path $data "Owl\config.json") -Raw | ConvertFrom-Json
  if ($cfg.pin_mode -ne "pass" -or [string]::IsNullOrEmpty($cfg.pin_hash)) { $fail += "setup did not set a passphrase" }
  if ([int]$cfg.compress -le 0) { $fail += "Data saver toggle did not turn it on" }
  $big = Get-ChildItem (Join-Path $userp "Downloads") -Recurse -Filter "big.bin" -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $big) { $fail += "fast download did not finish" }
  elseif ((Get-FileHash $big.FullName -Algorithm SHA256).Hash.ToLower() -ne $bigHash) { $fail += "downloaded file is corrupt" }
  if (Get-ChildItem (Join-Path $userp "Downloads") -Recurse -Filter "*.owlpart" -ErrorAction SilentlyContinue) { $fail += "a partial download was left behind" }
  $img = Get-ChildItem (Join-Path $userp "Downloads") -Recurse -Filter "pic*.jpg" -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $img) { $fail += "Save image as JPEG produced no file" }
  elseif ([IO.File]::ReadAllBytes($img.FullName)[0..1] -join "," -ne "255,216") { $fail += "saved image is not a JPEG" }
  if (-not (Test-Path $sealed)) { $fail += "shutdown did not seal the profile" }
  if (Test-Path (Join-Path $data "Owl\profile\EBWebView\Default\Network")) { $fail += "cookies were left readable after the shutdown seal" }
  $bytes = if (Test-Path $sealed) { [IO.File]::ReadAllBytes($sealed) } else { @() }
  if ($bytes.Length -ge 8 -and [Text.Encoding]::ASCII.GetString($bytes, 0, 8) -ne "OWLPROF3") { $fail += "sealed file is not in the current format" }
} finally {
  if (-not $owl.HasExited) { Stop-Process -Id $owl.Id -Force }
  if (-not $server.HasExited) { Stop-Process -Id $server.Id -Force }
  Remove-Item Env:OWL_TEST_CMDS -ErrorAction SilentlyContinue
}
if ($fail.Count) {
  "SMOKE TEST FAILED:"; $fail | ForEach-Object { " - $_" }
  "(files kept in $root)"; exit 1
}
"Smoke test passed: setup, fast download ($([math]::Round($big.Length/1MB,1)) MB, checksum ok), Data saver toggle, Save image as JPEG, shutdown seal (OWLPROF3)."
Remove-Item $root -Recurse -Force -ErrorAction SilentlyContinue
