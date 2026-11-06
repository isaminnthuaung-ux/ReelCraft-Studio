@echo off
setlocal
cd /d "%~dp0"
if exist "src-tauri\target\release\ReelCraftStudio.exe" (
    start "" "src-tauri\target\release\ReelCraftStudio.exe"
    exit /b 0
)
set "LONG_TO_SHOT_ROOT=%~dp0"
powershell.exe -NoProfile -Command "$r = Join-Path $env:LONG_TO_SHOT_ROOT 'src-tauri\target\release'; $paths = @((Join-Path $r 'long-to-shot-studio.exe'), (Join-Path $r 'long-to-shot-studio-ui.exe')); $app = Get-Item -LiteralPath $paths -ErrorAction SilentlyContinue | Sort-Object LastWriteTime -Descending | Select-Object -First 1; if ($app) { Start-Process -FilePath $app.FullName; exit 0 }; exit 1"
if not errorlevel 1 exit /b 0
echo The Windows executable has not been built yet.
echo Run BUILD.bat first. See README.md for requirements.
pause
