//! `Source/levels/themes.cpp`: the theme room placing algorithms.

use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Point, Rectangle, Size};
use crate::engine::path::{is_tile_not_solid, is_tile_solid};
use crate::enums::*;
use crate::levels::gendung::*;
use crate::objects::{add_object, is_object_at_position};

/// `ThemeStruct`
#[derive(Clone, Copy, Debug)]
pub struct ThemeStruct {
    pub ttype: theme_id,
    pub ttval: i16,
}

impl Default for ThemeStruct {
    fn default() -> Self {
        ThemeStruct { ttype: THEME_NONE, ttval: 0 }
    }
}

/// Globals of levels/themes.cpp.
pub struct ThemesState {
    /// `numthemes`
    pub numthemes: i32,
    /// `armorFlag`
    pub armorFlag: bool,
    /// `weaponFlag`
    pub weaponFlag: bool,
    /// `zharlib`
    pub zharlib: i32,
    /// `themes`
    pub themes: [ThemeStruct; MAXTHEMES],
    cauldronFlag: bool,
    bFountainFlag: bool,
    mFountainFlag: bool,
    pFountainFlag: bool,
    tFountainFlag: bool,
    treasureFlag: bool,
    themex: i32,
    themey: i32,
    themeVar1: i32,
}

impl Default for ThemesState {
    fn default() -> Self {
        ThemesState {
            numthemes: 0,
            armorFlag: false,
            weaponFlag: false,
            zharlib: -1,
            themes: [ThemeStruct::default(); MAXTHEMES],
            cauldronFlag: false,
            bFountainFlag: false,
            mFountainFlag: false,
            pFountainFlag: false,
            tFountainFlag: false,
            treasureFlag: false,
            themex: 0,
            themey: 0,
            themeVar1: 0,
        }
    }
}

fn tv(ctx: &Ctx, x: i32, y: i32) -> i16 {
    ctx.gendung.dTransVal[x as usize][y as usize] as i16
}

fn ttval(ctx: &Ctx, t: i32) -> i16 {
    ctx.themes.themes[t as usize].ttval
}

/// `leveltype - 1`, the index into the per-level-type frequency tables.
fn lt(ctx: &Ctx) -> usize {
    ctx.gendung.leveltype as i8 as usize - 1
}

fn flip(ctx: &mut Ctx, f: u32) -> bool {
    ctx.rng.flip_coin(f)
}

fn tile_has(ctx: &Ctx, x: i32, y: i32, p: TileProperties) -> bool {
    tile_has_any(ctx, ctx.gendung.dPiece[x as usize][y as usize] as i32, p)
}

/// Original: `TFit_Shrine` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::TFit_Shrine(int i)
fn tfit_shrine(ctx: &mut Ctx, i: i32) -> bool {
    let mut xp = 0;
    let mut yp = 0;
    let mut found = 0;
    let v = ttval(ctx, i);
    while found == 0 {
        let test_position = Point::new(xp, yp);
        if tv(ctx, xp, yp) == v {
            if tile_has(ctx, xp, yp - 1, TileProperties::Trap)
                && is_tile_not_solid(ctx, test_position + Direction::NorthWest)
                && is_tile_not_solid(ctx, test_position + Direction::SouthEast)
                && tv(ctx, xp - 1, yp) == v
                && tv(ctx, xp + 1, yp) == v
                && !is_object_at_position(ctx, test_position + Direction::North)
                && !is_object_at_position(ctx, test_position + Direction::East)
            {
                found = 1;
            }
            if found == 0
                && tile_has(ctx, xp - 1, yp, TileProperties::Trap)
                && is_tile_not_solid(ctx, test_position + Direction::NorthEast)
                && is_tile_not_solid(ctx, test_position + Direction::SouthWest)
                && tv(ctx, xp, yp - 1) == v
                && tv(ctx, xp, yp + 1) == v
                && !is_object_at_position(ctx, test_position + Direction::North)
                && !is_object_at_position(ctx, test_position + Direction::West)
            {
                found = 2;
            }
        }
        if found == 0 {
            xp += 1;
            if xp == MAXDUNX as i32 {
                xp = 0;
                yp += 1;
                if yp == MAXDUNY as i32 {
                    return false;
                }
            }
        }
    }
    ctx.themes.themex = xp;
    ctx.themes.themey = yp;
    ctx.themes.themeVar1 = found;
    true
}

