# Owl. Calm Home: Windows helper (native messaging host).
# The browser starts this when the home page asks, and talks to it over stdin/stdout with
# length-prefixed JSON. It only answers these requests:
#   {"cmd":"status"}               -> {"wifi":{...}, "volume":{"level":0-100,"muted":bool}}
#   {"cmd":"volume","level":0-100} -> sets the Windows volume, then answers like "status"
#   {"cmd":"mute","on":true|false} -> mutes or unmutes, then answers like "status"
# It reads nothing else, writes no files and opens no network connection.

$ErrorActionPreference = "Stop"
$stdin = [Console]::OpenStandardInput()
$stdout = [Console]::OpenStandardOutput()

# Windows' own audio API (Core Audio), for the default playback device
Add-Type -Language CSharp -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
namespace OwlHome {
  [Guid("5CDF2C82-841E-4546-9722-0CF74078229A"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown), ComImport]
  interface IAudioEndpointVolume {
    int RegisterControlChangeNotify(IntPtr p); int UnregisterControlChangeNotify(IntPtr p); int GetChannelCount(out int c);
    int SetMasterVolumeLevel(float l, ref Guid g); int SetMasterVolumeLevelScalar(float l, ref Guid g);
    int GetMasterVolumeLevel(out float l); int GetMasterVolumeLevelScalar(out float l);
    int SetChannelVolumeLevel(uint n, float l, ref Guid g); int SetChannelVolumeLevelScalar(uint n, float l, ref Guid g);
    int GetChannelVolumeLevel(uint n, out float l); int GetChannelVolumeLevelScalar(uint n, out float l);
    int SetMute([MarshalAs(UnmanagedType.Bool)] bool m, ref Guid g); int GetMute([MarshalAs(UnmanagedType.Bool)] out bool m);
  }
  [Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown), ComImport]
  interface IMMDevice { int Activate(ref Guid id, int ctx, IntPtr p, [MarshalAs(UnmanagedType.IUnknown)] out object o); }
  [Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown), ComImport]
  interface IMMDeviceEnumerator { int EnumAudioEndpoints(int f, int s, out IntPtr d); int GetDefaultAudioEndpoint(int flow, int role, out IMMDevice d); }
  [ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")] class MMDeviceEnumerator { }
  public static class Audio {
    static IAudioEndpointVolume Endpoint() {
      var e = (IMMDeviceEnumerator)new MMDeviceEnumerator();
      IMMDevice d; Marshal.ThrowExceptionForHR(e.GetDefaultAudioEndpoint(0, 1, out d));
      Guid iid = typeof(IAudioEndpointVolume).GUID; object o;
      Marshal.ThrowExceptionForHR(d.Activate(ref iid, 23, IntPtr.Zero, out o));
      return (IAudioEndpointVolume)o;
    }
    public static int GetLevel() { float l; Marshal.ThrowExceptionForHR(Endpoint().GetMasterVolumeLevelScalar(out l)); return (int)Math.Round(l * 100); }
    public static void SetLevel(int v) { Guid g = Guid.Empty; Marshal.ThrowExceptionForHR(Endpoint().SetMasterVolumeLevelScalar(Math.Max(0, Math.Min(100, v)) / 100f, ref g)); }
    public static bool GetMute() { bool m; Marshal.ThrowExceptionForHR(Endpoint().GetMute(out m)); return m; }
    public static void SetMute(bool m) { Guid g = Guid.Empty; Marshal.ThrowExceptionForHR(Endpoint().SetMute(m, ref g)); }
  }
}
"@ | Out-Null

function Read-Message {
  $head = New-Object byte[] 4; $got = 0
  while ($got -lt 4) { $r = $stdin.Read($head, $got, 4 - $got); if ($r -le 0) { return $null }; $got += $r }
  $len = [BitConverter]::ToInt32($head, 0)
  if ($len -le 0 -or $len -gt 65536) { return $null }
  $buf = New-Object byte[] $len; $got = 0
  while ($got -lt $len) { $r = $stdin.Read($buf, $got, $len - $got); if ($r -le 0) { return $null }; $got += $r }
  return [Text.Encoding]::UTF8.GetString($buf) | ConvertFrom-Json
}

