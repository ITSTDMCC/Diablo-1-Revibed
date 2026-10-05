//! `Source/levels/gendung.cpp`: dungeon globals shared by the level generators, plus the tile
//! map helpers (transparency, set pieces, theme rooms).
//!
//! Game-state fields keep the original C++ names so ported code reads like the original.

#![allow(non_snake_case)]

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::geometry::{Direction, Displacement, Point, Rectangle, Size};
use crate::enums::*;

pub const DMAXX: usize = 40;
pub const DMAXY: usize = 40;
pub const MAXDUNX: usize = 16 + DMAXX * 2 + 16;
pub const MAXDUNY: usize = 16 + DMAXY * 2 + 16;
pub const MAXTHEMES: usize = 50;
pub const MAXTILES: usize = 1379;

/// `dungeon_type`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, PartialOrd, Ord, Hash)]
#[repr(i8)]
pub enum DungeonType {
    #[default]
    Town = 0,
    Cathedral = 1,
    Catacombs = 2,
    Caves = 3,
    Hell = 4,
    Nest = 5,
    Crypt = 6,
    None = -1,
}

impl DungeonType {
    pub fn from_i8(v: i8) -> DungeonType {
        match v {
            0 => DungeonType::Town,
            1 => DungeonType::Cathedral,
            2 => DungeonType::Catacombs,
            3 => DungeonType::Caves,
            4 => DungeonType::Hell,
            5 => DungeonType::Nest,
            6 => DungeonType::Crypt,
            _ => DungeonType::None,
        }
    }
}

/// The C `dungeon_type` enumerators, for data tables that store the raw value.
pub mod dtype {
    pub const DTYPE_TOWN: i8 = 0;
    pub const DTYPE_CATHEDRAL: i8 = 1;
    pub const DTYPE_CATACOMBS: i8 = 2;
    pub const DTYPE_CAVES: i8 = 3;
    pub const DTYPE_HELL: i8 = 4;
    pub const DTYPE_NEST: i8 = 5;
    pub const DTYPE_CRYPT: i8 = 6;
    pub const DTYPE_NONE: i8 = -1;
}

/// Original: `devilution::IsArenaLevel` (levels/gendung.h).
// @port levels/gendung.h|devilution::IsArenaLevel(_setlevels setLevel) sha=2aac6ab36da4
pub fn is_arena_level(set_level: _setlevels) -> bool {
    matches!(set_level, SL_ARENA_CHURCH | SL_ARENA_HELL | SL_ARENA_CIRCLE_OF_LIFE)
}

/// `THEME_LOC`
#[derive(Clone, Copy, Debug, Default)]
pub struct THEME_LOC {
    pub room: Rectangle,
    pub ttval: i16,
}

/// `MegaTile`: four micro tile indices.
#[derive(Clone, Copy, Debug, Default)]
pub struct MegaTile {
    pub micro1: u16,
    pub micro2: u16,
    pub micro3: u16,
    pub micro4: u16,
}

/// `MICROS`
#[derive(Clone, Copy, Debug, Default)]
pub struct MICROS {
    pub mt: [u16; 16],
}

/// `ShadowStruct`
#[derive(Clone, Copy, Debug, Default)]
pub struct ShadowStruct {
    pub strig: u8,
    pub s1: u8,
    pub s2: u8,
    pub s3: u8,
    pub nv1: u8,
    pub nv2: u8,
    pub nv3: u8,
}

/// `Bitset2d<DMAXX, DMAXY>`: bit `y * DMAXX + x`. As in the original (whose `size_t` index wraps),
/// coordinates one step outside a row land in the neighbouring row; an index outside the whole
/// set panics where `std::bitset` throws `std::out_of_range`.
#[derive(Clone)]
pub struct Bitset2d {
    bits: [bool; DMAXX * DMAXY],
}

impl Default for Bitset2d {
    fn default() -> Self {
        Bitset2d { bits: [false; DMAXX * DMAXY] }
    }
}

impl Bitset2d {
    // @port utils/bitset2d.hpp|devilution::Bitset2d::index(size_t x, size_t y) sha=ce085462f20c
    fn index(x: i32, y: i32) -> usize {
        let i = y as i64 * DMAXX as i64 + x as i64;
        assert!((0..(DMAXX * DMAXY) as i64).contains(&i), "bitset::test: out_of_range");
        i as usize
    }
    // @port utils/bitset2d.hpp|devilution::Bitset2d::test(size_t x, size_t y) sha=0471f3c65f2d
    pub fn test(&self, x: i32, y: i32) -> bool {
        self.bits[Self::index(x, y)]
    }
    pub fn set(&mut self, x: i32, y: i32) {
        self.bits[Self::index(x, y)] = true;
    }
    // @port utils/bitset2d.hpp|devilution::Bitset2d::set(size_t x, size_t y, bool value = true) sha=b1f93cedefbc
    pub fn set_value(&mut self, x: i32, y: i32, v: bool) {
        self.bits[Self::index(x, y)] = v;
    }
    /// `reset(x, y)`
    // @port utils/bitset2d.hpp|devilution::Bitset2d::reset(size_t x, size_t y) sha=cca27b7368b6
    pub fn reset_at(&mut self, x: i32, y: i32) {
        self.bits[Self::index(x, y)] = false;
    }
    // @port utils/bitset2d.hpp|devilution::Bitset2d::reset() sha=ceab7821f115
    pub fn reset(&mut self) {
        self.bits = [false; DMAXX * DMAXY];
    }
    /// `count`
    // @port utils/bitset2d.hpp|devilution::Bitset2d::count() sha=96b9167bf434
    pub fn count(&self) -> usize {
        self.bits.iter().filter(|&&b| b).count()
    }
}

pub type DunArray<T> = [[T; MAXDUNY]; MAXDUNX];

