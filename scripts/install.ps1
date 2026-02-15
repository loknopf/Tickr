param(
    [string]$Repo = "loknopf/Tickr",
    [string]$Version = "latest",
    [string]$BinDir = "$env:LOCALAPPDATA\Tickr\bin",
    [switch]$AddToPath = $true
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

if ($Version -ne "latest") {
    $Version = $Version.TrimStart("v")
}

$apiUrl = if ($Version -eq "latest") {
    "https://api.github.com/repos/$Repo/releases/latest"
} else {
    "https://api.github.com/repos/$Repo/releases/tags/v$Version"
}

$release = Invoke-RestMethod -Uri $apiUrl -Headers @{ "Accept" = "application/vnd.github+json" }
$tag = $release.tag_name
$ver = $tag.TrimStart("v")
$assetName = "tickr-$ver-x86_64-pc-windows-msvc.zip"

$asset = $release.assets | Where-Object { $_.name -eq $assetName } | Select-Object -First 1
if (-not $asset) {
    throw "Asset not found: $assetName"
}

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

$tmpDir = Join-Path $env:TEMP ("tickr-" + [Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null

try {
    $zipPath = Join-Path $tmpDir $assetName
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $zipPath
    Expand-Archive -Path $zipPath -DestinationPath $tmpDir -Force

    $exePath = Join-Path $tmpDir "tickr.exe"
    if (-not (Test-Path $exePath)) {
        throw "tickr.exe not found in archive."
    }

    Copy-Item -Path $exePath -Destination (Join-Path $BinDir "tickr.exe") -Force
} finally {
    Remove-Item -Recurse -Force $tmpDir
}

if ($AddToPath) {
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($userPath -notlike "*$BinDir*") {
        $newPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $BinDir } else { "$userPath;$BinDir" }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    }
}

Write-Host "Installed tickr to $BinDir"
if ($AddToPath) {
    Write-Host "PATH updated for current user. Restart your shell."
} else {
    Write-Host "Add $BinDir to PATH or re-run with -AddToPath."
}
