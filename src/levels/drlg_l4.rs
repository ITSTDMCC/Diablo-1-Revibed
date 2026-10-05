//! `Source/levels/drlg_l4.cpp`: hell level generation.
//!
//! Unchecked one-past-the-edge `dungeon` accesses of the original go through
//! `gendung::dungeon_flat`/`set_dungeon_flat` (see there). Room coordinates use `u8` arithmetic
//! like the original's `WorldTilePosition`.

use crate::ctx::Ctx;
use crate::engine::geometry::{Displacement, Point, Rectangle, Size};
use crate::enums::*;
use crate::levels::drlg_l4_data::*;
use crate::levels::gendung::*;

/// Globals of levels/drlg_l4.cpp.
#[derive(Default)]
pub struct DrlgL4State {
    /// `DiabloQuad1`..`DiabloQuad4`
    pub diablo_quads: [Point; 4],
    /// `hallok`
    hallok: [bool; 20],
    /// `L4Hold`
    l4_hold: (u8, u8),
}

const DX: i32 = DMAXX as i32;
const DY: i32 = DMAXY as i32;

fn rnd(ctx: &mut Ctx, v: i32) -> i32 {
    ctx.rng.generate_rnd(v)
}

fn d(ctx: &Ctx, x: i32, y: i32) -> u8 {
    dungeon_flat(ctx, x, y)
}

fn sd(ctx: &mut Ctx, x: i32, y: i32, v: u8) {
    set_dungeon_flat(ctx, x, y, v);
}

/// `WorldTileRectangle`
#[derive(Clone, Copy, Default)]
struct WRect {
    pos: (u8, u8),
    size: (u8, u8),
}

/// Original: `ApplyShadowsPatterns` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::ApplyShadowsPatterns()
fn apply_shadows_patterns(ctx: &mut Ctx) {
    for y in 1..DY {
        for x in 1..DY {
            if !matches!(d(ctx, x, y), 3 | 4 | 8 | 15) {
                continue;
            }
            if d(ctx, x - 1, y) == 6 {
                sd(ctx, x - 1, y, 47);
            }
            if d(ctx, x - 1, y - 1) == 6 {
                sd(ctx, x - 1, y - 1, 48);
            }
        }
    }
}

/// Original: `LoadQuestSetPieces` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::LoadQuestSetPieces()
fn load_quest_set_pieces(ctx: &mut Ctx) {
    if crate::quests::is_quest_available(ctx, Q_WARLORD) {
        ctx.gendung.pSetPiece = Some(load_u16_file(ctx, "levels\\l4data\\warlord.dun"));
    } else if ctx.gendung.currlevel == 15 && crate::quests::use_multiplayer_quests(ctx) {
        ctx.gendung.pSetPiece = Some(load_u16_file(ctx, "levels\\l4data\\vile1.dun"));
    }
}

/// Original: `InitDungeonFlags` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::InitDungeonFlags()
fn init_dungeon_flags(ctx: &mut Ctx) {
    ctx.gendung.DungeonMask.reset();
    ctx.gendung.Protected.reset();
    ctx.gendung.dungeon = [[30; DMAXY]; DMAXX];
}

/// Original: `MapRoom` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::MapRoom(WorldTileRectangle room)
fn map_room(ctx: &mut Ctx, room: WRect) {
    let mut y: i32 = 0;
    while y < room.size.1 as i32 && y + (room.pos.1 as i32) < DY / 2 {
        let mut x: i32 = 0;
        while x < room.size.0 as i32 && x + (room.pos.0 as i32) < DX / 2 {
            ctx.gendung.DungeonMask.set(room.pos.0 as i32 + x, room.pos.1 as i32 + y);
            x += 1;
        }
        y += 1;
    }
}

/// Original: `CheckRoom` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::CheckRoom(WorldTileRectangle room)
fn check_room(ctx: &Ctx, room: WRect) -> bool {
    if room.pos.0 == 0 || room.pos.1 == 0 {
        return false;
    }
    for y in 0..room.size.1 as i32 {
        for x in 0..room.size.0 as i32 {
            let (px, py) = (x + room.pos.0 as i32, y + room.pos.1 as i32);
            if px < 0 || px >= DX / 2 || py < 0 || py >= DY / 2 {
                return false;
            }
            if ctx.gendung.DungeonMask.test(px, py) {
                return false;
            }
        }
    }
    true
}

fn wadd(p: (u8, u8), dx: i32, dy: i32) -> (u8, u8) {
    // WorldTileDisplacement is int8_t
    (p.0.wrapping_add(dx as i8 as u8), p.1.wrapping_add(dy as i8 as u8))
}

