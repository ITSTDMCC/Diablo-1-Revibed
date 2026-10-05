//! `Source/levels/drlg_l1.cpp`: cathedral (and Hellfire crypt) level generation.
#![allow(non_upper_case_globals)]

use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Displacement, Point, Rectangle, Size};
use crate::enums::*;
use crate::levels::gendung::*;

/// Function-local and file-scope globals of levels/drlg_l1.cpp.
#[derive(Default)]
pub struct DrlgL1State {
    /// `Chamber`: marks where walls may not be added to the level
    pub Chamber: Bitset2d,
    /// `VerticalLayout`
    pub VerticalLayout: bool,
    /// `HasChamber1`
    pub HasChamber1: bool,
    /// `HasChamber2`
    pub HasChamber2: bool,
    /// `HasChamber3`
    pub HasChamber3: bool,
}

/// Builds a `Miniset` from row slices (`search`/`replace` are indexed [y][x]).
pub fn miniset(width: i32, height: i32, search: &[&[u8]], replace: &[&[u8]]) -> Miniset {
    let mut m = Miniset { size: Size::new(width, height), search: [[0; 6]; 6], replace: [[0; 6]; 6] };
    for (y, row) in search.iter().enumerate() {
        for (x, &v) in row.iter().enumerate() {
            m.search[y][x] = v;
        }
    }
    for (y, row) in replace.iter().enumerate() {
        for (x, &v) in row.iter().enumerate() {
            m.replace[y][x] = v;
        }
    }
    m
}

/// `STAIRSUP`: stairs up on a corner wall.
fn stairsup() -> Miniset {
    miniset(4, 4, &[&[13, 13, 13, 13], &[2, 2, 2, 2], &[13, 13, 13, 13], &[13, 13, 13, 13]], &[&[0, 66, 6, 0], &[63, 64, 65, 0], &[0, 67, 68, 0], &[0, 0, 0, 0]])
}

/// `STAIRSDOWN`
fn stairsdown() -> Miniset {
    miniset(4, 3, &[&[13, 13, 13, 13], &[13, 13, 13, 13], &[13, 13, 13, 13]], &[&[62, 57, 58, 0], &[61, 59, 60, 0], &[0, 0, 0, 0]])
}

/// `LAMPS`: candlestick.
fn lamps() -> Miniset {
    miniset(2, 2, &[&[13, 0], &[13, 13]], &[&[129, 0], &[130, 128]])
}

/// `PWATERIN`: Poisoned Water Supply entrance.
fn pwaterin() -> Miniset {
    miniset(
        6,
        6,
        &[&[13; 6], &[13; 6], &[13; 6], &[13; 6], &[13; 6], &[13; 6]],
        &[&[0, 0, 0, 0, 0, 0], &[0, 202, 200, 200, 84, 0], &[0, 199, 203, 203, 83, 0], &[0, 85, 206, 80, 81, 0], &[0, 0, 134, 135, 0, 0], &[0, 0, 0, 0, 0, 0]],
    )
}

// `Tile`
const VWall: u8 = 1;
const HWall: u8 = 2;
const Corner: u8 = 3;
const DWall: u8 = 4;
const DArch: u8 = 5;
const VWallEnd: u8 = 6;
const HWallEnd: u8 = 7;
const HArchEnd: u8 = 8;
const VArchEnd: u8 = 9;
const HArchVWall: u8 = 10;
const VArch: u8 = 11;
const HArch: u8 = 12;
const Floor: u8 = 13;
const HWallVArch: u8 = 14;
const Pillar: u8 = 15;
const VCorner: u8 = 16;
const HCorner: u8 = 17;
const DirtHwall: u8 = 18;
const DirtVwall: u8 = 19;
const VDirtCorner: u8 = 20;
const HDirtCorner: u8 = 21;
const Dirt: u8 = 22;
const DirtHwallEnd: u8 = 23;
const DirtVwallEnd: u8 = 24;
const VDoor: u8 = 25;
const HDoor: u8 = 26;
const HFenceVWall: u8 = 27;
const HDoorVDoor: u8 = 28;
const DFence: u8 = 29;
const VDoorEnd: u8 = 30;
const HDoorEnd: u8 = 31;
const VFenceEnd: u8 = 32;
const VFence: u8 = 35;
const HFence: u8 = 36;
const HWallVFence: u8 = 37;
const HArchVFence: u8 = 38;
const HArchVDoor: u8 = 39;
const HArchVWall3: u8 = 40;
const DWall2: u8 = 41;
const HWallVArch2: u8 = 42;
const DWall3: u8 = 43;
const EntranceStairs: u8 = 64;
const VWall2: u8 = 79;
const HWall2: u8 = 80;
const DWall4: u8 = 82;
const VWallEnd2: u8 = 84;
const VWall4: u8 = 89;
const VWall5: u8 = 90;
const HWall4: u8 = 91;
const HWall5: u8 = 92;
const VWall8: u8 = 100;
const Floor12: u8 = 139;
const Floor13: u8 = 140;
const Floor14: u8 = 141;
const Floor15: u8 = 142;
const Floor16: u8 = 143;
const Floor17: u8 = 144;
const Floor18: u8 = 145;
const VWall17: u8 = 146;
const VArch5: u8 = 147;
const HWallShadow: u8 = 148;
const HArchShadow: u8 = 149;
const Floor19: u8 = 150;
const Floor20: u8 = 151;
const Floor21: u8 = 152;
const HArchShadow2: u8 = 153;
const HWallShadow2: u8 = 154;
const Floor22: u8 = 162;
const Floor23: u8 = 163;
const DirtHWall2: u8 = 199;
const DirtVWall2: u8 = 200;
const DirtCorner2: u8 = 202;
const DirtHWallEnd2: u8 = 204;
const DirtVWallEnd2: u8 = 205;

const fn sh(strig: u8, s1: u8, s2: u8, s3: u8, nv1: u8, nv2: u8, nv3: u8) -> ShadowStruct {
    ShadowStruct { strig, s1, s2, s3, nv1, nv2, nv3 }
}

