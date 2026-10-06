Diablo-1-Revibed - a Rust port of Diablo (1996) for Windows 10/11 (64-bit)
==========================================================================

This zip contains NO Diablo game data. You need your own copy of Diablo: GOG sells it as
"Diablo + Hellfire" (https://www.gog.com/en/game/diablo).

Getting started
---------------
1. Unzip this folder anywhere (for example C:\Games\Diablo-1-Revibed). Not inside
   "Program Files".
2. Copy your Diablo game files into this folder, next to diablo1_rs.exe:
     - DIABDAT.MPQ                                   (required)
     - hellfire.mpq, hfmonk.mpq, hfmusic.mpq,
       hfvoice.mpq                                   (optional, for Hellfire)
   You find them in your Diablo installation folder, by default
   C:\Program Files (x86)\GOG Galaxy\Games\Diablo
3. Double-click "Play Diablo.bat". (Windows may warn about running a downloaded file: choose
   "More info" > "Run anyway".)

Good to know
------------
- F11 switches between full screen and a window.
- Settings > Gameplay > Diablo 2 Free Movement (first line): move in any direction like a modern
  action RPG; with it on, X switches to an experimental first-person view.
- With all four Hellfire files the game asks once whether to play Diablo or Hellfire
  (change it later in Settings > Start Up > Game Mode). Without them, or with only some of
  them, it plays Diablo.
- Settings (diablo.ini) and saves are kept in this folder.
- devilutionx.mpq (included) holds DevilutionX's own fonts and interface pieces, from the
  official DevilutionX 1.5.3 release. It is not Blizzard game data.

Controls and known differences: see https://github.com/ITSTDMCC/Diablo-1-Revibed

Licence
-------
This is a modified version (a translation into Rust) of DevilutionX, distributed under the
Sustainable Use License: free, non-commercial use only. See LICENSE-DevilutionX.md.
Not affiliated with Blizzard Entertainment, GOG or the DevilutionX team. Built with AI
(Anthropic's Claude) and checked with automated tests against DevilutionX.
