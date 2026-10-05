//! `Source/lighting.cpp`: light sources, vision (what each player can see), the light
//! translation tables and the radius "crawl" helpers.

use crate::ctx::Ctx;
use crate::engine::geometry::{Displacement, Point, Rectangle};
use crate::enums::{DungeonFlag, TileProperties};
use crate::levels::gendung::{in_dungeon_bounds, tile_has_any, DungeonType, MAXDUNX, MAXDUNY};

pub const MAXLIGHTS: usize = 32;
pub const MAXVISION: usize = 4;
pub const NumLightingLevels: usize = 16;
pub const NO_LIGHT: i32 = -1;
pub const LightsMax: i8 = 15;
pub const MaxCrawlRadius: i32 = 18;
const NUM_LIGHT_RADIUSES: usize = 16;

/// `MapExplorationType` (automap.h)
pub type MapExplorationType = u8;
pub const MAP_EXP_NONE: MapExplorationType = 0;
pub const MAP_EXP_OLD: MapExplorationType = 1;
pub const MAP_EXP_SHRINE: MapExplorationType = 2;
pub const MAP_EXP_OTHERS: MapExplorationType = 3;
pub const MAP_EXP_SELF: MapExplorationType = 4;

/// `LightPosition`
#[derive(Clone, Copy, Debug, Default)]
pub struct LightPosition {
    pub tile: Point,
    /// `DisplacementOf<int8_t>`
    pub offset: Displacement,
    pub old: Point,
}

/// `Light`
#[derive(Clone, Copy, Debug, Default)]
pub struct Light {
    pub position: LightPosition,
    pub radius: u8,
    pub oldRadius: u8,
    pub isInvalid: bool,
    pub hasChanged: bool,
}

/// Globals of lighting.cpp.
pub struct LightingState {
    pub VisionActive: [bool; MAXVISION],
    pub VisionList: [Light; MAXVISION],
    pub Lights: [Light; MAXLIGHTS],
    pub ActiveLights: [u8; MAXLIGHTS],
    pub ActiveLightCount: i32,
    pub LightTables: Box<[[u8; 256]; NumLightingLevels]>,
    pub InfravisionTable: [u8; 256],
    pub StoneTable: [u8; 256],
    pub PauseTable: [u8; 256],
    pub UpdateLighting: bool,
    light_falloffs: Box<[[u8; 128]; NUM_LIGHT_RADIUSES]>,
    update_vision: bool,
    /// `LightConeInterpolations[8][8][16][16]`
    light_cone_interpolations: Box<[[[[u8; 16]; 16]; 8]; 8]>,
}

impl Default for LightingState {
    fn default() -> Self {
        LightingState {
            VisionActive: [false; MAXVISION],
            VisionList: [Light::default(); MAXVISION],
            Lights: [Light::default(); MAXLIGHTS],
            ActiveLights: [0; MAXLIGHTS],
            ActiveLightCount: 0,
            LightTables: Box::new([[0; 256]; NumLightingLevels]),
            InfravisionTable: [0; 256],
            StoneTable: [0; 256],
            PauseTable: [0; 256],
            UpdateLighting: false,
            light_falloffs: Box::new([[0; 128]; NUM_LIGHT_RADIUSES]),
            update_vision: false,
            light_cone_interpolations: Box::new([[[[0; 16]; 16]; 8]; 8]),
        }
    }
}

const fn d(x: i32, y: i32) -> Displacement {
    Displacement::new(x, y)
}