/// `ShadowPatterns`: shadows for 2x2 blocks of base tile IDs in the Cathedral.
const SHADOW_PATTERNS: [ShadowStruct; 37] = [
    sh(HWallEnd, Floor, 0, Floor, Floor17, 0, Floor15),
    sh(VCorner, Floor, 0, Floor, Floor17, 0, Floor15),
    sh(Pillar, Floor, 0, Floor, Floor18, 0, Floor15),
    sh(DArch, Floor, Floor, Floor, Floor21, Floor13, Floor12),
    sh(DArch, Floor, VWall, Floor, Floor16, VWall17, Floor12),
    sh(DArch, Floor, Floor, HWall, Floor16, Floor13, HWallShadow),
    sh(DArch, 0, VWall, HWall, 0, VWall17, HWallShadow),
    sh(DArch, Floor, VArch, Floor, Floor16, VArch5, Floor12),
    sh(DArch, Floor, Floor, HArch, Floor16, Floor13, HArchShadow),
    sh(DArch, Floor, VArch, HArch, Floor19, VArch5, HArchShadow),
    sh(DArch, Floor, VWall, HArch, Floor16, VWall17, HArchShadow),
    sh(DArch, Floor, VArch, HWall, Floor16, VArch5, HWallShadow),
    sh(VArchEnd, Floor, Floor, Floor, Floor17, Floor13, Floor15),
    sh(VArchEnd, Floor, VWall, Floor, Floor17, VWall17, Floor15),
    sh(VArchEnd, Floor, VArch, Floor, Floor20, VArch5, Floor15),
    sh(HArchEnd, Floor, 0, Floor, Floor17, 0, Floor12),
    sh(HArchEnd, Floor, 0, HArch, Floor16, 0, HArchShadow),
    sh(HArchEnd, 0, 0, HWall, 0, 0, HWallShadow),
    sh(VArch, 0, 0, Floor, 0, 0, Floor12),
    sh(VArch, Floor, 0, Floor, Floor12, 0, Floor12),
    sh(VArch, HWall, 0, Floor, HWallShadow, 0, Floor12),
    sh(VArch, HArch, 0, Floor, HArchShadow, 0, Floor12),
    sh(VArch, Floor, VArch, HArch, Floor12, 0, HArchShadow),
    sh(HWallVArch, 0, 0, Floor, 0, 0, Floor12),
    sh(HWallVArch, Floor, 0, Floor, Floor12, 0, Floor12),
    sh(HWallVArch, HWall, 0, Floor, HWallShadow, 0, Floor12),
    sh(HWallVArch, HArch, 0, Floor, HArchShadow, 0, Floor12),
    sh(HWallVArch, Floor, VArch, HArch, Floor12, 0, HArchShadow),
    sh(HArchVWall, 0, Floor, 0, 0, Floor13, 0),
    sh(HArchVWall, Floor, Floor, 0, Floor13, Floor13, 0),
    sh(HArchVWall, 0, VWall, 0, 0, VWall17, 0),
    sh(HArchVWall, Floor, VArch, 0, Floor13, VArch5, 0),
    sh(HArch, 0, Floor, 0, 0, Floor13, 0),
    sh(HArch, Floor, Floor, 0, Floor13, Floor13, 0),
    sh(HArch, 0, VWall, 0, 0, VWall17, 0),
    sh(HArch, Floor, VArch, 0, Floor13, VArch5, 0),
    sh(Corner, Floor, VArch, HArch, Floor19, 0, 0),
];

/// `BaseTypes`: maps tile IDs to their corresponding base tile ID.
const BASE_TYPES: [u8; 207] = [
    0, //
    VWall, HWall, Corner, DWall, DArch, VWallEnd, HWallEnd, HArchEnd, VArchEnd, //
    HArchVWall, VArch, HArch, Floor, HWallVArch, Pillar, VCorner, HCorner, //
    0, 0, 0, 0, 0, 0, 0, //
    VWall, HWall, HArchVWall, DWall, DArch, VWallEnd, HWallEnd, HArchEnd, //
    VArchEnd, HArchVWall, VArch, HArch, HWallVArch, DArch, HWallVArch, //
    HArchVWall, DWall, HWallVArch, DWall, DArch, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, //
    VWall, HWall, Corner, DWall, VWall, VWallEnd, HWallEnd, VCorner, HCorner, //
    HWall, VWall, VWall, HWall, HWall, VWall, VWall, HWall, HWall, HWall, HWall, //
    HWall, VWall, VWall, VArch, VWall, Floor, Floor, Floor, VWall, HWall, VWall, //
    HWall, VWall, HWall, VWall, HWall, HWall, HWall, HWall, HArch, //
    0, 0, //
    VArch, VWall, VArch, VWall, Floor, //
    0, 0, 0, 0, 0, 0, 0, //
    Floor, Floor, Floor, Floor, Floor, Floor, Floor, Floor, Floor, Floor, Floor, //
    Floor, Floor, VWall, VArch, HWall, HArch, Floor, Floor, Floor, HArch, HWall, //
    VWall, HWall, HWall, DWall, HWallVArch, DWall, HArchVWall, Floor, Floor, //
    DWall, DWall, VWall, VWall, DWall, HWall, HWall, Floor, Floor, Floor, Floor, //
    VDoor, HDoor, HDoorVDoor, VDoorEnd, HDoorEnd, DWall2, DWall3, HArchVWall3, //
    DWall2, HWallVArch2, DWall3, VDoor, DWall2, DWall3, HDoorVDoor, HDoorVDoor, //
    VWall, HWall, VDoor, HDoor, Dirt, Dirt, VDoor, HDoor, //
    0, 0, 0, 0, 0, 0, 0, 0,
];

