@echo off
rem Starts the free-movement build (move in any direction; X switches to the first-person view).
rem Runs the setup first if needed.
cd /d "%~dp0"
if not exist "data-dir.txt" (
    call "%~dp0Setup.bat"
    if errorlevel 1 exit /b 1
)
set /p DATADIR=<"data-dir.txt"
start "" "%~dp0diablo1_rs-free-movement.exe" --data-dir "%DATADIR%" %*
