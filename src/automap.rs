//! `Source/automap.cpp`: the in-game map overlay.

use crate::control::{can_panels_cover_view, get_main_panel, is_left_panel_open, is_right_panel_open};
use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::engine::render::automap_render::*;
use crate::engine::render::dun_render::TILE_HEIGHT;
use crate::engine::render::text_render::{draw_string_at, UiFlags};
use crate::engine::surface::Surface;
use crate::enums::*;
use crate::levels::gendung::{DungeonType, DMAXX, DMAXY, MAXDUNX, MAXDUNY};
use crate::lighting::*;
use crate::utils::language::tr;

pub use crate::control::do_auto_map;

/// `MapColors`
const MAP_COLORS_PLAYER: u8 = 152 + 1; // PAL8_ORANGE + 1
const MAP_COLORS_BRIGHT: u8 = 144; // PAL8_YELLOW
const MAP_COLORS_DIM: u8 = 192 + 8; // PAL16_YELLOW + 8
const MAP_COLORS_ITEM: u8 = 128 + 1; // PAL8_BLUE + 1

const PAL16_BEIGE: u8 = 160;
const PAL16_GRAY: u8 = 240;

/// `AutomapTile::Types`
#[allow(dead_code)]
mod types {
    pub const NONE: u8 = 0;
    pub const DIAMOND: u8 = 1;
    pub const VERTICAL: u8 = 2;
    pub const HORIZONTAL: u8 = 3;
    pub const CROSS: u8 = 4;
    pub const FENCE_VERTICAL: u8 = 5;
    pub const FENCE_HORIZONTAL: u8 = 6;
    pub const CORNER: u8 = 7;
    pub const CAVE_HORIZONTAL_CROSS: u8 = 8;
    pub const CAVE_VERTICAL_CROSS: u8 = 9;
    pub const CAVE_HORIZONTAL: u8 = 10;
    pub const CAVE_VERTICAL: u8 = 11;
    pub const CAVE_CROSS: u8 = 12;
    pub const BRIDGE: u8 = 13;
    pub const RIVER: u8 = 14;
    pub const RIVER_CORNER_EAST: u8 = 15;
    pub const RIVER_CORNER_NORTH: u8 = 16;
    pub const RIVER_CORNER_SOUTH: u8 = 17;
    pub const RIVER_CORNER_WEST: u8 = 18;
    pub const RIVER_FORK_IN: u8 = 19;
    pub const RIVER_FORK_OUT: u8 = 20;
    pub const RIVER_LEFT_IN: u8 = 21;
    pub const RIVER_LEFT_OUT: u8 = 22;
    pub const RIVER_RIGHT_IN: u8 = 23;
    pub const RIVER_RIGHT_OUT: u8 = 24;
}

/// `AutomapTile::Flags`
mod flags {
    pub const VERTICAL_DOOR: u8 = 1 << 0;
    pub const HORIZONTAL_DOOR: u8 = 1 << 1;
    pub const VERTICAL_ARCH: u8 = 1 << 2;
    pub const HORIZONTAL_ARCH: u8 = 1 << 3;
    pub const VERTICAL_GRATE: u8 = 1 << 4;
    pub const HORIZONTAL_GRATE: u8 = 1 << 5;
    pub const VERTICAL_PASSAGE: u8 = VERTICAL_DOOR | VERTICAL_ARCH | VERTICAL_GRATE;
    pub const HORIZONTAL_PASSAGE: u8 = HORIZONTAL_DOOR | HORIZONTAL_ARCH | HORIZONTAL_GRATE;
    pub const DIRT: u8 = 1 << 6;
    pub const STAIRS: u8 = 1 << 7;
}

/// `AutomapTile`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutomapTile {
    /// The general shape of the tile
    type_: u8,
    /// Additional details about the given tile
    flags: u8,
}

impl AutomapTile {
    const fn has_flag(&self, test: u8) -> bool {
        (self.flags & test) != 0
    }
}

/// Globals of automap.cpp.
pub struct AutomapState {
    pub AutomapActive: bool,
    /// `AutomapView[DMAXX][DMAXY]`
    pub AutomapView: Box<[[u8; DMAXY]; DMAXX]>,
    pub AutoMapScale: i32,
    pub AutomapOffset: Displacement,
    /// `Automap`
    automap: Point,
    /// `AutomapTypeTiles`: maps from tile_id to automap type.
    automap_type_tiles: [AutomapTile; 256],
}

impl Default for AutomapState {
    fn default() -> Self {
        AutomapState {
            AutomapActive: false,
            AutomapView: Box::new([[0; DMAXY]; DMAXX]),
            AutoMapScale: 0,
            AutomapOffset: Default::default(),
            automap: Point::default(),
            automap_type_tiles: [AutomapTile::default(); 256],
        }
    }
}

pub fn automap_active(ctx: &Ctx) -> bool {
    ctx.automap.AutomapActive
}

pub fn set_automap_active(ctx: &mut Ctx, active: bool) {
    ctx.automap.AutomapActive = active;
}

/// Original: `devilution::AmLine` (automap.h).
// @port automap.h|devilution::AmLine(int x) sha=966cafea1c0b
fn am_line(ctx: &Ctx, x: i32) -> i32 {
    debug_assert!((4..=64).contains(&x));
    debug_assert!((x & (x - 1)) == 0);
    ctx.automap.AutoMapScale * x / 100
}

fn p(x: i32, y: i32) -> Point {
    Point::new(x, y)
}

/// Original: `DrawDiamond` (automap.cpp).
// @port automap.cpp|devilution::DrawDiamond(const Surface &out, Point center, uint8_t color) sha=3bfbad62b463
fn draw_diamond(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    let a = |x| am_line(ctx, x);
    let left = p(center.x - a(16), center.y);
    let top = p(center.x, center.y - a(8));
    let bottom = p(center.x, center.y + a(8));
    draw_map_line_ne(out, left, a(8), color);
    draw_map_line_se(out, left, a(8), color);
    draw_map_line_se(out, top, a(8), color);
    draw_map_line_ne(out, bottom, a(8), color);
}