/// `TileDecorations`: maps tile IDs to their corresponding undecorated tile ID.
const TILE_DECORATIONS: [u8; 207] = [
    0, //
    VWall, HWall, Corner, DWall, DArch, VWallEnd, HWallEnd, HArchEnd, VArchEnd, //
    HArchVWall, VArch, HArch, Floor, HWallVArch, Pillar, VCorner, HCorner, //
    0, 0, 0, 0, 0, 0, 0, //
    VDoor, HDoor, //
    0, //
    HDoorVDoor, //
    0, //
    VDoorEnd, HDoorEnd, //
    0, 0, 0, 0, 0, 0, 0, 0, //
    HArchVWall3, DWall2, HWallVArch2, DWall3, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    VWall2, HWall2, //
    0, //
    DWall4, //
    0, 0, 0, 0, 0, 0, //
    VWall2, //
    0, //
    HWall2, //
    0, 0, //
    VWall2, HWall2, //
    0, //
    HWall, HWall, HWall, VWall, VWall, VArch, VDoor, Floor, Floor, Floor, VWall, //
    HWall, VWall, HWall, VWall, HWall, VWall, HWall, HWall, HWall, HWall, HArch, //
    0, 0, //
    VArch, VWall, VArch, VWall, Floor, //
    0, 0, 0, 0, 0, 0, 0, //
    Floor, Floor, Floor, Floor, Floor, Floor, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, //
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

const DX: i32 = DMAXX as i32;
const DY: i32 = DMAXY as i32;

fn rnd(ctx: &mut Ctx, v: i32) -> i32 {
    ctx.rng.generate_rnd(v)
}

fn dun(ctx: &Ctx, x: i32, y: i32) -> u8 {
    ctx.gendung.dungeon[x as usize][y as usize]
}

fn set_dun(ctx: &mut Ctx, x: i32, y: i32, v: u8) {
    ctx.gendung.dungeon[x as usize][y as usize] = v;
}

/// Original: `ApplyShadowsPatterns` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::ApplyShadowsPatterns()
fn apply_shadows_patterns(ctx: &mut Ctx) {
    let g = &mut ctx.gendung;
    for y in 1..DMAXY {
        for x in 1..DMAXX {
            let s00 = BASE_TYPES[g.dungeon[x][y] as usize];
            let s10 = BASE_TYPES[g.dungeon[x - 1][y] as usize];
            let s01 = BASE_TYPES[g.dungeon[x][y - 1] as usize];
            let s11 = BASE_TYPES[g.dungeon[x - 1][y - 1] as usize];
            for shadow in SHADOW_PATTERNS.iter() {
                if shadow.strig != s00 {
                    continue;
                }
                if shadow.s1 != 0 && shadow.s1 != s11 {
                    continue;
                }
                if shadow.s2 != 0 && shadow.s2 != s01 {
                    continue;
                }
                if shadow.s3 != 0 && shadow.s3 != s10 {
                    continue;
                }
                let (xi, yi) = (x as i32, y as i32);
                if shadow.nv1 != 0 && !g.Protected.test(xi - 1, yi - 1) {
                    g.dungeon[x - 1][y - 1] = shadow.nv1;
                }
                if shadow.nv2 != 0 && !g.Protected.test(xi, yi - 1) {
                    g.dungeon[x][y - 1] = shadow.nv2;
                }
                if shadow.nv3 != 0 && !g.Protected.test(xi - 1, yi) {
                    g.dungeon[x - 1][y] = shadow.nv3;
                }
            }
        }
    }
    let fence = |t: u8| matches!(t, DFence | VFenceEnd | VFence | HWallVFence | HArchVFence | HArchVDoor);
    for y in 1..DMAXY {
        for x in 1..DMAXX {
            if g.Protected.test(x as i32 - 1, y as i32) {
                continue;
            }
            if g.dungeon[x - 1][y] == Floor12 {
                g.dungeon[x - 1][y] = if fence(g.dungeon[x][y]) { Floor14 } else { Floor12 };
            }
            if g.dungeon[x - 1][y] == HArchShadow {
                g.dungeon[x - 1][y] = if fence(g.dungeon[x][y]) { HArchShadow2 } else { HArchShadow };
            }
            if g.dungeon[x - 1][y] == HWallShadow {
                g.dungeon[x - 1][y] = if fence(g.dungeon[x][y]) { HWallShadow2 } else { HWallShadow };
            }
        }
    }
}

/// Original: `CanReplaceTile` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::CanReplaceTile(uint8_t replace, Point tile)
fn can_replace_tile(ctx: &Ctx, replace: u8, tile: Point) -> bool {
    if replace < VWallEnd2 || replace > VWall8 {
        return true;
    }
    // BUGFIX: p2 is a workaround for a bug, only p1 should have been used (fixing this breaks compatability)
    let in_b = |p: Point| p.x >= 0 && p.x < DX && p.y >= 0 && p.y < DY;
    let cmp = |p1: Point, p2: Point| in_b(p1) && in_b(p2) && (dun(ctx, p1.x, p1.y) >= VWallEnd2 && dun(ctx, p2.x, p2.y) <= VWall8);
    let nw = tile + Direction::NorthWest;
    if cmp(tile + Direction::NorthWest, nw) || cmp(tile + Direction::SouthEast, nw) || cmp(tile + Direction::SouthWest, nw) || cmp(tile + Direction::NorthEast, nw) {
        return false;
    }
    true
}

/// Original: `FillFloor` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::FillFloor()
fn fill_floor(ctx: &mut Ctx) {
    for j in 0..DY {
        for i in 0..DX {
            if dun(ctx, i, j) != Floor || ctx.gendung.Protected.test(i, j) {
                continue;
            }
            let rv = rnd(ctx, 3);
            if rv == 1 {
                set_dun(ctx, i, j, Floor22);
            } else if rv == 2 {
                set_dun(ctx, i, j, Floor23);
            }
        }
    }
}

/// Original: `LoadQuestSetPieces` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::LoadQuestSetPieces()
fn load_quest_set_pieces(ctx: &mut Ctx) {
    use crate::quests::is_quest_available;
    if is_quest_available(ctx, Q_BUTCHER) {
        ctx.gendung.pSetPiece = Some(load_u16_file(ctx, "levels\\l1data\\rnd6.dun"));
    } else if is_quest_available(ctx, Q_SKELKING) && !crate::quests::use_multiplayer_quests(ctx) {
        ctx.gendung.pSetPiece = Some(load_u16_file(ctx, "levels\\l1data\\skngdo.dun"));
    } else if is_quest_available(ctx, Q_LTBANNER) {
        ctx.gendung.pSetPiece = Some(load_u16_file(ctx, "levels\\l1data\\banner2.dun"));
    }
}

/// Original: `InitDungeonPieces` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::InitDungeonPieces()
fn init_dungeon_pieces(ctx: &mut Ctx) {
    let g = &mut ctx.gendung;
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            let pc = match g.dPiece[i][j] {
                11 | 70 | 320 | 210 | 340 | 417 => 1,
                10 | 248 | 324 | 343 | 330 | 420 => 2,
                252 => 3,
                254 => 4,
                258 => 5,
                266 => 6,
                _ => continue,
            };
            g.dSpecial[i][j] = pc;
        }
    }
}

/// Original: `InitDungeonFlags` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::InitDungeonFlags()
fn init_dungeon_flags(ctx: &mut Ctx) {
    ctx.gendung.dungeon = [[Dirt; DMAXY]; DMAXX];
    ctx.gendung.Protected.reset();
    ctx.drlg_l1.Chamber.reset();
}

/// Original: `MapRoom` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::MapRoom(Rectangle room)
fn map_room(ctx: &mut Ctx, room: Rectangle) {
    for y in 0..room.size.height {
        for x in 0..room.size.width {
            ctx.gendung.DungeonMask.set(room.position.x + x, room.position.y + y);
        }
    }
}

/// Original: `CheckRoom` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::CheckRoom(Rectangle room)
fn check_room(ctx: &Ctx, room: Rectangle) -> bool {
    for j in 0..room.size.height {
        for i in 0..room.size.width {
            if i + room.position.x < 0 || i + room.position.x >= DX || j + room.position.y < 0 || j + room.position.y >= DY {
                return false;
            }
            if ctx.gendung.DungeonMask.test(i + room.position.x, j + room.position.y) {
                return false;
            }
        }
    }
    true
}

/// Original: `GenerateRoom` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::GenerateRoom(Rectangle area, bool verticalLayout)
fn generate_room(ctx: &mut Ctx, area: Rectangle, mut vertical_layout: bool) {
    let rotate = ctx.rng.flip_coin(4);
    vertical_layout = (!vertical_layout && rotate) || (vertical_layout && !rotate);

    let mut place_room1 = false;
    let mut room1 = Rectangle::default();
    for _ in 0..20 {
        let random_width = (rnd(ctx, 5) + 2) & !1;
        let random_height = (rnd(ctx, 5) + 2) & !1;
        room1.size = Size::new(random_width, random_height);
        room1.position = area.position;
        if vertical_layout {
            room1.position += Displacement::new(-room1.size.width, area.size.height / 2 - room1.size.height / 2);
            // BUGFIX: swap height and width ({ room1.size.width + 1, room1.size.height + 2 }) (workaround applied below)
            place_room1 = check_room(ctx, Rectangle::new(room1.position + Displacement::new(-1, -1), Size::new(room1.size.height + 2, room1.size.width + 1)));
        } else {
            room1.position += Displacement::new(area.size.width / 2 - room1.size.width / 2, -room1.size.height);
            place_room1 = check_room(ctx, Rectangle::new(room1.position + Displacement::new(-1, -1), Size::new(room1.size.width + 2, room1.size.height + 1)));
        }
        if place_room1 {
            break;
        }
    }

    if place_room1 {
        map_room(ctx, Rectangle::new(room1.position, Size::new((DX - room1.position.x).min(room1.size.width), (DX - room1.position.y).min(room1.size.height))));
    }

    let place_room2;
    let mut room2 = room1;
    if vertical_layout {
        room2.position.x = area.position.x + area.size.width;
        place_room2 = check_room(ctx, Rectangle::new(room2.position + Displacement::new(0, -1), Size::new(room2.size.width + 1, room2.size.height + 2)));
    } else {
        room2.position.y = area.position.y + area.size.height;
        place_room2 = check_room(ctx, Rectangle::new(room2.position + Displacement::new(-1, 0), Size::new(room2.size.width + 2, room2.size.height + 1)));
    }

    if place_room2 {
        map_room(ctx, room2);
    }
    if place_room1 {
        generate_room(ctx, room1, !vertical_layout);
    }
    if place_room2 {
        generate_room(ctx, room2, !vertical_layout);
    }
}

