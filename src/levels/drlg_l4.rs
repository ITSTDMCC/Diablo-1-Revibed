//! `Source/levels/drlg_l4.cpp` (pending).

use crate::ctx::Ctx;
use crate::enums::lvl_entry;

crate::pending_fn!(pub fn create_l4_dungeon(ctx: &mut Ctx, rseed: u32, entry: lvl_entry), "levels/drlg_l4.cpp|devilution::CreateL4Dungeon(uint32_t rseed, lvl_entry entry)");

/// Globals of levels/drlg_l4.cpp.
#[derive(Default)]
pub struct DrlgL4State {
    /// `DiabloQuad1`..`DiabloQuad4`
    pub diablo_quads: [crate::engine::geometry::Point; 4],
}
