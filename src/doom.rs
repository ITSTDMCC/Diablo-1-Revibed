//! `Source/doom.cpp`: the map of the stars quest.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::surface::Surface;

/// Globals of doom.cpp.
#[derive(Default)]
pub struct DoomState {
    /// `DoomSprite`
    doom_sprite: Option<ClxSpriteList>,
    /// `DoomFlag`
    pub DoomFlag: bool,
}

/// Original: `devilution::doom_init` (doom.cpp).
// @port doom.cpp|devilution::doom_init() sha=c4277e8e7be0
pub fn doom_init(ctx: &mut Ctx) {
    ctx.doom.doom_sprite = Some(crate::engine::load_sprites::load_cel(ctx, "items\\map\\mapztown", 640));
    ctx.doom.DoomFlag = true;
}

/// Original: `devilution::doom_close` (doom.cpp).
// @port doom.cpp|devilution::doom_close() sha=cc6321d5608b
pub fn doom_close(ctx: &mut Ctx) {
    ctx.doom.DoomFlag = false;
    ctx.doom.doom_sprite = None;
}

/// Original: `devilution::doom_draw` (doom.cpp).
// @port doom.cpp|devilution::doom_draw(const Surface &out) sha=b8ce144064a0
pub fn doom_draw(ctx: &mut Ctx, out: &Surface) {
    if !ctx.doom.DoomFlag {
        return;
    }
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    let sprite = ctx.doom.doom_sprite.as_ref().expect("DoomSprite").get(0);
    crate::engine::render::clx_render::clx_draw(out, (ui.x, ui.y + 352), &sprite);
}