/// Original: `FirstRoom` (levels/drlg_l1.cpp): generate a boolean dungeon room layout.
// @port levels/drlg_l1.cpp|devilution::FirstRoom()
fn first_room(ctx: &mut Ctx) {
    ctx.gendung.DungeonMask.reset();

    ctx.drlg_l1.VerticalLayout = ctx.rng.flip_coin(2);
    ctx.drlg_l1.HasChamber1 = !ctx.rng.flip_coin(2);
    ctx.drlg_l1.HasChamber2 = !ctx.rng.flip_coin(2);
    ctx.drlg_l1.HasChamber3 = !ctx.rng.flip_coin(2);

    if !ctx.drlg_l1.HasChamber1 || !ctx.drlg_l1.HasChamber3 {
        ctx.drlg_l1.HasChamber2 = true;
    }

    let mut chamber1 = Rectangle::new(Point::new(1, 15), Size::new(10, 10));
    let chamber2 = Rectangle::new(Point::new(15, 15), Size::new(10, 10));
    let mut chamber3 = Rectangle::new(Point::new(29, 15), Size::new(10, 10));
    let mut hallway = Rectangle::new(Point::new(1, 17), Size::new(38, 6));
    let s = &ctx.drlg_l1;
    let (c1, c2, c3, vertical) = (s.HasChamber1, s.HasChamber2, s.HasChamber3, s.VerticalLayout);
    if !c1 {
        hallway.position.x += 17;
        hallway.size.width -= 17;
    }
    if !c3 {
        hallway.size.width -= 16;
    }
    if vertical {
        std::mem::swap(&mut chamber1.position.x, &mut chamber1.position.y);
        std::mem::swap(&mut chamber3.position.x, &mut chamber3.position.y);
        std::mem::swap(&mut hallway.position.x, &mut hallway.position.y);
        std::mem::swap(&mut hallway.size.width, &mut hallway.size.height);
    }

    if c1 {
        map_room(ctx, chamber1);
    }
    if c2 {
        map_room(ctx, chamber2);
    }
    if c3 {
        map_room(ctx, chamber3);
    }

    map_room(ctx, hallway);

    if c1 {
        generate_room(ctx, chamber1, vertical);
    }
    if c2 {
        generate_room(ctx, chamber2, vertical);
    }
    if c3 {
        generate_room(ctx, chamber3, vertical);
    }
}

/// Original: `FindArea` (levels/drlg_l1.cpp): the number of mega tiles used by the layout.
// @port levels/drlg_l1.cpp|devilution::FindArea()
fn find_area(ctx: &Ctx) -> usize {
    let mut n = 0;
    for x in 0..DX {
        for y in 0..DY {
            if ctx.gendung.DungeonMask.test(x, y) {
                n += 1;
            }
        }
    }
    n
}

/// Original: `MakeDmt` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::MakeDmt()
fn make_dmt(ctx: &mut Ctx) {
    let g = &mut ctx.gendung;
    let m = |x: i32, y: i32| g.DungeonMask.test(x, y);
    let mut out = g.dungeon;
    for j in 0..DY - 1 {
        for i in 0..DX - 1 {
            out[i as usize][j as usize] = if m(i, j) {
                Floor
            } else if !m(i + 1, j + 1) && m(i, j + 1) && m(i + 1, j) {
                Floor // Remove diagonal corners
            } else if m(i + 1, j + 1) && m(i, j + 1) && m(i + 1, j) {
                VCorner
            } else if m(i, j + 1) {
                HWall
            } else if m(i + 1, j) {
                VWall
            } else if m(i + 1, j + 1) {
                DWall
            } else {
                Dirt
            };
        }
    }
    g.dungeon = out;
}

fn is_wall_end(t: u8) -> bool {
    matches!(t, Corner | DWall | DArch | VWallEnd | HWallEnd | VCorner | HCorner | DirtHwall | DirtVwall | VDirtCorner | HDirtCorner | DirtHwallEnd | DirtVwallEnd)
}

/// Original: `HorizontalWallOk` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::HorizontalWallOk(Point position)
fn horizontal_wall_ok(ctx: &Ctx, position: Point) -> i32 {
    let mut length = 1;
    while dun(ctx, position.x + length, position.y) == Floor {
        if dun(ctx, position.x + length, position.y - 1) != Floor
            || dun(ctx, position.x + length, position.y + 1) != Floor
            || ctx.gendung.Protected.test(position.x + length, position.y)
            || ctx.drlg_l1.Chamber.test(position.x + length, position.y)
        {
            break;
        }
        length += 1;
    }
    if length == 1 {
        return -1;
    }
    if !is_wall_end(dun(ctx, position.x + length, position.y)) {
        return -1;
    }
    length
}

/// Original: `VerticalWallOk` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::VerticalWallOk(Point position)
fn vertical_wall_ok(ctx: &Ctx, position: Point) -> i32 {
    let mut length = 1;
    while dun(ctx, position.x, position.y + length) == Floor {
        if dun(ctx, position.x - 1, position.y + length) != Floor
            || dun(ctx, position.x + 1, position.y + length) != Floor
            || ctx.gendung.Protected.test(position.x, position.y + length)
            || ctx.drlg_l1.Chamber.test(position.x, position.y + length)
        {
            break;
        }
        length += 1;
    }
    if length == 1 {
        return -1;
    }
    if !is_wall_end(dun(ctx, position.x, position.y + length)) {
        return -1;
    }
    length
}

