//! `Source/engine/render/dun_render.cpp`: rendering the level tiles (micro tiles of the
//! dungeon CEL).
//!
//! The original instantiates every combination of light type, transparency and mask through
//! templates; here they are runtime parameters with the same branch structure. Pointer
//! arithmetic follows the original (wrapping, only dereferenced inside the clipped area).

use crate::ctx::Ctx;
use crate::engine::geometry::Point;
use crate::engine::surface::Surface;
use crate::enums::{MaskType, TileType};
use crate::lighting::LightsMax;

pub const TILE_WIDTH: i32 = 64;
pub const TILE_HEIGHT: i32 = 32;

/// Width of a tile rendering primitive.
const WIDTH: i32 = TILE_WIDTH / 2;
/// Height of a tile rendering primitive (except triangles).
const HEIGHT: i32 = TILE_HEIGHT;
/// Height of the lower triangle of a triangular or a trapezoid tile.
const LOWER_HEIGHT: i32 = TILE_HEIGHT / 2;
/// Height of the upper triangle of a triangular tile.
const TRIANGLE_UPPER_HEIGHT: i32 = TILE_HEIGHT / 2 - 1;
/// Height of the upper rectangle of a trapezoid tile.
const TRAPEZOID_UPPER_HEIGHT: i32 = TILE_HEIGHT / 2;
const TRIANGLE_HEIGHT: i32 = LOWER_HEIGHT + TRIANGLE_UPPER_HEIGHT;
/// For triangles, for each pixel drawn vertically, this many pixels are drawn horizontally.
const X_STEP: i32 = 2;

/// `LevelCelBlock`
#[derive(Clone, Copy, Debug)]
pub struct LevelCelBlock(pub u16);

impl LevelCelBlock {
    pub fn has_value(self) -> bool {
        self.0 != 0
    }
    pub fn type_(self) -> TileType {
        TileType::from_raw(((self.0 & 0x7000) >> 12) as u8)
    }
    pub fn frame(self) -> u16 {
        self.0 & 0xFFF
    }
}

/// Original: `GetTileHeight` (engine/render/dun_render.cpp).
// @port engine/render/dun_render.cpp|devilution::GetTileHeight(TileType tile) sha=864b443a76ae
fn get_tile_height(tile: TileType) -> i32 {
    if tile == TileType::LeftTriangle || tile == TileType::RightTriangle {
        return TRIANGLE_HEIGHT;
    }
    HEIGHT
}

/// `LightType`
#[derive(Clone, Copy, PartialEq, Eq)]
enum LightType {
    FullyDark,
    PartiallyLit,
    FullyLit,
}

/// The template parameters of the original's blitters.
#[derive(Clone, Copy)]
struct Ctl<'a> {
    light: LightType,
    tbl: &'a [u8; 256],
    lut: &'a [[u8; 256]; 256],
}

/// `IsFoliage` / `SkipTransparentPixels`
fn skip_transparent_pixels(opaque_prefix: bool, prefix_increment: i8) -> bool {
    prefix_increment != 0 && (opaque_prefix == (prefix_increment > 0))
}

/// `LowerHalfTransparent`
fn lower_half_transparent(opaque_prefix: bool, prefix_increment: i8) -> bool {
    opaque_prefix == (prefix_increment >= 0)
}

/// `InitPrefix()`
fn init_prefix(prefix_increment: i8) -> i8 {
    if prefix_increment >= 0 {
        -32
    } else {
        64
    }
}

/// `InitPrefix(y)`
fn init_prefix_y(prefix_increment: i8, y: i8) -> i8 {
    init_prefix(prefix_increment).wrapping_add(prefix_increment.wrapping_mul(y))
}

/// `RenderLineOpaque<Light>`
#[inline(always)]
unsafe fn render_line_opaque(c: Ctl, dst: *mut u8, src: *const u8, n: usize) {
    match c.light {
        // BlitFillDirect
        LightType::FullyDark => std::ptr::write_bytes(dst, 0, n),
        // BlitPixelsDirect
        LightType::FullyLit => std::ptr::copy_nonoverlapping(src, dst, n),
        // BlitPixelsWithMap
        LightType::PartiallyLit => {
            for i in 0..n {
                *dst.add(i) = c.tbl[*src.add(i) as usize];
            }
        }
    }
}

/// `RenderLineTransparent<Light>`
#[inline(always)]
unsafe fn render_line_transparent(c: Ctl, dst: *mut u8, src: *const u8, n: usize) {
    match c.light {
        // BlitFillBlended(dst, n, 0)
        LightType::FullyDark => {
            let tbl = &c.lut[0];
            for i in 0..n {
                *dst.add(i) = tbl[*dst.add(i) as usize];
            }
        }
        // BlitPixelsBlended
        LightType::FullyLit => {
            for i in 0..n {
                *dst.add(i) = c.lut[*dst.add(i) as usize][*src.add(i) as usize];
            }
        }
        // BlitPixelsBlendedWithMap
        LightType::PartiallyLit => {
            for i in 0..n {
                *dst.add(i) = c.lut[*dst.add(i) as usize][c.tbl[*src.add(i) as usize] as usize];
            }
        }
    }
}

/// `RenderLineTransparentOrOpaque<Light, Transparent>`
#[inline(always)]
unsafe fn render_line_transparent_or_opaque(c: Ctl, transparent: bool, dst: *mut u8, src: *const u8, width: i32) {
    if width <= 0 {
        return;
    }
    if transparent {
        render_line_transparent(c, dst, src, width as usize);
    } else {
        render_line_opaque(c, dst, src, width as usize);
    }
}

