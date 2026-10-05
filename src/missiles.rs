//! `Source/missiles.cpp` and `Source/misdat.cpp`: missiles (only their graphics so far).

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteListOrSheet;

/// `MissileFileData` (only the loaded sprites so far; the static fields come with misdat).
#[derive(Default)]
pub struct MissileFileData {
    pub sprites: Option<ClxSpriteListOrSheet>,
}

impl MissileFileData {
    /// Original: `MissileFileData::FreeGFX` (misdat.h).
    // @port misdat.h|devilution::MissileFileData::FreeGFX() sha=67f9ef1c4ce8
    pub fn free_gfx(&mut self) {
        self.sprites = None;
    }
}

/// Globals of missiles.cpp / misdat.cpp.
#[derive(Default)]
pub struct MissilesState {
    /// `MissileSpriteData`
    pub missile_sprite_data: Vec<MissileFileData>,
}

/// Original: `devilution::FreeMissileGFX` (misdat.cpp).
// @port misdat.cpp|devilution::FreeMissileGFX() sha=dc3a9beaef6d
pub fn free_missile_gfx(ctx: &mut Ctx) {
    for missile_data in ctx.missiles.missile_sprite_data.iter_mut() {
        missile_data.free_gfx();
    }
}