function Send-Message($obj) {
  $bytes = [Text.Encoding]::UTF8.GetBytes(($obj | ConvertTo-Json -Compress -Depth 5))
  $stdout.Write([BitConverter]::GetBytes([int]$bytes.Length), 0, 4)
  $stdout.Write($bytes, 0, $bytes.Length)
  $stdout.Flush()
}

function Get-Network {
  # Wi-Fi first: name and signal from netsh (Windows 11 24H2 and later need location access for it)
  try {
    # netsh writes in the console code page; switch it to UTF-8 so names with symbols stay intact
    $prev = [Console]::OutputEncoding
    [Console]::OutputEncoding = [Text.Encoding]::UTF8
    try { $lines = cmd.exe /d /c "chcp 65001 >nul & netsh wlan show interfaces" 2>$null } finally { [Console]::OutputEncoding = $prev }
    $state = ($lines | Where-Object { $_ -match '^\s*State\s*:' } | Select-Object -First 1) -replace '^[^:]*:\s*', ''
    $ssid = ($lines | Where-Object { $_ -match '^\s*SSID\s*:' } | Select-Object -First 1) -replace '^[^:]*:\s*', ''
    $sig = ($lines | Where-Object { $_ -match '^\s*Signal\s*:' } | Select-Object -First 1) -replace '[^0-9]', ''
    if ($state -match 'connected' -and $state -notmatch 'disconnected' -and $ssid) {
      return @{ kind = "wifi"; name = $ssid.Trim(); signal = [int]("0" + $sig); connected = $true }
    }
  } catch { }
  # otherwise what Windows calls the connection (for Wi-Fi this is usually the network name)
  try {
    $up = Get-NetAdapter -Physical -ErrorAction Stop | Where-Object Status -eq "Up" | Sort-Object { $_.MediaType -ne "802.3" }
    foreach ($a in $up) {
      $p = Get-NetConnectionProfile -InterfaceIndex $a.ifIndex -ErrorAction SilentlyContinue | Select-Object -First 1
      $wifi = $a.PhysicalMediaType -match "802\.11" -or $a.MediaType -match "802\.11" -or $a.Name -match "Wi-?Fi|WLAN|Wireless"
      return @{ kind = $(if ($wifi) { "wifi" } else { "ethernet" }); name = $(if ($p) { $p.Name } else { $a.Name }); signal = $null; connected = $true; note = $(if ($wifi) { "Signal strength needs location access for apps in Windows settings." } else { "" }) }
    }
  } catch { }
  return @{ kind = "none"; name = ""; signal = $null; connected = $false }
}

function Get-Status {
  $vol = $null
  try { $vol = @{ level = [OwlHome.Audio]::GetLevel(); muted = [OwlHome.Audio]::GetMute() } } catch { $vol = @{ error = $_.Exception.Message } }
  return @{ ok = $true; network = (Get-Network); volume = $vol }
}

while ($true) {
  $msg = Read-Message
  if ($null -eq $msg) { break }
  try {
    switch ($msg.cmd) {
      "status" { Send-Message (Get-Status) }
      "volume" { [OwlHome.Audio]::SetLevel([int]$msg.level); if ([int]$msg.level -gt 0) { [OwlHome.Audio]::SetMute($false) }; Send-Message (Get-Status) }
      "mute" { [OwlHome.Audio]::SetMute([bool]$msg.on); Send-Message (Get-Status) }
      default { Send-Message @{ ok = $false; error = "unknown request" } }
    }
  } catch {
    Send-Message @{ ok = $false; error = $_.Exception.Message }
  }
}
