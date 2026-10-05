//! `Source/inv.cpp`: player inventory, belt and equipment slots.
//!
//! Functions taking a player take its index (`pnum`). Item lists are edited in place in
//! `ctx.players.Players[pnum]`; where the original passed a reference to an item that lives in
//! another container while also touching the context, the item is cloned first.

use crate::ctx::Ctx;
use crate::cursor::{get_inv_item_size, new_cursor, new_cursor_item, CURSOR_FIRSTITEM, CURSOR_HAND};
use crate::effects::play_sfx;
use crate::effects_data::*;
use crate::engine::backbuffer_state::{redraw_component, PanelDrawComponent};
use crate::engine::geometry::{left, opposite, right, Direction, Displacement, Point, Rectangle, Size};
use crate::enums::*;
use crate::items::Item;
use crate::levels::gendung::{in_dungeon_bounds, DungeonType};
use crate::player::{player_say, player_say_delayed, player_say_specific, InventoryGridCells, MaxBeltItems};
use crate::tables::items_tables::{ItemCAnimTbl, ItemInvSnds};

/// Globals of inv.cpp.
#[derive(Default)]
pub struct InvState {
    /// `invflag`
    pub invflag: bool,
    /// `pInvCels`
    pub p_inv_cels: Option<crate::engine::clx_sprite::ClxSpriteList>,
}

/// `INV_SLOT_SIZE_PX`
pub const INV_SLOT_SIZE_PX: i32 = 28;
/// `INV_SLOT_HALF_SIZE_PX`
pub const INV_SLOT_HALF_SIZE_PX: i32 = INV_SLOT_SIZE_PX / 2;
/// `InventorySizeInSlots`
pub const InventorySizeInSlots: Size = Size::new(10, 4);
/// `InventorySlotSizeInPixels`
pub const InventorySlotSizeInPixels: Size = Size::splat(INV_SLOT_SIZE_PX);

const fn r(x: i32, y: i32, w: i32, h: i32) -> Rectangle {
    Rectangle::new(Point::new(x, y), Size::new(w, h))
}

/// `InvRect`: maps from inventory slot to screen position (see inv.cpp for the layout).
#[rustfmt::skip]
pub static InvRect: [Rectangle; NUM_XY_SLOTS as usize] = [
    r(132, 2, 58, 59),  // helmet
    r(47, 177, 28, 29), // left ring
    r(248, 177, 28, 29), // right ring
    r(205, 32, 28, 29), // amulet
    r(17, 75, 58, 86),  // left hand
    r(248, 75, 58, 87), // right hand
    r(132, 75, 58, 87), // chest
    r(17, 222, 29, 29), r(46, 222, 29, 29), r(75, 222, 29, 29), r(104, 222, 29, 29), r(133, 222, 29, 29),
    r(162, 222, 29, 29), r(191, 222, 29, 29), r(220, 222, 29, 29), r(249, 222, 29, 29), r(278, 222, 29, 29),
    r(17, 251, 29, 29), r(46, 251, 29, 29), r(75, 251, 29, 29), r(104, 251, 29, 29), r(133, 251, 29, 29),
    r(162, 251, 29, 29), r(191, 251, 29, 29), r(220, 251, 29, 29), r(249, 251, 29, 29), r(278, 251, 29, 29),
    r(17, 280, 29, 29), r(46, 280, 29, 29), r(75, 280, 29, 29), r(104, 280, 29, 29), r(133, 280, 29, 29),
    r(162, 280, 29, 29), r(191, 280, 29, 29), r(220, 280, 29, 29), r(249, 280, 29, 29), r(278, 280, 29, 29),
    r(17, 309, 29, 29), r(46, 309, 29, 29), r(75, 309, 29, 29), r(104, 309, 29, 29), r(133, 309, 29, 29),
    r(162, 309, 29, 29), r(191, 309, 29, 29), r(220, 309, 29, 29), r(249, 309, 29, 29), r(278, 309, 29, 29),
    r(205, 5, 29, 29), r(234, 5, 29, 29), r(263, 5, 29, 29), r(292, 5, 29, 29), // belt
    r(321, 5, 29, 29), r(350, 5, 29, 29), r(379, 5, 29, 29), r(408, 5, 29, 29), // belt
];

/// `belt_item_type`
pub const BLT_HEALING: i32 = 0;
pub const BLT_MANA: i32 = 1;

fn is_my_player(ctx: &Ctx, pnum: usize) -> bool {
    ctx.players.MyPlayer == Some(pnum)
}

fn my_player(ctx: &Ctx) -> usize {
    ctx.players.MyPlayer.expect("MyPlayer")
}

fn panel_pos(r: crate::engine::surface::Rect) -> Point {
    Point::new(r.x, r.y)
}

/// `PlaySFX(ItemInvSnds[ItemCAnimTbl[curs]])`
fn play_item_inv_sfx(ctx: &mut Ctx, curs: u8) {
    play_sfx(ctx, ItemInvSnds[ItemCAnimTbl[curs as usize] as usize]);
}

/// Original: `AddItemToInvGrid` (inv.cpp).
// @port inv.cpp|devilution::AddItemToInvGrid(Player &player, int invGridIndex, int invListIndex, Size itemSize) sha=915212e2ee33
fn add_item_to_inv_grid(ctx: &mut Ctx, pnum: usize, inv_grid_index: i32, inv_list_index: i32, item_size: Size) {
    const PITCH: i32 = 10;
    let player = &mut ctx.players.Players[pnum];
    for y in 0..item_size.height {
        let row_grid_index = inv_grid_index + PITCH * y;
        for x in 0..item_size.width {
            let v = if x == 0 && y == item_size.height - 1 { inv_list_index } else { -inv_list_index };
            player.InvGrid[(row_grid_index + x) as usize] = v as i8;
        }
    }
    if is_my_player(ctx, pnum) {
        crate::msg::net_send_cmd_ch_inv_item(ctx, false, inv_grid_index);
    }
}

/// Original: `FitsInBeltSlot` (inv.cpp).
// @port inv.cpp|devilution::FitsInBeltSlot(const Item &item) sha=a4b02fe75a4c
fn fits_in_belt_slot(item: &Item) -> bool {
    get_inventory_size(item) == Size::new(1, 1)
}

/// Original: `CanEquip(const Item &item)` (inv.cpp).
// @port inv.cpp|devilution::CanEquip(const Item &item) sha=d6fedd7bba56
fn can_equip_item(item: &Item) -> bool {
    item.is_equipment() && item._iStatFlag
}

/// Original: `CanWield` (inv.cpp).
// @port inv.cpp|devilution::CanWield(Player &player, const Item &item) sha=e209881ac89c
fn can_wield(ctx: &Ctx, pnum: usize, item: &Item) -> bool {
    let player = &ctx.players.Players[pnum];
    if !can_equip_item(item) || !matches!(player.get_item_location(item), ILOC_ONEHAND | ILOC_TWOHAND) {
        return false;
    }
    let left_hand_item = &player.InvBody[INVLOC_HAND_LEFT as usize];
    let right_hand_item = &player.InvBody[INVLOC_HAND_RIGHT as usize];
    if left_hand_item.is_empty() && right_hand_item.is_empty() {
        return true;
    }
    if !left_hand_item.is_empty() && !right_hand_item.is_empty() {
        return false;
    }
    let occupied_hand = if !left_hand_item.is_empty() { left_hand_item } else { right_hand_item };
    if player._pClass == HeroClass::Bard {
        let occupied_ok =
            player.get_item_location(occupied_hand) == ILOC_ONEHAND && matches!(occupied_hand._itype, ItemType::Sword | ItemType::Mace);
        let equip_ok = player.get_item_location(item) == ILOC_ONEHAND && matches!(item._itype, ItemType::Sword | ItemType::Mace);
        if occupied_ok && equip_ok {
            return true;
        }
    }
    player.get_item_location(item) == ILOC_ONEHAND
        && player.get_item_location(occupied_hand) == ILOC_ONEHAND
        && item._iClass != occupied_hand._iClass
}

/// Original: `CanEquip(Player &player, const Item &item, inv_body_loc bodyLocation)` (inv.cpp).
// @port inv.cpp|devilution::CanEquip(Player &player, const Item &item, inv_body_loc bodyLocation) sha=a0e255c2b785
fn can_equip(ctx: &Ctx, pnum: usize, item: &Item, body_location: inv_body_loc) -> bool {
    let player = &ctx.players.Players[pnum];
    if !can_equip_item(item) || player._pmode > PM_WALK_SIDEWAYS || !player.InvBody[body_location as usize].is_empty() {
        return false;
    }
    match body_location {
        INVLOC_AMULET => item._iLoc == ILOC_AMULET,
        INVLOC_CHEST => item._iLoc == ILOC_ARMOR,
        INVLOC_HAND_LEFT | INVLOC_HAND_RIGHT => can_wield(ctx, pnum, item),
        INVLOC_HEAD => item._iLoc == ILOC_HELM,
        INVLOC_RING_LEFT | INVLOC_RING_RIGHT => item._iLoc == ILOC_RING,
        _ => false,
    }
}

/// Original: `ChangeEquipment` (inv.cpp).
// @port inv.cpp|devilution::ChangeEquipment(Player &player, inv_body_loc bodyLocation, const Item &item) sha=b476b7829d74
fn change_equipment(ctx: &mut Ctx, pnum: usize, body_location: inv_body_loc, item: Item) {
    ctx.players.Players[pnum].InvBody[body_location as usize] = item;
    if is_my_player(ctx, pnum) {
        crate::msg::net_send_cmd_ch_item(ctx, false, body_location as u8, true);
    }
}

/// Original: `AutoEquip(Player &player, const Item &item, inv_body_loc bodyLocation, bool persistItem)` (inv.cpp).
// @port inv.cpp|devilution::AutoEquip(Player &player, const Item &item, inv_body_loc bodyLocation, bool persistItem) sha=dfb78b6c0866
fn auto_equip_at(ctx: &mut Ctx, pnum: usize, item: &Item, body_location: inv_body_loc, persist_item: bool) -> bool {
    if !can_equip(ctx, pnum, item, body_location) {
        return false;
    }
    if persist_item {
        change_equipment(ctx, pnum, body_location, item.clone());
        if ctx.options.audio.auto_equip_sound.get() && is_my_player(ctx, pnum) {
            play_item_inv_sfx(ctx, item._iCurs);
        }
        crate::items::calc_plr_inv(ctx, pnum, true);
    }
    true
}

/// Original: `FindTargetSlotUnderItemCursor` (inv.cpp).
// @port inv.cpp|devilution::FindTargetSlotUnderItemCursor(Point cursorPosition, Size itemSize) sha=52fb55e3d183
fn find_target_slot_under_item_cursor(ctx: &Ctx, cursor_position: Point, item_size: Size) -> i32 {
    let rp = panel_pos(crate::control::get_right_panel(ctx));
    let mut panel_offset = Point::new(0, 0) - rp;
    for r in SLOTXY_EQUIPPED_FIRST..=SLOTXY_EQUIPPED_LAST {
        if InvRect[r as usize].contains(cursor_position + panel_offset) {
            return r as i32;
        }
    }
    for r in SLOTXY_INV_FIRST..=SLOTXY_INV_LAST {
        if InvRect[r as usize].contains(cursor_position + panel_offset) {
            if item_size.height <= 1 && item_size.width <= 1 {
                return r as i32;
            }
            let mut hot = Displacement::new((item_size.width - 1) / 2, (item_size.height - 1) / 2);
            if item_size.width % 2 == 0 && InvRect[r as usize].contains(cursor_position + panel_offset + Displacement::new(INV_SLOT_HALF_SIZE_PX, 0)) {
                hot.delta_x += 1;
            }
            if item_size.height % 2 == 0 && InvRect[r as usize].contains(cursor_position + panel_offset + Displacement::new(0, INV_SLOT_HALF_SIZE_PX)) {
                hot.delta_y += 1;
            }
            let hot_pixel_cell = r as i32 - SLOTXY_INV_FIRST as i32;
            let target_row = ((hot_pixel_cell / InventorySizeInSlots.width) - hot.delta_y).clamp(0, InventorySizeInSlots.height - item_size.height);
            let target_column = ((hot_pixel_cell % InventorySizeInSlots.width) - hot.delta_x).clamp(0, InventorySizeInSlots.width - item_size.width);
            return SLOTXY_INV_FIRST as i32 + target_row * InventorySizeInSlots.width + target_column;
        }
    }
    let mp = panel_pos(crate::control::get_main_panel(ctx));
    panel_offset = Point::new(0, 0) - mp;
    for r in SLOTXY_BELT_FIRST..=SLOTXY_BELT_LAST {
        if InvRect[r as usize].contains(cursor_position + panel_offset) {
            return r as i32;
        }
    }
    NUM_XY_SLOTS as i32
}

