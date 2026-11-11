# Prints a "Resolved issues" Markdown section: every issue closed as completed since the previous
# release, grouped by priority label. Prints nothing if there are none.
#   ./scripts/resolved-issues.ps1 -Repo owner/name -Since 2026-01-01T00:00:00Z
param(
  [Parameter(Mandatory = $true)][string]$Repo,
  [string]$Since = ""
)

$raw = gh issue list --repo $Repo --state closed --limit 300 --json number,title,labels,closedAt,stateReason
$issues = $raw | ConvertFrom-Json | Where-Object {
  $_.stateReason -eq "COMPLETED" -and ((-not $Since) -or ([datetime]$_.closedAt -gt [datetime]$Since))
}
if (-not $issues) { return }

function Prio($i) {
  foreach ($p in "P0-critical", "P1-high", "P2-medium", "P3-low") {
    if ($i.labels.name -contains $p) { return $p }
  }
  return "Other"
}

$names = @{ "P0-critical" = "Critical"; "P1-high" = "High priority"; "P2-medium" = "Medium priority"; "P3-low" = "Low priority"; "Other" = "Other" }
$out = New-Object System.Collections.Generic.List[string]
$out.Add("### Resolved issues")
$out.Add("")
foreach ($p in "P0-critical", "P1-high", "P2-medium", "P3-low", "Other") {
  $group = $issues | Where-Object { (Prio $_) -eq $p } | Sort-Object number
  if (-not $group) { continue }
  $out.Add("**$($names[$p])**")
  foreach ($i in $group) { $out.Add("- #$($i.number) $($i.title)") }
  $out.Add("")
}
$out -join "`n"
