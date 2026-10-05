//! `Source/engine/render/clx_render.cpp` + `blit_impl.hpp` + `utils/clx_decode.hpp`: drawing
//! CLX sprites (RLE, rows stored bottom-up, transparent runs may cross rows).
//!
//! Translated with raw pointers, step for step with the original, because the clipping
//! logic depends on exact pointer arithmetic (overruns across rows, negative remainders).

#![allow(clippy::too_many_arguments)]

use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList, ClxSpriteSheet};
use crate::engine::surface::Surface;

pub type Lut = [[u8; 256]; 256];

#[derive(Clone, Copy, PartialEq, Eq)]
enum BlitType {
    Transparent,
    Pixels,
    Fill,
}

#[derive(Clone, Copy)]
struct BlitCommand {
    type_: BlitType,
    /// pointer past the end of the command
    src_end: *const u8,
    length: u32,
    color: u8,
}

/// `IsClxOpaque` (utils/clx_decode.hpp)
// @port utils/clx_decode.hpp|devilution::IsClxOpaque(uint8_t control) sha=9a4fbd50d028
#[inline]
pub fn is_clx_opaque(control: u8) -> bool {
    control >= 0x80
}

/// `GetClxOpaquePixelsWidth` (utils/clx_decode.hpp)
// @port utils/clx_decode.hpp|devilution::GetClxOpaquePixelsWidth(uint8_t control) sha=71b4912196f7
#[inline]
pub fn get_clx_opaque_pixels_width(control: u8) -> u8 {
    (control as i8).wrapping_neg() as u8
}

/// `IsClxOpaqueFill` (utils/clx_decode.hpp)
// @port utils/clx_decode.hpp|devilution::IsClxOpaqueFill(uint8_t control) sha=435095ac9f24
#[inline]
pub fn is_clx_opaque_fill(control: u8) -> bool {
    control <= 0xBE
}

/// `GetClxOpaqueFillWidth` (utils/clx_decode.hpp)
// @port utils/clx_decode.hpp|devilution::GetClxOpaqueFillWidth(uint8_t control) sha=ed0dc94c23d9
#[inline]
pub fn get_clx_opaque_fill_width(control: u8) -> u8 {
    0xBF - control
}

/// `ClxGetBlitCommand` (utils/clx_decode.hpp)
// @port utils/clx_decode.hpp|devilution::ClxGetBlitCommand(const uint8_t *src) sha=e9e259b63518
#[inline]
unsafe fn clx_get_blit_command(src: *const u8) -> BlitCommand {
    unsafe {
        let control = *src;
        let src = src.add(1);
        if !is_clx_opaque(control) {
            return BlitCommand { type_: BlitType::Transparent, src_end: src, length: control as u32, color: 0 };
        }
        if is_clx_opaque_fill(control) {
            let width = get_clx_opaque_fill_width(control);
            let color = *src;
            return BlitCommand { type_: BlitType::Fill, src_end: src.add(1), length: width as u32, color };
        }
        let width = get_clx_opaque_pixels_width(control);
        BlitCommand { type_: BlitType::Pixels, src_end: src.add(width as usize), length: width as u32, color: 0 }
    }
}

/// The blit functors of blit_impl.hpp.
#[derive(Clone, Copy)]
enum Blit<'a> {
    /// `BlitDirect`
    Direct,
    /// `BlitWithMap`
    WithMap(&'a [u8; 256]),
    /// `BlitBlendedWithMap`
    BlendedWithMap(&'a [u8; 256], &'a Lut),
}

