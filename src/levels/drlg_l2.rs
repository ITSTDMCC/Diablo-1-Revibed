//! `Source/levels/drlg_l2.cpp`: catacombs level generation.
//!
//! Several loops of the original read (and a few write) one tile past the edge of `dungeon`.
//! Those accesses go through `dun`/`set_dun`, which use the flat array index the C++ code ends up
//! with: inside `dungeon` it is the neighbouring column, past the end it is `pdungeon` (declared
//! right after `dungeon` in gendung.cpp; inferred layout), and before the start it reads 0.

use std::collections::VecDeque;

use crate::ctx::Ctx;
use crate::engine::geometry::{Displacement, Point, Rectangle, Size};
use crate::enums::*;
use crate::levels::drlg_l2_data::*;
use crate::levels::gendung::*;

/// `HallDirection`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum HallDirection {
    None = 0,
    Up = 1,
    Right = 2,
    Down = 3,
    Left = 4,
}

/// `HallNode`
#[derive(Clone, Copy, Debug)]
struct HallNode {
    beginning: Point,
    end: Point,
    direction: HallDirection,
}

/// `RoomNode` (WorldTilePosition corners)
#[derive(Clone, Copy, Debug, Default)]
struct RoomNode {
    top_left: (u8, u8),
    bottom_right: (u8, u8),
}

/// File-scope globals of levels/drlg_l2.cpp.
pub struct DrlgL2State {
    nRoomCnt: i32,
    RoomList: [RoomNode; 81],
    HallList: VecDeque<HallNode>,
    /// `predungeon`: an ASCII representation of the level
    predungeon: [[u8; DMAXY]; DMAXX],
}

impl Default for DrlgL2State {
    fn default() -> Self {
        DrlgL2State { nRoomCnt: 0, RoomList: [RoomNode::default(); 81], HallList: VecDeque::new(), predungeon: [[0; DMAXY]; DMAXX] }
    }
}

/// `DirAdd`
const DIR_ADD: [Displacement; 5] = [Displacement::new(0, 0), Displacement::new(0, -1), Displacement::new(1, 0), Displacement::new(0, 1), Displacement::new(-1, 0)];

const DX: i32 = DMAXX as i32;
const DY: i32 = DMAXY as i32;

fn rnd(ctx: &mut Ctx, v: i32) -> i32 {
    ctx.rng.generate_rnd(v)
}

fn dun(ctx: &Ctx, x: i32, y: i32) -> u8 {
    dungeon_flat(ctx, x, y)
}

fn set_dun(ctx: &mut Ctx, x: i32, y: i32, v: u8) {
    set_dungeon_flat(ctx, x, y, v);
}

/// `predungeon[x][y]` (reads outside the array give 0, writes are dropped).
fn pd(ctx: &Ctx, x: i32, y: i32) -> u8 {
    let idx = x * DY + y;
    if (0..DX * DY).contains(&idx) {
        ctx.drlg_l2.predungeon[(idx / DY) as usize][(idx % DY) as usize]
    } else {
        0
    }
}

fn set_pd(ctx: &mut Ctx, x: i32, y: i32, v: u8) {
    let idx = x * DY + y;
    if (0..DX * DY).contains(&idx) {
        ctx.drlg_l2.predungeon[(idx / DY) as usize][(idx % DY) as usize] = v;
    }
}

/// Original: `ApplyShadowsPatterns` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::ApplyShadowsPatterns()
fn apply_shadows_patterns(ctx: &mut Ctx) {
    let d = &mut ctx.gendung.dungeon;
    for y in 1..DMAXY {
        for x in 1..DMAXX {
            let sd00 = BSTYPESL2[d[x][y] as usize];
            let sd10 = BSTYPESL2[d[x - 1][y] as usize];
            let sd01 = BSTYPESL2[d[x][y - 1] as usize];
            let sd11 = BSTYPESL2[d[x - 1][y - 1] as usize];
            for shadow in SPATSL2.iter() {
                if shadow.strig != sd00 {
                    continue;
                }
                if shadow.s1 != 0 && shadow.s1 != sd11 {
                    continue;
                }
                if shadow.s2 != 0 && shadow.s2 != sd01 {
                    continue;
                }
                if shadow.s3 != 0 && shadow.s3 != sd10 {
                    continue;
                }
                if shadow.nv1 != 0 {
                    d[x - 1][y - 1] = shadow.nv1;
                }
                if shadow.nv2 != 0 {
                    d[x][y - 1] = shadow.nv2;
                }
                if shadow.nv3 != 0 {
                    d[x - 1][y] = shadow.nv3;
                }
            }
        }
    }
}

/// Original: `PlaceMiniSetRandom` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::PlaceMiniSetRandom(const Miniset &miniset, int rndper)
fn place_mini_set_random(ctx: &mut Ctx, miniset: &Miniset, rndper: i32) {
    let sw = miniset.size.width;
    let sh = miniset.size.height;
    for sy in 0..DY - sh {
        for sx in 0..DX - sw {
            if ctx.gendung.SetPieceRoom.contains_xy(sx, sy) {
                continue;
            }
            if !miniset.matches(&ctx.gendung, Point::new(sx, sy), true) {
                continue;
            }
            let mut found = true;
            let mut yy = (sy - sh).max(0);
            while yy < (sy + 2 * sh).min(DY) && found {
                for xx in (sx - sw).max(0)..(sx + 2 * sw).min(DX) {
                    if ctx.gendung.dungeon[xx as usize][yy as usize] == miniset.replace[0][0] {
                        found = false;
                        break;
                    }
                }
                yy += 1;
            }
            if !found {
                continue;
            }
            if rnd(ctx, 100) >= rndper {
                continue;
            }
            miniset.place(&mut ctx.gendung, Point::new(sx, sy), false);
        }
    }
}

/// Original: `PlaceMiniSetRandom1x1` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::PlaceMiniSetRandom1x1(uint8_t search, uint8_t replace, int rndper)
fn place_mini_set_random_1x1(ctx: &mut Ctx, search: u8, replace: u8, rndper: i32) {
    let mut m = Miniset { size: Size::new(1, 1), search: [[0; 6]; 6], replace: [[0; 6]; 6] };
    m.search[0][0] = search;
    m.replace[0][0] = replace;
    place_mini_set_random(ctx, &m, rndper);
}

