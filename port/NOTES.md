# Port notes (read first)

## What this is ported from

- **Source of truth:** DevilutionX 1.5.3 (`working\Decomp\source_1.5.3\Source`, extracted from tag
  `1.5.3` = commit `ba5a9d6`), the code the shipped `devilutionx.exe` was built from. Chosen by the
  owner on 2026-10-05 over porting from the decompiled exe.
- **Reference:** the Ghidra export of `devilutionx.exe` (`working\Decomp\decompiled`, `working\Decomp\disasm`),
  used to confirm behaviour and the build configuration.
- **Database:** `working\portdb\diablo1.sqlite` (see `working\portdb\README.md`). Query it instead of
  grepping the source; it knows which code is compiled into the Windows build.
- "Faithful" means faithful to DevilutionX 1.5.3 as shipped for Windows, including its fixes and options.
  It is not retail Diablo 1.09 behaviour.

## Conventions

- Every ported function has a tag line right above it:
  `// @port <manifest key> sha=<first 12 hex of port_functions.sha256>`
  The key is `path|qualname(params)` from `port/db_functions.csv`. `cargo test` fails if a ported
  function has no tag, a tag has no ported row, or the source hash changed.
- Add a doc comment with the original name, file:line and binary address when one is known
  (`port/db_functions.csv` column `bin_address`), e.g. ``/// Original: `devilution::GenerateRnd` (engine/random.cpp:..., 0x140...)``.
- Code paths that reach unported code call `unported!(key, addr)`, which panics with the function
  key and address. Never stub silently.
- Manifest statuses: `pending`, `ported`, `replaced` (with `replaced_by` and `reason`). Update
  `port/manifest.csv` by hand when porting; `tools/gen_manifest.py` keeps existing statuses and adds
  new database functions as `pending`.
- Deliberate deviations from the original go in "Known differences" below and are reported to the owner.

## Replaced, not translated

Rules in `port/replace_rules.csv` (applied when a function first enters the manifest):

- Other platforms (3DS, Switch, Vita, Android, touch controls): not targets.
- Discord Rich Presence and ZeroTier online multiplayer: reach outside the game; off by default (log only).
- NOSOUND stubs: not part of the shipped build.
- C++ std/compat helpers (`static_vector`, `str_cat`, `str_split`, `string_or_view`, `enum_traits`,
  `endian`, `utf8`, logging): Rust std.
- SDL wrappers (`utils/sdl_*`, SDL1/SDL2 backports): the engine layer.

Third-party libraries the game depends on are reimplemented from their format rules, not linked:
MPQ archives (libmpq in DevilutionX), PKWARE DCL implode/explode (save files and MPQ sectors),
Smacker video (libsmackerdec), SHA-1 (save password hash, `sha.cpp` is game code), and the
libsodium functions behind network packet encryption (Argon2id, XSalsa20-Poly1305, in
`src/dvlnet/crypto.rs`, checked against the RFC and NaCl test vectors). asio's sockets are
`std::net` polled without blocking. Each needs an ask before adding a crate instead.

Game controllers use Bevy's own gilrs backend, behind the `gamepad` cargo feature (on by default,
approved by the owner 2026-10-05; `--no-default-features` leaves it out).

After the first pass every remaining manifest row was classified with `tools/classify.py` and
`port/classify_rules.csv` (regex -> status, replaced_by, reason). Ports that exist under another
name were tagged with `tools/auto_tag.py` (snake_case name match) and `tools/manual_tags.py`
(`port/manual_tags.csv`, hand-checked pairs).

## Data tables

Large constant tables (`itemdat.cpp`, `monstdat.cpp`, `misdat.cpp`, `objdat.cpp`, `spelldat.cpp`,
`textdat.cpp`, `levels/*` room templates, ...) live in `gap` segments, not functions, so the
function manifest does not list them. Port them verbatim alongside the code that uses them; they are
covered by the file-level parity in the database.

## Known database gaps

See `working\portdb\README.md` "Known gaps". In short: name-based call resolution, no source struct
offsets (serialisation is explicit, so not needed), headers not followed for macro values, and most
binary addresses unnamed.

## Timing, assets, persistence (to confirm while porting)