/// `VisionCrawlTable[23][15]`
const VISION_CRAWL_TABLE: [[Displacement; 15]; 23] = [
    [d(1, 0), d(2, 0), d(3, 0), d(4, 0), d(5, 0), d(6, 0), d(7, 0), d(8, 0), d(9, 0), d(10, 0), d(11, 0), d(12, 0), d(13, 0), d(14, 0), d(15, 0)],
    [d(1, 0), d(2, 0), d(3, 0), d(4, 0), d(5, 0), d(6, 0), d(7, 0), d(8, 1), d(9, 1), d(10, 1), d(11, 1), d(12, 1), d(13, 1), d(14, 1), d(15, 1)],
    [d(1, 0), d(2, 0), d(3, 0), d(4, 1), d(5, 1), d(6, 1), d(7, 1), d(8, 1), d(9, 1), d(10, 1), d(11, 1), d(12, 2), d(13, 2), d(14, 2), d(15, 2)],
    [d(1, 0), d(2, 0), d(3, 1), d(4, 1), d(5, 1), d(6, 1), d(7, 1), d(8, 2), d(9, 2), d(10, 2), d(11, 2), d(12, 2), d(13, 3), d(14, 3), d(15, 3)],
    [d(1, 0), d(2, 1), d(3, 1), d(4, 1), d(5, 1), d(6, 2), d(7, 2), d(8, 2), d(9, 3), d(10, 3), d(11, 3), d(12, 3), d(13, 4), d(14, 4), d(0, 0)],
    [d(1, 0), d(2, 1), d(3, 1), d(4, 1), d(5, 2), d(6, 2), d(7, 3), d(8, 3), d(9, 3), d(10, 4), d(11, 4), d(12, 4), d(13, 5), d(14, 5), d(0, 0)],
    [d(1, 0), d(2, 1), d(3, 1), d(4, 2), d(5, 2), d(6, 3), d(7, 3), d(8, 3), d(9, 4), d(10, 4), d(11, 5), d(12, 5), d(13, 6), d(14, 6), d(0, 0)],
    [d(1, 1), d(2, 1), d(3, 2), d(4, 2), d(5, 3), d(6, 3), d(7, 4), d(8, 4), d(9, 5), d(10, 5), d(11, 6), d(12, 6), d(13, 7), d(0, 0), d(0, 0)],
    [d(1, 1), d(2, 1), d(3, 2), d(4, 2), d(5, 3), d(6, 4), d(7, 4), d(8, 5), d(9, 6), d(10, 6), d(11, 7), d(12, 7), d(12, 8), d(13, 8), d(0, 0)],
    [d(1, 1), d(2, 2), d(3, 2), d(4, 3), d(5, 4), d(6, 5), d(7, 5), d(8, 6), d(9, 7), d(10, 7), d(10, 8), d(11, 8), d(12, 9), d(0, 0), d(0, 0)],
    [d(1, 1), d(2, 2), d(3, 3), d(4, 4), d(5, 5), d(6, 5), d(7, 6), d(8, 7), d(9, 8), d(10, 9), d(11, 9), d(11, 10), d(0, 0), d(0, 0), d(0, 0)],
    [d(1, 1), d(2, 2), d(3, 3), d(4, 4), d(5, 5), d(6, 6), d(7, 7), d(8, 8), d(9, 9), d(10, 10), d(11, 11), d(0, 0), d(0, 0), d(0, 0), d(0, 0)],
    [d(1, 1), d(2, 2), d(3, 3), d(4, 4), d(5, 5), d(5, 6), d(6, 7), d(7, 8), d(8, 9), d(9, 10), d(9, 11), d(10, 11), d(0, 0), d(0, 0), d(0, 0)],
    [d(1, 1), d(2, 2), d(2, 3), d(3, 4), d(4, 5), d(5, 6), d(5, 7), d(6, 8), d(7, 9), d(7, 10), d(8, 10), d(8, 11), d(9, 12), d(0, 0), d(0, 0)],
    [d(1, 1), d(1, 2), d(2, 3), d(2, 4), d(3, 5), d(4, 6), d(4, 7), d(5, 8), d(6, 9), d(6, 10), d(7, 11), d(7, 12), d(8, 12), d(8, 13), d(0, 0)],
    [d(1, 1), d(1, 2), d(2, 3), d(2, 4), d(3, 5), d(3, 6), d(4, 7), d(4, 8), d(5, 9), d(5, 10), d(6, 11), d(6, 12), d(7, 13), d(0, 0), d(0, 0)],
    [d(0, 1), d(1, 2), d(1, 3), d(2, 4), d(2, 5), d(3, 6), d(3, 7), d(3, 8), d(4, 9), d(4, 10), d(5, 11), d(5, 12), d(6, 13), d(6, 14), d(0, 0)],
    [d(0, 1), d(1, 2), d(1, 3), d(1, 4), d(2, 5), d(2, 6), d(3, 7), d(3, 8), d(3, 9), d(4, 10), d(4, 11), d(4, 12), d(5, 13), d(5, 14), d(0, 0)],
    [d(0, 1), d(1, 2), d(1, 3), d(1, 4), d(1, 5), d(2, 6), d(2, 7), d(2, 8), d(3, 9), d(3, 10), d(3, 11), d(3, 12), d(4, 13), d(4, 14), d(0, 0)],
    [d(0, 1), d(0, 2), d(1, 3), d(1, 4), d(1, 5), d(1, 6), d(1, 7), d(2, 8), d(2, 9), d(2, 10), d(2, 11), d(2, 12), d(3, 13), d(3, 14), d(3, 15)],
    [d(0, 1), d(0, 2), d(0, 3), d(1, 4), d(1, 5), d(1, 6), d(1, 7), d(1, 8), d(1, 9), d(1, 10), d(1, 11), d(2, 12), d(2, 13), d(2, 14), d(2, 15)],
    [d(0, 1), d(0, 2), d(0, 3), d(0, 4), d(0, 5), d(0, 6), d(0, 7), d(1, 8), d(1, 9), d(1, 10), d(1, 11), d(1, 12), d(1, 13), d(1, 14), d(1, 15)],
    [d(0, 1), d(0, 2), d(0, 3), d(0, 4), d(0, 5), d(0, 6), d(0, 7), d(0, 8), d(0, 9), d(0, 10), d(0, 11), d(0, 12), d(0, 13), d(0, 14), d(0, 15)],
];