/// Original: `CheckInvPaste` (inv.cpp).
// @port inv.cpp|devilution::CheckInvPaste(Player &player, Point cursorPosition) sha=d6dc96564f0e
fn check_inv_paste(ctx: &mut Ctx, pnum: usize, cursor_position: Point) {
    let item_size = get_inventory_size(&ctx.players.Players[pnum].HoldItem);
    let slot = find_target_slot_under_item_cursor(ctx, cursor_position, item_size);
    if slot == NUM_XY_SLOTS as i32 {
        return;
    }
    let mut il = ILOC_UNEQUIPABLE;
    if slot == SLOTXY_HEAD as i32 {
        il = ILOC_HELM;
    }
    if slot == SLOTXY_RING_LEFT as i32 || slot == SLOTXY_RING_RIGHT as i32 {
        il = ILOC_RING;
    }
    if slot == SLOTXY_AMULET as i32 {
        il = ILOC_AMULET;
    }
    if slot == SLOTXY_HAND_LEFT as i32 || slot == SLOTXY_HAND_RIGHT as i32 {
        il = ILOC_ONEHAND;
    }
    if slot == SLOTXY_CHEST as i32 {
        il = ILOC_ARMOR;
    }
    if slot >= SLOTXY_BELT_FIRST as i32 && slot <= SLOTXY_BELT_LAST as i32 {
        il = ILOC_BELT;
    }
    let desired_il = {
        let p = &ctx.players.Players[pnum];
        p.get_item_location(&p.HoldItem)
    };
    if il == ILOC_ONEHAND && desired_il == ILOC_TWOHAND {
        il = ILOC_TWOHAND;
    }

    let mut it: i8 = 0;
    if il == ILOC_UNEQUIPABLE {
        let player = &ctx.players.Players[pnum];
        let ii = (slot - SLOTXY_INV_FIRST as i32) as usize;
        if player.HoldItem._itype == ItemType::Gold {
            if player.InvGrid[ii] != 0 {
                let iv = player.InvGrid[ii];
                if iv > 0 {
                    if player.InvList[(iv - 1) as usize]._itype != ItemType::Gold {
                        it = iv;
                    }
                } else {
                    it = iv.wrapping_neg();
                }
            }
        } else {
            let origin_cell = (slot - SLOTXY_INV_FIRST as i32) as u32;
            let mut row_offset = 0u32;
            while row_offset < (item_size.height * InventorySizeInSlots.width) as u32 {
                for column_offset in 0..item_size.width as u32 {
                    let test_cell = (origin_cell + row_offset + column_offset) as usize;
                    assert!(test_cell < InventoryGridCells);
                    if player.InvGrid[test_cell] != 0 {
                        let iv = player.InvGrid[test_cell].wrapping_abs();
                        if it != 0 {
                            if it != iv {
                                return;
                            }
                        } else {
                            it = iv;
                        }
                    }
                }
                row_offset += InventorySizeInSlots.width as u32;
            }
        }
    } else if il == ILOC_BELT {
        let hold = ctx.players.Players[pnum].HoldItem.clone();
        if !can_be_placed_on_belt(ctx, &hold) {
            return;
        }
    } else if desired_il != il {
        return;
    }

    if !matches!(il, ILOC_UNEQUIPABLE | ILOC_BELT) {
        let p = &ctx.players.Players[pnum];
        if !p.can_use_item(&p.HoldItem) {
            player_say(ctx, pnum, HeroSpeech::ICantUseThisYet);
            return;
        }
    }
    if ctx.players.Players[pnum]._pmode > PM_WALK_SIDEWAYS && !matches!(il, ILOC_UNEQUIPABLE | ILOC_BELT) {
        return;
    }
    if is_my_player(ctx, pnum) {
        let c = ctx.players.Players[pnum].HoldItem._iCurs;
        play_item_inv_sfx(ctx, c);
    }

    match il {
        ILOC_HELM | ILOC_RING | ILOC_AMULET | ILOC_ARMOR => {
            let slot_loc = match il {
                ILOC_HELM => INVLOC_HEAD,
                ILOC_RING => {
                    if slot == SLOTXY_RING_LEFT as i32 {
                        INVLOC_RING_LEFT
                    } else {
                        INVLOC_RING_RIGHT
                    }
                }
                ILOC_AMULET => INVLOC_AMULET,
                ILOC_ARMOR => INVLOC_CHEST,
                _ => crate::appfat::app_fatal(ctx, "Unexpected equipment type"),
            };
            let previously_equipped_item = ctx.players.Players[pnum].InvBody[slot_loc as usize].clone();
            let held = ctx.players.Players[pnum].HoldItem.pop();
            change_equipment(ctx, pnum, slot_loc, held);
            if !previously_equipped_item.is_empty() {
                ctx.players.Players[pnum].HoldItem = previously_equipped_item;
            }
        }
        ILOC_ONEHAND => {
            let selected_hand = if slot == SLOTXY_HAND_LEFT as i32 { INVLOC_HAND_LEFT } else { INVLOC_HAND_RIGHT };
            let other_hand = if slot == SLOTXY_HAND_LEFT as i32 { INVLOC_HAND_RIGHT } else { INVLOC_HAND_LEFT };
            let (paste_into_selected_hand, dequip_two_handed_weapon) = {
                let p = &ctx.players.Players[pnum];
                let other = &p.InvBody[other_hand as usize];
                let paste = (other.is_empty() || other._iClass != p.HoldItem._iClass)
                    || (p._pClass == HeroClass::Bard && other._iClass == ICLASS_WEAPON && p.HoldItem._iClass == ICLASS_WEAPON);
                let dequip = !other.is_empty() && p.get_item_location(other) == ILOC_TWOHAND;
                (paste, dequip)
            };
            let paste_hand = if paste_into_selected_hand { selected_hand } else { other_hand };
            let previously_equipped_item = if dequip_two_handed_weapon {
                ctx.players.Players[pnum].InvBody[other_hand as usize].clone()
            } else {
                ctx.players.Players[pnum].InvBody[paste_hand as usize].clone()
            };
            if dequip_two_handed_weapon {
                remove_equipment(ctx, pnum, other_hand, false);
            }
            let held = ctx.players.Players[pnum].HoldItem.pop();
            change_equipment(ctx, pnum, paste_hand, held);
            if !previously_equipped_item.is_empty() {
                ctx.players.Players[pnum].HoldItem = previously_equipped_item;
            }
        }
        ILOC_TWOHAND => {
            let both = {
                let p = &ctx.players.Players[pnum];
                !p.InvBody[INVLOC_HAND_LEFT as usize].is_empty() && !p.InvBody[INVLOC_HAND_RIGHT as usize].is_empty()
            };
            if both {
                let mut location_to_unequip = INVLOC_HAND_LEFT;
                if ctx.players.Players[pnum].InvBody[INVLOC_HAND_RIGHT as usize]._itype == ItemType::Shield {
                    location_to_unequip = INVLOC_HAND_RIGHT;
                }
                let it2 = ctx.players.Players[pnum].InvBody[location_to_unequip as usize].clone();
                let done2h = auto_place_item_in_inventory(ctx, pnum, &it2, true);
                if !done2h {
                    return;
                }
                if location_to_unequip == INVLOC_HAND_RIGHT {
                    remove_equipment(ctx, pnum, INVLOC_HAND_RIGHT, false);
                } else {
                    ctx.players.Players[pnum].InvBody[INVLOC_HAND_LEFT as usize].clear();
                }
            }
            if ctx.players.Players[pnum].InvBody[INVLOC_HAND_RIGHT as usize].is_empty() {
                let previously_equipped_item = ctx.players.Players[pnum].InvBody[INVLOC_HAND_LEFT as usize].clone();
                let held = ctx.players.Players[pnum].HoldItem.pop();
                change_equipment(ctx, pnum, INVLOC_HAND_LEFT, held);
                if !previously_equipped_item.is_empty() {
                    ctx.players.Players[pnum].HoldItem = previously_equipped_item;
                }
            } else {
                let previously_equipped_item = ctx.players.Players[pnum].InvBody[INVLOC_HAND_RIGHT as usize].clone();
                remove_equipment(ctx, pnum, INVLOC_HAND_RIGHT, false);
                let held = ctx.players.Players[pnum].HoldItem.clone();
                change_equipment(ctx, pnum, INVLOC_HAND_LEFT, held);
                ctx.players.Players[pnum].HoldItem = previously_equipped_item;
            }
        }
        ILOC_UNEQUIPABLE => {
            let max_gold = ctx.items.MaxGold;
            if ctx.players.Players[pnum].HoldItem._itype == ItemType::Gold && it == 0 {
                let ii = (slot - SLOTXY_INV_FIRST as i32) as usize;
                let player = &mut ctx.players.Players[pnum];
                if player.InvGrid[ii] > 0 {
                    let inv_index = (player.InvGrid[ii] - 1) as usize;
                    let gt = player.InvList[inv_index]._ivalue;
                    let mut ig = player.HoldItem._ivalue + gt;
                    if ig <= max_gold {
                        player.InvList[inv_index]._ivalue = ig;
                        crate::items::set_plr_hand_gold_curs(&mut player.InvList[inv_index]);
                        player._pGold += player.HoldItem._ivalue;
                        player.HoldItem.clear();
                    } else {
                        ig = max_gold - gt;
                        player._pGold += ig;
                        player.HoldItem._ivalue -= ig;
                        crate::items::set_plr_hand_gold_curs(&mut player.HoldItem);
                        player.InvList[inv_index]._ivalue = max_gold;
                        player.InvList[inv_index]._iCurs = ICURS_GOLD_LARGE as u8;
                    }
                } else {
                    let inv_index = player._pNumInv as usize;
                    player._pGold += player.HoldItem._ivalue;
                    player.InvList[inv_index] = player.HoldItem.pop();
                    player._pNumInv += 1;
                    player.InvGrid[ii] = player._pNumInv as i8;
                }
                if is_my_player(ctx, pnum) {
                    crate::msg::net_send_cmd_ch_inv_item(ctx, false, ii as i32);
                }
            } else {
                let player = &mut ctx.players.Players[pnum];
                if it == 0 {
                    let n = player._pNumInv as usize;
                    player.InvList[n] = player.HoldItem.pop();
                    player._pNumInv += 1;
                    it = player._pNumInv as i8;
                } else {
                    let inv_index = (it - 1) as usize;
                    if player.HoldItem._itype == ItemType::Gold {
                        player._pGold += player.HoldItem._ivalue;
                    }
                    std::mem::swap(&mut player.InvList[inv_index], &mut player.HoldItem);
                    if player.HoldItem._itype == ItemType::Gold {
                        let g = calculate_gold(ctx, pnum);
                        ctx.players.Players[pnum]._pGold = g;
                    }
                    let player = &mut ctx.players.Players[pnum];
                    for item_index in player.InvGrid.iter_mut() {
                        if *item_index == it {
                            *item_index = 0;
                        }
                        if *item_index == it.wrapping_neg() {
                            *item_index = 0;
                        }
                    }
                }
                add_item_to_inv_grid(ctx, pnum, slot - SLOTXY_INV_FIRST as i32, it as i32, item_size);
            }
        }
        ILOC_BELT => {
            let ii = (slot - SLOTXY_BELT_FIRST as i32) as usize;
            let player = &mut ctx.players.Players[pnum];
            if player.SpdList[ii].is_empty() {
                player.SpdList[ii] = player.HoldItem.pop();
            } else {
                std::mem::swap(&mut player.SpdList[ii], &mut player.HoldItem);
                if player.HoldItem._itype == ItemType::Gold {
                    let g = calculate_gold(ctx, pnum);
                    ctx.players.Players[pnum]._pGold = g;
                }
            }
            if is_my_player(ctx, pnum) {
                crate::msg::net_send_cmd_ch_belt_item(ctx, false, ii as i32);
            }
            redraw_component(ctx, PanelDrawComponent::Belt);
        }
        _ => {}
    }
    crate::items::calc_plr_inv(ctx, pnum, true);
    if is_my_player(ctx, pnum) {
        let h = &ctx.players.Players[pnum].HoldItem;
        let (e, c) = (h.is_empty(), h._iCurs);
        new_cursor_item(ctx, e, c);
    }
}

/// `CheckInvCut` helper for the equipment slots: the original repeats this block per slot.
fn cut_body_slot(ctx: &mut Ctx, pnum: usize, loc: inv_body_loc, automatic_move: bool, moved: &mut bool, equipped: &mut bool, unequip: &mut bool) {
    let item = ctx.players.Players[pnum].InvBody[loc as usize].clone();
    if item.is_empty() {
        return;
    }
    ctx.players.Players[pnum].HoldItem = item.clone();
    if automatic_move {
        *unequip = true;
        *moved = auto_place_item_in_inventory(ctx, pnum, &item, true);
        *equipped = *moved;
    }
    if !automatic_move || *moved {
        remove_equipment(ctx, pnum, loc, false);
    }
}

