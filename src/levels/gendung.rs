//! `Source/levels/gendung.cpp`: dungeon globals.

#[allow(unused_imports)]
use crate::ctx::Ctx;

/// `dungeon_type`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(i8)]
pub enum DungeonType {
    #[default]
    Town = 0,
    Cathedral = 1,
    Catacombs = 2,
    Caves = 3,
    Hell = 4,
    Nest = 5,
    Crypt = 6,
    None = -1,
}

#[derive(Default)]
pub struct GendungState {
    /// `leveltype`
    pub leveltype: DungeonType,
    /// `pSpecialCels`
    pub p_special_cels: Option<crate::engine::clx_sprite::ClxSpriteList>,
    /// `pMegaTiles` (four u16 frame indices per mega tile)
    pub p_mega_tiles: Option<Vec<[u16; 4]>>,
    /// `pDungeonCels`
    pub p_dungeon_cels: Option<Vec<u8>>,
}

/// `leveltype`
pub fn leveltype(ctx: &Ctx) -> DungeonType {
    ctx.gendung.leveltype
}

/// `leveltype == DTYPE_TOWN`
pub fn leveltype_is_town(ctx: &Ctx) -> bool {
    ctx.gendung.leveltype == DungeonType::Town
}

