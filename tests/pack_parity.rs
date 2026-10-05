//! The item tests of DevilutionX's `test/pack_test.cpp`: items regenerated from their packed form
//! (seed, creation info, base item, ...) must equal what DevilutionX 1.5.3 produced, field by
//! field, and pack back to the same bytes. The data tables are generated into
//! `pack_parity_data` by `tools/gen_pack_tests.py`.

#![allow(non_snake_case)]

mod pack_parity_data;

use diablo1_rs::ctx::Ctx;
use diablo1_rs::enums::*;
use diablo1_rs::items::Item;
use diablo1_rs::pack::{pack_item, unpack_item, ItemPack};
use diablo1_rs::platform::Platform;
use pack_parity_data::*;

/// `PackTest::SetUp`
fn setup() -> Ctx {
    let mut ctx = Ctx::new(Platform::headless_from_env());
    ctx.players.Players.truncate(1);
    while ctx.players.Players.is_empty() {
        ctx.players.Players.push(Default::default());
    }
    ctx.players.MyPlayer = Some(0);
    ctx
}

fn unpack(ctx: &mut Ctx, packed: &ItemPack, is_hellfire: bool) -> Item {
    let player = ctx.players.Players[0].clone();
    let mut item = Item::default();
    unpack_item(ctx, packed, &player, &mut item, is_hellfire);
    item
}

/// `TestItemNameGeneration`
fn test_item_name_generation(ctx: &mut Ctx, item: &Item) {
    let allow_identified = item._iMiscId != IMISC_EAR; // Ears can't be identified.
    assert_eq!(allow_identified & item._iIdentified, item._iIdentified);

    let mut test_item = item.clone();

    test_item._iIdentified = false;
    assert_eq!(test_item.get_name(ctx), test_item._iName.as_str(), "unidentified name");
    if allow_identified {
        test_item._iIdentified = true;
        assert_eq!(test_item.get_name(ctx), test_item._iIName.as_str(), "identified name");

        // Check that UpdateHellfireFlag ensures that dwBuff is updated to get the correct name
        if item._iMagical == ITEM_QUALITY_MAGIC {
            let is_hellfire_item = (test_item.dwBuff & CF_HELLFIRE as u32) != 0;
            test_item.dwBuff = 0;
            let iname = test_item._iIName.as_str().to_string();
            diablo1_rs::items::update_hellfire_flag(ctx, &mut test_item, &iname);

            test_item._iIdentified = true;
            assert_eq!(test_item.get_name(ctx), test_item._iIName.as_str(), "identified name with UpdateHellfireFlag");
            assert!(is_hellfire_item || (item.dwBuff & CF_HELLFIRE as u32) != CF_HELLFIRE as u32, "item was wrongly converted to hellfire");
        }
    }
}