/// Original: `LoadQuestSetPieces` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::LoadQuestSetPieces()
fn load_quest_set_pieces(ctx: &mut Ctx) {
    use crate::quests::is_quest_available;
    if is_quest_available(ctx, Q_BLIND) {
        ctx.gendung.pSetPiece = Some(load_u16_file(ctx, "levels\\l2data\\blind1.dun"));
    } else if is_quest_available(ctx, Q_BLOOD) {
        ctx.gendung.pSetPiece = Some(load_u16_file(ctx, "levels\\l2data\\blood1.dun"));
    } else if is_quest_available(ctx, Q_SCHAMB) {
        ctx.gendung.pSetPiece = Some(load_u16_file(ctx, "levels\\l2data\\bonestr2.dun"));
    }
}

/// Original: `InitDungeonPieces` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::InitDungeonPieces()
fn init_dungeon_pieces(ctx: &mut Ctx) {
    let g = &mut ctx.gendung;
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            let pc = match g.dPiece[i][j] {
                540 | 177 | 550 => 5,
                541 | 552 => 6,
                _ => continue,
            };
            g.dSpecial[i][j] = pc;
        }
    }
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            if g.dPiece[i][j] == 131 {
                g.dSpecial[i][j + 1] = 2;
                g.dSpecial[i][j + 2] = 1;
            } else if g.dPiece[i][j] == 134 || g.dPiece[i][j] == 138 {
                g.dSpecial[i + 1][j] = 3;
                g.dSpecial[i + 2][j] = 4;
            }
        }
    }
}

/// Original: `InitDungeonFlags` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::InitDungeonFlags()
fn init_dungeon_flags(ctx: &mut Ctx) {
    ctx.gendung.Protected.reset();
    ctx.drlg_l2.predungeon = [[b' '; DMAXY]; DMAXX];
}

/// Original: `MapRoom` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::MapRoom(int x1, int y1, int x2, int y2)
fn map_room(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    for jj in y1..=y2 {
        for ii in x1..=x2 {
            set_pd(ctx, ii, jj, b'.');
        }
    }
    for jj in y1..=y2 {
        set_pd(ctx, x1, jj, b'#');
        set_pd(ctx, x2, jj, b'#');
    }
    for ii in x1..=x2 {
        set_pd(ctx, ii, y1, b'#');
        set_pd(ctx, ii, y2, b'#');
    }
}

/// Original: `DefineRoom` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::DefineRoom(Point topLeft, Point bottomRight, bool forceHW)
fn define_room(ctx: &mut Ctx, top_left: Point, mut bottom_right: Point, force_hw: bool) {
    set_pd(ctx, top_left.x, top_left.y, b'C');
    set_pd(ctx, top_left.x, bottom_right.y, b'E');
    set_pd(ctx, bottom_right.x, top_left.y, b'B');
    set_pd(ctx, bottom_right.x, bottom_right.y, b'A');
    ctx.drlg_l2.nRoomCnt += 1;
    let n = ctx.drlg_l2.nRoomCnt as usize;
    ctx.drlg_l2.RoomList[n] = RoomNode { top_left: (top_left.x as u8, top_left.y as u8), bottom_right: (bottom_right.x as u8, bottom_right.y as u8) };
    if force_hw {
        let mut i = top_left.x;
        while i < bottom_right.x {
            // BUGFIX: Should loop j between nY1 and nY2 instead of always using nY1.
            while i < bottom_right.y {
                ctx.gendung.Protected.set(i, top_left.y);
                i += 1;
            }
            i += 1;
        }
    }
    for i in top_left.x + 1..=bottom_right.x - 1 {
        set_pd(ctx, i, top_left.y, b'#');
        set_pd(ctx, i, bottom_right.y, b'#');
    }
    bottom_right.y -= 1;
    for j in top_left.y + 1..=bottom_right.y {
        set_pd(ctx, top_left.x, j, b'#');
        set_pd(ctx, bottom_right.x, j, b'#');
        for i in top_left.x + 1..bottom_right.x {
            set_pd(ctx, i, j, b'.');
        }
    }
}

/// Original: `CreateDoorType` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::CreateDoorType(Point position)
fn create_door_type(ctx: &mut Ctx, position: Point) {
    let (x, y) = (position.x, position.y);
    if pd(ctx, x - 1, y) == b'D' || pd(ctx, x + 1, y) == b'D' || pd(ctx, x, y - 1) == b'D' || pd(ctx, x, y + 1) == b'D' {
        return;
    }
    if matches!(pd(ctx, x, y), b'A' | b'B' | b'C' | b'E') {
        return;
    }
    set_pd(ctx, x, y, b'D');
}

/// Original: `PlaceHallExt` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::PlaceHallExt(Point position)
fn place_hall_ext(ctx: &mut Ctx, position: Point) {
    if pd(ctx, position.x, position.y) == b' ' {
        set_pd(ctx, position.x, position.y, b',');
    }
}

type WT = (u8, u8);

fn wt_add(a: WT, d: (i8, i8)) -> WT {
    (a.0.wrapping_add(d.0 as u8), a.1.wrapping_add(d.1 as u8))
}

fn wt_sub(a: WT, d: (i8, i8)) -> WT {
    (a.0.wrapping_sub(d.0 as u8), a.1.wrapping_sub(d.1 as u8))
}

fn wt_point(a: WT) -> Point {
    Point::new(a.0 as i32, a.1 as i32)
}