impl Blit<'_> {
    #[inline]
    unsafe fn apply(&self, cmd: BlitCommand, dst: *mut u8, src: *const u8) {
        unsafe {
            match (*self, cmd.type_) {
                (_, BlitType::Transparent) => {}
                // BlitFillDirect / BlitPixelsDirect
                (Blit::Direct, BlitType::Fill) => std::ptr::write_bytes(dst, cmd.color, cmd.length as usize),
                (Blit::Direct, BlitType::Pixels) => std::ptr::copy_nonoverlapping(src, dst, cmd.length as usize),
                // BlitFillWithMap / BlitPixelsWithMap
                (Blit::WithMap(map), BlitType::Fill) => std::ptr::write_bytes(dst, map[cmd.color as usize], cmd.length as usize),
                (Blit::WithMap(map), BlitType::Pixels) => {
                    for i in 0..cmd.length as usize {
                        *dst.add(i) = map[*src.add(i) as usize];
                    }
                }
                // BlitFillBlended(colorMap[color]) / BlitPixelsBlendedWithMap
                (Blit::BlendedWithMap(map, lut), BlitType::Fill) => {
                    let tbl = &lut[map[cmd.color as usize] as usize];
                    for i in 0..cmd.length as usize {
                        *dst.add(i) = tbl[*dst.add(i) as usize];
                    }
                }
                (Blit::BlendedWithMap(map, lut), BlitType::Pixels) => {
                    for i in 0..cmd.length as usize {
                        let d = dst.add(i);
                        *d = lut[*d as usize][map[*src.add(i) as usize] as usize];
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
struct ClipX {
    left: i32,
    right: i32,
    width: i32,
}

/// `CalculateClipX`
// @port engine/render/clx_render.cpp|devilution::CalculateClipX(int_fast16_t x, std::size_t w, const Surface &out) sha=ae7f0b8e1708
fn calculate_clip_x(x: i32, w: i32, out: &Surface) -> ClipX {
    let left = if x < 0 { -x } else { 0 };
    let right = if x + w > out.w() { x + w - out.w() } else { 0 };
    ClipX { left, right, width: w - left - right }
}

#[derive(Clone, Copy)]
struct RenderSrc {
    begin: *const u8,
    end: *const u8,
    width: i32,
}

#[derive(Clone, Copy, Default)]
struct SkipSize {
    whole_lines: i32,
    x_offset: i32,
}

/// `GetSkipSize`
// @port engine/render/clx_render.cpp|devilution::GetSkipSize(int_fast16_t remainingWidth, int_fast16_t srcWidth) sha=3460f19a6bfb
fn get_skip_size(remaining_width: i32, src_width: i32) -> SkipSize {
    if remaining_width < 0 {
        let overrun_lines = -remaining_width / src_width;
        return SkipSize { whole_lines: 1 + overrun_lines, x_offset: -remaining_width - src_width * overrun_lines };
    }
    SkipSize { whole_lines: 1, x_offset: 0 }
}

/// `SkipRestOfLineWithOverrun`
// @port engine/render/clx_render.cpp|devilution::SkipRestOfLineWithOverrun(const uint8_t *src, int_fast16_t srcWidth, SkipSize &skipSize) sha=b1322d455705
unsafe fn skip_rest_of_line_with_overrun(src: *const u8, src_width: i32, skip_size: &mut SkipSize) -> *const u8 {
    let mut src = src;
    let mut remaining_width = src_width - skip_size.x_offset;
    while remaining_width > 0 {
        let cmd = unsafe { clx_get_blit_command(src) };
        src = cmd.src_end;
        remaining_width -= cmd.length as i32;
    }
    *skip_size = get_skip_size(remaining_width, src_width);
    src
}

/// `SkipLinesForRenderBackwardsWithOverrun`: returns the horizontal overrun.
// @port engine/render/clx_render.cpp|devilution::SkipLinesForRenderBackwardsWithOverrun(Point &position, RenderSrc &src, int_fast16_t dstHeight) sha=5aca13847b34
unsafe fn skip_lines_for_render_backwards_with_overrun(position: &mut (i32, i32), src: &mut RenderSrc, dst_height: i32) -> i32 {
    let mut skip_size = SkipSize::default();
    while position.1 >= dst_height && src.begin != src.end {
        src.begin = unsafe { skip_rest_of_line_with_overrun(src.begin, src.width, &mut skip_size) };
        position.1 -= skip_size.whole_lines;
    }
    skip_size.x_offset
}

/// `DoRenderBackwardsClipY`
// @port engine/render/clx_render.cpp|devilution::DoRenderBackwardsClipY(const Surface &out, Point position, RenderSrc src, BlitFn &&blitFn) sha=3fc2d8647ea8
unsafe fn do_render_backwards_clip_y(out: &Surface, mut position: (i32, i32), mut src: RenderSrc, blit: Blit) {
    unsafe {
        let mut x_offset = skip_lines_for_render_backwards_with_overrun(&mut position, &mut src, out.h());
        if src.begin >= src.end {
            return;
        }
        let mut dst = out.at(position.0, position.1);
        let dst_begin = out.at(0, 0);
        let dst_pitch = out.pitch() as isize;
        while src.begin != src.end && dst >= dst_begin {
            let mut remaining_width = src.width - x_offset;
            dst = dst.offset(x_offset as isize);
            while remaining_width > 0 {
                let cmd = clx_get_blit_command(src.begin);
                blit.apply(cmd, dst, src.begin.add(1));
                src.begin = cmd.src_end;
                dst = dst.add(cmd.length as usize);
                remaining_width -= cmd.length as i32;
            }
            let skip_size = get_skip_size(remaining_width, src.width);
            x_offset = skip_size.x_offset;
            dst = dst.offset(-(skip_size.whole_lines as isize * dst_pitch + (src.width - remaining_width) as isize));
        }
    }
}

/// `DoRenderBackwardsClipXY`
// @port engine/render/clx_render.cpp|devilution::DoRenderBackwardsClipXY(const Surface &out, Point position, RenderSrc src, ClipX clipX, BlitFn &&blitFn) sha=848c369ced9f
unsafe fn do_render_backwards_clip_xy(out: &Surface, mut position: (i32, i32), mut src: RenderSrc, clip_x: ClipX, blit: Blit) {
    unsafe {
        let mut x_offset = skip_lines_for_render_backwards_with_overrun(&mut position, &mut src, out.h());
        if src.begin >= src.end {
            return;
        }
        position.0 += clip_x.left;
        let mut dst = out.at(position.0, position.1);
        let dst_begin = out.at(0, 0);
        let dst_pitch = out.pitch() as isize;
        while src.begin != src.end && dst >= dst_begin {
            let mut remaining_width = clip_x.width;
            let mut remaining_left_clip = clip_x.left - x_offset;
            if remaining_left_clip < 0 {
                dst = dst.add((remaining_width as u32).min((-remaining_left_clip) as u32) as usize);
                remaining_width += remaining_left_clip;
            }
            while remaining_left_clip > 0 {
                let mut cmd = clx_get_blit_command(src.begin);
                if cmd.length as i32 > remaining_left_clip {
                    let overshoot = cmd.length as i32 - remaining_left_clip;
                    cmd.length = (remaining_width as u32).min(overshoot as u32);
                    blit.apply(cmd, dst, src.begin.add(1 + remaining_left_clip as usize));
                    dst = dst.add(cmd.length as usize);
                    remaining_width -= overshoot;
                    src.begin = cmd.src_end;
                    break;
                }
                src.begin = cmd.src_end;
                remaining_left_clip -= cmd.length as i32;
            }
            while remaining_width > 0 {
                let mut cmd = clx_get_blit_command(src.begin);
                let unclipped_length = cmd.length;
                cmd.length = (remaining_width as u32).min(cmd.length);
                blit.apply(cmd, dst, src.begin.add(1));
                src.begin = cmd.src_end;
                dst = dst.add(cmd.length as usize);
                remaining_width -= unclipped_length as i32; // can be negative
            }
            remaining_width += clip_x.right;
            let mut skip_size;
            if remaining_width > 0 {
                skip_size = SkipSize { whole_lines: 0, x_offset: src.width - remaining_width };
                src.begin = skip_rest_of_line_with_overrun(src.begin, src.width, &mut skip_size);
            } else {
                skip_size = get_skip_size(remaining_width, src.width);
            }
            x_offset = skip_size.x_offset;
            dst = dst.offset(-(dst_pitch * skip_size.whole_lines as isize + clip_x.width as isize));
        }
    }
}

/// `DoRenderBackwards`
// @port engine/render/clx_render.cpp|devilution::DoRenderBackwards(const Surface &out, Point position, const uint8_t *src, size_t srcSize, unsigned srcWidth, unsigned srcHeight, BlitFn &&blitFn) sha=ccf884f06cbb
fn do_render_backwards(out: &Surface, position: (i32, i32), src: &[u8], src_width: i32, src_height: i32, blit: Blit) {
    if position.1 < 0 || position.1 + 1 >= out.h() + src_height {
        return;
    }
    let clip_x = calculate_clip_x(position.0, src_width, out);
    if clip_x.width <= 0 {
        return;
    }
    let range = src.as_ptr_range();
    let rs = RenderSrc { begin: range.start, end: range.end, width: src_width };
    // SAFETY: the sprite data is well-formed CLX and the destination is clipped to `out`.
    unsafe {
        if clip_x.width == src_width {
            do_render_backwards_clip_y(out, position, rs, blit);
        } else {
            do_render_backwards_clip_xy(out, position, rs, clip_x, blit);
        }
    }
}

/// Original: `devilution::ClxDraw` (engine/render/clx_render.cpp). `position` is the
/// bottom-left corner.
// @port engine/render/clx_render.cpp|devilution::ClxDraw(const Surface &out, Point position, ClxSprite clx) sha=02af0c6b3ff4
pub fn clx_draw(out: &Surface, position: (i32, i32), clx: &ClxSprite) {
    do_render_backwards(out, position, clx.pixel_data(), clx.width() as i32, clx.height() as i32, Blit::Direct);
}

/// Original: `devilution::ClxDrawTRN` (engine/render/clx_render.cpp).
// @port engine/render/clx_render.cpp|devilution::ClxDrawTRN(const Surface &out, Point position, ClxSprite clx, const uint8_t *trn) sha=7cc89548edba
pub fn clx_draw_trn(out: &Surface, position: (i32, i32), clx: &ClxSprite, trn: &[u8; 256]) {
    do_render_backwards(out, position, clx.pixel_data(), clx.width() as i32, clx.height() as i32, Blit::WithMap(trn));
}

/// Original: `devilution::ClxDrawBlendedTRN` (engine/render/clx_render.cpp).
// @port engine/render/clx_render.cpp|devilution::ClxDrawBlendedTRN(const Surface &out, Point position, ClxSprite clx, const uint8_t *trn) sha=551d9959b350
pub fn clx_draw_blended_trn(out: &Surface, position: (i32, i32), clx: &ClxSprite, trn: &[u8; 256], lut: &Lut) {
    do_render_backwards(out, position, clx.pixel_data(), clx.width() as i32, clx.height() as i32, Blit::BlendedWithMap(trn, lut));
}

/// `ClxApplyTrans(ClxSprite, trn)`: remaps the colour indices of the sprite's pixel commands.
// @port engine/render/clx_render.cpp|devilution::ClxApplyTrans(ClxSprite sprite, const uint8_t *trn) sha=28d0c9c8cbfb
fn clx_apply_trans_pixels(data: &mut [u8], trn: &[u8; 256]) {
    let mut i = 0usize;
    let mut remaining = data.len() as u16;
    while remaining != 0 {
        let val = data[i];
        i += 1;
        remaining = remaining.wrapping_sub(1);
        if !is_clx_opaque(val) {
            continue;
        }
        if is_clx_opaque_fill(val) {
            remaining = remaining.wrapping_sub(1);
            data[i] = trn[data[i] as usize];
            i += 1;
        } else {
            let w = get_clx_opaque_pixels_width(val);
            remaining = remaining.wrapping_sub(w as u16);
            for _ in 0..w {
                data[i] = trn[data[i] as usize];
                i += 1;
            }
        }
    }
}

/// Original: `devilution::ClxApplyTrans(ClxSpriteList, const uint8_t *trn)` (engine/render/clx_render.cpp).
// @port engine/render/clx_render.cpp|devilution::ClxApplyTrans(ClxSpriteList list, const uint8_t *trn) sha=36dd4e14fc0a
pub fn clx_apply_trans_list(list: &mut ClxSpriteList, trn: &[u8; 256]) {
    let ranges: Vec<(usize, usize)> = (0..list.num_sprites() as usize)
        .map(|i| {
            let (b, e) = (list.sprite_offset(i) as usize, list.sprite_offset(i + 1) as usize);
            (b + 10, e)
        })
        .collect();
    let data = list.data_mut();
    for (b, e) in ranges {
        clx_apply_trans_pixels(&mut data[b..e], trn);
    }
}

/// Original: `devilution::ClxApplyTrans(ClxSpriteSheet, const uint8_t *trn)` (engine/render/clx_render.cpp).
// @port engine/render/clx_render.cpp|devilution::ClxApplyTrans(ClxSpriteSheet sheet, const uint8_t *trn) sha=e6bd3fc8c9c4
pub fn clx_apply_trans_sheet(sheet: &mut ClxSpriteSheet, trn: &[u8; 256]) {
    let mut ranges = Vec::new();
    for l in 0..sheet.num_lists() as usize {
        let list = sheet.get(l);
        let base = sheet.sheet_offset(l) as usize;
        for i in 0..list.num_sprites() as usize {
            ranges.push((base + list.sprite_offset(i) as usize + 10, base + list.sprite_offset(i + 1) as usize));
        }
    }
    let data = sheet.data_mut();
    for (b, e) in ranges {
        clx_apply_trans_pixels(&mut data[b..e], trn);
    }
}

/// Original: `devilution::ClxMeasureSolidHorizontalBounds` (engine/render/clx_render.cpp).
// @port engine/render/clx_render.cpp|devilution::ClxMeasureSolidHorizontalBounds(ClxSprite clx) sha=40b875a6211c
pub fn clx_measure_solid_horizontal_bounds(clx: &ClxSprite) -> (i32, i32) {
    let src = clx.pixel_data();
    let width = clx.width() as i32;
    let (mut x_begin, mut x_end, mut x_cur) = (width, 0, 0);
    let mut i = 0usize;
    while i < src.len() {
        while x_cur < width {
            let mut val = src[i];
            i += 1;
            if !is_clx_opaque(val) {
                x_cur += val as i32;
                continue;
            }
            if is_clx_opaque_fill(val) {
                val = get_clx_opaque_fill_width(val);
                i += 1;
            } else {
                val = get_clx_opaque_pixels_width(val);
                i += val as usize;
            }
            x_begin = x_begin.min(x_cur);
            x_cur += val as i32;
            x_end = x_end.max(x_cur);
        }
        while x_cur >= width {
            x_cur -= width;
        }
        if x_begin == 0 && x_end == width {
            break;
        }
    }
    (x_begin, x_end)
}

// --- Outlines -------------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct Dirs {
    north: bool,
    west: bool,
    south: bool,
    east: bool,
}

const fn dirs(north: bool, west: bool, south: bool, east: bool) -> Dirs {
    Dirs { north, west, south, east }
}

/// `RenderOutlineForPixel(dst, pitch, color)`
// @port engine/render/clx_render.cpp|devilution::RenderOutlineForPixel(uint8_t *dst, int dstPitch, uint8_t color) sha=87a3ec1a4c0f
unsafe fn outline_pixel(d: Dirs, dst: *mut u8, pitch: isize, color: u8) {
    unsafe {
        if d.north {
            *dst.offset(-pitch) = color;
        }
        if d.west {
            *dst.offset(-1) = color;
        }
        if d.east {
            *dst.offset(1) = color;
        }
        if d.south {
            *dst.offset(pitch) = color;
        }
    }
}

/// `RenderOutlineForPixel(dst, pitch, srcColor, color)`
// @port engine/render/clx_render.cpp|devilution::RenderOutlineForPixel(uint8_t *dst, int dstPitch, uint8_t srcColor, uint8_t color) sha=afdf18e6492f
unsafe fn outline_pixel_src(d: Dirs, skip0: bool, dst: *mut u8, pitch: isize, src_color: u8, color: u8) {
    if skip0 && src_color == 0 {
        return;
    }
    unsafe { outline_pixel(d, dst, pitch, color) }
}

/// `RenderOutlineForPixels(dst, pitch, width, color)`
// @port engine/render/clx_render.cpp|devilution::RenderOutlineForPixels(uint8_t *dst, int dstPitch, int width, uint8_t color) sha=b25aabb7c9a4
unsafe fn outline_pixels(d: Dirs, dst: *mut u8, pitch: isize, width: i32, color: u8) {
    if width <= 0 {
        return;
    }
    let w = width as usize;
    unsafe {
        if d.north {
            std::ptr::write_bytes(dst.offset(-pitch), color, w);
        }
        if d.west && d.east {
            std::ptr::write_bytes(dst.offset(-1), color, w + 2);
        } else if d.west {
            std::ptr::write_bytes(dst.offset(-1), color, w);
        } else if d.east {
            std::ptr::write_bytes(dst.offset(1), color, w);
        }
        if d.south {
            std::ptr::write_bytes(dst.offset(pitch), color, w);
        }
    }
}

/// `RenderOutlineForPixels(dst, pitch, width, src, color)`
// @port engine/render/clx_render.cpp|devilution::RenderOutlineForPixels(uint8_t *dst, int dstPitch, int width, const uint8_t *src, uint8_t color) sha=642ae775aa67
unsafe fn outline_pixels_src(d: Dirs, skip0: bool, dst: *mut u8, pitch: isize, width: i32, src: *const u8, color: u8) {
    unsafe {
        if skip0 {
            for i in 0..width.max(0) as usize {
                outline_pixel_src(d, true, dst.add(i), pitch, *src.add(i), color);
            }
        } else {
            outline_pixels(d, dst, pitch, width, color);
        }
    }
}

/// `RenderClxOutlinePixelsCheckFirstColumn`
// @port engine/render/clx_render.cpp|devilution::RenderClxOutlinePixelsCheckFirstColumn(uint8_t *dst, int dstPitch, int dstX, const uint8_t *src, uint8_t width, uint8_t color) sha=a03bbd9f8680
unsafe fn outline_check_first_column(fill: bool, d: Dirs, skip0: bool, mut dst: *mut u8, pitch: isize, dst_x: i32, mut src: *const u8, mut width: u8, color: u8) {
    unsafe {
        if dst_x == -1 {
            let only_east = dirs(false, false, false, d.east);
            if fill {
                outline_pixel(only_east, dst, pitch, color);
            } else {
                outline_pixel_src(only_east, skip0, dst, pitch, *src, color);
                src = src.add(1);
            }
            dst = dst.add(1);
            width = width.wrapping_sub(1);
        }
        if width > 0 {
            let no_west = dirs(d.north, false, d.south, d.east);
            if fill {
                outline_pixel(no_west, dst, pitch, color);
            } else {
                outline_pixel_src(no_west, skip0, dst, pitch, *src, color);
                src = src.add(1);
            }
            dst = dst.add(1);
            width -= 1;
        }
        if width > 0 {
            if fill {
                outline_pixels(d, dst, pitch, width as i32, color);
            } else {
                outline_pixels_src(d, skip0, dst, pitch, width as i32, src, color);
            }
        }
    }
}

/// `RenderClxOutlinePixelsCheckLastColumn`
// @port engine/render/clx_render.cpp|devilution::RenderClxOutlinePixelsCheckLastColumn(uint8_t *dst, int dstPitch, int dstX, int dstW, const uint8_t *src, uint8_t width, uint8_t color) sha=f05283872537
unsafe fn outline_check_last_column(fill: bool, d: Dirs, skip0: bool, mut dst: *mut u8, pitch: isize, dst_x: i32, dst_w: i32, mut src: *const u8, mut width: u8, color: u8) {
    unsafe {
        let last_pixel = dst_x != dst_w;
        let oob_pixel = dst_x + width as i32 == dst_w + 1;
        let num_special = last_pixel as u8 + oob_pixel as u8;
        if width > num_special {
            width -= num_special;
            if fill {
                outline_pixels(d, dst, pitch, width as i32, color);
            } else {
                outline_pixels_src(d, skip0, dst, pitch, width as i32, src, color);
                src = src.add(width as usize);
            }
            dst = dst.add(width as usize);
        }
        if last_pixel {
            let no_east = dirs(d.north, d.west, d.south, false);
            if fill {
                outline_pixel(no_east, dst, pitch, color);
            } else {
                outline_pixel_src(no_east, skip0, dst, pitch, *src, color);
                src = src.add(1);
            }
            dst = dst.add(1);
        }
        if oob_pixel {
            let only_west = dirs(false, d.west, false, false);
            if fill {
                outline_pixel(only_west, dst, pitch, color);
            } else {
                outline_pixel_src(only_west, skip0, dst, pitch, *src, color);
            }
        }
    }
}

/// `RenderClxOutlinePixels`
// @port engine/render/clx_render.cpp|devilution::RenderClxOutlinePixels(uint8_t *dst, int dstPitch, int dstX, int dstW, const uint8_t *src, uint8_t width, uint8_t color) sha=c1fd9bfef02c
unsafe fn outline_render_pixels(
    fill: bool,
    d: Dirs,
    skip0: bool,
    check_first: bool,
    check_last: bool,
    dst: *mut u8,
    pitch: isize,
    dst_x: i32,
    dst_w: i32,
    src: *const u8,
    width: u8,
    color: u8,
) {
    unsafe {
        if skip0 && fill && *src == 0 {
            return;
        }
        if check_first && dst_x <= 0 {
            outline_check_first_column(fill, d, skip0, dst, pitch, dst_x, src, width, color);
        } else if check_last && dst_x + width as i32 >= dst_w {
            outline_check_last_column(fill, d, skip0, dst, pitch, dst_x, dst_w, src, width, color);
        } else if fill {
            outline_pixels(d, dst, pitch, width as i32, color);
        } else {
            outline_pixels_src(d, skip0, dst, pitch, width as i32, src, color);
        }
    }
}

/// `RenderClxOutlineRowClipped`
// @port engine/render/clx_render.cpp|devilution::RenderClxOutlineRowClipped(const Surface &out, Point position, const uint8_t *src, std::size_t srcWidth, ClipX clipX, uint8_t color, SkipSize &skipSize) sha=a6b1a36fb7d2
unsafe fn outline_row_clipped(
    d: Dirs,
    skip0: bool,
    clip_width: bool,
    check_first: bool,
    check_last: bool,
    out: &Surface,
    mut position: (i32, i32),
    mut src: *const u8,
    src_width: i32,
    clip_x: ClipX,
    color: u8,
    skip_size: &mut SkipSize,
) -> *const u8 {
    unsafe {
        let mut remaining_width = clip_x.width;
        let mut dst = out.at(position.0, position.1);
        let pitch = out.pitch() as isize;
        let out_w = out.w();
        // renderPixels lambda
        let render = |fill: bool, w: u8, v: u8, dst: &mut *mut u8, src: &mut *const u8, px: i32| {
            outline_render_pixels(fill, d, skip0, check_first, check_last, *dst, pitch, px, out_w, *src, w, color);
            if fill {
                *src = src.add(1);
            } else {
                *src = src.add(v as usize);
            }
            *dst = dst.add(w as usize);
        };
        if clip_width {
            let mut remaining_left_clip = clip_x.left - skip_size.x_offset;
            if skip_size.x_offset > clip_x.left {
                position.0 += skip_size.x_offset - clip_x.left;
                dst = dst.add((skip_size.x_offset - clip_x.left) as usize);
            }
            while remaining_left_clip > 0 {
                let mut v = *src;
                src = src.add(1);
                if is_clx_opaque(v) {
                    let fill = is_clx_opaque_fill(v);
                    v = if fill { get_clx_opaque_fill_width(v) } else { get_clx_opaque_pixels_width(v) };
                    if v as i32 > remaining_left_clip {
                        let overshoot = (v as i32 - remaining_left_clip) as u8;
                        render(fill, overshoot, v, &mut dst, &mut src, position.0);
                        position.0 += overshoot as i32;
                    } else {
                        src = src.add(if fill { 1 } else { v as usize });
                    }
                } else if v as i32 > remaining_left_clip {
                    let overshoot = (v as i32 - remaining_left_clip) as u8;
                    dst = dst.add(overshoot as usize);
                    position.0 += overshoot as i32;
                }
                remaining_left_clip -= v as i32;
            }
            remaining_width += remaining_left_clip;
        } else {
            position.0 += skip_size.x_offset;
            dst = dst.add(skip_size.x_offset as usize);
            remaining_width -= skip_size.x_offset;
        }
        while remaining_width > 0 {
            let mut v = *src;
            src = src.add(1);
            if is_clx_opaque(v) {
                let fill = is_clx_opaque_fill(v);
                v = if fill { get_clx_opaque_fill_width(v) } else { get_clx_opaque_pixels_width(v) };
                let w = if clip_width { remaining_width.min(v as i32) as u8 } else { v };
                render(fill, w, v, &mut dst, &mut src, position.0);
            } else {
                dst = dst.add(v as usize);
            }
            remaining_width -= v as i32;
            position.0 += v as i32;
        }
        if clip_width {
            remaining_width += clip_x.right;
            if remaining_width > 0 {
                skip_size.x_offset = src_width - remaining_width;
                return skip_rest_of_line_with_overrun(src, src_width, skip_size);
            }
        }
        *skip_size = get_skip_size(remaining_width, src_width);
        src
    }
}

/// `RenderClxOutlineClippedY`
// @port engine/render/clx_render.cpp|devilution::RenderClxOutlineClippedY(const Surface &out, Point position, RenderSrc src, uint8_t color) sha=07a804b5a3fb
unsafe fn outline_clipped_y(skip0: bool, out: &Surface, mut position: (i32, i32), mut src: RenderSrc, color: u8) {
    unsafe {
        let dst_height = out.h();
        let mut skip_size = SkipSize { whole_lines: 0, x_offset: skip_lines_for_render_backwards_with_overrun(&mut position, &mut src, dst_height) };
        if src.begin == src.end {
            return;
        }
        let clip_x = ClipX { left: 0, right: 0, width: src.width };
        let row = |d: Dirs, position: &mut (i32, i32), src: &mut RenderSrc, skip_size: &mut SkipSize| {
            src.begin = outline_row_clipped(d, skip0, false, false, false, out, *position, src.begin, src.width, clip_x, color, skip_size);
            position.1 -= skip_size.whole_lines;
        };
        if position.1 == dst_height {
            row(dirs(true, false, false, false), &mut position, &mut src, &mut skip_size);
        }
        if src.begin == src.end {
            return;
        }
        if position.1 + 1 == dst_height {
            row(dirs(true, true, false, true), &mut position, &mut src, &mut skip_size);
        }
        while position.1 > 0 && src.begin != src.end {
            row(dirs(true, true, true, true), &mut position, &mut src, &mut skip_size);
        }
        if src.begin == src.end {
            return;
        }
        if position.1 == 0 {
            row(dirs(false, true, true, true), &mut position, &mut src, &mut skip_size);
        }
        if src.begin == src.end {
            return;
        }
        if position.1 == -1 {
            outline_row_clipped(dirs(false, false, true, false), skip0, false, false, false, out, position, src.begin, src.width, clip_x, color, &mut skip_size);
        }
    }
}

/// `RenderClxOutlineClippedXY`
// @port engine/render/clx_render.cpp|devilution::RenderClxOutlineClippedXY(const Surface &out, Point position, RenderSrc src, uint8_t color) sha=bf63f414f91f
unsafe fn outline_clipped_xy(skip0: bool, out: &Surface, mut position: (i32, i32), mut src: RenderSrc, color: u8) {
    unsafe {
        let dst_height = out.h();
        let mut skip_size = SkipSize { whole_lines: 0, x_offset: skip_lines_for_render_backwards_with_overrun(&mut position, &mut src, dst_height) };
        if src.begin == src.end {
            return;
        }
        let mut clip_x = calculate_clip_x(position.0, src.width, out);
        if clip_x.width < 0 {
            return;
        }
        if clip_x.left > 0 {
            clip_x.left -= 1;
            clip_x.width += 1;
        } else if clip_x.right > 0 {
            clip_x.right -= 1;
            clip_x.width += 1;
        }
        position.0 += clip_x.left;
        let out_w = out.w();
        // the three column-check variants, chosen by the current x position
        let row = |d: Dirs, position: &mut (i32, i32), src: &mut RenderSrc, skip_size: &mut SkipSize| {
            let (first, last) = if position.0 <= 0 {
                (true, false)
            } else if position.0 + clip_x.width >= out_w {
                (false, true)
            } else {
                (false, false)
            };
            src.begin = outline_row_clipped(d, skip0, true, first, last, out, *position, src.begin, src.width, clip_x, color, skip_size);
            position.1 -= skip_size.whole_lines;
        };
        if position.1 == dst_height {
            row(dirs(true, false, false, false), &mut position, &mut src, &mut skip_size);
        }
        if src.begin == src.end {
            return;
        }
        if position.1 + 1 == dst_height {
            row(dirs(true, true, false, true), &mut position, &mut src, &mut skip_size);
        }
        while position.1 > 0 && src.begin != src.end {
            row(dirs(true, true, true, true), &mut position, &mut src, &mut skip_size);
        }
        if src.begin == src.end {
            return;
        }
        if position.1 == 0 {
            row(dirs(false, true, true, true), &mut position, &mut src, &mut skip_size);
        }
        if src.begin == src.end {
            return;
        }
        if position.1 == -1 {
            row(dirs(false, false, true, false), &mut position, &mut src, &mut skip_size);
        }
    }
}

/// `RenderClxOutline`
// @port engine/render/clx_render.cpp|devilution::RenderClxOutline(const Surface &out, Point position, const uint8_t *src, std::size_t srcSize, std::size_t srcWidth, uint8_t color) sha=5bf71159a9f9
fn render_clx_outline(skip0: bool, out: &Surface, position: (i32, i32), clx: &ClxSprite, color: u8) {
    let data = clx.pixel_data();
    let range = data.as_ptr_range();
    let src = RenderSrc { begin: range.start, end: range.end, width: clx.width() as i32 };
    // SAFETY: well-formed CLX, destination clipped.
    unsafe {
        if position.0 > 0 && position.0 + (clx.width() as i32) < out.w() {
            outline_clipped_y(skip0, out, position, src, color);
        } else {
            outline_clipped_xy(skip0, out, position, src, color);
        }
    }
}

/// Original: `devilution::ClxDrawOutline` (engine/render/clx_render.cpp).
// @port engine/render/clx_render.cpp|devilution::ClxDrawOutline(const Surface &out, uint8_t col, Point position, ClxSprite clx) sha=04ee943e0bd2
pub fn clx_draw_outline(out: &Surface, col: u8, position: (i32, i32), clx: &ClxSprite) {
    render_clx_outline(false, out, position, clx, col);
}

/// Original: `devilution::ClxDrawOutlineSkipColorZero` (engine/render/clx_render.cpp).
// @port engine/render/clx_render.cpp|devilution::ClxDrawOutlineSkipColorZero(const Surface &out, uint8_t col, Point position, ClxSprite clx) sha=888f77696574
pub fn clx_draw_outline_skip_color_zero(out: &Surface, col: u8, position: (i32, i32), clx: &ClxSprite) {
    render_clx_outline(true, out, position, clx, col);
}
