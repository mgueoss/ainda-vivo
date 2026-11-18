@echo off
cd /d "%~dp0"
if exist "%~dp0windows\ainda-vivo-v0.0.8.exe" (
    start "" "%~dp0windows\ainda-vivo-v0.0.8.exe"
    exit /b
)
where pwsh.exe >nul 2>nul
if %errorlevel% equ 0 (
    pwsh.exe -NoLogo -NoProfile -File "%~dp0scripts\dev.ps1" -Action Run
) else (
    powershell.exe -NoLogo -NoProfile -File "%~dp0scripts\dev.ps1" -Action Run
)
if errorlevel 1 pause