/// Original: `GenerateRoom` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::GenerateRoom(WorldTileRectangle area, bool verticalLayout)
fn generate_room(ctx: &mut Ctx, area: WRect, mut vertical_layout: bool) {
    let rotate = !ctx.rng.flip_coin(4);
    vertical_layout = (!vertical_layout && rotate) || (vertical_layout && !rotate);
    let mut place_room1 = false;
    let mut room1 = WRect::default();
    for _ in 0..20 {
        let random_width = (rnd(ctx, 5) + 2) & !1;
        let random_height = (rnd(ctx, 5) + 2) & !1;
        room1.size = (random_width as u8, random_height as u8);
        room1.pos = area.pos;
        if vertical_layout {
            room1.pos = wadd(room1.pos, -(room1.size.0 as i32), area.size.1 as i32 / 2 - room1.size.1 as i32 / 2);
            // BUGFIX: swap height and width ({ room1.size.width + 1, room1.size.height + 2 }) (workaround applied below)
            place_room1 = check_room(ctx, WRect { pos: wadd(room1.pos, -1, -1), size: (room1.size.1.wrapping_add(2), room1.size.0.wrapping_add(1)) });
        } else {
            room1.pos = wadd(room1.pos, area.size.0 as i32 / 2 - room1.size.0 as i32 / 2, -(room1.size.1 as i32));
            place_room1 = check_room(ctx, WRect { pos: wadd(room1.pos, -1, -1), size: (room1.size.0.wrapping_add(2), room1.size.1.wrapping_add(1)) });
        }
        if place_room1 {
            break;
        }
    }
    if place_room1 {
        let w = (DX - room1.pos.0 as i32).min(room1.size.0 as i32) as u8;
        let h = (DX - room1.pos.1 as i32).min(room1.size.1 as i32) as u8;
        map_room(ctx, WRect { pos: room1.pos, size: (w, h) });
    }
    let place_room2;
    let mut room2 = room1;
    if vertical_layout {
        room2.pos.0 = area.pos.0.wrapping_add(area.size.0);
        place_room2 = check_room(ctx, WRect { pos: wadd(room2.pos, 0, -1), size: (room2.size.0.wrapping_add(1), room2.size.1.wrapping_add(2)) });
    } else {
        room2.pos.1 = area.pos.1.wrapping_add(area.size.1);
        place_room2 = check_room(ctx, WRect { pos: wadd(room2.pos, -1, 0), size: (room2.size.0.wrapping_add(2), room2.size.1.wrapping_add(1)) });
    }
    if place_room2 {
        map_room(ctx, room2);
    }
    if place_room1 {
        generate_room(ctx, room1, vertical_layout);
    }
    if place_room2 {
        generate_room(ctx, room2, vertical_layout);
    }
}

/// Original: `FirstRoom` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::FirstRoom()
fn first_room(ctx: &mut Ctx) {
    let mut room = WRect { pos: (0, 0), size: (14, 14) };
    let currlevel = ctx.gendung.currlevel;
    let mpq = crate::quests::use_multiplayer_quests(ctx);
    if currlevel != 16 {
        let q = &ctx.quests.Quests;
        if currlevel == q[Q_WARLORD as usize]._qlevel && q[Q_WARLORD as usize]._qactive != QUEST_NOTAVAIL {
            room.size = (11, 11);
        } else if currlevel == q[Q_BETRAYER as usize]._qlevel && mpq {
            room.size = (11, 11);
        } else {
            let random_width = rnd(ctx, 5) + 2;
            let random_height = rnd(ctx, 5) + 2;
            room.size = (random_width as u8, random_height as u8);
        }
    }
    let xmin = (DX / 2 - room.size.0 as i32) / 2;
    let xmax = DX / 2 - 1 - room.size.0 as i32;
    let ymin = (DY / 2 - room.size.1 as i32) / 2;
    let ymax = DY / 2 - 1 - room.size.1 as i32;
    let random_x = rnd(ctx, xmax - xmin + 1) + xmin;
    let random_y = rnd(ctx, ymax - ymin + 1) + ymin;
    room.pos = (random_x as u8, random_y as u8);
    if currlevel == 16 {
        ctx.drlg_l4.l4_hold = room.pos;
    }
    if crate::quests::is_quest_available(ctx, Q_WARLORD) || (currlevel == ctx.quests.Quests[Q_BETRAYER as usize]._qlevel && mpq) {
        let p = wadd(room.pos, 1, 1);
        ctx.gendung.SetPieceRoom = Rectangle::new(Point::new(p.0 as i32, p.1 as i32), Size::new(room.size.0.wrapping_add(1) as i32, room.size.1.wrapping_add(1) as i32));
    } else {
        ctx.gendung.SetPieceRoom = Rectangle::default();
    }
    map_room(ctx, room);
    let v = !ctx.rng.flip_coin(2);
    generate_room(ctx, room, v);
}

/// Original: `MirrorDungeonLayout` (levels/drlg_l4.cpp): mirrors the first quadrant.
// @port levels/drlg_l4.cpp|devilution::MirrorDungeonLayout()
fn mirror_dungeon_layout(ctx: &mut Ctx) {
    let m = &mut ctx.gendung.DungeonMask;
    for y in 0..DY / 2 {
        for x in 0..DX / 2 {
            if m.test(x, y) {
                m.set(x, DY - 1 - y);
                m.set(DX - 1 - x, y);
                m.set(DX - 1 - x, DY - 1 - y);
            }
        }
    }
}

/// Original: `MakeDmt` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::MakeDmt()
fn make_dmt(ctx: &mut Ctx) {
    for y in 0..DY - 1 {
        for x in 0..DX - 1 {
            let m = &ctx.gendung.DungeonMask;
            let val = ((m.test(x + 1, y + 1) as usize) << 3) | ((m.test(x, y + 1) as usize) << 2) | ((m.test(x + 1, y) as usize) << 1) | (m.test(x, y) as usize);
            ctx.gendung.dungeon[x as usize][y as usize] = L4ConvTbl[val];
        }
    }
}

/// Original: `HorizontalWallOk` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::HorizontalWallOk(int i, int j)
fn horizontal_wall_ok(ctx: &Ctx, i: i32, j: i32) -> i32 {
    let mut x = 1;
    while d(ctx, i + x, j) == 6 {
        if ctx.gendung.Protected.test(i + x, j) {
            break;
        }
        if d(ctx, i + x, j - 1) != 6 {
            break;
        }
        if d(ctx, i + x, j + 1) != 6 {
            break;
        }
        x += 1;
    }
    if matches!(d(ctx, i + x, j), 10 | 12 | 13 | 15 | 16 | 21 | 22) && x > 3 {
        return x;
    }
    -1
}