/// `RadiusAdj`: from VisionCrawlTable index to lighting vision radius adjustment.
const RADIUS_ADJ: [u8; 23] = [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 4, 3, 2, 2, 2, 1, 1, 1, 0, 0, 0, 0];

/// `int8_t` displacement arithmetic for `RotateRadius`.
#[derive(Clone, Copy, Default)]
struct D8 {
    x: i8,
    y: i8,
}

/// Original: `RotateRadius` (lighting.cpp).
// @port lighting.cpp|devilution::RotateRadius(DisplacementOf<int8_t> &offset, DisplacementOf<int8_t> &dist, DisplacementOf<int8_t> &light, DisplacementOf<int8_t> &block) sha=703952a8fb90
fn rotate_radius(offset: &mut D8, dist: &mut D8, light: &mut D8, block: &mut D8) {
    *dist = D8 { x: 7i8.wrapping_sub(dist.y), y: dist.x };
    *light = D8 { x: 7i8.wrapping_sub(light.y), y: light.x };
    *offset = D8 { x: dist.x.wrapping_sub(light.x), y: dist.y.wrapping_sub(light.y) };
    block.x = 0;
    if offset.x < 0 {
        offset.x += 8;
        block.x = 1;
    }
    block.y = 0;
    if offset.y < 0 {
        offset.y += 8;
        block.y = 1;
    }
}

/// Original: `SetLight` (lighting.cpp).
// @port lighting.cpp|devilution::SetLight(Point position, uint8_t v) sha=016adbeb259e
fn set_light(ctx: &mut Ctx, position: Point, v: u8) {
    if ctx.objects.LoadingMapObjects {
        ctx.gendung.dPreLight[position.x as usize][position.y as usize] = v;
    } else {
        ctx.gendung.dLight[position.x as usize][position.y as usize] = v;
    }
}

/// Original: `GetLight` (lighting.cpp).
// @port lighting.cpp|devilution::GetLight(Point position) sha=193351cceae3
fn get_light(ctx: &Ctx, position: Point) -> u8 {
    if ctx.objects.LoadingMapObjects {
        return ctx.gendung.dPreLight[position.x as usize][position.y as usize];
    }
    ctx.gendung.dLight[position.x as usize][position.y as usize]
}

/// Original: `CrawlFlipsX` (lighting.cpp).
// @port lighting.cpp|devilution::CrawlFlipsX(Displacement mirrored, tl::function_ref<bool(Displacement)> function) sha=edd1a001de5f
fn crawl_flips_x(mirrored: Displacement, function: &mut dyn FnMut(Displacement) -> bool) -> bool {
    [mirrored.flip_x(), mirrored].into_iter().all(function)
}

/// Original: `CrawlFlipsY` (lighting.cpp).
// @port lighting.cpp|devilution::CrawlFlipsY(Displacement mirrored, tl::function_ref<bool(Displacement)> function) sha=642ff7fd83c9
fn crawl_flips_y(mirrored: Displacement, function: &mut dyn FnMut(Displacement) -> bool) -> bool {
    [mirrored, mirrored.flip_y()].into_iter().all(function)
}

/// Original: `CrawlFlipsXY` (lighting.cpp).
// @port lighting.cpp|devilution::CrawlFlipsXY(Displacement mirrored, tl::function_ref<bool(Displacement)> function) sha=20d7c047cce7
fn crawl_flips_xy(mirrored: Displacement, function: &mut dyn FnMut(Displacement) -> bool) -> bool {
    [mirrored.flip_x(), mirrored, mirrored.flip_xy(), mirrored.flip_y()].into_iter().all(function)
}

/// Original: `TileAllowsLight` (lighting.cpp).
// @port lighting.cpp|devilution::TileAllowsLight(Point position) sha=751c6bd6f541
fn tile_allows_light(ctx: &Ctx, position: Point) -> bool {
    if !in_dungeon_bounds(position) {
        return false;
    }
    !tile_has_any(ctx, ctx.gendung.dPiece[position.x as usize][position.y as usize] as i32, TileProperties::BlockLight)
}

