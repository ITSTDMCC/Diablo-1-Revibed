diablo1_rs - a Rust port of Diablo (1996) for Windows 10/11 (64-bit)
=====================================================================

This zip contains NO game data. You need your own copy of Diablo: GOG sells it as
"Diablo + Hellfire" (https://www.gog.com/en/game/diablo). Install it first.

Getting started
---------------
1. Unzip this folder anywhere (for example C:\Games\diablo1_rs). Not inside "Program Files".
2. Double-click "Setup.bat". It
     - finds your Diablo folder (the one with DIABDAT.MPQ) or asks you to pick it, and
     - downloads devilutionx.mpq (DevilutionX's fonts and interface pieces, about 10 MB)
       from the official DevilutionX 1.5.3 release and checks it.
   You only do this once. (Windows may warn about running a downloaded file: choose
   "More info" > "Run anyway".)
3. Play: double-click "Play Diablo.bat".
     - F11 switches between full screen and a window.
     - Settings > Gameplay > Free Movement: move in any direction like a modern action RPG;
       with it on, X switches to an experimental first-person view.
     - Only DIABDAT.MPQ is needed. Hellfire is optional: with all its files (hellfire.mpq,
       hfmonk.mpq, hfmusic.mpq, hfvoice.mpq) the game asks once whether to play Diablo or
       Hellfire (Settings > Start Up > Game Mode); otherwise it plays Diablo.

Alternatively copy DIABDAT.MPQ into this folder and run diablo1_rs.exe directly.

Controls, settings, saves and known differences: see the README on
https://github.com/ITSTDMCC/Diablo-1-Revibed

Settings (diablo.ini) and saves are kept in this folder.

If something goes wrong
-----------------------
- "DIABDAT.MPQ was not found": install Diablo, then run Setup.bat again, or copy DIABDAT.MPQ
  from your Diablo folder into this folder.
- No download possible: get devilutionx-windows-x86_64.zip from
  https://github.com/diasurgical/DevilutionX/releases/tag/1.5.3 and copy devilutionx.mpq from
  it into this folder.
- To pick a different Diablo folder, delete data-dir.txt and run Setup.bat again.

Licence
-------
This is a modified version (a translation into Rust) of DevilutionX, distributed under the
Sustainable Use License: free, non-commercial use only. See LICENSE-DevilutionX.md.
Not affiliated with Blizzard Entertainment, GOG or the DevilutionX team. Built with AI
(Anthropic's Claude) and checked with automated tests against DevilutionX.
