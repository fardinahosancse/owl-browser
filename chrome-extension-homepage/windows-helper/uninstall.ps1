# Owl. Calm Home: disconnects the Windows helper from your browsers.
$name = "com.owl.calmhome"
foreach ($root in "HKCU:\Software\Google\Chrome", "HKCU:\Software\Microsoft\Edge", "HKCU:\Software\BraveSoftware\Brave-Browser", "HKCU:\Software\Chromium", "HKCU:\Software\Vivaldi") {
  $key = "$root\NativeMessagingHosts\$name"
  if (Test-Path $key) { Remove-Item $key -Force; "Removed from $root" }
}
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
Remove-Item (Join-Path $here "$name.json") -ErrorAction SilentlyContinue
"Done."
