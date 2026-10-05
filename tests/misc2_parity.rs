//! Ports of DevilutionX 1.5.3's test/effects_test.cpp (sound position), quests_test.cpp (quest
//! pools per seed), stores_test.cpp (repair prices) and codec_test.cpp, assertion for assertion.

use diablo1_rs::ctx::Ctx;
use diablo1_rs::engine::geometry::Point;
use diablo1_rs::enums::*;
use diablo1_rs::platform::Platform;

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

// ---------------------------------------------------------------------------------------------
// effects_test.cpp

fn sound_position(player: (i32, i32), sound: (i32, i32), volume: i32, pan: i32) -> (bool, i32, i32) {
    let mut c = setup();
    c.players.Players[0].position.tile = Point::new(player.0, player.1);
    let (mut v, mut p) = (volume, pan);
    let r = diablo1_rs::engine::sound::calculate_sound_position(&c, Point::new(sound.0, sound.1), &mut v, &mut p);
    (r, v, p)
}

#[test]
fn effects_calculate_sound_position() {
    assert_eq!(sound_position((50, 50), (50, 50), 0, 0), (true, 0, 0), "center");
    assert_eq!(sound_position((50, 50), (55, 50), 0, 0), (true, -320, 1280), "near");
    assert_eq!(sound_position((12, 12), (112, 112), 1234, 0), (false, 1234, 0), "out of range");
    assert_eq!(sound_position((50, 50), (75, 25), 0, 0), (true, -2176, 6400), "extreme right");
    assert_eq!(sound_position((50, 50), (25, 75), 0, 0), (true, -2176, -6400), "extreme left");
}

// ---------------------------------------------------------------------------------------------
// quests_test.cpp

/// `ResetQuests` + `InitialiseQuestPools(seed, Quests)`; returns `_qactive` per quest id.
fn pools(seed: u32) -> impl Fn(quest_id) -> quest_state {
    let mut c = setup();
    for q in c.quests.Quests.iter_mut() {
        q._qactive = QUEST_INIT;
    }
    let mut quests = c.quests.Quests.clone();
    diablo1_rs::quests::initialise_quest_pools(&mut c, seed, &mut quests);
    move |id| quests[id as usize]._qactive
}

#[test]
fn quest_single_player_bad_pools() {
    // (INT_MIN >> 16) % 2 = 0, so the times when the RNG calls GenerateRnd(2) don't end up with a negative value.
    assert_eq!(pools(1457187811)(Q_SKELKING), QUEST_NOTAVAIL, "Skeleton King quest is deactivated with 'bad' seed");
    let q = pools(988045466);
    assert!([Q_BUTCHER, Q_LTBANNER, Q_GARBUD].iter().all(|&id| q(id) == QUEST_INIT), "All quests in pool 2 remain active with 'bad' seed");
    let q = pools(4203210069);
    assert!([Q_BLIND, Q_ROCK, Q_BLOOD].iter().all(|&id| q(id) == QUEST_INIT), "All quests in pool 3 remain active with 'bad' seed");
    let q = pools(2557708932);
    assert!([Q_MUSHROOM, Q_ZHAR, Q_ANVIL].iter().all(|&id| q(id) == QUEST_INIT), "All quests in pool 4 remain active with 'bad' seed");
    assert_eq!(pools(1272442071)(Q_VEIL), QUEST_NOTAVAIL, "Lachdan quest is deactivated with 'bad' seed");
}

#[test]
fn quest_single_player_good_pools() {
    let q = pools(509604);
    let expected = [
        (Q_SKELKING, QUEST_INIT),
        (Q_PWATER, QUEST_NOTAVAIL),
        (Q_BUTCHER, QUEST_INIT),
        (Q_LTBANNER, QUEST_INIT),
        (Q_GARBUD, QUEST_NOTAVAIL),
        (Q_BLIND, QUEST_INIT),
        (Q_ROCK, QUEST_NOTAVAIL),
        (Q_BLOOD, QUEST_INIT),
        (Q_MUSHROOM, QUEST_INIT),
        (Q_ZHAR, QUEST_NOTAVAIL),
        (Q_ANVIL, QUEST_INIT),
        (Q_VEIL, QUEST_NOTAVAIL),
        (Q_WARLORD, QUEST_INIT),
    ];
    for (id, state) in expected {
        assert_eq!(q(id), state, "quest {id} with seed 509604");
    }
}

// ---------------------------------------------------------------------------------------------
// stores_test.cpp

/// `AddStoreHoldRepair(&storehold[0], 0)` with `storenumh = 0`: the item is repriced in place.
fn repair(c: &mut Ctx) {
    c.stores.storenumh = 0;
    let item = c.stores.storehold[0].clone();
    diablo1_rs::stores::add_store_hold_repair(c, &item, 0);
}

#[test]
fn stores_add_store_hold_repair_magic() {
    let mut c = setup();
    {
        let item = &mut c.stores.storehold[0];
        item._iMaxDur = 60;
        item._iDurability = item._iMaxDur;
        item._iMagical = ITEM_QUALITY_MAGIC;
        item._iIdentified = true;
    }
    for i in 1..60 {
        let item = &mut c.stores.storehold[0];
        item._ivalue = 2000;
        item._iIvalue = 19000;
        item._iDurability = i;
        repair(&mut c);
        assert_eq!(1, c.stores.storenumh);
        assert_eq!(95 * (60 - i) / 2, c.stores.storehold[0]._ivalue);
    }
    {
        let item = &mut c.stores.storehold[0];
        item._iDurability = 59;
        item._ivalue = 500;
        item._iIvalue = 30; // Too cheap to repair
    }
    repair(&mut c);
    assert_eq!(0, c.stores.storenumh);
    assert_eq!(30, c.stores.storehold[0]._iIvalue);
    assert_eq!(500, c.stores.storehold[0]._ivalue);
}

#[test]
fn stores_add_store_hold_repair_normal() {
    let mut c = setup();
    {
        let item = &mut c.stores.storehold[0];
        item._iMaxDur = 20;
        item._iDurability = item._iMaxDur;
        item._iMagical = ITEM_QUALITY_NORMAL;
        item._iIdentified = true;
    }
    for i in 1..20 {
        let item = &mut c.stores.storehold[0];
        item._ivalue = 2000;
        item._iIvalue = 2000;
        item._iDurability = i;
        repair(&mut c);
        assert_eq!(1, c.stores.storenumh);
        assert_eq!(50 * (20 - i), c.stores.storehold[0]._ivalue);
    }
    {
        let item = &mut c.stores.storehold[0];
        item._iDurability = 19;
        item._ivalue = 10; // less than 1 per dur
        item._iIvalue = 10;
    }
    repair(&mut c);
    assert_eq!(1, c.stores.storenumh);
    assert_eq!(1, c.stores.storehold[0]._ivalue);
    assert_eq!(1, c.stores.storehold[0]._iIvalue);
}

// ---------------------------------------------------------------------------------------------
// codec_test.cpp

#[test]
fn codec_get_encoded_len() {
    assert_eq!(diablo1_rs::codec::codec_get_encoded_len(50), 72);
    assert_eq!(diablo1_rs::codec::codec_get_encoded_len(128), 136);
}