/// Original: `CheckThemeObj5` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::CheckThemeObj5(Point origin, int8_t regionId)
fn check_theme_obj5(ctx: &Ctx, origin: Point, region_id: i8) -> bool {
    for test_position in crate::engine::geometry::points_in_rectangle(Rectangle::from_center(origin, 2)) {
        // note out-of-bounds tiles are not solid, this function relies on the guard in TFit_Obj5 and dungeon border
        if is_tile_solid(ctx, test_position) {
            return false;
        }
        // If the theme object would extend into a different region then it doesn't fit.
        if ctx.gendung.dTransVal[test_position.x as usize][test_position.y as usize] != region_id {
            return false;
        }
    }
    true
}

/// Original: `TFit_Obj5` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::TFit_Obj5(int t)
fn tfit_obj5(ctx: &mut Ctx, t: i32) -> bool {
    let target_candidates = ctx.rng.generate_rnd(5);
    if target_candidates < 0 {
        // vanilla rng can return -3 for GenerateRnd(5), default behaviour is to set the output to 0,0 and return true in this case...
        ctx.themes.themex = 0;
        ctx.themes.themey = 0;
        return true;
    }
    let v = ttval(ctx, t);
    let mut candidates_found = 0;
    for tile in crate::engine::geometry::points_in_rectangle(Rectangle::new(Point::new(0, 0), Size::new(MAXDUNX as i32, MAXDUNY as i32))) {
        if tv(ctx, tile.x, tile.y) == v && is_tile_not_solid(ctx, tile) && check_theme_obj5(ctx, tile, v as i8) {
            // Use themex/y to keep track of the last candidate area found, in case we end up with fewer candidates than the target
            ctx.themes.themex = tile.x;
            ctx.themes.themey = tile.y;
            candidates_found += 1;
            if candidates_found > target_candidates {
                return true;
            }
        }
    }
    candidates_found > 0
}

/// Original: `TFit_SkelRoom` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::TFit_SkelRoom(int t)
fn tfit_skel_room(ctx: &mut Ctx, t: i32) -> bool {
    if !matches!(ctx.gendung.leveltype, DungeonType::Cathedral | DungeonType::Catacombs) {
        return false;
    }
    for i in 0..ctx.monster.LevelMonsterTypeCount {
        if crate::monster::is_skel(ctx.monster.LevelMonsterTypes[i].type_) {
            ctx.themes.themeVar1 = i as i32;
            return tfit_obj5(ctx, t);
        }
    }
    false
}

/// Original: `TFit_GoatShrine` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::TFit_GoatShrine(int t)
fn tfit_goat_shrine(ctx: &mut Ctx, t: i32) -> bool {
    for i in 0..ctx.monster.LevelMonsterTypeCount {
        if crate::monster::is_goat(ctx.monster.LevelMonsterTypes[i].type_) {
            ctx.themes.themeVar1 = i as i32;
            return tfit_obj5(ctx, t);
        }
    }
    false
}

/// Original: `CheckThemeObj3` (levels/themes.cpp). `frequency` defaults to 0.
// @port levels/themes.cpp|devilution::CheckThemeObj3(Point origin, int8_t regionId, unsigned frequency = 0)
fn check_theme_obj3(ctx: &mut Ctx, origin: Point, region_id: i8, frequency: u32) -> bool {
    for test_position in crate::engine::geometry::points_in_rectangle(Rectangle::from_center(origin, 1)) {
        if !in_dungeon_bounds(test_position) {
            return false;
        }
        if is_tile_solid(ctx, test_position) {
            return false;
        }
        // If the theme object would extend into a different region then it doesn't fit.
        if ctx.gendung.dTransVal[test_position.x as usize][test_position.y as usize] != region_id {
            return false;
        }
        if is_object_at_position(ctx, test_position) {
            return false;
        }
        if frequency > 0 && flip(ctx, frequency) {
            return false;
        }
    }
    true
}

/// Original: `TFit_Obj3` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::TFit_Obj3(int8_t regionId)
fn tfit_obj3(ctx: &mut Ctx, region_id: i8) -> bool {
    const OBJRND: [u32; 4] = [4, 4, 3, 5];
    for yp in 1..MAXDUNY as i32 - 1 {
        for xp in 1..MAXDUNX as i32 - 1 {
            let f = OBJRND[lt(ctx)];
            if check_theme_obj3(ctx, Point::new(xp, yp), region_id, f) {
                ctx.themes.themex = xp;
                ctx.themes.themey = yp;
                return true;
            }
        }
    }
    false
}

