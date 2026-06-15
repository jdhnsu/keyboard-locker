# package-portable.ps1
# Compile portable version and package as zip
# Usage: Run from Windows PowerShell: .\package-portable.ps1

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path

Write-Host '=== 1/4 Install npm dependencies ===' -ForegroundColor Cyan
npm install

Write-Host '=== 2/4 Build frontend (Vue + Vite) ===' -ForegroundColor Cyan
npm run build

Write-Host '=== 3/4 Build Tauri app (Rust release) ===' -ForegroundColor Cyan
npx tauri build

$releaseDir = "$projectRoot\src-tauri\target\release"
$exeName = 'keyboard-locker.exe'
$publishDir = "$projectRoot\publish\KeyboardLocker-Portable"

if (-not (Test-Path "$releaseDir\$exeName")) {
    Write-Error "Cannot find $releaseDir\$exeName, build may have failed"
    exit 1
}

Write-Host '=== 4/4 Package portable zip ===' -ForegroundColor Cyan
Remove-Item -Recurse -Force $publishDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $publishDir | Out-Null

Copy-Item "$releaseDir\$exeName" $publishDir
Copy-Item "$projectRoot\README.md" $publishDir

$zipPath = "$projectRoot\publish\KeyboardLocker-Portable-v0.1.0.zip"
Remove-Item $zipPath -ErrorAction SilentlyContinue

$items = Get-ChildItem -Path $publishDir
Compress-Archive -LiteralPath $items.FullName -DestinationPath $zipPath

Write-Host ''
Write-Host '=== Done! ==='  -ForegroundColor Green
Write-Host "Portable zip: $zipPath" -ForegroundColor Yellow
Write-Host "Size: $((Get-Item $zipPath).Length / 1KB) KB" -ForegroundColor Yellow
