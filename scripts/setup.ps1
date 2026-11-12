# Builds a TopCalc installer. Installs Node.js and Rust first when they are missing.
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

function Refresh-Path {
    $machine = [System.Environment]::GetEnvironmentVariable("Path", "Machine")
    $user = [System.Environment]::GetEnvironmentVariable("Path", "User")
    $env:Path = "$machine;$user"
    $extra = @(
        (Join-Path $env:USERPROFILE ".cargo\bin"),
        "C:\Program Files\nodejs"
    )
    foreach ($dir in $extra) {
        if (Test-Path $dir) { $env:Path = "$dir;$env:Path" }
    }
}

function Ensure-Tool($command, $wingetId, $label) {
    if (Get-Command $command -ErrorAction SilentlyContinue) { return }
    if (-not (Get-Command winget -ErrorAction SilentlyContinue)) {
        throw "$label is not installed, and winget is not available. Install $label, open a new PowerShell window, and run this script again."
    }
    Write-Host "Installing $label..."
    winget install --id $wingetId -e --accept-package-agreements --accept-source-agreements
    Refresh-Path
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "$label is installed. Open a new PowerShell window in this folder and run scripts\setup.ps1 again."
    }
}

Refresh-Path
Ensure-Tool node OpenJS.NodeJS.LTS "Node.js"
Ensure-Tool cargo Rustlang.Rustup "Rust"

Write-Host "Installing project packages..."
npm install
Write-Host "Building TopCalc. The first build can take several minutes."
npm run tauri build

$bundle = Join-Path $root "src-tauri\target\release\bundle"
Write-Host ""
Write-Host "Done. Double-click an installer in:"
Write-Host $bundle
Get-ChildItem -Recurse $bundle -Include *.exe, *.msi -ErrorAction SilentlyContinue |
    ForEach-Object { Write-Host $_.FullName }