/// `Miniset`: a pattern to search for and the tiles to put in its place ([y][x]).
pub struct Miniset {
    pub size: Size,
    pub search: [[u8; 6]; 6],
    pub replace: [[u8; 6]; 6],
}

impl Miniset {
    /// Original: `Miniset::matches` (levels/gendung.h). `respectProtected` defaults to true.
    // @port levels/gendung.h|devilution::Miniset::matches(WorldTilePosition position, bool respectProtected = true) sha=c3c46e61ee7c
    pub fn matches(&self, g: &GendungState, position: Point, respect_protected: bool) -> bool {
        for yy in 0..self.size.height {
            for xx in 0..self.size.width {
                let s = self.search[yy as usize][xx as usize];
                if s != 0 && g.dungeon[(xx + position.x) as usize][(yy + position.y) as usize] != s {
                    return false;
                }
                if respect_protected && g.Protected.test(xx + position.x, yy + position.y) {
                    return false;
                }
            }
        }
        true
    }

    /// Original: `Miniset::place` (levels/gendung.h). `protect` defaults to false.
    // @port levels/gendung.h|devilution::Miniset::place(WorldTilePosition position, bool protect = false) sha=380d4a1e27a1
    pub fn place(&self, g: &mut GendungState, position: Point, protect: bool) {
        for y in 0..self.size.height {
            for x in 0..self.size.width {
                let r = self.replace[y as usize][x as usize];
                if r == 0 {
                    continue;
                }
                g.dungeon[(x + position.x) as usize][(y + position.y) as usize] = r;
                if protect {
                    g.Protected.set(x + position.x, y + position.y);
                }
            }
        }
    }
}

/// Globals of gendung.cpp.
pub struct GendungState {
    /// `leveltype`
    pub leveltype: DungeonType,
    pub DungeonMask: Bitset2d,
    pub dungeon: [[u8; DMAXY]; DMAXX],
    pub pdungeon: [[u8; DMAXY]; DMAXX],
    pub Protected: Bitset2d,
    pub SetPieceRoom: Rectangle,
    pub SetPiece: Rectangle,
    /// `pSetPiece`: the DUN file of the active set piece (u16 little-endian words).
    pub pSetPiece: Option<Vec<u16>>,
    pub pSpecialCels: Option<ClxSpriteList>,
    pub pMegaTiles: Option<Vec<MegaTile>>,
    pub pDungeonCels: Option<Vec<u8>>,
    pub SOLData: [TileProperties; MAXTILES],
    pub dminPosition: Point,
    pub dmaxPosition: Point,
    pub currlevel: u8,
    pub setlevel: bool,
    pub setlvlnum: _setlevels,
    pub setlvltype: DungeonType,
    pub ViewPosition: Point,
    pub MicroTileLen: u8,
    pub TransVal: i8,
    pub TransList: [bool; 256],
    pub dPiece: Box<DunArray<u16>>,
    pub DPieceMicros: Box<[MICROS; MAXTILES]>,
    pub dTransVal: Box<DunArray<i8>>,
    pub dLight: Box<DunArray<u8>>,
    pub dPreLight: Box<DunArray<u8>>,
    pub dFlags: Box<DunArray<DungeonFlag>>,
    pub dPlayer: Box<DunArray<i8>>,
    pub dMonster: Box<DunArray<i16>>,
    pub dCorpse: Box<DunArray<i8>>,
    pub dObject: Box<DunArray<i8>>,
    pub dSpecial: Box<DunArray<i8>>,
    pub themeCount: i32,
    pub themeLoc: [THEME_LOC; MAXTHEMES],
}

impl Default for GendungState {
    fn default() -> Self {
        GendungState {
            leveltype: DungeonType::Town,
            DungeonMask: Bitset2d::default(),
            dungeon: [[0; DMAXY]; DMAXX],
            pdungeon: [[0; DMAXY]; DMAXX],
            Protected: Bitset2d::default(),
            SetPieceRoom: Rectangle::default(),
            SetPiece: Rectangle::default(),
            pSetPiece: None,
            pSpecialCels: None,
            pMegaTiles: None,
            pDungeonCels: None,
            SOLData: [TileProperties(0); MAXTILES],
            dminPosition: Point::default(),
            dmaxPosition: Point::default(),
            currlevel: 0,
            setlevel: false,
            setlvlnum: SL_NONE,
            setlvltype: DungeonType::Town,
            ViewPosition: Point::default(),
            MicroTileLen: 0,
            TransVal: 0,
            TransList: [false; 256],
            dPiece: Box::new([[0; MAXDUNY]; MAXDUNX]),
            DPieceMicros: Box::new([MICROS::default(); MAXTILES]),
            dTransVal: Box::new([[0; MAXDUNY]; MAXDUNX]),
            dLight: Box::new([[0; MAXDUNY]; MAXDUNX]),
            dPreLight: Box::new([[0; MAXDUNY]; MAXDUNX]),
            dFlags: Box::new([[DungeonFlag(0); MAXDUNY]; MAXDUNX]),
            dPlayer: Box::new([[0; MAXDUNY]; MAXDUNX]),
            dMonster: Box::new([[0; MAXDUNY]; MAXDUNX]),
            dCorpse: Box::new([[0; MAXDUNY]; MAXDUNX]),
            dObject: Box::new([[0; MAXDUNY]; MAXDUNX]),
            dSpecial: Box::new([[0; MAXDUNY]; MAXDUNX]),
            themeCount: 0,
            themeLoc: [THEME_LOC::default(); MAXTHEMES],
        }
    }
}

/// `leveltype`
pub fn leveltype(ctx: &Ctx) -> DungeonType {
    ctx.gendung.leveltype
}

/// `leveltype == DTYPE_TOWN`
pub fn leveltype_is_town(ctx: &Ctx) -> bool {
    ctx.gendung.leveltype == DungeonType::Town
}