/// Original: `DoVisionFlags` (lighting.cpp).
// @port lighting.cpp|devilution::DoVisionFlags(Point position, MapExplorationType doAutomap, bool visible) sha=7e69dfa2b27b
fn do_vision_flags(ctx: &mut Ctx, position: Point, do_automap: MapExplorationType, visible: bool) {
    let (x, y) = (position.x as usize, position.y as usize);
    if do_automap != MAP_EXP_NONE {
        if ctx.gendung.dFlags[x][y] != DungeonFlag::None_ {
            crate::automap::set_automap_view(ctx, position, do_automap);
        }
        ctx.gendung.dFlags[x][y] |= DungeonFlag::Explored;
    }
    if visible {
        ctx.gendung.dFlags[x][y] |= DungeonFlag::Lit;
    }
    ctx.gendung.dFlags[x][y] |= DungeonFlag::Visible;
}

/// Original: `devilution::DoCrawl(unsigned radius, ...)` (lighting.cpp).
// @port lighting.cpp|devilution::DoCrawl(unsigned radius, tl::function_ref<bool(Displacement)> function) sha=905f039f74e4
pub fn do_crawl(radius: u32, function: &mut dyn FnMut(Displacement) -> bool) -> bool {
    let r = radius as i32;
    if radius == 0 {
        return function(Displacement::new(0, 0));
    }
    if !crawl_flips_y(Displacement::new(0, r), function) {
        return false;
    }
    for i in 1..r {
        if !crawl_flips_xy(Displacement::new(i, r), function) {
            return false;
        }
    }
    if radius > 1 && !crawl_flips_xy(Displacement::new(r - 1, r - 1), function) {
        return false;
    }
    if !crawl_flips_x(Displacement::new(r, 0), function) {
        return false;
    }
    for i in 1..r {
        if !crawl_flips_xy(Displacement::new(r, i), function) {
            return false;
        }
    }
    true
}

/// Original: `devilution::DoCrawl(unsigned minRadius, unsigned maxRadius, ...)` (lighting.cpp).
// @port lighting.cpp|devilution::DoCrawl(unsigned minRadius, unsigned maxRadius, tl::function_ref<bool(Displacement)> function) sha=2981d89652f3
pub fn do_crawl_range(min_radius: u32, max_radius: u32, function: &mut dyn FnMut(Displacement) -> bool) -> bool {
    for i in min_radius..=max_radius {
        if !do_crawl(i, function) {
            return false;
        }
    }
    true
}

/// `Crawl(radius, F)`: the first `Some` the function returns.
// @port lighting.h|devilution::Crawl(unsigned radius, F function) sha=c1751d96e170
pub fn crawl<T>(radius: u32, function: &mut dyn FnMut(Displacement) -> Option<T>) -> Option<T> {
    let mut result = None;
    do_crawl(radius, &mut |d| {
        result = function(d);
        result.is_none()
    });
    result
}

/// `Crawl(minRadius, maxRadius, F)`: the first `Some` the function returns.
// @port lighting.h|devilution::Crawl(unsigned minRadius, unsigned maxRadius, F function) sha=6d1d7aacbdef
pub fn crawl_range<T>(min_radius: u32, max_radius: u32, function: &mut dyn FnMut(Displacement) -> Option<T>) -> Option<T> {
    let mut result = None;
    do_crawl_range(min_radius, max_radius, &mut |d| {
        result = function(d);
        result.is_none()
    });
    result
}

/// Original: `devilution::DoUnLight` (lighting.cpp).
// @port lighting.cpp|devilution::DoUnLight(Point position, uint8_t radius) sha=227b98fb6a37
pub fn do_un_light(ctx: &mut Ctx, position: Point, radius: u8) {
    let radius = radius.wrapping_add(2) as i32;
    // WorldTileRectangle { position, radius }: uint8_t coordinates
    let area = Rectangle::from_center(position, radius);
    let pos = Point::new(area.position.x & 0xff, area.position.y & 0xff);
    for target in crate::engine::geometry::points_in_rectangle(Rectangle::new(pos, area.size)) {
        let t = Point::new(target.x & 0xff, target.y & 0xff);
        if in_dungeon_bounds(t) {
            ctx.gendung.dLight[t.x as usize][t.y as usize] = ctx.gendung.dPreLight[t.x as usize][t.y as usize];
        }
    }
}

