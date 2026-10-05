//! `Source/engine/trn.cpp`: palette translation tables.

use crate::ctx::Ctx;
use crate::enums::HeroClass;

/// Original: `devilution::GetInfravisionTRN` (engine/trn.cpp).
// @port engine/trn.cpp|devilution::GetInfravisionTRN() sha=429a3d6fd40b
pub fn get_infravision_trn(ctx: &Ctx) -> &[u8; 256] {
    &ctx.lighting.InfravisionTable
}

/// Original: `devilution::GetStoneTRN` (engine/trn.cpp).
// @port engine/trn.cpp|devilution::GetStoneTRN() sha=9b7c6f770bfb
pub fn get_stone_trn(ctx: &Ctx) -> &[u8; 256] {
    &ctx.lighting.StoneTable
}

/// Original: `devilution::GetPauseTRN` (engine/trn.cpp).
// @port engine/trn.cpp|devilution::GetPauseTRN() sha=af251da54294
pub fn get_pause_trn(ctx: &Ctx) -> &[u8; 256] {
    &ctx.lighting.PauseTable
}

/// Original: `devilution::GetClassTRN` (engine/trn.cpp). Takes the class instead of the player.
// @port engine/trn.cpp|devilution::GetClassTRN(Player &player) sha=a8321050bde9
pub fn get_class_trn(ctx: &mut Ctx, class: HeroClass) -> Option<[u8; 256]> {
    let path = match class {
        HeroClass::Warrior => r"plrgfx\warrior.trn",
        HeroClass::Rogue => r"plrgfx\rogue.trn",
        HeroClass::Sorcerer => r"plrgfx\sorcerer.trn",
        HeroClass::Monk => r"plrgfx\monk.trn",
        HeroClass::Bard => r"plrgfx\bard.trn",
        HeroClass::Barbarian => r"plrgfx\barbarian.trn",
    };
    let mut trn = [0u8; 256];
    if crate::engine::load_file::load_optional_file_in_mem(ctx, path, &mut trn) {
        return Some(trn);
    }
    None
}