/// Original: `CheckInvCut` (inv.cpp).
// @port inv.cpp|devilution::CheckInvCut(Player &player, Point cursorPosition, bool automaticMove, bool dropItem) sha=8ef208ed7bd1
fn check_inv_cut(ctx: &mut Ctx, pnum: usize, cursor_position: Point, automatic_move: bool, drop_item: bool) {
    if ctx.players.Players[pnum]._pmode > PM_WALK_SIDEWAYS {
        return;
    }
    if ctx.control.drop_gold_flag {
        crate::control::close_gold_drop(ctx);
        ctx.control.drop_gold_value = 0;
    }
    let mut r = 0u32;
    while r < NUM_XY_SLOTS as u32 {
        let mut o = panel_pos(crate::control::get_right_panel(ctx));
        if r >= SLOTXY_BELT_FIRST as u32 {
            o = panel_pos(crate::control::get_main_panel(ctx));
        }
        if InvRect[r as usize].contains(cursor_position - Displacement::new(o.x, o.y)) {
            break;
        }
        r += 1;
    }
    if r == NUM_XY_SLOTS as u32 {
        return;
    }

    ctx.players.Players[pnum].HoldItem.clear();
    let mut automatically_moved = false;
    let mut automatically_equipped = false;
    let mut automatically_unequip = false;

    for (slot, loc) in [
        (SLOTXY_HEAD, INVLOC_HEAD),
        (SLOTXY_RING_LEFT, INVLOC_RING_LEFT),
        (SLOTXY_RING_RIGHT, INVLOC_RING_RIGHT),
        (SLOTXY_AMULET, INVLOC_AMULET),
        (SLOTXY_HAND_LEFT, INVLOC_HAND_LEFT),
        (SLOTXY_HAND_RIGHT, INVLOC_HAND_RIGHT),
        (SLOTXY_CHEST, INVLOC_CHEST),
    ] {
        if r == slot as u32 {
            cut_body_slot(ctx, pnum, loc, automatic_move, &mut automatically_moved, &mut automatically_equipped, &mut automatically_unequip);
        }
    }

    if r >= SLOTXY_INV_FIRST as u32 && r <= SLOTXY_INV_LAST as u32 {
        let ig = (r - SLOTXY_INV_FIRST as u32) as usize;
        let ii = ctx.players.Players[pnum].InvGrid[ig];
        if ii != 0 {
            let iv = if ii < 0 { -(ii as i32) } else { ii as i32 };
            let ivu = (iv - 1) as usize;
            ctx.players.Players[pnum].HoldItem = ctx.players.Players[pnum].InvList[ivu].clone();
            if automatic_move {
                let hold = ctx.players.Players[pnum].HoldItem.clone();
                if can_be_placed_on_belt(ctx, &hold) {
                    automatically_moved = auto_place_item_in_belt(ctx, pnum, &hold, true);
                } else if can_equip_item(&hold) {
                    automatically_unequip = true;
                    let mut invloc = NUM_INVLOC as i32;
                    let loc = ctx.players.Players[pnum].get_item_location(&hold);
                    match loc {
                        ILOC_ARMOR => invloc = INVLOC_CHEST as i32,
                        ILOC_HELM => invloc = INVLOC_HEAD as i32,
                        ILOC_AMULET => invloc = INVLOC_AMULET as i32,
                        ILOC_ONEHAND => {
                            let p = &ctx.players.Players[pnum];
                            let inv_item = &p.InvList[ivu];
                            let lh = &p.InvBody[INVLOC_HAND_LEFT as usize];
                            let rh = &p.InvBody[INVLOC_HAND_RIGHT as usize];
                            if inv_item._iClass == lh._iClass && p.get_item_location(inv_item) == p.get_item_location(lh) {
                                invloc = INVLOC_HAND_LEFT as i32;
                            }
                            if inv_item._iClass == rh._iClass && p.get_item_location(inv_item) == p.get_item_location(rh) {
                                invloc = INVLOC_HAND_RIGHT as i32;
                            }
                            if p.get_item_location(lh) == ILOC_TWOHAND {
                                invloc = INVLOC_HAND_LEFT as i32;
                            }
                        }
                        ILOC_TWOHAND => 'twohand: {
                            if !ctx.players.Players[pnum].InvBody[INVLOC_HAND_RIGHT as usize].is_empty() {
                                let rh = ctx.players.Players[pnum].InvBody[INVLOC_HAND_RIGHT as usize].clone();
                                ctx.players.Players[pnum].HoldItem = rh.clone();
                                if !auto_place_item_in_inventory(ctx, pnum, &rh, true) {
                                    break 'twohand;
                                }
                                let lh = ctx.players.Players[pnum].InvBody[INVLOC_HAND_LEFT as usize].clone();
                                ctx.players.Players[pnum].HoldItem = lh.clone();
                                if !auto_place_item_in_inventory(ctx, pnum, &lh, false) {
                                    let p = &mut ctx.players.Players[pnum];
                                    let last = (p._pNumInv - 1) as usize;
                                    p.InvBody[INVLOC_HAND_RIGHT as usize] = p.InvList[last].clone();
                                    let n = p._pNumInv - 1;
                                    crate::player::remove_inv_item(ctx, pnum, n, false);
                                    break 'twohand;
                                }
                                remove_equipment(ctx, pnum, INVLOC_HAND_RIGHT, false);
                                invloc = INVLOC_HAND_LEFT as i32;
                            } else {
                                invloc = INVLOC_HAND_LEFT as i32;
                            }
                        }
                        _ => automatically_unequip = false,
                    }
                    if invloc != NUM_INVLOC as i32 {
                        let body = ctx.players.Players[pnum].InvBody[invloc as usize].clone();
                        ctx.players.Players[pnum].HoldItem = body.clone();
                        if body._itype != ItemType::None && auto_place_item_in_inventory(ctx, pnum, &body, true) {
                            ctx.players.Players[pnum].InvBody[invloc as usize].clear();
                        }
                    }
                    let hold = ctx.players.Players[pnum].InvList[ivu].clone();
                    ctx.players.Players[pnum].HoldItem = hold.clone();
                    automatically_moved = auto_equip(ctx, pnum, &hold, true);
                    automatically_equipped = automatically_moved;
                }
            }
            if !automatic_move || automatically_moved {
                crate::player::remove_inv_item(ctx, pnum, iv - 1, false);
            }
        }
    }

    if r >= SLOTXY_BELT_FIRST as u32 {
        let bi = (r - SLOTXY_BELT_FIRST as u32) as usize;
        let belt_item = ctx.players.Players[pnum].SpdList[bi].clone();
        if !belt_item.is_empty() {
            ctx.players.Players[pnum].HoldItem = belt_item.clone();
            if automatic_move {
                automatically_moved = auto_place_item_in_inventory(ctx, pnum, &belt_item, true);
            }
            if !automatic_move || automatically_moved {
                crate::player::remove_spd_bar_item(ctx, pnum, bi as i32);
            }
        }
    }

    if !ctx.players.Players[pnum].HoldItem.is_empty() {
        if ctx.players.Players[pnum].HoldItem._itype == ItemType::Gold {
            ctx.players.Players[pnum]._pGold = calculate_gold(ctx, pnum);
        }
        crate::items::calc_plr_inv(ctx, pnum, true);
        {
            let p = &mut ctx.players.Players[pnum];
            p.HoldItem._iStatFlag = p.can_use_item(&p.HoldItem);
        }
        if is_my_player(ctx, pnum) {
            let hold = ctx.players.Players[pnum].HoldItem.clone();
            if automatically_equipped {
                play_item_inv_sfx(ctx, hold._iCurs);
            } else if !automatic_move || automatically_moved {
                play_sfx(ctx, IS_IGRAB);
            }
            if automatic_move {
                if !automatically_moved {
                    if can_be_placed_on_belt(ctx, &hold) || automatically_unequip {
                        player_say_specific(ctx, pnum, HeroSpeech::IHaveNoRoom);
                    } else {
                        player_say_specific(ctx, pnum, HeroSpeech::ICantDoThat);
                    }
                }
                ctx.players.Players[pnum].HoldItem.clear();
            } else {
                new_cursor_item(ctx, hold.is_empty(), hold._iCurs);
            }
        }
    }

    if drop_item && !ctx.players.Players[pnum].HoldItem.is_empty() {
        crate::controls::plrctrls::try_drop_item(ctx);
    }
}

/// `HasInventoryItemWithId` (inv.h): the inventory range skips empty items.
// @port inv.h|devilution::HasInventoryItemWithId(Player &player, _item_indexes id) sha=53deeeb17fac
pub fn has_inventory_item_with_id(ctx: &Ctx, pnum: usize, id: _item_indexes) -> bool {
    let p = &ctx.players.Players[pnum];
    p.InvList[..p._pNumInv as usize].iter().any(|i| !i.is_empty() && i.IDidx == id)
}

/// `HasInventoryOrBeltItem` (inv.h)
// @port inv.h|devilution::HasInventoryOrBeltItem(Player &player, Predicate &&predicate) sha=c8db1c360421
pub fn has_inventory_or_belt_item(ctx: &Ctx, pnum: usize, pred: impl Fn(&Item) -> bool) -> bool {
    let p = &ctx.players.Players[pnum];
    p.InvList[..p._pNumInv as usize].iter().any(|i| !i.is_empty() && pred(i)) || p.SpdList.iter().any(|i| !i.is_empty() && pred(i))
}

/// `RemoveInventoryItem` (inv.h)
// @port inv.h|devilution::RemoveInventoryItem(Player &player, Predicate &&predicate) sha=53f17008c059
pub fn remove_inventory_item(ctx: &mut Ctx, pnum: usize, pred: impl Fn(&Item) -> bool) -> bool {
    let p = &ctx.players.Players[pnum];
    let Some(i) = p.InvList[..p._pNumInv as usize].iter().position(|i| !i.is_empty() && pred(i)) else { return false };
    crate::player::remove_inv_item(ctx, pnum, i as i32, true);
    true
}

/// `RemoveBeltItem` (inv.h)
// @port inv.h|devilution::RemoveBeltItem(Player &player, Predicate &&predicate) sha=6afe0be792d0
pub fn remove_belt_item(ctx: &mut Ctx, pnum: usize, pred: impl Fn(&Item) -> bool) -> bool {
    let Some(i) = ctx.players.Players[pnum].SpdList.iter().position(|i| !i.is_empty() && pred(i)) else { return false };
    crate::player::remove_spd_bar_item(ctx, pnum, i as i32);
    true
}

/// `RemoveInventoryOrBeltItem` (inv.h)
// @port inv.h|devilution::RemoveInventoryOrBeltItem(Player &player, Predicate &&predicate) sha=412e26950379
pub fn remove_inventory_or_belt_item(ctx: &mut Ctx, pnum: usize, pred: impl Fn(&Item) -> bool) -> bool {
    remove_inventory_item(ctx, pnum, &pred) || remove_belt_item(ctx, pnum, &pred)
}

/// `RemoveInventoryItemById` (inv.h)
// @port inv.h|devilution::RemoveInventoryItemById(Player &player, _item_indexes id) sha=fc5d7a903557
pub fn remove_inventory_item_by_id(ctx: &mut Ctx, pnum: usize, id: _item_indexes) -> bool {
    remove_inventory_item(ctx, pnum, |i| i.IDidx == id)
}

/// Original: `TryCombineNaKrulNotes` (inv.cpp). `note_item` is the item being picked up.
// @port inv.cpp|devilution::TryCombineNaKrulNotes(Player &player, Item &noteItem) sha=bfac8abd8580
fn try_combine_na_krul_notes(ctx: &mut Ctx, pnum: usize, note_item: &mut Item) {
    let idx = note_item.IDidx;
    let notes = [IDI_NOTE1, IDI_NOTE2, IDI_NOTE3];
    if !notes.contains(&idx) {
        return;
    }
    for note in notes {
        if idx != note && !has_inventory_item_with_id(ctx, pnum, note) {
            return;
        }
    }
    let me = my_player(ctx);
    player_say_delayed(ctx, me, HeroSpeech::JustWhatIWasLookingFor, 10);
    for note in notes {
        if idx != note {
            remove_inventory_item_by_id(ctx, pnum, note);
        }
    }
    let position = note_item.position;
    *note_item = Item::default();
    crate::items::get_item_attrs(ctx, note_item, IDI_FULLNOTE, 16);
    crate::items::setup_item(ctx, note_item);
    note_item.position = position;
}

