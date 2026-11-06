# Build a separately named UI update without replacing a running application.
$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
. "$PSScriptRoot\scripts\env.ps1"
& npm.cmd run build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$taskStage = Join-Path $PSScriptRoot '.tools\ui-build'
New-Item -ItemType Directory -Force -Path "$taskStage\icons", "$taskStage\capabilities" | Out-Null
$taskManifest = Get-Content -LiteralPath 'src-tauri\Cargo.toml' -Raw
$taskManifest = $taskManifest.Replace('name = "long-to-shot-studio"', 'name = "long-to-shot-studio-ui"')
$taskManifest = $taskManifest.Replace('edition = "2021"', "edition = `"2021`"`nbuild = `"../../src-tauri/build.rs`"")
$taskManifest += "`n[[bin]]`nname = `"long-to-shot-studio-ui`"`npath = `"../../src-tauri/src/main.rs`"`n"
[System.IO.File]::WriteAllText("$taskStage\Cargo.toml", $taskManifest)
$taskConfig = Get-Content -LiteralPath 'src-tauri\tauri.conf.json' -Raw | ConvertFrom-Json
$taskConfig.build.frontendDist = (Join-Path $PSScriptRoot 'dist')
[System.IO.File]::WriteAllText("$taskStage\tauri.conf.json", ($taskConfig | ConvertTo-Json -Depth 20))
Copy-Item -Path 'src-tauri\icons\*' -Destination "$taskStage\icons" -Force
Copy-Item -Path 'src-tauri\capabilities\*' -Destination "$taskStage\capabilities" -Force
& cargo build --offline --release --manifest-path "$taskStage\Cargo.toml" --target-dir (Join-Path $PSScriptRoot 'src-tauri\target')
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
Copy-Item -LiteralPath "$PSScriptRoot\src-tauri\target\release\long-to-shot-studio-ui.exe" -Destination "$PSScriptRoot\src-tauri\target\release\ReelCraftStudio.exe" -Force
Write-Host "Built: $PSScriptRoot\src-tauri\target\release\ReelCraftStudio.exe"