/// Original: `CreateRoom` (levels/drlg_l2.cpp): draws a random room rectangle, then subdivides
/// the rest of the area into four and recurses. Coordinates are `uint8_t` as in the original.
// @port levels/drlg_l2.cpp|devilution::CreateRoom(WorldTilePosition topLeft, WorldTilePosition bottomRight, int nRDest, HallDirection nHDir, std::optional<WorldTileSize> size)
fn create_room(ctx: &mut Ctx, top_left: WT, bottom_right: WT, n_r_dest: i32, n_h_dir: HallDirection, size: Option<(u8, u8)>) {
    const AREA_MIN: i32 = 2;
    if ctx.drlg_l2.nRoomCnt >= 80 || top_left.0 as i32 + AREA_MIN > bottom_right.0 as i32 || top_left.1 as i32 + AREA_MIN > bottom_right.1 as i32 {
        return;
    }
    // WorldTileDisplacement is int8_t; WorldTileSize is uint8_t
    let area = ((bottom_right.0 as i32 - top_left.0 as i32) as i8 as u8, (bottom_right.1 as i32 - top_left.1 as i32) as i8 as u8);
    const ROOM_MAX: u8 = 10;
    const ROOM_MIN: u8 = 4;
    let mut room_size = area;
    if area.0 > ROOM_MIN {
        room_size.0 = (rnd(ctx, area.0.min(ROOM_MAX) as i32 - ROOM_MIN as i32) + ROOM_MIN as i32) as u8;
    }
    if area.1 > ROOM_MIN {
        room_size.1 = (rnd(ctx, area.1.min(ROOM_MAX) as i32 - ROOM_MIN as i32) + ROOM_MIN as i32) as u8;
    }
    if let Some(s) = size {
        room_size = s;
    }
    let random_width = rnd(ctx, area.0 as i32);
    let random_height = rnd(ctx, area.1 as i32);
    let mut room_top_left = wt_add(top_left, (random_width as i8, random_height as i8));
    let mut room_bottom_right = wt_add(room_top_left, (room_size.0 as i8, room_size.1 as i8));
    if room_bottom_right.0 > bottom_right.0 {
        room_bottom_right.0 = bottom_right.0;
        room_top_left.0 = bottom_right.0.wrapping_sub(room_size.0);
    }
    if room_bottom_right.1 > bottom_right.1 {
        room_bottom_right.1 = bottom_right.1;
        room_top_left.1 = bottom_right.1.wrapping_sub(room_size.1);
    }
    room_top_left.0 = room_top_left.0.clamp(1, 38);
    room_top_left.1 = room_top_left.1.clamp(1, 38);
    room_bottom_right.0 = room_bottom_right.0.clamp(1, 38);
    room_bottom_right.1 = room_bottom_right.1.clamp(1, 38);

    define_room(ctx, wt_point(room_top_left), wt_point(room_bottom_right), size.is_some());

    const STANDOFF: (i8, i8) = (2, 2);
    if size.is_some() {
        let p = wt_add(room_top_left, STANDOFF);
        ctx.gendung.SetPieceRoom = Rectangle::new(wt_point(p), Size::new(room_size.0.wrapping_sub(1) as i32, room_size.1.wrapping_sub(1) as i32));
    }

    let n_rid = ctx.drlg_l2.nRoomCnt;

    if n_r_dest != 0 {
        let dest = ctx.drlg_l2.RoomList[n_r_dest as usize];
        let (mut n_hx1, mut n_hy1, mut n_hx2, mut n_hy2) = (0u8, 0u8, 0u8, 0u8);
        if n_h_dir == HallDirection::Up {
            n_hx1 = (rnd(ctx, room_size.0 as i32 - 2) + room_top_left.0 as i32 + 1) as u8;
            n_hy1 = room_top_left.1;
            let n_hw = dest.bottom_right.0 as i32 - dest.top_left.0 as i32 - 2;
            n_hx2 = (rnd(ctx, n_hw) + dest.top_left.0 as i32 + 1) as u8;
            n_hy2 = dest.bottom_right.1;
        }
        if n_h_dir == HallDirection::Down {
            n_hx1 = (rnd(ctx, room_size.0 as i32 - 2) + room_top_left.0 as i32 + 1) as u8;
            n_hy1 = room_bottom_right.1;
            let n_hw = dest.bottom_right.0 as i32 - dest.top_left.0 as i32 - 2;
            n_hx2 = (rnd(ctx, n_hw) + dest.top_left.0 as i32 + 1) as u8;
            n_hy2 = dest.top_left.1;
        }
        if n_h_dir == HallDirection::Right {
            n_hx1 = room_bottom_right.0;
            n_hy1 = (rnd(ctx, room_size.1 as i32 - 2) + room_top_left.1 as i32 + 1) as u8;
            n_hx2 = dest.top_left.0;
            let n_hh = dest.bottom_right.1 as i32 - dest.top_left.1 as i32 - 2;
            n_hy2 = (rnd(ctx, n_hh) + dest.top_left.1 as i32 + 1) as u8;
        }
        if n_h_dir == HallDirection::Left {
            n_hx1 = room_top_left.0;
            n_hy1 = (rnd(ctx, room_size.1 as i32 - 2) + room_top_left.1 as i32 + 1) as u8;
            n_hx2 = dest.bottom_right.0;
            let n_hh = dest.bottom_right.1 as i32 - dest.top_left.1 as i32 - 2;
            n_hy2 = (rnd(ctx, n_hh) + dest.top_left.1 as i32 + 1) as u8;
        }
        ctx.drlg_l2.HallList.push_back(HallNode { beginning: wt_point((n_hx1, n_hy1)), end: wt_point((n_hx2, n_hy2)), direction: n_h_dir });
    }

    let room_bottom_left = (room_top_left.0, room_bottom_right.1);
    let room_top_right = (room_bottom_right.0, room_top_left.1);
    if room_size.1 > room_size.0 {
        create_room(ctx, wt_add(top_left, STANDOFF), wt_sub(room_bottom_left, STANDOFF), n_rid, HallDirection::Right, None);
        create_room(ctx, wt_add(room_top_right, STANDOFF), wt_sub(bottom_right, STANDOFF), n_rid, HallDirection::Left, None);
        create_room(ctx, wt_add((top_left.0, room_bottom_right.1), STANDOFF), wt_sub((room_bottom_right.0, bottom_right.1), STANDOFF), n_rid, HallDirection::Up, None);
        create_room(ctx, wt_add((room_top_left.0, top_left.1), STANDOFF), wt_sub((bottom_right.0, room_top_left.1), STANDOFF), n_rid, HallDirection::Down, None);
    } else {
        create_room(ctx, wt_add(top_left, STANDOFF), wt_sub(room_top_right, STANDOFF), n_rid, HallDirection::Down, None);
        create_room(ctx, wt_add(room_bottom_left, STANDOFF), wt_sub(bottom_right, STANDOFF), n_rid, HallDirection::Up, None);
        create_room(ctx, wt_add((top_left.0, room_top_left.1), STANDOFF), wt_sub((room_top_left.0, bottom_right.1), STANDOFF), n_rid, HallDirection::Right, None);
        create_room(ctx, wt_add((room_bottom_right.0, top_left.1), STANDOFF), wt_sub((bottom_right.0, room_bottom_right.1), STANDOFF), n_rid, HallDirection::Left, None);
    }
}

