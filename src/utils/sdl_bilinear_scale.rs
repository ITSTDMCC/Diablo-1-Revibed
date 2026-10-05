//! `Source/utils/sdl_bilinear_scale.cpp`: the 8-bit half-size downscale. (`BilinearScale32` and
//! its helpers scale the 32-bit output surface, which the Bevy renderer does instead.)

use crate::engine::surface::Surface;

/// Original: `devilution::BilinearDownscaleByHalf8` (utils/sdl_bilinear_scale.cpp). `src` and `dst`
/// are the clip rectangles of the original's surfaces.
// @port utils/sdl_bilinear_scale.cpp|devilution::BilinearDownscaleByHalf8(const SDL_Surface *src, const std::array<std::array<Uint8, 256>, 256> &paletteBlendingTable, SDL_Surface *dst, uint8_t transparentIndex) sha=9f45a6288d2d
pub fn bilinear_downscale_by_half8(src: &Surface, palette_blending_table: &[[u8; 256]; 256], dst: &Surface, transparent_index: u8) {
    for y in 0..dst.h() {
        for x in 0..dst.w() {
            let mut quad = [src.get(2 * x, 2 * y), src.get(2 * x + 1, 2 * y), src.get(2 * x, 2 * y + 1), src.get(2 * x + 1, 2 * y + 1)];
            // Attempt to avoid blending with transparent pixels
            if quad[0] == transparent_index {
                quad[0] = quad[1];
            }
            if quad[1] == transparent_index {
                quad[1] = quad[0];
            }
            if quad[2] == transparent_index {
                quad[2] = quad[3];
            }
            if quad[3] == transparent_index {
                quad[3] = quad[2];
            }
            let mut top = palette_blending_table[quad[0] as usize][quad[1] as usize];
            let mut bottom = palette_blending_table[quad[2] as usize][quad[3] as usize];
            if top == transparent_index {
                top = bottom;
            }
            if bottom == transparent_index {
                bottom = top;
            }
            dst.put(x, y, palette_blending_table[top as usize][bottom as usize]);
        }
    }
}
