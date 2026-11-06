$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskVersion = (Get-Content "$taskRoot\package.json" -Raw | ConvertFrom-Json).version
$taskDownloads = Join-Path $taskRoot 'FinalBuild\Downloads'
$taskZip = Join-Path $taskDownloads "ReelCraftStudio-$taskVersion-Windows-x64.zip"
$taskExtract = Join-Path $taskRoot ('.tools\package-test-' + [guid]::NewGuid().ToString('N'))
Expand-Archive -LiteralPath $taskZip -DestinationPath $taskExtract
$taskApp = Join-Path $taskExtract 'ReelCraftStudio'
foreach ($taskRequired in @('ReelCraftStudio.exe','bin\ffmpeg.exe','bin\ffprobe.exe','READ-ME.md','licenses\ReelCraftStudio-MIT.txt','licenses\vidcoder\LICENSE.txt','licenses\ffmpeg-provenance.json','licenses\SOURCE-INFORMATION.txt','prerequisites\Install-WebView2.exe')) {
    if (-not (Test-Path -LiteralPath (Join-Path $taskApp $taskRequired) -PathType Leaf)) { throw "ZIP missing: $taskRequired" }
}
$taskOriginal = Join-Path $taskRoot 'src-tauri\target\release\long-to-shot-studio.exe'
if ((Get-FileHash $taskOriginal).Hash -ne (Get-FileHash "$taskApp\ReelCraftStudio.exe").Hash) { throw 'Packaged app differs from build.' }
$taskSample = Join-Path $taskExtract 'synthetic.mp4'
& "$taskApp\bin\ffmpeg.exe" -hide_banner -loglevel error -f lavfi -i 'testsrc2=size=320x240:rate=10:duration=1' -f lavfi -i 'sine=frequency=440:duration=1' -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest $taskSample
if ($LASTEXITCODE -ne 0) { throw 'Bundled FFmpeg failed synthetic H.264/AAC encode.' }
$taskProbe = & "$taskApp\bin\ffprobe.exe" -v error -show_streams -of json $taskSample
if ($LASTEXITCODE -ne 0) { throw 'Bundled ffprobe failed.' }
$taskStreams = ($taskProbe | ConvertFrom-Json).streams
if (-not ($taskStreams | Where-Object codec_name -eq h264) -or -not ($taskStreams | Where-Object codec_name -eq aac)) { throw 'Expected H.264 and AAC streams not found.' }
foreach ($taskLine in Get-Content (Join-Path $taskDownloads 'SHA256SUMS.txt')) {
    $taskHash,$taskName = $taskLine -split '  ',2
    if ((Get-FileHash -LiteralPath (Join-Path $taskDownloads $taskName) -Algorithm SHA256).Hash.ToLowerInvariant() -ne $taskHash) { throw "Checksum failed: $taskName" }
}
Write-Host 'Extracted Windows ZIP passed: app identity, required files, H.264/AAC encode, ffprobe, and checksums.'