/// Original: `devilution::DoLighting` (lighting.cpp).
// @port lighting.cpp|devilution::DoLighting(Point position, uint8_t radius, DisplacementOf<int8_t> offset) sha=8e2422ab15d8
pub fn do_lighting(ctx: &mut Ctx, mut position: Point, radius: u8, offset_in: Displacement) {
    let mut offset = D8 { x: offset_in.delta_x as i8, y: offset_in.delta_y as i8 };
    let mut light = D8::default();
    let mut block = D8::default();
    if offset.x < 0 {
        offset.x += 8;
        position.x -= 1;
    }
    if offset.y < 0 {
        offset.y += 8;
        position.y -= 1;
    }
    let mut dist = offset;
    let mut min_x = 15;
    if position.x - 15 < 0 {
        min_x = position.x + 1;
    }
    let mut max_x = 15;
    if position.x + 15 > MAXDUNX as i32 {
        max_x = MAXDUNX as i32 - position.x;
    }
    let mut min_y = 15;
    if position.y - 15 < 0 {
        min_y = position.y + 1;
    }
    let mut max_y = 15;
    if position.y + 15 > MAXDUNY as i32 {
        max_y = MAXDUNY as i32 - position.y;
    }
    let r = radius as usize;
    if matches!(ctx.gendung.leveltype, DungeonType::Nest | DungeonType::Crypt) {
        let f = ctx.lighting.light_falloffs[r][0];
        if get_light(ctx, position) > f {
            set_light(ctx, position, f);
        }
    } else {
        set_light(ctx, position, 0);
    }
    for i in 0..4 {
        let y_bound = if i > 0 && i < 3 { max_y } else { min_y };
        let x_bound = if i < 2 { max_x } else { min_x };
        for y in 0..y_bound {
            for x in 1..x_bound {
                let linear_distance = ctx.lighting.light_cone_interpolations[offset.x as usize][offset.y as usize][(x + block.x as i32) as usize][(y + block.y as i32) as usize]
                    as usize;
                if linear_distance >= 128 {
                    continue;
                }
                let temp = position + Displacement::new(x, y).rotate(-i);
                let v = ctx.lighting.light_falloffs[r][linear_distance];
                if !in_dungeon_bounds(temp) {
                    continue;
                }
                if v < get_light(ctx, temp) {
                    set_light(ctx, temp, v);
                }
            }
        }
        rotate_radius(&mut offset, &mut dist, &mut light, &mut block);
    }
}

/// Original: `devilution::DoUnVision` (lighting.cpp).
// @port lighting.cpp|devilution::DoUnVision(Point position, uint8_t radius) sha=bf1f4467fedb
pub fn do_un_vision(ctx: &mut Ctx, position: Point, radius: u8) {
    let radius = radius.wrapping_add(2) as i32;
    let area = Rectangle::from_center(position, radius);
    let pos = Point::new(area.position.x & 0xff, area.position.y & 0xff);
    for target in crate::engine::geometry::points_in_rectangle(Rectangle::new(pos, area.size)) {
        let t = Point::new(target.x & 0xff, target.y & 0xff);
        if in_dungeon_bounds(t) {
            ctx.gendung.dFlags[t.x as usize][t.y as usize] &= !(DungeonFlag::Visible | DungeonFlag::Lit);
        }
    }
}

/// Original: `devilution::DoVision` (lighting.cpp).
// @port lighting.cpp|devilution::DoVision(Point position, uint8_t radius, MapExplorationType doAutomap, bool visible) sha=6d85efe149df
pub fn do_vision(ctx: &mut Ctx, position: Point, radius: u8, do_automap: MapExplorationType, visible: bool) {
    do_vision_flags(ctx, position, do_automap, visible);
    const FACTORS: [Displacement; 4] = [Displacement::new(1, 1), Displacement::new(-1, 1), Displacement::new(1, -1), Displacement::new(-1, -1)];
    for factor in FACTORS {
        for j in 0..23 {
            let line_len = radius as i32 - RADIUS_ADJ[j] as i32;
            for k in 0..line_len.max(0) as usize {
                let entry = VISION_CRAWL_TABLE[j][k];
                let crawl = position + entry * factor;
                if !in_dungeon_bounds(crawl) {
                    break;
                }
                let blocker_flag = tile_has_any(ctx, ctx.gendung.dPiece[crawl.x as usize][crawl.y as usize] as i32, TileProperties::BlockLight);
                let mut tile_ok = !blocker_flag;
                if entry.delta_x > 0 && entry.delta_y > 0 {
                    tile_ok = tile_ok || tile_allows_light(ctx, crawl + Displacement::new(-factor.delta_x, 0));
                    tile_ok = tile_ok || tile_allows_light(ctx, crawl + Displacement::new(0, -factor.delta_y));
                }
                if !tile_ok {
                    break;
                }
                do_vision_flags(ctx, crawl, do_automap, visible);
                if blocker_flag {
                    break;
                }
                let trans = ctx.gendung.dTransVal[crawl.x as usize][crawl.y as usize];
                if trans != 0 {
                    ctx.gendung.TransList[trans as u8 as usize] = true;
                }
            }
        }
    }
}