/// Original: `HorizontalWall` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::HorizontalWall(Point position, Tile start, int maxX)
fn horizontal_wall(ctx: &mut Ctx, position: Point, mut start: u8, max_x: i32) {
    let mut wall_tile = HWall;
    let mut door_tile = HDoor;
    match rnd(ctx, 4) {
        2 => {
            // Add arch
            wall_tile = HArch;
            door_tile = HArch;
            if start == HWall {
                start = HArch;
            } else if start == DWall {
                start = HArchVWall;
            }
        }
        3 => {
            // Add Fence
            wall_tile = HFence;
            if start == HWall {
                start = HFence;
            } else if start == DWall {
                start = HFenceVWall;
            }
        }
        _ => {}
    }
    if rnd(ctx, 6) == 5 {
        door_tile = HArch;
    }
    set_dun(ctx, position.x, position.y, start);
    for x in 1..max_x {
        set_dun(ctx, position.x + x, position.y, wall_tile);
    }
    let x = rnd(ctx, max_x - 1) + 1;
    set_dun(ctx, position.x + x, position.y, door_tile);
    if door_tile == HDoor {
        ctx.gendung.Protected.set(position.x + x, position.y);
    }
}

/// Original: `VerticalWall` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::VerticalWall(Point position, Tile start, int maxY)
fn vertical_wall(ctx: &mut Ctx, position: Point, mut start: u8, max_y: i32) {
    let mut wall_tile = VWall;
    let mut door_tile = VDoor;
    match rnd(ctx, 4) {
        2 => {
            // Add arch
            wall_tile = VArch;
            door_tile = VArch;
            if start == VWall {
                start = VArch;
            } else if start == DWall {
                start = HWallVArch;
            }
        }
        3 => {
            // Add Fence
            wall_tile = VFence;
            if start == VWall {
                start = VFence;
            } else if start == DWall {
                start = HWallVFence;
            }
        }
        _ => {}
    }
    if rnd(ctx, 6) == 5 {
        door_tile = VArch;
    }
    set_dun(ctx, position.x, position.y, start);
    for y in 1..max_y {
        set_dun(ctx, position.x, position.y + y, wall_tile);
    }
    let y = rnd(ctx, max_y - 1) + 1;
    set_dun(ctx, position.x, position.y + y, door_tile);
    if door_tile == VDoor {
        ctx.gendung.Protected.set(position.x, position.y + y);
    }
}

/// Original: `AddWall` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::AddWall()
fn add_wall(ctx: &mut Ctx) {
    for j in 0..DY {
        for i in 0..DX {
            if ctx.gendung.Protected.test(i, j) || ctx.drlg_l1.Chamber.test(i, j) {
                continue;
            }
            let p = Point::new(i, j);
            if dun(ctx, i, j) == Corner {
                ctx.rng.discard_random_values(1);
                let max_x = horizontal_wall_ok(ctx, p);
                if max_x != -1 {
                    horizontal_wall(ctx, p, HWall, max_x);
                }
            }
            if dun(ctx, i, j) == Corner {
                ctx.rng.discard_random_values(1);
                let max_y = vertical_wall_ok(ctx, p);
                if max_y != -1 {
                    vertical_wall(ctx, p, VWall, max_y);
                }
            }
            if dun(ctx, i, j) == VWallEnd {
                ctx.rng.discard_random_values(1);
                let max_x = horizontal_wall_ok(ctx, p);
                if max_x != -1 {
                    horizontal_wall(ctx, p, DWall, max_x);
                }
            }
            if dun(ctx, i, j) == HWallEnd {
                ctx.rng.discard_random_values(1);
                let max_y = vertical_wall_ok(ctx, p);
                if max_y != -1 {
                    vertical_wall(ctx, p, DWall, max_y);
                }
            }
            if dun(ctx, i, j) == HWall {
                ctx.rng.discard_random_values(1);
                let max_x = horizontal_wall_ok(ctx, p);
                if max_x != -1 {
                    horizontal_wall(ctx, p, HWall, max_x);
                }
            }
            if dun(ctx, i, j) == VWall {
                ctx.rng.discard_random_values(1);
                let max_y = vertical_wall_ok(ctx, p);
                if max_y != -1 {
                    vertical_wall(ctx, p, VWall, max_y);
                }
            }
        }
    }
}

/// Original: `GenerateChamber` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::GenerateChamber(Point position, bool connectPrevious, bool connectNext, bool verticalLayout)
fn generate_chamber(ctx: &mut Ctx, mut position: Point, connect_previous: bool, connect_next: bool, vertical_layout: bool) {
    let (px, py) = (position.x, position.y);
    if connect_previous {
        if vertical_layout {
            set_dun(ctx, px + 2, py, HArch);
            set_dun(ctx, px + 3, py, HArch);
            set_dun(ctx, px + 4, py, Corner);
            set_dun(ctx, px + 7, py, VArchEnd);
            set_dun(ctx, px + 8, py, HArch);
            set_dun(ctx, px + 9, py, HWall);
        } else {
            set_dun(ctx, px, py + 2, VArch);
            set_dun(ctx, px, py + 3, VArch);
            set_dun(ctx, px, py + 4, Corner);
            set_dun(ctx, px, py + 7, HArchEnd);
            set_dun(ctx, px, py + 8, VArch);
            set_dun(ctx, px, py + 9, VWall);
        }
    }
    if connect_next {
        if vertical_layout {
            position.y += 11;
            let (px, py) = (position.x, position.y);
            set_dun(ctx, px + 2, py, HArchVWall);
            set_dun(ctx, px + 3, py, HArch);
            set_dun(ctx, px + 4, py, HArchEnd);
            set_dun(ctx, px + 7, py, DArch);
            set_dun(ctx, px + 8, py, HArch);
            if dun(ctx, px + 9, py) != DWall {
                set_dun(ctx, px + 9, py, HDirtCorner);
            }
            position.y -= 11;
        } else {
            position.x += 11;
            let (px, py) = (position.x, position.y);
            set_dun(ctx, px, py + 2, HWallVArch);
            set_dun(ctx, px, py + 3, VArch);
            set_dun(ctx, px, py + 4, VArchEnd);
            set_dun(ctx, px, py + 7, DArch);
            set_dun(ctx, px, py + 8, VArch);
            if dun(ctx, px, py + 9) != DWall {
                set_dun(ctx, px, py + 9, HDirtCorner);
            }
            position.x -= 11;
        }
    }
    for y in 1..11 {
        for x in 1..11 {
            set_dun(ctx, position.x + x, position.y + y, Floor);
            ctx.drlg_l1.Chamber.set(position.x + x, position.y + y);
        }
    }
    set_dun(ctx, position.x + 4, position.y + 4, Pillar);
    set_dun(ctx, position.x + 7, position.y + 4, Pillar);
    set_dun(ctx, position.x + 4, position.y + 7, Pillar);
    set_dun(ctx, position.x + 7, position.y + 7, Pillar);
}

/// Original: `GenerateHall` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::GenerateHall(Point start, int length, bool verticalLayout)
fn generate_hall(ctx: &mut Ctx, start: Point, length: i32, vertical_layout: bool) {
    if vertical_layout {
        for i in start.y..start.y + length {
            set_dun(ctx, start.x, i, VArch);
            set_dun(ctx, start.x + 3, i, VArch);
        }
    } else {
        for i in start.x..start.x + length {
            set_dun(ctx, i, start.y, HArch);
            set_dun(ctx, i, start.y + 3, HArch);
        }
    }
}

