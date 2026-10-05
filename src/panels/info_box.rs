//! `Source/panels/info_box.cpp`: the info box frame and scrollbar graphics.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;

/// Globals of panels/info_box.cpp.
#[derive(Default)]
pub struct InfoBoxState {
    /// `pSTextBoxCels`: info box frame, used in stores, the quest log, the help window and the
    /// unique item info window.
    pub p_s_text_box_cels: Option<ClxSpriteList>,
    /// `pSTextSlidCels`: info box scrollbar graphics, used in stores and `DrawDiabloMsg`.
    pub p_s_text_slid_cels: Option<ClxSpriteList>,
}

/// Original: `devilution::InitInfoBoxGfx` (panels/info_box.cpp).
// @port panels/info_box.cpp|devilution::InitInfoBoxGfx() sha=fb4476c27eaa
pub fn init_info_box_gfx(ctx: &mut Ctx) {
    ctx.info_box.p_s_text_slid_cels = Some(crate::engine::load_sprites::load_cel(ctx, "data\\textslid", 12));
    ctx.info_box.p_s_text_box_cels = Some(crate::engine::load_sprites::load_cel(ctx, "data\\textbox2", 271));
}

/// Original: `devilution::FreeInfoBoxGfx` (panels/info_box.cpp).
// @port panels/info_box.cpp|devilution::FreeInfoBoxGfx() sha=eb340aaa22aa
pub fn free_info_box_gfx(ctx: &mut Ctx) {
    ctx.info_box.p_s_text_box_cels = None;
    ctx.info_box.p_s_text_slid_cels = None;
}
