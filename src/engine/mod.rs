//! `Source/engine/*`
pub mod actor_position;
pub mod animationinfo;
pub mod assets;
pub mod backbuffer_state;
pub mod demomode;
pub mod dx;
pub mod events;
pub mod geometry;
pub mod load_file;
pub mod palette;
pub mod path;
pub mod random;
pub mod render;
pub mod surface;
pub mod trn;
pub mod sound;
pub mod clx_sprite;
pub mod load_sprites;

use crate::ctx::Ctx;

/// Original: `devilution::GetAnimationFrame` (engine.h). `fps` defaults to 60 in the original
/// (it is really milliseconds per frame).
// @port engine.h|devilution::GetAnimationFrame(int frames, int fps = 60) sha=b8f57ee56fbc
pub fn get_animation_frame(ctx: &Ctx, frames: i32, fps: i32) -> i32 {
    let frame = (ctx.platform.ticks() / fps as u32) as i32 % frames;
    if frame > frames { 0 } else { frame }
}

/// Original: `DrawHalfTransparentUnalignedBlendedRectTo` (engine.cpp). The 16-bit LUT variant of
/// the original is an optimisation with identical output.
// @port engine.cpp|devilution::DrawHalfTransparentUnalignedBlendedRectTo(const Surface &out, unsigned sx, unsigned sy, unsigned width, unsigned height) sha=bf40f40c6c6c
fn draw_half_transparent_unaligned_blended_rect_to(ctx: &Ctx, out: &surface::Surface, sx: i32, sy: i32, width: i32, height: i32) {
    let lut = &ctx.dx.pal.palette_transparency_lookup[0];
    for y in 0..height {
        let row = out.row(sy + y);
        for x in 0..width {
            let p = &mut row[(sx + x) as usize];
            *p = lut[*p as usize];
        }
    }
}

/// Original: `devilution::DrawHorizontalLine` (engine.cpp).
// @port engine.cpp|devilution::DrawHorizontalLine(const Surface &out, Point from, int width, std::uint8_t colorIndex) sha=8612f55ab043
pub fn draw_horizontal_line(out: &surface::Surface, mut from: geometry::Point, mut width: i32, color_index: u8) {
    if from.y < 0 || from.y >= out.h() || from.x >= out.w() || width <= 0 || from.x + width <= 0 {
        return;
    }
    if from.x < 0 {
        width += from.x;
        from.x = 0;
    }
    if from.x + width > out.w() {
        width = out.w() - from.x;
    }
    unsafe_draw_horizontal_line(out, from, width, color_index);
}

/// Original: `devilution::UnsafeDrawHorizontalLine` (engine.cpp).
// @port engine.cpp|devilution::UnsafeDrawHorizontalLine(const Surface &out, Point from, int width, std::uint8_t colorIndex) sha=3b4507e9c2e7
pub fn unsafe_draw_horizontal_line(out: &surface::Surface, from: geometry::Point, width: i32, color_index: u8) {
    let row = out.row(from.y);
    row[from.x as usize..(from.x + width) as usize].fill(color_index);
}

/// Original: `devilution::DrawVerticalLine` (engine.cpp).
// @port engine.cpp|devilution::DrawVerticalLine(const Surface &out, Point from, int height, std::uint8_t colorIndex) sha=cf44e7a19554
pub fn draw_vertical_line(out: &surface::Surface, mut from: geometry::Point, mut height: i32, color_index: u8) {
    if from.x < 0 || from.x >= out.w() || from.y >= out.h() || height <= 0 || from.y + height <= 0 {
        return;
    }
    if from.y < 0 {
        height += from.y;
        from.y = 0;
    }
    if from.y + height > out.h() {
        height = (from.y + height) - out.h();
    }
    unsafe_draw_vertical_line(out, from, height, color_index);
}

/// Original: `devilution::UnsafeDrawVerticalLine` (engine.cpp).
// @port engine.cpp|devilution::UnsafeDrawVerticalLine(const Surface &out, Point from, int height, std::uint8_t colorIndex) sha=69e0b6020b16
pub fn unsafe_draw_vertical_line(out: &surface::Surface, from: geometry::Point, height: i32, color_index: u8) {
    for y in 0..height {
        out.put(from.x, from.y + y, color_index);
    }
}

/// Original: `devilution::DrawHalfTransparentRectTo` (engine.cpp).
// @port engine.cpp|devilution::DrawHalfTransparentRectTo(const Surface &out, int sx, int sy, int width, int height) sha=6a2c5489a8b5
pub fn draw_half_transparent_rect_to(ctx: &Ctx, out: &surface::Surface, mut sx: i32, mut sy: i32, mut width: i32, mut height: i32) {
    if sx + width < 0 || sy + height < 0 || sx >= out.w() || sy >= out.h() {
        return;
    }
    if sx < 0 {
        width += sx;
        sx = 0;
    } else if sx + width >= out.w() {
        width = out.w() - sx;
    }
    if sy < 0 {
        height += sy;
        sy = 0;
    } else if sy + height >= out.h() {
        height = out.h() - sy;
    }
    draw_half_transparent_unaligned_blended_rect_to(ctx, out, sx, sy, width, height);
}

/// Original: `devilution::UnsafeDrawBorder2px` (engine.cpp).
// @port engine.cpp|devilution::UnsafeDrawBorder2px(const Surface &out, Rectangle rect, uint8_t color) sha=050aa8a08aec
pub fn unsafe_draw_border_2px(out: &surface::Surface, rect: geometry::Rectangle, color: u8) {
    let (w, h) = (rect.size.width, rect.size.height);
    let (x0, mut y) = (rect.position.x, rect.position.y);
    for _ in 0..2 {
        out.row(y)[x0 as usize..(x0 + w) as usize].fill(color);
        y += 1;
    }
    for _ in 4..h {
        let row = out.row(y);
        row[x0 as usize] = color;
        row[x0 as usize + 1] = color;
        row[(x0 + w - 2) as usize] = color;
        row[(x0 + w - 1) as usize] = color;
        y += 1;
    }
    for _ in 0..2 {
        out.row(y)[x0 as usize..(x0 + w) as usize].fill(color);
        y += 1;
    }
}

/// Original: `devilution::GetDirection` (engine.cpp).
// @port engine.cpp|devilution::GetDirection(Point start, Point destination) sha=dcb92e0f58f2
pub fn get_direction(start: geometry::Point, destination: geometry::Point) -> geometry::Direction {
    use geometry::Direction as D;
    let mut mx = destination.x - start.x;
    let mut my = destination.y - start.y;
    let mut md;
    if mx >= 0 {
        if my >= 0 {
            if 5 * mx <= my * 2 {
                return D::SouthWest;
            }
            md = D::South;
        } else {
            my = -my;
            if 5 * mx <= my * 2 {
                return D::NorthEast;
            }
            md = D::East;
        }
        if 5 * my <= mx * 2 {
            md = D::SouthEast;
        }
    } else {
        mx = -mx;
        if my >= 0 {
            if 5 * mx <= my * 2 {
                return D::SouthWest;
            }
            md = D::West;
        } else {
            my = -my;
            if 5 * mx <= my * 2 {
                return D::NorthEast;
            }
            md = D::North;
        }
        if 5 * my <= mx * 2 {
            md = D::NorthWest;
        }
    }
    md
}

/// Original: `devilution::CalculateWidth2` (engine.cpp).
// @port engine.cpp|devilution::CalculateWidth2(int width) sha=b922aba9c107
pub fn calculate_width2(width: i32) -> i32 {
    (width - 64) / 2
}