/// Original: `LoadMinData` (levels/gendung.cpp): the level's .min file as u16 words.
// @port levels/gendung.cpp|devilution::LoadMinData(size_t &tileCount) sha=fdc86926ed6b
fn load_min_data(ctx: &mut Ctx) -> Vec<u16> {
    let path = match ctx.gendung.leveltype {
        DungeonType::Town => {
            if ctx.init.gb_is_hellfire {
                "nlevels\\towndata\\town.min"
            } else {
                "levels\\towndata\\town.min"
            }
        }
        DungeonType::Cathedral => "levels\\l1data\\l1.min",
        DungeonType::Catacombs => "levels\\l2data\\l2.min",
        DungeonType::Caves => "levels\\l3data\\l3.min",
        DungeonType::Hell => "levels\\l4data\\l4.min",
        DungeonType::Nest => "nlevels\\l6data\\l6.min",
        DungeonType::Crypt => "nlevels\\l5data\\l5.min",
        DungeonType::None => crate::appfat::app_fatal(ctx, "LoadMinData"),
    };
    load_u16_file(ctx, path)
}

/// `LoadFileInMem<uint16_t>`: a whole file as little-endian u16 words.
pub fn load_u16_file(ctx: &mut Ctx, path: &str) -> Vec<u16> {
    let bytes = crate::engine::load_file::load_file_in_mem(ctx, path).unwrap_or_default();
    bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

/// Original: `GetSizeForThemeRoom` (levels/gendung.cpp). Coordinates are `uint8_t` in the original.
// @port levels/gendung.cpp|devilution::GetSizeForThemeRoom(uint8_t floor, WorldTilePosition origin, WorldTileCoord minSize, WorldTileCoord maxSize) sha=55ca3fc26b3b
fn get_size_for_theme_room(ctx: &Ctx, floor: u8, origin: Point, min_size: i32, max_size: i32) -> Option<Size> {
    let g = &ctx.gendung;
    if origin.x + max_size > DMAXX as i32 && origin.y + max_size > DMAXY as i32 {
        return None; // Original broken bounds check, avoids lower right corner
    }
    if is_near_theme_room(ctx, origin) {
        return None;
    }
    let max_width = max_size.min(DMAXX as i32 - origin.x);
    let max_height = max_size.min(DMAXY as i32 - origin.y);
    let mut room = Size::new(max_width, max_height);
    for i in 0..max_size {
        let mut width = if i < room.height { i } else { 0 };
        if i < max_height {
            while width < room.width {
                if g.dungeon[(origin.x + width) as usize][(origin.y + i) as usize] != floor {
                    break;
                }
                width += 1;
            }
        }
        let mut height = if i < room.width { i } else { 0 };
        if i < max_width {
            while height < room.height {
                if g.dungeon[(origin.x + i) as usize][(origin.y + height) as usize] != floor {
                    break;
                }
                height += 1;
            }
        }
        if width < min_size || height < min_size {
            if i < min_size {
                return None;
            }
            break;
        }
        room = Size::new(room.width.min(width), room.height.min(height));
    }
    // uint8_t arithmetic
    Some(Size::new((room.width - 2) & 0xff, (room.height - 2) & 0xff))
}

/// Original: `CreateThemeRoom` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::CreateThemeRoom(int themeIndex) sha=84f0cde2a715
fn create_theme_room(ctx: &mut Ctx, theme_index: usize) {
    let lt = ctx.gendung.leveltype;
    let room = ctx.gendung.themeLoc[theme_index].room;
    let lx = room.position.x as usize;
    let ly = room.position.y as usize;
    let hx = lx + room.size.width as usize;
    let hy = ly + room.size.height as usize;
    let caves = matches!(lt, DungeonType::Caves | DungeonType::Nest);
    {
        let d = &mut ctx.gendung.dungeon;
        for yy in ly..hy {
            for xx in lx..hx {
                let edge_y = yy == ly || yy == hy - 1;
                let edge_x = xx == lx || xx == hx - 1;
                if lt == DungeonType::Catacombs {
                    d[xx][yy] = if edge_y { 2 } else if edge_x { 1 } else { 3 };
                }
                if caves {
                    d[xx][yy] = if edge_y { 134 } else if edge_x { 137 } else { 7 };
                }
                if lt == DungeonType::Hell {
                    d[xx][yy] = if edge_y { 2 } else if edge_x { 1 } else { 6 };
                }
            }
        }
        if lt == DungeonType::Catacombs {
            d[lx][ly] = 8;
            d[hx - 1][ly] = 7;
            d[lx][hy - 1] = 9;
            d[hx - 1][hy - 1] = 6;
        }
        if caves {
            d[lx][ly] = 150;
            d[hx - 1][ly] = 151;
            d[lx][hy - 1] = 152;
            d[hx - 1][hy - 1] = 138;
        }
        if lt == DungeonType::Hell {
            d[lx][ly] = 9;
            d[hx - 1][ly] = 16;
            d[lx][hy - 1] = 15;
            d[hx - 1][hy - 1] = 12;
        }
    }
    if lt == DungeonType::Catacombs {
        if ctx.rng.flip_coin(2) {
            ctx.gendung.dungeon[hx - 1][(ly + hy) / 2] = 4;
        } else {
            ctx.gendung.dungeon[(lx + hx) / 2][hy - 1] = 5;
        }
    }
    if caves {
        if ctx.rng.flip_coin(2) {
            ctx.gendung.dungeon[hx - 1][(ly + hy) / 2] = 147;
        } else {
            ctx.gendung.dungeon[(lx + hx) / 2][hy - 1] = 146;
        }
    }
    if lt == DungeonType::Hell {
        if ctx.rng.flip_coin(2) {
            let yy = (ly + hy) / 2;
            let d = &mut ctx.gendung.dungeon;
            d[hx - 1][yy - 1] = 53;
            d[hx - 1][yy] = 6;
            d[hx - 1][yy + 1] = 52;
            d[hx - 2][yy - 1] = 54;
        } else {
            let xx = (lx + hx) / 2;
            let d = &mut ctx.gendung.dungeon;
            d[xx - 1][hy - 1] = 57;
            d[xx][hy - 1] = 6;
            d[xx + 1][hy - 1] = 56;
            d[xx][hy - 2] = 59;
            d[xx - 1][hy - 2] = 58;
        }
    }
}

/// Original: `IsFloor` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::IsFloor(Point p, uint8_t floorID) sha=0ebe0fc3f654
fn is_floor(g: &GendungState, p: Point, floor_id: u8) -> bool {
    let i = (p.x - 16) / 2;
    let j = (p.y - 16) / 2;
    if i < 0 || i >= DMAXX as i32 {
        return false;
    }
    if j < 0 || j >= DMAXY as i32 {
        return false;
    }
    g.dungeon[i as usize][j as usize] == floor_id
}

/// Original: `FillTransparencyValues` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::FillTransparencyValues(Point floor, uint8_t floorID) sha=83d84167052a
fn fill_transparency_values(g: &mut GendungState, floor: Point, floor_id: u8) {
    const ALL: [Direction; 8] = [
        Direction::North,
        Direction::South,
        Direction::East,
        Direction::West,
        Direction::NorthEast,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::SouthWest,
    ];
    for dir in ALL {
        let adjacent = floor + dir;
        if !is_floor(g, adjacent, floor_id) {
            g.dTransVal[adjacent.x as usize][adjacent.y as usize] = g.TransVal;
        }
    }
    g.dTransVal[floor.x as usize][floor.y as usize] = g.TransVal;
}

/// Original: `FindTransparencyValues` (levels/gendung.cpp): span flood fill including diagonals.
// @port levels/gendung.cpp|devilution::FindTransparencyValues(Point floor, uint8_t floorID) sha=d9b2825decf8
fn find_transparency_values(g: &mut GendungState, floor: Point, floor_id: u8) {
    let mut seed_stack: Vec<(i32, i32, i32, i32)> = vec![(floor.x, floor.x + 1, floor.y, 1)];
    let is_inside = |g: &GendungState, x: i32, y: i32| -> bool {
        if g.dTransVal[x as usize][y as usize] != 0 {
            return false;
        }
        is_floor(g, Point::new(x, y), floor_id)
    };
    let left = Displacement::new(-1, 0);
    let right = Displacement::new(1, 0);
    let check_diagonals = |g: &GendungState, stack: &mut Vec<(i32, i32, i32, i32)>, p: Point, direction: Displacement| {
        let up = p + Displacement::new(0, -1);
        let up_over = up + direction;
        if !is_inside(g, up.x, up.y) && is_inside(g, up_over.x, up_over.y) {
            stack.push((up_over.x, up_over.x + 1, up_over.y, -1));
        }
        let down = p + Displacement::new(0, 1);
        let down_over = down + direction;
        if !is_inside(g, down.x, down.y) && is_inside(g, down_over.x, down_over.y) {
            stack.push((down_over.x, down_over.x + 1, down_over.y, 1));
        }
    };
    while let Some((scan_start, scan_end, y, dy)) = seed_stack.pop() {
        let mut scan_left = scan_start;
        if is_inside(g, scan_left, y) {
            while is_inside(g, scan_left - 1, y) {
                fill_transparency_values(g, Point::new(scan_left - 1, y), floor_id);
                scan_left -= 1;
            }
            check_diagonals(g, &mut seed_stack, Point::new(scan_left, y), left);
        }
        if scan_left < scan_start {
            seed_stack.push((scan_left, scan_start - 1, y - dy, -dy));
        }
        let mut scan_right = scan_start;
        while scan_right < scan_end {
            while is_inside(g, scan_right, y) {
                fill_transparency_values(g, Point::new(scan_right, y), floor_id);
                scan_right += 1;
            }
            seed_stack.push((scan_left, scan_right - 1, y + dy, dy));
            if scan_right - 1 > scan_end {
                seed_stack.push((scan_end + 1, scan_right - 1, y - dy, -dy));
            }
            if scan_left < scan_right {
                check_diagonals(g, &mut seed_stack, Point::new(scan_right - 1, y), right);
            }
            while scan_right < scan_end && !is_inside(g, scan_right, y) {
                scan_right += 1;
            }
            scan_left = scan_right;
            if scan_left < scan_end {
                check_diagonals(g, &mut seed_stack, Point::new(scan_left, y), left);
            }
        }
    }
}

/// Original: `InitGlobals` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::InitGlobals() sha=36f527fa9f1f
fn init_globals(ctx: &mut Ctx) {
    {
        let g = &mut ctx.gendung;
        *g.dFlags = [[DungeonFlag(0); MAXDUNY]; MAXDUNX];
        *g.dPlayer = [[0; MAXDUNY]; MAXDUNX];
        *g.dMonster = [[0; MAXDUNY]; MAXDUNX];
        *g.dCorpse = [[0; MAXDUNY]; MAXDUNX];
        *g.dObject = [[0; MAXDUNY]; MAXDUNX];
        *g.dSpecial = [[0; MAXDUNY]; MAXDUNX];
        let default_light = if g.leveltype == DungeonType::Town { 0 } else { 15 };
        *g.dLight = [[default_light; MAXDUNY]; MAXDUNX];
    }
    *ctx.items.dItem = [[0; MAXDUNY]; MAXDUNX];
    drlg_init_trans(ctx);
    let g = &mut ctx.gendung;
    g.dminPosition = Point::new(0, 0).mega_to_world();
    g.dmaxPosition = Point::new(40, 40).mega_to_world();
    g.SetPieceRoom = Rectangle::default();
    g.SetPiece = Rectangle::default();
}

/// Original: `devilution::GetLevelType` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::GetLevelType(int level) sha=463b28fb246e
pub fn get_level_type(level: i32) -> DungeonType {
    match level {
        0 => DungeonType::Town,
        1..=4 => DungeonType::Cathedral,
        5..=8 => DungeonType::Catacombs,
        9..=12 => DungeonType::Caves,
        13..=16 => DungeonType::Hell,
        17..=20 => DungeonType::Nest,
        21..=24 => DungeonType::Crypt,
        l if l < 0 => DungeonType::Cathedral,
        _ => DungeonType::None,
    }
}

/// Original: `devilution::CreateDungeon` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::CreateDungeon(uint32_t rseed, lvl_entry entry) sha=c16bba2351e0
pub fn create_dungeon(ctx: &mut Ctx, rseed: u32, entry: lvl_entry) {
    init_globals(ctx);
    match ctx.gendung.leveltype {
        DungeonType::Town => crate::levels::town::create_town(ctx, entry),
        DungeonType::Cathedral | DungeonType::Crypt => crate::levels::drlg_l1::create_l5_dungeon(ctx, rseed, entry),
        DungeonType::Catacombs => crate::levels::drlg_l2::create_l2_dungeon(ctx, rseed, entry),
        DungeonType::Caves | DungeonType::Nest => crate::levels::drlg_l3::create_l3_dungeon(ctx, rseed, entry),
        DungeonType::Hell => crate::levels::drlg_l4::create_l4_dungeon(ctx, rseed, entry),
        DungeonType::None => crate::appfat::app_fatal(ctx, "Invalid level type"),
    }
    let sp = ctx.gendung.SetPiece;
    make_set_pc(ctx, sp);
}

/// Original: `devilution::TileHasAny` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::TileHasAny(int tileId, TileProperties property) sha=9f6e85afc2ff
pub fn tile_has_any(ctx: &Ctx, tile_id: i32, property: TileProperties) -> bool {
    ctx.gendung.SOLData[tile_id as usize].has_any_of(property)
}

/// Original: `devilution::LoadLevelSOLData` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::LoadLevelSOLData() sha=9554e0202e82
pub fn load_level_sol_data(ctx: &mut Ctx) {
    let path = match ctx.gendung.leveltype {
        DungeonType::Town => {
            if ctx.init.gb_is_hellfire {
                "nlevels\\towndata\\town.sol"
            } else {
                "levels\\towndata\\town.sol"
            }
        }
        DungeonType::Cathedral => "levels\\l1data\\l1.sol",
        DungeonType::Catacombs => "levels\\l2data\\l2.sol",
        DungeonType::Caves => "levels\\l3data\\l3.sol",
        DungeonType::Hell => "levels\\l4data\\l4.sol",
        DungeonType::Nest => "nlevels\\l6data\\l6.sol",
        DungeonType::Crypt => "nlevels\\l5data\\l5.sol",
        DungeonType::None => crate::appfat::app_fatal(ctx, "LoadLevelSOLData"),
    };
    let mut raw = [0u8; MAXTILES];
    crate::engine::load_file::load_file_in_mem_exact(ctx, path, &mut raw);
    let sol = &mut ctx.gendung.SOLData;
    for (d, s) in sol.iter_mut().zip(raw.iter()) {
        *d = TileProperties(*s);
    }
    let bl = TileProperties::BlockLight;
    let bm = TileProperties::BlockMissile;
    match ctx.gendung.leveltype {
        DungeonType::Cathedral => {
            // Fix incorrectly marked arched tiles
            for t in [9, 15, 16, 20, 21] {
                sol[t] |= bl | bm;
            }
            sol[27] |= bm;
            sol[28] |= bm;
            for t in [51, 56, 58, 61, 63, 65, 72, 208, 247, 253, 257, 323] {
                sol[t] |= bl | bm;
            }
            sol[403] |= bl;
            // Fix incorrectly marked pillar tile
            sol[24] |= bl;
            // Fix incorrectly marked wall tile
            sol[450] |= bl | bm;
        }
        DungeonType::Hell => {
            sol[210] = TileProperties::None; // Tile is incorrectly marked as being solid
        }
        _ => {}
    }
}

/// Original: `devilution::SetDungeonMicros` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::SetDungeonMicros() sha=153f7818b05e
pub fn set_dungeon_micros(ctx: &mut Ctx) {
    let mut blocks = 10usize;
    ctx.gendung.MicroTileLen = 10;
    if ctx.gendung.leveltype == DungeonType::Town {
        ctx.gendung.MicroTileLen = 16;
        blocks = 16;
    } else if ctx.gendung.leveltype == DungeonType::Hell {
        ctx.gendung.MicroTileLen = 12;
        blocks = 16;
    }
    let level_pieces = load_min_data(ctx);
    let tile_count = level_pieces.len();
    for i in 0..(tile_count / blocks).min(MAXTILES) {
        let pieces = &level_pieces[blocks * i..];
        for block in 0..blocks {
            ctx.gendung.DPieceMicros[i].mt[block] = pieces[blocks - 2 + (block & 1) - (block & 0xE)];
        }
    }
}

/// Original: `devilution::DRLG_InitTrans` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::DRLG_InitTrans() sha=6706f194072d
pub fn drlg_init_trans(ctx: &mut Ctx) {
    *ctx.gendung.dTransVal = [[0; MAXDUNY]; MAXDUNX];
    ctx.gendung.TransList = [false; 256];
    ctx.gendung.TransVal = 1;
}

/// Original: `devilution::DRLG_RectTrans` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::DRLG_RectTrans(WorldTileRectangle area) sha=be25ade10a93
pub fn drlg_rect_trans(ctx: &mut Ctx, area: Rectangle) {
    let g = &mut ctx.gendung;
    let (p, s) = (area.position, area.size);
    for j in p.y..=p.y + s.height {
        for i in p.x..=p.x + s.width {
            g.dTransVal[i as usize][j as usize] = g.TransVal;
        }
    }
    g.TransVal = g.TransVal.wrapping_add(1);
}

/// Original: `devilution::DRLG_MRectTrans(WorldTileRectangle)` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::DRLG_MRectTrans(WorldTileRectangle area) sha=89c75ae28729
pub fn drlg_m_rect_trans(ctx: &mut Ctx, area: Rectangle) {
    // WorldTileSize is uint8_t: size * 2 - 1
    let size = Size::new((area.size.width * 2 - 1) & 0xff, (area.size.height * 2 - 1) & 0xff);
    drlg_rect_trans(ctx, Rectangle::new(area.position.mega_to_world() + Displacement::new(1, 1), size));
}

/// Original: `devilution::DRLG_MRectTrans(WorldTilePosition, WorldTilePosition)` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::DRLG_MRectTrans(WorldTilePosition origin, WorldTilePosition extent) sha=143ce76f70fd
pub fn drlg_m_rect_trans_points(ctx: &mut Ctx, origin: Point, extent: Point) {
    drlg_m_rect_trans(ctx, Rectangle::new(origin, Size::new((extent.x - origin.x) & 0xff, (extent.y - origin.y) & 0xff)));
}

/// Original: `devilution::DRLG_CopyTrans` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::DRLG_CopyTrans(int sx, int sy, int dx, int dy) sha=b67e13f7e3a5
pub fn drlg_copy_trans(ctx: &mut Ctx, sx: i32, sy: i32, dx: i32, dy: i32) {
    let g = &mut ctx.gendung;
    g.dTransVal[dx as usize][dy as usize] = g.dTransVal[sx as usize][sy as usize];
}

/// Original: `devilution::LoadTransparency` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::LoadTransparency(const uint16_t *dunData) sha=e637350f561c
pub fn load_transparency(ctx: &mut Ctx, dun_data: &[u16]) {
    let mut width = dun_data[0] as usize;
    let mut height = dun_data[1] as usize;
    let layer2_offset = 2 + width * height;
    width *= 2;
    height *= 2;
    let transparent_layer = &dun_data[layer2_offset + width * height * 3..];
    let mut k = 0;
    for j in 0..height {
        for i in 0..width {
            ctx.gendung.dTransVal[16 + i][16 + j] = transparent_layer[k] as i8;
            k += 1;
        }
    }
}

/// Original: `devilution::LoadDungeonBase` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::LoadDungeonBase(const char *path, Point spawn, int floorId, int dirtId) sha=1c1aa35d52d7
pub fn load_dungeon_base(ctx: &mut Ctx, path: &str, spawn: Point, floor_id: i32, dirt_id: i32) {
    ctx.gendung.ViewPosition = spawn;
    init_globals(ctx);
    ctx.gendung.dungeon = [[dirt_id as u8; DMAXY]; DMAXX];
    let dun_data = load_u16_file(ctx, path);
    place_dun_tiles(ctx, &dun_data, Point::new(0, 0), floor_id);
    load_transparency(ctx, &dun_data);
    crate::monster::set_map_monsters(ctx, &dun_data, Point::new(0, 0).mega_to_world());
    crate::objects::set_map_objects(ctx, &dun_data, 0, 0);
}

/// Original: `devilution::Make_SetPC` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::Make_SetPC(WorldTileRectangle area) sha=7c9f71e12d71
pub fn make_set_pc(ctx: &mut Ctx, area: Rectangle) {
    let position = area.position.mega_to_world();
    let size = Size::new((area.size.width * 2) & 0xff, (area.size.height * 2) & 0xff);
    for j in 0..size.height {
        for i in 0..size.width {
            ctx.gendung.dFlags[(position.x + i) as usize][(position.y + j) as usize] |= DungeonFlag::Populated;
        }
    }
}

/// Original: `devilution::PlaceMiniSet` (levels/gendung.cpp). Defaults: tries 199, drlg1Quirk false.
// @port levels/gendung.cpp|devilution::PlaceMiniSet(const Miniset &miniset, int tries, bool drlg1Quirk) sha=8ef8ac409bb4
pub fn place_mini_set(ctx: &mut Ctx, miniset: &Miniset, tries: i32, drlg1_quirk: bool) -> Option<Point> {
    let sw = miniset.size.width;
    let sh = miniset.size.height;
    let x = ctx.rng.generate_rnd(DMAXX as i32 - sw);
    let y = ctx.rng.generate_rnd(DMAXY as i32 - sh);
    let mut position = Point::new(x, y);
    let mut i = 0;
    while i < tries {
        if position.x == DMAXX as i32 - sw {
            position.x = 0;
            position.y += 1;
            if position.y == DMAXY as i32 - sh {
                position.y = 0;
            }
        }
        let mut skip = false;
        if drlg1_quirk {
            let mut valid = true;
            if position.x <= 12 {
                position.x += 1;
                valid = false;
            }
            if position.y <= 12 {
                position.y += 1;
                valid = false;
            }
            if !valid {
                skip = true;
            }
        }
        if !skip && !ctx.gendung.SetPieceRoom.contains(position) && miniset.matches(&ctx.gendung, position, true) {
            miniset.place(&mut ctx.gendung, position, false);
            return Some(position);
        }
        i += 1;
        position.x += 1;
    }
    None
}

/// Original: `devilution::PlaceDunTiles` (levels/gendung.cpp). `floorId` defaults to 0.
// @port levels/gendung.cpp|devilution::PlaceDunTiles(const uint16_t *dunData, Point position, int floorId) sha=9f9e8cbea023
pub fn place_dun_tiles(ctx: &mut Ctx, dun_data: &[u16], position: Point, floor_id: i32) {
    let width = dun_data[0] as i32;
    let height = dun_data[1] as i32;
    let tile_layer = &dun_data[2..];
    let g = &mut ctx.gendung;
    for j in 0..height {
        for i in 0..width {
            let tile_id = tile_layer[(j * width + i) as usize] as u8;
            let (x, y) = ((position.x + i) as usize, (position.y + j) as usize);
            if tile_id != 0 {
                g.dungeon[x][y] = tile_id;
                g.Protected.set(position.x + i, position.y + j);
            } else if floor_id != 0 {
                g.dungeon[x][y] = floor_id as u8;
            }
        }
    }
}

/// Original: `devilution::DRLG_PlaceThemeRooms` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::DRLG_PlaceThemeRooms(int minSize, int maxSize, int floor, int freq, bool rndSize) sha=bfefd57387ef
pub fn drlg_place_theme_rooms(ctx: &mut Ctx, min_size: i32, max_size: i32, floor: i32, freq: i32, rnd_size: bool) {
    ctx.gendung.themeCount = 0;
    ctx.gendung.themeLoc[0] = THEME_LOC::default(); // memset of the first element only (original)
    for j in 0..DMAXY as i32 {
        for i in 0..DMAXX as i32 {
            if ctx.gendung.dungeon[i as usize][j as usize] as i32 == floor && ctx.rng.flip_coin(freq as u32) {
                let Some(mut theme_size) = get_size_for_theme_room(ctx, floor as u8, Point::new(i, j), min_size, max_size) else {
                    continue;
                };
                if rnd_size {
                    let min = min_size - 2;
                    let max = max_size - 2;
                    let r = ctx.rng.generate_rnd(theme_size.width - min + 1);
                    theme_size.width = (min + ctx.rng.generate_rnd(r)) & 0xff;
                    if theme_size.width < min || theme_size.width > max {
                        theme_size.width = min;
                    }
                    let r = ctx.rng.generate_rnd(theme_size.height - min + 1);
                    theme_size.height = (min + ctx.rng.generate_rnd(r)) & 0xff;
                    if theme_size.height < min || theme_size.height > max {
                        theme_size.height = min;
                    }
                }
                let tc = ctx.gendung.themeCount as usize;
                let room = Rectangle::new(Point::new(i, j) + Direction::South, theme_size);
                ctx.gendung.themeLoc[tc].room = room;
                if matches!(ctx.gendung.leveltype, DungeonType::Caves | DungeonType::Nest) {
                    let size = Size::new((room.size.width * 2 - 5) & 0xff, (room.size.height * 2 - 5) & 0xff);
                    drlg_rect_trans(ctx, Rectangle::new((room.position + Direction::South).mega_to_world(), size));
                } else {
                    drlg_m_rect_trans(ctx, Rectangle::new(room.position, Size::new((room.size.width - 1) & 0xff, (room.size.height - 1) & 0xff)));
                }
                ctx.gendung.themeLoc[tc].ttval = (ctx.gendung.TransVal as i16) - 1;
                create_theme_room(ctx, tc);
                ctx.gendung.themeCount += 1;
            }
        }
    }
}

/// Original: `devilution::DRLG_HoldThemeRooms` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::DRLG_HoldThemeRooms() sha=90b630d405e9
pub fn drlg_hold_theme_rooms(ctx: &mut Ctx) {
    let g = &mut ctx.gendung;
    for i in 0..g.themeCount as usize {
        let r = g.themeLoc[i].room;
        for y in r.position.y..r.position.y + r.size.height - 1 {
            for x in r.position.x..r.position.x + r.size.width - 1 {
                let xx = (2 * x + 16) as usize;
                let yy = (2 * y + 16) as usize;
                g.dFlags[xx][yy] |= DungeonFlag::Populated;
                g.dFlags[xx + 1][yy] |= DungeonFlag::Populated;
                g.dFlags[xx][yy + 1] |= DungeonFlag::Populated;
                g.dFlags[xx + 1][yy + 1] |= DungeonFlag::Populated;
            }
        }
    }
}

/// Original: `devilution::SetSetPieceRoom` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::SetSetPieceRoom(WorldTilePosition position, int floorId) sha=c23006ba6126
pub fn set_set_piece_room(ctx: &mut Ctx, position: Point, floor_id: i32) {
    let Some(sp) = ctx.gendung.pSetPiece.clone() else { return };
    place_dun_tiles(ctx, &sp, position, floor_id);
    ctx.gendung.SetPiece = Rectangle::new(position, Size::new((sp[0] & 0xff) as i32, (sp[1] & 0xff) as i32));
}

/// Original: `devilution::FreeQuestSetPieces` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::FreeQuestSetPieces() sha=eacb1a5699de
pub fn free_quest_set_pieces(ctx: &mut Ctx) {
    ctx.gendung.pSetPiece = None;
}

/// Original: `devilution::DRLG_LPass3` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::DRLG_LPass3(int lv) sha=f5548ffa0772
pub fn drlg_l_pass3(ctx: &mut Ctx, lv: i32) {
    let g = &mut ctx.gendung;
    let megas = g.pMegaTiles.as_ref().expect("pMegaTiles");
    {
        let mega = megas[lv as usize];
        for j in (0..MAXDUNY).step_by(2) {
            for i in (0..MAXDUNX).step_by(2) {
                g.dPiece[i][j] = mega.micro1;
                g.dPiece[i + 1][j] = mega.micro2;
                g.dPiece[i][j + 1] = mega.micro3;
                g.dPiece[i + 1][j + 1] = mega.micro4;
            }
        }
    }
    let mut yy = 16;
    for j in 0..DMAXY {
        let mut xx = 16;
        for i in 0..DMAXX {
            let tile_id = g.dungeon[i][j] as i32 - 1;
            // A zero tile wraps to the end of the table in the original (read past the array);
            // the shipped generators never leave zeros here.
            let mega = megas[tile_id.max(0) as usize];
            g.dPiece[xx][yy] = mega.micro1;
            g.dPiece[xx + 1][yy] = mega.micro2;
            g.dPiece[xx][yy + 1] = mega.micro3;
            g.dPiece[xx + 1][yy + 1] = mega.micro4;
            xx += 2;
        }
        yy += 2;
    }
}

/// Original: `devilution::IsNearThemeRoom` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::IsNearThemeRoom(WorldTilePosition testPosition) sha=7cad24f9f33e
pub fn is_near_theme_room(ctx: &Ctx, test_position: Point) -> bool {
    let g = &ctx.gendung;
    for i in 0..g.themeCount as usize {
        let r = g.themeLoc[i].room;
        // uint8_t arithmetic on WorldTilePosition / WorldTileSize
        let pos = Point::new((r.position.x - 2) & 0xff, (r.position.y - 2) & 0xff);
        let size = Size::new((r.size.width + 5) & 0xff, (r.size.height + 5) & 0xff);
        let tp = Point::new(test_position.x & 0xff, test_position.y & 0xff);
        if Rectangle::new(pos, size).contains(tp) {
            return true;
        }
    }
    false
}

/// Original: `devilution::InitLevels` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::InitLevels() sha=a7653bb1ae65
pub fn init_levels(ctx: &mut Ctx) {
    ctx.gendung.currlevel = 0;
    ctx.gendung.leveltype = DungeonType::Town;
    ctx.gendung.setlevel = false;
}

/// Original: `devilution::FloodTransparencyValues` (levels/gendung.cpp).
// @port levels/gendung.cpp|devilution::FloodTransparencyValues(uint8_t floorID) sha=26dc2803af1a
pub fn flood_transparency_values(ctx: &mut Ctx, floor_id: u8) {
    let g = &mut ctx.gendung;
    let mut yy = 16;
    for j in 0..DMAXY {
        let mut xx = 16;
        for i in 0..DMAXX {
            if g.dungeon[i][j] == floor_id && g.dTransVal[xx][yy] == 0 {
                find_transparency_values(g, Point::new(xx as i32, yy as i32), floor_id);
                g.TransVal = g.TransVal.wrapping_add(1);
            }
            xx += 2;
        }
        yy += 2;
    }
}

/// Original: `devilution::InDungeonBounds` (levels/gendung.h).
// @port levels/gendung.h|devilution::InDungeonBounds(Point position) sha=9be2294a5ba1
pub const fn in_dungeon_bounds(position: Point) -> bool {
    position.x >= 0 && position.x < MAXDUNX as i32 && position.y >= 0 && position.y < MAXDUNY as i32
}

fn dflag_has(ctx: &Ctx, position: Point, f: DungeonFlag) -> bool {
    in_dungeon_bounds(position) && ctx.gendung.dFlags[position.x as usize][position.y as usize].has_any_of(f)
}

/// Original: `devilution::TileContainsMissile` (levels/gendung.h).
// @port levels/gendung.h|devilution::TileContainsMissile(Point position) sha=9eb4609f76a9
pub fn tile_contains_missile(ctx: &Ctx, position: Point) -> bool {
    dflag_has(ctx, position, DungeonFlag::Missile)
}

/// Original: `devilution::TileContainsDeadPlayer` (levels/gendung.h).
// @port levels/gendung.h|devilution::TileContainsDeadPlayer(Point position) sha=171dd2f14f88
pub fn tile_contains_dead_player(ctx: &Ctx, position: Point) -> bool {
    dflag_has(ctx, position, DungeonFlag::DeadPlayer)
}

/// Original: `devilution::TileContainsSetPiece` (levels/gendung.h).
// @port levels/gendung.h|devilution::TileContainsSetPiece(Point position) sha=97be2be21c0a
pub fn tile_contains_set_piece(ctx: &Ctx, position: Point) -> bool {
    dflag_has(ctx, position, DungeonFlag::Populated)
}

/// Original: `devilution::IsTileVisible` (levels/gendung.h).
// @port levels/gendung.h|devilution::IsTileVisible(Point position) sha=263b2d02a1c0
pub fn is_tile_visible(ctx: &Ctx, position: Point) -> bool {
    dflag_has(ctx, position, DungeonFlag::Visible)
}

/// Original: `devilution::IsTileLit` (levels/gendung.h).
// @port levels/gendung.h|devilution::IsTileLit(Point position) sha=fd368f94b97f
pub fn is_tile_lit(ctx: &Ctx, position: Point) -> bool {
    dflag_has(ctx, position, DungeonFlag::Lit)
}

/// `dungeon[x][y]` as the generators' unchecked C++ reads see it: the flat index `x * DMAXY + y`
/// into `dungeon`, continuing into `pdungeon` (declared right after it in gendung.cpp; inferred
/// layout) and reading 0 before the start.
pub fn dungeon_flat(ctx: &Ctx, x: i32, y: i32) -> u8 {
    let (w, h) = (DMAXX as i32, DMAXY as i32);
    let idx = x * h + y;
    if (0..w * h).contains(&idx) {
        ctx.gendung.dungeon[(idx / h) as usize][(idx % h) as usize]
    } else if (w * h..2 * w * h).contains(&idx) {
        let i = idx - w * h;
        ctx.gendung.pdungeon[(i / h) as usize][(i % h) as usize]
    } else {
        0
    }
}

/// `dungeon[x][y] = v` for the generators' unchecked C++ writes (see `dungeon_flat`); writes
/// outside both arrays are dropped.
pub fn set_dungeon_flat(ctx: &mut Ctx, x: i32, y: i32, v: u8) {
    let (w, h) = (DMAXX as i32, DMAXY as i32);
    let idx = x * h + y;
    if (0..w * h).contains(&idx) {
        ctx.gendung.dungeon[(idx / h) as usize][(idx % h) as usize] = v;
    } else if (w * h..2 * w * h).contains(&idx) {
        let i = idx - w * h;
        ctx.gendung.pdungeon[(i / h) as usize][(i % h) as usize] = v;
    }
}
