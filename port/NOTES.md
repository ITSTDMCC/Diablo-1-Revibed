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
Smacker video, SHA-1 (save password hash, `sha.cpp` is game code). Each needs an ask before adding
a crate instead.

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

None yet.
