//! `Source/inv.cpp`

#[allow(unused_imports)]
use crate::ctx::Ctx;

/// `belt_item_type`
pub const BLT_HEALING: i32 = 0;
pub const BLT_MANA: i32 = 1;

crate::pending_fn!(pub fn use_belt_item_slot(ctx: &mut Ctx, i: usize), "diablo.cpp|devilution::InitKeymapActions() BeltItem lambda");

crate::pending_fn!(pub fn use_belt_item(ctx: &mut Ctx, type_: i32), "inv.cpp|devilution::UseBeltItem(int type)");