/// Original: `CheckQuestItem` (inv.cpp).
// @port inv.cpp|devilution::CheckQuestItem(Player &player, Item &questItem) sha=d455866e939d
fn check_quest_item(ctx: &mut Ctx, pnum: usize, quest_item: &mut Item) {
    let me = my_player(ctx);
    let sp = ctx.gendung.SetPiece.position.mega_to_world();
    let q = |ctx: &Ctx, id: quest_id| ctx.quests.Quests[id as usize]._qactive;

    if q(ctx, Q_BLIND) == QUEST_ACTIVE
        && (quest_item.IDidx == IDI_OPTAMULET
            || (crate::quests::is_quest_available(ctx, Q_BLIND) && quest_item.position == sp + Displacement::new(5, 5)))
    {
        ctx.quests.Quests[Q_BLIND as usize]._qactive = QUEST_DONE;
        crate::msg::net_send_cmd_quest(ctx, true, Q_BLIND as usize);
    }
    if quest_item.IDidx == IDI_MUSHROOM && q(ctx, Q_MUSHROOM) == QUEST_ACTIVE && ctx.quests.Quests[Q_MUSHROOM as usize]._qvar1 as i32 == QS_MUSHSPAWNED {
        player_say_delayed(ctx, pnum, HeroSpeech::NowThatsOneBigMushroom, 10);
        ctx.quests.Quests[Q_MUSHROOM as usize]._qvar1 = QS_MUSHPICKED as u8;
        crate::msg::net_send_cmd_quest(ctx, true, Q_MUSHROOM as usize);
    }
    if quest_item.IDidx == IDI_ANVIL && q(ctx, Q_ANVIL) != QUEST_NOTAVAIL {
        if q(ctx, Q_ANVIL) == QUEST_INIT {
            ctx.quests.Quests[Q_ANVIL as usize]._qactive = QUEST_ACTIVE;
            crate::msg::net_send_cmd_quest(ctx, true, Q_ANVIL as usize);
        }
        if ctx.quests.Quests[Q_ANVIL as usize]._qlog {
            player_say_delayed(ctx, me, HeroSpeech::INeedToGetThisToGriswold, 10);
        }
    }
    if quest_item.IDidx == IDI_GLDNELIX && q(ctx, Q_VEIL) != QUEST_NOTAVAIL {
        player_say_delayed(ctx, me, HeroSpeech::INeedToGetThisToLachdanan, 30);
    }
    if quest_item.IDidx == IDI_ROCK && q(ctx, Q_ROCK) != QUEST_NOTAVAIL {
        if q(ctx, Q_ROCK) == QUEST_INIT {
            ctx.quests.Quests[Q_ROCK as usize]._qactive = QUEST_ACTIVE;
            crate::msg::net_send_cmd_quest(ctx, true, Q_ROCK as usize);
        }
        if ctx.quests.Quests[Q_ROCK as usize]._qlog {
            player_say_delayed(ctx, me, HeroSpeech::ThisMustBeWhatGriswoldWanted, 10);
        }
    }
    if q(ctx, Q_BLOOD) == QUEST_ACTIVE
        && (quest_item.IDidx == IDI_ARMOFVAL
            || (crate::quests::is_quest_available(ctx, Q_BLOOD) && quest_item.position == sp + Displacement::new(9, 3)))
    {
        ctx.quests.Quests[Q_BLOOD as usize]._qactive = QUEST_DONE;
        crate::msg::net_send_cmd_quest(ctx, true, Q_BLOOD as usize);
        player_say_delayed(ctx, me, HeroSpeech::MayTheSpiritOfArkaineProtectMe, 20);
    }
    if quest_item.IDidx == IDI_MAPOFDOOM {
        ctx.quests.Quests[Q_GRAVE as usize]._qactive = QUEST_ACTIVE;
        if ctx.quests.Quests[Q_GRAVE as usize]._qvar1 != 1 {
            player_say_delayed(ctx, me, HeroSpeech::UhHuh, 10);
            ctx.quests.Quests[Q_GRAVE as usize]._qvar1 = 1;
        }
    }
    try_combine_na_krul_notes(ctx, pnum, quest_item);
}

/// Original: `CleanupItems` (inv.cpp).
// @port inv.cpp|devilution::CleanupItems(int ii) sha=2249f539da27
fn cleanup_items(ctx: &mut Ctx, ii: i32) {
    let pos = ctx.items.Items[ii as usize].position;
    ctx.items.dItem[pos.x as usize][pos.y as usize] = 0;
    if crate::items::CornerStoneStruct::is_available(ctx) && pos == ctx.items.CornerStone.position {
        let cs = &mut ctx.items.CornerStone.item;
        cs.clear();
        cs._iSelFlag = 0;
        cs.position = Point::new(0, 0);
        cs._iAnimFlag = false;
        cs._iIdentified = false;
        cs._iPostDraw = false;
    }
    let mut i = 0;
    while i < ctx.items.ActiveItemCount as i32 {
        if ctx.items.ActiveItems[i as usize] as i32 == ii {
            crate::items::delete_item(ctx, i);
            i = 0;
            continue;
        }
        i += 1;
    }
}

/// Original: `CanUseStaff(Item &staff, SpellID spell)` (inv.cpp).
// @port inv.cpp|devilution::CanUseStaff(Item &staff, SpellID spell) sha=5869f339261d
fn can_use_staff_item(staff: &Item, spell: SpellID) -> bool {
    !staff.is_empty() && matches!(staff._iMiscId, IMISC_STAFF | IMISC_UNIQUE) && staff._iSpell == spell && staff._iCharges > 0
}

/// Original: `StartGoldDrop` (inv.cpp).
// @port inv.cpp|devilution::StartGoldDrop() sha=6220c410040d
fn start_gold_drop(ctx: &mut Ctx) {
    crate::qol::stash::close_gold_withdraw(ctx);
    let pcursinvitem = ctx.cursor.pcursinvitem;
    ctx.control.initial_drop_gold_index = pcursinvitem;
    let me = my_player(ctx);
    let p = &ctx.players.Players[me];
    ctx.control.initial_drop_gold_value = if pcursinvitem as i32 <= INVITEM_INV_LAST as i32 {
        p.InvList[(pcursinvitem as i32 - INVITEM_INV_FIRST as i32) as usize]._ivalue
    } else {
        p.SpdList[(pcursinvitem as i32 - INVITEM_BELT_FIRST as i32) as usize]._ivalue
    };
    if ctx.control.talkflag {
        crate::control::control_reset_talk(ctx);
    }
    let start = crate::control::get_panel_position(ctx, UiPanels::Inventory, Point::new(67, 128));
    ctx.platform.text_input_rect = (start.x, start.y, 180, 20);
    ctx.control.drop_gold_flag = true;
    ctx.control.drop_gold_value = 0;
    ctx.platform.start_text_input();
}

/// Original: `CreateGoldItemInInventorySlot` (inv.cpp).
// @port inv.cpp|devilution::CreateGoldItemInInventorySlot(Player &player, int slotIndex, int value) sha=21641f60c30d
fn create_gold_item_in_inventory_slot(ctx: &mut Ctx, pnum: usize, slot_index: i32, mut value: i32) -> i32 {
    if ctx.players.Players[pnum].InvGrid[slot_index as usize] != 0 {
        return value;
    }
    let n = ctx.players.Players[pnum]._pNumInv as usize;
    let mut gold_item = std::mem::take(&mut ctx.players.Players[pnum].InvList[n]);
    let v = value.min(ctx.items.MaxGold);
    crate::items::make_gold_stack(ctx, &mut gold_item, v);
    let gv = gold_item._ivalue;
    let p = &mut ctx.players.Players[pnum];
    p.InvList[n] = gold_item;
    p._pNumInv += 1;
    p.InvGrid[slot_index as usize] = p._pNumInv as i8;
    if is_my_player(ctx, pnum) {
        crate::msg::net_send_cmd_ch_inv_item(ctx, false, slot_index);
    }
    value -= gv;
    value
}


/// Original: `devilution::CanBePlacedOnBelt` (inv.cpp).
// @port inv.cpp|devilution::CanBePlacedOnBelt(const Item &item) sha=859f44f09b1a
pub fn can_be_placed_on_belt(ctx: &Ctx, item: &Item) -> bool {
    fits_in_belt_slot(item)
        && item._itype != ItemType::Gold
        && ctx.players.Players[my_player(ctx)].can_use_item(item)
        && item.is_usable(ctx)
}

/// Original: `devilution::FreeInvGFX` (inv.cpp).
// @port inv.cpp|devilution::FreeInvGFX() sha=2f0a2b8569c7
pub fn free_inv_gfx(ctx: &mut Ctx) {
    ctx.inv.p_inv_cels = None;
}

/// Original: `devilution::InitInv` (inv.cpp).
// @port inv.cpp|devilution::InitInv() sha=32268a08eff2
pub fn init_inv(ctx: &mut Ctx) {
    let me = my_player(ctx);
    let w = crate::control::SIDE_PANEL_SIZE.0 as u16;
    let name = match ctx.players.Players[me]._pClass {
        HeroClass::Warrior | HeroClass::Barbarian => "data\\inv\\inv",
        HeroClass::Rogue | HeroClass::Bard => "data\\inv\\inv_rog",
        HeroClass::Sorcerer => "data\\inv\\inv_sor",
        HeroClass::Monk => {
            if !ctx.init.gb_is_spawn {
                "data\\inv\\inv_sor"
            } else {
                "data\\inv\\inv"
            }
        }
    };
    ctx.inv.p_inv_cels = Some(crate::engine::load_sprites::load_cel(ctx, name, w));
}


/// Original: `devilution::RemoveEquipment` (inv.cpp).
// @port inv.cpp|devilution::RemoveEquipment(Player &player, inv_body_loc bodyLocation, bool hiPri) sha=06404a9a24c7
pub fn remove_equipment(ctx: &mut Ctx, pnum: usize, body_location: inv_body_loc, hi_pri: bool) {
    if is_my_player(ctx, pnum) {
        crate::msg::net_send_cmd_del_item(ctx, hi_pri, body_location as u8);
    }
    ctx.players.Players[pnum].InvBody[body_location as usize].clear();
}

/// Original: `devilution::AutoPlaceItemInBelt` (inv.cpp).
// @port inv.cpp|devilution::AutoPlaceItemInBelt(Player &player, const Item &item, bool persistItem) sha=0f957dd7d717
pub fn auto_place_item_in_belt(ctx: &mut Ctx, pnum: usize, item: &Item, persist_item: bool) -> bool {
    if !can_be_placed_on_belt(ctx, item) {
        return false;
    }
    for belt_index in 0..MaxBeltItems {
        if ctx.players.Players[pnum].SpdList[belt_index].is_empty() {
            if persist_item {
                ctx.players.Players[pnum].SpdList[belt_index] = item.clone();
                crate::player::calc_scrolls(ctx, pnum);
                redraw_component(ctx, PanelDrawComponent::Belt);
                if is_my_player(ctx, pnum) {
                    crate::msg::net_send_cmd_ch_belt_item(ctx, false, belt_index as i32);
                }
            }
            return true;
        }
    }
    false
}

/// Original: `devilution::AutoEquip(Player &player, const Item &item, bool persistItem)` (inv.cpp).
// @port inv.cpp|devilution::AutoEquip(Player &player, const Item &item, bool persistItem) sha=767ef08c3199
pub fn auto_equip(ctx: &mut Ctx, pnum: usize, item: &Item, persist_item: bool) -> bool {
    if !can_equip_item(item) {
        return false;
    }
    for body_location in INVLOC_HEAD..NUM_INVLOC {
        if auto_equip_at(ctx, pnum, item, body_location, persist_item) {
            return true;
        }
    }
    false
}

/// Original: `devilution::AutoEquipEnabled` (inv.cpp).
// @port inv.cpp|devilution::AutoEquipEnabled(const Player &player, const Item &item) sha=c46004a12b5d
pub fn auto_equip_enabled(ctx: &Ctx, pnum: usize, item: &Item) -> bool {
    let g = &ctx.options.gameplay;
    if item.is_weapon() {
        return ctx.players.Players[pnum]._pClass != HeroClass::Monk && g.auto_equip_weapons.get();
    }
    if item.is_armor() {
        return g.auto_equip_armor.get();
    }
    if item.is_helm() {
        return g.auto_equip_helms.get();
    }
    if item.is_shield() {
        return g.auto_equip_shields.get();
    }
    if item.is_jewelry() {
        return g.auto_equip_jewelry.get();
    }
    true
}

