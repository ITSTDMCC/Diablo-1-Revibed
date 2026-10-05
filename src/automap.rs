//! `Source/automap` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn do_auto_map(ctx: &mut Ctx), "automap.cpp|devilution::DoAutoMap()");

use crate::levels::gendung::{DMAXX, DMAXY};

/// Globals of automap.cpp.
pub struct AutomapState {
    pub AutomapActive: bool,
    /// `AutomapView[DMAXX][DMAXY]`
    pub AutomapView: Box<[[u8; DMAXY]; DMAXX]>,
    pub AutoMapScale: i32,
    pub AutomapOffset: crate::engine::geometry::Displacement,
}

impl Default for AutomapState {
    fn default() -> Self {
        AutomapState { AutomapActive: false, AutomapView: Box::new([[0; DMAXY]; DMAXX]), AutoMapScale: 0, AutomapOffset: Default::default() }
    }
}

pub fn automap_active(ctx: &Ctx) -> bool {
    ctx.automap.AutomapActive
}

pub fn set_automap_active(ctx: &mut Ctx, active: bool) {
    ctx.automap.AutomapActive = active;
}

crate::pending_fn!(pub fn automap_zoom_reset(ctx: &mut Ctx), "automap.cpp|devilution::AutomapZoomReset()");

crate::pending_fn!(pub fn set_automap_view(ctx: &mut Ctx, position: crate::engine::geometry::Point, explore_type: crate::lighting::MapExplorationType), "automap.cpp|devilution::SetAutomapView(Point position, MapExplorationType explorationType)");
