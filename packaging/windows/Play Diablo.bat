@echo off
rem Starts the game. Put your Diablo game files (DIABDAT.MPQ, and Hellfire's files if you have
rem them) in this folder first; see README.txt.
cd /d "%~dp0"
if not exist "DIABDAT.MPQ" (
    echo DIABDAT.MPQ is not in this folder.
    echo.
    echo Copy DIABDAT.MPQ from your own Diablo installation into:
    echo   %~dp0
    echo then run Play Diablo.bat again. See README.txt.
    echo.
    pause
    exit /b 1
)
start "" "%~dp0diablo1_rs.exe" %*
