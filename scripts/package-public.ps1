$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path -Parent $PSScriptRoot
$taskVersion = (Get-Content -LiteralPath (Join-Path $taskRoot 'package.json') -Raw | ConvertFrom-Json).version
$taskRelease = Join-Path $taskRoot "Production\ReelCraftStudio-$taskVersion-2026-10-30"
$taskPortable = Join-Path $taskRelease 'ReelCraftStudio'
if (Test-Path -LiteralPath $taskRelease) { throw "Release already exists: $taskRelease. Use a new release directory." }
$taskExecutable = Join-Path $taskRoot 'src-tauri\target\release\long-to-shot-studio.exe'
if (-not (Test-Path -LiteralPath $taskExecutable)) { throw 'Run BUILD-WINDOWS.bat successfully before packaging.' }
New-Item -ItemType Directory -Path "$taskPortable\bin", "$taskPortable\licenses" -Force | Out-Null
Copy-Item -LiteralPath $taskExecutable -Destination "$taskPortable\ReelCraftStudio.exe"
foreach ($taskTool in @('ffmpeg', 'ffprobe')) {
    Copy-Item -LiteralPath "$taskRoot\Copy\bin\$taskTool.exe" -Destination "$taskPortable\bin\$taskTool.exe"
}
Copy-Item -Path "$taskRoot\Copy\licenses\*" -Destination "$taskPortable\licenses"
Copy-Item -LiteralPath "$taskRoot\docs\PUBLIC-SHARE-GUIDE.md" -Destination "$taskPortable\READ-ME.md"
Copy-Item -LiteralPath "$taskRoot\docs\PUBLIC-SHARE-GUIDE.md" -Destination "$taskPortable\READ-ME.txt"
Copy-Item -LiteralPath "$taskRoot\docs\PUBLIC-SHARE-GUIDE.md" -Destination "$taskRelease\PUBLIC-SHARE-GUIDE.md"
Copy-Item -LiteralPath "$taskRoot\docs\RELEASE-0.1.0-2026-10-30.md" -Destination "$taskRelease\RELEASE-NOTES.md"

# Explicit allowlist keeps personal data, caches, local credentials and build tools out.
$taskSource = Join-Path $taskRelease 'Source'
New-Item -ItemType Directory -Path "$taskSource\src-tauri", "$taskSource\scripts", "$taskSource\docs" -Force | Out-Null
foreach ($taskDirectory in @('src', 'public', 'tests', 'branding')) {
    Copy-Item -LiteralPath "$taskRoot\$taskDirectory" -Destination $taskSource -Recurse
}
foreach ($taskDirectory in @('src', 'icons', 'capabilities')) {
    Copy-Item -LiteralPath "$taskRoot\src-tauri\$taskDirectory" -Destination "$taskSource\src-tauri" -Recurse
}
foreach ($taskFile in @('Cargo.toml', 'Cargo.lock', 'build.rs', 'tauri.conf.json')) {
    Copy-Item -LiteralPath "$taskRoot\src-tauri\$taskFile" -Destination "$taskSource\src-tauri"
}
foreach ($taskFile in @('package.json', 'package-lock.json', 'tsconfig.json', 'vite.config.ts', 'playwright.config.ts', 'index.html', 'README.md', 'README.en.md', 'BUILD-WINDOWS.bat', 'BUILD-LINUX.sh', 'BUILD-MAC.sh', '.gitignore', '.gitattributes', 'LICENSE')) {
    Copy-Item -LiteralPath "$taskRoot\$taskFile" -Destination $taskSource
}
foreach ($taskScript in @('env.ps1', 'build-final-windows.ps1', 'build-final-unix.sh')) { Copy-Item -LiteralPath "$taskRoot\scripts\$taskScript" -Destination "$taskSource\scripts" }
Copy-Item -LiteralPath "$taskRoot\docs\PUBLIC-SHARE-GUIDE.md" -Destination "$taskSource\docs"
# Preserve notices for direct frontend dependencies in addition to bundled media tools.
foreach ($taskDependency in @('react', 'react-dom', 'lucide-react', '@tauri-apps/api', '@tauri-apps/plugin-dialog')) {
    $taskDependencyPath = Join-Path "$taskRoot\node_modules" $taskDependency
    $taskLicenseName = $taskDependency.Replace('/', '-').Replace('@', '')
    Get-ChildItem -LiteralPath $taskDependencyPath -File | Where-Object { $_.Name -match '^LICENSE' } | ForEach-Object {
        Copy-Item -LiteralPath $_.FullName -Destination "$taskPortable\licenses\$taskLicenseName-$($_.Name).txt"
    }
}
Copy-Item -LiteralPath "$taskPortable\licenses" -Destination $taskSource -Recurse
$taskPortableZip = Join-Path $taskRelease "ReelCraftStudio-$taskVersion-Windows-x64.zip"
$taskSourceZip = Join-Path $taskRelease "ReelCraftStudio-$taskVersion-Source.zip"
Compress-Archive -LiteralPath $taskPortable -DestinationPath $taskPortableZip -CompressionLevel Optimal
Compress-Archive -LiteralPath $taskSource -DestinationPath $taskSourceZip -CompressionLevel Optimal
$taskHashes = foreach ($taskArtifact in @($taskPortableZip, $taskSourceZip)) {
    $taskHash = Get-FileHash -LiteralPath $taskArtifact -Algorithm SHA256
    "$($taskHash.Hash.ToLowerInvariant())  $([IO.Path]::GetFileName($taskArtifact))"
}
$taskHashes | Set-Content -LiteralPath "$taskRelease\SHA256SUMS.txt" -Encoding ASCII
Write-Output "Packaged release: $taskRelease"