/// `RenderLineTransparentAndOpaque<Light, OpaquePrefix, PrefixIncrement>`
#[inline(always)]
unsafe fn render_line_transparent_and_opaque(c: Ctl, opaque_prefix: bool, prefix_increment: i8, dst: *mut u8, src: *const u8, prefix_width: i32, width: i32) {
    let skip = skip_transparent_pixels(opaque_prefix, prefix_increment);
    if opaque_prefix {
        render_line_opaque(c, dst, src, prefix_width as usize);
        if !skip {
            render_line_transparent(c, dst.add(prefix_width as usize), src.add(prefix_width as usize), (width - prefix_width) as usize);
        }
    } else {
        if !skip {
            render_line_transparent(c, dst, src, prefix_width as usize);
        }
        render_line_opaque(c, dst.add(prefix_width as usize), src.add(prefix_width as usize), (width - prefix_width) as usize);
    }
}

/// `RenderLine<Light, OpaquePrefix, PrefixIncrement>`
#[inline(always)]
unsafe fn render_line(c: Ctl, opaque_prefix: bool, prefix_increment: i8, dst: *mut u8, src: *const u8, n: i32, prefix: i8) {
    let skip = skip_transparent_pixels(opaque_prefix, prefix_increment);
    if prefix_increment == 0 {
        render_line_transparent_or_opaque(c, opaque_prefix, dst, src, n);
    } else if prefix as i32 >= n as i8 as i32 {
        // We clamp the prefix to (0, n] and avoid calling `RenderLineTransparent/Opaque` with width=0.
        if opaque_prefix {
            render_line_opaque(c, dst, src, n as usize);
        } else if !skip {
            render_line_transparent(c, dst, src, n as usize);
        }
    } else if prefix <= 0 {
        if !opaque_prefix {
            render_line_opaque(c, dst, src, n as usize);
        } else if !skip {
            render_line_transparent(c, dst, src, n as usize);
        }
    } else {
        render_line_transparent_and_opaque(c, opaque_prefix, prefix_increment, dst, src, prefix as i32, n);
    }
}

/// `Clip`
#[derive(Clone, Copy, Debug)]
struct Clip {
    top: i32,
    bottom: i32,
    left: i32,
    right: i32,
    width: i32,
    height: i32,
}

/// Original: `CalculateClip` (engine/render/dun_render.cpp).
// @port engine/render/dun_render.cpp|devilution::CalculateClip(int_fast16_t x, int_fast16_t y, int_fast16_t w, int_fast16_t h, const Surface &out) sha=29fc479862ee
fn calculate_clip(x: i32, y: i32, w: i32, h: i32, out: &Surface) -> Clip {
    let top = if y + 1 < h { h - (y + 1) } else { 0 };
    let bottom = if y + 1 > out.h() { (y + 1) - out.h() } else { 0 };
    let left = if x < 0 { -x } else { 0 };
    let right = if x + w > out.w() { x + w - out.w() } else { 0 };
    Clip { top, bottom, left, right, width: w - left - right, height: h - top - bottom }
}

#[inline(always)]
fn up(dst: *mut u8, n: isize) -> *mut u8 {
    dst.wrapping_offset(-n)
}

/// `RenderSquare` (full and clipped)
unsafe fn render_square(c: Ctl, transparent: bool, mut dst: *mut u8, pitch: isize, mut src: *const u8, clip: Clip) {
    if clip.width == WIDTH && clip.height == HEIGHT {
        for _ in 0..HEIGHT {
            render_line_transparent_or_opaque(c, transparent, dst, src, WIDTH);
            src = src.add(WIDTH as usize);
            dst = up(dst, pitch);
        }
    } else {
        src = src.add((clip.bottom * HEIGHT + clip.left) as usize);
        for _ in 0..clip.height {
            render_line_transparent_or_opaque(c, transparent, dst, src, clip.width);
            src = src.add(WIDTH as usize);
            dst = up(dst, pitch);
        }
    }
}

/// `RenderTransparentSquareFull`
unsafe fn render_transparent_square_full(c: Ctl, opaque_prefix: bool, prefix_increment: i8, mut dst: *mut u8, pitch: isize, mut src: *const u8) {
    let mut prefix = init_prefix(prefix_increment);
    for _ in 0..HEIGHT {
        let mut draw_width = WIDTH;
        while draw_width > 0 {
            let mut v = *src as i8 as i32;
            src = src.add(1);
            if v > 0 {
                render_line(c, opaque_prefix, prefix_increment, dst, src, v, prefix.wrapping_sub((WIDTH - draw_width) as i8));
                src = src.add(v as usize);
            } else {
                v = -v;
            }
            dst = dst.wrapping_offset(v as isize);
            draw_width -= v;
        }
        prefix = prefix.wrapping_add(prefix_increment);
        dst = up(dst, pitch + WIDTH as isize);
    }
}

