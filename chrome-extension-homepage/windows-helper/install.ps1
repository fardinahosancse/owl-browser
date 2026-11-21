# Owl. Calm Home: connects the Windows helper to your browsers, for your Windows user only (no
# administrator rights). Lets the home page show the Wi-Fi name and signal and control the volume.
# Run:  powershell -ExecutionPolicy Bypass -File install.ps1      Undo:  uninstall.ps1
$ErrorActionPreference = "Stop"
$name = "com.owl.calmhome"
# the extension's fixed ID (from the "key" in manifest.json): only it may start the helper
$origin = "chrome-extension://lejabghpgomlgigjhibhnocaddmlofid/"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$manifest = Join-Path $here "$name.json"
@{
  name            = $name
  description     = "Owl. Calm Home: Wi-Fi name and signal, Windows volume"
  path            = (Join-Path $here "owl_home_host.cmd")
  type            = "stdio"
  allowed_origins = @($origin)
} | ConvertTo-Json | Set-Content -Path $manifest -Encoding UTF8

$browsers = [ordered]@{
  "Google Chrome"  = "HKCU:\Software\Google\Chrome\NativeMessagingHosts"
  "Microsoft Edge" = "HKCU:\Software\Microsoft\Edge\NativeMessagingHosts"
  "Brave"          = "HKCU:\Software\BraveSoftware\Brave-Browser\NativeMessagingHosts"
  "Chromium"       = "HKCU:\Software\Chromium\NativeMessagingHosts"
  "Vivaldi"        = "HKCU:\Software\Vivaldi\NativeMessagingHosts"
}
foreach ($b in $browsers.GetEnumerator()) {
  $key = Join-Path $b.Value $name
  New-Item -Path $key -Force | Out-Null
  Set-Item -Path $key -Value $manifest
  "Connected for $($b.Key)"
}
"Done. Open a new tab: Wi-Fi and volume appear at the bottom right."
