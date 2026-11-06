$ErrorActionPreference = 'Stop'
Set-Location $PSScriptRoot
. "$PSScriptRoot\scripts\env.ps1"
& npm.cmd run tauri -- dev
exit $LASTEXITCODE
