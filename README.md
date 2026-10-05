# diablo1_rs

A Rust port of **Diablo** (1996), translated function by function from the
[DevilutionX 1.5.3](https://github.com/diasurgical/devilutionX/releases/tag/1.5.3) source code,
with [Bevy](https://bevyengine.org/) 0.19 for the window and input.

> **Please read first**
>
> - This port was **built with AI** (Anthropic's Claude, working as a coding agent) and checked
>   with automated tests against DevilutionX. Expect rough edges; see
>   [Known differences](#known-differences-from-the-original).
> - It contains **no game data**. To play you must **own Diablo**: it is sold by GOG as
>   [Diablo + Hellfire](https://www.gog.com/en/game/diablo). You need the `DIABDAT.MPQ` file from
>   your copy.
> - It is a modified version (a translation into Rust) of DevilutionX, which is distributed under
>   the Sustainable Use License: **free, non-commercial use only**. A copy of those terms is in
>   [LICENSE-DevilutionX.md](LICENSE-DevilutionX.md). Not affiliated with Blizzard Entertainment,
>   GOG or the DevilutionX team.

## Requirements

Software:

- Windows 10 or 11, 64-bit (the only system it was built and tested on).
- A graphics driver with DirectX 12 or Vulkan (anything from the last several years).
- To build it: [Rust](https://rustup.rs) (stable; tested with 1.99.0) and the Microsoft C++
  Build Tools that the Rust installer offers to install.
- Your copy of Diablo (`DIABDAT.MPQ`), and `devilutionx.mpq` from the DevilutionX 1.5.3 release
  (fonts and interface pieces DevilutionX adds; the port uses the same file).

Hardware: any PC that runs Windows 10 well is enough; the game draws a 640x480 picture. Tested on
an AMD Ryzen 5 5600X with a Radeon RX 9060 XT and 16 GB of RAM, where it never came close to
loading the machine (an observation, not a measurement). The built program is about 50 MB; a
build needs about 3 GB of disk for the compiler's work files.

## Setup, from a fresh Windows install

1. **Install your copy of Diablo.** In GOG Galaxy (or the GOG offline installer) install
   *Diablo + Hellfire*. Find its folder (by default `C:\Program Files (x86)\GOG Galaxy\Games\Diablo`)
   and check that it contains `DIABDAT.MPQ`. Hellfire's `hellfire.mpq` and friends are optional.
2. **Get `devilutionx.mpq`.** Download `devilutionx-windows-x86_64.zip` from the
   [DevilutionX 1.5.3 release page](https://github.com/diasurgical/devilutionX/releases/tag/1.5.3),
   open the zip and copy `devilutionx.mpq` (only that file) into your Diablo folder next to
   `DIABDAT.MPQ`. Use the 1.5.3 file: other versions or modded copies may not match.
3. **Install Rust.** Run the installer from <https://rustup.rs> and accept the defaults. When it
   offers to install the Visual Studio C++ Build Tools, say yes. Restart the terminal afterwards.
4. **Get the port's source.** Clone or download this repository, for example to
   `C:\Games\diablo1_rs`.
5. **Build it.** Open *PowerShell* in that folder and run:

   ```
   cargo build --release
   ```

   The first build downloads the libraries it needs and takes several minutes. The program is
   then `target\release\diablo1_rs.exe`.
6. **Play.** Point the port at your Diablo folder:

   ```
   .\target\release\diablo1_rs.exe --data-dir "C:\Program Files (x86)\GOG Galaxy\Games\Diablo"
   ```

   Or copy `diablo1_rs.exe` into the Diablo folder and double-click it there. A shortcut with the
   command above works too. Add `-n` to skip the intro movies.

**Game controllers** (Xbox, PlayStation, Switch Pro and others) work out of the box: plug one in
and press a button. (`cargo build --release --no-default-features` builds without controller
support.)

### Free-movement build (optional)

A second build of the game where your hero moves in **any direction**, like a modern action RPG,
instead of stepping from tile to tile in eight directions. Everything else is the original game.
Build it into its own folder so it sits next to the normal build:

```
cargo build --release --features free-movement --target-dir target\free-movement
```

and start `target\free-movement\release\diablo1_rs.exe` with the same `--data-dir` as above.

- **Mouse:** click the ground to walk straight to that exact spot (around walls when needed), or
  hold the button and the hero follows the cursor. Clicking a monster, item, door or chest walks
  up to it and then attacks, picks it up or opens it, as usual.
- **Controller:** the left stick walks in the stick's exact direction.
- The hero slides along walls instead of stopping. Walking speed is close to the original's.
- It applies to single player. Multiplayer games use the original movement, so they stay
  compatible with the normal build and with DevilutionX.
- It shares the normal build's settings and heroes (saves work in both).

**First-person view (experimental, free-movement build only).** Press **X** to switch the dungeon
view to the hero's eyes and back.

- **W / S** walk forward and back, **A / D** turn, **Shift + A / D** step sideways.
- **Controller:** the left stick walks (up is forward, sideways steps sideways), the right stick
  turns. The hero faces the way the view looks, so the usual automatic targeting picks what is
  in front.
- The mouse still aims: whatever is under (or right next to) the cursor in the first-person view
  is what a click attacks, picks up, opens or talks to, and where a spell is cast. Clicking the floor walks there.
- The control panel moves to the top of the screen while the view is on, and while you attack or cast
  your hero shows faintly at the bottom of the view, so you see the swing.
- Panels, inventory, belt, spells and menus work as usual. While the view is on, **S** walks back
  instead of opening the speedbook.
- The walls, floors, monsters and items are the game's own isometric art, re-projected, so it is
  rough: arches look solid below their top, town houses and trees look flat, and close-up sprites
  are blocky.

### HD art

Not available yet. The port shows the original 640x480 art (scaled to the window).

## Controls

Keyboard and mouse are those of DevilutionX 1.5.3 (all can be changed in *Settings > Keymapping*):

| Key | Action |
|---|---|
| Left click | Walk, attack, talk, pick up, use |
| Shift + left click | Attack without moving |
| Right click | Cast the current spell |
| I / C / Q / B | Inventory / Character / Quests / Spellbook |
| S | Speedbook (choose the right-click spell) |
| F5 - F8 | Quick spells (hover a spell in the Speedbook and press the key to assign it) |
| 1 - 8 | Use belt item |
| Tab | Automap (with + / - to zoom it) |
| Alt (hold) / Right Ctrl | Show items on the ground / keep showing them |
| Space | Close all panels |
| Z | Zoom |
| P or Pause | Pause |
| F / G | Brighter / darker |
| F1 | Help |
| F2 / F3 | Quick save / quick load (single player) |
| F9 - F12 | Quick chat messages (multiplayer); Enter opens chat |
| V / L | Game info / chat log |
| Print Screen | Screenshot |
| Esc | Game menu (save, options, quit) |

Game controller defaults (*Settings > Padmapping* to change): left stick or d-pad
to walk, right stick moves the cursor, **B** primary action (attack, talk, pick up and place in
the inventory), **Y** secondary action (open, pick up), **X** cast spell, **A** speedbook / back,
**LB / RB** health / mana potion, **LT / RT** character / inventory, **Back + LT / RT** quests /
spellbook, **Start** held for the menu shortcuts, **Back + Start** game menu.

Multiplayer: *Multi Player > Client-Server (TCP)*. One player chooses *Create Game* (or *Create
Public Game* for no password), the others *Join Game* and type the host's IP address (port 6112;
the host may need to allow it through the Windows firewall or forward it on the router). Games are
compatible with DevilutionX 1.5.3: the port and DevilutionX players can play together.

## Where settings and saves go

`%APPDATA%\diablo1_rs\devilution\` (paste that into the Explorer address bar):

- `diablo.ini`: settings,
- `single_0.sv` ... and `multi_0.sv` ...: heroes and saved games,
- screenshots (Print Screen).

It is a separate folder from DevilutionX's, so the port never touches your DevilutionX heroes. To
use other folders run with `--save-dir <folder>` and `--config-dir <folder>`. `--help` lists the
other options.

## Known differences from the original

The full list, with the technical details, is in [port/NOTES.md](port/NOTES.md). The ones a
player may notice:

- Settings and saves live in their own folder (above). Saves are the same format as DevilutionX's
  and can be copied between them.
- The game is not found automatically in the GOG install folder: use `--data-dir` or put the
  program next to `DIABDAT.MPQ`.
- Multiplayer offers *Client-Server (TCP)* and *Offline*; DevilutionX's ZeroTier internet
  option is not available.
- Touch controls are not supported.
- The optional free-movement build (above) changes how the hero walks; the normal build moves
  exactly like the original.
- Sound is mixed by the port's own simple mixer; it may sound very slightly different. MP3 music
  replacements are not supported.
- Pasting from the clipboard into text fields does not work.
- HD art does not exist yet.

## Building with an AI agent

You can have a coding agent (Claude Code, for example) do the build for you. Open the agent in
this folder and give it a prompt like this one, which asks it to build only:

> Build this Rust project for me on Windows. Read README.md first. Check that Rust is installed
> (`cargo --version`); if it is not, stop and tell me to install it from rustup.rs. Then run
> `cargo build --release` and
> fix nothing in the code: if the build fails, show me the error. When it succeeds, tell me the
> path of `diablo1_rs.exe` and the command to start it with my Diablo folder
> (`--data-dir "<my Diablo folder>"`). **Do not launch the game and do not run any tests or test
> runs**; I will start the game myself.

## For developers

- `port/NOTES.md`: how the port is organised, the conventions (every function is tagged with
  the DevilutionX function it ports), what was replaced instead of translated, and the evidence
  that it matches DevilutionX.
- `cargo test --release`: parity tests. Tests that need game data read `DIABLO_DATA_DIR` (a folder
  with `DIABDAT.MPQ` and `devilutionx.mpq`); DevilutionX's test fixtures are read from
  `DEVILUTIONX_SOURCE` (default `../Decomp/source_1.5.3`).
- `port/manifest.csv`: every DevilutionX function and whether it is ported or replaced (and by
  what). `python tools/mark.py ported` updates it from the tags.

Test hooks (environment variables, for automated runs only):

| Variable | Effect |
|---|---|
| `DIABLO_HEADLESS=1` | No window; frames are still drawn (for screenshots) |
| `DIABLO_FIXED_STEP=1` | Virtual clock: time moves only when the game waits, so runs replay exactly |
| `DIABLO_FIXED_STEP=pace` | As `1`, but waits for real too (two games talking over the network) |
| `DIABLO_INPUT_SCRIPT=<file>` | Scripted input, one `<frame> <command>` per line: `key <SDL keycode>`, `click <x> <y>`, `move <x> <y>`, `text <string>`, `quit`, `warp <level>`, `setwarp <quest level> <type>`, `store <TalkID>`, `killdiablo`, and a virtual game controller: `padadd <USB vendor>`, `pad <button>`, `paddown`/`padup <button>`, `padaxis <axis> <value>` |
| `DIABLO_SCREENSHOT_FRAMES=n,m,...` | Save `frame_<n>.png` at those frames |
| `DIABLO_SCREENSHOT_DIR=<folder>` | Where the screenshots go |
| `DIABLO_MAX_FRAMES=n` | Quit after n frames |
| `DIABLO_TIME=<seconds>` | Fixed start time for `time()` |
| `DIABLO_NO_AUDIO=1`, `DIABLO_NO_SAVE=1` | No sound device / no writing saves |
| `DIABLO_DEMO_TRACE=1` | Print the player's state on every demo tick |
| `DIABLO_VERBOSE=1`, `RUST_BACKTRACE=1` | Verbose log / backtrace on a fatal error |

Ready-made input scripts are in `tools/input_scripts/` (new game, ending, save and load, gamepad,
two-player TCP). `tools/tcp_smoke.py` runs a two-player game; `tools/interop_original.ps1` plays a
TCP game between the port and the original DevilutionX 1.5.3. Screenshots they produce show game
art: keep them out of the repository.