/// Original: `VerticalWallOk` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::VerticalWallOk(int i, int j)
fn vertical_wall_ok(ctx: &Ctx, i: i32, j: i32) -> i32 {
    let mut y = 1;
    while d(ctx, i, j + y) == 6 {
        if ctx.gendung.Protected.test(i, j + y) {
            break;
        }
        if d(ctx, i - 1, j + y) != 6 {
            break;
        }
        if d(ctx, i + 1, j + y) != 6 {
            break;
        }
        y += 1;
    }
    if matches!(d(ctx, i, j + y), 8 | 9 | 11 | 14 | 15 | 16 | 21 | 23) && y > 3 {
        return y;
    }
    -1
}

/// Original: `HorizontalWall` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::HorizontalWall(int i, int j, int dx)
fn horizontal_wall(ctx: &mut Ctx, i: i32, j: i32, dx: i32) {
    match d(ctx, i, j) {
        13 => sd(ctx, i, j, 17),
        16 => sd(ctx, i, j, 11),
        12 => sd(ctx, i, j, 14),
        _ => {}
    }
    for xx in 1..dx {
        sd(ctx, i + xx, j, 2);
    }
    match d(ctx, i + dx, j) {
        15 => sd(ctx, i + dx, j, 14),
        10 => sd(ctx, i + dx, j, 17),
        21 => sd(ctx, i + dx, j, 23),
        22 => sd(ctx, i + dx, j, 29),
        _ => {}
    }
    let xx = rnd(ctx, dx - 3) + 1;
    sd(ctx, i + xx, j, 57);
    sd(ctx, i + xx + 2, j, 56);
    sd(ctx, i + xx + 1, j, 60);
    if d(ctx, i + xx, j - 1) == 6 {
        sd(ctx, i + xx, j - 1, 58);
    }
    if d(ctx, i + xx + 1, j - 1) == 6 {
        sd(ctx, i + xx + 1, j - 1, 59);
    }
}

/// Original: `VerticalWall` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::VerticalWall(int i, int j, int dy)
fn vertical_wall(ctx: &mut Ctx, i: i32, j: i32, dy: i32) {
    match d(ctx, i, j) {
        14 => sd(ctx, i, j, 17),
        8 => sd(ctx, i, j, 9),
        15 => sd(ctx, i, j, 10),
        _ => {}
    }
    for yy in 1..dy {
        sd(ctx, i, j + yy, 1);
    }
    match d(ctx, i, j + dy) {
        11 => sd(ctx, i, j + dy, 17),
        9 => sd(ctx, i, j + dy, 10),
        16 => sd(ctx, i, j + dy, 13),
        21 => sd(ctx, i, j + dy, 22),
        23 => sd(ctx, i, j + dy, 29),
        _ => {}
    }
    let yy = rnd(ctx, dy - 3) + 1;
    sd(ctx, i, j + yy, 53);
    sd(ctx, i, j + yy + 2, 52);
    sd(ctx, i, j + yy + 1, 6);
    if d(ctx, i - 1, j + yy) == 6 {
        sd(ctx, i - 1, j + yy, 54);
    }
    if d(ctx, i - 1, j + yy - 1) == 6 {
        sd(ctx, i - 1, j + yy - 1, 55);
    }
}

/// Original: `AddWall` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::AddWall()
fn add_wall(ctx: &mut Ctx) {
    for j in 0..DY {
        for i in 0..DX {
            if ctx.gendung.Protected.test(i, j) {
                continue;
            }
            for dv in [10, 12, 13, 15, 16, 21, 22] {
                if dv == d(ctx, i, j) {
                    ctx.rng.discard_random_values(1);
                    let x = horizontal_wall_ok(ctx, i, j);
                    if x != -1 {
                        horizontal_wall(ctx, i, j, x);
                    }
                }
            }
            for dv in [8, 9, 11, 14, 15, 16, 21, 23] {
                if dv == d(ctx, i, j) {
                    ctx.rng.discard_random_values(1);
                    let y = vertical_wall_ok(ctx, i, j);
                    if y != -1 {
                        vertical_wall(ctx, i, j, y);
                    }
                }
            }
        }
    }
}

/// A `FixTilesPatterns` rule: if the tile and the listed neighbours have these values, set the
/// target neighbour.
struct Rule {
    tile: u8,
    checks: &'static [(i32, i32, u8)],
    target: (i32, i32),
    value: u8,
}

const fn r(tile: u8, checks: &'static [(i32, i32, u8)], target: (i32, i32), value: u8) -> Rule {
    Rule { tile, checks, target, value }
}

fn apply_rules(ctx: &mut Ctx, rules: &[Rule]) {
    for j in 0..DY {
        for i in 0..DX {
            for rule in rules {
                if d(ctx, i, j) == rule.tile && rule.checks.iter().all(|&(dx, dy, v)| d(ctx, i + dx, j + dy) == v) {
                    sd(ctx, i + rule.target.0, j + rule.target.1, rule.value);
                }
            }
        }
    }
}

/// The first pass of `FixTilesPatterns`, in source order.
const PASS1: [Rule; 3] = [r(2, &[(1, 0, 6)], (1, 0), 5), r(2, &[(1, 0, 1)], (1, 0), 13), r(1, &[(0, 1, 2)], (0, 1), 14)];

const PASS2: [Rule; 8] = [
    r(2, &[(1, 0, 6)], (1, 0), 2),
    r(2, &[(1, 0, 9)], (1, 0), 11),
    r(9, &[(1, 0, 6)], (1, 0), 12),
    r(14, &[(1, 0, 1)], (1, 0), 13),
    r(6, &[(1, 0, 14)], (1, 0), 15),
    r(6, &[(0, 1, 13)], (0, 1), 16),
    r(1, &[(0, 1, 9)], (0, 1), 10),
    r(6, &[(0, -1, 1)], (0, -1), 1),
];

