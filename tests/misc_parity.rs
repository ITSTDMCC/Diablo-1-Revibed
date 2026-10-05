//! Ports of DevilutionX 1.5.3's smaller unit tests: test/missiles_test.cpp, player_test.cpp,
//! dead_test.cpp and lighting_test.cpp, assertion for assertion.

use std::collections::HashMap;

use diablo1_rs::ctx::Ctx;
use diablo1_rs::engine::geometry::{Direction, Point};
use diablo1_rs::enums::*;
use diablo1_rs::platform::Platform;


fn setup() -> Ctx {
    let mut ctx = Ctx::new(Platform::headless_from_env());
    ctx.diablo.headless_mode = true; // test/main.cpp: HeadlessMode = true
    ctx.players.Players.truncate(1);
    while ctx.players.Players.is_empty() {
        ctx.players.Players.push(Default::default());
    }
    ctx.players.MyPlayerId = 0;
    ctx.players.MyPlayer = Some(0);
    ctx
}

// ---------------------------------------------------------------------------------------------
// missiles_test.cpp

fn rotations(ctx: &mut Ctx, mi: usize, frame: bool, start: i32) -> HashMap<i32, u32> {
    let mut observed = HashMap::new();
    for _ in 0..100 {
        if frame {
            ctx.missiles.Missiles[mi]._miAnimFrame = start;
        } else {
            ctx.missiles.Missiles[mi]._mimfnum = start;
        }
        diablo1_rs::missiles::test_rotate_blocked_missile(ctx, mi);
        let v = if frame { ctx.missiles.Missiles[mi]._miAnimFrame } else { ctx.missiles.Missiles[mi]._mimfnum };
        *observed.entry(v).or_insert(0) += 1;
    }
    observed
}

/// `TestArrowRotatesUniformly` / `TestAnimatedMissileRotatesUniformly`
fn assert_rotates_uniformly(observed: &HashMap<i32, u32>, left: i32, right: i32) {
    assert_eq!(observed.len(), 2, "{observed:?}");
    for v in [left, right] {
        let n = *observed.get(&v).unwrap_or(&0);
        assert!(n > 30 && n < 70, "should rotate either direction roughly 50% of the time: {observed:?}");
    }
}

#[test]
fn missiles_rotate_blocked_missile_arrow() {
    let mut c = setup();
    let add = |c: &mut Ctx, t: MissileID| {
        diablo1_rs::missiles::add_missile(c, Point::new(0, 0), Point::new(0, 0), Direction::South, t, TARGET_MONSTERS, 0, 0, 0, None).expect("missile added")
    };
    let mi = add(&mut c, MissileID::Arrow);
    // Arrows have a hardcoded frame count and use 1-indexed sprites
    assert_eq!(c.missiles.Missiles[mi]._miAnimFrame, 1);
    for (start, left, right) in [(5, 4, 6), (1, 16, 2), (16, 15, 1)] {
        let o = rotations(&mut c, mi, true, start);
        assert_rotates_uniformly(&o, left, right);
    }

    // All other missiles use the number of 0-indexed sprites defined in MissileSpriteData
    let mi = add(&mut c, MissileID::Firebolt);
    assert_eq!(c.missiles.Missiles[mi]._mimfnum, 0);
    for (start, left, right) in [(5, 4, 6), (0, 15, 1), (15, 14, 0)] {
        let o = rotations(&mut c, mi, false, start);
        assert_rotates_uniformly(&o, left, right);
    }
}

