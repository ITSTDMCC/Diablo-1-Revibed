@echo off
rem One-time setup: finds your Diablo folder and gets devilutionx.mpq. See README.txt.
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup.ps1" %*
exit /b %ERRORLEVEL%