fn dir_add(d: HallDirection) -> Displacement {
    DIR_ADD[d as usize]
}

/// Original: `ConnectHall` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::ConnectHall(const HallNode &node)
fn connect_hall(ctx: &mut Ctx, node: HallNode) {
    use HallDirection as H;
    let mut beginning = node.beginning;
    let mut end = node.end;
    let f_minus_flag = rnd(ctx, 100) < 50;
    let f_plus_flag = rnd(ctx, 100) < 50;
    create_door_type(ctx, beginning);
    create_door_type(ctx, end);
    let mut n_currd = node.direction;
    end -= dir_add(n_currd);
    set_pd(ctx, end.x, end.y, b',');
    let mut f_inroom = false;
    loop {
        if beginning.x >= 38 && n_currd == H::Right {
            n_currd = H::Left;
        }
        if beginning.y >= 38 && n_currd == H::Down {
            n_currd = H::Up;
        }
        if beginning.x <= 1 && n_currd == H::Left {
            n_currd = H::Right;
        }
        if beginning.y <= 1 && n_currd == H::Up {
            n_currd = H::Down;
        }
        let c = pd(ctx, beginning.x, beginning.y);
        if c == b'C' && matches!(n_currd, H::Up | H::Left) {
            n_currd = H::Right;
        }
        if c == b'B' && matches!(n_currd, H::Up | H::Right) {
            n_currd = H::Down;
        }
        if c == b'E' && matches!(n_currd, H::Left | H::Down) {
            n_currd = H::Up;
        }
        if c == b'A' && matches!(n_currd, H::Right | H::Down) {
            n_currd = H::Left;
        }
        beginning += dir_add(n_currd);
        if pd(ctx, beginning.x, beginning.y) == b' ' {
            if f_inroom {
                create_door_type(ctx, beginning - dir_add(n_currd));
                f_inroom = false;
            } else {
                if f_minus_flag {
                    if !matches!(n_currd, H::Up | H::Down) {
                        place_hall_ext(ctx, beginning + Displacement::new(0, -1)); // Up
                    } else {
                        place_hall_ext(ctx, beginning + Displacement::new(-1, 0)); // Left
                    }
                }
                if f_plus_flag {
                    if !matches!(n_currd, H::Up | H::Down) {
                        place_hall_ext(ctx, beginning + Displacement::new(0, 1)); // Down
                    } else {
                        place_hall_ext(ctx, beginning + Displacement::new(1, 0)); // Right
                    }
                }
            }
            set_pd(ctx, beginning.x, beginning.y, b',');
        } else {
            if !f_inroom && pd(ctx, beginning.x, beginning.y) == b'#' {
                create_door_type(ctx, beginning);
            }
            if pd(ctx, beginning.x, beginning.y) != b',' {
                f_inroom = true;
            }
        }
        let n_dx = (end.x - beginning.x).abs();
        let n_dy = (end.y - beginning.y).abs();
        if n_dx > n_dy {
            let n_rp = (2 * n_dx).min(30);
            if rnd(ctx, 100) < n_rp {
                n_currd = if end.x <= beginning.x || beginning.x >= DX { H::Left } else { H::Right };
            }
        } else {
            let n_rp = (5 * n_dy).min(80);
            if rnd(ctx, 100) < n_rp {
                n_currd = if end.y <= beginning.y || beginning.y >= DY { H::Up } else { H::Down };
            }
        }
        if n_dy < 10 && beginning.x == end.x && matches!(n_currd, H::Right | H::Left) {
            n_currd = if end.y <= beginning.y || beginning.y >= DY { H::Up } else { H::Down };
        }
        if n_dx < 10 && beginning.y == end.y && matches!(n_currd, H::Up | H::Down) {
            n_currd = if end.x <= beginning.x || beginning.x >= DX { H::Left } else { H::Right };
        }
        if n_dy == 1 && n_dx > 1 && matches!(n_currd, H::Up | H::Down) {
            n_currd = if end.x <= beginning.x || beginning.x >= DX { H::Left } else { H::Right };
        }
        if n_dx == 1 && n_dy > 1 && matches!(n_currd, H::Right | H::Left) {
            n_currd = if end.y <= beginning.y || beginning.x >= DX { H::Up } else { H::Down };
        }
        if n_dx == 0 && pd(ctx, beginning.x, beginning.y) != b' ' && matches!(n_currd, H::Right | H::Left) {
            n_currd = if end.x <= node.beginning.x || beginning.x >= DX { H::Up } else { H::Down };
        }
        if n_dy == 0 && pd(ctx, beginning.x, beginning.y) != b' ' && matches!(n_currd, H::Up | H::Down) {
            n_currd = if end.y <= node.beginning.y || beginning.y >= DY { H::Left } else { H::Right };
        }
        if beginning == end {
            break;
        }
    }
}

/// Original: `DoPatternCheck` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::DoPatternCheck(int i, int j)
fn do_pattern_check(ctx: &mut Ctx, i: i32, j: i32) {
    let mut k = 0;
    while Patterns[k][4] != 255 {
        let mut x = i - 1;
        let mut y = j - 1;
        let mut n_ok = 254;
        let mut l = 0;
        while l < 9 && n_ok == 254 {
            n_ok = 255;
            if l == 3 || l == 6 {
                y += 1;
                x = i - 1;
            }
            if x >= 0 && x < DX && y >= 0 && y < DY {
                let c = pd(ctx, x, y);
                let ok = match Patterns[k][l] {
                    0 => true,
                    1 => c == b'#',
                    2 => c == b'.',
                    4 => c == b' ',
                    3 => c == b'D',
                    5 => c == b'D' || c == b'.',
                    6 => c == b'D' || c == b'#',
                    7 => c == b' ' || c == b'.',
                    8 => c == b'D' || c == b'#' || c == b'.',
                    _ => false,
                };
                if ok {
                    n_ok = 254;
                }
            } else {
                n_ok = 254;
            }
            x += 1;
            l += 1;
        }
        if n_ok == 254 {
            ctx.gendung.dungeon[i as usize][j as usize] = Patterns[k][9] as u8;
        }
        k += 1;
    }
}