/// Original: `devilution::MakeLightTable` (lighting.cpp).
// @port lighting.cpp|devilution::MakeLightTable() sha=9025feb03237
pub fn make_light_table(ctx: &mut Ctx) {
    let mut shade: u8 = 0;
    const BLACK: u8 = 0;
    const WHITE: i32 = 255;
    for light_table in ctx.lighting.LightTables.iter_mut() {
        let mut color_index: u8 = 0;
        for steps in [16u8, 16, 16, 16, 16, 16, 16, 16, 8, 8, 8, 8, 16, 16, 16, 16, 16, 16] {
            let shading = (shade as u32 * steps as u32 / 16) as u8;
            let shade_start = color_index;
            let shade_end = shade_start.wrapping_add(steps).wrapping_sub(1);
            for step in 0..steps {
                if color_index == BLACK {
                    light_table[color_index as usize] = BLACK;
                    color_index = color_index.wrapping_add(1);
                    continue;
                }
                let mut color = shade_start as i32 + step as i32 + shading as i32;
                if color > shade_end as i32 || color == WHITE {
                    color = BLACK as i32;
                }
                light_table[color_index as usize] = color as u8;
                color_index = color_index.wrapping_add(1);
            }
        }
        shade += 1;
    }
    ctx.lighting.LightTables[15] = [0; 256];
    let lt = ctx.gendung.leveltype;
    if lt == DungeonType::Hell {
        let shades = NumLightingLevels as i32 - 1;
        for i in 0..shades {
            let light_table = &mut ctx.lighting.LightTables[i as usize];
            let range = 16;
            for j in 0..range {
                let mut color = ((((range - 1) << 4) / shades * (shades - i) / range * (j + 1)) as u8) as i32;
                color = 1 + (color >> 4);
                light_table[(j + 1) as usize] = color as u8;
                light_table[(31 - j) as usize] = color as u8;
            }
        }
    } else if matches!(lt, DungeonType::Nest | DungeonType::Crypt) {
        for light_table in ctx.lighting.LightTables.iter_mut() {
            for (i, v) in light_table.iter_mut().take(16).enumerate() {
                *v = i as u8;
            }
        }
        ctx.lighting.LightTables[15][0] = 0;
        for v in ctx.lighting.LightTables[15][1..16].iter_mut() {
            *v = 1;
        }
    }
    let mut buf = [0u8; 256];
    crate::engine::load_file::load_file_in_mem_exact(ctx, "plrgfx\\infra.trn", &mut buf);
    ctx.lighting.InfravisionTable = buf;
    crate::engine::load_file::load_file_in_mem_exact(ctx, "plrgfx\\stone.trn", &mut buf);
    ctx.lighting.StoneTable = buf;
    crate::engine::load_file::load_file_in_mem_exact(ctx, "gendata\\pause.trn", &mut buf);
    ctx.lighting.PauseTable = buf;

    let max_darkness: f32 = 15.0;
    let max_brightness: f32 = 0.0;
    for radius in 0..NUM_LIGHT_RADIUSES {
        let max_distance = (radius + 1) * 8;
        for distance in 0..128usize {
            if distance > max_distance {
                ctx.lighting.light_falloffs[radius][distance] = 15;
            } else {
                let factor = distance as f32 / max_distance as f32;
                let scaled = if matches!(lt, DungeonType::Nest | DungeonType::Crypt) {
                    let brightness = (radius as f64 * 1.25) as f32;
                    (factor * factor * brightness + (max_darkness - brightness)).max(max_brightness)
                } else {
                    factor * max_darkness
                };
                ctx.lighting.light_falloffs[radius][distance] = (scaled + 0.5) as u8;
            }
        }
    }
    for offset_y in 0..8 {
        for offset_x in 0..8 {
            for y in 0..16 {
                for x in 0..16 {
                    let a = 8 * x - offset_y;
                    let b = 8 * y - offset_x;
                    ctx.lighting.light_cone_interpolations[offset_x as usize][offset_y as usize][x as usize][y as usize] = ((a * a + b * b) as f64).sqrt() as u8;
                }
            }
        }
    }
}

/// Original: `devilution::InitLighting` (lighting.cpp).
// @port lighting.cpp|devilution::InitLighting() sha=8aa4ab868001
pub fn init_lighting(ctx: &mut Ctx) {
    let l = &mut ctx.lighting;
    l.ActiveLightCount = 0;
    l.UpdateLighting = false;
    l.update_vision = false;
    for (i, a) in l.ActiveLights.iter_mut().enumerate() {
        *a = i as u8;
    }
    l.VisionActive = [false; MAXVISION];
    ctx.gendung.TransList = [false; 256];
}