/// Original's `/* check */` rule: `dungeon[i + 1][j + 1] != 0` is an inequality, handled apart.
const PASS3: [Rule; 76] = [
    r(13, &[(0, 1, 30)], (0, 1), 27),
    r(27, &[(1, 0, 30)], (1, 0), 19),
    r(1, &[(0, 1, 30)], (0, 1), 27),
    r(27, &[(1, 0, 1)], (1, 0), 16),
    r(19, &[(1, 0, 27)], (1, 0), 26),
    r(27, &[(1, 0, 30)], (1, 0), 19),
    r(2, &[(1, 0, 15)], (1, 0), 14),
    r(14, &[(1, 0, 15)], (1, 0), 14),
    r(22, &[(1, 0, 1)], (1, 0), 16),
    r(27, &[(1, 0, 1)], (1, 0), 16),
    // index 10: the `/* check */` rule (see fix_tiles_patterns)
    r(0, &[], (0, 0), 0),
    r(22, &[(1, 0, 30)], (1, 0), 19),
    r(21, &[(1, 0, 1), (1, -1, 1)], (1, 0), 13),
    r(14, &[(1, 0, 30), (0, 1, 6)], (1, 0), 28),
    r(16, &[(1, 0, 6), (0, 1, 30)], (0, 1), 27),
    r(16, &[(0, 1, 30), (1, 1, 30)], (0, 1), 27),
    r(6, &[(1, 0, 30), (1, -1, 6)], (1, 0), 21),
    r(2, &[(1, 0, 27), (1, 1, 9)], (1, 0), 29),
    r(9, &[(1, 0, 15)], (1, 0), 14),
    r(15, &[(1, 0, 27), (1, 1, 2)], (1, 0), 29),
    r(19, &[(1, 0, 18)], (1, 0), 24),
    r(9, &[(1, 0, 15)], (1, 0), 14),
    r(19, &[(1, 0, 19), (1, -1, 30)], (1, 0), 24),
    r(24, &[(0, -1, 30), (0, -2, 6)], (0, -1), 21),
    r(2, &[(1, 0, 30)], (1, 0), 28),
    r(15, &[(1, 0, 30)], (1, 0), 28),
    r(28, &[(0, 1, 30)], (0, 1), 18),
    r(28, &[(0, 1, 2)], (0, 1), 15),
    r(19, &[(2, 0, 2), (1, -1, 18), (1, 1, 1)], (1, 0), 17),
    r(19, &[(2, 0, 2), (1, -1, 22), (1, 1, 1)], (1, 0), 17),
    r(19, &[(2, 0, 2), (1, -1, 18), (1, 1, 13)], (1, 0), 17),
    r(21, &[(2, 0, 2), (1, -1, 18), (1, 1, 1)], (1, 0), 17),
    r(21, &[(1, 1, 1), (1, -1, 22), (2, 0, 3)], (1, 0), 17),
    r(15, &[(1, 0, 28), (2, 0, 30), (1, -1, 6)], (1, 0), 23),
    r(14, &[(1, 0, 28), (2, 0, 1)], (1, 0), 23),
    r(15, &[(1, 0, 27), (1, 1, 30)], (1, 0), 29),
    r(28, &[(0, 1, 9)], (0, 1), 15),
    r(21, &[(1, -1, 21)], (1, 0), 24),
    r(2, &[(1, 0, 27), (1, 1, 30)], (1, 0), 29),
    r(2, &[(1, 0, 18)], (1, 0), 25),
    r(21, &[(1, 0, 9), (2, 0, 2)], (1, 0), 11),
    r(19, &[(1, 0, 10)], (1, 0), 17),
    r(15, &[(0, 1, 3)], (0, 1), 4),
    r(22, &[(0, 1, 9)], (0, 1), 15),
    r(18, &[(0, 1, 30)], (0, 1), 18),
    r(24, &[(-1, 0, 30)], (-1, 0), 19),
    r(21, &[(0, 1, 2)], (0, 1), 15),
    r(21, &[(0, 1, 9)], (0, 1), 10),
    r(22, &[(0, 1, 30)], (0, 1), 18),
    r(21, &[(0, 1, 30)], (0, 1), 18),
    r(16, &[(0, 1, 2)], (0, 1), 15),
    r(13, &[(0, 1, 2)], (0, 1), 15),
    r(22, &[(0, 1, 2)], (0, 1), 15),
    r(21, &[(1, 0, 18), (2, 0, 30)], (1, 0), 24),
    r(21, &[(1, 0, 9), (1, 1, 1)], (1, 0), 16),
    r(2, &[(1, 0, 27), (1, 1, 2)], (1, 0), 29),
    r(23, &[(0, 1, 2)], (0, 1), 15),
    r(23, &[(0, 1, 9)], (0, 1), 15),
    r(25, &[(0, 1, 2)], (0, 1), 15),
    r(22, &[(1, 0, 9)], (1, 0), 11),
    r(23, &[(1, 0, 9)], (1, 0), 11),
    r(15, &[(1, 0, 1)], (1, 0), 16),
    r(11, &[(1, 0, 15)], (1, 0), 14),
    r(23, &[(1, 0, 1)], (1, 0), 16),
    r(21, &[(1, 0, 27)], (1, 0), 26),
    r(21, &[(1, 0, 18)], (1, 0), 24),
    r(26, &[(1, 0, 1)], (1, 0), 16),
    r(29, &[(1, 0, 1)], (1, 0), 16),
    r(29, &[(0, 1, 2)], (0, 1), 15),
    r(1, &[(0, -1, 15)], (0, -1), 10),
    r(18, &[(0, 1, 2)], (0, 1), 15),
    r(23, &[(0, 1, 30)], (0, 1), 18),
    r(18, &[(0, 1, 9)], (0, 1), 10),
    r(14, &[(1, 0, 30), (1, 1, 30)], (1, 0), 23),
    r(2, &[(1, 0, 28), (1, -1, 6)], (1, 0), 23),
    r(23, &[(1, 0, 18), (0, -1, 6)], (1, 0), 24),
];