/// `CompareItems`
fn compare_items(item1: &Item, item2: &TestItem) {
    let name = item2._iIName;
    assert_eq!(item1._iIName.as_str(), item2._iIName);
    assert_eq!(item1._itype, item2._itype, "{name} _itype");
    let checks: [(&str, i64, i64); 39] = [
        ("_iClass", item1._iClass as i64, item2._iClass as i64),
        ("_iCurs", item1._iCurs as i64, item2._iCurs as i64),
        ("_iIvalue", item1._iIvalue as i64, item2._iIvalue as i64),
        ("_iMinDam", item1._iMinDam as i64, item2._iMinDam as i64),
        ("_iMaxDam", item1._iMaxDam as i64, item2._iMaxDam as i64),
        ("_iAC", item1._iAC as i64, item2._iAC as i64),
        ("_iFlags", item1._iFlags.0 as i64, item2._iFlags.0 as i64),
        ("_iMiscId", item1._iMiscId as i64, item2._iMiscId as i64),
        ("_iSpell", item1._iSpell as i64, item2._iSpell as i64),
        ("_iCharges", item1._iCharges as i64, item2._iCharges as i64),
        ("_iMaxCharges", item1._iMaxCharges as i64, item2._iMaxCharges as i64),
        ("_iDurability", item1._iDurability as i64, item2._iDurability as i64),
        ("_iMaxDur", item1._iMaxDur as i64, item2._iMaxDur as i64),
        ("_iPLDam", item1._iPLDam as i64, item2._iPLDam as i64),
        ("_iPLToHit", item1._iPLToHit as i64, item2._iPLToHit as i64),
        ("_iPLAC", item1._iPLAC as i64, item2._iPLAC as i64),
        ("_iPLStr", item1._iPLStr as i64, item2._iPLStr as i64),
        ("_iPLMag", item1._iPLMag as i64, item2._iPLMag as i64),
        ("_iPLDex", item1._iPLDex as i64, item2._iPLDex as i64),
        ("_iPLVit", item1._iPLVit as i64, item2._iPLVit as i64),
        ("_iPLFR", item1._iPLFR as i64, item2._iPLFR as i64),
        ("_iPLLR", item1._iPLLR as i64, item2._iPLLR as i64),
        ("_iPLMR", item1._iPLMR as i64, item2._iPLMR as i64),
        ("_iPLMana", item1._iPLMana as i64, item2._iPLMana as i64),
        ("_iPLHP", item1._iPLHP as i64, item2._iPLHP as i64),
        ("_iPLDamMod", item1._iPLDamMod as i64, item2._iPLDamMod as i64),
        ("_iPLGetHit", item1._iPLGetHit as i64, item2._iPLGetHit as i64),
        ("_iPLLight", item1._iPLLight as i64, item2._iPLLight as i64),
        ("_iSplLvlAdd", item1._iSplLvlAdd as i64, item2._iSplLvlAdd as i64),
        ("_iUid", item1._iUid as i64, item2._iUid as i64),
        ("_iFMinDam", item1._iFMinDam as i64, item2._iFMinDam as i64),
        ("_iFMaxDam", item1._iFMaxDam as i64, item2._iFMaxDam as i64),
        ("_iLMinDam", item1._iLMinDam as i64, item2._iLMinDam as i64),
        ("_iLMaxDam", item1._iLMaxDam as i64, item2._iLMaxDam as i64),
        ("_iPrePower", item1._iPrePower as i64, item2._iPrePower as i64),
        ("_iSufPower", item1._iSufPower as i64, item2._iSufPower as i64),
        ("_iMinStr", item1._iMinStr as i64, item2._iMinStr as i64),
        ("_iMinMag", item1._iMinMag as i64, item2._iMinMag as i64),
        ("_iMinDex", item1._iMinDex as i64, item2._iMinDex as i64),
    ];
    let bad: Vec<_> = checks.iter().filter(|(_, a, b)| a != b).map(|(f, a, b)| format!("{f}: got {a}, expected {b}")).collect();
    assert!(bad.is_empty(), "{name}: {bad:?}");
    assert_eq!(item1.IDidx as i64, item2.IDidx as i64, "{name} IDidx");
}

/// The item names are measured with the game's fonts (devilutionx.mpq), so these tests need
/// `DIABLO_DATA_DIR` (the folder with the game's MPQ files); skipped without it.
fn setup_with_fonts() -> Option<Ctx> {
    let Some(data_dir) = std::env::var_os("DIABLO_DATA_DIR") else {
        eprintln!("skipped: set DIABLO_DATA_DIR to the folder with the game's MPQ files");
        return None;
    };
    let mut ctx = setup();
    ctx.paths.set_base_path(&format!("{}/", std::path::Path::new(&data_dir).display()));
    diablo1_rs::init::load_core_archives(&mut ctx);
    Some(ctx)
}

fn run_table(packed: &[ItemPack], expected: &[TestItem], hellfire: bool, multiplayer: bool, spawn: bool) {
    let Some(mut ctx) = setup_with_fonts() else { return };
    ctx.init.gb_is_hellfire = hellfire;
    ctx.init.gb_is_multiplayer = multiplayer;
    ctx.init.gb_is_spawn = spawn;
    ctx.players.Players[0]._pMaxManaBase = 125 << 6;
    ctx.players.Players[0]._pMaxHPBase = 125 << 6;
    for (p, e) in packed.iter().zip(expected) {
        let id = unpack(&mut ctx, p, hellfire);
        compare_items(&id, e);
        test_item_name_generation(&mut ctx, &id);

        let mut is = pack_item(&ctx, &id, hellfire);
        if hellfire {
            is.dwBuff &= !(CF_HELLFIRE as u32);
        }
        assert_eq!(is, *p, "{} packs back to the same bytes", e._iIName);
    }
}

#[test]
fn unpack_item_diablo() {
    run_table(&PACKED_DIABLO_ITEMS, &DIABLO_ITEMS, false, false, false);
}

#[test]
fn unpack_item_spawn() {
    run_table(&PACKED_SPAWN_ITEMS, &SPAWN_ITEMS, false, false, true);
}

#[test]
fn unpack_item_diablo_multiplayer() {
    run_table(&PACKED_DIABLO_MP_ITEMS, &DIABLO_MP_ITEMS, false, true, false);
}

#[test]
fn unpack_item_hellfire() {
    run_table(&PACKED_HELLFIRE_ITEMS, &HELLFIRE_ITEMS, true, false, false);
}