/// `RenderTransparentSquareClipped`
unsafe fn render_transparent_square_clipped(c: Ctl, opaque_prefix: bool, prefix_increment: i8, mut dst: *mut u8, pitch: isize, mut src: *const u8, clip: Clip) {
    let skip_rest_of_the_line = |src: &mut *const u8, mut remaining_width: i32| {
        while remaining_width > 0 {
            let v = **src as i8 as i32;
            *src = src.add(1);
            if v > 0 {
                *src = src.add(v as usize);
                remaining_width -= v;
            } else {
                remaining_width -= -v;
            }
        }
        debug_assert!(remaining_width == 0);
    };
    // Skip the bottom clipped lines.
    for _ in 0..clip.bottom {
        skip_rest_of_the_line(&mut src, WIDTH);
    }
    let mut prefix = init_prefix_y(prefix_increment, clip.bottom as i8);
    for _ in 0..clip.height {
        let mut draw_width = clip.width;
        // Skip initial src if clipping on the left.
        // Handles overshoot, i.e. when the RLE segment goes into the unclipped area.
        let mut remaining_left_clip = clip.left;
        while remaining_left_clip > 0 {
            let mut v = *src as i8 as i32;
            src = src.add(1);
            if v > 0 {
                if v > remaining_left_clip {
                    let overshoot = v - remaining_left_clip;
                    render_line(c, opaque_prefix, prefix_increment, dst, src.add(remaining_left_clip as usize), overshoot, prefix.wrapping_sub((WIDTH - remaining_left_clip) as i8));
                    dst = dst.wrapping_offset(overshoot as isize);
                    draw_width -= overshoot;
                }
                src = src.add(v as usize);
            } else {
                v = -v;
                if v > remaining_left_clip {
                    let overshoot = v - remaining_left_clip;
                    dst = dst.wrapping_offset(overshoot as isize);
                    draw_width -= overshoot;
                }
            }
            remaining_left_clip -= v;
        }
        // Draw the non-clipped segment
        while draw_width > 0 {
            let mut v = *src as i8 as i32;
            src = src.add(1);
            if v > 0 {
                if v > draw_width {
                    render_line(c, opaque_prefix, prefix_increment, dst, src, draw_width, prefix.wrapping_sub((WIDTH - draw_width) as i8));
                    src = src.add(v as usize);
                    dst = dst.wrapping_offset(draw_width as isize);
                    draw_width -= v;
                    break;
                }
                render_line(c, opaque_prefix, prefix_increment, dst, src, v, prefix.wrapping_sub((WIDTH - draw_width) as i8));
                src = src.add(v as usize);
            } else {
                v = -v;
                if v > draw_width {
                    dst = dst.wrapping_offset(draw_width as isize);
                    draw_width -= v;
                    break;
                }
            }
            dst = dst.wrapping_offset(v as isize);
            draw_width -= v;
        }
        // Skip the rest of src line if clipping on the right
        debug_assert!(draw_width <= 0);
        skip_rest_of_the_line(&mut src, clip.right + draw_width);
        prefix = prefix.wrapping_add(prefix_increment);
        dst = up(dst, pitch + clip.width as isize);
    }
}

/// `RenderTransparentSquare`
unsafe fn render_transparent_square(c: Ctl, opaque_prefix: bool, prefix_increment: i8, dst: *mut u8, pitch: isize, src: *const u8, clip: Clip) {
    if clip.width == WIDTH && clip.height == HEIGHT {
        render_transparent_square_full(c, opaque_prefix, prefix_increment, dst, pitch, src);
    } else {
        render_transparent_square_clipped(c, opaque_prefix, prefix_increment, dst, pitch, src, clip);
    }
}

/// `DiamondClipY`
#[derive(Clone, Copy, Debug)]
struct DiamondClipY {
    lower_bottom: i32,
    lower_top: i32,
    upper_bottom: i32,
    upper_top: i32,
}

/// Original: `CalculateDiamondClipY` (engine/render/dun_render.cpp).
// @port engine/render/dun_render.cpp|devilution::CalculateDiamondClipY(const Clip &clip) sha=f2cc56336f7a
fn calculate_diamond_clip_y(clip: &Clip, upper_height: i32) -> DiamondClipY {
    if clip.bottom > LOWER_HEIGHT {
        DiamondClipY { lower_bottom: LOWER_HEIGHT, upper_bottom: clip.bottom - LOWER_HEIGHT, lower_top: 0, upper_top: 0 }
    } else if clip.top > upper_height {
        DiamondClipY { upper_top: upper_height, lower_top: clip.top - upper_height, upper_bottom: 0, lower_bottom: 0 }
    } else {
        DiamondClipY { upper_top: clip.top, lower_bottom: clip.bottom, lower_top: 0, upper_bottom: 0 }
    }
}

/// Original: `CalculateTriangleSourceSkipLowerBottom` (engine/render/dun_render.cpp).
// @port engine/render/dun_render.cpp|devilution::CalculateTriangleSourceSkipLowerBottom(int_fast16_t numLines) sha=a86d2ff534d7
fn calculate_triangle_source_skip_lower_bottom(num_lines: i32) -> usize {
    (X_STEP * num_lines * (num_lines + 1) / 2 + 2 * ((num_lines + 1) / 2)) as usize
}

/// Original: `CalculateTriangleSourceSkipUpperBottom` (engine/render/dun_render.cpp).
// @port engine/render/dun_render.cpp|devilution::CalculateTriangleSourceSkipUpperBottom(int_fast16_t numLines) sha=fb079fde2d78
fn calculate_triangle_source_skip_upper_bottom(num_lines: i32) -> usize {
    (2 * TRIANGLE_UPPER_HEIGHT * num_lines - num_lines * (num_lines - 1) + 2 * ((num_lines + 1) / 2)) as usize
}

/// `RenderLeftTriangleLower` (full; `dst`/`src` advanced as by reference)
unsafe fn render_left_triangle_lower(c: Ctl, transparent: bool, dst: &mut *mut u8, pitch: isize, src: &mut *const u8) {
    *dst = dst.wrapping_offset((X_STEP * (LOWER_HEIGHT - 1)) as isize);
    for i in 1..=LOWER_HEIGHT {
        *src = src.add((2 * (i % 2)) as usize);
        let width = X_STEP * i;
        render_line_transparent_or_opaque(c, transparent, *dst, *src, width);
        *src = src.add(width as usize);
        *dst = up(*dst, pitch + X_STEP as isize);
    }
}

