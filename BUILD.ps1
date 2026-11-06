$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
. "$PSScriptRoot\scripts\env.ps1"
& npm.cmd run tauri -- build --no-bundle
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$taskRelease = Join-Path $PSScriptRoot 'src-tauri\target\release'
New-Item -ItemType Directory -Force -Path (Join-Path $taskRelease 'bin') | Out-Null
foreach ($taskTool in @('ffmpeg', 'ffprobe')) {
    # Chocolatey exposes shims, so prefer its actual FFmpeg distribution.
    $taskBinary = Get-ChildItem 'C:\ProgramData\chocolatey\lib\ffmpeg*' -Recurse -Filter "$taskTool.exe" -ErrorAction SilentlyContinue | Select-Object -First 1 -ExpandProperty FullName
    if (-not $taskBinary) { $taskBinary = (Get-Command "$taskTool.exe" -ErrorAction SilentlyContinue).Source }
    if ($taskBinary) { Copy-Item -LiteralPath $taskBinary -Destination (Join-Path $taskRelease "bin\$taskTool.exe") -Force }
}
Copy-Item -LiteralPath "$taskRelease\long-to-shot-studio.exe" -Destination "$taskRelease\ReelCraftStudio.exe" -Force
Write-Host "Built: $taskRelease\ReelCraftStudio.exe"