const PASS3B: [Rule; 19] = [
    r(14, &[(1, 0, 23), (2, 0, 30)], (1, 0), 28),
    r(14, &[(1, 0, 28), (2, 0, 30), (1, -1, 6)], (1, 0), 23),
    r(23, &[(1, 0, 30)], (1, 0), 19),
    r(29, &[(1, 0, 30)], (1, 0), 19),
    r(29, &[(0, 1, 30)], (0, 1), 18),
    r(19, &[(1, 0, 30)], (1, 0), 19),
    r(21, &[(1, 0, 30)], (1, 0), 19),
    r(26, &[(1, 0, 30)], (1, 0), 19),
    r(16, &[(0, 1, 30)], (0, 1), 18),
    r(13, &[(0, 1, 9)], (0, 1), 10),
    r(25, &[(0, 1, 30)], (0, 1), 18),
    r(18, &[(0, 1, 2)], (0, 1), 15),
    r(11, &[(1, 0, 3)], (1, 0), 5),
    r(19, &[(1, 0, 9)], (1, 0), 11),
    r(19, &[(1, 0, 1)], (1, 0), 13),
    r(19, &[(1, 0, 13), (1, -1, 6)], (1, 0), 16),
    r(0, &[], (0, 0), 0),
    r(0, &[], (0, 0), 0),
    r(0, &[], (0, 0), 0),
];

const PASS4: [Rule; 35] = [
    r(21, &[(0, 1, 24), (0, 2, 1)], (0, 1), 17),
    r(15, &[(1, 1, 9), (1, -1, 1), (2, 0, 16)], (1, 0), 29),
    r(2, &[(-1, 0, 6)], (-1, 0), 8),
    r(1, &[(0, -1, 6)], (0, -1), 7),
    r(6, &[(1, 0, 15), (1, 1, 4)], (1, 0), 10),
    r(1, &[(0, 1, 3)], (0, 1), 4),
    r(1, &[(0, 1, 6)], (0, 1), 4),
    r(9, &[(0, 1, 3)], (0, 1), 4),
    r(10, &[(0, 1, 3)], (0, 1), 4),
    r(13, &[(0, 1, 3)], (0, 1), 4),
    r(1, &[(0, 1, 5)], (0, 1), 12),
    r(1, &[(0, 1, 16)], (0, 1), 13),
    r(6, &[(0, 1, 13)], (0, 1), 16),
    r(25, &[(0, 1, 9)], (0, 1), 10),
    r(13, &[(0, 1, 5)], (0, 1), 12),
    r(28, &[(0, -1, 6), (1, 0, 1)], (1, 0), 23),
    r(19, &[(1, 0, 10)], (1, 0), 17),
    r(21, &[(1, 0, 9)], (1, 0), 11),
    r(11, &[(1, 0, 3)], (1, 0), 5),
    r(10, &[(1, 0, 4)], (1, 0), 12),
    r(14, &[(1, 0, 4)], (1, 0), 12),
    r(27, &[(1, 0, 9)], (1, 0), 11),
    r(15, &[(1, 0, 4)], (1, 0), 12),
    r(21, &[(1, 0, 1)], (1, 0), 16),
    r(11, &[(1, 0, 4)], (1, 0), 12),
    r(2, &[(1, 0, 3)], (1, 0), 5),
    r(9, &[(1, 0, 3)], (1, 0), 5),
    r(14, &[(1, 0, 3)], (1, 0), 5),
    r(15, &[(1, 0, 3)], (1, 0), 5),
    r(2, &[(1, 0, 5), (1, -1, 16)], (1, 0), 12),
    r(2, &[(1, 0, 4)], (1, 0), 12),
    r(9, &[(1, 0, 4)], (1, 0), 12),
    r(1, &[(0, -1, 8)], (0, -1), 9),
    r(28, &[(1, 0, 23), (1, 1, 3)], (1, 0), 16),
    r(0, &[], (0, 0), 0),
];

const PASS5: [Rule; 19] = [
    r(21, &[(1, 0, 10)], (1, 0), 17),
    r(17, &[(1, 0, 4)], (1, 0), 12),
    r(10, &[(1, 0, 4)], (1, 0), 12),
    r(17, &[(0, 1, 5)], (0, 1), 12),
    r(29, &[(0, 1, 9)], (0, 1), 10),
    r(13, &[(0, 1, 5)], (0, 1), 12),
    r(9, &[(0, 1, 16)], (0, 1), 13),
    r(10, &[(0, 1, 16)], (0, 1), 13),
    r(16, &[(0, 1, 3)], (0, 1), 4),
    r(11, &[(0, 1, 5)], (0, 1), 12),
    r(10, &[(1, 0, 3), (1, -1, 16)], (1, 0), 12),
    r(16, &[(0, 1, 5)], (0, 1), 12),
    r(1, &[(0, 1, 6)], (0, 1), 4),
    r(21, &[(1, 0, 13), (0, 1, 10)], (1, 1), 12),
    r(15, &[(1, 0, 10)], (1, 0), 17),
    r(22, &[(0, 1, 11)], (0, 1), 17),
    r(15, &[(1, 0, 28), (2, 0, 16)], (1, 0), 23),
    r(28, &[(1, 0, 23), (1, 1, 1), (2, 0, 6)], (1, 0), 16),
    r(0, &[], (0, 0), 0),
];

const PASS6: [Rule; 3] = [
    r(15, &[(1, 0, 28), (2, 0, 16)], (1, 0), 23),
    r(21, &[(1, -1, 21), (1, 1, 13), (2, 0, 2)], (1, 0), 17),
    r(19, &[(1, 0, 15), (1, 1, 12)], (1, 0), 17),
];