/// Original: `DrawMapVerticalDoor` (automap.cpp).
// @port automap.cpp|devilution::DrawMapVerticalDoor(const Surface &out, Point center, uint8_t colorBright, uint8_t colorDim) sha=6fcf8e8da027
fn draw_map_vertical_door(ctx: &Ctx, out: &Surface, center: Point, color_bright: u8, color_dim: u8) {
    let a = |x| am_line(ctx, x);
    if ctx.gendung.leveltype != DungeonType::Catacombs {
        draw_map_line_ne(out, p(center.x + a(8), center.y - a(4)), a(4), color_dim);
        draw_map_line_ne(out, p(center.x - a(16), center.y + a(8)), a(4), color_dim);
        draw_diamond(ctx, out, center, color_bright);
    } else {
        draw_map_line_ne(out, p(center.x - a(8), center.y + a(4)), a(8), color_dim);
        draw_map_line_ne(out, p(center.x - a(16), center.y + a(8)), a(4), color_dim);
        draw_diamond(ctx, out, p(center.x + a(16), center.y - a(8)), color_bright);
    }
}

/// Original: `DrawMapHorizontalDoor` (automap.cpp).
// @port automap.cpp|devilution::DrawMapHorizontalDoor(const Surface &out, Point center, uint8_t colorBright, uint8_t colorDim) sha=88f99df36961
fn draw_map_horizontal_door(ctx: &Ctx, out: &Surface, center: Point, color_bright: u8, color_dim: u8) {
    let a = |x| am_line(ctx, x);
    if ctx.gendung.leveltype != DungeonType::Catacombs {
        draw_map_line_se(out, p(center.x - a(16), center.y - a(8)), a(4), color_dim);
        draw_map_line_se(out, p(center.x + a(8), center.y + a(4)), a(4), color_dim);
        draw_diamond(ctx, out, center, color_bright);
    } else {
        draw_map_line_se(out, p(center.x - a(8), center.y - a(4)), a(8), color_dim);
        draw_map_line_se(out, p(center.x + a(8), center.y + a(4)), a(4), color_dim);
        draw_diamond(ctx, out, p(center.x - a(16), center.y - a(8)), color_bright);
    }
}

/// Sets each of `points` (offsets in `AmLine` units of `center`) to `color`.
/// Each entry is `(x terms, y terms)`, where a term `(sign, x)` adds `sign * AmLine(x)`.
fn set_pixels(ctx: &Ctx, out: &Surface, center: Point, points: &[(&[(i32, i32)], &[(i32, i32)])], color: u8) {
    for (xs, ys) in points {
        let x = center.x + xs.iter().map(|&(s, v)| s * am_line(ctx, v)).sum::<i32>();
        let y = center.y + ys.iter().map(|&(s, v)| s * am_line(ctx, v)).sum::<i32>();
        out.set_pixel(x, y, color);
    }
}

// Point shorthands used by the dirt/river patterns (see DrawDirt in the original).
const Z: &[(i32, i32)] = &[];
const P8: &[(i32, i32)] = &[(1, 8)];
const M8: &[(i32, i32)] = &[(-1, 8)];
const P16: &[(i32, i32)] = &[(1, 16)];
const M16: &[(i32, i32)] = &[(-1, 16)];
const P4: &[(i32, i32)] = &[(1, 4)];
const M4: &[(i32, i32)] = &[(-1, 4)];
/// `+ AmLine(8) - AmLine(32)`
const P8M32: &[(i32, i32)] = &[(1, 8), (-1, 32)];
/// `- AmLine(8) + AmLine(32)`
const M8P32: &[(i32, i32)] = &[(-1, 8), (1, 32)];
/// `+ AmLine(16) - AmLine(4)`
const P16M4: &[(i32, i32)] = &[(1, 16), (-1, 4)];

/// Original: `DrawDirt` (automap.cpp).
// @port automap.cpp|devilution::DrawDirt(const Surface &out, Point center, uint8_t color) sha=118b4d7a2daa
fn draw_dirt(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(
        ctx,
        out,
        center,
        &[
            (P8M32, P4),
            (M16, Z),
            (M16, P8),
            (M8, M4),
            (M8, P4),
            (M8, P16M4),
            (Z, M8),
            (Z, Z),
            (Z, P8),
            (Z, P16),
            (P8, M4),
            (P8, P4),
            (P8, P16M4),
            (P16, Z),
            (P16, P8),
            (M8P32, P4),
        ],
        color,
    );
}

/// Original: `DrawBridge` (automap.cpp).
// @port automap.cpp|devilution::DrawBridge(const Surface &out, Point center, uint8_t color) sha=0d5e3b630a02
fn draw_bridge(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(Z, Z), (P8, M4), (P8, P4), (P16, Z), (P16, P8), (M8P32, P4)], color);
}

/// Original: `DrawRiverRightIn` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverRightIn(const Surface &out, Point center, uint8_t color) sha=116807ff5e9c
fn draw_river_right_in(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(M16, P8), (M8, P4), (M8, P16M4), (Z, Z), (Z, P8), (Z, P16), (P8, P4), (P8, P16M4), (P16, Z), (P16, P8), (M8P32, P4)], color);
}

/// Original: `DrawRiverCornerSouth` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverCornerSouth(const Surface &out, Point center, uint8_t color) sha=ea917e4c016e
fn draw_river_corner_south(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(Z, P16)], color);
}

/// Original: `DrawRiverCornerNorth` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverCornerNorth(const Surface &out, Point center, uint8_t color) sha=88e8b693b89c
fn draw_river_corner_north(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(M8, M4), (Z, M8), (P8, M4)], color);
}

/// Original: `DrawRiverLeftOut` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverLeftOut(const Surface &out, Point center, uint8_t color) sha=f244a67e169b
fn draw_river_left_out(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(
        ctx,
        out,
        center,
        &[(P8M32, P4), (M16, Z), (M16, P8), (M8, P4), (M8, P16M4), (Z, Z), (Z, P8), (P8, M4), (P8, P4), (P8, P16M4), (P16, Z), (P16, P8), (M8P32, P4)],
        color,
    );
}

/// Original: `DrawRiverLeftIn` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverLeftIn(const Surface &out, Point center, uint8_t color) sha=241b6e3693e7
fn draw_river_left_in(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(M16, Z), (M16, P8), (M8, M4), (M8, P4), (M8, P16M4), (Z, M8), (Z, Z), (Z, P8), (Z, P16), (P8, M4), (P8, P4), (P8, P16M4)], color);
}