/// Original: `CheckThemeReqs` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::CheckThemeReqs(theme_id t)
fn check_theme_reqs(ctx: &Ctx, t: theme_id) -> bool {
    let lt = ctx.gendung.leveltype;
    let s = &ctx.themes;
    match t {
        THEME_SHRINE | THEME_SKELROOM | THEME_LIBRARY => !matches!(lt, DungeonType::Caves | DungeonType::Hell),
        THEME_ARMORSTAND | THEME_WEAPONRACK => lt != DungeonType::Cathedral,
        THEME_CAULDRON => lt == DungeonType::Hell && s.cauldronFlag,
        THEME_BLOODFOUNTAIN => s.bFountainFlag,
        THEME_PURIFYINGFOUNTAIN => s.pFountainFlag,
        THEME_MURKYFOUNTAIN => s.mFountainFlag,
        THEME_TEARFOUNTAIN => s.tFountainFlag,
        THEME_TREASURE => s.treasureFlag,
        _ => true,
    }
}

/// Original: `SpecialThemeFit` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::SpecialThemeFit(int i, theme_id t)
fn special_theme_fit(ctx: &mut Ctx, i: i32, t: theme_id) -> bool {
    let mut rv = check_theme_reqs(ctx, t);
    match t {
        THEME_SHRINE | THEME_LIBRARY => {
            if rv {
                rv = tfit_shrine(ctx, i);
            }
        }
        THEME_SKELROOM => {
            if rv {
                rv = tfit_skel_room(ctx, i);
            }
        }
        THEME_BLOODFOUNTAIN | THEME_PURIFYINGFOUNTAIN | THEME_MURKYFOUNTAIN | THEME_TEARFOUNTAIN | THEME_CAULDRON => {
            if rv {
                rv = tfit_obj5(ctx, i);
            }
            if rv {
                let s = &mut ctx.themes;
                match t {
                    THEME_BLOODFOUNTAIN => s.bFountainFlag = false,
                    THEME_PURIFYINGFOUNTAIN => s.pFountainFlag = false,
                    THEME_MURKYFOUNTAIN => s.mFountainFlag = false,
                    THEME_TEARFOUNTAIN => s.tFountainFlag = false,
                    _ => s.cauldronFlag = false,
                }
            }
        }
        THEME_GOATSHRINE => {
            if rv {
                rv = tfit_goat_shrine(ctx, i);
            }
        }
        THEME_TORTURE | THEME_DECAPITATED | THEME_ARMORSTAND | THEME_BRNCROSS | THEME_WEAPONRACK => {
            if rv {
                let v = ttval(ctx, i) as i8;
                rv = tfit_obj3(ctx, v);
            }
        }
        THEME_TREASURE => {
            if rv {
                ctx.themes.treasureFlag = false;
            }
        }
        _ => {}
    }
    rv
}

/// Original: `CheckThemeRoom` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::CheckThemeRoom(int tv)
fn check_theme_room(ctx: &Ctx, tvv: i32) -> bool {
    for i in 0..ctx.trigs.numtrigs as usize {
        let p = ctx.trigs.trigs[i].position;
        if tv(ctx, p.x, p.y) as i32 == tvv {
            return false;
        }
    }
    let mut tarea = 0;
    for j in 0..MAXDUNY as i32 {
        for i in 0..MAXDUNX as i32 {
            if tv(ctx, i, j) as i32 != tvv {
                continue;
            }
            if tile_contains_set_piece(ctx, Point::new(i, j)) {
                return false;
            }
            tarea += 1;
        }
    }
    if ctx.gendung.leveltype == DungeonType::Cathedral && !(9..=100).contains(&tarea) {
        return false;
    }
    for j in 0..MAXDUNY as i32 {
        for i in 0..MAXDUNX as i32 {
            if tv(ctx, i, j) as i32 != tvv || tile_has(ctx, i, j, TileProperties::Solid) {
                continue;
            }
            if tv(ctx, i - 1, j) as i32 != tvv && is_tile_not_solid(ctx, Point::new(i - 1, j)) {
                return false;
            }
            if tv(ctx, i + 1, j) as i32 != tvv && is_tile_not_solid(ctx, Point::new(i + 1, j)) {
                return false;
            }
            if tv(ctx, i, j - 1) as i32 != tvv && is_tile_not_solid(ctx, Point::new(i, j - 1)) {
                return false;
            }
            if tv(ctx, i, j + 1) as i32 != tvv && is_tile_not_solid(ctx, Point::new(i, j + 1)) {
                return false;
            }
        }
    }
    true
}

