//! `Source/utils/clx_encode.hpp`: writing CLX pixel commands.

/// Original: `devilution::AppendClxTransparentRun` (utils/clx_encode.hpp).
// @port utils/clx_encode.hpp|devilution::AppendClxTransparentRun(unsigned width, std::vector<uint8_t> &out) sha=4195a9c3fbd2
pub fn append_clx_transparent_run(mut width: u32, out: &mut Vec<u8>) {
    while width >= 0x7F {
        out.push(0x7F);
        width -= 0x7F;
    }
    if width == 0 {
        return;
    }
    out.push(width as u8);
}

/// Original: `devilution::AppendClxFillRun` (utils/clx_encode.hpp).
// @port utils/clx_encode.hpp|devilution::AppendClxFillRun(uint8_t color, unsigned width, std::vector<uint8_t> &out) sha=f162b5f02ec5
pub fn append_clx_fill_run(color: u8, mut width: u32, out: &mut Vec<u8>) {
    while width >= 0x3F {
        out.push(0x80);
        out.push(color);
        width -= 0x3F;
    }
    if width == 0 {
        return;
    }
    out.push((0xBF - width) as u8);
    out.push(color);
}

/// Original: `devilution::AppendClxPixelsRun` (utils/clx_encode.hpp).
// @port utils/clx_encode.hpp|devilution::AppendClxPixelsRun(const uint8_t *src, unsigned width, std::vector<uint8_t> &out) sha=0b9dbff1ac27
pub fn append_clx_pixels_run(src: &[u8], mut width: u32, out: &mut Vec<u8>) {
    let mut src = src;
    while width >= 0x41 {
        out.push(0xBF);
        out.extend_from_slice(&src[..0x41]);
        width -= 0x41;
        src = &src[0x41..];
    }
    if width == 0 {
        return;
    }
    out.push((256 - width) as u8);
    out.extend_from_slice(&src[..width as usize]);
}

/// Original: `devilution::AppendClxPixelsOrFillRun` (utils/clx_encode.hpp): runs of 3+ equal
/// pixels become fill commands; a final run of 2+ too (it is followed by transparency).
// @port utils/clx_encode.hpp|devilution::AppendClxPixelsOrFillRun(const uint8_t *src, unsigned length, std::vector<uint8_t> &out) sha=1644cc19c9de
pub fn append_clx_pixels_or_fill_run(src: &[u8], length: u32, out: &mut Vec<u8>) {
    const MIN_FILL_RUN_LENGTH: u32 = 3;
    let mut begin = 0usize;
    let mut prev_color_begin = 0usize;
    let mut prev_color_run_length = 1u32;
    let mut prev_color = src[0];
    let mut i = 1usize;
    let mut remaining = length;
    loop {
        remaining -= 1;
        if remaining == 0 {
            break;
        }
        let color = src[i];
        if prev_color == color {
            prev_color_run_length += 1;
        } else {
            if prev_color_run_length >= MIN_FILL_RUN_LENGTH {
                append_clx_pixels_run(&src[begin..], (prev_color_begin - begin) as u32, out);
                append_clx_fill_run(prev_color, prev_color_run_length, out);
                begin = i;
            }
            prev_color_begin = i;
            prev_color_run_length = 1;
            prev_color = color;
        }
        i += 1;
    }
    if prev_color_run_length >= 2 {
        append_clx_pixels_run(&src[begin..], (prev_color_begin - begin) as u32, out);
        append_clx_fill_run(prev_color, prev_color_run_length, out);
    } else {
        append_clx_pixels_run(&src[begin..], (prev_color_begin - begin) as u32 + prev_color_run_length, out);
    }
}