/// Original: `FixTilesPatterns` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::FixTilesPatterns()
fn fix_tiles_patterns(ctx: &mut Ctx) {
    for j in 0..DY {
        for i in 0..DX {
            if dun(ctx, i, j) == 1 && dun(ctx, i, j + 1) == 3 {
                set_dun(ctx, i, j + 1, 1);
            }
            if dun(ctx, i, j) == 3 && dun(ctx, i, j + 1) == 1 {
                set_dun(ctx, i, j + 1, 3);
            }
            if dun(ctx, i, j) == 3 && dun(ctx, i + 1, j) == 7 {
                set_dun(ctx, i + 1, j, 3);
            }
            if dun(ctx, i, j) == 2 && dun(ctx, i + 1, j) == 3 {
                set_dun(ctx, i + 1, j, 2);
            }
            if dun(ctx, i, j) == 11 && dun(ctx, i + 1, j) == 14 {
                set_dun(ctx, i + 1, j, 16);
            }
        }
    }
}

/// Original: `Substitution` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::Substitution()
fn substitution(ctx: &mut Ctx) {
    for y in 0..DY {
        for x in 0..DX {
            if ctx.gendung.SetPieceRoom.contains_xy(x, y) {
                continue;
            }
            if !ctx.rng.flip_coin(4) {
                continue;
            }
            let c = BTYPESL2[ctx.gendung.dungeon[x as usize][y as usize] as usize];
            if c != 0 {
                let mut rv = rnd(ctx, 16);
                let mut i: i32 = -1;
                while rv >= 0 {
                    i += 1;
                    if i == BTYPESL2.len() as i32 {
                        i = 0;
                    }
                    if c == BTYPESL2[i as usize] {
                        rv -= 1;
                    }
                }
                let mut j = y - 2;
                while j < y + 2 {
                    let mut k = x - 2;
                    while k < x + 2 {
                        if dun(ctx, k, j) as i32 == i {
                            j = y + 3;
                            k = x + 2;
                        }
                        k += 1;
                    }
                    j += 1;
                }
                if j < y + 3 {
                    ctx.gendung.dungeon[x as usize][y as usize] = i as u8;
                }
            }
        }
    }
}

/// Original: `CountEmptyTiles` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::CountEmptyTiles()
fn count_empty_tiles(ctx: &Ctx) -> i32 {
    ctx.drlg_l2.predungeon.iter().flatten().filter(|&&c| c == b' ').count() as i32
}

/// Original: `KnockWalls` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::KnockWalls(int x1, int y1, int x2, int y2)
fn knock_walls(ctx: &mut Ctx, x1: i32, y1: i32, x2: i32, y2: i32) {
    for ii in x1 + 1..x2 {
        if pd(ctx, ii, y1 - 1) == b'.' && pd(ctx, ii, y1 + 1) == b'.' {
            set_pd(ctx, ii, y1, b'.');
        }
        if pd(ctx, ii, y2 - 1) == b'.' && pd(ctx, ii, y2 + 1) == b'.' {
            set_pd(ctx, ii, y2, b'.');
        }
        if pd(ctx, ii, y1 - 1) == b'D' {
            set_pd(ctx, ii, y1 - 1, b'.');
        }
        if pd(ctx, ii, y2 + 1) == b'D' {
            set_pd(ctx, ii, y2 + 1, b'.');
        }
    }
    for jj in y1 + 1..y2 {
        if pd(ctx, x1 - 1, jj) == b'.' && pd(ctx, x1 + 1, jj) == b'.' {
            set_pd(ctx, x1, jj, b'.');
        }
        if pd(ctx, x2 - 1, jj) == b'.' && pd(ctx, x2 + 1, jj) == b'.' {
            set_pd(ctx, x2, jj, b'.');
        }
        if pd(ctx, x1 - 1, jj) == b'D' {
            set_pd(ctx, x1 - 1, jj, b'.');
        }
        if pd(ctx, x2 + 1, jj) == b'D' {
            set_pd(ctx, x2 + 1, jj, b'.');
        }
    }
}