#[test]
fn missiles_get_direction8() {
    use diablo1_rs::engine::get_direction as d;
    let p = |x, y| Point::new(x, y);
    let cases = [
        (Direction::South, p(0, 0), p(15, 15)),
        (Direction::SouthWest, p(0, 0), p(0, 15)),
        (Direction::South, p(0, 0), p(8, 15)),
        (Direction::South, p(0, 0), p(8, 8)),
        (Direction::South, p(0, 0), p(15, 8)),
        (Direction::South, p(0, 0), p(15, 7)),
        (Direction::South, p(0, 0), p(11, 7)),
        (Direction::South, p(0, 0), p(8, 11)),
        (Direction::North, p(15, 15), p(0, 0)),
        (Direction::NorthEast, p(0, 15), p(0, 0)),
        (Direction::North, p(8, 15), p(0, 0)),
        (Direction::North, p(8, 8), p(0, 0)),
        (Direction::North, p(15, 8), p(0, 0)),
        (Direction::North, p(15, 7), p(0, 0)),
        (Direction::North, p(11, 7), p(0, 0)),
        (Direction::North, p(8, 11), p(0, 0)),
        (Direction::East, p(0, 15), p(15, 0)),
        (Direction::SouthEast, p(0, 0), p(15, 0)),
        (Direction::East, p(0, 8), p(15, 0)),
        (Direction::East, p(0, 8), p(8, 0)),
        (Direction::East, p(0, 15), p(8, 0)),
        (Direction::East, p(0, 15), p(7, 0)),
        (Direction::East, p(0, 11), p(7, 0)),
        (Direction::East, p(0, 8), p(11, 0)),
        (Direction::South, p(1, 1), p(2, 2)),
        (Direction::SouthWest, p(1, 1), p(1, 2)),
        (Direction::West, p(1, 1), p(0, 2)),
        (Direction::NorthWest, p(1, 1), p(0, 1)),
        (Direction::North, p(1, 1), p(0, 0)),
        (Direction::NorthEast, p(1, 1), p(1, 0)),
        (Direction::East, p(1, 1), p(2, 0)),
        (Direction::SouthEast, p(1, 1), p(2, 1)),
        // GetDirection defaults to SouthWest when the points occupy the same tile
        (Direction::SouthWest, p(0, 0), p(0, 0)),
    ];
    for (expected, a, b) in cases {
        assert_eq!(expected, d(a, b), "GetDirection({a:?}, {b:?})");
    }
}

#[test]
fn missiles_get_direction16() {
    use diablo1_rs::missiles::get_direction16 as d;
    use Direction16 as D;
    let p = |x, y| Point::new(x, y);
    let cases = [
        (D::South, p(0, 0), p(15, 15)),
        (D::SouthWest, p(0, 0), p(0, 15)),
        (D::South_SouthWest, p(0, 0), p(8, 15)),
        (D::South, p(0, 0), p(8, 8)),
        (D::South_SouthEast, p(0, 0), p(15, 8)),
        (D::South_SouthEast, p(0, 0), p(15, 7)),
        (D::South_SouthEast, p(0, 0), p(11, 7)),
        (D::South, p(0, 0), p(8, 11)),
        (D::North, p(15, 15), p(0, 0)),
        (D::NorthEast, p(0, 15), p(0, 0)),
        (D::North_NorthEast, p(8, 15), p(0, 0)),
        (D::North, p(8, 8), p(0, 0)),
        (D::North_NorthWest, p(15, 8), p(0, 0)),
        (D::North_NorthWest, p(15, 7), p(0, 0)),
        (D::North_NorthWest, p(11, 7), p(0, 0)),
        (D::North, p(8, 11), p(0, 0)),
        (D::East, p(0, 15), p(15, 0)),
        (D::SouthEast, p(0, 0), p(15, 0)),
        (D::East_SouthEast, p(0, 8), p(15, 0)),
        (D::East, p(0, 8), p(8, 0)),
        (D::East_NorthEast, p(0, 15), p(8, 0)),
        (D::East_NorthEast, p(0, 15), p(7, 0)),
        (D::East_NorthEast, p(0, 11), p(7, 0)),
        (D::East, p(0, 8), p(11, 0)),
        (D::South, p(2, 2), p(3, 3)),
        (D::South_SouthWest, p(2, 2), p(3, 4)),
        (D::SouthWest, p(2, 2), p(2, 4)),
        (D::West_SouthWest, p(2, 2), p(1, 4)),
        (D::West, p(2, 2), p(1, 3)),
        (D::West_NorthWest, p(2, 2), p(0, 3)),
        (D::NorthWest, p(2, 2), p(0, 2)),
        (D::North_NorthWest, p(2, 2), p(0, 1)),
        (D::North, p(2, 2), p(1, 1)),
        (D::North_NorthEast, p(2, 2), p(1, 0)),
        (D::NorthEast, p(2, 2), p(2, 0)),
        (D::East_NorthEast, p(2, 2), p(3, 0)),
        (D::East, p(2, 2), p(3, 1)),
        (D::East_SouthEast, p(2, 2), p(4, 1)),
        (D::SouthEast, p(2, 2), p(4, 2)),
        (D::South_SouthEast, p(2, 2), p(4, 3)),
        // GetDirection16 defaults to South_SouthWest when the points occupy the same tile
        (D::South_SouthWest, p(0, 0), p(0, 0)),
    ];
    for (expected, a, b) in cases {
        assert_eq!(expected, d(a, b), "GetDirection16({a:?}, {b:?})");
    }
}

