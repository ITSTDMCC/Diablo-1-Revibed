//! `Source/qol/autopickup.cpp`: automatically picking up gold, potions, elixirs and oils.

use crate::ctx::Ctx;
use crate::enums::*;
use crate::items::Item;

fn my_player(ctx: &Ctx) -> usize {
    ctx.players.MyPlayer.expect("MyPlayer")
}

/// Original: `HasRoomForGold` (qol/autopickup.cpp).
// @port qol/autopickup.cpp|devilution::HasRoomForGold() sha=63af19ee8392
fn has_room_for_gold(ctx: &Ctx) -> bool {
    let p = &ctx.players.Players[my_player(ctx)];
    for &idx in p.InvGrid.iter() {
        // Secondary item cell. No need to check those as we'll go through the main item cells anyway.
        if idx < 0 {
            continue;
        }
        // Empty cell. 1x1 space available.
        if idx == 0 {
            return true;
        }
        // Main item cell. Potentially a gold pile so check it.
        let item = &p.InvList[(idx - 1) as usize];
        if item._itype == ItemType::Gold && item._ivalue < ctx.items.MaxGold {
            return true;
        }
    }
    false
}

/// Original: `NumMiscItemsInInv` (qol/autopickup.cpp).
// @port qol/autopickup.cpp|devilution::NumMiscItemsInInv(int iMiscId) sha=21f1d3c7fe81
fn num_misc_items_in_inv(ctx: &Ctx, i_misc_id: item_misc_id) -> i32 {
    let p = &ctx.players.Players[my_player(ctx)];
    let n = p._pNumInv as usize;
    p.InvList[..n].iter().chain(p.SpdList.iter()).filter(|item| !item.is_empty() && item._iMiscId == i_misc_id).count() as i32
}

/// Original: `DoPickup` (qol/autopickup.cpp).
// @port qol/autopickup.cpp|devilution::DoPickup(Item item) sha=c91a8ff48933
fn do_pickup(ctx: &mut Ctx, item: Item) -> bool {
    let g = &ctx.options.gameplay;
    if item._itype == ItemType::Gold && g.auto_gold_pickup.get() && has_room_for_gold(ctx) {
        return true;
    }
    let me = my_player(ctx);
    if item._itype == ItemType::Misc && (crate::inv::auto_place_item_in_inventory(ctx, me, &item, false) || crate::inv::auto_place_item_in_belt(ctx, me, &item, false)) {
        let g = &ctx.options.gameplay;
        let n = num_misc_items_in_inv(ctx, item._iMiscId);
        return match item._iMiscId {
            IMISC_HEAL => g.num_heal_potion_pickup.get() > n,
            IMISC_FULLHEAL => g.num_full_heal_potion_pickup.get() > n,
            IMISC_MANA => g.num_mana_potion_pickup.get() > n,
            IMISC_FULLMANA => g.num_full_mana_potion_pickup.get() > n,
            IMISC_REJUV => g.num_reju_potion_pickup.get() > n,
            IMISC_FULLREJUV => g.num_full_reju_potion_pickup.get() > n,
            IMISC_ELIXSTR | IMISC_ELIXMAG | IMISC_ELIXDEX | IMISC_ELIXVIT => g.auto_elixir_pickup.get(),
            IMISC_OILFIRST | IMISC_OILOF | IMISC_OILACC | IMISC_OILMAST | IMISC_OILSHARP | IMISC_OILDEATH | IMISC_OILSKILL | IMISC_OILBSMTH | IMISC_OILFORT | IMISC_OILPERM
            | IMISC_OILHARD | IMISC_OILIMP | IMISC_OILLAST => g.auto_oil_pickup.get(),
            _ => false,
        };
    }
    false
}

/// Original: `devilution::AutoPickup` (qol/autopickup.cpp).
// @port qol/autopickup.cpp|devilution::AutoPickup(const Player &player) sha=6dd5aa3bda14
pub fn auto_pickup(ctx: &mut Ctx, pnum: usize) {
    if Some(pnum) != ctx.players.MyPlayer {
        return;
    }
    if ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town && !ctx.options.gameplay.auto_pickup_in_town.get() {
        return;
    }
    for path_dir in crate::engine::path::PATH_DIRS {
        let tile = ctx.players.Players[pnum].position.tile + path_dir;
        let d = ctx.items.dItem[tile.x as usize][tile.y as usize];
        if d != 0 {
            let item_index = (d - 1) as usize;
            let item = ctx.items.Items[item_index].clone();
            if do_pickup(ctx, item) {
                crate::msg::net_send_cmd_g_item(ctx, true, CMD_REQUESTAGITEM, pnum as u8, item_index as u8);
                ctx.items.Items[item_index]._iRequest = true;
            }
        }
    }
}
