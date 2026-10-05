//! `Source/towners.cpp`: the people of Tristram (only their graphics so far).

use crate::ctx::Ctx;
use crate::engine::clx_sprite::{ClxSpriteList, ClxSpriteSheet};

pub const NUM_TOWNERS: usize = 16;

/// `Towner` (only the fields ported so far).
#[derive(Default)]
pub struct Towner {
    /// `ownedAnim`
    pub owned_anim: Option<ClxSpriteList>,
}

/// Globals of towners.cpp.
pub struct TownersState {
    /// `Towners`
    pub towners: Vec<Towner>,
    /// `CowSprites`
    pub cow_sprites: Option<ClxSpriteSheet>,
}

impl Default for TownersState {
    fn default() -> Self {
        TownersState { towners: (0..NUM_TOWNERS).map(|_| Towner::default()).collect(), cow_sprites: None }
    }
}

/// Original: `devilution::FreeTownerGFX` (towners.cpp).
// @port towners.cpp|devilution::FreeTownerGFX() sha=9849e64e50c3
pub fn free_towner_gfx(ctx: &mut Ctx) {
    for towner in ctx.towners.towners.iter_mut() {
        towner.owned_anim = None;
    }
    ctx.towners.cow_sprites = None;
}

crate::pending_fn!(pub fn talk_to_towner(ctx: &mut Ctx, pnum: usize, t: i32), "towners.cpp|devilution::TalkToTowner(Player &player, int t)");
