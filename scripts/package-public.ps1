param(
    [string]$Executable = '',
    [string]$DownloadCache = '',
    [string]$WebViewBootstrapper = '',
    [switch]$SkipWebViewDownload
)
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
Set-Location $taskRoot
if (-not $Executable) { $Executable = Join-Path $taskRoot 'src-tauri\target\release\long-to-shot-studio.exe' }
if (-not $DownloadCache) { $DownloadCache = Join-Path $taskRoot '.tools\release-downloads' }
if (-not (Test-Path -LiteralPath $Executable -PathType Leaf)) { throw 'Build the Windows release executable first.' }
$taskPackageInfo = Get-Content package.json -Raw | ConvertFrom-Json
$taskVersion = $taskPackageInfo.version
$taskBuildLabel = if ($taskPackageInfo.buildNumber) { "-Build-$($taskPackageInfo.buildNumber)" } else { '' }
$taskArtifactName = "ReelCraftStudio-$taskVersion$taskBuildLabel"
$taskManifest = Get-Content "$PSScriptRoot\ffmpeg-windows.json" -Raw | ConvertFrom-Json
$taskOutput = Join-Path $taskRoot 'FinalBuild\Downloads'
$taskStage = Join-Path $taskRoot ('.tools\portable-' + [guid]::NewGuid().ToString('N'))
$taskPortable = Join-Path $taskStage 'ReelCraftStudio'
New-Item -ItemType Directory -Force -Path $DownloadCache,$taskOutput,"$taskPortable\bin","$taskPortable\licenses","$taskPortable\prerequisites" | Out-Null