/// `RenderLeftTriangleLowerClipVertical`
unsafe fn render_left_triangle_lower_clip_vertical(c: Ctl, transparent: bool, clip_y: &DiamondClipY, dst: &mut *mut u8, pitch: isize, src: &mut *const u8) {
    *src = src.add(calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom));
    *dst = dst.wrapping_offset((X_STEP * (LOWER_HEIGHT - clip_y.lower_bottom - 1)) as isize);
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        *src = src.add((2 * (i % 2)) as usize);
        let width = X_STEP * i;
        render_line_transparent_or_opaque(c, transparent, *dst, *src, width);
        *src = src.add(width as usize);
        *dst = up(*dst, pitch + X_STEP as isize);
    }
}

/// `RenderLeftTriangleLowerClipLeftAndVertical`
unsafe fn render_left_triangle_lower_clip_left_and_vertical(c: Ctl, transparent: bool, clip_left: i32, clip_y: &DiamondClipY, dst: &mut *mut u8, pitch: isize, src: &mut *const u8) {
    *src = src.add(calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom));
    *dst = dst.wrapping_offset((X_STEP * (LOWER_HEIGHT - clip_y.lower_bottom - 1) - clip_left) as isize);
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        *src = src.add((2 * (i % 2)) as usize);
        let width = X_STEP * i;
        let start_x = WIDTH - X_STEP * i;
        let skip = if start_x < clip_left { clip_left - start_x } else { 0 };
        if width > skip {
            render_line_transparent_or_opaque(c, transparent, dst.wrapping_offset(skip as isize), src.add(skip as usize), width - skip);
        }
        *src = src.add(width as usize);
        *dst = up(*dst, pitch + X_STEP as isize);
    }
}

/// `RenderLeftTriangleLowerClipRightAndVertical`
unsafe fn render_left_triangle_lower_clip_right_and_vertical(c: Ctl, transparent: bool, clip_right: i32, clip_y: &DiamondClipY, dst: &mut *mut u8, pitch: isize, src: &mut *const u8) {
    *src = src.add(calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom));
    *dst = dst.wrapping_offset((X_STEP * (LOWER_HEIGHT - clip_y.lower_bottom - 1)) as isize);
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        *src = src.add((2 * (i % 2)) as usize);
        let width = X_STEP * i;
        if width > clip_right {
            render_line_transparent_or_opaque(c, transparent, *dst, *src, width - clip_right);
        }
        *src = src.add(width as usize);
        *dst = up(*dst, pitch + X_STEP as isize);
    }
}

/// `RenderLeftTriangle` (all clip variants)
unsafe fn render_left_triangle(c: Ctl, transparent: bool, mut dst: *mut u8, pitch: isize, mut src: *const u8, clip: Clip) {
    if clip.width == WIDTH {
        if clip.height == TRIANGLE_HEIGHT {
            // RenderLeftTriangleFull
            render_left_triangle_lower(c, transparent, &mut dst, pitch, &mut src);
            dst = dst.wrapping_offset((2 * X_STEP) as isize);
            for i in 1..=TRIANGLE_UPPER_HEIGHT {
                src = src.add((2 * (i % 2)) as usize);
                let width = WIDTH - X_STEP * i;
                render_line_transparent_or_opaque(c, transparent, dst, src, width);
                src = src.add(width as usize);
                dst = up(dst, pitch - X_STEP as isize);
            }
        } else {
            // RenderLeftTriangleClipVertical
            let clip_y = calculate_diamond_clip_y(&clip, TRIANGLE_UPPER_HEIGHT);
            render_left_triangle_lower_clip_vertical(c, transparent, &clip_y, &mut dst, pitch, &mut src);
            src = src.add(calculate_triangle_source_skip_upper_bottom(clip_y.upper_bottom));
            dst = dst.wrapping_offset((2 * X_STEP + X_STEP * clip_y.upper_bottom) as isize);
            let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
            for i in (1 + clip_y.upper_bottom)..=upper_max {
                src = src.add((2 * (i % 2)) as usize);
                let width = WIDTH - X_STEP * i;
                render_line_transparent_or_opaque(c, transparent, dst, src, width);
                src = src.add(width as usize);
                dst = up(dst, pitch - X_STEP as isize);
            }
        }
    } else if clip.right == 0 {
        // RenderLeftTriangleClipLeftAndVertical
        let clip_y = calculate_diamond_clip_y(&clip, TRIANGLE_UPPER_HEIGHT);
        let clip_left = clip.left;
        render_left_triangle_lower_clip_left_and_vertical(c, transparent, clip_left, &clip_y, &mut dst, pitch, &mut src);
        src = src.add(calculate_triangle_source_skip_upper_bottom(clip_y.upper_bottom));
        dst = dst.wrapping_offset((2 * X_STEP + X_STEP * clip_y.upper_bottom) as isize);
        let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
        for i in (1 + clip_y.upper_bottom)..=upper_max {
            src = src.add((2 * (i % 2)) as usize);
            let width = WIDTH - X_STEP * i;
            let start_x = X_STEP * i;
            let skip = if start_x < clip_left { clip_left - start_x } else { 0 };
            render_line_transparent_or_opaque(c, transparent, dst.wrapping_offset(skip as isize), src.add(skip as usize), if width > skip { width - skip } else { 0 });
            src = src.add(width as usize);
            dst = up(dst, pitch - X_STEP as isize);
        }
    } else {
        // RenderLeftTriangleClipRightAndVertical
        let clip_y = calculate_diamond_clip_y(&clip, TRIANGLE_UPPER_HEIGHT);
        let clip_right = clip.right;
        render_left_triangle_lower_clip_right_and_vertical(c, transparent, clip_right, &clip_y, &mut dst, pitch, &mut src);
        src = src.add(calculate_triangle_source_skip_upper_bottom(clip_y.upper_bottom));
        dst = dst.wrapping_offset((2 * X_STEP + X_STEP * clip_y.upper_bottom) as isize);
        let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
        for i in (1 + clip_y.upper_bottom)..=upper_max {
            src = src.add((2 * (i % 2)) as usize);
            let width = WIDTH - X_STEP * i;
            if width <= clip_right {
                break;
            }
            render_line_transparent_or_opaque(c, transparent, dst, src, width - clip_right);
            src = src.add(width as usize);
            dst = up(dst, pitch - X_STEP as isize);
        }
    }
}

