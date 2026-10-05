//! `Source/inv.cpp`

#[allow(unused_imports)]
use crate::ctx::Ctx;
use crate::engine::geometry::Point;
use crate::enums::*;
use crate::items::Item;

/// Globals of inv.cpp.
#[derive(Default)]
pub struct InvState {
    pub invflag: bool,
}

crate::pending_fn!(pub fn auto_place_item_in_inventory_slot(ctx: &mut Ctx, pnum: usize, slot_index: i32, item: &Item, persist_item: bool) -> bool, "inv.cpp|devilution::AutoPlaceItemInInventorySlot(Player &player, int slotIndex, const Item &item, bool persistItem)");
crate::pending_fn!(pub fn calculate_gold(ctx: &Ctx, pnum: usize) -> i32, "inv.cpp|devilution::CalculateGold(Player &player)");
crate::pending_fn!(pub fn can_put(ctx: &Ctx, position: Point) -> bool, "inv.cpp|devilution::CanPut(Point position)");
crate::pending_fn!(pub fn find_get_item(ctx: &Ctx, iseed: u32, idx: _item_indexes, ci: u16) -> i32, "inv.cpp|devilution::FindGetItem(uint32_t iseed, _item_indexes idx, uint16_t createInfo)");
crate::pending_fn!(pub fn gold_auto_place(ctx: &mut Ctx, pnum: usize, gold_stack: &mut Item) -> bool, "inv.cpp|devilution::GoldAutoPlace(Player &player, Item &goldStack)");
crate::pending_fn!(pub fn remove_equipment(ctx: &mut Ctx, pnum: usize, body_location: inv_body_loc, hi_pri: bool), "inv.cpp|devilution::RemoveEquipment(Player &player, inv_body_loc bodyLocation, bool hiPri)");
crate::pending_fn!(pub fn consume_scroll(ctx: &mut Ctx, pnum: usize), "inv.cpp|devilution::ConsumeScroll(Player &player)");
crate::pending_fn!(pub fn consume_staff_charge(ctx: &mut Ctx, pnum: usize), "inv.cpp|devilution::ConsumeStaffCharge(Player &player)");
crate::pending_fn!(pub fn can_use_scroll(ctx: &Ctx, pnum: usize, spell: SpellID) -> bool, "inv.cpp|devilution::CanUseScroll(Player &player, SpellID spell)");
crate::pending_fn!(pub fn can_use_staff(ctx: &Ctx, pnum: usize, spell: SpellID) -> bool, "inv.cpp|devilution::CanUseStaff(Player &player, SpellID spell)");

/// `belt_item_type`
pub const BLT_HEALING: i32 = 0;
pub const BLT_MANA: i32 = 1;

crate::pending_fn!(pub fn use_belt_item_slot(ctx: &mut Ctx, i: usize), "diablo.cpp|devilution::InitKeymapActions() BeltItem lambda");

crate::pending_fn!(pub fn use_belt_item(ctx: &mut Ctx, type_: i32), "inv.cpp|devilution::UseBeltItem(int type)");

/// Original: `devilution::ClampDurability` (inv.cpp).
// @port inv.cpp|devilution::ClampDurability(const Item &item, int durability) sha=82377ac66375
pub fn clamp_durability(item: &Item, durability: i32) -> i32 {
    if item._iMaxDur == 0 {
        return 0;
    }
    durability.clamp(1, item._iMaxDur)
}