$taskArchive = Join-Path $DownloadCache 'ffmpeg.zip'
if (-not (Test-Path -LiteralPath $taskArchive)) {
    Invoke-WebRequest -Uri $taskManifest.url -OutFile $taskArchive
}
if ((Get-FileHash -LiteralPath $taskArchive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $taskManifest.sha256) {
    throw 'FFmpeg checksum mismatch. Remove the cached ffmpeg.zip and retry; no unverified executable will be packaged.'
}
Expand-Archive -LiteralPath $taskArchive -DestinationPath "$taskStage\ffmpeg"
$taskFFmpeg = Get-ChildItem -LiteralPath "$taskStage\ffmpeg" -Directory | Select-Object -First 1
foreach ($taskTool in @('ffmpeg','ffprobe')) {
    Copy-Item -LiteralPath (Join-Path $taskFFmpeg.FullName "bin\$taskTool.exe") -Destination "$taskPortable\bin"
    & "$taskPortable\bin\$taskTool.exe" -version | Out-File "$taskPortable\licenses\$taskTool-version.txt" -Encoding utf8
    if ($LASTEXITCODE -ne 0) { throw "$taskTool cannot run." }
}
Copy-Item -LiteralPath $Executable -Destination "$taskPortable\ReelCraftStudio.exe"
# Preserve every upstream non-binary document, including GPL notices and build details.
$taskFFmpegNotices = Join-Path $taskPortable 'licenses\ffmpeg'
New-Item -ItemType Directory -Path $taskFFmpegNotices | Out-Null
Get-ChildItem -LiteralPath $taskFFmpeg.FullName | Where-Object { $_.Name -ne 'bin' } | Copy-Item -Destination $taskFFmpegNotices -Recurse
Copy-Item -LiteralPath "$taskRoot\LICENSE" -Destination "$taskPortable\licenses\ReelCraftStudio-MIT.txt"
Copy-Item -LiteralPath "$taskRoot\public\icons\vidcoder" -Destination "$taskPortable\licenses\vidcoder" -Recurse
Copy-Item -LiteralPath "$PSScriptRoot\ffmpeg-windows.json" -Destination "$taskPortable\licenses\ffmpeg-provenance.json"
Copy-Item -LiteralPath "$taskRoot\docs\WINDOWS-DOWNLOAD.md" -Destination "$taskPortable\READ-ME.md"
Copy-Item -LiteralPath "$taskRoot\UPDATE_LOG.md" -Destination "$taskPortable\UPDATE_LOG.md"
# Include dependency notices for the actual installed frontend and native dependencies.
$taskNpmNotices = Join-Path $taskPortable 'licenses\npm'
New-Item -ItemType Directory -Path $taskNpmNotices | Out-Null
Get-ChildItem -LiteralPath "$taskRoot\node_modules" -Filter package.json -Recurse -File | ForEach-Object {
    $taskPackage = Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json
    if ($taskPackage.name -and $taskPackage.version) {
        $taskNoticeName = ($taskPackage.name -replace '[/@]','_') + '-' + $taskPackage.version
        $taskLicenseFiles = @(Get-ChildItem -LiteralPath $_.DirectoryName -File | Where-Object { $_.Name -match '^(LICENSE|LICENCE|COPYING|NOTICE)' })
        if ($taskLicenseFiles.Count) {
            $taskNoticeFolder = Join-Path $taskNpmNotices $taskNoticeName
            New-Item -ItemType Directory -Path $taskNoticeFolder -Force | Out-Null
            $taskLicenseFiles | Copy-Item -Destination $taskNoticeFolder -Force
        }
    }
}
$taskMetadataText = & cargo metadata --locked --format-version 1 --filter-platform x86_64-pc-windows-msvc --manifest-path src-tauri/Cargo.toml
if ($LASTEXITCODE -ne 0) { throw 'Could not collect Rust dependency notices.' }
$taskMetadata = $taskMetadataText | ConvertFrom-Json
$taskCargoNotices = Join-Path $taskPortable 'licenses\rust'
New-Item -ItemType Directory -Path $taskCargoNotices | Out-Null
foreach ($taskCrate in $taskMetadata.packages) {
    $taskCratePath = Split-Path -Parent $taskCrate.manifest_path
    $taskNoticeFolder = Join-Path $taskCargoNotices ($taskCrate.name + '-' + $taskCrate.version)
    New-Item -ItemType Directory -Path $taskNoticeFolder -Force | Out-Null
    Get-ChildItem -LiteralPath $taskCratePath -File | Where-Object { $_.Name -match '^(LICENSE|LICENCE|COPYING|NOTICE)' } | Copy-Item -Destination $taskNoticeFolder
    if ($taskCrate.license_file -and (Test-Path -LiteralPath (Join-Path $taskCratePath $taskCrate.license_file))) {
        Copy-Item -LiteralPath (Join-Path $taskCratePath $taskCrate.license_file) -Destination $taskNoticeFolder -Force
    }
}
$taskMetadata.packages | Select-Object name,version,license,source,repository | ConvertTo-Json -Depth 5 | Set-Content "$taskPortable\licenses\rust-dependencies.json" -Encoding utf8

if (-not $SkipWebViewDownload) {
    $taskBootstrap = Join-Path $taskPortable 'prerequisites\Install-WebView2.exe'
    if ($WebViewBootstrapper) {
        Copy-Item -LiteralPath $WebViewBootstrapper -Destination $taskBootstrap
    } else {
        Invoke-WebRequest -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $taskBootstrap
    }
    $taskSignature = Get-AuthenticodeSignature -LiteralPath $taskBootstrap
    if ($taskSignature.Status -ne 'Valid' -or $taskSignature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation') {
        throw 'The WebView2 bootstrapper does not have a valid Microsoft signature.'
    }
}
$taskCommit = & git rev-parse HEAD
if ($LASTEXITCODE -ne 0) { throw 'Cannot identify source commit.' }
@"
ReelCraft Studio $taskVersion $taskBuildLabel Beta
Source base commit: $taskCommit (Source.zip includes the current local working tree.)
FFmpeg $($taskManifest.version), checksum verified against scripts/ffmpeg-windows.json.
FFmpeg source: $($taskManifest.source)
FFmpeg build provider and dependency/build information: $($taskManifest.provider)
Original FFmpeg GPL notices and build configuration are retained in licenses/ffmpeg/ and ffmpeg-version.txt.
ReelCraft Studio source is supplied as a separate Source.zip beside the application ZIP.
The app's MIT license does not replace the licenses of bundled third-party components.
WebView2 Runtime terms: https://developer.microsoft.com/microsoft-edge/webview2/
"@ | Set-Content "$taskPortable\licenses\SOURCE-INFORMATION.txt" -Encoding utf8
$taskZip = Join-Path $taskOutput "$taskArtifactName-Windows-x64.zip"
# Some npm notices use pre-1980 timestamps, which the ZIP format cannot represent.
Get-ChildItem -LiteralPath $taskPortable -Recurse -Force | ForEach-Object {
    if ($_.LastWriteTime.Year -lt 1980 -or $_.LastWriteTime.Year -gt 2107) {
        $_.LastWriteTime = [datetime]'2026-01-01T00:00:00'
    }
}
Compress-Archive -LiteralPath $taskPortable -DestinationPath $taskZip -CompressionLevel Optimal -Force -ErrorAction Stop
if (-not (Test-Path -LiteralPath $taskZip)) { throw 'Windows ZIP was not created.' }
$taskSourceZip = Join-Path $taskOutput "$taskArtifactName-Source.zip"
# Include current changes and new source files, while respecting Git exclusions.
# Archiving HEAD alone would silently ship stale source for a local beta build.
$taskSourceFiles = & git -c core.quotepath=false ls-files --cached --others --exclude-standard
if ($LASTEXITCODE -ne 0) { throw 'Could not enumerate the current source snapshot.' }
Add-Type -AssemblyName System.IO.Compression.FileSystem
$taskSourceTemp = $taskSourceZip + '.' + [guid]::NewGuid().ToString('N') + '.partial'
$taskSourceArchive = [IO.Compression.ZipFile]::Open($taskSourceTemp, [IO.Compression.ZipArchiveMode]::Create)
try {
    foreach ($taskRelativeFile in ($taskSourceFiles | Sort-Object -Unique)) {
        $taskFullFile = [IO.Path]::GetFullPath((Join-Path $taskRoot $taskRelativeFile))
        if (-not $taskFullFile.StartsWith($taskRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Source path leaves the project: $taskRelativeFile"
        }
        if (Test-Path -LiteralPath $taskFullFile -PathType Leaf) {
            [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($taskSourceArchive, $taskFullFile, $taskRelativeFile.Replace('\','/'), [IO.Compression.CompressionLevel]::Optimal) | Out-Null
        }
    }
} finally { $taskSourceArchive.Dispose() }
Move-Item -LiteralPath $taskSourceTemp -Destination $taskSourceZip -Force
$taskHashes = foreach ($taskArtifact in @($taskZip,$taskSourceZip)) {
    $taskHash = Get-FileHash -LiteralPath $taskArtifact -Algorithm SHA256
    "$($taskHash.Hash.ToLowerInvariant())  $([IO.Path]::GetFileName($taskArtifact))"
}
$taskHashes | Set-Content (Join-Path $taskOutput 'SHA256SUMS.txt') -Encoding ascii
Write-Host "Windows download: $taskZip"
Write-Host "Portable app: $taskPortable"