/// Original: `FillVoid` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::FillVoid(bool xf1, bool yf1, bool xf2, bool yf2, int xx, int yy)
fn fill_void(ctx: &mut Ctx, mut xf1: bool, mut yf1: bool, mut xf2: bool, mut yf2: bool, xx: i32, yy: i32) {
    let mut x1 = xx;
    if xf1 {
        x1 -= 1;
    }
    let mut x2 = xx;
    if xf2 {
        x2 += 1;
    }
    let mut y1 = yy;
    if yf1 {
        y1 -= 1;
    }
    let mut y2 = yy;
    if yf2 {
        y2 += 1;
    }
    let sp = |ctx: &Ctx, x: i32, y: i32| pd(ctx, x, y) != b' ';
    if !xf1 {
        while yf1 || yf2 {
            if y1 == 0 {
                yf1 = false;
            }
            if y2 == DY - 1 {
                yf2 = false;
            }
            if y2 - y1 >= 14 {
                yf1 = false;
                yf2 = false;
            }
            if yf1 {
                y1 -= 1;
            }
            if yf2 {
                y2 += 1;
            }
            if sp(ctx, x2, y1) {
                yf1 = false;
            }
            if sp(ctx, x2, y2) {
                yf2 = false;
            }
        }
        y1 += 2;
        y2 -= 2;
        if y2 - y1 > 5 {
            while xf2 {
                if x2 == 39 {
                    xf2 = false;
                }
                if x2 - x1 >= 12 {
                    xf2 = false;
                }
                for jj in y1..=y2 {
                    if sp(ctx, x2, jj) {
                        xf2 = false;
                    }
                }
                if xf2 {
                    x2 += 1;
                }
            }
            x2 -= 2;
            if x2 - x1 > 5 {
                map_room(ctx, x1, y1, x2, y2);
                knock_walls(ctx, x1, y1, x2, y2);
            }
        }
    } else if !xf2 {
        while yf1 || yf2 {
            if y1 == 0 {
                yf1 = false;
            }
            if y2 == DY - 1 {
                yf2 = false;
            }
            if y2 - y1 >= 14 {
                yf1 = false;
                yf2 = false;
            }
            if yf1 {
                y1 -= 1;
            }
            if yf2 {
                y2 += 1;
            }
            if sp(ctx, x1, y1) {
                yf1 = false;
            }
            if sp(ctx, x1, y2) {
                yf2 = false;
            }
        }
        y1 += 2;
        y2 -= 2;
        if y2 - y1 > 5 {
            while xf1 {
                if x1 == 0 {
                    xf1 = false;
                }
                if x2 - x1 >= 12 {
                    xf1 = false;
                }
                for jj in y1..=y2 {
                    if sp(ctx, x1, jj) {
                        xf1 = false;
                    }
                }
                if xf1 {
                    x1 -= 1;
                }
            }
            x1 += 2;
            if x2 - x1 > 5 {
                map_room(ctx, x1, y1, x2, y2);
                knock_walls(ctx, x1, y1, x2, y2);
            }
        }
    } else if !yf1 {
        while xf1 || xf2 {
            if x1 == 0 {
                xf1 = false;
            }
            if x2 == DX - 1 {
                xf2 = false;
            }
            if x2 - x1 >= 14 {
                xf1 = false;
                xf2 = false;
            }
            if xf1 {
                x1 -= 1;
            }
            if xf2 {
                x2 += 1;
            }
            if sp(ctx, x1, y2) {
                xf1 = false;
            }
            if sp(ctx, x2, y2) {
                xf2 = false;
            }
        }
        x1 += 2;
        x2 -= 2;
        if x2 - x1 > 5 {
            while yf2 {
                if y2 == DY - 1 {
                    yf2 = false;
                }
                if y2 - y1 >= 12 {
                    yf2 = false;
                }
                for ii in x1..=x2 {
                    if sp(ctx, ii, y2) {
                        yf2 = false;
                    }
                }
                if yf2 {
                    y2 += 1;
                }
            }
            y2 -= 2;
            if y2 - y1 > 5 {
                map_room(ctx, x1, y1, x2, y2);
                knock_walls(ctx, x1, y1, x2, y2);
            }
        }
    } else if !yf2 {
        while xf1 || xf2 {
            if x1 == 0 {
                xf1 = false;
            }
            if x2 == DX - 1 {
                xf2 = false;
            }
            if x2 - x1 >= 14 {
                xf1 = false;
                xf2 = false;
            }
            if xf1 {
                x1 -= 1;
            }
            if xf2 {
                x2 += 1;
            }
            if sp(ctx, x1, y1) {
                xf1 = false;
            }
            if sp(ctx, x2, y1) {
                xf2 = false;
            }
        }
        x1 += 2;
        x2 -= 2;
        if x2 - x1 > 5 {
            while yf1 {
                if y1 == 0 {
                    yf1 = false;
                }
                if y2 - y1 >= 12 {
                    yf1 = false;
                }
                for ii in x1..=x2 {
                    if sp(ctx, ii, y1) {
                        yf1 = false;
                    }
                }
                if yf1 {
                    y1 -= 1;
                }
            }
            y1 += 2;
            if y2 - y1 > 5 {
                map_room(ctx, x1, y1, x2, y2);
                knock_walls(ctx, x1, y1, x2, y2);
            }
        }
    }
}

/// Original: `FillVoids` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::FillVoids()
fn fill_voids(ctx: &mut Ctx) -> bool {
    let mut to = 0;
    while count_empty_tiles(ctx) > 700 && to < 100 {
        let xx = rnd(ctx, 38) + 1;
        let yy = rnd(ctx, 38) + 1;
        if pd(ctx, xx, yy) != b'#' {
            continue;
        }
        let (mut xf1, mut xf2, mut yf1, mut yf2) = (false, false, false, false);
        let p = |dx: i32, dy: i32| pd(ctx, xx + dx, yy + dy);
        if p(-1, 0) == b' ' && p(1, 0) == b'.' {
            if p(1, -1) == b'.' && p(1, 1) == b'.' && p(-1, -1) == b' ' && p(-1, 1) == b' ' {
                xf1 = true;
                yf1 = true;
                yf2 = true;
            }
        } else if p(1, 0) == b' ' && p(-1, 0) == b'.' {
            if p(-1, -1) == b'.' && p(-1, 1) == b'.' && p(1, -1) == b' ' && p(1, 1) == b' ' {
                xf2 = true;
                yf1 = true;
                yf2 = true;
            }
        } else if p(0, -1) == b' ' && p(0, 1) == b'.' {
            if p(-1, 1) == b'.' && p(1, 1) == b'.' && p(-1, -1) == b' ' && p(1, -1) == b' ' {
                yf1 = true;
                xf1 = true;
                xf2 = true;
            }
        } else if p(0, 1) == b' ' && p(0, -1) == b'.' && p(-1, -1) == b'.' && p(1, -1) == b'.' && p(-1, 1) == b' ' && p(1, 1) == b' ' {
            yf2 = true;
            xf1 = true;
            xf2 = true;
        }
        if xf1 || yf1 || xf2 || yf2 {
            fill_void(ctx, xf1, yf1, xf2, yf2, xx, yy);
        }
        to += 1;
    }
    count_empty_tiles(ctx) <= 700
}

/// Original: `CreateDungeon` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::CreateDungeon()
fn create_dungeon(ctx: &mut Ctx) -> bool {
    let mut size = None;
    match ctx.gendung.currlevel {
        5 => {
            if ctx.quests.Quests[Q_BLOOD as usize]._qactive != QUEST_NOTAVAIL {
                size = Some((14, 20));
            }
        }
        6 => {
            if ctx.quests.Quests[Q_SCHAMB as usize]._qactive != QUEST_NOTAVAIL {
                size = Some((10, 10));
            }
        }
        7 => {
            if ctx.quests.Quests[Q_BLIND as usize]._qactive != QUEST_NOTAVAIL {
                size = Some((15, 15));
            }
        }
        _ => {}
    }
    create_room(ctx, (2, 2), ((DX - 1) as u8, (DY - 1) as u8), 0, HallDirection::None, size);
    while let Some(node) = ctx.drlg_l2.HallList.pop_front() {
        connect_hall(ctx, node);
    }
    for j in 0..DY {
        // BUGFIX: change '<=' to '<' (fixed)
        for i in 0..DX {
            // BUGFIX: change '<=' to '<' (fixed)
            if matches!(pd(ctx, i, j), b'A' | b'B' | b'C' | b'E') {
                set_pd(ctx, i, j, b'#');
            }
            if pd(ctx, i, j) == b',' {
                set_pd(ctx, i, j, b'.');
                for a in -1..=1 {
                    for b in -1..=1 {
                        if a == 0 && b == 0 {
                            continue;
                        }
                        if i + a < 0 || j + b < 0 {
                            continue;
                        }
                        if i + a >= DX || j + b >= DY {
                            continue;
                        }
                        if pd(ctx, i + a, j + b) == b' ' {
                            set_pd(ctx, i + a, j + b, b'#');
                        }
                    }
                }
            }
        }
    }
    if !fill_voids(ctx) {
        return false;
    }
    for j in 0..DY {
        for i in 0..DX {
            do_pattern_check(ctx, i, j);
        }
    }
    true
}