/// Original: `PlaceThemeMonsts` (levels/themes.cpp): places theme monsters with frequency 1/f.
// @port levels/themes.cpp|devilution::PlaceThemeMonsts(int t, int f)
fn place_theme_monsts(ctx: &mut Ctx, t: i32, f: i32) {
    let mut scattertypes: Vec<usize> = Vec::new();
    for i in 0..ctx.monster.LevelMonsterTypeCount {
        if (ctx.monster.LevelMonsterTypes[i].placeFlags & PLACE_SCATTER) != 0 {
            scattertypes.push(i);
        }
    }
    let r = ctx.rng.generate_rnd(scattertypes.len() as i32) as usize;
    let mtype = scattertypes[r];
    let v = ttval(ctx, t);
    for yp in 0..MAXDUNY as i32 {
        for xp in 0..MAXDUNX as i32 {
            let p = Point::new(xp, yp);
            if tv(ctx, xp, yp) == v && is_tile_not_solid(ctx, p) && ctx.items.dItem[xp as usize][yp as usize] == 0 && !is_object_at_position(ctx, p) && flip(ctx, f as u32) {
                let d = Direction::from_u8(ctx.rng.generate_rnd(8) as u8);
                crate::monster::add_monster(ctx, p, d, mtype, true);
            }
        }
    }
}

/// Original: `Theme_Barrel` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_Barrel(int t)
fn theme_barrel(ctx: &mut Ctx, t: i32) {
    const BARRND: [u32; 4] = [2, 6, 4, 8];
    const MONSTRND: [i32; 4] = [5, 7, 3, 9];
    let v = ttval(ctx, t);
    for yp in 0..MAXDUNY as i32 {
        for xp in 0..MAXDUNX as i32 {
            if tv(ctx, xp, yp) == v && is_tile_not_solid(ctx, Point::new(xp, yp)) && flip(ctx, BARRND[lt(ctx)]) {
                let r = if flip(ctx, BARRND[lt(ctx)]) { OBJ_BARREL } else { OBJ_BARRELEX };
                add_object(ctx, r, Point::new(xp, yp));
            }
        }
    }
    place_theme_monsts(ctx, t, MONSTRND[lt(ctx)]);
}

/// Original: `Theme_Shrine` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_Shrine(int t)
fn theme_shrine(ctx: &mut Ctx, t: i32) {
    const MONSTRND: [i32; 4] = [6, 6, 3, 9];
    tfit_shrine(ctx, t);
    let (x, y) = (ctx.themes.themex, ctx.themes.themey);
    if ctx.themes.themeVar1 == 1 {
        add_object(ctx, OBJ_CANDLE2, Point::new(x - 1, y));
        add_object(ctx, OBJ_SHRINER, Point::new(x, y));
        add_object(ctx, OBJ_CANDLE2, Point::new(x + 1, y));
    } else {
        add_object(ctx, OBJ_CANDLE2, Point::new(x, y - 1));
        add_object(ctx, OBJ_SHRINEL, Point::new(x, y));
        add_object(ctx, OBJ_CANDLE2, Point::new(x, y + 1));
    }
    place_theme_monsts(ctx, t, MONSTRND[lt(ctx)]);
}

/// Original: `Theme_MonstPit` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_MonstPit(int t)
fn theme_monst_pit(ctx: &mut Ctx, t: i32) {
    const MONSTRND: [i32; 4] = [6, 7, 3, 9];
    let mut r = ctx.rng.generate_rnd(100) + 1;
    let mut ixp = 0;
    let mut iyp = 0;
    let v = ttval(ctx, t);
    while r > 0 {
        if tv(ctx, ixp, iyp) == v && is_tile_not_solid(ctx, Point::new(ixp, iyp)) {
            r -= 1;
        }
        if r <= 0 {
            continue;
        }
        ixp += 1;
        if ixp == MAXDUNX as i32 {
            ixp = 0;
            iyp += 1;
            if iyp == MAXDUNY as i32 {
                iyp = 0;
            }
        }
    }
    crate::items::create_rnd_item(ctx, Point::new(ixp, iyp), true, false, true);
    crate::items::item_no_flippy(ctx);
    place_theme_monsts(ctx, t, MONSTRND[lt(ctx)]);
}

/// Original: `SpawnObjectOrSkeleton` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::SpawnObjectOrSkeleton(unsigned frequency, _object_id objectType, Point tile)
fn spawn_object_or_skeleton(ctx: &mut Ctx, frequency: u32, object_type: _object_id, tile: Point) {
    if flip(ctx, frequency) {
        add_object(ctx, object_type, tile);
    } else if let Some(skeleton) = crate::monster::pre_spawn_skeleton(ctx) {
        crate::monster::activate_skeleton(ctx, skeleton, tile);
    }
}

