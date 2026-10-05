//! `Source/controls/touch/*`: on-screen virtual gamepad for touch devices. Replaced (not a
//! target platform, see port/replace_rules.csv): on desktop it never activates, so the
//! entry points used by shared code do nothing.

#[allow(unused_imports)]
use crate::ctx::Ctx;

/// `InitializeVirtualGamepad` (replaced: no touch input on the targets)
pub fn initialize_virtual_gamepad(ctx: &mut Ctx) {
    let _ = ctx;
}

/// `DeactivateVirtualGamepad` (replaced)
pub fn deactivate_virtual_gamepad(ctx: &mut Ctx) {
    let _ = ctx;
}

/// `FreeVirtualGamepadGFX` (replaced)
pub fn free_virtual_gamepad_gfx(ctx: &mut Ctx) {
    let _ = ctx;
}

crate::pending_fn!(pub fn render_virtual_gamepad(ctx: &mut crate::ctx::Ctx), "controls/touch/renderers.cpp|devilution::RenderVirtualGamepad(SDL_Renderer *renderer)");