// ---------------------------------------------------------------------------------------------
// player_test.cpp

/// `RunBlockTest`
fn run_block_test(c: &mut Ctx, frames: i32, flags: ItemSpecialEffect) -> i32 {
    c.players.Players[0]._pHFrames = frames as _;
    c.players.Players[0]._pIFlags = flags;
    diablo1_rs::player::start_plr_hit(c, 0, 5, false);
    let mut i = 1;
    while i < 100 {
        diablo1_rs::player::test_player_do_got_hit(c, 0);
        if c.players.Players[0]._pmode != PM_GOTHIT {
            break;
        }
        c.players.Players[0].AnimInfo.currentFrame += 1;
        i += 1;
    }
    i
}

#[test]
fn player_pm_do_got_hit() {
    let normal = ItemSpecialEffect::None_;
    let balance = ItemSpecialEffect::FastHitRecovery;
    let stability = ItemSpecialEffect::FasterHitRecovery;
    let harmony = ItemSpecialEffect::FastestHitRecovery;
    let (warrior, rogue, sorcerer) = (6, 7, 8);
    let data = [
        (6, warrior, normal),
        (7, rogue, normal),
        (8, sorcerer, normal),
        (5, warrior, balance),
        (6, rogue, balance),
        (7, sorcerer, balance),
        (4, warrior, stability),
        (5, rogue, stability),
        (6, sorcerer, stability),
        (3, warrior, harmony),
        (4, rogue, harmony),
        (5, sorcerer, harmony),
        (4, warrior, balance | stability),
        (5, rogue, balance | stability),
        (6, sorcerer, balance | stability),
        (3, warrior, balance | harmony),
        (4, rogue, balance | harmony),
        (5, sorcerer, balance | harmony),
        (3, warrior, stability | harmony),
        (4, rogue, stability | harmony),
        (5, sorcerer, stability | harmony),
    ];
    let mut c = setup();
    for (expected, frames, flags) in data {
        assert_eq!(expected, run_block_test(&mut c, frames, flags), "frames {frames} flags {flags:?}");
    }
}