/// Original: `Theme_SkelRoom` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_SkelRoom(int t)
fn theme_skel_room(ctx: &mut Ctx, t: i32) {
    const MONSTRND: [u32; 4] = [6, 7, 3, 9];
    tfit_skel_room(ctx, t);
    let xp = ctx.themes.themex;
    let yp = ctx.themes.themey;
    let f = MONSTRND[lt(ctx)];
    add_object(ctx, OBJ_SKFIRE, Point::new(xp, yp));
    spawn_object_or_skeleton(ctx, f, OBJ_BANNERL, Point::new(xp - 1, yp - 1));
    if let Some(skeleton) = crate::monster::pre_spawn_skeleton(ctx) {
        crate::monster::activate_skeleton(ctx, skeleton, Point::new(xp, yp - 1));
    }
    spawn_object_or_skeleton(ctx, f, OBJ_BANNERR, Point::new(xp + 1, yp - 1));
    spawn_object_or_skeleton(ctx, f, OBJ_BANNERM, Point::new(xp - 1, yp));
    spawn_object_or_skeleton(ctx, f, OBJ_BANNERM, Point::new(xp + 1, yp));
    spawn_object_or_skeleton(ctx, f, OBJ_BANNERR, Point::new(xp - 1, yp + 1));
    if let Some(skeleton) = crate::monster::pre_spawn_skeleton(ctx) {
        crate::monster::activate_skeleton(ctx, skeleton, Point::new(xp, yp + 1));
    }
    spawn_object_or_skeleton(ctx, f, OBJ_BANNERL, Point::new(xp + 1, yp + 1));
    if !is_object_at_position(ctx, Point::new(xp, yp - 3)) {
        add_object(ctx, OBJ_SKELBOOK, Point::new(xp, yp - 2));
    }
    if !is_object_at_position(ctx, Point::new(xp, yp + 3)) {
        add_object(ctx, OBJ_SKELBOOK, Point::new(xp, yp + 2));
    }
}

/// Original: `Theme_Treasure` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_Treasure(int t)
fn theme_treasure(ctx: &mut Ctx, t: i32) {
    const TREASRND: [i8; 4] = [4, 9, 7, 10];
    const MONSTRND: [i32; 4] = [6, 8, 3, 7];
    ctx.rng.discard_random_values(1);
    let v = ttval(ctx, t);
    for yp in 0..MAXDUNY as i32 {
        for xp in 0..MAXDUNX as i32 {
            if tv(ctx, xp, yp) == v && is_tile_not_solid(ctx, Point::new(xp, yp)) {
                let treasure_type = TREASRND[lt(ctx)] as i32;
                let rv = ctx.rng.generate_rnd(treasure_type);
                // BUGFIX: this used to be `2*GenerateRnd(treasureType) == 0` however 2*0 has no effect, should probably be `FlipCoin(2*treasureType)`
                if flip(ctx, treasure_type as u32) {
                    crate::items::create_type_item(ctx, Point::new(xp, yp), false, ItemType::Gold, IMISC_NONE as i32, false, true, false);
                    crate::items::item_no_flippy(ctx);
                }
                if rv == 0 {
                    crate::items::create_rnd_item(ctx, Point::new(xp, yp), false, false, true);
                    crate::items::item_no_flippy(ctx);
                }
                // BUGFIX: the following code is likely not working as intended.
                //    `rv >= treasureType - 2` is not connected to either
                //    of the item creation branches above, thus the last (unrelated)
                //    item spawned/dropped on ground would be halved in value.
                if rv >= treasure_type - 2 && ctx.gendung.leveltype != DungeonType::Cathedral {
                    let idx = ctx.items.ActiveItems[ctx.items.ActiveItemCount as usize - 1] as usize;
                    let item = &mut ctx.items.Items[idx];
                    if item.IDidx == IDI_GOLD {
                        item._ivalue = (item._ivalue / 2).max(1);
                    }
                }
            }
        }
    }
    place_theme_monsts(ctx, t, MONSTRND[lt(ctx)]);
}

