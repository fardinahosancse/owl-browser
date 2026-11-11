# Publishes docs/wiki/*.md to the GitHub wiki of this repository.
# GitHub creates the wiki's git repository only after the first page is added in the web UI
# (Wiki tab > "Create the first page"). Do that once, then run this script.
param([string]$Repo = "fardinahosancse/owl-browser")
$ErrorActionPreference = "Stop"
$src = Join-Path $PSScriptRoot "..\docs\wiki"
$tmp = Join-Path ([IO.Path]::GetTempPath()) ("owl-wiki-" + [guid]::NewGuid())
git clone "https://github.com/$Repo.wiki.git" $tmp
Copy-Item "$src\*.md" $tmp -Force
Push-Location $tmp
git add -A
if (git status --porcelain) { git commit -m "Update wiki from docs/wiki"; git push origin HEAD } else { "Wiki already up to date." }
Pop-Location
Remove-Item $tmp -Recurse -Force
