$taskStudioRoot = Split-Path -Parent $PSScriptRoot
$taskLocalCargo = Join-Path $taskStudioRoot '.tools\cargo'
if (Test-Path -LiteralPath (Join-Path $taskLocalCargo 'bin\cargo.exe')) {
    $env:CARGO_HOME = $taskLocalCargo
    $env:RUSTUP_HOME = Join-Path $taskStudioRoot '.tools\rustup'
    $env:PATH = (Join-Path $taskLocalCargo 'bin') + ';' + $env:PATH
}
# Discover MSVC even when Visual Studio's installer metadata is incomplete.
$taskCompilerRoots = @(
    'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC',
    'C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\MSVC'
)
foreach ($taskCompilerRoot in $taskCompilerRoots) {
    $taskCompiler = Get-ChildItem -LiteralPath $taskCompilerRoot -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1
    if ($taskCompiler) {
        $env:PATH = (Join-Path $taskCompiler.FullName 'bin\Hostx64\x64') + ';' + $env:PATH
        $env:LIB = (Join-Path $taskCompiler.FullName 'lib\x64')
        $env:INCLUDE = (Join-Path $taskCompiler.FullName 'include')
        break
    }
}
$taskSdkRoot = 'C:\Program Files (x86)\Windows Kits\10'
$taskSdk = Get-ChildItem -LiteralPath (Join-Path $taskSdkRoot 'Lib') -Directory -ErrorAction SilentlyContinue | Sort-Object Name -Descending | Select-Object -First 1
if ($taskSdk) {
    $env:LIB += ';' + (Join-Path $taskSdk.FullName 'um\x64') + ';' + (Join-Path $taskSdk.FullName 'ucrt\x64')
    $env:PATH = (Join-Path $taskSdkRoot "bin\$($taskSdk.Name)\x64") + ';' + $env:PATH
    $env:INCLUDE += ';' + (Join-Path $taskSdkRoot "Include\$($taskSdk.Name)\ucrt") + ';' + (Join-Path $taskSdkRoot "Include\$($taskSdk.Name)\shared") + ';' + (Join-Path $taskSdkRoot "Include\$($taskSdk.Name)\um")
}