/// `RenderRightTriangleLower`
unsafe fn render_right_triangle_lower(c: Ctl, transparent: bool, dst: &mut *mut u8, pitch: isize, src: &mut *const u8) {
    for i in 1..=LOWER_HEIGHT {
        let width = X_STEP * i;
        render_line_transparent_or_opaque(c, transparent, *dst, *src, width);
        *src = src.add((width + 2 * (i % 2)) as usize);
        *dst = up(*dst, pitch);
    }
}

/// `RenderRightTriangleLowerClipVertical`
unsafe fn render_right_triangle_lower_clip_vertical(c: Ctl, transparent: bool, clip_y: &DiamondClipY, dst: &mut *mut u8, pitch: isize, src: &mut *const u8) {
    *src = src.add(calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom));
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        let width = X_STEP * i;
        render_line_transparent_or_opaque(c, transparent, *dst, *src, width);
        *src = src.add((width + 2 * (i % 2)) as usize);
        *dst = up(*dst, pitch);
    }
}

/// `RenderRightTriangleLowerClipLeftAndVertical`
unsafe fn render_right_triangle_lower_clip_left_and_vertical(c: Ctl, transparent: bool, clip_left: i32, clip_y: &DiamondClipY, dst: &mut *mut u8, pitch: isize, src: &mut *const u8) {
    *src = src.add(calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom));
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        let width = X_STEP * i;
        if width > clip_left {
            render_line_transparent_or_opaque(c, transparent, *dst, src.add(clip_left as usize), width - clip_left);
        }
        *src = src.add((width + 2 * (i % 2)) as usize);
        *dst = up(*dst, pitch);
    }
}

/// `RenderRightTriangleLowerClipRightAndVertical`
unsafe fn render_right_triangle_lower_clip_right_and_vertical(c: Ctl, transparent: bool, clip_right: i32, clip_y: &DiamondClipY, dst: &mut *mut u8, pitch: isize, src: &mut *const u8) {
    *src = src.add(calculate_triangle_source_skip_lower_bottom(clip_y.lower_bottom));
    let lower_max = LOWER_HEIGHT - clip_y.lower_top;
    for i in (1 + clip_y.lower_bottom)..=lower_max {
        let width = X_STEP * i;
        let skip = if WIDTH - width < clip_right { clip_right - (WIDTH - width) } else { 0 };
        if width > skip {
            render_line_transparent_or_opaque(c, transparent, *dst, *src, width - skip);
        }
        *src = src.add((width + 2 * (i % 2)) as usize);
        *dst = up(*dst, pitch);
    }
}

/// `RenderRightTriangle` (all clip variants)
unsafe fn render_right_triangle(c: Ctl, transparent: bool, mut dst: *mut u8, pitch: isize, mut src: *const u8, clip: Clip) {
    if clip.width == WIDTH {
        if clip.height == TRIANGLE_HEIGHT {
            render_right_triangle_lower(c, transparent, &mut dst, pitch, &mut src);
            for i in 1..=TRIANGLE_UPPER_HEIGHT {
                let width = WIDTH - X_STEP * i;
                render_line_transparent_or_opaque(c, transparent, dst, src, width);
                src = src.add((width + 2 * (i % 2)) as usize);
                dst = up(dst, pitch);
            }
        } else {
            let clip_y = calculate_diamond_clip_y(&clip, TRIANGLE_UPPER_HEIGHT);
            render_right_triangle_lower_clip_vertical(c, transparent, &clip_y, &mut dst, pitch, &mut src);
            src = src.add(calculate_triangle_source_skip_upper_bottom(clip_y.upper_bottom));
            let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
            for i in (1 + clip_y.upper_bottom)..=upper_max {
                let width = WIDTH - X_STEP * i;
                render_line_transparent_or_opaque(c, transparent, dst, src, width);
                src = src.add((width + 2 * (i % 2)) as usize);
                dst = up(dst, pitch);
            }
        }
    } else if clip.right == 0 {
        let clip_y = calculate_diamond_clip_y(&clip, TRIANGLE_UPPER_HEIGHT);
        let clip_left = clip.left;
        render_right_triangle_lower_clip_left_and_vertical(c, transparent, clip_left, &clip_y, &mut dst, pitch, &mut src);
        src = src.add(calculate_triangle_source_skip_upper_bottom(clip_y.upper_bottom));
        let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
        for i in (1 + clip_y.upper_bottom)..=upper_max {
            let width = WIDTH - X_STEP * i;
            if width <= clip_left {
                break;
            }
            render_line_transparent_or_opaque(c, transparent, dst, src.add(clip_left as usize), width - clip_left);
            src = src.add((width + 2 * (i % 2)) as usize);
            dst = up(dst, pitch);
        }
    } else {
        let clip_y = calculate_diamond_clip_y(&clip, TRIANGLE_UPPER_HEIGHT);
        let clip_right = clip.right;
        render_right_triangle_lower_clip_right_and_vertical(c, transparent, clip_right, &clip_y, &mut dst, pitch, &mut src);
        src = src.add(calculate_triangle_source_skip_upper_bottom(clip_y.upper_bottom));
        let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
        for i in (1 + clip_y.upper_bottom)..=upper_max {
            let width = WIDTH - X_STEP * i;
            let skip = if WIDTH - width < clip_right { clip_right - (WIDTH - width) } else { 0 };
            render_line_transparent_or_opaque(c, transparent, dst, src, if width > skip { width - skip } else { 0 });
            src = src.add((width + 2 * (i % 2)) as usize);
            dst = up(dst, pitch);
        }
    }
}