/// Original: `devilution::AutoPlaceItemInInventory` (inv.cpp).
// @port inv.cpp|devilution::AutoPlaceItemInInventory(Player &player, const Item &item, bool persistItem) sha=30407c299b08
pub fn auto_place_item_in_inventory(ctx: &mut Ctx, pnum: usize, item: &Item, persist_item: bool) -> bool {
    let item_size = get_inventory_size(item);
    if item_size.height == 1 {
        for i in 30..=39 {
            if auto_place_item_in_inventory_slot(ctx, pnum, i, item, persist_item) {
                return true;
            }
        }
        for x in (0..=9).rev() {
            for y in (0..=2).rev() {
                if auto_place_item_in_inventory_slot(ctx, pnum, 10 * y + x, item, persist_item) {
                    return true;
                }
            }
        }
        return false;
    }
    if item_size.height == 2 {
        let mut x = 10 - item_size.width;
        while x >= 0 {
            for y in 0..3 {
                if auto_place_item_in_inventory_slot(ctx, pnum, 10 * y + x, item, persist_item) {
                    return true;
                }
            }
            x -= item_size.width;
        }
        if item_size.width == 2 {
            let mut x = 7;
            while x >= 0 {
                for y in 0..3 {
                    if auto_place_item_in_inventory_slot(ctx, pnum, 10 * y + x, item, persist_item) {
                        return true;
                    }
                }
                x -= 2;
            }
        }
        return false;
    }
    if item_size == Size::new(1, 3) {
        for i in 0..20 {
            if auto_place_item_in_inventory_slot(ctx, pnum, i, item, persist_item) {
                return true;
            }
        }
        return false;
    }
    if item_size == Size::new(2, 3) {
        for i in 0..9 {
            if auto_place_item_in_inventory_slot(ctx, pnum, i, item, persist_item) {
                return true;
            }
        }
        for i in 10..19 {
            if auto_place_item_in_inventory_slot(ctx, pnum, i, item, persist_item) {
                return true;
            }
        }
        return false;
    }
    crate::appfat::app_fatal(ctx, &format!("Unknown item size: {}x{}", item_size.width, item_size.height))
}

/// Original: `devilution::AutoPlaceItemInInventorySlot` (inv.cpp).
// @port inv.cpp|devilution::AutoPlaceItemInInventorySlot(Player &player, int slotIndex, const Item &item, bool persistItem) sha=daf832e2ef26
pub fn auto_place_item_in_inventory_slot(ctx: &mut Ctx, pnum: usize, slot_index: i32, item: &Item, persist_item: bool) -> bool {
    let mut yy = if slot_index > 0 { 10 * (slot_index / 10) } else { 0 };
    let item_size = get_inventory_size(item);
    {
        let player = &ctx.players.Players[pnum];
        for _j in 0..item_size.height {
            if yy >= InventoryGridCells as i32 {
                return false;
            }
            let mut xx = if slot_index > 0 { slot_index % 10 } else { 0 };
            for _i in 0..item_size.width {
                if xx >= 10 || player.InvGrid[(xx + yy) as usize] != 0 {
                    return false;
                }
                xx += 1;
            }
            yy += 10;
        }
    }
    if persist_item {
        let p = &mut ctx.players.Players[pnum];
        let n = p._pNumInv as usize;
        p.InvList[n] = item.clone();
        p._pNumInv += 1;
        let num = p._pNumInv;
        add_item_to_inv_grid(ctx, pnum, slot_index, num, item_size);
        crate::player::calc_scrolls(ctx, pnum);
    }
    true
}

/// Original: `devilution::RoomForGold` (inv.cpp).
// @port inv.cpp|devilution::RoomForGold() sha=0a6d3919e8e1
pub fn room_for_gold(ctx: &Ctx) -> i32 {
    let max_gold = ctx.items.MaxGold;
    let p = &ctx.players.Players[my_player(ctx)];
    let mut amount = 0;
    for &item_index in p.InvGrid.iter() {
        if item_index < 0 {
            continue;
        }
        if item_index == 0 {
            amount += max_gold;
            continue;
        }
        let gold_item = &p.InvList[(item_index - 1) as usize];
        if gold_item._itype != ItemType::Gold || gold_item._ivalue == max_gold {
            continue;
        }
        amount += max_gold - gold_item._ivalue;
    }
    amount
}

/// Original: `devilution::AddGoldToInventory` (inv.cpp).
// @port inv.cpp|devilution::AddGoldToInventory(Player &player, int value) sha=975c5b037853
pub fn add_gold_to_inventory(ctx: &mut Ctx, pnum: usize, mut value: i32) -> i32 {
    let max_gold = ctx.items.MaxGold;
    let mut i = 0;
    while i < ctx.players.Players[pnum]._pNumInv && value > 0 {
        let gold_item = &mut ctx.players.Players[pnum].InvList[i as usize];
        if gold_item._itype != ItemType::Gold || gold_item._ivalue >= max_gold {
            i += 1;
            continue;
        }
        if gold_item._ivalue + value > max_gold {
            value -= max_gold - gold_item._ivalue;
            gold_item._ivalue = max_gold;
        } else {
            gold_item._ivalue += value;
            value = 0;
        }
        crate::msg::net_sync_inv_item(ctx, pnum, i);
        crate::items::set_plr_hand_gold_curs(&mut ctx.players.Players[pnum].InvList[i as usize]);
        i += 1;
    }
    let mut i = 39;
    while i >= 30 && value > 0 {
        value = create_gold_item_in_inventory_slot(ctx, pnum, i, value);
        i -= 1;
    }
    let mut x = 9;
    while x >= 0 && value > 0 {
        let mut y = 2;
        while y >= 0 && value > 0 {
            value = create_gold_item_in_inventory_slot(ctx, pnum, 10 * y + x, value);
            y -= 1;
        }
        x -= 1;
    }
    value
}

/// Original: `devilution::GoldAutoPlace` (inv.cpp).
// @port inv.cpp|devilution::GoldAutoPlace(Player &player, Item &goldStack) sha=528725348e1a
pub fn gold_auto_place(ctx: &mut Ctx, pnum: usize, gold_stack: &mut Item) -> bool {
    gold_stack._ivalue = add_gold_to_inventory(ctx, pnum, gold_stack._ivalue);
    crate::items::set_plr_hand_gold_curs(gold_stack);
    ctx.players.Players[pnum]._pGold = calculate_gold(ctx, pnum);
    gold_stack._ivalue == 0
}