/// Original: `devilution::AddLight` (lighting.cpp).
// @port lighting.cpp|devilution::AddLight(Point position, uint8_t radius) sha=30705dfdb6c2
pub fn add_light(ctx: &mut Ctx, position: Point, radius: u8) -> i32 {
    let l = &mut ctx.lighting;
    if l.ActiveLightCount >= MAXLIGHTS as i32 {
        return NO_LIGHT;
    }
    let lid = l.ActiveLights[l.ActiveLightCount as usize] as i32;
    l.ActiveLightCount += 1;
    let light = &mut l.Lights[lid as usize];
    light.position.tile = position;
    light.radius = radius;
    light.position.offset = Displacement::new(0, 0);
    light.isInvalid = false;
    light.hasChanged = false;
    l.UpdateLighting = true;
    lid
}

/// Original: `devilution::AddUnLight` (lighting.cpp).
// @port lighting.cpp|devilution::AddUnLight(int i) sha=5625b926d0b3
pub fn add_un_light(ctx: &mut Ctx, i: i32) {
    if i == NO_LIGHT {
        return;
    }
    ctx.lighting.Lights[i as usize].isInvalid = true;
    ctx.lighting.UpdateLighting = true;
}

/// Original: `devilution::ChangeLightRadius` (lighting.cpp).
// @port lighting.cpp|devilution::ChangeLightRadius(int i, uint8_t radius) sha=8d859710619e
pub fn change_light_radius(ctx: &mut Ctx, i: i32, radius: u8) {
    if i == NO_LIGHT {
        return;
    }
    let light = &mut ctx.lighting.Lights[i as usize];
    light.hasChanged = true;
    light.position.old = light.position.tile;
    light.oldRadius = light.radius;
    light.radius = radius;
    ctx.lighting.UpdateLighting = true;
}

/// Original: `devilution::ChangeLightXY` (lighting.cpp).
// @port lighting.cpp|devilution::ChangeLightXY(int i, Point position) sha=5cf25839f36a
pub fn change_light_xy(ctx: &mut Ctx, i: i32, position: Point) {
    if i == NO_LIGHT {
        return;
    }
    let light = &mut ctx.lighting.Lights[i as usize];
    light.hasChanged = true;
    light.position.old = light.position.tile;
    light.oldRadius = light.radius;
    light.position.tile = position;
    ctx.lighting.UpdateLighting = true;
}

/// Original: `devilution::ChangeLightOffset` (lighting.cpp).
// @port lighting.cpp|devilution::ChangeLightOffset(int i, DisplacementOf<int8_t> offset) sha=b2cb72017f77
pub fn change_light_offset(ctx: &mut Ctx, i: i32, offset: Displacement) {
    if i == NO_LIGHT {
        return;
    }
    let offset = Displacement::new(offset.delta_x as i8 as i32, offset.delta_y as i8 as i32);
    let light = &mut ctx.lighting.Lights[i as usize];
    if light.position.offset == offset {
        return;
    }
    light.hasChanged = true;
    light.position.old = light.position.tile;
    light.oldRadius = light.radius;
    light.position.offset = offset;
    ctx.lighting.UpdateLighting = true;
}

/// Original: `devilution::ChangeLight` (lighting.cpp).
// @port lighting.cpp|devilution::ChangeLight(int i, Point position, uint8_t radius) sha=259dd894ca7e
pub fn change_light(ctx: &mut Ctx, i: i32, position: Point, radius: u8) {
    if i == NO_LIGHT {
        return;
    }
    let light = &mut ctx.lighting.Lights[i as usize];
    light.hasChanged = true;
    light.position.old = light.position.tile;
    light.oldRadius = light.radius;
    light.position.tile = position;
    light.radius = radius;
    ctx.lighting.UpdateLighting = true;
}