/// `RenderTrapezoidUpperHalf`
unsafe fn render_trapezoid_upper_half(c: Ctl, opaque_prefix: bool, prefix_increment: i8, mut dst: *mut u8, pitch: isize, mut src: *const u8) {
    let src_end = src.add((WIDTH * TRAPEZOID_UPPER_HEIGHT) as usize);
    if prefix_increment != 0 {
        // The first and the last line are always fully transparent/opaque (or vice-versa).
        let first_line_is_transparent = opaque_prefix ^ (prefix_increment < 0);
        if first_line_is_transparent {
            if !skip_transparent_pixels(opaque_prefix, prefix_increment) {
                render_line_transparent(c, dst, src, WIDTH as usize);
            }
        } else {
            render_line_opaque(c, dst, src, WIDTH as usize);
        }
        src = src.add(WIDTH as usize);
        dst = up(dst, pitch);
        let mut prefix_width: u8 = (if prefix_increment < 0 { 32u8 } else { 0u8 }).wrapping_add(prefix_increment as u8);
        loop {
            render_line_transparent_and_opaque(c, opaque_prefix, prefix_increment, dst, src, prefix_width as i32, WIDTH);
            prefix_width = prefix_width.wrapping_add(prefix_increment as u8);
            src = src.add(WIDTH as usize);
            dst = up(dst, pitch);
            if src == src_end {
                break;
            }
        }
    } else {
        loop {
            render_line_transparent_or_opaque(c, opaque_prefix, dst, src, WIDTH);
            src = src.add(WIDTH as usize);
            dst = up(dst, pitch);
            if src == src_end {
                break;
            }
        }
    }
}

/// `RenderTrapezoidUpperHalfClipVertical` / `ClipLeftAndVertical` / `ClipRightAndVertical`
unsafe fn render_trapezoid_upper_half_clipped(c: Ctl, opaque_prefix: bool, prefix_increment: i8, clip: &Clip, clip_y: &DiamondClipY, mut dst: *mut u8, pitch: isize, mut src: *const u8, width: i32, prefix_offset: i32) {
    let upper_max = TRAPEZOID_UPPER_HEIGHT - clip_y.upper_top;
    let mut prefix = init_prefix_y(prefix_increment, clip.bottom as i8);
    for _ in (1 + clip_y.upper_bottom)..=upper_max {
        render_line(c, opaque_prefix, prefix_increment, dst, src, width, prefix.wrapping_sub(prefix_offset as i8));
        src = src.add(WIDTH as usize);
        dst = up(dst, pitch);
        prefix = prefix.wrapping_add(prefix_increment);
    }
}

/// `RenderLeftTrapezoid` (all clip variants)
unsafe fn render_left_trapezoid(c: Ctl, opaque_prefix: bool, prefix_increment: i8, mut dst: *mut u8, pitch: isize, mut src: *const u8, clip: Clip) {
    let lower_transparent = lower_half_transparent(opaque_prefix, prefix_increment);
    if clip.width == WIDTH {
        if clip.height == HEIGHT {
            render_left_triangle_lower(c, lower_transparent, &mut dst, pitch, &mut src);
            dst = dst.wrapping_offset(X_STEP as isize);
            render_trapezoid_upper_half(c, opaque_prefix, prefix_increment, dst, pitch, src);
        } else {
            let clip_y = calculate_diamond_clip_y(&clip, TRAPEZOID_UPPER_HEIGHT);
            render_left_triangle_lower_clip_vertical(c, lower_transparent, &clip_y, &mut dst, pitch, &mut src);
            src = src.add((clip_y.upper_bottom * WIDTH) as usize);
            dst = dst.wrapping_offset(X_STEP as isize);
            render_trapezoid_upper_half_clipped(c, opaque_prefix, prefix_increment, &clip, &clip_y, dst, pitch, src, WIDTH, 0);
        }
    } else if clip.right == 0 {
        let clip_y = calculate_diamond_clip_y(&clip, TRAPEZOID_UPPER_HEIGHT);
        render_left_triangle_lower_clip_left_and_vertical(c, lower_transparent, clip.left, &clip_y, &mut dst, pitch, &mut src);
        src = src.add((clip_y.upper_bottom * WIDTH + clip.left) as usize);
        dst = dst.wrapping_offset((X_STEP + clip.left) as isize);
        render_trapezoid_upper_half_clipped(c, opaque_prefix, prefix_increment, &clip, &clip_y, dst, pitch, src, clip.width, clip.left);
    } else {
        let clip_y = calculate_diamond_clip_y(&clip, TRAPEZOID_UPPER_HEIGHT);
        render_left_triangle_lower_clip_right_and_vertical(c, lower_transparent, clip.right, &clip_y, &mut dst, pitch, &mut src);
        src = src.add((clip_y.upper_bottom * WIDTH) as usize);
        dst = dst.wrapping_offset(X_STEP as isize);
        render_trapezoid_upper_half_clipped(c, opaque_prefix, prefix_increment, &clip, &clip_y, dst, pitch, src, clip.width, 0);
    }
}