#[test]
fn player_create_player() {
    let mut c = setup();
    diablo1_rs::player::create_player(&mut c, 0, HeroClass::Rogue);
    let p = &c.players.Players[0];
    let count_items = |items: &[diablo1_rs::items::Item]| items.iter().filter(|x| !x.is_empty()).count();
    assert_eq!(p._pSplLvl.iter().filter(|&&x| x != 0).count(), 0);
    assert_eq!(p.InvGrid.iter().filter(|&&x| x != 0).count(), 1);
    assert_eq!(count_items(&p.InvBody), 1);
    assert_eq!(count_items(&p.InvList), 1);
    assert_eq!(count_items(&p.SpdList), 2);
    assert_eq!(count_items(std::slice::from_ref(&p.HoldItem)), 0);
    assert_eq!((p.position.tile.x, p.position.tile.y, p.position.future.x, p.position.future.y), (0, 0, 0, 0));
    assert_eq!(p.plrlevel, 0);
    assert_eq!(p.destAction as i32, 0);
    assert_eq!(p._pName.as_str(), "");
    assert_eq!(p._pClass, HeroClass::Rogue);
    let ints: [(&str, i64, i64); 46] = [
        ("_pBaseStr", p._pBaseStr as i64, 20),
        ("_pStrength", p._pStrength as i64, 20),
        ("_pBaseMag", p._pBaseMag as i64, 15),
        ("_pMagic", p._pMagic as i64, 15),
        ("_pBaseDex", p._pBaseDex as i64, 30),
        ("_pDexterity", p._pDexterity as i64, 30),
        ("_pBaseVit", p._pBaseVit as i64, 20),
        ("_pVitality", p._pVitality as i64, 20),
        ("_pLevel", p._pLevel as i64, 1),
        ("_pStatPts", p._pStatPts as i64, 0),
        ("_pExperience", p._pExperience as i64, 0),
        ("_pGold", p._pGold as i64, 100),
        ("_pMaxHPBase", p._pMaxHPBase as i64, 2880),
        ("_pHPBase", p._pHPBase as i64, 2880),
        ("_pBaseToBlk", p._pBaseToBlk as i64, 20),
        ("_pMaxManaBase", p._pMaxManaBase as i64, 1440),
        ("_pManaBase", p._pManaBase as i64, 1440),
        ("_pMemSpells", p._pMemSpells as i64, 0),
        ("_pNumInv", p._pNumInv as i64, 1),
        ("wReflections", p.wReflections as i64, 0),
        ("pTownWarps", p.pTownWarps as i64, 0),
        ("pDungMsgs", p.pDungMsgs as i64, 0),
        ("pDungMsgs2", p.pDungMsgs2 as i64, 0),
        ("pLvlLoad", p.pLvlLoad as i64, 0),
        ("pDiabloKillLevel", p.pDiabloKillLevel as i64, 0),
        ("pManaShield", p.pManaShield as i64, 0),
        ("pDamAcFlags", p.pDamAcFlags.0 as i64, 0),
        ("_pmode", p._pmode as i64, 0),
        ("walkpath", p.walkpath.iter().filter(|&&x| x != 0).count() as i64, 0),
        ("queuedSpell.spellFrom", p.queuedSpell.spellFrom as i64, 0),
        ("_pAblSpells", p._pAblSpells as i64, 134217728),
        ("_pScrlSpells", p._pScrlSpells as i64, 0),
        ("_pBlockFlag", p._pBlockFlag as i64, 0),
        ("_pLightRad", p._pLightRad as i64, 10),
        ("_pDamageMod", p._pDamageMod as i64, 0),
        ("_pHitPoints", p._pHitPoints as i64, 2880),
        ("_pMaxHP", p._pMaxHP as i64, 2880),
        ("_pMana", p._pMana as i64, 1440),
        ("_pMaxMana", p._pMaxMana as i64, 1440),
        ("_pNextExper", p._pNextExper as i64, 2000),
        ("_pMagResist", p._pMagResist as i64, 0),
        ("_pFireResist", p._pFireResist as i64, 0),
        ("_pLghtResist", p._pLghtResist as i64, 0),
        ("_pLvlVisited", p._pLvlVisited.iter().filter(|&&x| x).count() as i64, 0),
        ("_pSLvlVisited", p._pSLvlVisited.iter().filter(|&&x| x).count() as i64, 0),
        ("_pIMinDam", p._pIMinDam as i64, 1),
    ];
    let more: [(&str, i64, i64); 16] = [
        ("_pIMaxDam", p._pIMaxDam as i64, 1),
        ("_pIAC", p._pIAC as i64, 0),
        ("_pIBonusDam", p._pIBonusDam as i64, 0),
        ("_pIBonusToHit", p._pIBonusToHit as i64, 0),
        ("_pIBonusAC", p._pIBonusAC as i64, 0),
        ("_pIBonusDamMod", p._pIBonusDamMod as i64, 0),
        ("_pISpells", p._pISpells as i64, 0),
        ("_pIFlags", p._pIFlags.0 as i64, 0),
        ("_pIGetHit", p._pIGetHit as i64, 0),
        ("_pISplLvlAdd", p._pISplLvlAdd as i64, 0),
        ("_pIEnAc", p._pIEnAc as i64, 0),
        ("_pIFMinDam", p._pIFMinDam as i64, 0),
        ("_pIFMaxDam", p._pIFMaxDam as i64, 0),
        ("_pILMinDam", p._pILMinDam as i64, 0),
        ("_pILMaxDam", p._pILMaxDam as i64, 0),
        ("walkpath again", p.walkpath.iter().filter(|&&x| x != 0).count() as i64, 0),
    ];
    let bad: Vec<_> = ints.iter().chain(more.iter()).filter(|(_, a, b)| a != b).map(|(f, a, b)| format!("{f}: got {a}, expected {b}")).collect();
    assert!(bad.is_empty(), "{bad:?}");
    assert_eq!(p.queuedSpell.spellId, SpellID::Null);
    assert_eq!(p.queuedSpell.spellType, SpellType::Skill);
    assert_eq!(p.inventorySpell, SpellID::Null);
    assert_eq!(p._pRSpell, SpellID::TrapDisarm);
    assert_eq!(p._pRSplType, SpellType::Skill);
    assert_eq!(p._pSBkSpell, SpellID::Null);
    assert_eq!(p._pSpellFlags, SpellFlag::None_);
}

