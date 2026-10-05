//! Sprite loading: `engine/load_cel.cpp`/`.hpp`, `engine/load_cl2.cpp`/`.hpp`,
//! `engine/load_clx.cpp`/`.hpp`, `engine/load_pcx.cpp`/`.hpp`.

use crate::ctx::Ctx;
use crate::engine::assets::{asset_size, failed_to_open_file_error, find_asset, open_asset, open_asset_ref};
use crate::engine::clx_sprite::{ClxSpriteList, ClxSpriteListOrSheet, ClxSpriteSheet};
use crate::engine::dx::Color;
use crate::engine::load_file::load_file_in_mem;
use crate::platform::log;
use crate::utils::cel_to_clx::{cel_to_clx, cl2_to_clx, Widths};
use crate::utils::pcx_to_clx::pcx_to_clx;

/// Original: `devilution::LoadCelListOrSheet` (engine/load_cel.cpp).
// @port engine/load_cel.cpp|devilution::LoadCelListOrSheet(const char *pszName, PointerOrValue<uint16_t> widthOrWidths) sha=36e0a656f403
pub fn load_cel_list_or_sheet(ctx: &mut Ctx, name: &str, widths: Widths) -> ClxSpriteListOrSheet {
    let path = format!("{name}.cel");
    let data = load_file_in_mem(ctx, &path).unwrap_or_default();
    let size = data.len();
    cel_to_clx(&data, size, widths)
}

/// Original: `devilution::LoadCel(const char *pszName, uint16_t width)` (engine/load_cel.hpp).
// @port engine/load_cel.hpp|devilution::LoadCel(const char *pszName, uint16_t width) sha=f4540a5e5cc5
pub fn load_cel(ctx: &mut Ctx, name: &str, width: u16) -> ClxSpriteList {
    load_cel_list_or_sheet(ctx, name, Widths::Value(width)).list().clone()
}

/// Original: `devilution::LoadCel(const char *pszName, const uint16_t *widths)` (engine/load_cel.hpp).
// @port engine/load_cel.hpp|devilution::LoadCel(const char *pszName, const uint16_t *widths) sha=7404db3506ca
pub fn load_cel_widths(ctx: &mut Ctx, name: &str, widths: &[u16]) -> ClxSpriteList {
    load_cel_list_or_sheet(ctx, name, Widths::Pointer(widths)).list().clone()
}

/// Original: `devilution::LoadCelSheet` (engine/load_cel.hpp).
// @port engine/load_cel.hpp|devilution::LoadCelSheet(const char *pszName, uint16_t width) sha=d9c61e187c23
pub fn load_cel_sheet(ctx: &mut Ctx, name: &str, width: u16) -> ClxSpriteSheet {
    load_cel_list_or_sheet(ctx, name, Widths::Value(width)).sheet().clone()
}

/// Original: `devilution::LoadCl2ListOrSheet` (engine/load_cl2.cpp).
// @port engine/load_cl2.cpp|devilution::LoadCl2ListOrSheet(const char *pszName, PointerOrValue<uint16_t> widthOrWidths) sha=824be29a020b
pub fn load_cl2_list_or_sheet(ctx: &mut Ctx, name: &str, widths: Widths) -> ClxSpriteListOrSheet {
    let path = format!("{name}.cl2");
    let data = load_file_in_mem(ctx, &path).unwrap_or_default();
    cl2_to_clx_owned(&data, data.len(), widths)
}

/// Original: `devilution::Cl2ToClx(std::unique_ptr<uint8_t[]> &&data, size_t size, PointerOrValue<uint16_t>)`
/// (utils/cl2_to_clx.hpp).
// @port utils/cl2_to_clx.hpp|devilution::Cl2ToClx(std::unique_ptr<uint8_t[]> &&data, size_t size, PointerOrValue<uint16_t> widthOrWidths) sha=dc3ed51bfdad
pub fn cl2_to_clx_owned(data: &[u8], size: usize, widths: Widths) -> ClxSpriteListOrSheet {
    let mut clx = Vec::new();
    let num_lists = cl2_to_clx(data, size, widths, &mut clx);
    if num_lists == 0 {
        ClxSpriteListOrSheet::List(ClxSpriteList::from_vec(clx))
    } else {
        ClxSpriteListOrSheet::Sheet(ClxSpriteSheet::from_vec(clx, num_lists))
    }
}

/// Original: `devilution::LoadCl2(const char *pszName, uint16_t width)` (engine/load_cl2.hpp).
// @port engine/load_cl2.hpp|devilution::LoadCl2(const char *pszName, uint16_t width) sha=deb9a086d8cb
pub fn load_cl2(ctx: &mut Ctx, name: &str, width: u16) -> ClxSpriteList {
    load_cl2_list_or_sheet(ctx, name, Widths::Value(width)).list().clone()
}

/// Original: `devilution::LoadCl2(const char *pszName, const uint16_t *widths)` (engine/load_cl2.hpp).
// @port engine/load_cl2.hpp|devilution::LoadCl2(const char *pszName, const uint16_t *widths) sha=b33a67c7dcfc
pub fn load_cl2_widths(ctx: &mut Ctx, name: &str, widths: &[u16]) -> ClxSpriteList {
    load_cl2_list_or_sheet(ctx, name, Widths::Pointer(widths)).list().clone()
}

/// Original: `devilution::LoadCl2Sheet` (engine/load_cl2.hpp).
// @port engine/load_cl2.hpp|devilution::LoadCl2Sheet(const char *pszName, uint16_t width) sha=1029ada44ef3
pub fn load_cl2_sheet(ctx: &mut Ctx, name: &str, width: u16) -> ClxSpriteSheet {
    load_cl2_list_or_sheet(ctx, name, Widths::Value(width)).sheet().clone()
}