/// Original: `DrawRiverCornerWest` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverCornerWest(const Surface &out, Point center, uint8_t color) sha=3987ba0289ad
fn draw_river_corner_west(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(P8M32, P4), (M16, Z)], color);
}

/// Original: `DrawRiverCornerEast` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverCornerEast(const Surface &out, Point center, uint8_t color) sha=98bda143fb52
fn draw_river_corner_east(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(P16, Z), (P16, P8), (M8P32, P4)], color);
}

/// Original: `DrawRiverRightOut` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverRightOut(const Surface &out, Point center, uint8_t color) sha=784837a473f1
fn draw_river_right_out(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(M8, P4), (M8, P16M4), (Z, Z), (Z, P8), (Z, P16), (P8, M4), (P8, P4), (P8, P16M4), (P16, Z), (P16, P8), (M8P32, P4)], color);
}

/// Original: `DrawRiver` (automap.cpp).
// @port automap.cpp|devilution::DrawRiver(const Surface &out, Point center, uint8_t color) sha=c282d75bc48e
fn draw_river(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(M16, P8), (M8, P4), (M8, P16M4), (Z, Z), (Z, P8), (Z, P16), (P8, M4), (P8, P4), (P8, P16M4), (P16, Z), (P16, P8), (M8P32, P4)], color);
}

/// Original: `DrawRiverForkIn` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverForkIn(const Surface &out, Point center, uint8_t color) sha=6b327329ea07
fn draw_river_fork_in(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(
        ctx,
        out,
        center,
        &[(M16, Z), (M16, P8), (M8, M4), (M8, P4), (M8, P16M4), (Z, M8), (Z, Z), (Z, P8), (Z, P16), (P8, M4), (P8, P4), (P8, P16M4), (P16, Z), (P16, P8), (M8P32, P4)],
        color,
    );
}

/// Original: `DrawRiverForkOut` (automap.cpp).
// @port automap.cpp|devilution::DrawRiverForkOut(const Surface &out, Point center, uint8_t color) sha=1c42628570ff
fn draw_river_fork_out(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    set_pixels(ctx, out, center, &[(P8M32, P4), (M16, Z), (M16, P8), (M8, P16M4), (Z, P16), (P8, P16M4)], color);
}

/// Original: `DrawStairs` (automap.cpp).
// @port automap.cpp|devilution::DrawStairs(const Surface &out, Point center, uint8_t color) sha=db22e00bfbc9
fn draw_stairs(ctx: &Ctx, out: &Surface, center: Point, color: u8) {
    const NUM_STAIR_STEPS: i32 = 4;
    let a = |x| am_line(ctx, x);
    let offset = Displacement::new(-a(8), a(4));
    let mut pt = p(center.x - a(8), center.y - a(8) - a(4));
    for _ in 0..NUM_STAIR_STEPS {
        draw_map_line_se(out, pt, a(16), color);
        pt = pt + offset;
    }
}

/// Original: `DrawHorizontal` (automap.cpp): left-facing obstacle.
// @port automap.cpp|devilution::DrawHorizontal(const Surface &out, Point center, AutomapTile tile, uint8_t colorBright, uint8_t colorDim) sha=27b4626a3563
fn draw_horizontal(ctx: &Ctx, out: &Surface, center: Point, tile: AutomapTile, color_bright: u8, color_dim: u8) {
    let a = |x| am_line(ctx, x);
    if !tile.has_flag(flags::HORIZONTAL_PASSAGE) {
        draw_map_line_se(out, p(center.x, center.y - a(16)), a(16), color_dim);
        return;
    }
    if tile.has_flag(flags::HORIZONTAL_DOOR) {
        draw_map_horizontal_door(ctx, out, p(center.x + a(16), center.y - a(8)), color_bright, color_dim);
    }
    if tile.has_flag(flags::HORIZONTAL_GRATE) {
        draw_map_line_se(out, p(center.x + a(16), center.y - a(8)), a(8), color_dim);
        draw_diamond(ctx, out, p(center.x, center.y - a(8)), color_dim);
    } else if tile.has_flag(flags::HORIZONTAL_ARCH) {
        draw_diamond(ctx, out, p(center.x, center.y - a(8)), color_dim);
    }
}

/// Original: `DrawVertical` (automap.cpp): right-facing obstacle.
// @port automap.cpp|devilution::DrawVertical(const Surface &out, Point center, AutomapTile tile, uint8_t colorBright, uint8_t colorDim) sha=a838cd99024c
fn draw_vertical(ctx: &Ctx, out: &Surface, center: Point, tile: AutomapTile, color_bright: u8, color_dim: u8) {
    let a = |x| am_line(ctx, x);
    if !tile.has_flag(flags::VERTICAL_PASSAGE) {
        draw_map_line_ne(out, p(center.x - a(32), center.y), a(16), color_dim);
        return;
    }
    if tile.has_flag(flags::VERTICAL_DOOR) {
        // two wall segments with a door in the middle
        draw_map_vertical_door(ctx, out, p(center.x - a(16), center.y - a(8)), color_bright, color_dim);
    }
    if tile.has_flag(flags::VERTICAL_GRATE) {
        // right-facing half-wall
        draw_map_line_ne(out, p(center.x - a(32), center.y), a(8), color_dim);
        draw_diamond(ctx, out, p(center.x, center.y - a(8)), color_dim);
    } else if tile.has_flag(flags::VERTICAL_ARCH) {
        // window or passable column
        draw_diamond(ctx, out, p(center.x, center.y - a(8)), color_dim);
    }
}

/// Original: `DrawCaveHorizontal` (automap.cpp): for caves the horizontal/vertical flags are swapped.
// @port automap.cpp|devilution::DrawCaveHorizontal(const Surface &out, Point center, AutomapTile tile, uint8_t colorBright, uint8_t colorDim) sha=809f3d7bd7c1
fn draw_cave_horizontal(ctx: &Ctx, out: &Surface, center: Point, tile: AutomapTile, color_bright: u8, color_dim: u8) {
    let a = |x| am_line(ctx, x);
    if tile.has_flag(flags::VERTICAL_DOOR) {
        draw_map_horizontal_door(ctx, out, p(center.x - a(16), center.y + a(8)), color_bright, color_dim);
    } else {
        draw_map_line_se(out, p(center.x - a(32), center.y), a(16), color_dim);
    }
}