/// Original: `devilution::CheckInvSwap(Player &player, inv_body_loc bLoc)` (inv.cpp).
// @port inv.cpp|devilution::CheckInvSwap(Player &player, inv_body_loc bLoc) sha=82f1aac8eafd
pub fn check_inv_swap_body(ctx: &mut Ctx, pnum: usize, b_loc: inv_body_loc) {
    let p = &mut ctx.players.Players[pnum];
    let loc = p.get_item_location(&p.InvBody[b_loc as usize]);
    if b_loc == INVLOC_HAND_LEFT && loc == ILOC_TWOHAND {
        p.InvBody[INVLOC_HAND_RIGHT as usize].clear();
    } else if b_loc == INVLOC_HAND_RIGHT && loc == ILOC_TWOHAND {
        p.InvBody[INVLOC_HAND_LEFT as usize].clear();
    }
    crate::items::calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::inv_update_rem_item` (inv.cpp).
// @port inv.cpp|devilution::inv_update_rem_item(Player &player, inv_body_loc iv) sha=abfe4d692501
pub fn inv_update_rem_item(ctx: &mut Ctx, pnum: usize, iv: inv_body_loc) {
    ctx.players.Players[pnum].InvBody[iv as usize].clear();
    let loadgfx = ctx.players.Players[pnum]._pmode != PM_DEATH;
    crate::items::calc_plr_inv(ctx, pnum, loadgfx);
}

/// Original: `devilution::CheckInvSwap(Player &player, const Item &item, int invGridIndex)` (inv.cpp).
// @port inv.cpp|devilution::CheckInvSwap(Player &player, const Item &item, int invGridIndex) sha=6d6e13974e25
pub fn check_inv_swap_grid(ctx: &mut Ctx, pnum: usize, item: &Item, inv_grid_index: i32) {
    let item_size = get_inventory_size(item);
    const PITCH: i32 = 10;
    let p = &mut ctx.players.Players[pnum];
    let inv_list_index: i32 = 'found: {
        for y in 0..item_size.height {
            let row_grid_index = inv_grid_index + PITCH * y;
            for x in 0..item_size.width {
                let grid_index = (row_grid_index + x) as usize;
                if p.InvGrid[grid_index] != 0 {
                    break 'found (p.InvGrid[grid_index] as i32).abs();
                }
            }
        }
        p._pNumInv += 1;
        p._pNumInv
    };
    if inv_list_index < p._pNumInv {
        for item_index in p.InvGrid.iter_mut() {
            if *item_index as i32 == inv_list_index {
                *item_index = 0;
            }
            if *item_index as i32 == -inv_list_index {
                *item_index = 0;
            }
        }
    }
    p.InvList[(inv_list_index - 1) as usize] = item.clone();
    for y in 0..item_size.height {
        let row_grid_index = inv_grid_index + PITCH * y;
        for x in 0..item_size.width {
            let v = if x == 0 && y == item_size.height - 1 { inv_list_index } else { -inv_list_index };
            p.InvGrid[(row_grid_index + x) as usize] = v as i8;
        }
    }
    crate::items::calc_plr_inv(ctx, pnum, true);
}

/// Original: `devilution::CheckInvRemove` (inv.cpp).
// @port inv.cpp|devilution::CheckInvRemove(Player &player, int invGridIndex) sha=98f001364a0f
pub fn check_inv_remove(ctx: &mut Ctx, pnum: usize, inv_grid_index: i32) {
    let inv_list_index = (ctx.players.Players[pnum].InvGrid[inv_grid_index as usize] as i32).abs() - 1;
    if inv_list_index >= 0 {
        crate::player::remove_inv_item(ctx, pnum, inv_list_index, true);
    }
}

/// Original: `devilution::TransferItemToStash` (inv.cpp).
// @port inv.cpp|devilution::TransferItemToStash(Player &player, int location) sha=d40fb6072ad0
pub fn transfer_item_to_stash(ctx: &mut Ctx, pnum: usize, location: i32) {
    if location == -1 {
        return;
    }
    let item = get_inventory_item(ctx, pnum, location).clone();
    if !crate::qol::stash::auto_place_item_in_stash(ctx, pnum, &item, true) {
        player_say_specific(ctx, pnum, HeroSpeech::WhereWouldIPutThis);
        return;
    }
    play_item_inv_sfx(ctx, item._iCurs);
    if location < INVITEM_INV_FIRST as i32 {
        remove_equipment(ctx, pnum, location as inv_body_loc, false);
        crate::items::calc_plr_inv(ctx, pnum, true);
    } else if location <= INVITEM_INV_LAST as i32 {
        crate::player::remove_inv_item(ctx, pnum, location - INVITEM_INV_FIRST as i32, true);
    } else {
        crate::player::remove_spd_bar_item(ctx, pnum, location - INVITEM_BELT_FIRST as i32);
    }
}

/// Original: `devilution::CheckInvItem` (inv.cpp).
// @port inv.cpp|devilution::CheckInvItem(bool isShiftHeld, bool isCtrlHeld) sha=aacf1d932035
pub fn check_inv_item(ctx: &mut Ctx, is_shift_held: bool, is_ctrl_held: bool) {
    if crate::player::is_inspecting_player(ctx) {
        return;
    }
    let me = my_player(ctx);
    let mouse = Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1);
    if !ctx.players.Players[me].HoldItem.is_empty() {
        check_inv_paste(ctx, me, mouse);
    } else if ctx.stash.IsStashOpen && is_ctrl_held {
        let loc = ctx.cursor.pcursinvitem as i32;
        transfer_item_to_stash(ctx, me, loc);
    } else {
        check_inv_cut(ctx, me, mouse, is_shift_held, is_ctrl_held);
    }
}

/// Original: `devilution::CheckInvScrn` (inv.cpp).
// @port inv.cpp|devilution::CheckInvScrn(bool isShiftHeld, bool isCtrlHeld) sha=a3d6bc63c26f
pub fn check_inv_scrn(ctx: &mut Ctx, is_shift_held: bool, is_ctrl_held: bool) {
    let mp = panel_pos(crate::control::get_main_panel(ctx));
    let (mx, my) = ctx.diablo.mouse_position;
    if mx > 190 + mp.x && mx < 437 + mp.x && my > mp.y && my < 33 + mp.y {
        check_inv_item(ctx, is_shift_held, is_ctrl_held);
    }
}

/// Original: `devilution::InvGetItem` (inv.cpp).
// @port inv.cpp|devilution::InvGetItem(Player &player, int ii) sha=815f9b73da73
pub fn inv_get_item(ctx: &mut Ctx, pnum: usize, ii: i32) {
    if ctx.control.drop_gold_flag {
        crate::control::close_gold_drop(ctx);
        ctx.control.drop_gold_value = 0;
    }
    let pos = ctx.items.Items[ii as usize].position;
    if ctx.items.dItem[pos.x as usize][pos.y as usize] == 0 {
        return;
    }
    let mut item = ctx.items.Items[ii as usize].clone();
    item._iCreateInfo &= !(CF_PREGEN as u16);
    check_quest_item(ctx, pnum, &mut item);
    item.update_required_stats_cache_for_player(&ctx.players.Players[pnum]);
    if item._itype == ItemType::Gold && gold_auto_place(ctx, pnum, &mut item) {
        ctx.items.Items[ii as usize] = item;
        if is_my_player(ctx, pnum) {
            play_sfx(ctx, IS_GOLD);
        }
    } else {
        ctx.items.Items[ii as usize] = item.clone();
        if is_my_player(ctx, pnum) && !ctx.players.Players[pnum].HoldItem.is_empty() {
            let tile = ctx.players.Players[pnum].position.tile;
            let hold = ctx.players.Players[pnum].HoldItem.clone();
            crate::msg::net_send_cmd_p_item(ctx, true, CMD_SYNCPUTITEM, tile, &hold);
        }
        ctx.players.Players[pnum].HoldItem = item.clone();
        new_cursor_item(ctx, item.is_empty(), item._iCurs);
    }
    cleanup_items(ctx, ii);
    ctx.cursor.pcursitem = -1;
}

/// Original: `devilution::FindAdjacentPositionForItem` (inv.cpp).
// @port inv.cpp|devilution::FindAdjacentPositionForItem(Point origin, Direction facing) sha=a13c7301a7c8
pub fn find_adjacent_position_for_item(ctx: &Ctx, origin: Point, facing: Direction) -> Option<Point> {
    if ctx.items.ActiveItemCount as usize >= crate::items::MAXITEMS {
        return None;
    }
    let candidates = [
        facing,
        left(facing),
        right(facing),
        left(left(facing)),
        right(right(facing)),
        left(left(left(facing))),
        right(right(right(facing))),
        opposite(facing),
    ];
    for d in candidates {
        let p = origin + d;
        if can_put(ctx, p) {
            return Some(p);
        }
    }
    if can_put(ctx, origin) {
        return Some(origin);
    }
    None
}

/// Original: `devilution::AutoGetItem` (inv.cpp). The item is `Items[ii]`.
// @port inv.cpp|devilution::AutoGetItem(Player &player, Item *itemPointer, int ii) sha=f2c7ce2a7761
pub fn auto_get_item(ctx: &mut Ctx, pnum: usize, ii: i32) {
    if ctx.control.drop_gold_flag {
        crate::control::close_gold_drop(ctx);
        ctx.control.drop_gold_value = 0;
    }
    let pos = ctx.items.Items[ii as usize].position;
    if ctx.items.dItem[pos.x as usize][pos.y as usize] == 0 {
        return;
    }
    let mut item = ctx.items.Items[ii as usize].clone();
    item._iCreateInfo &= !(CF_PREGEN as u16);
    check_quest_item(ctx, pnum, &mut item);
    item.update_required_stats_cache_for_player(&ctx.players.Players[pnum]);

    let done;
    let mut auto_equipped = false;
    if item._itype == ItemType::Gold {
        done = gold_auto_place(ctx, pnum, &mut item);
        if !done {
            crate::items::set_plr_hand_gold_curs(&mut item);
        }
    } else {
        let mut d = auto_equip_enabled(ctx, pnum, &item) && auto_equip(ctx, pnum, &item, true);
        if d {
            auto_equipped = true;
        }
        if !d {
            d = auto_place_item_in_belt(ctx, pnum, &item, true);
        }
        if !d {
            d = auto_place_item_in_inventory(ctx, pnum, &item, true);
        }
        done = d;
    }
    ctx.items.Items[ii as usize] = item;

    if done {
        if !auto_equipped && ctx.options.audio.item_pickup_sound.get() && is_my_player(ctx, pnum) {
            play_sfx(ctx, IS_IGRAB);
        }
        cleanup_items(ctx, ii);
        return;
    }
    if is_my_player(ctx, pnum) {
        player_say(ctx, pnum, HeroSpeech::ICantCarryAnymore);
    }
    let mut item = std::mem::take(&mut ctx.items.Items[ii as usize]);
    crate::items::respawn_item(ctx, &mut item, true);
    let position = item.position;
    ctx.items.Items[ii as usize] = item.clone();
    crate::msg::net_send_cmd_p_item(ctx, true, CMD_SPAWNITEM, position, &item);
}

/// Original: `devilution::FindGetItem` (inv.cpp). Returns an `ActiveItems` index.
// @port inv.cpp|devilution::FindGetItem(uint32_t iseed, _item_indexes idx, uint16_t createInfo) sha=44eb36d02b6c
pub fn find_get_item(ctx: &Ctx, iseed: u32, idx: _item_indexes, create_info: u16) -> i32 {
    for i in 0..ctx.items.ActiveItemCount {
        let item = &ctx.items.Items[ctx.items.ActiveItems[i as usize] as usize];
        if item.key_attributes_match(iseed, idx, create_info) {
            return i as i32;
        }
    }
    -1
}

/// Original: `devilution::SyncGetItem` (inv.cpp).
// @port inv.cpp|devilution::SyncGetItem(Point position, uint32_t iseed, _item_indexes idx, uint16_t ci) sha=c47c1205065d
pub fn sync_get_item(ctx: &mut Ctx, position: Point, iseed: u32, idx: _item_indexes, ci: u16) {
    let mut ii = ctx.items.dItem[position.x as usize][position.y as usize] as i32 - 1;
    if ii >= 0 && (ii as usize) < crate::items::MAXITEMS && !ctx.items.Items[ii as usize].key_attributes_match(iseed, idx, ci) {
        ii = -1;
    }
    if ii == -1 {
        ii = find_get_item(ctx, iseed, idx, ci);
        if ii != -1 {
            ii = ctx.items.ActiveItems[ii as usize] as i32;
        }
    }
    if ii == -1 {
        return;
    }
    cleanup_items(ctx, ii);
}

/// Original: `devilution::CanPut` (inv.cpp).
// @port inv.cpp|devilution::CanPut(Point position) sha=ad19156902c1
pub fn can_put(ctx: &Ctx, position: Point) -> bool {
    if !in_dungeon_bounds(position) {
        return false;
    }
    if crate::engine::path::is_tile_solid(ctx, position) {
        return false;
    }
    let (x, y) = (position.x as usize, position.y as usize);
    if ctx.items.dItem[x][y] != 0 {
        return false;
    }
    if ctx.gendung.leveltype == DungeonType::Town {
        if ctx.gendung.dMonster[x][y] != 0 {
            return false;
        }
        if ctx.gendung.dMonster[x + 1][y + 1] != 0 {
            return false;
        }
    }
    if crate::objects::is_item_blocking_object_at_position(ctx, position) {
        return false;
    }
    true
}

/// Original: `devilution::ClampDurability` (inv.cpp).
// @port inv.cpp|devilution::ClampDurability(const Item &item, int durability) sha=82377ac66375
pub fn clamp_durability(item: &Item, durability: i32) -> i32 {
    if item._iMaxDur == 0 {
        return 0;
    }
    durability.clamp(1, item._iMaxDur)
}

/// Original: `devilution::ClampToHit` (inv.cpp).
// @port inv.cpp|devilution::ClampToHit(const Item &item, int16_t toHit) sha=18586eacac58
pub fn clamp_to_hit(item: &Item, to_hit: i16) -> i16 {
    if to_hit < item._iPLToHit || to_hit > 51 {
        return item._iPLToHit;
    }
    to_hit
}

/// Original: `devilution::ClampMaxDam` (inv.cpp).
// @port inv.cpp|devilution::ClampMaxDam(const Item &item, uint8_t maxDam) sha=531f9e9f87d4
pub fn clamp_max_dam(item: &Item, max_dam: u8) -> u8 {
    if max_dam < item._iMaxDam || max_dam as i32 - item._iMinDam as i32 > 30 {
        return item._iMaxDam;
    }
    max_dam
}

/// Original: `devilution::SyncDropItem` (inv.cpp).
// @port inv.cpp|devilution::SyncDropItem(Point position, _item_indexes idx, uint16_t icreateinfo, int iseed, int id, int dur, int mdur, int ch, int mch, int ivalue, uint32_t ibuff, int toHit, int maxDam) sha=bd504e82cefc
#[allow(clippy::too_many_arguments)]
pub fn sync_drop_item(
    ctx: &mut Ctx,
    position: Point,
    idx: _item_indexes,
    icreateinfo: u16,
    iseed: i32,
    id: i32,
    dur: i32,
    mdur: i32,
    ch: i32,
    mch: i32,
    ivalue: i32,
    ibuff: u32,
    to_hit: i32,
    max_dam: i32,
) -> i32 {
    if ctx.items.ActiveItemCount as usize >= crate::items::MAXITEMS {
        return -1;
    }
    let mut item = Item::default();
    let me = ctx.players.Players[my_player(ctx)].clone();
    crate::items::recreate_item(ctx, &me, &mut item, idx, icreateinfo, iseed as u32, ivalue, (ibuff & CF_HELLFIRE as u32) != 0);
    if id != 0 {
        item._iIdentified = true;
    }
    item._iMaxDur = mdur;
    item._iDurability = clamp_durability(&item, dur);
    item._iMaxCharges = mch.clamp(0, item._iMaxCharges);
    item._iCharges = ch.clamp(0, item._iMaxCharges);
    if ctx.init.gb_is_hellfire {
        item._iPLToHit = clamp_to_hit(&item, to_hit as i16);
        item._iMaxDam = clamp_max_dam(&item, max_dam as u8);
    }
    item.dwBuff = ibuff;
    crate::items::place_item_in_world(ctx, item, position) as i32
}

/// Original: `devilution::SyncDropEar` (inv.cpp).
// @port inv.cpp|devilution::SyncDropEar(Point position, uint16_t icreateinfo, uint32_t iseed, uint8_t cursval, string_view heroname) sha=accf86f0042f
pub fn sync_drop_ear(ctx: &mut Ctx, position: Point, icreateinfo: u16, iseed: u32, cursval: u8, heroname: &str) -> i32 {
    if ctx.items.ActiveItemCount as usize >= crate::items::MAXITEMS {
        return -1;
    }
    let mut item = Item::default();
    crate::items::recreate_ear(ctx, &mut item, icreateinfo, iseed, cursval, heroname);
    crate::items::place_item_in_world(ctx, item, position) as i32
}

/// Original: `devilution::CheckInvHLight` (inv.cpp).
// @port inv.cpp|devilution::CheckInvHLight() sha=631be0ffc8b2
pub fn check_inv_h_light(ctx: &mut Ctx) -> i8 {
    let mouse = Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1);
    let mut r: i32 = 0;
    while r < NUM_XY_SLOTS as i32 {
        let mut o = panel_pos(crate::control::get_right_panel(ctx));
        if r >= SLOTXY_BELT_FIRST as i32 {
            o = panel_pos(crate::control::get_main_panel(ctx));
        }
        if InvRect[r as usize].contains(mouse - Displacement::new(o.x, o.y)) {
            break;
        }
        r += 1;
    }
    if r >= NUM_XY_SLOTS as i32 {
        return -1;
    }
    let mut rv: i8 = -1;
    ctx.control.info_color = crate::engine::render::text_render::UiFlags::COLOR_WHITE;
    let ip = ctx.players.InspectPlayer.expect("InspectPlayer");
    let pi: Item;
    let mut is_belt = false;
    {
        let my_player = &ctx.players.Players[ip];
        if r == SLOTXY_HEAD as i32 {
            rv = INVLOC_HEAD as i8;
            pi = my_player.InvBody[rv as usize].clone();
        } else if r == SLOTXY_RING_LEFT as i32 {
            rv = INVLOC_RING_LEFT as i8;
            pi = my_player.InvBody[rv as usize].clone();
        } else if r == SLOTXY_RING_RIGHT as i32 {
            rv = INVLOC_RING_RIGHT as i8;
            pi = my_player.InvBody[rv as usize].clone();
        } else if r == SLOTXY_AMULET as i32 {
            rv = INVLOC_AMULET as i8;
            pi = my_player.InvBody[rv as usize].clone();
        } else if r == SLOTXY_HAND_LEFT as i32 {
            rv = INVLOC_HAND_LEFT as i8;
            pi = my_player.InvBody[rv as usize].clone();
        } else if r == SLOTXY_HAND_RIGHT as i32 {
            let lh = &my_player.InvBody[INVLOC_HAND_LEFT as usize];
            if lh.is_empty() || my_player.get_item_location(lh) != ILOC_TWOHAND {
                rv = INVLOC_HAND_RIGHT as i8;
                pi = my_player.InvBody[rv as usize].clone();
            } else {
                rv = INVLOC_HAND_LEFT as i8;
                pi = lh.clone();
            }
        } else if r == SLOTXY_CHEST as i32 {
            rv = INVLOC_CHEST as i8;
            pi = my_player.InvBody[rv as usize].clone();
        } else if r >= SLOTXY_INV_FIRST as i32 && r <= SLOTXY_INV_LAST as i32 {
            let item_id = my_player.InvGrid[(r - SLOTXY_INV_FIRST as i32) as usize].wrapping_abs();
            if item_id == 0 {
                return -1;
            }
            let ii = item_id as i32 - 1;
            rv = (ii + INVITEM_INV_FIRST as i32) as i8;
            pi = my_player.InvList[ii as usize].clone();
        } else {
            r -= SLOTXY_BELT_FIRST as i32;
            is_belt = true;
            pi = my_player.SpdList[r as usize].clone();
            if !pi.is_empty() {
                rv = (r + INVITEM_BELT_FIRST as i32) as i8;
            }
        }
    }
    if is_belt {
        redraw_component(ctx, PanelDrawComponent::Belt);
    }
    if pi.is_empty() {
        return -1;
    }
    if pi._itype == ItemType::Gold {
        let n_gold = pi._ivalue;
        let fmt = crate::utils::language::ngettext("{:s} gold piece", "{:s} gold pieces", n_gold);
        ctx.control.info_string = fmt.replacen("{:s}", &crate::utils::format_int::format_integer(n_gold), 1);
    } else {
        ctx.control.info_color = pi.get_text_color();
        ctx.control.info_string = pi.get_name(ctx);
        if pi._iIdentified {
            crate::items::print_item_details(ctx, &pi);
        } else {
            crate::items::print_item_dur(ctx, &pi);
        }
    }
    rv
}

/// Original: `devilution::ConsumeScroll` (inv.cpp).
// @port inv.cpp|devilution::ConsumeScroll(Player &player) sha=9fe7f337378a
pub fn consume_scroll(ctx: &mut Ctx, pnum: usize) {
    let spell_id = ctx.players.Players[pnum].executedSpell.spellId;
    let is_current_spell = move |item: &Item| item.is_scroll_of(spell_id) || item.is_rune_of(spell_id);
    let item_slot = ctx.players.Players[pnum].executedSpell.spellFrom as i8 as i32;
    if item_slot >= INVITEM_INV_FIRST as i32 && item_slot <= INVITEM_INV_LAST as i32 {
        let item_index = item_slot - INVITEM_INV_FIRST as i32;
        let item = &ctx.players.Players[pnum].InvList[item_index as usize];
        if !item.is_empty() && is_current_spell(item) {
            crate::player::remove_inv_item(ctx, pnum, item_index, true);
            return;
        }
    } else if item_slot >= INVITEM_BELT_FIRST as i32 && item_slot <= INVITEM_BELT_LAST as i32 {
        let item_index = item_slot - INVITEM_BELT_FIRST as i32;
        let item = &ctx.players.Players[pnum].SpdList[item_index as usize];
        if !item.is_empty() && is_current_spell(item) {
            crate::player::remove_spd_bar_item(ctx, pnum, item_index);
            return;
        }
    } else if item_slot != 0 {
        crate::appfat::app_fatal(ctx, &format!("ConsumeScroll: Invalid item index {item_slot}"));
    }
    remove_inventory_or_belt_item(ctx, pnum, is_current_spell);
}

/// Original: `devilution::CanUseScroll` (inv.cpp).
// @port inv.cpp|devilution::CanUseScroll(Player &player, SpellID spell) sha=80513d266e36
pub fn can_use_scroll(ctx: &Ctx, pnum: usize, spell: SpellID) -> bool {
    if ctx.gendung.leveltype == DungeonType::Town && !crate::items::get_spell_data(spell).is_allowed_in_town() {
        return false;
    }
    has_inventory_or_belt_item(ctx, pnum, |item| item.is_scroll_of(spell) || item.is_rune_of(spell))
}

/// Original: `devilution::ConsumeStaffCharge` (inv.cpp).
// @port inv.cpp|devilution::ConsumeStaffCharge(Player &player) sha=846481b893ea
pub fn consume_staff_charge(ctx: &mut Ctx, pnum: usize) {
    let p = &mut ctx.players.Players[pnum];
    let spell = p.executedSpell.spellId;
    let staff = &mut p.InvBody[INVLOC_HAND_LEFT as usize];
    if !can_use_staff_item(staff, spell) {
        return;
    }
    staff._iCharges -= 1;
    crate::player::calc_plr_staff(ctx, pnum);
}

/// Original: `devilution::CanUseStaff(Player &player, SpellID spellId)` (inv.cpp).
// @port inv.cpp|devilution::CanUseStaff(Player &player, SpellID spellId) sha=124b9c060abe
pub fn can_use_staff(ctx: &Ctx, pnum: usize, spell_id: SpellID) -> bool {
    can_use_staff_item(&ctx.players.Players[pnum].InvBody[INVLOC_HAND_LEFT as usize], spell_id)
}

/// Original: `devilution::GetInventoryItem` (inv.cpp).
// @port inv.cpp|devilution::GetInventoryItem(Player &player, int location) sha=3f6220c7865e
pub fn get_inventory_item(ctx: &mut Ctx, pnum: usize, location: i32) -> &mut Item {
    let p = &mut ctx.players.Players[pnum];
    if location < INVITEM_INV_FIRST as i32 {
        return &mut p.InvBody[location as usize];
    }
    if location <= INVITEM_INV_LAST as i32 {
        return &mut p.InvList[(location - INVITEM_INV_FIRST as i32) as usize];
    }
    &mut p.SpdList[(location - INVITEM_BELT_FIRST as i32) as usize]
}

/// Original: `devilution::UseInvItem` (inv.cpp).
// @port inv.cpp|devilution::UseInvItem(int cii) sha=af3318a0ee08
pub fn use_inv_item(ctx: &mut Ctx, cii: i32) -> bool {
    if crate::player::is_inspecting_player(ctx) {
        return false;
    }
    let me = my_player(ctx);
    {
        let p = &ctx.players.Players[me];
        if p._pInvincible && p._pHitPoints == 0 {
            return true;
        }
    }
    if ctx.cursor.pcurs != CURSOR_HAND {
        return true;
    }
    if !crate::stores::stextflag_is_none(ctx) {
        return true;
    }
    if cii < INVITEM_INV_FIRST as i32 {
        return false;
    }

    let mut speedlist = false;
    let mut c: i32;
    let item: Item;
    let auto_refill = ctx.options.gameplay.auto_refill_belt.get();
    {
        let p = &ctx.players.Players[me];
        if cii <= INVITEM_INV_LAST as i32 {
            c = cii - INVITEM_INV_FIRST as i32;
            item = p.InvList[c as usize].clone();
        } else {
            if ctx.control.talkflag {
                return true;
            }
            c = cii - INVITEM_BELT_FIRST as i32;
            let mut it = p.SpdList[c as usize].clone();
            speedlist = true;
            if auto_refill {
                for i in 0..p._pNumInv {
                    let inv = &p.InvList[i as usize];
                    if inv._iMiscId == it._iMiscId && inv._iSpell == it._iSpell {
                        c = i;
                        it = inv.clone();
                        speedlist = false;
                        break;
                    }
                }
            }
            if speedlist && auto_refill {
                let mut i = (INVITEM_BELT_LAST - INVITEM_BELT_FIRST) as i32;
                while i > c {
                    let candidate = &p.SpdList[i as usize];
                    if !candidate.is_empty() && candidate._iMiscId == it._iMiscId && candidate._iSpell == it._iSpell {
                        c = i;
                        it = candidate.clone();
                        break;
                    }
                    i -= 1;
                }
            }
            item = it;
        }
    }

    const SPEECH_DELAY: i32 = 10;
    if item.IDidx == IDI_MUSHROOM {
        player_say_delayed(ctx, me, HeroSpeech::NowThatsOneBigMushroom, SPEECH_DELAY);
        return true;
    }
    if item.IDidx == IDI_FUNGALTM {
        play_sfx(ctx, IS_IBOOK);
        player_say_delayed(ctx, me, HeroSpeech::ThatDidntDoAnything, SPEECH_DELAY);
        return true;
    }
    if ctx.players.Players[me].is_on_level(0) {
        let tile = ctx.players.Players[me].position.tile;
        if crate::items::use_item_opens_hive(ctx, &item, tile) {
            crate::levels::town::open_hive(ctx);
            crate::player::remove_inv_item(ctx, me, c, true);
            return true;
        }
        if crate::items::use_item_opens_grave(ctx, &item, tile) {
            crate::levels::town::open_grave(ctx);
            crate::player::remove_inv_item(ctx, me, c, true);
            return true;
        }
    }
    if !item.is_usable(ctx) {
        return false;
    }
    if !ctx.players.Players[me].can_use_item(&item) {
        player_say(ctx, me, HeroSpeech::ICantUseThisYet);
        return true;
    }
    if item._iMiscId == IMISC_NONE && item._itype == ItemType::Gold {
        start_gold_drop(ctx);
        return true;
    }
    if ctx.control.drop_gold_flag {
        crate::control::close_gold_drop(ctx);
        ctx.control.drop_gold_value = 0;
    }
    if item.is_scroll() && ctx.gendung.leveltype == DungeonType::Town && !crate::items::get_spell_data(item._iSpell).is_allowed_in_town() {
        return true;
    }
    if item._iMiscId > IMISC_RUNEFIRST && item._iMiscId < IMISC_RUNELAST && ctx.gendung.leveltype == DungeonType::Town {
        return true;
    }
    if item._iMiscId == IMISC_ARENAPOT && !ctx.players.Players[me].is_on_arena_level() {
        player_say(ctx, me, HeroSpeech::ThatWontWorkHere);
        return true;
    }
    if item._iMiscId == IMISC_BOOK {
        play_sfx(ctx, IS_RBOOK);
    } else {
        play_item_inv_sfx(ctx, item._iCurs);
    }
    crate::items::use_item(ctx, me, item._iMiscId, item._iSpell, cii);

    if speedlist {
        if ctx.players.Players[me].SpdList[c as usize]._iMiscId == IMISC_NOTE {
            crate::minitext::init_q_text_msg(ctx, TEXT_BOOK9);
            close_inventory(ctx);
            return true;
        }
        if !item.is_scroll() && !item.is_rune() {
            crate::player::remove_spd_bar_item(ctx, me, c);
        }
        return true;
    }
    if ctx.players.Players[me].InvList[c as usize]._iMiscId == IMISC_MAPOFDOOM {
        return true;
    }
    if ctx.players.Players[me].InvList[c as usize]._iMiscId == IMISC_NOTE {
        crate::minitext::init_q_text_msg(ctx, TEXT_BOOK9);
        close_inventory(ctx);
        return true;
    }
    if !item.is_scroll() && !item.is_rune() {
        crate::player::remove_inv_item(ctx, me, c, true);
    }
    true
}

/// Original: `devilution::CloseInventory` (inv.cpp).
// @port inv.cpp|devilution::CloseInventory() sha=304dc3585d0f
pub fn close_inventory(ctx: &mut Ctx) {
    crate::qol::stash::close_gold_withdraw(ctx);
    close_stash(ctx);
    ctx.inv.invflag = false;
}

/// Original: `devilution::CloseStash` (inv.cpp).
// @port inv.cpp|devilution::CloseStash() sha=40ba1c319ef7
pub fn close_stash(ctx: &mut Ctx) {
    if !ctx.stash.IsStashOpen {
        return;
    }
    let me = my_player(ctx);
    if !ctx.players.Players[me].HoldItem.is_empty() {
        let (future, dir) = (ctx.players.Players[me].position.future, ctx.players.Players[me]._pdir);
        let item_tile = find_adjacent_position_for_item(ctx, future, dir);
        let hold = ctx.players.Players[me].HoldItem.clone();
        if let Some(t) = item_tile {
            crate::msg::net_send_cmd_p_item(ctx, true, CMD_PUTITEM, t, &hold);
        } else {
            if !auto_place_item_in_belt(ctx, me, &hold, true)
                && !auto_place_item_in_inventory(ctx, me, &hold, true)
                && !crate::qol::stash::auto_place_item_in_stash(ctx, me, &hold, true)
            {
                crate::appfat::app_fatal(ctx, &crate::utils::language::tr("No room for item"));
            }
            play_item_inv_sfx(ctx, hold._iCurs);
        }
        ctx.players.Players[me].HoldItem.clear();
        new_cursor(ctx, CURSOR_HAND);
    }
    ctx.stash.IsStashOpen = false;
}

/// Original: `devilution::DoTelekinesis` (inv.cpp).
// @port inv.cpp|devilution::DoTelekinesis() sha=9c70d6cc180d
pub fn do_telekinesis(ctx: &mut Ctx) {
    let my_id = ctx.players.MyPlayerId;
    if let Some(oi) = ctx.cursor.ObjectUnderCursor {
        if !ctx.objects.Objects[oi].is_disabled_opt(ctx.options.gameplay.disable_crippling_shrines.get()) {
            let cp = ctx.cursor.cursPosition;
            crate::msg::net_send_cmd_loc(ctx, my_id, true, CMD_OPOBJT, cp);
        }
    }
    if ctx.cursor.pcursitem != -1 {
        let ci = ctx.cursor.pcursitem as u8;
        crate::msg::net_send_cmd_g_item(ctx, true, CMD_REQUESTAGITEM, my_id as u8, ci);
    }
    if ctx.cursor.pcursmonst != -1 {
        let m = ctx.cursor.pcursmonst as usize;
        if !crate::monster::m_talker(ctx, m) && ctx.monster.Monsters[m].talkMsg == TEXT_NONE {
            crate::msg::net_send_cmd_param1(ctx, true, CMD_KNOCKBACK, m as u16);
        }
    }
    new_cursor(ctx, CURSOR_HAND);
}

/// Original: `devilution::CalculateGold` (inv.cpp).
// @port inv.cpp|devilution::CalculateGold(Player &player) sha=e34ab17ee279
pub fn calculate_gold(ctx: &Ctx, pnum: usize) -> i32 {
    let p = &ctx.players.Players[pnum];
    let mut gold = 0;
    for i in 0..p._pNumInv as usize {
        if p.InvList[i]._itype == ItemType::Gold {
            gold += p.InvList[i]._ivalue;
        }
    }
    gold
}

/// Original: `devilution::GetInventorySize` (inv.cpp).
// @port inv.cpp|devilution::GetInventorySize(const Item &item) sha=24ba44a15646
pub fn get_inventory_size(item: &Item) -> Size {
    let item_size_index = item._iCurs as i32 + CURSOR_FIRSTITEM;
    let size = get_inv_item_size(item_size_index);
    Size::new(size.width / InventorySlotSizeInPixels.width, size.height / InventorySlotSizeInPixels.height)
}

/// The `BeltItem{}` key action of `InitKeymapActions` (diablo.cpp).
// From the hotkey lambdas in `devilution::InitKeymapActions` (diablo.cpp).
pub fn use_belt_item_slot(ctx: &mut Ctx, i: usize) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let item = &ctx.players.Players[me].SpdList[i];
    if !item.is_empty() && item._itype != ItemType::Gold {
        use_inv_item(ctx, INVITEM_BELT_FIRST as i32 + i as i32);
    }
}


/// Original: `devilution::InvDrawSlotBack` (inv.cpp): tints the slot background under an item
/// (`targetPosition` is the bottom-left corner; rows are drawn upwards). The original only
/// clips the target position; pixels outside the surface are skipped here.
// @port inv.cpp|devilution::InvDrawSlotBack(const Surface &out, Point targetPosition, Size size, item_quality itemQuality) sha=1a54479446df
pub fn inv_draw_slot_back(ctx: &mut Ctx, out: &crate::engine::surface::Surface, target_position: Point, size: Size, item_quality: item_quality) {
    use crate::panels::spell_icons::{PAL16_BEIGE, PAL16_BLUE, PAL16_GRAY, PAL16_ORANGE, PAL16_YELLOW};
    let mut src_rect = crate::engine::surface::Rect::new(0, 0, size.width, size.height);
    let mut target = (target_position.x, target_position.y);
    out.clip(&mut src_rect, &mut target);
    if size.width <= 0 || size.height <= 0 {
        return;
    }
    let inspecting = crate::player::is_inspecting_player(ctx);
    let base = match item_quality {
        ITEM_QUALITY_MAGIC => {
            if !inspecting {
                PAL16_BLUE
            } else {
                PAL16_ORANGE
            }
        }
        ITEM_QUALITY_UNIQUE => {
            if !inspecting {
                PAL16_YELLOW
            } else {
                PAL16_ORANGE
            }
        }
        _ => {
            if !inspecting {
                PAL16_BEIGE
            } else {
                PAL16_ORANGE
            }
        }
    };
    for row in 0..size.height {
        let y = target.1 - row;
        for col in 0..size.width {
            let x = target.0 + col;
            if !out.in_bounds(x, y) {
                continue;
            }
            let mut pix = out.get(x, y);
            if pix >= PAL16_GRAY {
                pix = pix.wrapping_sub(PAL16_GRAY.wrapping_sub(base).wrapping_sub(1));
            }
            out.put(x, y, pix);
        }
    }
}

/// Original: `devilution::DrawInv` (inv.cpp).
// @port inv.cpp|devilution::DrawInv(const Surface &out) sha=fccb8c630765
pub fn draw_inv(ctx: &mut Ctx, out: &crate::engine::surface::Surface) {
    use crate::control::get_panel_position;
    use crate::cursor::{get_inv_item_size, get_inv_item_sprite, CURSOR_FIRSTITEM};
    let p = get_panel_position(ctx, UiPanels::Inventory, Point::new(0, 351));
    crate::engine::render::clx_render::clx_draw(out, (p.x, p.y), &ctx.inv.p_inv_cels.as_ref().expect("pInvCels").get(0));
    let slot_size: [Size; 7] = [
        Size::new(2, 2), // head
        Size::new(1, 1), // left ring
        Size::new(1, 1), // right ring
        Size::new(1, 1), // amulet
        Size::new(2, 3), // left hand
        Size::new(2, 3), // right hand
        Size::new(2, 3), // chest
    ];
    let slot_pos: [Point; 7] = [
        Point::new(133, 59),  // head
        Point::new(48, 205),  // left ring
        Point::new(249, 205), // right ring
        Point::new(205, 60),  // amulet
        Point::new(17, 160),  // left hand
        Point::new(248, 160), // right hand
        Point::new(133, 160), // chest
    ];
    let pi = ctx.players.InspectPlayer.expect("InspectPlayer");
    let px = |s: Size| Size::new(s.width * InventorySlotSizeInPixels.width, s.height * InventorySlotSizeInPixels.height);
    for slot in INVLOC_HEAD as usize..NUM_INVLOC as usize {
        let item = ctx.players.Players[pi].InvBody[slot].clone();
        if item.is_empty() {
            continue;
        }
        let mut screen_x = slot_pos[slot].x;
        let mut screen_y = slot_pos[slot].y;
        let pos = get_panel_position(ctx, UiPanels::Inventory, Point::new(screen_x, screen_y));
        inv_draw_slot_back(ctx, out, pos, px(slot_size[slot]), item._iMagical);
        let curs_id = item._iCurs as i32 + CURSOR_FIRSTITEM;
        let frame_size = get_inv_item_size(curs_id);
        // calc item offsets for weapons/armor smaller than 2x3 slots
        if slot == INVLOC_HAND_LEFT as usize || slot == INVLOC_HAND_RIGHT as usize || slot == INVLOC_CHEST as usize {
            screen_x += if frame_size.width == InventorySlotSizeInPixels.width { INV_SLOT_HALF_SIZE_PX } else { 0 };
            screen_y += if frame_size.height == 3 * InventorySlotSizeInPixels.height { 0 } else { -INV_SLOT_HALF_SIZE_PX };
        }
        let sprite = get_inv_item_sprite(ctx, curs_id);
        let position = get_panel_position(ctx, UiPanels::Inventory, Point::new(screen_x, screen_y));
        if ctx.cursor.pcursinvitem as i32 == slot as i32 {
            let color = crate::items::get_outline_color(&item, true);
            crate::engine::render::clx_render::clx_draw_outline(out, color, (position.x, position.y), &sprite);
        }
        crate::cursor::draw_item(ctx, &item, out, position, &sprite);
        if slot == INVLOC_HAND_LEFT as usize && ctx.players.Players[pi].get_item_location(&item) == ILOC_TWOHAND {
            let pos = get_panel_position(ctx, UiPanels::Inventory, slot_pos[INVLOC_HAND_RIGHT as usize]);
            inv_draw_slot_back(ctx, out, pos, px(slot_size[INVLOC_HAND_RIGHT as usize]), item._iMagical);
            ctx.scrollrt.LightTableIndex = 0;
            let rp = crate::control::get_right_panel(ctx);
            let dst_x = rp.x + slot_pos[INVLOC_HAND_RIGHT as usize].x + if frame_size.width == InventorySlotSizeInPixels.width { INV_SLOT_HALF_SIZE_PX } else { 0 } - 1;
            let dst_y = rp.y + slot_pos[INVLOC_HAND_RIGHT as usize].y;
            crate::engine::render::scrollrt::clx_draw_light_blended(ctx, out, Point::new(dst_x, dst_y), &sprite);
        }
    }
    for i in 0..InventoryGridCells {
        let g = ctx.players.Players[pi].InvGrid[i];
        if g != 0 {
            let quality = ctx.players.Players[pi].InvList[(g as i32).unsigned_abs() as usize - 1]._iMagical;
            let pos = get_panel_position(ctx, UiPanels::Inventory, InvRect[i + SLOTXY_INV_FIRST as usize].position) + Displacement::new(0, InventorySlotSizeInPixels.height);
            inv_draw_slot_back(ctx, out, pos, InventorySlotSizeInPixels, quality);
        }
    }
    for j in 0..InventoryGridCells {
        let g = ctx.players.Players[pi].InvGrid[j];
        if g > 0 {
            // first slot of an item
            let ii = (g - 1) as usize;
            let item = ctx.players.Players[pi].InvList[ii].clone();
            let curs_id = item._iCurs as i32 + CURSOR_FIRSTITEM;
            let sprite = get_inv_item_sprite(ctx, curs_id);
            let position = get_panel_position(ctx, UiPanels::Inventory, InvRect[j + SLOTXY_INV_FIRST as usize].position) + Displacement::new(0, InventorySlotSizeInPixels.height);
            if ctx.cursor.pcursinvitem as i32 == ii as i32 + INVITEM_INV_FIRST as i32 {
                let color = crate::items::get_outline_color(&item, true);
                crate::engine::render::clx_render::clx_draw_outline(out, color, (position.x, position.y), &sprite);
            }
            crate::cursor::draw_item(ctx, &item, out, position, &sprite);
        }
    }
}

/// Original: `devilution::DrawInvBelt` (inv.cpp).
// @port inv.cpp|devilution::DrawInvBelt(const Surface &out) sha=7c66068a19ab
pub fn draw_inv_belt(ctx: &mut Ctx, out: &crate::engine::surface::Surface) {
    use crate::cursor::{get_inv_item_sprite, CURSOR_FIRSTITEM};
    use crate::engine::render::text_render::{draw_string, UiFlags};
    if ctx.control.talkflag {
        return;
    }
    let mp = crate::control::get_main_panel(ctx);
    let main_panel_position = Point::new(mp.x, mp.y);
    crate::control::draw_panel_box(ctx, out, crate::engine::surface::Rect::new(205, 21, 232, 28), main_panel_position + Displacement::new(205, 5));
    let pi = ctx.players.InspectPlayer.expect("InspectPlayer");
    for i in 0..MaxBeltItems {
        let item = ctx.players.Players[pi].SpdList[i].clone();
        if item.is_empty() {
            continue;
        }
        let r = InvRect[i + SLOTXY_BELT_FIRST as usize].position;
        let position = Point::new(r.x + main_panel_position.x, r.y + main_panel_position.y + InventorySlotSizeInPixels.height);
        inv_draw_slot_back(ctx, out, position, InventorySlotSizeInPixels, item._iMagical);
        let curs_id = item._iCurs as i32 + CURSOR_FIRSTITEM;
        let sprite = get_inv_item_sprite(ctx, curs_id);
        if ctx.cursor.pcursinvitem as i32 == i as i32 + INVITEM_BELT_FIRST as i32
            && (ctx.controls.control_mode == crate::controls::ControlTypes::KeyboardAndMouse || ctx.inv.invflag)
        {
            let color = crate::items::get_outline_color(&item, true);
            crate::engine::render::clx_render::clx_draw_outline(out, color, (position.x, position.y), &sprite);
        }
        crate::cursor::draw_item(ctx, &item, out, position, &sprite);
        if item.is_usable(ctx) && item._itype != ItemType::Gold {
            let rect = crate::engine::surface::Rect::new(position.x, position.y - 12, InventorySlotSizeInPixels.width, InventorySlotSizeInPixels.height);
            draw_string(ctx, out, &(i + 1).to_string(), rect, UiFlags::COLOR_WHITE | UiFlags::ALIGN_RIGHT, 1, -1);
        }
    }
}

pub use crate::controls::plrctrls::use_belt_item;
