# Prints the CHANGELOG.md section for one version (without the heading) to stdout and,
# if -OutFile is given, to a file. Fails if the version has no section.
param(
  [Parameter(Mandatory = $true)][string]$Version,
  [string]$OutFile = "",
  [string]$Changelog = (Join-Path $PSScriptRoot "..\CHANGELOG.md")
)

$Version = $Version.TrimStart("v")
$lines = Get-Content -LiteralPath $Changelog -Encoding UTF8
$start = -1
for ($i = 0; $i -lt $lines.Count; $i++) {
  if ($lines[$i] -match "^## \[$([regex]::Escape($Version))\]") { $start = $i + 1; break }
}
if ($start -lt 0) { Write-Error "No section for version $Version in CHANGELOG.md"; exit 1 }

$body = New-Object System.Collections.Generic.List[string]
for ($j = $start; $j -lt $lines.Count; $j++) {
  if ($lines[$j] -match "^## \[" -or $lines[$j] -match "^\[[^\]]+\]: ") { break }
  $body.Add($lines[$j])
}
$text = ($body -join "`n").Trim()
if (-not $text) { Write-Error "The section for $Version is empty"; exit 1 }

if ($OutFile) { Set-Content -LiteralPath $OutFile -Value $text -Encoding UTF8 }
$text