/// Original: `FixTilesPatterns` (levels/drlg_l4.cpp). The rules are kept as data in source order;
/// tile value 0 never occurs in the passes that use the placeholder entries, so those never fire.
// @port levels/drlg_l4.cpp|devilution::FixTilesPatterns()
fn fix_tiles_patterns(ctx: &mut Ctx) {
    apply_rules(ctx, &PASS1);
    apply_rules(ctx, &PASS2);
    // third pass: PASS3 then PASS3B per tile, with the `/* check */` inequality rule at index 10
    for j in 0..DY {
        for i in 0..DX {
            for (k, rule) in PASS3.iter().enumerate() {
                if k == 10 {
                    if d(ctx, i, j) == 6 && d(ctx, i + 1, j) == 27 && d(ctx, i + 1, j + 1) != 0 {
                        // check
                        sd(ctx, i + 1, j, 22);
                    }
                    continue;
                }
                if d(ctx, i, j) == rule.tile && rule.checks.iter().all(|&(dx, dy, v)| d(ctx, i + dx, j + dy) == v) {
                    sd(ctx, i + rule.target.0, j + rule.target.1, rule.value);
                }
            }
            for rule in PASS3B.iter().take(16) {
                if d(ctx, i, j) == rule.tile && rule.checks.iter().all(|&(dx, dy, v)| d(ctx, i + dx, j + dy) == v) {
                    sd(ctx, i + rule.target.0, j + rule.target.1, rule.value);
                }
            }
        }
    }
    apply_rules(ctx, &PASS4[..34]);
    apply_rules(ctx, &PASS5[..18]);
    apply_rules(ctx, &PASS6);
}

/// Original: `Substitution` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::Substitution()
fn substitution(ctx: &mut Ctx) {
    for y in 0..DY {
        for x in 0..DX {
            if ctx.rng.flip_coin(3) {
                let c = L4BTYPES[d(ctx, x, y) as usize];
                if c != 0 && !ctx.gendung.Protected.test(x, y) {
                    let mut rv = rnd(ctx, 16);
                    let mut i: i32 = -1;
                    while rv >= 0 {
                        i += 1;
                        if i == L4BTYPES.len() as i32 {
                            i = 0;
                        }
                        if c == L4BTYPES[i as usize] {
                            rv -= 1;
                        }
                    }
                    sd(ctx, x, y, i as u8);
                }
            }
        }
    }
    for y in 0..DY {
        for x in 0..DX {
            if ctx.rng.flip_coin(10) {
                let c = d(ctx, x, y);
                if L4BTYPES[c as usize] == 6 && !ctx.gendung.Protected.test(x, y) {
                    let v = rnd(ctx, 3) + 95;
                    sd(ctx, x, y, v as u8);
                }
            }
        }
    }
}

/// Original: `PrepareInnerBorders` (levels/drlg_l4.cpp): sets up the inner borders of the first
/// quadrant so there are valid paths after mirroring.
// @port levels/drlg_l4.cpp|devilution::PrepareInnerBorders()
fn prepare_inner_borders(ctx: &mut Ctx) {
    let (hw, hh) = (DX / 2, DY / 2);
    let mut y = hh - 1;
    while y >= 0 {
        let mut x = hw - 1;
        while x >= 0 {
            let m = &ctx.gendung.DungeonMask;
            if !m.test(x, y) {
                ctx.drlg_l4.hallok[y as usize] = false;
            } else {
                let has_south_west_room = y + 1 < hh && m.test(x, y + 1);
                let has_south_room = x + 1 < hw && y + 1 < hh && m.test(x + 1, y + 1);
                ctx.drlg_l4.hallok[y as usize] = has_south_west_room && !has_south_room;
                x = 0;
            }
            x -= 1;
        }
        y -= 1;
    }
    let mut ry = rnd(ctx, hh - 1) + 1;
    loop {
        if ctx.drlg_l4.hallok[ry as usize] {
            let mut x = hw - 1;
            while x >= 0 {
                if ctx.gendung.DungeonMask.test(x, ry) {
                    x = -1;
                    ry = 0;
                } else {
                    ctx.gendung.DungeonMask.set(x, ry);
                    ctx.gendung.DungeonMask.set(x, ry + 1);
                }
                x -= 1;
            }
        } else {
            ry += 1;
            if ry == hh {
                ry = 1;
            }
        }
        if ry == 0 {
            break;
        }
    }
    let mut x = hw - 1;
    while x >= 0 {
        let mut y = hh - 1;
        while y >= 0 {
            let m = &ctx.gendung.DungeonMask;
            if !m.test(x, y) {
                ctx.drlg_l4.hallok[x as usize] = false;
            } else {
                let has_south_east_room = x + 1 < hw && m.test(x + 1, y);
                let has_south_room = x + 1 < hw && y + 1 < hh && m.test(x + 1, y + 1);
                ctx.drlg_l4.hallok[x as usize] = has_south_east_room && !has_south_room;
                y = 0;
            }
            y -= 1;
        }
        x -= 1;
    }
    let mut rx = rnd(ctx, hw - 1) + 1;
    loop {
        if ctx.drlg_l4.hallok[rx as usize] {
            let mut y = hh - 1;
            while y >= 0 {
                if ctx.gendung.DungeonMask.test(rx, y) {
                    y = -1;
                    rx = 0;
                } else {
                    ctx.gendung.DungeonMask.set(rx, y);
                    ctx.gendung.DungeonMask.set(rx + 1, y);
                }
                y -= 1;
            }
        } else {
            rx += 1;
            if rx == hw {
                rx = 1;
            }
        }
        if rx == 0 {
            break;
        }
    }
}

/// Original: `FindArea` (levels/drlg_l4.cpp): the tiles used by the mirrored layout.
// @port levels/drlg_l4.cpp|devilution::FindArea()
fn find_area(ctx: &Ctx) -> usize {
    // Hell layouts are mirrored based on a single quadrant, this function is called after the quadrant has been
    // generated but before mirroring the layout. We need to multiply by 4 to get the expected number of tiles
    ctx.gendung.DungeonMask.count() * 4
}

