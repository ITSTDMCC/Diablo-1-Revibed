//! `Source/engine/render/automap_render.cpp`: line drawing routines for the automap.
//!
//! `DrawMapLine*` draw 2 horizontal pixels for each vertical step; `DrawMapLineSteep*` draw 2
//! vertical pixels for each horizontal step. Both draw one extra pixel at the end and clip to
//! the output buffer.

use crate::engine::geometry::Point;
use crate::engine::surface::Surface;

/// Original: `DrawMapLine<DirX, DirY>` (engine/render/automap_render.cpp).
// @port engine/render/automap_render.cpp|devilution::DrawMapLine(const Surface &out, Point from, int height, std::uint8_t colorIndex) sha=0d6a88ffc6f4
fn draw_map_line(out: &Surface, mut from: Point, mut height: i32, color_index: u8, dir_x: i32, dir_y: i32) {
    while height > 0 {
        height -= 1;
        out.set_pixel(from.x, from.y + 1, 0);
        out.set_pixel(from.x, from.y, color_index);
        from.x += dir_x;
        out.set_pixel(from.x, from.y + 1, 0);
        out.set_pixel(from.x, from.y, color_index);
        from.x += dir_x;
        from.y += dir_y;
    }
    out.set_pixel(from.x, from.y + 1, 0);
    out.set_pixel(from.x, from.y, color_index);
}

/// Original: `DrawMapLineSteep<DirX, DirY>` (engine/render/automap_render.cpp).
// @port engine/render/automap_render.cpp|devilution::DrawMapLineSteep(const Surface &out, Point from, int width, std::uint8_t colorIndex) sha=3eb10301afe7
fn draw_map_line_steep(out: &Surface, mut from: Point, mut width: i32, color_index: u8, dir_x: i32, dir_y: i32) {
    while width > 0 {
        width -= 1;
        out.set_pixel(from.x, from.y + 1, 0);
        out.set_pixel(from.x, from.y, color_index);
        from.y += dir_y;
        out.set_pixel(from.x, from.y + 1, 0);
        out.set_pixel(from.x, from.y, color_index);
        from.y += dir_y;
        from.x += dir_x;
    }
    out.set_pixel(from.x, from.y + 1, 0);
    out.set_pixel(from.x, from.y, color_index);
}

const EAST: i32 = 1;
const WEST: i32 = -1;
const SOUTH: i32 = 1;
const NORTH: i32 = -1;

// @port engine/render/automap_render.cpp|devilution::DrawMapLineNE(const Surface &out, Point from, int height, std::uint8_t colorIndex) sha=3203953f8cee
pub fn draw_map_line_ne(out: &Surface, from: Point, height: i32, color_index: u8) {
    draw_map_line(out, from, height, color_index, EAST, NORTH);
}

// @port engine/render/automap_render.cpp|devilution::DrawMapLineSE(const Surface &out, Point from, int height, std::uint8_t colorIndex) sha=de9f21fd46e0
pub fn draw_map_line_se(out: &Surface, from: Point, height: i32, color_index: u8) {
    draw_map_line(out, from, height, color_index, EAST, SOUTH);
}

// @port engine/render/automap_render.cpp|devilution::DrawMapLineNW(const Surface &out, Point from, int height, std::uint8_t colorIndex) sha=0a052ef0ab8b
pub fn draw_map_line_nw(out: &Surface, from: Point, height: i32, color_index: u8) {
    draw_map_line(out, from, height, color_index, WEST, NORTH);
}

// @port engine/render/automap_render.cpp|devilution::DrawMapLineSW(const Surface &out, Point from, int height, std::uint8_t colorIndex) sha=db94c7238486
pub fn draw_map_line_sw(out: &Surface, from: Point, height: i32, color_index: u8) {
    draw_map_line(out, from, height, color_index, WEST, SOUTH);
}

// @port engine/render/automap_render.cpp|devilution::DrawMapLineSteepNE(const Surface &out, Point from, int width, std::uint8_t colorIndex) sha=3f1b840698f2
pub fn draw_map_line_steep_ne(out: &Surface, from: Point, width: i32, color_index: u8) {
    draw_map_line_steep(out, from, width, color_index, EAST, NORTH);
}

// @port engine/render/automap_render.cpp|devilution::DrawMapLineSteepSE(const Surface &out, Point from, int width, std::uint8_t colorIndex) sha=ca1216b6c4c4
pub fn draw_map_line_steep_se(out: &Surface, from: Point, width: i32, color_index: u8) {
    draw_map_line_steep(out, from, width, color_index, EAST, SOUTH);
}

// @port engine/render/automap_render.cpp|devilution::DrawMapLineSteepNW(const Surface &out, Point from, int width, std::uint8_t colorIndex) sha=234725617e88
pub fn draw_map_line_steep_nw(out: &Surface, from: Point, width: i32, color_index: u8) {
    draw_map_line_steep(out, from, width, color_index, WEST, NORTH);
}

// @port engine/render/automap_render.cpp|devilution::DrawMapLineSteepSW(const Surface &out, Point from, int width, std::uint8_t colorIndex) sha=504457cd9432
pub fn draw_map_line_steep_sw(out: &Surface, from: Point, width: i32, color_index: u8) {
    draw_map_line_steep(out, from, width, color_index, WEST, SOUTH);
}