/// Original: `Theme_Library` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_Library(int t)
fn theme_library(ctx: &mut Ctx, t: i32) {
    const LIBRND: [u32; 4] = [1, 2, 2, 5];
    const MONSTRND: [i32; 4] = [5, 7, 3, 9];
    tfit_shrine(ctx, t);
    let (x, y) = (ctx.themes.themex, ctx.themes.themey);
    if ctx.themes.themeVar1 == 1 {
        add_object(ctx, OBJ_BOOKCANDLE, Point::new(x - 1, y));
        add_object(ctx, OBJ_BOOKCASER, Point::new(x, y));
        add_object(ctx, OBJ_BOOKCANDLE, Point::new(x + 1, y));
    } else {
        add_object(ctx, OBJ_BOOKCANDLE, Point::new(x, y - 1));
        add_object(ctx, OBJ_BOOKCASEL, Point::new(x, y));
        add_object(ctx, OBJ_BOOKCANDLE, Point::new(x, y + 1));
    }
    let v = ttval(ctx, t) as i8;
    for yp in 1..MAXDUNY as i32 - 1 {
        for xp in 1..MAXDUNX as i32 - 1 {
            if check_theme_obj3(ctx, Point::new(xp, yp), v, 0) && ctx.gendung.dMonster[xp as usize][yp as usize] == 0 && flip(ctx, LIBRND[lt(ctx)]) {
                let bookstand = add_object(ctx, OBJ_BOOKSTAND, Point::new(xp, yp));
                if !flip(ctx, 2 * LIBRND[lt(ctx)]) {
                    if let Some(b) = bookstand {
                        let o = &mut ctx.objects.Objects[b];
                        o._oSelFlag = 0;
                        o._oAnimFrame += 2;
                    }
                }
            }
        }
    }
    if crate::quests::is_quest_available(ctx, Q_ZHAR) && t == ctx.themes.zharlib {
        return;
    }
    place_theme_monsts(ctx, t, MONSTRND[lt(ctx)]);
}

/// Shared body of `Theme_Torture`, `Theme_Decap` and `Theme_BrnCross`.
fn theme_scatter_obj3(ctx: &mut Ctx, t: i32, start: i32, end: i32, freq: [u32; 4], obj: _object_id, monstrnd: [i32; 4]) {
    let v = ttval(ctx, t);
    for yp in start..end {
        for xp in start..MAXDUNX as i32 - (MAXDUNY as i32 - end) {
            if tv(ctx, xp, yp) == v && is_tile_not_solid(ctx, Point::new(xp, yp)) && check_theme_obj3(ctx, Point::new(xp, yp), v as i8, 0) && flip(ctx, freq[lt(ctx)]) {
                add_object(ctx, obj, Point::new(xp, yp));
            }
        }
    }
    place_theme_monsts(ctx, t, monstrnd[lt(ctx)]);
}

/// Original: `Theme_Torture` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_Torture(int t)
fn theme_torture(ctx: &mut Ctx, t: i32) {
    theme_scatter_obj3(ctx, t, 1, MAXDUNY as i32 - 1, [6, 8, 3, 8], OBJ_TNUDEM2, [6, 8, 3, 9]);
}

/// Original: `Theme_BloodFountain` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_BloodFountain(int t)
fn theme_blood_fountain(ctx: &mut Ctx, t: i32) {
    theme_obj5(ctx, t, OBJ_BLOODFTN, [6, 8, 3, 9]);
}

/// Shared body of the fountain and cauldron themes.
fn theme_obj5(ctx: &mut Ctx, t: i32, obj: _object_id, monstrnd: [i32; 4]) {
    tfit_obj5(ctx, t);
    let p = Point::new(ctx.themes.themex, ctx.themes.themey);
    add_object(ctx, obj, p);
    place_theme_monsts(ctx, t, monstrnd[lt(ctx)]);
}

/// Original: `Theme_Decap` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_Decap(int t)
fn theme_decap(ctx: &mut Ctx, t: i32) {
    theme_scatter_obj3(ctx, t, 1, MAXDUNY as i32 - 1, [6, 8, 3, 8], OBJ_DECAP, [6, 8, 3, 9]);
}

/// Original: `Theme_PurifyingFountain` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_PurifyingFountain(int t)
fn theme_purifying_fountain(ctx: &mut Ctx, t: i32) {
    theme_obj5(ctx, t, OBJ_PURIFYINGFTN, [6, 7, 3, 9]);
}

/// Original: `Theme_ArmorStand` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_ArmorStand(int t)
fn theme_armor_stand(ctx: &mut Ctx, t: i32) {
    if ctx.themes.armorFlag {
        let v = ttval(ctx, t) as i8;
        tfit_obj3(ctx, v);
        let p = Point::new(ctx.themes.themex, ctx.themes.themey);
        add_object(ctx, OBJ_ARMORSTAND, p);
    }
    theme_scatter_obj3(ctx, t, 0, MAXDUNY as i32, [6, 8, 3, 8], OBJ_ARMORSTANDN, [6, 7, 3, 9]);
    ctx.themes.armorFlag = false;
}

