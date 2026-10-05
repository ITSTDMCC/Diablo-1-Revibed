//! The port of DevilutionX 1.5.3's `test/inv_test.cpp` (scroll use, gold, removing inventory
//! and belt items, item sizes), assertion for assertion.

use diablo1_rs::ctx::Ctx;
use diablo1_rs::engine::geometry::Size;
use diablo1_rs::enums::*;
use diablo1_rs::inv::*;
use diablo1_rs::items::{Item, GOLD_MAX_LIMIT};
use diablo1_rs::levels::gendung::DungeonType;
use diablo1_rs::platform::Platform;

/// `InvTest::SetUp`
fn setup() -> Ctx {
    let mut ctx = Ctx::new(Platform::headless_from_env());
    ctx.diablo.headless_mode = true; // test/main.cpp: HeadlessMode = true
    ctx.players.Players.truncate(1);
    while ctx.players.Players.is_empty() {
        ctx.players.Players.push(Default::default());
    }
    ctx.players.MyPlayer = Some(0);
    ctx
}

#[derive(Clone, Copy)]
enum Slot {
    Inv(usize),
    Belt(usize),
}

fn slot(ctx: &mut Ctx, s: Slot) -> &mut Item {
    match s {
        Slot::Inv(i) => &mut ctx.players.Players[0].InvList[i],
        Slot::Belt(i) => &mut ctx.players.Players[0].SpdList[i],
    }
}

/// `set_up_scroll`: set up a given item as a spell scroll, allowing for its usage.
fn set_up_scroll(ctx: &mut Ctx, s: Slot, spell: SpellID) {
    ctx.cursor.pcurs = diablo1_rs::cursor::CURSOR_HAND;
    ctx.gendung.leveltype = DungeonType::Catacombs;
    ctx.players.Players[0]._pRSpell = spell;
    let item = slot(ctx, s);
    item._itype = ItemType::Misc;
    item._iMiscId = IMISC_SCROLL;
    item._iSpell = spell;
}

/// `clear_inventory`
fn clear_inventory(ctx: &mut Ctx) {
    let p = &mut ctx.players.Players[0];
    for i in 0..p.InvList.len() {
        p.InvList[i] = Item::default();
        p.InvGrid[i] = 0;
    }
    p._pNumInv = 0;
}

fn clear_belt(ctx: &mut Ctx) {
    for i in ctx.players.Players[0].SpdList.iter_mut() {
        i.clear();
    }
}

#[test]
fn use_scroll_from_inventory() {
    let mut c = setup();
    set_up_scroll(&mut c, Slot::Inv(2), SpellID::Firebolt);
    c.players.Players[0]._pNumInv = 5;
    assert!(can_use_scroll(&c, 0, SpellID::Firebolt));
}

#[test]
fn use_scroll_from_belt() {
    let mut c = setup();
    set_up_scroll(&mut c, Slot::Belt(2), SpellID::Firebolt);
    assert!(can_use_scroll(&c, 0, SpellID::Firebolt));
}

fn invalid_conditions(s: Slot) {
    let mut c = setup();
    match s {
        // Empty the belt to prevent using a scroll from the belt
        Slot::Inv(_) => {
            clear_belt(&mut c);
            c.players.Players[0]._pNumInv = 5;
        }
        // Disable the inventory to prevent using a scroll from the inventory
        Slot::Belt(_) => c.players.Players[0]._pNumInv = 0,
    }

    set_up_scroll(&mut c, s, SpellID::Firebolt);
    c.gendung.leveltype = DungeonType::Town;
    assert!(!can_use_scroll(&c, 0, SpellID::Firebolt));

    set_up_scroll(&mut c, s, SpellID::Firebolt);
    c.players.Players[0]._pRSpell = SpellID::Healing;
    assert!(!can_use_scroll(&c, 0, SpellID::Healing));

    set_up_scroll(&mut c, s, SpellID::Firebolt);
    slot(&mut c, s)._iMiscId = IMISC_STAFF;
    assert!(!can_use_scroll(&c, 0, SpellID::Firebolt));

    set_up_scroll(&mut c, s, SpellID::Firebolt);
    slot(&mut c, s).clear();
    assert!(!can_use_scroll(&c, 0, SpellID::Firebolt));
}

#[test]
fn use_scroll_from_inventory_invalid_conditions() {
    invalid_conditions(Slot::Inv(2));
}

#[test]
fn use_scroll_from_belt_invalid_conditions() {
    invalid_conditions(Slot::Belt(2));
}

#[test]
fn calculate_gold_sums_gold_slots() {
    let mut c = setup();
    let p = &mut c.players.Players[0];
    p._pNumInv = 10;
    for (i, v) in [(1, 100), (5, 200), (2, 3), (3, 30)] {
        p.InvList[i]._itype = ItemType::Gold;
        p.InvList[i]._ivalue = v;
    }
    assert_eq!(calculate_gold(&c, 0), 333);
}

#[test]
fn gold_auto_place_fills_stacks() {
    let mut c = setup();
    diablo1_rs::storm::storm_net::snet_initialize_provider(&mut c, SELCONN_LOOPBACK as u32);
    clear_inventory(&mut c);
    // | 1000 | ... | ...
    c.players.Players[0].InvList[0]._itype = ItemType::Gold;
    c.players.Players[0].InvList[0]._ivalue = 1000;
    c.players.Players[0]._pNumInv = 1;
    // Put (max gold - 100) gold, which is 4900, into the player's hand
    let mut hold = c.players.Players[0].HoldItem.clone();
    hold._itype = ItemType::Gold;
    hold._ivalue = GOLD_MAX_LIMIT - 100;
    gold_auto_place(&mut c, 0, &mut hold);
    c.players.Players[0].HoldItem = hold;
    // | 5000 | 900 | ...
    assert_eq!(c.players.Players[0].InvList[0]._ivalue, GOLD_MAX_LIMIT);
    assert_eq!(c.players.Players[0].InvList[1]._ivalue, 900);
}

