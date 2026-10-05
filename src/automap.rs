//! `Source/automap` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn do_auto_map(ctx: &mut Ctx), "automap.cpp|devilution::DoAutoMap()");

/// Globals of automap.cpp.
#[derive(Default)]
pub struct AutomapState {
    /// `AutomapActive`
    pub automap_active: bool,
}

pub fn automap_active(ctx: &Ctx) -> bool {
    ctx.automap.automap_active
}

pub fn set_automap_active(ctx: &mut Ctx, active: bool) {
    ctx.automap.automap_active = active;
}