/// Original: `Theme_GoatShrine` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_GoatShrine(int t)
fn theme_goat_shrine(ctx: &mut Ctx, t: i32) {
    tfit_goat_shrine(ctx, t);
    let (x, y) = (ctx.themes.themex, ctx.themes.themey);
    add_object(ctx, OBJ_GOATSHRINE, Point::new(x, y));
    let v = ttval(ctx, t);
    for yy in y - 1..=y + 1 {
        for xx in x - 1..=x + 1 {
            if tv(ctx, xx, yy) == v && is_tile_not_solid(ctx, Point::new(xx, yy)) && (xx != x || yy != y) {
                let mt = ctx.themes.themeVar1 as usize;
                crate::monster::add_monster(ctx, Point::new(xx, yy), Direction::SouthWest, mt, true);
            }
        }
    }
}

/// Original: `Theme_Cauldron` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_Cauldron(int t)
fn theme_cauldron(ctx: &mut Ctx, t: i32) {
    theme_obj5(ctx, t, OBJ_CAULDRON, [6, 7, 3, 9]);
}

/// Original: `Theme_MurkyFountain` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_MurkyFountain(int t)
fn theme_murky_fountain(ctx: &mut Ctx, t: i32) {
    theme_obj5(ctx, t, OBJ_MURKYFTN, [6, 7, 3, 9]);
}

/// Original: `Theme_TearFountain` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_TearFountain(int t)
fn theme_tear_fountain(ctx: &mut Ctx, t: i32) {
    theme_obj5(ctx, t, OBJ_TEARFTN, [6, 7, 3, 9]);
}

/// Original: `Theme_BrnCross` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_BrnCross(int t)
fn theme_brn_cross(ctx: &mut Ctx, t: i32) {
    theme_scatter_obj3(ctx, t, 0, MAXDUNY as i32, [5, 7, 3, 8], OBJ_TBCROSS, [6, 8, 3, 9]);
}

/// Original: `Theme_WeaponRack` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::Theme_WeaponRack(int t)
fn theme_weapon_rack(ctx: &mut Ctx, t: i32) {
    if ctx.themes.weaponFlag {
        let v = ttval(ctx, t) as i8;
        tfit_obj3(ctx, v);
        let p = Point::new(ctx.themes.themex, ctx.themes.themey);
        add_object(ctx, OBJ_WEAPONRACK, p);
    }
    theme_scatter_obj3(ctx, t, 0, MAXDUNY as i32, [6, 8, 5, 8], OBJ_WEAPONRACKN, [6, 7, 3, 9]);
    ctx.themes.weaponFlag = false;
}

/// Original: `UpdateL4Trans` (levels/themes.cpp): sets each non-zero transparency value to 1.
// @port levels/themes.cpp|devilution::UpdateL4Trans()
fn update_l4_trans(ctx: &mut Ctx) {
    for col in ctx.gendung.dTransVal.iter_mut() {
        for v in col.iter_mut() {
            if *v != 0 {
                *v = 1;
            }
        }
    }
}

/// `ThemeGood`: the set of special theme IDs from which one will be selected at random.
const THEME_GOOD: [theme_id; 4] = [THEME_GOATSHRINE, THEME_SHRINE, THEME_SKELROOM, THEME_LIBRARY];