/// Original: `DrawCaveVertical` (automap.cpp): for caves the horizontal/vertical flags are swapped.
// @port automap.cpp|devilution::DrawCaveVertical(const Surface &out, Point center, AutomapTile tile, uint8_t colorBright, uint8_t colorDim) sha=f8a80230f2d6
fn draw_cave_vertical(ctx: &Ctx, out: &Surface, center: Point, tile: AutomapTile, color_bright: u8, color_dim: u8) {
    let a = |x| am_line(ctx, x);
    if tile.has_flag(flags::HORIZONTAL_DOOR) {
        draw_map_vertical_door(ctx, out, p(center.x + a(16), center.y + a(8)), color_bright, color_dim);
    } else {
        draw_map_line_ne(out, p(center.x, center.y + a(16)), a(16), color_dim);
    }
}

/// Original: `HasAutomapFlag` (automap.cpp). The original checks y against DMAXX too.
// @port automap.cpp|devilution::HasAutomapFlag(Point position, AutomapTile::Flags type) sha=354d7a17bd1b
fn has_automap_flag(ctx: &Ctx, position: Point, type_: u8) -> bool {
    if position.x < 0 || position.x >= DMAXX as i32 || position.y < 0 || position.y >= DMAXX as i32 {
        return false;
    }
    ctx.automap.automap_type_tiles[ctx.gendung.dungeon[position.x as usize][position.y as usize] as usize].has_flag(type_)
}

/// Original: `GetAutomapType` (automap.cpp).
// @port automap.cpp|devilution::GetAutomapType(Point position) sha=f74c9c5ab9d6
fn get_automap_type(ctx: &Ctx, position: Point) -> AutomapTile {
    if position.x < 0 || position.x >= DMAXX as i32 || position.y < 0 || position.y >= DMAXX as i32 {
        return AutomapTile::default();
    }
    let mut tile = ctx.automap.automap_type_tiles[ctx.gendung.dungeon[position.x as usize][position.y as usize] as usize];
    if tile.type_ == types::CORNER
        && has_automap_flag(ctx, p(position.x - 1, position.y), flags::HORIZONTAL_ARCH)
        && has_automap_flag(ctx, p(position.x, position.y - 1), flags::VERTICAL_ARCH)
    {
        tile.type_ = types::DIAMOND;
    }
    tile
}

/// Original: `GetAutomapTypeView` (automap.cpp).
// @port automap.cpp|devilution::GetAutomapTypeView(Point map) sha=11f3704a14bb
fn get_automap_type_view(ctx: &Ctx, map: Point) -> AutomapTile {
    let view = &ctx.automap.AutomapView;
    let dirt = AutomapTile { type_: types::NONE, flags: flags::DIRT };
    if map.x == -1 && map.y >= 0 && map.y < DMAXY as i32 && view[0][map.y as usize] != MAP_EXP_NONE {
        if has_automap_flag(ctx, p(0, map.y + 1), flags::DIRT) && has_automap_flag(ctx, p(0, map.y), flags::DIRT) && has_automap_flag(ctx, p(0, map.y - 1), flags::DIRT) {
            return AutomapTile::default();
        }
        return dirt;
    }
    if map.y == -1 && map.x >= 0 && map.x < DMAXY as i32 && view[map.x as usize][0] != MAP_EXP_NONE {
        if has_automap_flag(ctx, p(map.x + 1, 0), flags::DIRT) && has_automap_flag(ctx, p(map.x, 0), flags::DIRT) && has_automap_flag(ctx, p(map.x - 1, 0), flags::DIRT) {
            return AutomapTile::default();
        }
        return dirt;
    }
    if map.x < 0 || map.x >= DMAXX as i32 {
        return AutomapTile::default();
    }
    if map.y < 0 || map.y >= DMAXX as i32 {
        return AutomapTile::default();
    }
    if view[map.x as usize][map.y as usize] == MAP_EXP_NONE {
        return AutomapTile::default();
    }
    get_automap_type(ctx, map)
}