/// Original: `FixTilesPatterns` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::FixTilesPatterns()
fn fix_tiles_patterns(ctx: &mut Ctx) {
    // BUGFIX: Bounds checks are required in all loop bodies.
    // See https://github.com/diasurgical/devilutionX/pull/401
    let d = &mut ctx.gendung.dungeon;
    for j in 0..DMAXY {
        for i in 0..DMAXX {
            if i + 1 < DMAXX {
                if d[i][j] == HWall && d[i + 1][j] == Dirt {
                    d[i + 1][j] = DirtHwallEnd;
                }
                if d[i][j] == Floor && d[i + 1][j] == Dirt {
                    d[i + 1][j] = DirtHwall;
                }
                if d[i][j] == Floor && d[i + 1][j] == HWall {
                    d[i + 1][j] = HWallEnd;
                }
                if d[i][j] == VWallEnd && d[i + 1][j] == Dirt {
                    d[i + 1][j] = DirtVwallEnd;
                }
            }
            if j + 1 < DMAXY {
                if d[i][j] == VWall && d[i][j + 1] == Dirt {
                    d[i][j + 1] = DirtVwallEnd;
                }
                if d[i][j] == Floor && d[i][j + 1] == VWall {
                    d[i][j + 1] = VWallEnd;
                }
                if d[i][j] == Floor && d[i][j + 1] == Dirt {
                    d[i][j + 1] = DirtVwall;
                }
            }
        }
    }

    for j in 0..DMAXY {
        for i in 0..DMAXX {
            if i + 1 < DMAXX {
                let pairs: [(u8, u8, u8); 17] = [
                    (Floor, DirtVwall, HDirtCorner),
                    (Floor, Dirt, VDirtCorner),
                    (HWallEnd, Dirt, DirtHwallEnd),
                    (Floor, DirtVwallEnd, HDirtCorner),
                    (DirtVwall, Dirt, VDirtCorner),
                    (HWall, DirtVwall, HDirtCorner),
                    (DirtVwall, VWall, VWallEnd),
                    (HWallEnd, DirtVwall, HDirtCorner),
                    (HWall, VWall, VWallEnd),
                    (Corner, Dirt, DirtVwallEnd),
                    (HDirtCorner, VWall, VWallEnd),
                    (HWallEnd, VWall, VWallEnd),
                    (HWallEnd, DirtVwallEnd, HDirtCorner),
                    (DWall, VCorner, HCorner),
                    (HWallEnd, Floor, HCorner),
                    (HWall, DirtVwallEnd, HDirtCorner),
                    (HWall, Floor, HCorner),
                ];
                for &(a, b, c) in pairs.iter() {
                    if d[i][j] == a && d[i + 1][j] == b {
                        d[i + 1][j] = c;
                    }
                }
            }
            if i > 0 {
                if d[i][j] == DirtHwallEnd && d[i - 1][j] == Dirt {
                    d[i - 1][j] = DirtVwall;
                }
                if d[i][j] == DirtVwall && d[i - 1][j] == DirtHwallEnd {
                    d[i - 1][j] = HDirtCorner;
                }
                if d[i][j] == VWallEnd && d[i - 1][j] == Dirt {
                    d[i - 1][j] = DirtVwallEnd;
                }
                if d[i][j] == VWallEnd && d[i - 1][j] == DirtHwallEnd {
                    d[i - 1][j] = HDirtCorner;
                }
            }
            if j + 1 < DMAXY {
                let pairs: [(u8, u8, u8); 9] = [
                    (VWall, HWall, HWallEnd),
                    (VWallEnd, DirtHwall, HDirtCorner),
                    (DirtHwall, HWall, HWallEnd),
                    (VWallEnd, HWall, HWallEnd),
                    (HDirtCorner, HWall, HWallEnd),
                    (VWallEnd, Dirt, DirtVwallEnd),
                    (VWallEnd, Floor, VCorner),
                    (VWall, Floor, VCorner),
                    (Floor, VCorner, HCorner),
                ];
                for &(a, b, c) in pairs.iter() {
                    if d[i][j] == a && d[i][j + 1] == b {
                        d[i][j + 1] = c;
                    }
                }
            }
            if j > 0 {
                if d[i][j] == VWallEnd && d[i][j - 1] == Dirt {
                    d[i][j - 1] = HWallEnd;
                }
                if d[i][j] == VWallEnd && d[i][j - 1] == Dirt {
                    d[i][j - 1] = DirtVwallEnd;
                }
                if d[i][j] == HWallEnd && d[i][j - 1] == DirtVwallEnd {
                    d[i][j - 1] = HDirtCorner;
                }
                if d[i][j] == DirtHwall && d[i][j - 1] == DirtVwallEnd {
                    d[i][j - 1] = HDirtCorner;
                }
            }
        }
    }

    for j in 0..DMAXY {
        for i in 0..DMAXX {
            if j + 1 < DMAXY && d[i][j] == DWall && d[i][j + 1] == HWall {
                d[i][j + 1] = HWallEnd;
            }
            if i + 1 < DMAXX && d[i][j] == HWall && d[i + 1][j] == DirtVwall {
                d[i + 1][j] = HDirtCorner;
            }
            if j + 1 < DMAXY && d[i][j] == DirtHwall && d[i][j + 1] == Dirt {
                d[i][j + 1] = VDirtCorner;
            }
        }
    }
}

/// Original: `Substitution` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::Substitution()
fn substitution(ctx: &mut Ctx) {
    for y in 0..DY {
        for x in 0..DX {
            if ctx.rng.flip_coin(4) {
                let c = TILE_DECORATIONS[dun(ctx, x, y) as usize];
                if c != 0 && !ctx.gendung.Protected.test(x, y) {
                    let mut rv = rnd(ctx, 16);
                    let mut i: i32 = -1;
                    while rv >= 0 {
                        i += 1;
                        if i == TILE_DECORATIONS.len() as i32 {
                            i = 0;
                        }
                        if c == TILE_DECORATIONS[i as usize] {
                            rv -= 1;
                        }
                    }
                    // BUGFIX: Add `&& y > 0` to the if statement. (fixed)
                    if i == VWall4 as i32 && y > 0 {
                        if TILE_DECORATIONS[dun(ctx, x, y - 1) as usize] != VWall2 || ctx.gendung.Protected.test(x, y - 1) {
                            i = VWall2 as i32;
                        } else {
                            set_dun(ctx, x, y - 1, VWall5);
                        }
                    }
                    // BUGFIX: Add `&& x + 1 < DMAXX` to the if statement. (fixed)
                    if i == HWall4 as i32 && x + 1 < DX {
                        if TILE_DECORATIONS[dun(ctx, x + 1, y) as usize] != HWall2 || ctx.gendung.Protected.test(x + 1, y) {
                            i = HWall2 as i32;
                        } else {
                            set_dun(ctx, x + 1, y, HWall5);
                        }
                    }
                    set_dun(ctx, x, y, i as u8);
                }
            }
        }
    }
}