#[test]
fn remove_inv_item_alone() {
    let mut c = setup();
    diablo1_rs::storm::storm_net::snet_initialize_provider(&mut c, SELCONN_LOOPBACK as u32);
    clear_inventory(&mut c);
    // | (item) | (item) | ... | ...
    let p = &mut c.players.Players[0];
    p._pNumInv = 1;
    p.InvGrid[0] = 1;
    p.InvGrid[1] = -1;
    p.InvList[0]._itype = ItemType::Misc;

    diablo1_rs::player::remove_inv_item(&mut c, 0, 0, true);
    let p = &c.players.Players[0];
    assert_eq!(p.InvGrid[0], 0);
    assert_eq!(p.InvGrid[1], 0);
    assert_eq!(p._pNumInv, 0);
}

#[test]
fn remove_inv_item_other_item() {
    let mut c = setup();
    diablo1_rs::storm::storm_net::snet_initialize_provider(&mut c, SELCONN_LOOPBACK as u32);
    clear_inventory(&mut c);
    // | (item) | (item) | (ring) | ...
    let p = &mut c.players.Players[0];
    p._pNumInv = 2;
    p.InvGrid[0] = 1;
    p.InvGrid[1] = -1;
    p.InvList[0]._itype = ItemType::Misc;
    p.InvGrid[2] = 2;
    p.InvList[1]._itype = ItemType::Ring;

    diablo1_rs::player::remove_inv_item(&mut c, 0, 0, true);
    let p = &c.players.Players[0];
    assert_eq!(p.InvGrid[0], 0);
    assert_eq!(p.InvGrid[1], 0);
    assert_eq!(p.InvGrid[2], 1);
    assert_eq!(p.InvList[0]._itype, ItemType::Ring);
    assert_eq!(p._pNumInv, 1);
}

#[test]
fn remove_spd_bar_item_clears_slot() {
    let mut c = setup();
    diablo1_rs::storm::storm_net::snet_initialize_provider(&mut c, SELCONN_LOOPBACK as u32);
    clear_belt(&mut c);
    // | x | x | item | x | x | x | x | x |
    c.players.Players[0].SpdList[3]._itype = ItemType::Misc;
    diablo1_rs::player::remove_spd_bar_item(&mut c, 0, 3);
    assert!(c.players.Players[0].SpdList[3].is_empty());
}

fn scroll_in(c: &mut Ctx, s: Slot, spell_from: i8) {
    let p = &mut c.players.Players[0];
    p.executedSpell.spellId = SpellID::Firebolt;
    p.executedSpell.spellFrom = spell_from;
    let item = slot(c, s);
    item._itype = ItemType::Misc;
    item._iMiscId = IMISC_SCROLL;
    item._iSpell = SpellID::Firebolt;
}

#[test]
fn remove_current_spell_scroll_from_inventory() {
    let mut c = setup();
    clear_inventory(&mut c);
    c.players.Players[0]._pNumInv = 1;
    scroll_in(&mut c, Slot::Inv(0), INVITEM_INV_FIRST as i8);
    consume_scroll(&mut c, 0);
    assert_eq!(c.players.Players[0].InvGrid[0], 0);
    assert_eq!(c.players.Players[0]._pNumInv, 0);
}

#[test]
fn remove_current_spell_scroll_from_inventory_first_match() {
    let mut c = setup();
    clear_inventory(&mut c);
    c.players.Players[0]._pNumInv = 1;
    scroll_in(&mut c, Slot::Inv(0), 0); // any matching scroll
    consume_scroll(&mut c, 0);
    assert_eq!(c.players.Players[0].InvGrid[0], 0);
    assert_eq!(c.players.Players[0]._pNumInv, 0);
}

#[test]
fn remove_current_spell_scroll_belt() {
    let mut c = setup();
    diablo1_rs::storm::storm_net::snet_initialize_provider(&mut c, SELCONN_LOOPBACK as u32);
    clear_belt(&mut c);
    scroll_in(&mut c, Slot::Belt(3), (INVITEM_BELT_FIRST + 3) as i8);
    consume_scroll(&mut c, 0);
    assert!(c.players.Players[0].SpdList[3].is_empty());
}

#[test]
fn remove_current_spell_scroll_first_match_from_belt() {
    let mut c = setup();
    diablo1_rs::storm::storm_net::snet_initialize_provider(&mut c, SELCONN_LOOPBACK as u32);
    clear_belt(&mut c);
    scroll_in(&mut c, Slot::Belt(3), 0); // any matching scroll
    consume_scroll(&mut c, 0);
    assert!(c.players.Players[0].SpdList[3].is_empty());
}

#[test]
fn item_size() {
    let c = setup();
    let mut item = Item::default();
    // rune of stone and grey suit are adjacent in the sprite list: an easy check for off-by-one errors
    diablo1_rs::items::initialize_item(&c, &mut item, IDI_RUNEOFSTONE);
    assert_eq!(get_inventory_size(&item), Size::new(1, 1));
    diablo1_rs::items::initialize_item(&c, &mut item, IDI_GREYSUIT);
    assert_eq!(get_inventory_size(&item), Size::new(2, 2));
    // auric amulet is the first used hellfire sprite
    diablo1_rs::items::initialize_item(&c, &mut item, IDI_AURIC);
    assert_eq!(get_inventory_size(&item), Size::new(1, 1));
    // gold is the last diablo sprite
    diablo1_rs::items::initialize_item(&c, &mut item, IDI_GOLD);
    assert_eq!(get_inventory_size(&item), Size::new(1, 1));
}