/// Original: `ProtectQuads` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::ProtectQuads()
fn protect_quads(ctx: &mut Ctx) {
    let (hx, hy) = (ctx.drlg_l4.l4_hold.0 as i32, ctx.drlg_l4.l4_hold.1 as i32);
    let p = &mut ctx.gendung.Protected;
    for y in 0..14 {
        for x in 0..14 {
            p.set(hx + x, hy + y);
            p.set(DX - 1 - x - hx, hy + y);
            p.set(hx + x, DY - 1 - y - hy);
            p.set(DX - 1 - x - hx, DY - 1 - y - hy);
        }
    }
}

/// Original: `LoadDiabQuads` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::LoadDiabQuads(bool preflag)
fn load_diab_quads(ctx: &mut Ctx, preflag: bool) {
    let hold = ctx.drlg_l4.l4_hold;
    let q1 = wadd(hold, 4, 4);
    let q2 = (27u8.wrapping_sub(hold.0), 1u8.wrapping_add(hold.1));
    let q3 = (1u8.wrapping_add(hold.0), 27u8.wrapping_sub(hold.1));
    let q4 = (28u8.wrapping_sub(hold.0), 28u8.wrapping_sub(hold.1));
    let files = [
        ("levels\\l4data\\diab1.dun", q1),
        (if preflag { "levels\\l4data\\diab2b.dun" } else { "levels\\l4data\\diab2a.dun" }, q2),
        (if preflag { "levels\\l4data\\diab3b.dun" } else { "levels\\l4data\\diab3a.dun" }, q3),
        (if preflag { "levels\\l4data\\diab4b.dun" } else { "levels\\l4data\\diab4a.dun" }, q4),
    ];
    for (k, (path, q)) in files.into_iter().enumerate() {
        let dun_data = load_u16_file(ctx, path);
        let p = Point::new(q.0 as i32, q.1 as i32);
        ctx.drlg_l4.diablo_quads[k] = p;
        place_dun_tiles(ctx, &dun_data, p, 6);
    }
}

/// Original: `IsDURightWall` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::IsDURightWall(char d)
fn is_du_right_wall(d: u8) -> bool {
    matches!(d, 25 | 28 | 23)
}

/// Original: `IsDLLeftWall` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::IsDLLeftWall(char dd)
fn is_dl_left_wall(dd: u8) -> bool {
    matches!(dd, 27 | 26 | 22)
}

/// Original: `FixTransparency` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::FixTransparency()
fn fix_transparency(ctx: &mut Ctx) {
    let mut yy = 16usize;
    for j in 0..DY {
        let mut xx = 16usize;
        for i in 0..DX {
            let dv = d(ctx, i, j);
            // BUGFIX: Should check for `j > 0` first.
            let up = d(ctx, i, j - 1);
            // BUGFIX: Should check for `i + 1 < DMAXY` first.
            let right = d(ctx, i + 1, j);
            let t = &mut ctx.gendung.dTransVal;
            let v = t[xx][yy];
            if is_du_right_wall(dv) && up == 18 {
                t[xx + 1][yy] = v;
                t[xx + 1][yy + 1] = v;
            }
            if is_dl_left_wall(dv) && right == 19 {
                t[xx][yy + 1] = v;
                t[xx + 1][yy + 1] = v;
            }
            if dv == 18 {
                t[xx + 1][yy] = v;
                t[xx + 1][yy + 1] = v;
            }
            if dv == 19 {
                t[xx][yy + 1] = v;
                t[xx + 1][yy + 1] = v;
            }
            if dv == 24 {
                t[xx + 1][yy] = v;
                t[xx][yy + 1] = v;
                t[xx + 1][yy + 1] = v;
            }
            if dv == 57 {
                t[xx - 1][yy] = t[xx][yy + 1];
                t[xx][yy] = t[xx][yy + 1];
            }
            if dv == 53 {
                t[xx][yy - 1] = t[xx + 1][yy];
                t[xx][yy] = t[xx + 1][yy];
            }
            xx += 2;
        }
        yy += 2;
    }
}

/// Original: `FixCornerTiles` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::FixCornerTiles()
fn fix_corner_tiles(ctx: &mut Ctx) {
    for j in 1..DY - 1 {
        for i in 1..DX - 1 {
            let v = d(ctx, i, j);
            if (18..=30).contains(&v) && (d(ctx, i + 1, j) < 18 || d(ctx, i, j + 1) < 18) {
                sd(ctx, i, j, v.wrapping_add(98));
            }
        }
    }
}

/// Original: `CloseOuterBorders` (levels/drlg_l4.cpp): marks the map edge as not part of the layout.
// @port levels/drlg_l4.cpp|devilution::CloseOuterBorders()
fn close_outer_borders(ctx: &mut Ctx) {
    for x in 0..DX / 2 {
        ctx.gendung.DungeonMask.reset_at(x, 0);
    }
    for y in 0..DY / 2 {
        ctx.gendung.DungeonMask.reset_at(0, y);
    }
}

/// Original: `GeneralFix` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::GeneralFix()
fn general_fix(ctx: &mut Ctx) {
    for j in 0..DY - 1 {
        for i in 0..DX - 1 {
            if (d(ctx, i, j) == 24 || d(ctx, i, j) == 122) && d(ctx, i + 1, j) == 2 && d(ctx, i, j + 1) == 5 {
                sd(ctx, i, j, 17);
            }
        }
    }
}

