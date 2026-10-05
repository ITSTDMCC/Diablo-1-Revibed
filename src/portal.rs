//! `Source/portal.cpp`: town portals (pending).

use crate::ctx::Ctx;
use crate::engine::geometry::Point;
use crate::levels::gendung::DungeonType;

pub const MAXPORTAL: usize = 4;

/// `Portal`
#[derive(Clone, Copy, Debug)]
pub struct Portal {
    pub open: bool,
    pub position: Point,
    pub level: i32,
    pub ltype: DungeonType,
    pub setlvl: bool,
}

impl Default for Portal {
    fn default() -> Self {
        Portal { open: false, position: Point::default(), level: 0, ltype: DungeonType::Town, setlvl: false }
    }
}

/// Globals of portal.cpp.
#[derive(Default)]
pub struct PortalState {
    pub Portals: [Portal; MAXPORTAL],
    /// `portalindex`
    pub portalindex: usize,
}

crate::pending_fn!(pub fn set_current_portal(ctx: &mut Ctx, p: usize), "portal.cpp|devilution::SetCurrentPortal(size_t p)");
crate::pending_fn!(pub fn pos_ok_portal(ctx: &Ctx, lvl: i32, position: Point) -> bool, "portal.cpp|devilution::PosOkPortal(int lvl, Point position)");
crate::pending_fn!(pub fn init_portals(ctx: &mut Ctx), "portal.cpp|devilution::InitPortals()");