- Game logic runs at 20 updates/s (`gnTickDelay = 50` ms in DevilutionX; confirm in `diablo.cpp`
  and `nthread.cpp` before relying on it). The engine's frame rate must not drive game updates.
- Rendering is an 8-bit palettised 640x480 software surface (configurable in DevilutionX options),
  converted to the window at present time. CEL/CL2/CLX sprites and TRN palette remaps follow
  `engine/render/*` and `engine/load_*`.
- Settings persist to `diablo.ini`, saves to `single_N.sv` / `multi_N.sv` MPQs, in the DevilutionX
  config folder; the port uses its own folder so the owner's real saves are never touched.

## Known differences from the original

### Free-movement build (`--features free-movement`, off by default)

Asked for by the owner on 2026-10-05: a separate build where the player moves in any direction
like a modern ARPG. The normal build is unaffected: the code is in `src/freemove.rs` and every hook
checks `freemove::active_for`, which is false unless the feature is on (the parity tests, which
run the normal build, still pass byte for byte).

- The game logic stays tile-based; the player also has a continuous position. The tile is the one
  the position rounds to (`dPlayer`, monsters, missiles, triggers, lighting and saves use it). The
  sprite and camera are offset by the difference, interpolated between ticks.
- While moving the player stays in `PM_STAND` with the walk animation, facing the nearest of the
  eight directions; attacks, spells and hits interrupt it as they interrupt standing.
- `MakePlrPath` for the local player becomes a walk through a route found by the port's own A*
  search (`find_route`: up to 12000 tiles searched, where the original's `FindPath` stops at 25
  steps), cutting corners where a straight line is clear (`line_clear`), ending at the exact
  cursor point for ground clicks. A destination that cannot be reached (a roof, a wall) is walked
  toward as far as possible. A walk keeps its route while the destination tile is unchanged.
- Facing: the heading is smoothed over a couple of ticks; the facing changes when the smoothed
  heading is more than 28.5 degrees from it (so the sprite never points far from where the hero
  goes, which looks like gliding), and a new facing holds for 4 ticks against a turn back to a
  neighbouring direction (no flicker at the boundary between two directions). A walk goes straight to the cursor when nothing is in the way.
- The walk animation runs only while the player really moves (pressing into a wall shows the
  stand animation), and short pauses do not switch to standing. Which animation is showing is
  checked by its sprites, not its frame count (some classes have equal stand and walk lengths). Walking up to a monster/player/item/object stops when the player's tile is next to it;
  `CheckNewPath` waits until then and starts the original action.
- Collision: a tile may be entered if `PosOkPlayer` allows it; diagonal steps may not cut a solid
  corner. Blocked moves slide along one axis or stop at the edge of the current tile.
- The sprite is drawn in the drawing pass of the southernmost tile it stands between (as the
  original's walking player is), with the player's own tile deciding visibility.
- Speed: the original's step speeds (one step per 8 ticks: 1 tile along a tile axis, 1.41 along
  a tile diagonal), interpolated for directions in between, so legs and ground keep pace; x1.5
  in town with "run in town".
- Single player only; in multiplayer the original movement is used. Demo files recorded with the
  original movement do not replay in this build.
- Test hooks: `DIABLO_FREEMOVE_TRACE=1` prints the player's state every tick; input scripts can
  hold the mouse with `press <x> <y>` / `release <x> <y>`.

#### First-person view (same build, key X)

Asked for by the owner on 2026-10-05 as an optional gameplay feature; `src/firstperson.rs`.

- Drawn instead of `DrawGame` in `DrawView` with a column raycaster into the 8-bit back buffer;
  the panels and overlays are drawn over it as usual (floating damage numbers are skipped).
- Level art: each level piece is rendered once with `RenderTile` into a 64 x (MicroTileLen/2 x 32)
  picture (twice, on black and on white, to find its transparent pixels). A 3D point at (fx, fy)
  from a tile centre and height h takes the picture's pixel at its isometric projection
  (32 + (fx - fy) x 32, H - 16 + (fx + fy) x 16 - h x 45.25). Walls stand on the back edges of the
  tiles (x - 0.5 for the picture's left half, y - 0.5 for the right half), as the art draws them;
  a half counts as a wall when it has more than 64 pixels above the floor diamond. Wall art on a
  tile the hero can walk (an archway) is only drawn above 1.9 tiles, as its painted opening would
  otherwise hide what is behind it.
- Shading: the game's light tables with `dLight` of the tile (the wall's or the tile in front of
  it, whichever is lighter), plus distance fog in town, where `dLight` is 0 everywhere.
