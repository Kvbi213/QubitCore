@echo off
setlocal
cd /d "%~dp0"
echo ======================================================================
echo          QUBIT ECOSYSTEM -- INSTALATOR WINDOWS 11 / 10                
echo ======================================================================
echo Uruchamianie procedury instalacyjnej w srodowisku PowerShell...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"
if %ERRORLEVEL% equ 0 (
    echo.
    echo ======================================================================
    echo [+] Sukces: Ekosystem QubitCore zostal pomyslnie zainstalowany!
    echo ======================================================================
) else (
    echo.
    echo [-] Blad podczas instalacji (kod %ERRORLEVEL%).
)
pause
