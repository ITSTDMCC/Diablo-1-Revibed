//! `Source/engine/load_file.hpp`: reading whole asset files.

use crate::ctx::Ctx;
use crate::engine::assets::{open_asset, validate_handle};

/// Original: `LoadFileInMem(const char *path, std::array<T, N> &data)` /
/// `LoadFileInMem(path, data, count)` (engine/load_file.hpp): fills `data` from the start of
/// the file (fatal error if the file is missing; a short file leaves the rest untouched).
// @port engine/load_file.hpp|devilution::LoadFileInMem(const char *path, std::array<T, N> &data) sha=6750e2db1790
// @port engine/load_file.hpp|devilution::LoadFileInMem(const char *path, T *data, std::size_t count) sha=3d3fdcd4a661
pub fn load_file_in_mem_exact(ctx: &mut Ctx, path: &str, data: &mut [u8]) {
    let mut handle = open_asset(ctx, path);
    if !validate_handle(ctx, path, &handle) {
        return;
    }
    handle.read(data);
}

/// Original: `LoadFileInMem<T>(const char *path, std::size_t *numRead)` (engine/load_file.hpp):
/// the whole file, or None (after a fatal error dialog) if it cannot be opened.
// @port engine/load_file.hpp|devilution::LoadFileInMem(const char *path, std::size_t *numRead = nullptr) sha=b7e1072e819c
pub fn load_file_in_mem(ctx: &mut Ctx, path: &str) -> Option<Vec<u8>> {
    let handle = open_asset(ctx, path);
    if !validate_handle(ctx, path, &handle) {
        return None;
    }
    Some(handle.into_bytes())
}

/// Original: `LoadOptionalFileInMem` (engine/load_file.hpp).
// @port engine/load_file.hpp|devilution::LoadOptionalFileInMem(const char *path, T *data, std::size_t count) sha=ba646a7d2838
pub fn load_optional_file_in_mem(ctx: &mut Ctx, path: &str, data: &mut [u8]) -> bool {
    let mut handle = open_asset(ctx, path);
    handle.ok() && handle.read(data)
}