/// Original: `devilution::ProcessLightList` (lighting.cpp).
// @port lighting.cpp|devilution::ProcessLightList() sha=e21d8ffc53b7
pub fn process_light_list(ctx: &mut Ctx) {
    if !ctx.lighting.UpdateLighting {
        return;
    }
    for i in 0..ctx.lighting.ActiveLightCount as usize {
        let li = ctx.lighting.ActiveLights[i] as usize;
        let light = ctx.lighting.Lights[li];
        if light.isInvalid {
            do_un_light(ctx, light.position.tile, light.radius);
        }
        if light.hasChanged {
            do_un_light(ctx, light.position.old, light.oldRadius);
            ctx.lighting.Lights[li].hasChanged = false;
        }
    }
    let mut i = 0i32;
    while i < ctx.lighting.ActiveLightCount {
        let li = ctx.lighting.ActiveLights[i as usize] as usize;
        let light = ctx.lighting.Lights[li];
        if light.isInvalid {
            ctx.lighting.ActiveLightCount -= 1;
            let n = ctx.lighting.ActiveLightCount as usize;
            ctx.lighting.ActiveLights.swap(n, i as usize);
            i -= 1;
            i += 1;
            continue;
        }
        let t = light.position.tile;
        if tile_has_any(ctx, ctx.gendung.dPiece[t.x as usize][t.y as usize] as i32, TileProperties::Solid) {
            i += 1;
            continue;
        }
        do_lighting(ctx, t, light.radius, light.position.offset);
        i += 1;
    }
    ctx.lighting.UpdateLighting = false;
}

/// Original: `devilution::SavePreLighting` (lighting.cpp).
// @port lighting.cpp|devilution::SavePreLighting() sha=b6fd7aa72dea
pub fn save_pre_lighting(ctx: &mut Ctx) {
    *ctx.gendung.dPreLight = *ctx.gendung.dLight;
}

/// Original: `devilution::ActivateVision` (lighting.cpp).
// @port lighting.cpp|devilution::ActivateVision(Point position, int r, int id) sha=ffbe4629bd3a
pub fn activate_vision(ctx: &mut Ctx, position: Point, r: i32, id: i32) {
    let v = &mut ctx.lighting.VisionList[id as usize];
    v.position.tile = position;
    v.radius = r as u8;
    v.isInvalid = false;
    v.hasChanged = false;
    ctx.lighting.VisionActive[id as usize] = true;
    ctx.lighting.update_vision = true;
}

/// Original: `devilution::ChangeVisionRadius` (lighting.cpp).
// @port lighting.cpp|devilution::ChangeVisionRadius(int id, int r) sha=493e46c4801f
pub fn change_vision_radius(ctx: &mut Ctx, id: i32, r: i32) {
    let v = &mut ctx.lighting.VisionList[id as usize];
    v.hasChanged = true;
    v.position.old = v.position.tile;
    v.oldRadius = v.radius;
    v.radius = r as u8;
    ctx.lighting.update_vision = true;
}

/// Original: `devilution::ChangeVisionXY` (lighting.cpp).
// @port lighting.cpp|devilution::ChangeVisionXY(int id, Point position) sha=95fc95c9f0eb
pub fn change_vision_xy(ctx: &mut Ctx, id: i32, position: Point) {
    let v = &mut ctx.lighting.VisionList[id as usize];
    v.hasChanged = true;
    v.position.old = v.position.tile;
    v.oldRadius = v.radius;
    v.position.tile = position;
    ctx.lighting.update_vision = true;
}

/// Original: `devilution::ProcessVisionList` (lighting.cpp).
// @port lighting.cpp|devilution::ProcessVisionList() sha=3ba1cc3dc446
pub fn process_vision_list(ctx: &mut Ctx) {
    if !ctx.lighting.update_vision {
        return;
    }
    ctx.gendung.TransList = [false; 256];
    for id in 0..ctx.players.Players.len() {
        if !ctx.lighting.VisionActive[id] {
            continue;
        }
        let vision = ctx.lighting.VisionList[id];
        if !ctx.players.Players[id].plractive || !crate::player::is_on_active_level(ctx, id) {
            do_un_vision(ctx, vision.position.tile, vision.radius);
            ctx.lighting.VisionActive[id] = false;
            continue;
        }
        if vision.hasChanged {
            do_un_vision(ctx, vision.position.old, vision.oldRadius);
            ctx.lighting.VisionList[id].hasChanged = false;
        }
    }
    for id in 0..ctx.players.Players.len() {
        if !ctx.lighting.VisionActive[id] {
            continue;
        }
        let vision = ctx.lighting.VisionList[id];
        let is_me = ctx.players.MyPlayer == Some(id);
        let mut doautomap = MAP_EXP_SELF;
        if !is_me {
            doautomap = if ctx.players.Players[id].friendlyMode { MAP_EXP_OTHERS } else { MAP_EXP_NONE };
        }
        do_vision(ctx, vision.position.tile, vision.radius, doautomap, is_me);
    }
    ctx.lighting.update_vision = false;
}

/// Original: `devilution::lighting_color_cycling` (lighting.cpp).
// @port lighting.cpp|devilution::lighting_color_cycling() sha=cb063de70765
pub fn lighting_color_cycling(ctx: &mut Ctx) {
    for light_table in ctx.lighting.LightTables.iter_mut() {
        light_table[1..32].rotate_left(1);
    }
}

pub use crate::objects::redo_player_vision;