/// Original: `devilution::LoadMultipleCl2Sheet` (engine/load_cl2.hpp): several CL2 files as
/// the lists of one sheet.
// @port engine/load_cl2.hpp|devilution::LoadMultipleCl2Sheet(tl::function_ref<const char *(size_t)> filenames, size_t count, uint16_t width) sha=9d4da74ec3d8
pub fn load_multiple_cl2_sheet(ctx: &mut Ctx, filenames: &[String], width: u16) -> ClxSpriteSheet {
    let count = filenames.len();
    let header = 4 * count;
    let mut refs = Vec::with_capacity(count);
    let mut total = header;
    for path in filenames {
        let r = find_asset(ctx, path);
        if !r.ok() {
            failed_to_open_file_error(ctx, path, "File not found");
        }
        let size = asset_size(ctx, &r);
        total += size;
        refs.push((r, size));
    }
    let mut data = vec![0u8; total];
    let mut accumulated = header;
    for (i, (r, size)) in refs.into_iter().enumerate() {
        let mut h = open_asset_ref(ctx, r);
        if !h.ok() || !h.read(&mut data[accumulated..accumulated + size]) {
            let e = h.error().to_string();
            failed_to_open_file_error(ctx, &filenames[i], &e);
        }
        data[i * 4..i * 4 + 4].copy_from_slice(&(accumulated as u32).to_le_bytes());
        accumulated += size;
    }
    cl2_to_clx_owned(&data, accumulated, Widths::Value(width)).sheet().clone()
}

/// Original: `devilution::LoadOptionalClxListOrSheet` (engine/load_clx.cpp).
// @port engine/load_clx.cpp|devilution::LoadOptionalClxListOrSheet(const char *path) sha=639fba59fa48
pub fn load_optional_clx_list_or_sheet(ctx: &mut Ctx, path: &str) -> Option<ClxSpriteListOrSheet> {
    let r = find_asset(ctx, path);
    if !r.ok() {
        return None;
    }
    let size = asset_size(ctx, &r);
    let mut data = vec![0u8; size];
    let mut h = open_asset_ref(ctx, r);
    if !h.ok() || !h.read(&mut data) {
        return None;
    }
    Some(ClxSpriteListOrSheet::from_buffer(data))
}

/// Original: `devilution::LoadClxListOrSheet` (engine/load_clx.cpp).
// @port engine/load_clx.cpp|devilution::LoadClxListOrSheet(const char *path) sha=99990e7aaf2a
pub fn load_clx_list_or_sheet(ctx: &mut Ctx, path: &str) -> ClxSpriteListOrSheet {
    let data = load_file_in_mem(ctx, path).unwrap_or_default();
    ClxSpriteListOrSheet::from_buffer(data)
}

/// Original: `devilution::LoadClx` (engine/load_clx.hpp).
// @port engine/load_clx.hpp|devilution::LoadClx(const char *path) sha=01feabb0a648
pub fn load_clx(ctx: &mut Ctx, path: &str) -> ClxSpriteList {
    load_clx_list_or_sheet(ctx, path).list().clone()
}

/// Original: `devilution::LoadClxSheet` (engine/load_clx.hpp).
// @port engine/load_clx.hpp|devilution::LoadClxSheet(const char *path) sha=a55d34ce961c
pub fn load_clx_sheet(ctx: &mut Ctx, path: &str) -> ClxSpriteSheet {
    load_clx_list_or_sheet(ctx, path).sheet().clone()
}

/// Original: `devilution::LoadOptionalClx` (engine/load_clx.hpp).
// @port engine/load_clx.hpp|devilution::LoadOptionalClx(const char *path) sha=4d94ff4c696f
pub fn load_optional_clx(ctx: &mut Ctx, path: &str) -> Option<ClxSpriteList> {
    load_optional_clx_list_or_sheet(ctx, path).map(|r| r.list().clone())
}

/// Original: `devilution::LoadPcxSpriteList` (engine/load_pcx.cpp).
// @port engine/load_pcx.cpp|devilution::LoadPcxSpriteList(const char *filename, int numFramesOrFrameHeight, std::optional<uint8_t> transparentColor, SDL_Color *outPalette, bool logError) sha=9005b9b1efd8
pub fn load_pcx_sprite_list(
    ctx: &mut Ctx,
    filename: &str,
    num_frames_or_frame_height: i32,
    transparent_color: Option<u8>,
    out_palette: Option<&mut [Color; 256]>,
    log_error: bool,
) -> Option<ClxSpriteList> {
    let path = format!("{filename}.pcx");
    let mut handle = open_asset(ctx, &path);
    if !handle.ok() {
        if log_error {
            log::error!("Missing file: {}", path);
        }
        return None;
    }
    let size = handle.size();
    pcx_to_clx(&mut handle, size, num_frames_or_frame_height, transparent_color, out_palette)
}

/// Original: `devilution::LoadPcx` (engine/load_pcx.hpp).
// @port engine/load_pcx.hpp|devilution::LoadPcx(const char *filename, std::optional<uint8_t> transparentColor = std::nullopt, SDL_Color *outPalette = nullptr, bool logError = true) sha=cbd1ee064336
pub fn load_pcx(ctx: &mut Ctx, filename: &str, transparent_color: Option<u8>, out_palette: Option<&mut [Color; 256]>, log_error: bool) -> Option<ClxSpriteList> {
    load_pcx_sprite_list(ctx, filename, 1, transparent_color, out_palette, log_error)
}
