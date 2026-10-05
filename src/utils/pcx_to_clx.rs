//! `Source/utils/pcx_to_clx.cpp`: PCX images to CLX sprite lists at load time.

use crate::engine::assets::AssetHandle;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::dx::Color;
use crate::utils::clx_encode::{append_clx_pixels_or_fill_run, append_clx_transparent_run};

/// `PcxHeaderSize`
pub const PCX_HEADER_SIZE: usize = 128;

/// Original: `GetReservationSize` (utils/pcx_to_clx.cpp): a capacity hint only.
// @port utils/pcx_to_clx.cpp|devilution::GetReservationSize(size_t pcxSize) sha=46912da7ed80
fn get_reservation_size(pcx_size: usize) -> usize {
    match pcx_size {
        2352187 => 2464867,
        172347 => 172347,
        157275 => 173367,
        _ => pcx_size,
    }
}

/// Original: `LoadPcxMeta` (utils/pcx_to_clx.cpp): width, height, bits per pixel.
// @port utils/pcx_to_clx.cpp|devilution::LoadPcxMeta(AssetHandle &handle, int &width, int &height, uint8_t &bpp) sha=63babfb0c0e0
fn load_pcx_meta(handle: &mut AssetHandle) -> Option<(i32, i32, u8)> {
    let mut h = [0u8; PCX_HEADER_SIZE];
    if !handle.read(&mut h) {
        return None;
    }
    let u16_at = |o: usize| u16::from_le_bytes([h[o], h[o + 1]]) as i32;
    Some((u16_at(8) - u16_at(4) + 1, u16_at(10) - u16_at(6) + 1, h[3]))
}

/// Original: `devilution::PcxToClx` (utils/pcx_to_clx.cpp). `num_frames_or_frame_height` > 0
/// is a frame count, otherwise minus the frame height. Pixels equal to `transparent_color`
/// become transparent runs, which (as in the original) continue across lines.
// @port utils/pcx_to_clx.cpp|devilution::PcxToClx(AssetHandle &handle, size_t fileSize, int numFramesOrFrameHeight, std::optional<uint8_t> transparentColor, SDL_Color *outPalette) sha=bf0804ec671c
pub fn pcx_to_clx(
    handle: &mut AssetHandle,
    file_size: usize,
    num_frames_or_frame_height: i32,
    transparent_color: Option<u8>,
    out_palette: Option<&mut [Color; 256]>,
) -> Option<ClxSpriteList> {
    let (width, height, bpp) = load_pcx_meta(handle)?;
    assert_eq!(bpp, 8);
    let (num_frames, frame_height) = if num_frames_or_frame_height > 0 {
        let n = num_frames_or_frame_height as u32;
        (n, height as u32 / n)
    } else {
        let fh = (-num_frames_or_frame_height) as u32;
        (height as u32 / fh, fh)
    };
    if file_size <= PCX_HEADER_SIZE {
        return None;
    }
    let pixel_data_size = file_size - PCX_HEADER_SIZE;
    let mut file_buffer = vec![0u8; pixel_data_size];
    if !handle.read(&mut file_buffer) {
        return None;
    }
    let width = width as usize;
    let mut cl2: Vec<u8> = Vec::with_capacity(get_reservation_size(pixel_data_size));
    cl2.resize(4 * (2 + num_frames as usize), 0);
    cl2[0..4].copy_from_slice(&num_frames.to_le_bytes());
    // One frame at a time, because the lines are stored bottom-up in CLX.
    let mut frame_buffer = vec![0u8; frame_height as usize * width + 64];
    let src_skip = width % 2;
    let mut p = 0usize;
    for frame in 1..=num_frames as usize {
        let pos = cl2.len() as u32;
        cl2[4 * frame..4 * frame + 4].copy_from_slice(&pos.to_le_bytes());
        const FRAME_HEADER_SIZE: u16 = 10;
        cl2.extend_from_slice(&FRAME_HEADER_SIZE.to_le_bytes());
        cl2.extend_from_slice(&(width as u16).to_le_bytes());
        cl2.extend_from_slice(&(frame_height as u16).to_le_bytes());
        cl2.extend_from_slice(&[0; 4]);
        for j in 0..frame_height as usize {
            let mut b = j * width;
            let mut x = 0usize;
            while x < width {
                const PCX_MAX_SINGLE_PIXEL: u8 = 0xBF;
                let byte = file_buffer[p];
                p += 1;
                if byte <= PCX_MAX_SINGLE_PIXEL {
                    frame_buffer[b] = byte;
                    b += 1;
                    x += 1;
                    continue;
                }
                let run_length = (byte & 0x3F) as usize;
                let value = file_buffer[p];
                p += 1;
                // a run may cross the end of the line (memset in the original)
                let end = (b + run_length).min(frame_buffer.len());
                frame_buffer[b..end].fill(value);
                b += run_length;
                x += run_length;
            }
            p += src_skip;
        }
        let mut transparent_run_width = 0u32;
        for line in 0..frame_height as usize {
            let row_start = (frame_height as usize - (line + 1)) * width;
            let src = &frame_buffer[row_start..row_start + width];
            if let Some(tc) = transparent_color {
                let mut solid_run_width = 0u32;
                for (i, &px) in src.iter().enumerate() {
                    if px == tc {
                        if solid_run_width != 0 {
                            // transparentRunWidth is 0 here: it was flushed at the run's first pixel
                            let s = i - transparent_run_width as usize - solid_run_width as usize;
                            append_clx_pixels_or_fill_run(&src[s..], solid_run_width, &mut cl2);
                            solid_run_width = 0;
                        }
                        transparent_run_width += 1;
                    } else {
                        append_clx_transparent_run(transparent_run_width, &mut cl2);
                        transparent_run_width = 0;
                        solid_run_width += 1;
                    }
                }
                if solid_run_width != 0 {
                    append_clx_pixels_or_fill_run(&src[width - solid_run_width as usize..], solid_run_width, &mut cl2);
                }
            } else {
                append_clx_pixels_or_fill_run(src, width as u32, &mut cl2);
            }
        }
        append_clx_transparent_run(transparent_run_width, &mut cl2);
    }
    let total = cl2.len() as u32;
    let o = 4 * (1 + num_frames as usize);
    cl2[o..o + 4].copy_from_slice(&total.to_le_bytes());
    if let Some(pal) = out_palette {
        // 0x0C separator, then 256 RGB entries
        p += 1;
        for c in pal.iter_mut() {
            *c = [file_buffer[p], file_buffer[p + 1], file_buffer[p + 2]];
            p += 3;
        }
    }
    Some(ClxSpriteList::from_vec(cl2))
}