/// Original: `devilution::InitThemes` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::InitThemes()
pub fn init_themes(ctx: &mut Ctx) {
    {
        let s = &mut ctx.themes;
        s.zharlib = -1;
        s.numthemes = 0;
        s.armorFlag = true;
        s.bFountainFlag = true;
        s.cauldronFlag = true;
        s.mFountainFlag = true;
        s.pFountainFlag = true;
        s.tFountainFlag = true;
        s.treasureFlag = true;
        s.weaponFlag = true;
    }
    if ctx.gendung.currlevel == 16 || matches!(ctx.gendung.leveltype, DungeonType::Nest | DungeonType::Crypt) {
        return;
    }
    if ctx.gendung.leveltype == DungeonType::Cathedral {
        let mut i = 0;
        while i < 256 && (ctx.themes.numthemes as usize) < MAXTHEMES {
            if check_theme_room(ctx, i) {
                let n = ctx.themes.numthemes;
                ctx.themes.themes[n as usize].ttval = i as i16;
                let mut j = THEME_GOOD[ctx.rng.generate_rnd(4) as usize];
                while !special_theme_fit(ctx, n, j) {
                    j = ctx.rng.generate_rnd(17) as theme_id;
                }
                ctx.themes.themes[n as usize].ttype = j;
                ctx.themes.numthemes += 1;
            }
            i += 1;
        }
        return;
    }
    let theme_count = ctx.gendung.themeCount;
    for i in 0..theme_count as usize {
        ctx.themes.themes[i].ttype = THEME_NONE;
    }
    if crate::quests::is_quest_available(ctx, Q_ZHAR) {
        for j in 0..theme_count {
            ctx.themes.themes[j as usize].ttval = ctx.gendung.themeLoc[j as usize].ttval;
            if special_theme_fit(ctx, j, THEME_LIBRARY) {
                ctx.themes.themes[j as usize].ttype = THEME_LIBRARY;
                ctx.themes.zharlib = j;
                break;
            }
        }
    }
    for i in 0..theme_count {
        if ctx.themes.themes[i as usize].ttype == THEME_NONE {
            ctx.themes.themes[i as usize].ttval = ctx.gendung.themeLoc[i as usize].ttval;
            let mut j = THEME_GOOD[ctx.rng.generate_rnd(4) as usize];
            while !special_theme_fit(ctx, i, j) {
                j = ctx.rng.generate_rnd(17) as theme_id;
            }
            ctx.themes.themes[i as usize].ttype = j;
        }
    }
    ctx.themes.numthemes += theme_count;
}

/// Original: `devilution::HoldThemeRooms` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::HoldThemeRooms()
pub fn hold_theme_rooms(ctx: &mut Ctx) {
    if ctx.gendung.currlevel == 16 || matches!(ctx.gendung.leveltype, DungeonType::Nest | DungeonType::Crypt) {
        return;
    }
    if ctx.gendung.leveltype != DungeonType::Cathedral {
        drlg_hold_theme_rooms(ctx);
        return;
    }
    for i in 0..ctx.themes.numthemes as usize {
        let v = ctx.themes.themes[i].ttval as i8;
        for y in 0..MAXDUNY {
            for x in 0..MAXDUNX {
                if ctx.gendung.dTransVal[x][y] == v {
                    ctx.gendung.dFlags[x][y] |= DungeonFlag::Populated;
                }
            }
        }
    }
}

/// Original: `devilution::CreateThemeRooms` (levels/themes.cpp).
// @port levels/themes.cpp|devilution::CreateThemeRooms()
pub fn create_theme_rooms(ctx: &mut Ctx) {
    if ctx.gendung.currlevel == 16 || matches!(ctx.gendung.leveltype, DungeonType::Nest | DungeonType::Crypt) {
        return;
    }
    for i in 0..ctx.themes.numthemes {
        ctx.themes.themex = 0;
        ctx.themes.themey = 0;
        match ctx.themes.themes[i as usize].ttype {
            THEME_BARREL => theme_barrel(ctx, i),
            THEME_SHRINE => theme_shrine(ctx, i),
            THEME_MONSTPIT => theme_monst_pit(ctx, i),
            THEME_SKELROOM => theme_skel_room(ctx, i),
            THEME_TREASURE => theme_treasure(ctx, i),
            THEME_LIBRARY => theme_library(ctx, i),
            THEME_TORTURE => theme_torture(ctx, i),
            THEME_BLOODFOUNTAIN => theme_blood_fountain(ctx, i),
            THEME_DECAPITATED => theme_decap(ctx, i),
            THEME_PURIFYINGFOUNTAIN => theme_purifying_fountain(ctx, i),
            THEME_ARMORSTAND => theme_armor_stand(ctx, i),
            THEME_GOATSHRINE => theme_goat_shrine(ctx, i),
            THEME_CAULDRON => theme_cauldron(ctx, i),
            THEME_MURKYFOUNTAIN => theme_murky_fountain(ctx, i),
            THEME_TEARFOUNTAIN => theme_tear_fountain(ctx, i),
            THEME_BRNCROSS => theme_brn_cross(ctx, i),
            THEME_WEAPONRACK => theme_weapon_rack(ctx, i),
            ttype => crate::appfat::app_fatal(ctx, &format!("Unknown theme type: {ttype}")),
        }
    }
    if ctx.gendung.leveltype == DungeonType::Hell && ctx.gendung.themeCount > 0 {
        update_l4_trans(ctx);
    }
}