/// `RenderRightTrapezoid` (all clip variants)
unsafe fn render_right_trapezoid(c: Ctl, opaque_prefix: bool, prefix_increment: i8, mut dst: *mut u8, pitch: isize, mut src: *const u8, clip: Clip) {
    let lower_transparent = lower_half_transparent(opaque_prefix, prefix_increment);
    if clip.width == WIDTH {
        if clip.height == HEIGHT {
            render_right_triangle_lower(c, lower_transparent, &mut dst, pitch, &mut src);
            render_trapezoid_upper_half(c, opaque_prefix, prefix_increment, dst, pitch, src);
        } else {
            let clip_y = calculate_diamond_clip_y(&clip, TRAPEZOID_UPPER_HEIGHT);
            render_right_triangle_lower_clip_vertical(c, lower_transparent, &clip_y, &mut dst, pitch, &mut src);
            src = src.add((clip_y.upper_bottom * WIDTH) as usize);
            render_trapezoid_upper_half_clipped(c, opaque_prefix, prefix_increment, &clip, &clip_y, dst, pitch, src, WIDTH, 0);
        }
    } else if clip.right == 0 {
        let clip_y = calculate_diamond_clip_y(&clip, TRAPEZOID_UPPER_HEIGHT);
        render_right_triangle_lower_clip_left_and_vertical(c, lower_transparent, clip.left, &clip_y, &mut dst, pitch, &mut src);
        src = src.add((clip_y.upper_bottom * WIDTH + clip.left) as usize);
        render_trapezoid_upper_half_clipped(c, opaque_prefix, prefix_increment, &clip, &clip_y, dst, pitch, src, clip.width, clip.left);
    } else {
        let clip_y = calculate_diamond_clip_y(&clip, TRAPEZOID_UPPER_HEIGHT);
        render_right_triangle_lower_clip_right_and_vertical(c, lower_transparent, clip.right, &clip_y, &mut dst, pitch, &mut src);
        src = src.add((clip_y.upper_bottom * WIDTH) as usize);
        render_trapezoid_upper_half_clipped(c, opaque_prefix, prefix_increment, &clip, &clip_y, dst, pitch, src, clip.width, 0);
    }
}

/// `RenderTileType<Light, Transparent>`
unsafe fn render_tile_type(c: Ctl, transparent: bool, tile: TileType, dst: *mut u8, pitch: isize, src: *const u8, clip: Clip) {
    match tile {
        TileType::Square => render_square(c, transparent, dst, pitch, src, clip),
        TileType::TransparentSquare => render_transparent_square(c, transparent, 0, dst, pitch, src, clip),
        TileType::LeftTriangle => render_left_triangle(c, transparent, dst, pitch, src, clip),
        TileType::RightTriangle => render_right_triangle(c, transparent, dst, pitch, src, clip),
        TileType::LeftTrapezoid => render_left_trapezoid(c, transparent, 0, dst, pitch, src, clip),
        TileType::RightTrapezoid => render_right_trapezoid(c, transparent, 0, dst, pitch, src, clip),
    }
}

/// Original: `devilution::RenderTile` (engine/render/dun_render.cpp). `tbl` is
/// `LightTables[light_index]`; the original picks the blitter by comparing it with the fully
/// dark and fully lit tables.
// @port engine/render/dun_render.cpp|devilution::RenderTile(const Surface &out, Point position, LevelCelBlock levelCelBlock, MaskType maskType, const uint8_t *tbl) sha=6e3cce83055c
pub fn render_tile(ctx: &Ctx, out: &Surface, position: Point, level_cel_block: LevelCelBlock, mask_type: MaskType, light_index: usize) {
    let tile = level_cel_block.type_();
    let clip = calculate_clip(position.x, position.y, WIDTH, get_tile_height(tile), out);
    if clip.width <= 0 || clip.height <= 0 {
        return;
    }
    let cels = ctx.gendung.pDungeonCels.as_ref().expect("pDungeonCels");
    let frame = level_cel_block.frame() as usize;
    let off = u32::from_le_bytes([cels[frame * 4], cels[frame * 4 + 1], cels[frame * 4 + 2], cels[frame * 4 + 3]]) as usize;
    let src = cels[off..].as_ptr();
    let dst = out.at(position.x + clip.left, position.y - clip.bottom);
    let pitch = out.pitch() as isize;
    let tables = &ctx.lighting.LightTables;
    let light = if light_index == LightsMax as usize {
        LightType::FullyDark
    } else if light_index == 0 {
        LightType::FullyLit
    } else {
        LightType::PartiallyLit
    };
    let c = Ctl { light, tbl: &tables[light_index], lut: &ctx.dx.pal.palette_transparency_lookup };
    // SAFETY: the clip keeps every write inside `out`, and the CEL frame data holds every byte
    // the tile type reads (as in the original).
    unsafe {
        match mask_type {
            MaskType::Solid => render_tile_type(c, false, tile, dst, pitch, src, clip),
            MaskType::Transparent => render_tile_type(c, true, tile, dst, pitch, src, clip),
            MaskType::Left => match tile {
                TileType::TransparentSquare => render_transparent_square(c, false, 2, dst, pitch, src, clip),
                TileType::LeftTrapezoid => render_left_trapezoid(c, false, 2, dst, pitch, src, clip),
                _ => panic!("Given mask can only be applied to TransparentSquare or LeftTrapezoid tiles"),
            },
            MaskType::Right => match tile {
                TileType::TransparentSquare => render_transparent_square(c, true, -2, dst, pitch, src, clip),
                TileType::RightTrapezoid => render_right_trapezoid(c, true, -2, dst, pitch, src, clip),
                _ => panic!("Given mask can only be applied to TransparentSquare or LeftTrapezoid tiles"),
            },
            MaskType::LeftFoliage => render_transparent_square(c, true, 2, dst, pitch, src, clip),
            MaskType::RightFoliage => render_transparent_square(c, false, -2, dst, pitch, src, clip),
        }
    }
}