/// Original: `FixTransparency` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::FixTransparency()
fn fix_transparency(ctx: &mut Ctx) {
    let mut yy = 16usize;
    for j in 0..DY {
        let mut xx = 16usize;
        for i in 0..DX {
            let d = dun(ctx, i, j);
            let t = &mut ctx.gendung.dTransVal;
            let v = t[xx][yy];
            // BUGFIX: Should check for `j > 0` first.
            let up = {
                let idx = i * DY + j - 1;
                if idx >= 0 {
                    ctx.gendung.dungeon[(idx / DY) as usize][(idx % DY) as usize]
                } else {
                    0
                }
            };
            let t = &mut ctx.gendung.dTransVal;
            if d == 14 && up == 10 {
                t[xx + 1][yy] = v;
                t[xx + 1][yy + 1] = v;
            }
            // BUGFIX: Should check for `i + 1 < DMAXY` first.
            let right = dun(ctx, i + 1, j);
            let t = &mut ctx.gendung.dTransVal;
            if d == 15 && right == 11 {
                t[xx][yy + 1] = v;
                t[xx + 1][yy + 1] = v;
            }
            if d == 10 {
                t[xx + 1][yy] = v;
                t[xx + 1][yy + 1] = v;
            }
            if d == 11 {
                t[xx][yy + 1] = v;
                t[xx + 1][yy + 1] = v;
            }
            if d == 16 {
                t[xx + 1][yy] = v;
                t[xx][yy + 1] = v;
                t[xx + 1][yy + 1] = v;
            }
            xx += 2;
        }
        yy += 2;
    }
}

/// Original: `FixDirtTiles` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::FixDirtTiles()
fn fix_dirt_tiles(ctx: &mut Ctx) {
    for j in 0..DY {
        for i in 0..DX {
            if dun(ctx, i, j) == 13 && dun(ctx, i + 1, j) != 11 {
                set_dun(ctx, i, j, 146);
            }
            if dun(ctx, i, j) == 11 && dun(ctx, i + 1, j) != 11 {
                set_dun(ctx, i, j, 144);
            }
            if dun(ctx, i, j) == 15 && dun(ctx, i + 1, j) != 11 {
                set_dun(ctx, i, j, 148);
            }
            if dun(ctx, i, j) == 10 && dun(ctx, i, j + 1) != 10 {
                set_dun(ctx, i, j, 143);
            }
            if dun(ctx, i, j) == 13 && dun(ctx, i, j + 1) != 10 {
                set_dun(ctx, i, j, 146);
            }
            if dun(ctx, i, j) == 14 && dun(ctx, i, j + 1) != 15 {
                set_dun(ctx, i, j, 147);
            }
        }
    }
}

/// Original: `FixLockout` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::FixLockout()
fn fix_lockout(ctx: &mut Ctx) {
    for j in 0..DY {
        for i in 0..DX {
            if dun(ctx, i, j) == 4 && dun(ctx, i - 1, j) != 3 {
                set_dun(ctx, i, j, 1);
            }
            if dun(ctx, i, j) == 5 && dun(ctx, i, j - 1) != 3 {
                set_dun(ctx, i, j, 2);
            }
        }
    }
    for j in 1..DY - 1 {
        let mut i = 1;
        while i < DX - 1 {
            if ctx.gendung.Protected.test(i, j) {
                i += 1;
                continue;
            }
            if (dun(ctx, i, j) == 2 || dun(ctx, i, j) == 5) && dun(ctx, i, j - 1) == 3 && dun(ctx, i, j + 1) == 3 {
                let mut doorok = false;
                loop {
                    if dun(ctx, i, j) != 2 && dun(ctx, i, j) != 5 {
                        break;
                    }
                    if dun(ctx, i, j - 1) != 3 || dun(ctx, i, j + 1) != 3 {
                        break;
                    }
                    if dun(ctx, i, j) == 5 {
                        doorok = true;
                    }
                    i += 1;
                }
                if !doorok && !ctx.gendung.Protected.test(i - 1, j) {
                    set_dun(ctx, i - 1, j, 5);
                }
            }
            i += 1;
        }
    }
    for j in 1..DX - 1 {
        // check: might be flipped
        let mut i = 1;
        while i < DY - 1 {
            if ctx.gendung.Protected.test(j, i) {
                i += 1;
                continue;
            }
            if (dun(ctx, j, i) == 1 || dun(ctx, j, i) == 4) && dun(ctx, j - 1, i) == 3 && dun(ctx, j + 1, i) == 3 {
                let mut doorok = false;
                loop {
                    if dun(ctx, j, i) != 1 && dun(ctx, j, i) != 4 {
                        break;
                    }
                    if dun(ctx, j - 1, i) != 3 || dun(ctx, j + 1, i) != 3 {
                        break;
                    }
                    if dun(ctx, j, i) == 4 {
                        doorok = true;
                    }
                    i += 1;
                }
                if !doorok && !ctx.gendung.Protected.test(j, i - 1) {
                    set_dun(ctx, j, i - 1, 4);
                }
            }
            i += 1;
        }
    }
}

/// Original: `FixDoors` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::FixDoors()
fn fix_doors(ctx: &mut Ctx) {
    let d = &mut ctx.gendung.dungeon;
    for j in 1..DMAXY {
        for i in 1..DMAXX {
            if d[i][j] == 4 && d[i][j - 1] == 3 {
                d[i][j] = 7;
            }
            if d[i][j] == 5 && d[i - 1][j] == 3 {
                d[i][j] = 9;
            }
        }
    }
}

