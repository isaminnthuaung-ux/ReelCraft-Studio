$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskPackageInfo = Get-Content "$taskRoot\package.json" -Raw | ConvertFrom-Json
$taskVersion = $taskPackageInfo.version
$taskBuildLabel = if ($taskPackageInfo.buildNumber) { "-Build-$($taskPackageInfo.buildNumber)" } else { '' }
$taskArtifactName = "ReelCraftStudio-$taskVersion$taskBuildLabel"
$taskDownloads = Join-Path $taskRoot 'FinalBuild\Downloads'
$taskZip = Join-Path $taskDownloads "$taskArtifactName-Windows-x64.zip"
$taskExtract = Join-Path $taskRoot ('.tools\package-test-' + [guid]::NewGuid().ToString('N'))
Expand-Archive -LiteralPath $taskZip -DestinationPath $taskExtract
$taskApp = Join-Path $taskExtract 'ReelCraftStudio'
foreach ($taskRequired in @('ReelCraftStudio.exe','bin\ffmpeg.exe','bin\ffprobe.exe','READ-ME.md','UPDATE_LOG.md','licenses\ReelCraftStudio-MIT.txt','licenses\vidcoder\LICENSE.txt','licenses\ffmpeg-provenance.json','licenses\SOURCE-INFORMATION.txt','prerequisites\Install-WebView2.exe')) {
    if (-not (Test-Path -LiteralPath (Join-Path $taskApp $taskRequired) -PathType Leaf)) { throw "ZIP missing: $taskRequired" }
}
$taskOriginal = Join-Path $taskRoot 'src-tauri\target\release\long-to-shot-studio.exe'
if ((Get-FileHash $taskOriginal).Hash -ne (Get-FileHash "$taskApp\ReelCraftStudio.exe").Hash) { throw 'Packaged app differs from build.' }
$taskProductVersion = (Get-Item -LiteralPath "$taskApp\ReelCraftStudio.exe").VersionInfo.ProductVersion
if ($taskProductVersion -ne $taskVersion) { throw "Packaged app version mismatch: $taskProductVersion" }
if ($taskPackageInfo.buildNumber -and (Get-Item -LiteralPath "$taskApp\ReelCraftStudio.exe").VersionInfo.ProductName -ne $taskArtifactName) { throw 'Packaged app build name mismatch.' }
Add-Type -AssemblyName System.IO.Compression.FileSystem
$taskSourceZip = Join-Path $taskDownloads "$taskArtifactName-Source.zip"
$taskSourceArchive = [IO.Compression.ZipFile]::OpenRead($taskSourceZip)
try {
    foreach ($taskSourceName in @('package.json','src/App.tsx','src/macos.css','src/SystemInfo.tsx','src/WallpaperControls.tsx','src/WallpaperHandles.tsx','src/projectHistory.ts','src-tauri/src/system_info.rs','src-tauri/src/media.rs','src-tauri/src/export.rs','UPDATE_LOG.md')) {
        $taskSourceEntry = $taskSourceArchive.GetEntry($taskSourceName)
        if (-not $taskSourceEntry) { throw "Source archive missing: $taskSourceName" }
        $taskSourceStream = $taskSourceEntry.Open()
        try { $taskArchivedHash = (Get-FileHash -InputStream $taskSourceStream -Algorithm SHA256).Hash }
        finally { $taskSourceStream.Dispose() }
        if ($taskArchivedHash -ne (Get-FileHash -LiteralPath (Join-Path $taskRoot $taskSourceName) -Algorithm SHA256).Hash) {
            throw "Source archive is stale: $taskSourceName"
        }
    }
} finally { $taskSourceArchive.Dispose() }
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
