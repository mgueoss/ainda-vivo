@echo off
cd /d "%~dp0"
if exist "%~dp0windows\ainda-vivo-v0.0.6.exe" (
    start "" "%~dp0windows\ainda-vivo-v0.0.6.exe"
    exit /b
)
if exist "%~dp0windows\ainda-vivo-v0.0.5.exe" (
    start "" "%~dp0windows\ainda-vivo-v0.0.5.exe"
    exit /b
)
if exist "%~dp0windows\ainda-vivo-v0.0.4.exe" (
    start "" "%~dp0windows\ainda-vivo-v0.0.4.exe"
    exit /b
)
if exist "%~dp0windows\ainda-vivo-v0.0.3.exe" (
    start "" "%~dp0windows\ainda-vivo-v0.0.3.exe"
    exit /b
)
if exist "%~dp0windows\ainda-vivo-v0.0.2.exe" (
    start "" "%~dp0windows\ainda-vivo-v0.0.2.exe"
    exit /b
)
if exist "%~dp0windows\ainda-vivo.exe" (
    start "" "%~dp0windows\ainda-vivo.exe"
    exit /b
)
where pwsh.exe >nul 2>nul
if %errorlevel% equ 0 (
    pwsh.exe -NoLogo -NoProfile -File "%~dp0scripts\dev.ps1" -Action Run
) else (
    powershell.exe -NoLogo -NoProfile -File "%~dp0scripts\dev.ps1" -Action Run
)
if errorlevel 1 pause