#[test]
fn unpack_item_diablo_unique_bug() {
    let pk_item_bug = ItemPack { iSeed: 6, iCreateInfo: (15 | CF_UPER1 | CF_UPER15 | CF_UNIQUE) as u16, idx: 14, bId: 5, bDur: 60, bMDur: 60, bCh: 0, bMCh: 0, wValue: 0, dwBuff: 0 }; // Veil of Steel - with morph bug
    let pk_item = ItemPack { iSeed: 6, iCreateInfo: (15 | CF_UPER15 | CF_UNIQUE) as u16, idx: 14, bId: 5, bDur: 60, bMDur: 60, bCh: 0, bMCh: 0, wValue: 0, dwBuff: 0 }; // Veil of Steel - fixed
    let mut ctx = setup();
    let id = unpack(&mut ctx, &pk_item_bug, false);
    assert_eq!(id._iIName.as_str(), "Veil of Steel");
    assert_eq!(id._itype, ItemType::Helm);
    assert_eq!(id._iClass, ICLASS_ARMOR);
    assert_eq!(id._iCurs, 85);
    assert_eq!(id._iIvalue, 63800);
    assert_eq!(id._iAC, 18);
    assert_eq!(id._iMiscId, IMISC_UNIQUE);
    assert_eq!(id._iPLAC, 60);
    assert_eq!(id._iPLStr, 15);
    assert_eq!(id._iPLVit, 15);
    assert_eq!(id._iPLFR, 50);
    assert_eq!(id._iPLLR, 50);
    assert_eq!(id._iPLMR, 50);
    assert_eq!(id._iPLMana, -1920);
    assert_eq!(id._iPLLight, -2);
    assert_eq!(id._iUid, 6);
    assert_eq!(id.IDidx as i32, IDI_STEELVEIL as i32);
    assert_eq!(pack_item(&ctx, &id, false), pk_item);
}

#[test]
fn unpack_item_diablo_strip_hellfire_items() {
    let is = ItemPack { iSeed: 1478792102, iCreateInfo: (3 | CF_UPER1) as u16, idx: 92, ..Default::default() }; // Scroll of Search
    let mut ctx = setup();
    let id = unpack(&mut ctx, &is, true);
    test_item_name_generation(&mut ctx, &id);
    assert_eq!(id._itype, ItemType::None);
}

#[test]
fn unpack_item_empty() {
    let is = ItemPack { idx: 0xFFFF, ..Default::default() };
    let mut ctx = setup();
    let id = unpack(&mut ctx, &is, false);
    test_item_name_generation(&mut ctx, &id);
    assert_eq!(id._itype, ItemType::None);
}

#[test]
fn pack_item_empty() {
    let mut ctx = setup();
    let mut id = Item::default();
    id._itype = ItemType::None;
    let is = pack_item(&ctx, &id, false);
    assert_eq!(is.idx, 0xFFFF);
    test_item_name_generation(&mut ctx, &id);
}

/// `compareGold`
fn compare_gold(is: &ItemPack, i_curs: i32) {
    let mut ctx = setup();
    let id = unpack(&mut ctx, is, false);
    assert_eq!(id._iCurs as i32, i_curs);
    assert_eq!(id.IDidx as i32, IDI_GOLD as i32);
    assert_eq!(id._ivalue, is.wValue as i32);
    assert_eq!(id._itype, ItemType::Gold);
    assert_eq!(id._iClass, ICLASS_GOLD);
    test_item_name_generation(&mut ctx, &id);
    assert_eq!(pack_item(&ctx, &id, false), *is);
}

#[test]
fn unpack_item_gold_small() {
    compare_gold(&ItemPack { idx: IDI_GOLD as u16, wValue: 1000, ..Default::default() }, ICURS_GOLD_SMALL as i32);
}

#[test]
fn unpack_item_gold_medium() {
    compare_gold(&ItemPack { idx: IDI_GOLD as u16, wValue: 1001, ..Default::default() }, ICURS_GOLD_MEDIUM as i32);
}

#[test]
fn unpack_item_gold_large() {
    compare_gold(&ItemPack { idx: IDI_GOLD as u16, wValue: 2500, ..Default::default() }, ICURS_GOLD_LARGE as i32);
}

#[test]
fn unpack_item_ear() {
    let is = ItemPack { iSeed: 1633955154, iCreateInfo: 17509, idx: 23, bId: 111, bDur: 103, bMDur: 117, bCh: 101, bMCh: 68, wValue: 19843, dwBuff: 0 };
    let mut ctx = setup();
    let id = unpack(&mut ctx, &is, false);
    assert_eq!(id._iName.as_str(), "Ear of Dead-RogueDM");
    assert_eq!(id._ivalue, 3);
    test_item_name_generation(&mut ctx, &id);
    assert_eq!(pack_item(&ctx, &id, false), is);
}
