//! `Source/objects.cpp`: dungeon objects (only their graphics so far).

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;

/// Globals of objects.cpp.
pub struct ObjectsState {
    /// `pObjCels`
    pub p_obj_cels: Vec<Option<ClxSpriteList>>,
    /// `numobjfiles`
    pub numobjfiles: i32,
}

impl Default for ObjectsState {
    fn default() -> Self {
        ObjectsState { p_obj_cels: (0..40).map(|_| None).collect(), numobjfiles: 0 }
    }
}

/// Original: `devilution::FreeObjectGFX` (objects.cpp).
// @port objects.cpp|devilution::FreeObjectGFX() sha=39f3f2da4e30
pub fn free_object_gfx(ctx: &mut Ctx) {
    for i in 0..ctx.objects.numobjfiles as usize {
        ctx.objects.p_obj_cels[i] = None;
    }
    ctx.objects.numobjfiles = 0;
}
