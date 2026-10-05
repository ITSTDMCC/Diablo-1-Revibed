//! `Source/utils/cel_to_clx.cpp` and `utils/cl2_to_clx.cpp`: converting the original CEL and
//! CL2 sprite formats to CLX when loading.

use crate::engine::clx_sprite::ClxSpriteListOrSheet;
use crate::engine::render::clx_render::{get_clx_opaque_fill_width, get_clx_opaque_pixels_width, is_clx_opaque, is_clx_opaque_fill};
use crate::utils::clx_encode::{append_clx_pixels_or_fill_run, append_clx_transparent_run};

/// `PointerOrValue<uint16_t>`: one width for all frames, or one per frame.
#[derive(Clone, Copy, Debug)]
pub enum Widths<'a> {
    Value(u16),
    Pointer(&'a [u16]),
}

impl Widths<'_> {
    fn of_frame(&self, frame: usize) -> u16 {
        match self {
            Widths::Value(w) => *w,
            Widths::Pointer(ws) => ws[frame - 1],
        }
    }
}

fn le16(d: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([d[o], d[o + 1]])
}

fn le32(d: &[u8], o: usize) -> u32 {
    u32::from_le_bytes([d[o], d[o + 1], d[o + 2], d[o + 3]])
}

fn put16(d: &mut [u8], o: usize, v: u16) {
    d[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

fn put32(d: &mut [u8], o: usize, v: u32) {
    d[o..o + 4].copy_from_slice(&v.to_le_bytes());
}

/// `IsCelTransparent` (utils/cel_to_clx.cpp)
// @port utils/cel_to_clx.cpp|devilution::IsCelTransparent(uint8_t control) sha=2cc05417593b
fn is_cel_transparent(control: u8) -> bool {
    control >= 0x80
}

/// `GetCelTransparentWidth` (utils/cel_to_clx.cpp)
// @port utils/cel_to_clx.cpp|devilution::GetCelTransparentWidth(uint8_t control) sha=e17529631002
fn get_cel_transparent_width(control: u8) -> u8 {
    (control as i8).wrapping_neg() as u8
}

/// Original: `devilution::CelToClx` (utils/cel_to_clx.cpp).
// @port utils/cel_to_clx.cpp|devilution::CelToClx(const uint8_t *data, size_t size, PointerOrValue<uint16_t> widthOrWidths) sha=16a769683959
pub fn cel_to_clx(data: &[u8], size: usize, widths: Widths) -> ClxSpriteListOrSheet {
    let mut num_groups = 1u32;
    let maybe_num_frames = le32(data, 0);
    let mut cl2: Vec<u8> = Vec::with_capacity(size + 4445);
    let mut base = 0usize;
    if le32(data, maybe_num_frames as usize * 4 + 4) as usize != size {
        num_groups = maybe_num_frames / 4;
        base = maybe_num_frames as usize;
        cl2.resize(maybe_num_frames as usize, 0);
    }
    for group in 0..num_groups as usize {
        let d = &data[base..];
        let num_frames = if num_groups == 1 {
            maybe_num_frames
        } else {
            let pos = cl2.len() as u32;
            put32(&mut cl2, 4 * group, pos);
            le32(d, 0)
        };
        let cl2_data_offset = cl2.len();
        cl2.resize(cl2.len() + 4 * (2 + num_frames as usize), 0);
        put32(&mut cl2, cl2_data_offset, num_frames);
        let mut src_end = le32(d, 4) as usize;
        for frame in 1..=num_frames as usize {
            let mut src = src_end;
            src_end = le32(d, 4 * (frame + 1)) as usize;
            let rel = (cl2.len() - cl2_data_offset) as u32;
            put32(&mut cl2, cl2_data_offset + 4 * frame, rel);
            const CEL_FRAME_HEADER_SIZE: u16 = 10;
            if le16(d, src) == CEL_FRAME_HEADER_SIZE {
                src += CEL_FRAME_HEADER_SIZE as usize;
            }
            let frame_width = widths.of_frame(frame) as u32;
            let frame_header_pos = cl2.len();
            cl2.resize(cl2.len() + 10, 0);
            put16(&mut cl2, frame_header_pos, 10);
            put16(&mut cl2, frame_header_pos + 2, frame_width as u16);
            let mut transparent_run_width = 0u32;
            let mut frame_height = 0usize;
            while src != src_end {
                let mut remaining = frame_width;
                while remaining != 0 {
                    let mut val = d[src];
                    src += 1;
                    if is_cel_transparent(val) {
                        val = get_cel_transparent_width(val);
                        transparent_run_width += val as u32;
                    } else {
                        append_clx_transparent_run(transparent_run_width, &mut cl2);
                        transparent_run_width = 0;
                        append_clx_pixels_or_fill_run(&d[src..], val as u32, &mut cl2);
                        src += val as usize;
                    }
                    remaining = remaining.wrapping_sub(val as u32);
                }
                frame_height += 1;
            }
            put16(&mut cl2, frame_header_pos + 4, frame_height as u16);
            cl2[frame_header_pos + 6..frame_header_pos + 10].fill(0);
            append_clx_transparent_run(transparent_run_width, &mut cl2);
        }
        let end = (cl2.len() - cl2_data_offset) as u32;
        put32(&mut cl2, cl2_data_offset + 4 * (1 + num_frames as usize), end);
        base += src_end;
    }
    if num_groups == 1 {
        ClxSpriteListOrSheet::List(crate::engine::clx_sprite::ClxSpriteList::from_vec(cl2))
    } else {
        ClxSpriteListOrSheet::Sheet(crate::engine::clx_sprite::ClxSpriteSheet::from_vec(cl2, num_groups as u16))
    }
}

/// `GetSkipSize` (utils/cl2_to_clx.cpp)
// @port utils/cl2_to_clx.cpp|devilution::GetSkipSize(int_fast16_t overrun, int_fast16_t srcWidth) sha=4bde1ad5db9c
fn cl2_get_skip_size(overrun: i32, src_width: i32) -> (i32, i32) {
    let whole_lines = overrun / src_width;
    (whole_lines, overrun - src_width * whole_lines)
}

/// Original: `devilution::Cl2ToClx(const uint8_t *data, size_t size, PointerOrValue<uint16_t>, std::vector<uint8_t> &)`
/// (utils/cl2_to_clx.cpp). Returns the number of lists (0 for a single list).
// @port utils/cl2_to_clx.cpp|devilution::Cl2ToClx(const uint8_t *data, size_t size, PointerOrValue<uint16_t> widthOrWidths, std::vector<uint8_t> &clxData) sha=928b3c38c513
pub fn cl2_to_clx(data: &[u8], size: usize, widths: Widths, clx: &mut Vec<u8>) -> u16 {
    let mut num_groups = 1u32;
    let maybe_num_frames = le32(data, 0);
    let mut group_begin = 0usize;
    if le32(data, maybe_num_frames as usize * 4 + 4) as usize != size {
        num_groups = maybe_num_frames / 4;
        clx.resize(maybe_num_frames as usize, 0);
    }
    let mut pixels: Vec<u8> = Vec::with_capacity(4096);
    for group in 0..num_groups as usize {
        let num_frames = if num_groups == 1 {
            maybe_num_frames
        } else {
            group_begin = le32(data, group * 4) as usize;
            let pos = clx.len() as u32;
            put32(clx, 4 * group, pos);
            le32(data, group_begin)
        };
        let off = clx.len();
        clx.resize(clx.len() + 4 * (2 + num_frames as usize), 0);
        put32(clx, off, num_frames);
        let g = &data[group_begin..];
        let mut frame_end = le32(g, 4) as usize;
        for frame in 1..=num_frames as usize {
            let rel = (clx.len() - off) as u32;
            put32(clx, off + 4 * frame, rel);
            let frame_begin = frame_end;
            frame_end = le32(g, 4 * (frame + 1)) as usize;
            let frame_width = widths.of_frame(frame) as i32;
            let frame_header_pos = clx.len();
            clx.resize(clx.len() + 10, 0);
            put16(clx, frame_header_pos, 10);
            put16(clx, frame_header_pos + 2, frame_width as u16);
            let mut transparent_run_width = 0u32;
            let mut x_offset = 0i32;
            let mut frame_height = 0i32;
            let mut src = frame_begin + 10;
            while src != frame_end {
                let mut remaining_width = frame_width - x_offset;
                while remaining_width > 0 {
                    let control = g[src];
                    let (len, end) = if !is_clx_opaque(control) {
                        if !pixels.is_empty() {
                            append_clx_pixels_or_fill_run(&pixels, pixels.len() as u32, clx);
                            pixels.clear();
                        }
                        transparent_run_width += control as u32;
                        (control as u32, src + 1)
                    } else {
                        append_clx_transparent_run(transparent_run_width, clx);
                        transparent_run_width = 0;
                        if is_clx_opaque_fill(control) {
                            let w = get_clx_opaque_fill_width(control);
                            pixels.extend(std::iter::repeat_n(g[src + 1], w as usize));
                            (w as u32, src + 2)
                        } else {
                            let w = get_clx_opaque_pixels_width(control) as usize;
                            pixels.extend_from_slice(&g[src + 1..src + 1 + w]);
                            (w as u32, src + 1 + w)
                        }
                    };
                    src = end;
                    remaining_width -= len as i32;
                }
                frame_height += 1;
                if remaining_width < 0 {
                    let (whole, xo) = cl2_get_skip_size(-remaining_width, frame_width);
                    x_offset = xo;
                    frame_height += whole;
                } else {
                    x_offset = 0;
                }
            }
            if !pixels.is_empty() {
                append_clx_pixels_or_fill_run(&pixels, pixels.len() as u32, clx);
                pixels.clear();
            }
            append_clx_transparent_run(transparent_run_width, clx);
            put16(clx, frame_header_pos + 4, frame_height as u16);
            clx[frame_header_pos + 6..frame_header_pos + 10].fill(0);
        }
        let end = (clx.len() - off) as u32;
        put32(clx, off + 4 * (1 + num_frames as usize), end);
    }
    if num_groups == 1 { 0 } else { num_groups as u16 }
}