// ---------------------------------------------------------------------------------------------
// dead_test.cpp

#[test]
fn corpses_add_corpse() {
    let mut c = setup();
    diablo1_rs::dead::add_corpse(&mut c, Point::new(21, 48), 8, Direction::West);
    assert_eq!(c.gendung.dCorpse[21][48] as i32, 8 + ((Direction::West as i32) << 5));
}

#[test]
fn corpses_add_corpse_oob() {
    let mut c = setup();
    diablo1_rs::dead::add_corpse(&mut c, Point::new(21, 48), (diablo1_rs::dead::MaxCorpses + 1) as i8, Direction::West);
    assert_eq!(c.gendung.dCorpse[21][48] as i32, (Direction::West as i32) << 5);
}

// ---------------------------------------------------------------------------------------------
// lighting_test.cpp

#[test]
fn lighting_crawl_tables() {
    use diablo1_rs::lighting::MaxCrawlRadius;
    let mut added = [[false; 40]; 40];
    let (x, y) = (20i32, 20i32);
    diablo1_rs::lighting::crawl_range::<()>(0, MaxCrawlRadius as u32, &mut |d| {
        let (dx, dy) = ((x + d.delta_x) as usize, (y + d.delta_y) as usize);
        assert!(!added[dx][dy], "displacement {}:{} added twice", d.delta_x, d.delta_y);
        added[dx][dy] = true;
        None
    });
    for i in -MaxCrawlRadius..=MaxCrawlRadius {
        for j in -MaxCrawlRadius..=MaxCrawlRadius {
            if added[(i + 20) as usize][(j + 20) as usize] {
                continue;
            }
            if i.abs() == MaxCrawlRadius && j.abs() == MaxCrawlRadius {
                continue; // Limit of the crawl table range
            }
            panic!("while checking location {i}:{j}");
        }
    }
}