/// Original: `PlaceStairs` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::PlaceStairs(lvl_entry entry)
fn place_stairs(ctx: &mut Ctx, entry: lvl_entry) -> bool {
    // Place stairs up
    let Some(position) = place_mini_set(ctx, &USTAIRS, 199, false) else {
        return false;
    };
    if entry == ENTRY_MAIN {
        ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(5, 4);
    }
    // Place stairs down
    let Some(position) = place_mini_set(ctx, &DSTAIRS, 199, false) else {
        return false;
    };
    if entry == ENTRY_PREV {
        ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(4, 6);
    }
    // Place town warp stairs
    if ctx.gendung.currlevel == 5 {
        let Some(position) = place_mini_set(ctx, &WARPSTAIRS, 199, false) else {
            return false;
        };
        if entry == ENTRY_TWARPDN {
            ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(5, 4);
        }
    }
    true
}

/// Original: `GenerateLevel` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::GenerateLevel(lvl_entry entry)
fn generate_level(ctx: &mut Ctx, entry: lvl_entry) {
    load_quest_set_pieces(ctx);
    loop {
        ctx.drlg_l2.nRoomCnt = 0;
        init_dungeon_flags(ctx);
        drlg_init_trans(ctx);
        if !create_dungeon(ctx) {
            continue;
        }
        fix_tiles_patterns(ctx);
        let p = ctx.gendung.SetPieceRoom.position;
        set_set_piece_room(ctx, p, 3);
        flood_transparency_values(ctx, 3);
        fix_transparency(ctx);
        if place_stairs(ctx, entry) {
            break;
        }
    }
    free_quest_set_pieces(ctx);
    fix_lockout(ctx);
    fix_doors(ctx);
    fix_dirt_tiles(ctx);
    drlg_place_theme_rooms(ctx, 6, 10, 3, 0, false);
    for m in [&CTRDOOR1, &CTRDOOR2, &CTRDOOR3, &CTRDOOR4, &CTRDOOR5, &CTRDOOR6, &CTRDOOR7, &CTRDOOR8] {
        place_mini_set_random(ctx, m, 100);
    }
    for m in [&VARCH33, &VARCH34, &VARCH35, &VARCH36, &VARCH37, &VARCH38, &VARCH39, &VARCH40] {
        place_mini_set_random(ctx, m, 100);
    }
    for m in [
        &VARCH1, &VARCH2, &VARCH3, &VARCH4, &VARCH5, &VARCH6, &VARCH7, &VARCH8, &VARCH9, &VARCH10, &VARCH11, &VARCH12, &VARCH13, &VARCH14, &VARCH15, &VARCH16, &VARCH17, &VARCH18, &VARCH19, &VARCH20, &VARCH21,
        &VARCH22, &VARCH23, &VARCH24, &VARCH25, &VARCH26, &VARCH27, &VARCH28, &VARCH29, &VARCH30, &VARCH31, &VARCH32,
    ] {
        place_mini_set_random(ctx, m, 100);
    }
    for m in [
        &HARCH1, &HARCH2, &HARCH3, &HARCH4, &HARCH5, &HARCH6, &HARCH7, &HARCH8, &HARCH9, &HARCH10, &HARCH11, &HARCH12, &HARCH13, &HARCH14, &HARCH15, &HARCH16, &HARCH17, &HARCH18, &HARCH19, &HARCH20, &HARCH21,
        &HARCH22, &HARCH23, &HARCH24, &HARCH25, &HARCH26, &HARCH27, &HARCH28, &HARCH29, &HARCH30, &HARCH31, &HARCH32, &HARCH33, &HARCH34, &HARCH35, &HARCH36, &HARCH37, &HARCH38, &HARCH39, &HARCH40,
    ] {
        place_mini_set_random(ctx, m, 100);
    }
    place_mini_set_random(ctx, &CRUSHCOL, 99);
    place_mini_set_random_1x1(ctx, 1, 80, 10);
    place_mini_set_random_1x1(ctx, 1, 81, 10);
    place_mini_set_random_1x1(ctx, 1, 82, 10);
    place_mini_set_random_1x1(ctx, 2, 84, 10);
    place_mini_set_random_1x1(ctx, 2, 85, 10);
    place_mini_set_random_1x1(ctx, 2, 86, 10);
    place_mini_set_random_1x1(ctx, 8, 87, 50);
    place_mini_set_random(ctx, &PANCREAS1, 1);
    place_mini_set_random(ctx, &PANCREAS2, 1);
    for (m, p) in [(&BIG1, 3), (&BIG2, 3), (&BIG3, 3), (&BIG4, 3), (&BIG5, 3), (&BIG6, 20), (&BIG7, 20), (&BIG8, 3), (&BIG9, 20), (&BIG10, 20)] {
        place_mini_set_random(ctx, m, p);
    }
    substitution(ctx);
    apply_shadows_patterns(ctx);
    ctx.gendung.pdungeon = ctx.gendung.dungeon;
    let p = ctx.gendung.SetPieceRoom.position;
    crate::quests::drlg_check_quests(ctx, p);
}

/// Original: `Pass3` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::Pass3()
fn pass3(ctx: &mut Ctx) {
    drlg_l_pass3(ctx, 12 - 1);
    init_dungeon_pieces(ctx);
}

/// Original: `devilution::CreateL2Dungeon` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::CreateL2Dungeon(uint32_t rseed, lvl_entry entry)
pub fn create_l2_dungeon(ctx: &mut Ctx, rseed: u32, entry: lvl_entry) {
    ctx.rng.set_rnd_seed(rseed);
    generate_level(ctx, entry);
    pass3(ctx);
}

/// Original: `devilution::LoadPreL2Dungeon` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::LoadPreL2Dungeon(const char *path)
pub fn load_pre_l2_dungeon(ctx: &mut Ctx, path: &str) {
    ctx.gendung.dungeon = [[12; DMAXY]; DMAXX];
    let dun_data = load_u16_file(ctx, path);
    place_dun_tiles(ctx, &dun_data, Point::new(0, 0), 3);
    ctx.gendung.pdungeon = ctx.gendung.dungeon;
}

/// Original: `devilution::LoadL2Dungeon` (levels/drlg_l2.cpp).
// @port levels/drlg_l2.cpp|devilution::LoadL2Dungeon(const char *path, Point spawn)
pub fn load_l2_dungeon(ctx: &mut Ctx, path: &str, spawn: Point) {
    load_dungeon_base(ctx, path, spawn, 3, 12);
    pass3(ctx);
    crate::objects::add_l2_objs(ctx, 0, 0, MAXDUNX as i32, MAXDUNY as i32);
}
