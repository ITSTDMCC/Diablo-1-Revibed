//! `Source/loadsave.cpp`: save games (pending).

use crate::ctx::Ctx;

crate::pending_fn!(pub fn load_hotkeys(ctx: &mut Ctx), "loadsave.cpp|devilution::LoadHotkeys()");

use crate::enums::{_item_indexes, IDI_SORCERER, IDI_SORCERER_DIABLO};

/// Original: `devilution::RemapItemIdxFromDiablo` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemapItemIdxFromDiablo(_item_indexes i) sha=ca61ae3c3622
pub fn remap_item_idx_from_diablo(i: _item_indexes) -> _item_indexes {
    let mut i = i as i32;
    if i == IDI_SORCERER as i32 {
        return IDI_SORCERER_DIABLO;
    }
    if i >= 156 {
        i += 5; // Hellfire exclusive items
    }
    if i >= 88 {
        i += 1; // Scroll of Search
    }
    if i >= 83 {
        i += 4; // Oils
    }
    i as _item_indexes
}

/// Original: `devilution::RemapItemIdxToDiablo` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemapItemIdxToDiablo(_item_indexes i) sha=2c7b3e61041e
pub fn remap_item_idx_to_diablo(i: _item_indexes) -> _item_indexes {
    let mut i = i as i32;
    if i == IDI_SORCERER_DIABLO as i32 {
        return IDI_SORCERER;
    }
    if (83..=86).contains(&i) || i == 92 || i >= 161 {
        return -1; // Hellfire exclusive items
    }
    if i >= 93 {
        i -= 1; // Scroll of Search
    }
    if i >= 87 {
        i -= 4; // Oils
    }
    i as _item_indexes
}

/// Original: `devilution::RemapItemIdxFromSpawn` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemapItemIdxFromSpawn(_item_indexes i) sha=d52799cf14a5
pub fn remap_item_idx_from_spawn(i: _item_indexes) -> _item_indexes {
    let mut i = i as i32;
    if i >= 62 {
        i += 9; // Medium and heavy armors
    }
    if i >= 96 {
        i += 1; // Scroll of Stone Curse
    }
    if i >= 98 {
        i += 1; // Scroll of Guardian
    }
    if i >= 99 {
        i += 1; // Scroll of ...
    }
    if i >= 101 {
        i += 1; // Scroll of Golem
    }
    if i >= 102 {
        i += 1; // Scroll of None
    }
    if i >= 104 {
        i += 1; // Scroll of Apocalypse
    }
    i as _item_indexes
}

/// Original: `devilution::RemapItemIdxToSpawn` (loadsave.cpp).
// @port loadsave.cpp|devilution::RemapItemIdxToSpawn(_item_indexes i) sha=b5f9bda16a70
pub fn remap_item_idx_to_spawn(i: _item_indexes) -> _item_indexes {
    let mut i = i as i32;
    if i >= 104 {
        i -= 1; // Scroll of Apocalypse
    }
    if i >= 102 {
        i -= 1; // Scroll of None
    }
    if i >= 101 {
        i -= 1; // Scroll of Golem
    }
    if i >= 99 {
        i -= 1; // Scroll of ...
    }
    if i >= 98 {
        i -= 1; // Scroll of Guardian
    }
    if i >= 96 {
        i -= 1; // Scroll of Stone Curse
    }
    if i >= 71 {
        i -= 9; // Medium and heavy armors
    }
    i as _item_indexes
}
