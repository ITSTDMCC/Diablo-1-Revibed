//! `Source/utils/surface_to_clx.cpp`: encoding a surface as a CLX sprite list.

use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::surface::Surface;
use crate::utils::clx_encode::{append_clx_pixels_or_fill_run, append_clx_transparent_run};

/// Original: `devilution::SurfaceToClx` (utils/surface_to_clx.cpp). The surface is split
/// vertically into `num_frames` frames of equal height.
// @port utils/surface_to_clx.cpp|devilution::SurfaceToClx(const Surface &surface, unsigned numFrames, std::optional<uint8_t> transparentColor) sha=bdfffcdb4e45
pub fn surface_to_clx(surface: &Surface, num_frames: u32, transparent_color: Option<u8>) -> ClxSpriteList {
    // CLX header: frame count, frame offset for each frame, file size
    let mut clx_data: Vec<u8> = vec![0; 4 * (2 + num_frames as usize)];
    clx_data[0..4].copy_from_slice(&num_frames.to_le_bytes());

    let height = surface.h() as u32;
    let width = surface.w() as u32;
    let frame_height = height / num_frames;

    // We process the surface a whole frame at a time because the lines are reversed in CEL.
    let mut frame_top: u32 = 0;
    for frame in 1..=num_frames as usize {
        let size = clx_data.len() as u32;
        clx_data[4 * frame..4 * frame + 4].copy_from_slice(&size.to_le_bytes());

        // Frame header: 5 16-bit values:
        // 1. Offset to start of the pixel data.
        // 2. Width
        // 3. Height
        // 4..5. Unused (0)
        const FRAME_HEADER_SIZE: u16 = 10;
        clx_data.extend_from_slice(&FRAME_HEADER_SIZE.to_le_bytes());
        clx_data.extend_from_slice(&(width as u16).to_le_bytes());
        clx_data.extend_from_slice(&(frame_height as u16).to_le_bytes());
        clx_data.extend_from_slice(&[0, 0, 0, 0]);

        let mut transparent_run_width: u32 = 0;
        for line in 0..frame_height {
            // Process line:
            let y = (frame_top + frame_height - (line + 1)) as i32;
            let src: Vec<u8> = surface.row(y).to_vec();
            if let Some(tc) = transparent_color {
                let mut solid_run_width: u32 = 0;
                for (x, &px) in src.iter().enumerate() {
                    if px == tc {
                        if solid_run_width != 0 {
                            let start = x - solid_run_width as usize;
                            append_clx_pixels_or_fill_run(&src[start..], solid_run_width, &mut clx_data);
                            solid_run_width = 0;
                        }
                        transparent_run_width += 1;
                    } else {
                        append_clx_transparent_run(transparent_run_width, &mut clx_data);
                        transparent_run_width = 0;
                        solid_run_width += 1;
                    }
                }
                if solid_run_width != 0 {
                    let start = src.len() - solid_run_width as usize;
                    append_clx_pixels_or_fill_run(&src[start..], solid_run_width, &mut clx_data);
                }
            } else {
                append_clx_pixels_or_fill_run(&src, width, &mut clx_data);
            }
        }
        append_clx_transparent_run(transparent_run_width, &mut clx_data);
        frame_top += frame_height;
    }

    let size = clx_data.len() as u32;
    let at = 4 * (1 + num_frames as usize);
    clx_data[at..at + 4].copy_from_slice(&size.to_le_bytes());
    ClxSpriteList::from_vec(clx_data)
}