/// `memset(dst, 0, n)` for the black tile.
#[inline(always)]
unsafe fn zero(dst: *mut u8, n: i32) {
    if n > 0 {
        std::ptr::write_bytes(dst, 0, n as usize);
    }
}

/// Original: `devilution::world_draw_black_tile` (engine/render/dun_render.cpp).
// @port engine/render/dun_render.cpp|devilution::world_draw_black_tile(const Surface &out, int sx, int sy) sha=b4ed31c0ba10
pub fn world_draw_black_tile(out: &Surface, sx: i32, sy: i32) {
    let clip = calculate_clip(sx, sy, TILE_WIDTH, TRIANGLE_HEIGHT, out);
    if clip.width <= 0 || clip.height <= 0 {
        return;
    }
    let clip_y = calculate_diamond_clip_y(&clip, TRIANGLE_UPPER_HEIGHT);
    let mut dst = out.at(sx, sy - clip.bottom);
    let pitch = out.pitch() as isize;
    // SAFETY: as in the original, the clip keeps the writes inside `out`.
    unsafe {
        if clip.width == TILE_WIDTH {
            if clip.height == TRIANGLE_HEIGHT {
                // RenderBlackTileFull
                dst = dst.wrapping_offset((X_STEP * (LOWER_HEIGHT - 1)) as isize);
                for i in 1..=LOWER_HEIGHT {
                    zero(dst, 2 * X_STEP * i);
                    dst = up(dst, pitch + X_STEP as isize);
                }
                dst = dst.wrapping_offset((2 * X_STEP) as isize);
                for i in 1..=TRIANGLE_UPPER_HEIGHT {
                    zero(dst, TILE_WIDTH - 2 * X_STEP * i);
                    dst = up(dst, pitch - X_STEP as isize);
                }
            } else {
                // RenderBlackTileClipY
                dst = dst.wrapping_offset((X_STEP * (LOWER_HEIGHT - clip_y.lower_bottom - 1)) as isize);
                let lower_max = LOWER_HEIGHT - clip_y.lower_top;
                for i in (1 + clip_y.lower_bottom)..=lower_max {
                    zero(dst, 2 * X_STEP * i);
                    dst = up(dst, pitch + X_STEP as isize);
                }
                dst = dst.wrapping_offset((2 * X_STEP + X_STEP * clip_y.upper_bottom) as isize);
                let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
                for i in (1 + clip_y.upper_bottom)..=upper_max {
                    zero(dst, TILE_WIDTH - 2 * X_STEP * i);
                    dst = up(dst, pitch - X_STEP as isize);
                }
            }
        } else if clip.right == 0 {
            // RenderBlackTileClipLeftAndVertical
            dst = dst.wrapping_offset((X_STEP * (LOWER_HEIGHT - clip_y.lower_bottom - 1)) as isize);
            let lower_max = LOWER_HEIGHT - clip_y.lower_top;
            for i in (clip_y.lower_bottom + 1)..=lower_max {
                let w = 2 * X_STEP * i;
                let cur_x = sx + TILE_WIDTH / 2 - X_STEP * i;
                if cur_x >= 0 {
                    zero(dst, w);
                } else if -cur_x <= w {
                    zero(dst.wrapping_offset(-cur_x as isize), w + cur_x);
                }
                dst = up(dst, pitch + X_STEP as isize);
            }
            dst = dst.wrapping_offset((2 * X_STEP + X_STEP * clip_y.upper_bottom) as isize);
            let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
            for i in clip_y.upper_bottom..upper_max {
                let w = 2 * X_STEP * (TRIANGLE_UPPER_HEIGHT - i);
                let cur_x = sx + TILE_WIDTH / 2 - X_STEP * (TRIANGLE_UPPER_HEIGHT - i);
                if cur_x >= 0 {
                    zero(dst, w);
                } else if -cur_x <= w {
                    zero(dst.wrapping_offset(-cur_x as isize), w + cur_x);
                } else {
                    break;
                }
                dst = up(dst, pitch - X_STEP as isize);
            }
        } else {
            // RenderBlackTileClipRightAndVertical
            let max_width = clip.width;
            dst = dst.wrapping_offset((X_STEP * (LOWER_HEIGHT - clip_y.lower_bottom - 1)) as isize);
            let lower_max = LOWER_HEIGHT - clip_y.lower_top;
            for i in (clip_y.lower_bottom + 1)..=lower_max {
                let width = 2 * X_STEP * i;
                let end_x = TILE_WIDTH / 2 + X_STEP * i;
                let skip = if end_x > max_width { end_x - max_width } else { 0 };
                if width > skip {
                    zero(dst, width - skip);
                }
                dst = up(dst, pitch + X_STEP as isize);
            }
            dst = dst.wrapping_offset((2 * X_STEP + X_STEP * clip_y.upper_bottom) as isize);
            let upper_max = TRIANGLE_UPPER_HEIGHT - clip_y.upper_top;
            for i in (1 + clip_y.upper_bottom)..=upper_max {
                let width = TILE_WIDTH - 2 * X_STEP * i;
                let end_x = TILE_WIDTH / 2 + X_STEP * (TRIANGLE_UPPER_HEIGHT - i + 1);
                let skip = if end_x > max_width { end_x - max_width } else { 0 };
                if width <= skip {
                    break;
                }
                zero(dst, width - skip);
                dst = up(dst, pitch - X_STEP as isize);
            }
        }
    }
}
