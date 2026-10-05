//! `Source/qol/stash.cpp`: the shared stash (only its graphics so far).

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;

/// Globals of stash.cpp.
#[derive(Default)]
pub struct StashState {
    /// `StashPanelArt`
    pub stash_panel_art: Option<ClxSpriteList>,
    /// `StashNavButtonArt`
    pub stash_nav_button_art: Option<ClxSpriteList>,
}

/// Original: `devilution::FreeStashGFX` (qol/stash.cpp).
// @port qol/stash.cpp|devilution::FreeStashGFX() sha=5a2700c004bb
pub fn free_stash_gfx(ctx: &mut Ctx) {
    ctx.stash.stash_nav_button_art = None;
    ctx.stash.stash_panel_art = None;
}

crate::pending_fn!(pub fn sfile_write_stash(ctx: &mut Ctx), "pfile.cpp|devilution::sfile_write_stash()");