- Sprites: items, objects (not doors), monsters (found per tile as `DrawMonsterHelper` finds them,
  so walking monsters are placed by the original's walking offset), NPCs in town and missiles are
  upright billboards, depth-tested per pixel. A monster's sprite is swapped for the direction it
  shows the viewer (the isometric camera looks North; the turn relative to that is applied).
  Visibility follows the original: unlit monsters are hidden unless the hero has infravision.
- Aiming: `CheckCursMove` takes the tile from the first-person view when it is on: the monster,
  item or object drawn under the mouse, else the floor point under it (or just in front of the
  wall there). The rest of `CheckCursMove` and the click handlers are unchanged.
- Movement: W/A/S/D drive the free movement's stick direction relative to the view. The tile
  rules let the hero stand right at a wall, so in this view moves that come closer than 0.3 tiles
  to a wall (solid at waist height) are refused, and the eye sits up to 0.4 tiles behind the hero.
- Keys: X toggles (only where free movement is active: single player); W/A/S/D are taken before
  the keymapper while the view is on.
- Known limits: the isometric art is a picture of 3D shapes seen from one side, so wall caps and
  town houses look like sheared planes, arch openings are solid below 1.9 tiles, sprites are flat
  and pixelated up close, the player's own sprite is not drawn, there is no ceiling.
- Test hooks: `DIABLO_FP_YAW=<degrees>|monster` sets the view direction when the view is switched
  on (`monster`: the nearest monster or NPC in plain sight); `DIABLO_FP_DUMP=<dir>` writes every
  level piece picture as PPM; input scripts can hold keys with `keydown <key>` / `keyup <key>`;
  `tools/input_scripts/first_person.txt` walks around town and dungeon level 2.

### Both builds

Behaviour the owner would notice:

- **Config and save folder:** `%APPDATA%\diablo1_rs\devilution` instead of DevilutionX's folder, so
  the owner's real `diablo.ini` and saves are never touched. Screenshots (PrintScreen, PCX) go there too.
- **Multiplayer:** "Client-Server (TCP)" and "Offline" only. ZeroTier needs libzt (a whole
  user-space network stack and an outside service) and is not ported, so the connection screen
  does not list it, as in a DevilutionX build with `DISABLE_ZERO_TIER`. TCP games, with and without
  a password, interoperate with the original DevilutionX 1.5.3 in both directions (see "Parity
  evidence").
- **Multiplayer turn thread:** the original sends and receives turns on a second thread while the
  main thread loads a level. The port's game state is single-threaded, so that handler runs
  cooperatively, whenever a frame is presented while the main thread has released the mutex
  (loading screen, fades, the progress dialog). Between two loading-screen updates no turns go
  out; the other players' games wait out the gap (they have two turns in hand).
- **Network event handlers** (a player joined or left) run when the network call that received the
  event returns, instead of from inside it; the same handlers run in the same order.
- **Network error texts** come from the operating system through Rust (e.g. "No connection could be
  made because the target machine actively refused it. (os error 10061)") instead of asio.
- **TCP server dropping a player** sends one disconnect notice; the original can send it twice (its
  read and timer handlers both drop the connection), which the clients ignore anyway.
- **Game controllers** are read through Bevy's gilrs backend (the `gamepad` feature, on by
  default). Every pad gilrs has a mapping for is delivered as an SDL game controller; raw SDL
  joystick events are not produced (on Windows the original ignores them anyway: no `JOY_*`
  mappings are compiled in). The button-label style (Xbox / PlayStation / Nintendo / generic) is
  chosen from the pad's USB vendor (Microsoft, Sony, Nintendo); SDL also looks at the product id.
  Keyboard and mouse are unchanged; touch controls are not a target.
- **Audio:** WAV sounds are fully decoded on load and mixed with linear resampling (cpal output);
  DevilutionX streams some sounds and uses SDL_audiolib's Speex resampler. Mute is immediate (Aulib
  fades). MP3 music replacements are not supported (the shipped data is WAV). The audio device
  option offers the system default device only.
- **Movies:** played by the port's own Smacker decoder (`src/storm/smacker.rs`), written from the
  format description because libsmackerdec is not in the 1.5.3 source tree. Checked by decoding
  every frame of logo, intro, ending and the Hellfire intro (`examples/smk_probe.rs`): pictures are
  clean and the audio length matches the video length. Frames are shown at the video's size and
  letterboxed by the window, like DevilutionX's renderer path; the movie's own frame timing is
  used without an extra vsync wait.
- **In-game error dialogs** (`UiErrorOkDialog` while the game window is active) are ported but were
  not triggered in a test run. The overloads that draw a list of items behind the dialog are not
  used: "Unable to create character" shows over a black screen instead of over the hero screen.
- **Window events:** Bevy reports minimise/restore as window occlusion; it is mapped to
  `SDL_WINDOWEVENT_HIDDEN`/`SHOWN`. Closing the window is `SDL_QUIT`.
- **GOG install lookup:** the original also searches a GOG copy's install folder (from the
  registry) for the MPQ files; the port does not, so `--data-dir` (or the program's own folder)
  is needed.
- **`devilutionx.mpq`** (DevilutionX's own interface/font archive) is required, as by the
  original; players take it from the DevilutionX 1.5.3 release (README).
- **Clipboard paste** in text fields is not supported.
- **Discord Rich Presence / ZeroTier:** off (see "Replaced, not translated").

Faithful emulation of undefined or implementation-defined behaviour (same results as the shipped
exe where it could be determined; marked "inferred" where the memory layout was inferred):

- **`rand()`:** MSVCRT's `rand`/`srand` (the shipped exe imports them from MSVCRT.DLL) is
  reimplemented (`src/utils/crt_rand.rs`).
- **`std::sort`:** reimplemented from libstdc++'s algorithm (`src/utils/stdsort.rs`) so equal
  elements end up in the same order; not checked against a compiled reference (no C++ compiler on
  this machine).
- **Out-of-range `dungeon[x][y]` reads/writes** in the level generators (e.g. `AddObjTraps` walking off
  the map, L2/L3/L4 room code): emulated as reads of the flat `dungeon` array continuing into
  `pdungeon` (declared right after it in gendung.cpp; the layout is **inferred**), reads before the
  start return 0 and writes outside both arrays are dropped (`gendung::dungeon_flat`).
- **`Bitset2d`:** a flat bit array indexed `y * DMAXX + x` like the original's `std::bitset`; a
  total index out of range panics, as `std::bitset::test` throws.
- **L3 river table** and L2 `predungeon` accessors: flat tables with the same out-of-range rules.
- **Room coordinates** in L2 `CreateRoom` and L4 `GenerateRoom` wrap as `uint8_t`, as the original's
  `WorldTilePosition` does.
- **Monster health bar blue TRN:** the original leaves all but three entries uninitialised; they are
  identity here (the sprite only uses those three colours).
- **`InvDrawSlotBack`:** the original only clips the target position; pixels outside the surface
  are skipped here instead of written past the buffer.
- **Delta object records** (`DLevel::object`, an `unordered_map`) iterate in insertion order rather
  than the MSVC hash order; this only matters for multiplayer level sync.

Structural differences with the same results:

- **`AddBerserk`:** the target search lambda consumes randomness; it runs on a copy of the RNG
  state that is written back, matching the original's shared global state.
- **Headless runs** (`DIABLO_HEADLESS`) load no sprites; `AddRhino` keeps the non-graphical part of
  its animation set-up there. Normal runs use the original path.
- **Monster graphics** are converted to CLX per animation instead of one shared buffer
  (`MultiFileLoader`).
- **Animation progress:** `ProgressToNextGameTick` is passed explicitly to `AnimationInfo` instead
  of being read from a global; same values.
- **Debug builds:** `_DEBUG`-only code (text commands, the `SL_NONE` test map) is omitted, as in
  the shipped release build.
- **MPQ reader:** a stored (uncompressed) file whose packed size is padded past its unpacked size
  (hellfire.mpq's `gendata\Hellfire.smk`) reads the unpacked size from the last sector, as libmpq does.

Test-only additions (not reachable in a normal run): input-script commands `warp`, `setwarp` and
`store` (`DIABLO_INPUT_SCRIPT`) jump to a dungeon level, a quest level or a store page through the
game's own functions (`StartNewLvl`, `StartStore`); `killdiablo` kills Diablo through
`M_StartKill` (to check the ending); `padadd`, `pad`, `paddown`, `padup`,
`padaxis` drive a virtual game controller; `DIABLO_FIXED_STEP=pace` keeps the virtual clock in
step with the real one so two scripted games can play over the network; `DIABLO_DEMO_TRACE` prints
the player's state on every replayed demo tick; without a window (`DIABLO_HEADLESS`) fatal-error
message boxes are logged instead of shown, so automated runs never wait on a modal dialog.
`RUST_BACKTRACE=1` adds a backtrace to a fatal error.

## Parity evidence

- `tests/drlg_parity.rs` (generated by `tools/gen_drlg_tests.py`): DevilutionX's 43 dungeon
  generation tests, every tile and room index of all level types for fixed seeds.
- `tests/pack_parity.rs`: DevilutionX's item tests, 188 items regenerated from their seeds
  (Diablo, spawn, multiplayer, Hellfire) field by field, packing back to the same bytes.
- `tests/writehero.rs`: a level-50 rogue's multiplayer save is byte-identical (SHA-256) to
  DevilutionX's.
- `tests/timedemo.rs`: DevilutionX's WarriorLevel1to2 demo (6414 game ticks of play: walking,
  combat, loot, level change) replays to saves byte-identical to DevilutionX's reference.
- DevilutionX's other unit tests, assertion for assertion: `tests/inv_parity.rs`,
  `path_parity.rs`, `random_parity.rs`, `misc_parity.rs` (missiles, player, dead, lighting),
  `misc2_parity.rs` (effects, quests, stores, codec), `scrollrt_parity.rs` (and diablo),
  `automap_parity.rs` (and drlg_common), `animation_parity.rs` (generated by
  `tools/gen_anim_tests.py`: 339 animation steps) and `util_parity.rs` (cursor, format_int, math,
  rectangle). DevilutionX's tests of C++ helpers that became Rust's standard library
  (`utf8`, `str_cat`, `file_util`, `appfat`) have nothing to test in the port.
- `tests/net_parity.rs`: BLAKE2b, Poly1305, Argon2d/i/id, HSalsa20 and secretbox against
  RFC 7693, RFC 8439, RFC 9106 and NaCl's test vectors; the packet and frame wire format; a host
  and a guest playing turns and messages over loopback TCP with and without a password.
  `tests/writehero.rs` also sends the level-50 rogue through `PackNetPlayer`/`UnPackNetPlayer`
  (every validation passes, the rebuilt player matches; a tampered value is rejected).
- `tools/tcp_smoke.py`: two port games, a host and a guest, join, walk and leave over TCP.
- `tools/interop_original.ps1`: the original DevilutionX 1.5.3 and the port in one
  password-protected TCP game, either one hosting. Both see the other join ("Player 'Orig' (level
  1) is already in the game" / "just joined"), walk and leave; so the packet format, encryption,
  turn protocol and player packing match the original.
- Whole game and saves (`tools/input_scripts/`): `ending.txt` warps to level 16 and kills Diablo;
  his death, the victory movie and the end loop play. `save_game.txt` / `load_game.txt` save on
  dungeon level 3 and load that save in a new run: the same level, positions and corpses come
  back. Every dungeon type (cathedral, catacombs, caves, hell, Hellfire's nest
  and crypt) and the Skeleton King, Chamber of Bone and Lazarus quest levels were entered with
  `warp`/`setwarp` earlier.
- The tests that need game data read it from `DIABLO_DATA_DIR`; fixtures are read from the
  DevilutionX source (`DEVILUTIONX_SOURCE`, default `../Decomp/source_1.5.3`).
- For finding divergences: `examples/save_diff.rs` (field-level save diff using DevilutionX's
  memory maps), `examples/levelgen.rs` (generate the next level from a save), and the original
  1.5.3 exe replaying a cut demo (`--demo 0 --timedemo` with `--save-dir`/`--config-dir` pointing
  at a scratch copy) to get its state at any tick.