/// Original: `PlaceStairs` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::PlaceStairs(lvl_entry entry)
fn place_stairs(ctx: &mut Ctx, entry: lvl_entry) -> bool {
    // Place stairs up
    let Some(position) = place_mini_set(ctx, &L4USTAIRS, 199, false) else {
        return false;
    };
    if entry == ENTRY_MAIN {
        ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(6, 6);
    }
    let currlevel = ctx.gendung.currlevel;
    if currlevel != 15 {
        // Place stairs down
        if currlevel != 16 {
            if crate::quests::is_quest_available(ctx, Q_WARLORD) {
                if entry == ENTRY_PREV {
                    ctx.gendung.ViewPosition = ctx.gendung.SetPiece.position.mega_to_world() + Displacement::new(7, 7);
                }
            } else {
                let Some(position) = place_mini_set(ctx, &L4DSTAIRS, 199, false) else {
                    return false;
                };
                if entry == ENTRY_PREV {
                    ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(7, 5);
                }
            }
        }
        // Place town warp stairs
        if currlevel == 13 {
            let Some(position) = place_mini_set(ctx, &L4TWARP, 199, false) else {
                return false;
            };
            if entry == ENTRY_TWARPDN {
                ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(6, 6);
            }
        }
    } else {
        // Place hell gate
        let Some(position) = place_mini_set(ctx, &L4PENTA2, 199, false) else {
            return false;
        };
        ctx.quests.Quests[Q_DIABLO as usize].position = position;
        if entry == ENTRY_PREV {
            ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(6, 5);
        }
    }
    true
}

/// Original: `GenerateLevel` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::GenerateLevel(lvl_entry entry)
fn generate_level(ctx: &mut Ctx, entry: lvl_entry) {
    load_quest_set_pieces(ctx);
    loop {
        drlg_init_trans(ctx);
        const MINAREA: usize = 692;
        loop {
            init_dungeon_flags(ctx);
            first_room(ctx);
            close_outer_borders(ctx);
            if find_area(ctx) >= MINAREA {
                break;
            }
        }
        prepare_inner_borders(ctx);
        mirror_dungeon_layout(ctx);
        make_dmt(ctx);
        fix_tiles_patterns(ctx);
        if ctx.gendung.currlevel == 16 {
            protect_quads(ctx);
        }
        let currlevel = ctx.gendung.currlevel;
        if crate::quests::is_quest_available(ctx, Q_WARLORD) || (currlevel == ctx.quests.Quests[Q_BETRAYER as usize]._qlevel && crate::quests::use_multiplayer_quests(ctx)) {
            let r = ctx.gendung.SetPieceRoom;
            for spi in r.position.x..r.position.x + r.size.width - 1 {
                for spj in r.position.y..r.position.y + r.size.height - 1 {
                    ctx.gendung.Protected.set(spi, spj);
                }
            }
        }
        add_wall(ctx);
        flood_transparency_values(ctx, 6);
        fix_transparency(ctx);
        let p = ctx.gendung.SetPieceRoom.position;
        set_set_piece_room(ctx, p, 6);
        if ctx.gendung.currlevel == 16 {
            load_diab_quads(ctx, true);
        }
        if place_stairs(ctx, entry) {
            break;
        }
    }
    free_quest_set_pieces(ctx);
    general_fix(ctx);
    if ctx.gendung.currlevel != 16 {
        drlg_place_theme_rooms(ctx, 7, 10, 6, 8, true);
    }
    apply_shadows_patterns(ctx);
    fix_corner_tiles(ctx);
    substitution(ctx);
    ctx.gendung.pdungeon = ctx.gendung.dungeon;
    let p = ctx.gendung.SetPieceRoom.position;
    crate::quests::drlg_check_quests(ctx, p);
    if ctx.gendung.currlevel == 15 {
        let is_gate_open = crate::quests::use_multiplayer_quests(ctx) || ctx.quests.Quests[Q_DIABLO as usize]._qactive == QUEST_ACTIVE;
        if !is_gate_open {
            let qp = ctx.quests.Quests[Q_DIABLO as usize].position;
            L4PENTA.place(&mut ctx.gendung, qp, false);
        }
        for j in 1..DY {
            for i in 1..DX {
                if matches!(d(ctx, i, j), 98 | 107) {
                    make_set_pc(ctx, Rectangle::new(Point::new(i - 1, j - 1), Size::new(5, 5)));
                    // Set the portal position to the location of the northmost pentagram tile.
                    ctx.quests.Quests[Q_BETRAYER as usize].position = Point::new(i, j).mega_to_world();
                }
            }
        }
    }
    if ctx.gendung.currlevel == 16 {
        load_diab_quads(ctx, false);
    }
}

/// Original: `Pass3` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::Pass3()
fn pass3(ctx: &mut Ctx) {
    drlg_l_pass3(ctx, 30 - 1);
}

/// Original: `devilution::CreateL4Dungeon` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::CreateL4Dungeon(uint32_t rseed, lvl_entry entry)
pub fn create_l4_dungeon(ctx: &mut Ctx, rseed: u32, entry: lvl_entry) {
    ctx.rng.set_rnd_seed(rseed);
    generate_level(ctx, entry);
    pass3(ctx);
}

/// Original: `devilution::LoadPreL4Dungeon` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::LoadPreL4Dungeon(const char *path)
pub fn load_pre_l4_dungeon(ctx: &mut Ctx, path: &str) {
    ctx.gendung.dungeon = [[30; DMAXY]; DMAXX];
    let dun_data = load_u16_file(ctx, path);
    place_dun_tiles(ctx, &dun_data, Point::new(0, 0), 6);
    ctx.gendung.pdungeon = ctx.gendung.dungeon;
}

/// Original: `devilution::LoadL4Dungeon` (levels/drlg_l4.cpp).
// @port levels/drlg_l4.cpp|devilution::LoadL4Dungeon(const char *path, Point spawn)
pub fn load_l4_dungeon(ctx: &mut Ctx, path: &str, spawn: Point) {
    load_dungeon_base(ctx, path, spawn, 6, 30);
    pass3(ctx);
}
