@echo off
rem Starts the game with the original movement. Runs the setup first if needed.
cd /d "%~dp0"
if not exist "data-dir.txt" (
    call "%~dp0Setup.bat"
    if errorlevel 1 exit /b 1
)
set /p DATADIR=<"data-dir.txt"
start "" "%~dp0diablo1_rs.exe" --data-dir "%DATADIR%" %*
