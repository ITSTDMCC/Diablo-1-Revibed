//! Locating the player's own game data for tests. Tests that need it skip (and say so) when it is
//! missing, so the repository never needs game files.

use std::path::PathBuf;

/// Folder holding DIABDAT.MPQ: `$DIABLO_DATA_DIR`, else the owner's DevilutionX folder in Downloads.
pub fn data_dir() -> Option<PathBuf> {
    let dir = std::env::var_os("DIABLO_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join("Downloads").join("devilutionx")))?;
    dir.join("DIABDAT.MPQ").is_file().then_some(dir)
}

pub fn diabdat() -> Option<PathBuf> {
    let d = data_dir().map(|d| d.join("DIABDAT.MPQ"));
    if d.is_none() {
        eprintln!("skipped: DIABDAT.MPQ not found (set DIABLO_DATA_DIR)");
    }
    d
}