/// Original: `FillChambers` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::FillChambers()
fn fill_chambers(ctx: &mut Ctx) {
    let mut chamber1 = Point::new(0, 14);
    let mut chamber3 = Point::new(28, 14);
    let mut hall1 = Point::new(12, 18);
    let mut hall2 = Point::new(26, 18);
    let s = &ctx.drlg_l1;
    let (c1, c2, c3, vertical) = (s.HasChamber1, s.HasChamber2, s.HasChamber3, s.VerticalLayout);
    if vertical {
        std::mem::swap(&mut chamber1.x, &mut chamber1.y);
        std::mem::swap(&mut chamber3.x, &mut chamber3.y);
        std::mem::swap(&mut hall1.x, &mut hall1.y);
        std::mem::swap(&mut hall2.x, &mut hall2.y);
    }

    if c1 {
        generate_chamber(ctx, chamber1, false, true, vertical);
    }
    if c2 {
        generate_chamber(ctx, Point::new(14, 14), c1, c3, vertical);
    }
    if c3 {
        generate_chamber(ctx, chamber3, true, false, vertical);
    }

    if c2 {
        if c1 {
            generate_hall(ctx, hall1, 2, vertical);
        }
        if c3 {
            generate_hall(ctx, hall2, 2, vertical);
        }
    } else {
        generate_hall(ctx, hall1, 16, vertical);
    }

    if ctx.gendung.leveltype == DungeonType::Crypt {
        if ctx.gendung.currlevel == 24 {
            crate::levels::crypt::set_crypt_room(ctx);
        } else if crate::items::CornerStoneStruct::is_available(ctx) {
            crate::levels::crypt::set_corner_room(ctx);
        }
    } else if ctx.gendung.pSetPiece.is_some() {
        let c = select_chamber(ctx);
        set_set_piece_room(ctx, c, Floor as i32);
    }
}

/// Original: `FixTransparency` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::FixTransparency()
fn fix_transparency(ctx: &mut Ctx) {
    let g = &mut ctx.gendung;
    let mut yy = 16;
    for j in 0..DMAXY {
        let mut xx = 16;
        for i in 0..DMAXX {
            let t = &mut g.dTransVal;
            let v = t[xx][yy];
            let d = g.dungeon[i][j];
            // BUGFIX: Should check for `j > 0` first. (fixed)
            if d == DirtHwallEnd && j > 0 && g.dungeon[i][j - 1] == DirtHwall {
                t[xx + 1][yy] = v;
                t[xx + 1][yy + 1] = v;
            }
            // BUGFIX: Should check for `i + 1 < DMAXY` first. (fixed)
            if d == DirtVwallEnd && i + 1 < DMAXY && g.dungeon[i + 1][j] == DirtVwall {
                t[xx][yy + 1] = v;
                t[xx + 1][yy + 1] = v;
            }
            if d == DirtHwall {
                t[xx + 1][yy] = v;
                t[xx + 1][yy + 1] = v;
            }
            if d == DirtVwall {
                t[xx][yy + 1] = v;
                t[xx + 1][yy + 1] = v;
            }
            if d == VDirtCorner {
                t[xx + 1][yy] = v;
                t[xx][yy + 1] = v;
                t[xx + 1][yy + 1] = v;
            }
            xx += 2;
        }
        yy += 2;
    }
}

/// Original: `FixDirtTiles` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::FixDirtTiles()
fn fix_dirt_tiles(ctx: &mut Ctx) {
    let d = &mut ctx.gendung.dungeon;
    for j in 0..DMAXY - 1 {
        for i in 0..DMAXX - 1 {
            if d[i][j] == HDirtCorner && d[i + 1][j] != DirtVwall {
                d[i][j] = DirtCorner2;
            }
            if d[i][j] == DirtVwall && d[i + 1][j] != DirtVwall {
                d[i][j] = DirtVWall2;
            }
            if d[i][j] == DirtVwallEnd && d[i + 1][j] != DirtVwall {
                d[i][j] = DirtVWallEnd2;
            }
            if d[i][j] == DirtHwall && d[i][j + 1] != DirtHwall {
                d[i][j] = DirtHWall2;
            }
            if d[i][j] == HDirtCorner && d[i][j + 1] != DirtHwall {
                d[i][j] = DirtCorner2;
            }
            if d[i][j] == DirtHwallEnd && d[i][j + 1] != DirtHwall {
                d[i][j] = DirtHWallEnd2;
            }
        }
    }
}

/// Original: `FixCornerTiles` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::FixCornerTiles()
fn fix_corner_tiles(ctx: &mut Ctx) {
    let g = &mut ctx.gendung;
    for j in 1..DMAXY - 1 {
        for i in 1..DMAXX - 1 {
            let d = &mut g.dungeon;
            if !g.Protected.test(i as i32, j as i32) && d[i][j] == HCorner && d[i - 1][j] == Floor && d[i][j - 1] == VWall {
                d[i][j] = VCorner;
                // BUGFIX: Set tile as Protected
            }
            if d[i][j] == DirtCorner2 && d[i + 1][j] == Floor && d[i][j + 1] == VWall {
                d[i][j] = HArchEnd;
            }
            if d[i][j] == DirtCorner2 && d[i][j + 1] == Floor && d[i + 1][j] == HWall {
                d[i][j] = VArchEnd;
            }
        }
    }
}

/// Original: `PlaceCathedralStairs` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::PlaceCathedralStairs(lvl_entry entry)
fn place_cathedral_stairs(ctx: &mut Ctx, entry: lvl_entry) -> bool {
    use crate::quests::is_quest_available;
    let mut success = true;
    let tries = DX * DY;

    // Place poison water entrance
    if is_quest_available(ctx, Q_PWATER) {
        match place_mini_set(ctx, &pwaterin(), tries, true) {
            None => success = false,
            Some(mini_position) => {
                let t = ctx.gendung.TransVal;
                ctx.gendung.TransVal = 0;
                drlg_m_rect_trans(ctx, Rectangle::new(mini_position + Displacement::new(0, 2), Size::new(5, 2)));
                ctx.gendung.TransVal = t;
                ctx.quests.Quests[Q_PWATER as usize].position = mini_position.mega_to_world() + Displacement::new(5, 6);
                if entry == ENTRY_RTNLVL {
                    ctx.gendung.ViewPosition = ctx.quests.Quests[Q_PWATER as usize].position;
                }
            }
        }
    }

    // Place stairs up
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let original = ctx.players.Players[me].pOriginalCathedral;
    let stairs = if original && !is_quest_available(ctx, Q_LTBANNER) { crate::levels::crypt::l5_stairs_up() } else { stairsup() };
    match place_mini_set(ctx, &stairs, tries, true) {
        None => {
            if original {
                return false;
            }
            success = false;
        }
        Some(position) => {
            if entry == ENTRY_MAIN {
                ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(3, 4);
            }
        }
    }

    // Place stairs down
    if is_quest_available(ctx, Q_LTBANNER) {
        if entry == ENTRY_PREV {
            ctx.gendung.ViewPosition = ctx.gendung.SetPiece.position.mega_to_world() + Displacement::new(3, 11);
        }
    } else {
        match place_mini_set(ctx, &stairsdown(), tries, true) {
            None => success = false,
            Some(position) => {
                if entry == ENTRY_PREV {
                    ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(3, 3);
                }
            }
        }
    }
    success
}