/// Original: `DrawAutomapTile` (automap.cpp): renders the given automap shape at the specified screen coordinates.
// @port automap.cpp|devilution::DrawAutomapTile(const Surface &out, Point center, Point map) sha=b868eb598b79
fn draw_automap_tile(ctx: &Ctx, out: &Surface, center: Point, map: Point) {
    let tile = get_automap_type_view(ctx, map);
    let mut color_bright = MAP_COLORS_BRIGHT;
    let mut color_dim = MAP_COLORS_DIM;
    let exploration_type = ctx.automap.AutomapView[map.x.clamp(0, DMAXX as i32 - 1) as usize][map.y.clamp(0, DMAXY as i32 - 1) as usize];
    match exploration_type {
        MAP_EXP_SHRINE => {
            color_dim = PAL16_GRAY + 11;
            color_bright = PAL16_GRAY + 3;
        }
        MAP_EXP_OTHERS => {
            color_dim = PAL16_BEIGE + 10;
            color_bright = PAL16_BEIGE + 2;
        }
        _ => {}
    }
    if tile.has_flag(flags::DIRT) {
        draw_dirt(ctx, out, center, color_dim);
    }
    if tile.has_flag(flags::STAIRS) {
        draw_stairs(ctx, out, center, color_bright);
    }
    let a = |x| am_line(ctx, x);
    match tile.type_ {
        types::DIAMOND => draw_diamond(ctx, out, p(center.x, center.y - a(8)), color_dim), // stand-alone column or other unpassable object
        types::VERTICAL | types::FENCE_VERTICAL => draw_vertical(ctx, out, center, tile, color_bright, color_dim),
        types::HORIZONTAL | types::FENCE_HORIZONTAL => draw_horizontal(ctx, out, center, tile, color_bright, color_dim),
        types::CROSS => {
            draw_vertical(ctx, out, center, tile, color_bright, color_dim);
            draw_horizontal(ctx, out, center, tile, color_bright, color_dim);
        }
        types::CAVE_HORIZONTAL_CROSS => {
            draw_vertical(ctx, out, center, tile, color_bright, color_dim);
            draw_cave_horizontal(ctx, out, center, tile, color_bright, color_dim);
        }
        types::CAVE_VERTICAL_CROSS => {
            draw_horizontal(ctx, out, center, tile, color_bright, color_dim);
            draw_cave_vertical(ctx, out, center, tile, color_bright, color_dim);
        }
        types::CAVE_HORIZONTAL => draw_cave_horizontal(ctx, out, center, tile, color_bright, color_dim),
        types::CAVE_VERTICAL => draw_cave_vertical(ctx, out, center, tile, color_bright, color_dim),
        types::CAVE_CROSS => {
            draw_cave_horizontal(ctx, out, center, tile, color_bright, color_dim);
            draw_cave_vertical(ctx, out, center, tile, color_bright, color_dim);
        }
        types::BRIDGE => draw_bridge(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER => draw_river(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_CORNER_EAST => draw_river_corner_east(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_CORNER_NORTH => draw_river_corner_north(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_CORNER_SOUTH => draw_river_corner_south(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_CORNER_WEST => draw_river_corner_west(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_FORK_IN => draw_river_fork_in(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_FORK_OUT => draw_river_fork_out(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_LEFT_IN => draw_river_left_in(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_LEFT_OUT => draw_river_left_out(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_RIGHT_IN => draw_river_right_in(ctx, out, center, MAP_COLORS_ITEM),
        types::RIVER_RIGHT_OUT => draw_river_right_out(ctx, out, center, MAP_COLORS_ITEM),
        _ => {} // Corner, None
    }
}

/// Original: `SearchAutomapItem` (automap.cpp).
// @port automap.cpp|devilution::SearchAutomapItem(const Surface &out, const Displacement &myPlayerOffset, int searchRadius, tl::function_ref<bool(Point position)> highlightTile) sha=4483781f98a5
fn search_automap_item(ctx: &Ctx, out: &Surface, my_player_offset: Displacement, search_radius: i32, highlight_tile: impl Fn(&Ctx, Point) -> bool) {
    let player = &ctx.players.Players[ctx.players.MyPlayer.expect("MyPlayer")];
    let mut tile = player.position.tile;
    if player._pmode == PM_WALK_SIDEWAYS {
        tile = player.position.future;
        if player._pdir == Direction::West {
            tile.x += 1;
        } else {
            tile.y += 1;
        }
    }
    let start_x = (tile.x - search_radius).clamp(0, MAXDUNX as i32);
    let start_y = (tile.y - search_radius).clamp(0, MAXDUNY as i32);
    let end_x = (tile.x + search_radius).clamp(0, MAXDUNX as i32);
    let end_y = (tile.y + search_radius).clamp(0, MAXDUNY as i32);
    let scale = ctx.automap.AutoMapScale;
    let off = ctx.automap.AutomapOffset;
    let view = ctx.gendung.ViewPosition;
    let (sw, sh) = (ctx.dx.gn_screen_width, ctx.dx.gn_screen_height);
    for i in start_x..end_x {
        for j in start_y..end_y {
            if !highlight_tile(ctx, p(i, j)) {
                continue;
            }
            let px = i - 2 * off.delta_x - view.x;
            let py = j - 2 * off.delta_y - view.y;
            let mut screen = p(
                (my_player_offset.delta_x * scale / 100 / 2) + (px - py) * am_line(ctx, 16) + sw / 2,
                (my_player_offset.delta_y * scale / 100 / 2) + (px + py) * am_line(ctx, 8) + (sh - get_main_panel(ctx).h) / 2,
            );
            if can_panels_cover_view(ctx) {
                if is_right_panel_open(ctx) {
                    screen.x -= 160;
                }
                if is_left_panel_open(ctx) {
                    screen.x += 160;
                }
            }
            screen.y -= am_line(ctx, 8);
            draw_diamond(ctx, out, screen, MAP_COLORS_ITEM);
        }
    }
}

/// Original: `DrawAutomapPlr` (automap.cpp): renders an arrow on the automap, centered on and
/// facing the direction of the player.
// @port automap.cpp|devilution::DrawAutomapPlr(const Surface &out, const Displacement &myPlayerOffset, int playerId) sha=2966ba6a2618
fn draw_automap_plr(ctx: &Ctx, out: &Surface, my_player_offset: Displacement, player_id: usize) {
    let player_color = (MAP_COLORS_PLAYER as i32 + (8 * player_id as i32) % 128) as u8;
    let player = &ctx.players.Players[player_id];
    let mut tile = player.position.tile;
    if player._pmode == PM_WALK_SIDEWAYS {
        tile = player.position.future;
    }
    let off = ctx.automap.AutomapOffset;
    let view = ctx.gendung.ViewPosition;
    let px = tile.x - 2 * off.delta_x - view.x;
    let py = tile.y - 2 * off.delta_y - view.y;
    let mut player_offset = Displacement::default();
    if player.is_walking() {
        player_offset = crate::engine::render::scrollrt::get_offset_for_walking(ctx, &player.AnimInfo, player._pdir, false);
    }
    let scale = ctx.automap.AutoMapScale;
    let (sw, sh) = (ctx.dx.gn_screen_width, ctx.dx.gn_screen_height);
    let a = |x| am_line(ctx, x);
    let mut base = p(
        ((player_offset.delta_x + my_player_offset.delta_x) * scale / 100 / 2) + (px - py) * a(16) + sw / 2,
        ((player_offset.delta_y + my_player_offset.delta_y) * scale / 100 / 2) + (px + py) * a(8) + (sh - get_main_panel(ctx).h) / 2,
    );
    if can_panels_cover_view(ctx) {
        if is_right_panel_open(ctx) {
            base.x -= sw / 4;
        }
        if is_left_panel_open(ctx) {
            base.x += sw / 4;
        }
    }
    base.y -= a(16);
    let hline = crate::engine::draw_horizontal_line;
    let vline = crate::engine::draw_vertical_line;
    match player._pdir {
        Direction::North => {
            let point = p(base.x, base.y - a(16));
            vline(out, point, a(16), player_color);
            draw_map_line_steep_ne(out, p(point.x - a(4), point.y + 2 * a(4)), a(4), player_color);
            draw_map_line_steep_nw(out, p(point.x + a(4), point.y + 2 * a(4)), a(4), player_color);
        }
        Direction::NorthEast => {
            let point = p(base.x + a(16), base.y - a(8));
            hline(out, p(point.x - a(8), point.y), a(8), player_color);
            draw_map_line_ne(out, p(point.x - 2 * a(8), point.y + a(8)), a(8), player_color);
            draw_map_line_steep_sw(out, point, a(4), player_color);
        }
        Direction::East => {
            let point = p(base.x + a(16), base.y);
            draw_map_line_nw(out, point, a(4), player_color);
            hline(out, p(point.x - a(16), point.y), a(16), player_color);
            draw_map_line_sw(out, point, a(4), player_color);
        }
        Direction::SouthEast => {
            let point = p(base.x + a(16), base.y + a(8));
            draw_map_line_steep_nw(out, point, a(4), player_color);
            draw_map_line_se(out, p(point.x - 2 * a(8), point.y - a(8)), a(8), player_color);
            hline(out, p(point.x - (a(8) + 1), point.y), a(8) + 1, player_color);
        }
        Direction::South => {
            let point = p(base.x, base.y + a(16));
            vline(out, p(point.x, point.y - a(16)), a(16), player_color);
            draw_map_line_steep_sw(out, p(point.x + a(4), point.y - 2 * a(4)), a(4), player_color);
            draw_map_line_steep_se(out, p(point.x - a(4), point.y - 2 * a(4)), a(4), player_color);
        }
        Direction::SouthWest => {
            let point = p(base.x - a(16), base.y + a(8));
            draw_map_line_steep_ne(out, point, a(4), player_color);
            draw_map_line_sw(out, p(point.x + 2 * a(8), point.y - a(8)), a(8), player_color);
            hline(out, point, a(8) + 1, player_color);
        }
        Direction::West => {
            let point = p(base.x - a(16), base.y);
            draw_map_line_ne(out, point, a(4), player_color);
            hline(out, point, a(16) + 1, player_color);
            draw_map_line_se(out, point, a(4), player_color);
        }
        Direction::NorthWest => {
            let point = p(base.x - a(16), base.y - a(8));
            draw_map_line_nw(out, p(point.x + 2 * a(8), point.y + a(8)), a(8), player_color);
            hline(out, point, a(8) + 1, player_color);
            draw_map_line_steep_se(out, point, a(4), player_color);
        }
        Direction::NoDirection => {}
    }
}

/// Original: `DrawAutomapText` (automap.cpp): renders game info, such as the name of the
/// current level, and in multi player the name of the game and the game password.
// @port automap.cpp|devilution::DrawAutomapText(const Surface &out) sha=c3c6541bb683
fn draw_automap_text(ctx: &mut Ctx, out: &Surface) {
    let mut line_position = (8, 8);
    let mut draw = |ctx: &mut Ctx, s: &str, pos: (i32, i32)| {
        draw_string_at(ctx, out, s, pos, UiFlags::NONE, 1, -1);
    };
    if ctx.init.gb_is_multiplayer {
        if ctx.multi.GameName != "0.0.0.0" && !ctx.multi.IsLoopback {
            let description = format!("{}{}", tr("Game: "), ctx.multi.GameName);
            draw(ctx, &description, line_position);
            line_position.1 += 15;
        }
        let description = if ctx.multi.IsLoopback {
            tr("Offline Game")
        } else if !ctx.multi.PublicGame {
            format!("{}{}", tr("Password: "), ctx.multi.GamePassword)
        } else {
            tr("Public Game")
        };
        draw(ctx, &description, line_position);
        line_position.1 += 15;
    }
    if ctx.gendung.setlevel {
        let name = tr(crate::levels::setmaps::QUEST_LEVEL_NAMES[ctx.gendung.setlvlnum as usize]);
        draw(ctx, &name, line_position);
        return;
    }
    let currlevel = ctx.gendung.currlevel as i32;
    let fmt1 = |f: String, v: String| f.replacen("{:d}", &v, 1).replacen("{:s}", &v, 1);
    let description = match ctx.gendung.leveltype {
        DungeonType::Nest => fmt1(tr("Level: Nest {:d}"), (currlevel - 16).to_string()),
        DungeonType::Crypt => fmt1(tr("Level: Crypt {:d}"), (currlevel - 20).to_string()),
        DungeonType::Town => tr("Town"),
        _ => fmt1(tr("Level: {:d}"), currlevel.to_string()),
    };
    draw(ctx, &description, line_position);
    line_position.1 += 15;
    let difficulty = match ctx.multi.sgGameInitInfo.nDifficulty {
        DIFF_NORMAL => tr("Normal"),
        DIFF_NIGHTMARE => tr("Nightmare"),
        DIFF_HELL => tr("Hell"),
        _ => String::new(),
    };
    let difficulty_string = fmt1(tr("Difficulty: {:s}"), difficulty);
    draw(ctx, &difficulty_string, line_position);
}

/// Original: `LoadAutomapData` (automap.cpp).
// @port automap.cpp|devilution::LoadAutomapData(size_t &tileCount) sha=2de2e6a06cd2
fn load_automap_data(ctx: &mut Ctx) -> Vec<AutomapTile> {
    let path = match ctx.gendung.leveltype {
        DungeonType::Town => "levels\\towndata\\automap.amp",
        DungeonType::Cathedral => "levels\\l1data\\l1.amp",
        DungeonType::Catacombs => "levels\\l2data\\l2.amp",
        DungeonType::Caves => "levels\\l3data\\l3.amp",
        DungeonType::Hell => "levels\\l4data\\l4.amp",
        DungeonType::Nest => "nlevels\\l6data\\l6.amp",
        DungeonType::Crypt => "nlevels\\l5data\\l5.amp",
        _ => return Vec::new(),
    };
    let data = crate::engine::load_file::load_file_in_mem(ctx, path).unwrap_or_default();
    data.chunks_exact(2).map(|c| AutomapTile { type_: c[0], flags: c[1] }).collect()
}

/// Original: `devilution::InitAutomapOnce` (automap.cpp).
// @port automap.cpp|devilution::InitAutomapOnce() sha=18cf0a1366a5
pub fn init_automap_once(ctx: &mut Ctx) {
    ctx.automap.AutomapActive = false;
    ctx.automap.AutoMapScale = 50;
}

/// Original: `devilution::InitAutomap` (automap.cpp).
// @port automap.cpp|devilution::InitAutomap() sha=f34fb45dbd81
pub fn init_automap(ctx: &mut Ctx) {
    let tile_types = load_automap_data(ctx);
    for (i, t) in tile_types.iter().enumerate() {
        ctx.automap.automap_type_tiles[i + 1] = *t;
    }
    *ctx.automap.AutomapView = [[0; DMAXY]; DMAXX];
    for column in ctx.gendung.dFlags.iter_mut() {
        for d_flag in column.iter_mut() {
            d_flag.0 &= !DungeonFlag::Explored.0;
        }
    }
}

/// Original: `devilution::StartAutomap` (automap.cpp).
// @port automap.cpp|devilution::StartAutomap() sha=5d1f6e4fd7e6
pub fn start_automap(ctx: &mut Ctx) {
    ctx.automap.AutomapOffset = Displacement::new(0, 0);
    ctx.automap.AutomapActive = true;
}

/// Original: `devilution::AutomapUp` (automap.cpp).
// @port automap.cpp|devilution::AutomapUp() sha=f1a1bcea5ef6
pub fn automap_up(ctx: &mut Ctx) {
    ctx.automap.AutomapOffset.delta_x -= 1;
    ctx.automap.AutomapOffset.delta_y -= 1;
}

/// Original: `devilution::AutomapDown` (automap.cpp).
// @port automap.cpp|devilution::AutomapDown() sha=964562e3a7af
pub fn automap_down(ctx: &mut Ctx) {
    ctx.automap.AutomapOffset.delta_x += 1;
    ctx.automap.AutomapOffset.delta_y += 1;
}

/// Original: `devilution::AutomapLeft` (automap.cpp).
// @port automap.cpp|devilution::AutomapLeft() sha=ffd59b3b33fe
pub fn automap_left(ctx: &mut Ctx) {
    ctx.automap.AutomapOffset.delta_x -= 1;
    ctx.automap.AutomapOffset.delta_y += 1;
}

/// Original: `devilution::AutomapRight` (automap.cpp).
// @port automap.cpp|devilution::AutomapRight() sha=ddee013f9496
pub fn automap_right(ctx: &mut Ctx) {
    ctx.automap.AutomapOffset.delta_x += 1;
    ctx.automap.AutomapOffset.delta_y -= 1;
}

/// Original: `devilution::AutomapZoomIn` (automap.cpp).
// @port automap.cpp|devilution::AutomapZoomIn() sha=cdec5fd8248c
pub fn automap_zoom_in(ctx: &mut Ctx) {
    if ctx.automap.AutoMapScale >= 200 {
        return;
    }
    ctx.automap.AutoMapScale += 5;
}

/// Original: `devilution::AutomapZoomOut` (automap.cpp).
// @port automap.cpp|devilution::AutomapZoomOut() sha=7bb7236b0af5
pub fn automap_zoom_out(ctx: &mut Ctx) {
    if ctx.automap.AutoMapScale <= 50 {
        return;
    }
    ctx.automap.AutoMapScale -= 5;
}

/// Original: `devilution::DrawAutomap` (automap.cpp), release build (no debug highlight).
// @port automap.cpp|devilution::DrawAutomap(const Surface &out) sha=2d79eaa2380a
pub fn draw_automap(ctx: &mut Ctx, out: &Surface) {
    let view = ctx.gendung.ViewPosition;
    let mut automap = p((view.x - 8) / 2, (view.y - 8) / 2);
    if ctx.gendung.leveltype != DungeonType::Town {
        automap = automap + Displacement::new(-4, -4);
    }
    {
        let off = &mut ctx.automap.AutomapOffset;
        while automap.x + off.delta_x < 0 {
            off.delta_x += 1;
        }
        while automap.x + off.delta_x >= DMAXX as i32 {
            off.delta_x -= 1;
        }
        while automap.y + off.delta_y < 0 {
            off.delta_y += 1;
        }
        while automap.y + off.delta_y >= DMAXY as i32 {
            off.delta_y -= 1;
        }
        automap = automap + *off;
    }
    ctx.automap.automap = automap;

    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let mut my_player_offset = Displacement::default();
    if ctx.players.Players[me].is_walking() {
        let pl = &ctx.players.Players[me];
        my_player_offset = crate::engine::render::scrollrt::get_offset_for_walking(ctx, &pl.AnimInfo, pl._pdir, true);
    }
    let caves = ctx.gendung.leveltype == DungeonType::Caves;
    my_player_offset = my_player_offset + Displacement::new(-1, if !caves { TILE_HEIGHT - 1 } else { -1 });

    let scale = ctx.automap.AutoMapScale;
    let (sw, sh) = (ctx.dx.gn_screen_width, ctx.dx.gn_screen_height);
    let d = (scale * 64) / 100;
    let mut cells = 2 * (sw / 2 / d) + 1;
    if ((sw / 2) % d) != 0 {
        cells += 1;
    }
    if ((sw / 2) % d) >= (scale * 32) / 100 {
        cells += 1;
    }
    if (my_player_offset.delta_x + my_player_offset.delta_y) != 0 {
        cells += 1;
    }
    let a = |ctx: &Ctx, x| am_line(ctx, x);
    let mut screen = p(sw / 2, (sh - get_main_panel(ctx).h) / 2);
    if (cells & 1) != 0 {
        screen.x -= a(ctx, 64) * ((cells - 1) / 2);
        screen.y -= a(ctx, 32) * ((cells + 1) / 2);
    } else {
        screen.x -= a(ctx, 64) * (cells / 2) - a(ctx, 32);
        screen.y -= a(ctx, 32) * (cells / 2) + a(ctx, 16);
    }
    if (view.x & 1) != 0 {
        screen.x -= a(ctx, 16);
        screen.y -= a(ctx, 8);
    }
    if (view.y & 1) != 0 {
        screen.x += a(ctx, 16);
        screen.y -= a(ctx, 8);
    }
    screen.x += scale * my_player_offset.delta_x / 100 / 2;
    screen.y += scale * my_player_offset.delta_y / 100 / 2;
    if can_panels_cover_view(ctx) {
        if is_right_panel_open(ctx) {
            screen.x -= sw / 4;
        }
        if is_left_panel_open(ctx) {
            screen.x += sw / 4;
        }
    }

    let mut map = p(automap.x - cells, automap.y - 1);
    for _ in 0..=cells + 1 {
        let mut tile1 = screen;
        for j in 0..cells {
            draw_automap_tile(ctx, out, tile1, p(map.x + j, map.y - j));
            tile1.x += a(ctx, 64);
        }
        map.y += 1;
        let mut tile2 = p(screen.x - a(ctx, 32), screen.y + a(ctx, 16));
        for j in 0..=cells {
            draw_automap_tile(ctx, out, tile2, p(map.x + j, map.y - j));
            tile2.x += a(ctx, 64);
        }
        map.x += 1;
        screen.y += a(ctx, 32);
    }

    if caves {
        my_player_offset.delta_y += TILE_HEIGHT;
    }
    for player_id in 0..ctx.players.Players.len() {
        let player = &ctx.players.Players[player_id];
        if crate::player::is_on_active_level(ctx, player_id) && player.plractive && !player._pLvlChanging && (Some(player_id) == ctx.players.MyPlayer || player.friendlyMode) {
            draw_automap_plr(ctx, out, my_player_offset, player_id);
        }
    }
    my_player_offset.delta_y -= TILE_HEIGHT / 2;
    if ctx.scrollrt.AutoMapShowItems {
        search_automap_item(ctx, out, my_player_offset, 8, |ctx, position| ctx.items.dItem[position.x as usize][position.y as usize] != 0);
    }
    draw_automap_text(ctx, out);
}

/// Original: `devilution::UpdateAutomapExplorer` (automap.cpp).
// @port automap.cpp|devilution::UpdateAutomapExplorer(Point map, MapExplorationType explorer) sha=e8dc0d7f163a
pub fn update_automap_explorer(ctx: &mut Ctx, map: Point, explorer: MapExplorationType) {
    let v = &mut ctx.automap.AutomapView[map.x as usize][map.y as usize];
    if *v < explorer {
        *v = explorer;
    }
}

/// `tile.type == Corner && tile.HasFlag(Dirt)` for the tile at `pt`.
fn is_dirt_corner(ctx: &Ctx, pt: Point) -> bool {
    let t = get_automap_type(ctx, pt);
    t.type_ == types::CORNER && t.has_flag(flags::DIRT)
}

/// Original: `devilution::SetAutomapView` (automap.cpp).
// @port automap.cpp|devilution::SetAutomapView(Point position, MapExplorationType explorer) sha=2ad1a5c1661f
pub fn set_automap_view(ctx: &mut Ctx, position: Point, explorer: MapExplorationType) {
    let map = p((position.x - 16) / 2, (position.y - 16) / 2);
    if map.x < 0 || map.x >= DMAXX as i32 || map.y < 0 || map.y >= DMAXY as i32 {
        return;
    }
    update_automap_explorer(ctx, map, explorer);
    let tile = get_automap_type(ctx, map);
    let solid = tile.has_flag(flags::DIRT);
    let (x, y) = (map.x, map.y);
    let mut updates: Vec<Point> = Vec::new();
    match tile.type_ {
        types::VERTICAL => {
            if solid {
                if is_dirt_corner(ctx, p(x, y + 1)) {
                    updates.push(p(x, y + 1));
                }
            } else if has_automap_flag(ctx, p(x - 1, y), flags::DIRT) {
                updates.push(p(x - 1, y));
            }
        }
        types::HORIZONTAL => {
            if solid {
                if is_dirt_corner(ctx, p(x + 1, y)) {
                    updates.push(p(x + 1, y));
                }
            } else if has_automap_flag(ctx, p(x, y - 1), flags::DIRT) {
                updates.push(p(x, y - 1));
            }
        }
        types::CROSS => {
            if solid {
                if is_dirt_corner(ctx, p(x, y + 1)) {
                    updates.push(p(x, y + 1));
                }
                if is_dirt_corner(ctx, p(x + 1, y)) {
                    updates.push(p(x + 1, y));
                }
            } else {
                if has_automap_flag(ctx, p(x - 1, y), flags::DIRT) {
                    updates.push(p(x - 1, y));
                }
                if has_automap_flag(ctx, p(x, y - 1), flags::DIRT) {
                    updates.push(p(x, y - 1));
                }
                if has_automap_flag(ctx, p(x - 1, y - 1), flags::DIRT) {
                    updates.push(p(x - 1, y - 1));
                }
            }
        }
        types::FENCE_VERTICAL => {
            if solid {
                if has_automap_flag(ctx, p(x, y - 1), flags::DIRT) {
                    updates.push(p(x, y - 1));
                }
                if is_dirt_corner(ctx, p(x, y + 1)) {
                    updates.push(p(x, y + 1));
                }
            } else if has_automap_flag(ctx, p(x - 1, y), flags::DIRT) {
                updates.push(p(x - 1, y));
            }
        }
        types::FENCE_HORIZONTAL => {
            if solid {
                if has_automap_flag(ctx, p(x - 1, y), flags::DIRT) {
                    updates.push(p(x - 1, y));
                }
                if is_dirt_corner(ctx, p(x + 1, y)) {
                    updates.push(p(x + 1, y));
                }
            } else if has_automap_flag(ctx, p(x, y - 1), flags::DIRT) {
                updates.push(p(x, y - 1));
            }
        }
        _ => {}
    }
    // The checks above only read the dungeon/tile types, which the updates do not change.
    for u in updates {
        update_automap_explorer(ctx, u, explorer);
    }
}

/// Original: `devilution::AutomapZoomReset` (automap.cpp).
// @port automap.cpp|devilution::AutomapZoomReset() sha=a04c76a712f5
pub fn automap_zoom_reset(ctx: &mut Ctx) {
    ctx.automap.AutomapOffset = Displacement::new(0, 0);
}
