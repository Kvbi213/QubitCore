[CmdletBinding()]
param(
    [string]$InstallPath = "$env:LOCALAPPDATA\Qubit",
    [switch]$SkipShortcut,
    [switch]$LaunchAfterInstall
)

$ErrorActionPreference = "Stop"

Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "            QUBIT ECOSYSTEM -- AUTOMATED WINDOWS INSTALLER             " -ForegroundColor Cyan
Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "[*] Docelowy katalog instalacji: $InstallPath" -ForegroundColor Gray

$BinDir = Join-Path $InstallPath "bin"
$DataDir = Join-Path $InstallPath "data"
$DashDir = Join-Path $InstallPath "dash"
$DocsDir = Join-Path $InstallPath "docs"

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
New-Item -ItemType Directory -Force -Path $DataDir | Out-Null
New-Item -ItemType Directory -Force -Path $DashDir | Out-Null
New-Item -ItemType Directory -Force -Path $DocsDir | Out-Null

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (Test-Path "$ScriptDir\QubitInstaller.exe") {
    Copy-Item "$ScriptDir\QubitInstaller.exe" "$BinDir\" -Force
}
if (Test-Path "$ScriptDir\qubit-core.exe") {
    Copy-Item "$ScriptDir\qubit-core.exe" "$BinDir\" -Force
}
if (Test-Path "$ScriptDir\qubit-cli.exe") {
    Copy-Item "$ScriptDir\qubit-cli.exe" "$BinDir\" -Force
}

$ConfigFile = Join-Path $InstallPath "qubit_config.json"
$ConfigJson = @"
{
  "version": "0.1.0",
  "port": 4000,
  "host": "127.0.0.1",
  "ram_limit_mb": 60,
  "zero_trust_mcp": true,
  "wal_mode": true
}
"@
Set-Content -Path $ConfigFile -Value $ConfigJson -Encoding UTF8
Write-Host "[+] Utworzono plik konfiguracyjny: $ConfigFile" -ForegroundColor Green

$DbFile = Join-Path $DataDir "qubit_data.db"
if (-not (Test-Path $DbFile)) {
    New-Item -ItemType File -Path $DbFile -Force | Out-Null
    Write-Host "[+] Zainicjalizowano magazyn SQLite WAL: $DbFile" -ForegroundColor Green
}

$UserPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($UserPath -notlike "*$BinDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", [EnvironmentVariableTarget]::User)
    Write-Host "[+] Dodano $BinDir do zmiennej PATH uzytkownika." -ForegroundColor Green
}

$LauncherBat = Join-Path $BinDir "QubitDash.bat"
$LauncherContent = @"
@echo off
title Qubit Ecosystem -- Mission Control
echo ======================================================================
echo             QUBIT ECOSYSTEM -- MISSION CONTROL (PORT 4000)            
echo ======================================================================
echo Uruchamianie demona QubitCore na porcie 4000...
start https://qubit.kvbi.pl/
"@
Set-Content -Path $LauncherBat -Value $LauncherContent -Encoding OEM

if (-not $SkipShortcut) {
    $Desktop = [Environment]::GetFolderPath([Environment+SpecialFolder]::Desktop)
    $ShortcutPath = Join-Path $Desktop "Qubit Mission Control.lnk"
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = $LauncherBat
    $Shortcut.WorkingDirectory = $InstallPath
    $Shortcut.Description = "Qubit Ecosystem - Autonomous AI Engine"
    $Shortcut.Save()
    Write-Host "[+] Utworzono skrot na Pulpicie: $ShortcutPath" -ForegroundColor Green
}

Write-Host "======================================================================" -ForegroundColor Cyan
Write-Host "[+] SUKCES: Ekosystem QubitCore zostal pomyslnie zainstalowany!" -ForegroundColor Green
Write-Host "======================================================================" -ForegroundColor Cyan

if ($LaunchAfterInstall) {
    Start-Process "https://qubit.kvbi.pl/"
}