/// Original: `PlaceStairs` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::PlaceStairs(lvl_entry entry)
fn place_stairs(ctx: &mut Ctx, entry: lvl_entry) -> bool {
    if ctx.gendung.leveltype == DungeonType::Crypt {
        return crate::levels::crypt::place_crypt_stairs(ctx, entry);
    }
    place_cathedral_stairs(ctx, entry)
}

/// Original: `GenerateLevel` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::GenerateLevel(lvl_entry entry)
fn generate_level(ctx: &mut Ctx, entry: lvl_entry) {
    let minarea: usize = match ctx.gendung.currlevel {
        1 => 533,
        2 => 693,
        _ => 761,
    };

    load_quest_set_pieces(ctx);

    loop {
        drlg_init_trans(ctx);
        loop {
            first_room(ctx);
            if find_area(ctx) >= minarea {
                break;
            }
        }
        init_dungeon_flags(ctx);
        make_dmt(ctx);
        fill_chambers(ctx);
        fix_tiles_patterns(ctx);
        add_wall(ctx);
        flood_transparency_values(ctx, 13);
        if place_stairs(ctx, entry) {
            break;
        }
    }

    free_quest_set_pieces(ctx);

    for j in 0..DY {
        for i in 0..DX {
            if dun(ctx, i, j) == EntranceStairs {
                let xx = 2 * i + 16; /* todo: fix loop */
                let yy = 2 * j + 16;
                drlg_copy_trans(ctx, xx, yy + 1, xx, yy);
                drlg_copy_trans(ctx, xx + 1, yy + 1, xx + 1, yy);
            }
        }
    }

    fix_transparency(ctx);
    if ctx.gendung.leveltype == DungeonType::Crypt {
        crate::levels::crypt::fix_crypt_dirt_tiles(ctx);
    } else {
        fix_dirt_tiles(ctx);
    }
    fix_corner_tiles(ctx);

    if ctx.gendung.leveltype == DungeonType::Crypt {
        crate::levels::crypt::crypt_substitution(ctx);
    } else {
        substitution(ctx);
        apply_shadows_patterns(ctx);

        let numt = rnd(ctx, 5) + 5;
        let l = lamps();
        for _ in 0..numt {
            place_mini_set(ctx, &l, DX * DY, true);
        }

        fill_floor(ctx);
    }

    ctx.gendung.pdungeon = ctx.gendung.dungeon;

    let p = ctx.gendung.SetPiece.position;
    crate::quests::drlg_check_quests(ctx, p);
}

/// Original: `Pass3` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::Pass3()
fn pass3(ctx: &mut Ctx) {
    drlg_l_pass3(ctx, Dirt as i32 - 1);
    if ctx.gendung.leveltype == DungeonType::Crypt {
        crate::levels::crypt::init_crypt_pieces(ctx);
    } else {
        init_dungeon_pieces(ctx);
    }
}

/// Original: `devilution::PlaceMiniSetRandom` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::PlaceMiniSetRandom(const Miniset &miniset, int rndper)
pub fn place_mini_set_random(ctx: &mut Ctx, miniset: &Miniset, rndper: i32) {
    let sw = miniset.size.width;
    let sh = miniset.size.height;
    for sy in 0..DY - sh {
        for sx in 0..DX - sw {
            if !miniset.matches(&ctx.gendung, Point::new(sx, sy), false) {
                continue;
            }
            // BUGFIX: This code is copied from Cave and should not be applied for crypt
            if !can_replace_tile(ctx, miniset.replace[0][0], Point::new(sx, sy)) {
                continue;
            }
            if rnd(ctx, 100) >= rndper {
                continue;
            }
            miniset.place(&mut ctx.gendung, Point::new(sx, sy), false);
        }
    }
}

/// Original: `devilution::SelectChamber` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::SelectChamber()
pub fn select_chamber(ctx: &mut Ctx) -> Point {
    let s = &ctx.drlg_l1;
    let (c1, c2, c3, vertical) = (s.HasChamber1, s.HasChamber2, s.HasChamber3, s.VerticalLayout);
    let chamber = if c1 && c2 && c3 {
        rnd(ctx, 3) + 1
    } else if c1 && c2 {
        ctx.rng.pick_randomly_among(&[2, 1]) // Reverse order to match vanilla
    } else if c1 && c3 {
        ctx.rng.pick_randomly_among(&[3, 1]) // Reverse order to match vanilla
    } else if c2 && c3 {
        ctx.rng.pick_randomly_among(&[2, 3])
    } else {
        // The dungeon generation logic ensures that chamber 2 is available if
        // either (or both of) 1 or 3 aren't, so if we ever end up with a single
        // chamber layout it's always chamber 2.
        2
    };
    match chamber {
        1 => {
            if vertical {
                Point::new(16, 2)
            } else {
                Point::new(2, 16)
            }
        }
        3 => {
            if vertical {
                Point::new(16, 30)
            } else {
                Point::new(30, 16)
            }
        }
        _ => Point::new(16, 16),
    }
}

/// Original: `devilution::CreateL5Dungeon` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::CreateL5Dungeon(uint32_t rseed, lvl_entry entry)
pub fn create_l5_dungeon(ctx: &mut Ctx, rseed: u32, entry: lvl_entry) {
    ctx.rng.set_rnd_seed(rseed);
    ctx.crypt.UberRow = 0;
    ctx.crypt.UberCol = 0;
    generate_level(ctx, entry);
    pass3(ctx);
    if ctx.gendung.leveltype == DungeonType::Crypt {
        crate::levels::crypt::place_crypt_lights(ctx);
        crate::levels::crypt::set_crypt_set_piece_room(ctx);
    }
}

/// Original: `devilution::LoadPreL1Dungeon` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::LoadPreL1Dungeon(const char *path)
pub fn load_pre_l1_dungeon(ctx: &mut Ctx, path: &str) {
    init_dungeon_flags(ctx);
    let dun_data = load_u16_file(ctx, path);
    place_dun_tiles(ctx, &dun_data, Point::new(0, 0), Floor as i32);
    if ctx.gendung.setlvltype == DungeonType::Cathedral {
        fill_floor(ctx);
    }
    ctx.gendung.pdungeon = ctx.gendung.dungeon;
}

/// Original: `devilution::LoadL1Dungeon` (levels/drlg_l1.cpp).
// @port levels/drlg_l1.cpp|devilution::LoadL1Dungeon(const char *path, Point spawn)
pub fn load_l1_dungeon(ctx: &mut Ctx, path: &str, spawn: Point) {
    load_dungeon_base(ctx, path, spawn, Floor as i32, Dirt as i32);
    if ctx.gendung.setlvltype == DungeonType::Cathedral {
        fill_floor(ctx);
    }
    pass3(ctx);
    if ctx.gendung.setlvltype == DungeonType::Crypt {
        crate::objects::add_crypt_objects(ctx, 0, 0, MAXDUNX as i32, MAXDUNY as i32);
        crate::levels::crypt::place_crypt_lights(ctx);
    } else {
        crate::objects::add_l1_objs(ctx, 0, 0, MAXDUNX as i32, MAXDUNY as i32);
    }
}
