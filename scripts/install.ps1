$ErrorActionPreference = 'Stop'

$Repository = if ($env:DEVPTP_REPO) { $env:DEVPTP_REPO } else { 'SirMomster/devptp' }
$InstallDirectory = if ($env:DEVPTP_INSTALL_DIR) {
    $env:DEVPTP_INSTALL_DIR
} else {
    Join-Path $env:LOCALAPPDATA 'devptp\bin'
}

$Architecture = if ($env:PROCESSOR_ARCHITEW6432) {
    $env:PROCESSOR_ARCHITEW6432
} else {
    $env:PROCESSOR_ARCHITECTURE
}
if ($Architecture -ne 'AMD64') {
    throw "Unsupported Windows architecture: $Architecture. The release supports x86_64 (AMD64)."
}

$Asset = 'devptp-windows-x86_64.zip'
$Url = "https://github.com/$Repository/releases/latest/download/$Asset"
$TemporaryDirectory = Join-Path ([IO.Path]::GetTempPath()) ("devptp-" + [guid]::NewGuid())
$Archive = Join-Path $TemporaryDirectory $Asset

try {
    New-Item -ItemType Directory -Path $TemporaryDirectory -Force | Out-Null
    Write-Host "Downloading devptp for Windows x86_64..."
    Invoke-WebRequest -Uri $Url -OutFile $Archive

    Expand-Archive -Path $Archive -DestinationPath $TemporaryDirectory -Force
    New-Item -ItemType Directory -Path $InstallDirectory -Force | Out-Null
    Copy-Item (Join-Path $TemporaryDirectory 'devptp.exe') (Join-Path $InstallDirectory 'devptp.exe') -Force

    Write-Host "Installed devptp to $(Join-Path $InstallDirectory 'devptp.exe')"
    $UserPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not (($UserPath -split ';') -contains $InstallDirectory)) {
        Write-Host "Add $InstallDirectory to your user PATH to run devptp from any terminal."
    }
} finally {
    if (Test-Path $TemporaryDirectory) {
        Remove-Item $TemporaryDirectory -Recurse -Force
    }
}
