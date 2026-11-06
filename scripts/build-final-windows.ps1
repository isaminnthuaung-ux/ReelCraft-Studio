$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
Set-Location $taskRoot
. "$PSScriptRoot\env.ps1"
foreach ($taskCommand in @('npm.cmd', 'cargo', 'ffmpeg', 'ffprobe')) {
    if (-not (Get-Command $taskCommand -ErrorAction SilentlyContinue)) { throw "Missing $taskCommand. See README build requirements." }
}
& npm.cmd ci
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
& "$taskRoot\BUILD.ps1"
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
$taskRelease = Join-Path $taskRoot 'src-tauri\target\release'
$taskOutput = Join-Path $taskRoot 'FinalBuild\Windows'
New-Item -ItemType Directory -Force -Path $taskOutput | Out-Null
Copy-Item -LiteralPath "$taskRelease\ReelCraftStudio.exe" -Destination $taskOutput -Force
Write-Host "Final app: $taskOutput\ReelCraftStudio.exe"
Write-Host 'FFmpeg and ffprobe must be installed on the destination machine and available on PATH.'
Write-Host 'Microsoft Edge WebView2 Runtime is also required. This is a portable app, not an installer.'
