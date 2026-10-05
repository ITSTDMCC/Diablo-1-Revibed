//! The port of DevilutionX's `test/writehero_test.cpp`: unpack a level-50 rogue, check ~90 derived
//! player values, write her multiplayer save and require the file's SHA-256 to equal the one
//! DevilutionX 1.5.3 produces (pinning item regeneration, player stats, packing, PKWARE implode,
//! save encryption and the MPQ writer byte for byte).
//!
//! The item names are measured with the game's fonts, so the test needs `DIABLO_DATA_DIR`
//! (the folder with the game's MPQ files); skipped without it.

#![allow(non_snake_case)]

use diablo1_rs::ctx::Ctx;
use diablo1_rs::enums::*;
use diablo1_rs::items::Item;
use diablo1_rs::pack::{unpack_player, ItemPack, PlayerPack};
use diablo1_rs::platform::Platform;
use diablo1_rs::player::Player;

/// SHA-256 (FIPS 180-4), for the file hash the original test checks with picosha2.
fn sha256(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let mut h: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((data.len() as u64) * 8).to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[4 * i], chunk[4 * i + 1], chunk[4 * i + 2], chunk[4 * i + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7].wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [t1.wrapping_add(t2), v[0], v[1], v[2], v[3].wrapping_add(t1), v[4], v[5], v[6]];
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(v[i]);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

#[test]
fn sha256_known_answer() {
    assert_eq!(sha256(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}

const SPELL_DAT_VANILLA: [i32; 37] = [0, 1, 1, 4, 5, -1, 3, 3, 6, -1, 7, 6, 8, 9, 8, 9, -1, -1, -1, -1, 3, 11, -1, 14, -1, -1, -1, -1, -1, 8, 1, 1, -1, 2, 1, 14, 9];

fn pack(idx: u16, create_info: u16, b_id: u8, dur: u8, ch: u8, seed: u32) -> ItemPack {
    ItemPack { iSeed: seed, iCreateInfo: create_info, idx, bId: b_id, bDur: dur, bMDur: dur, bCh: ch, bMCh: ch, wValue: 0, dwBuff: 0 }
}

/// `PrepareInvSlot`
fn prepare_inv_slot(p: &mut PlayerPack, ret: &mut i8, pos: usize, size: i32) -> usize {
    *ret += 1;
    let r = *ret;
    match size {
        0 => p.InvGrid[pos] = r,
        1 => {
            p.InvGrid[pos] = r;
            p.InvGrid[pos - 10] = -r;
            p.InvGrid[pos - 20] = -r;
        }
        2 => {
            p.InvGrid[pos] = r;
            p.InvGrid[pos + 1] = -r;
            p.InvGrid[pos - 10] = -r;
            p.InvGrid[pos - 10 + 1] = -r;
            p.InvGrid[pos - 20] = -r;
            p.InvGrid[pos - 20 + 1] = -r;
        }
        3 => {
            p.InvGrid[pos] = r;
            p.InvGrid[pos + 1] = -r;
            p.InvGrid[pos - 10] = -r;
            p.InvGrid[pos - 10 + 1] = -r;
        }
        _ => panic!(),
    }
    (r - 1) as usize
}

/// `PackPlayerTest`
fn pack_player_test() -> PlayerPack {
    let magic = 1 + 2 * ITEM_QUALITY_MAGIC as u8;
    let mut p = PlayerPack::default();
    p.destAction = -1;
    p.plrlevel = 0;
    p.pExperience = 1583495809;
    p.pLevel = 50;
    p.px = 75;
    p.py = 68;
    p.targx = 75;
    p.targy = 68;
    p.pGold = 0;
    p.pStatPts = 0;
    p.pDiabloKillLevel = 3;
    for i in p.InvList.iter_mut() {
        i.idx = 0xFFFF;
    }
    for i in p.InvBody.iter_mut() {
        i.idx = 0xFFFF;
    }
    let seeds = [0x7C253335u32, 0x3EEFBFF8, 0x76AFB1A9, 0x38EB45FE, 0x1154E197, 0x5964B644, 0x76B58BEB, 0x002A6E5A];
    for (i, item) in p.SpdList.iter_mut().enumerate() {
        // PackItemFullRejuv
        *item = pack(diablo1_rs::objects::item_misc_id_idx(IMISC_FULLREJUV) as u16, 0, 2 * ITEM_QUALITY_NORMAL as u8, 0, 0, seeds[i]);
    }
    for i in 1..37 {
        if SPELL_DAT_VANILLA[i] != -1 {
            p.pMemSpells |= 1u64 << (i - 1);
            p.pSplLvl[i] = 15;
        }
    }
    let name = b"TestPlayer";
    p.pName[..name.len()].copy_from_slice(name);
    p.pClass = HeroClass::Rogue as u8;
    p.pBaseStr = 20 + 35;
    p.pBaseMag = 15 + 55;
    p.pBaseDex = 30 + 220;
    p.pBaseVit = 20 + 60;
    p.pHPBase = ((20 + 10) << 6) + ((20 + 10) << 5) + 48 * 128 + (60 << 6);
    p.pMaxHPBase = p.pHPBase;
    p.pManaBase = (15 << 6) + (15 << 5) + 48 * 128 + (55 << 6);
    p.pMaxManaBase = p.pManaBase;

    p.InvBody[INVLOC_HEAD as usize] = pack(52, 0x2DE, 1 + 2 * ITEM_QUALITY_UNIQUE as u8, 40, 0, 0x1C0C44B0); // PackItemUnique
    p.InvBody[INVLOC_RING_LEFT as usize] = pack(153, 0xDE, magic, 0, 0, 0x5B41AFA8);
    p.InvBody[INVLOC_RING_RIGHT as usize] = pack(153, 0xDE, magic, 0, 0, 0x1E41FEFC);
    p.InvBody[INVLOC_AMULET as usize] = pack(155, 0xDE, magic, 0, 0, 0x70A0383A);
    p.InvBody[INVLOC_CHEST as usize] = pack(70, 0xDE, magic, 90, 0, 0x63AAC49B);
    p.InvBody[INVLOC_HAND_LEFT as usize] = pack(145, 0x0814, magic, 60, 0, 0x449D8992); // PackItemBow

    let mut ret = 0i8;
    let staff = prepare_inv_slot(&mut p, &mut ret, 28, 2);
    p.InvList[staff] = pack(150, 0x2010, magic, 75, 12, 0x2A15243F);
    let sword = prepare_inv_slot(&mut p, &mut ret, 20, 1);
    p.InvList[sword] = pack(122, 0x081E, magic, 60, 0, 0x680FAC02);

    p._pNumInv = 2;
    p
}

fn count_items(items: &[Item]) -> usize {
    items.iter().filter(|x| !x.is_empty()).count()
}

/// `AssertPlayer`
fn assert_player(player: &Player) {
    assert_eq!(player._pSplLvl.iter().filter(|&&x| x != 0).count(), 23);
    assert_eq!(player.InvGrid.iter().filter(|&&x| x != 0).count(), 9);
    assert_eq!(count_items(&player.InvBody), 6);
    assert_eq!(count_items(&player.InvList), 2);
    assert_eq!(count_items(&player.SpdList), 8);
    assert_eq!(count_items(std::slice::from_ref(&player.HoldItem)), 0);

    assert_eq!((player.position.tile.x, player.position.tile.y), (75, 68));
    assert_eq!((player.position.future.x, player.position.future.y), (75, 68));
    assert_eq!(player.plrlevel, 0);
    assert_eq!(player.destAction as i32, -1);
    assert_eq!(player._pName.as_str(), "TestPlayer");
    assert_eq!(player._pClass, HeroClass::Rogue);
    let ints: [(&str, i64, i64); 50] = [
        ("_pBaseStr", player._pBaseStr as i64, 55),
        ("_pStrength", player._pStrength as i64, 124),
        ("_pBaseMag", player._pBaseMag as i64, 70),
        ("_pMagic", player._pMagic as i64, 80),
        ("_pBaseDex", player._pBaseDex as i64, 250),
        ("_pDexterity", player._pDexterity as i64, 281),
        ("_pBaseVit", player._pBaseVit as i64, 80),
        ("_pVitality", player._pVitality as i64, 90),
        ("_pLevel", player._pLevel as i64, 50),
        ("_pStatPts", player._pStatPts as i64, 0),
        ("_pExperience", player._pExperience as i64, 1583495809),
        ("_pGold", player._pGold as i64, 0),
        ("_pMaxHPBase", player._pMaxHPBase as i64, 12864),
        ("_pHPBase", player._pHPBase as i64, 12864),
        ("_pBaseToBlk", player._pBaseToBlk as i64, 20),
        ("_pMaxManaBase", player._pMaxManaBase as i64, 11104),
        ("_pManaBase", player._pManaBase as i64, 11104),
        ("_pMemSpells", player._pMemSpells as i64, 66309357295),
        ("_pNumInv", player._pNumInv as i64, 2),
        ("wReflections", player.wReflections as i64, 0),
        ("pTownWarps", player.pTownWarps as i64, 0),
        ("pDungMsgs", player.pDungMsgs as i64, 0),
        ("pDungMsgs2", player.pDungMsgs2 as i64, 0),
        ("pLvlLoad", player.pLvlLoad as i64, 0),
        ("pDiabloKillLevel", player.pDiabloKillLevel as i64, 3),
        ("pManaShield", player.pManaShield as i64, 0),
        ("pDamAcFlags", player.pDamAcFlags.0 as i64, 0),
        ("_pmode", player._pmode as i64, 0),
        ("walkpath!=0", player.walkpath.iter().filter(|&&x| x != 0).count() as i64, 25),
        ("_pgfxnum", player._pgfxnum as i64, 36),
        ("AnimInfo.ticksPerFrame", player.AnimInfo.ticksPerFrame as i64, 4),
        ("AnimInfo.tickCounterOfCurrentFrame", player.AnimInfo.tickCounterOfCurrentFrame as i64, 1),
        ("AnimInfo.numberOfFrames", player.AnimInfo.numberOfFrames as i64, 20),
        ("AnimInfo.currentFrame", player.AnimInfo.currentFrame as i64, 0),
        ("queuedSpell.spellFrom", player.queuedSpell.spellFrom as i64, 0),
        ("_pAblSpells", player._pAblSpells as i64, 134217728),
        ("_pScrlSpells", player._pScrlSpells as i64, 0),
        ("_pBlockFlag", player._pBlockFlag as i64, 0),
        ("_pLightRad", player._pLightRad as i64, 11),
        ("_pDamageMod", player._pDamageMod as i64, 101),
        ("_pHitPoints", player._pHitPoints as i64, 16640),
        ("_pMaxHP", player._pMaxHP as i64, 16640),
        ("_pMana", player._pMana as i64, 14624),
        ("_pMaxMana", player._pMaxMana as i64, 14624),
        ("_pNextExper", player._pNextExper as i64, 1310707109),
        ("_pMagResist", player._pMagResist as i64, 75),
        ("_pFireResist", player._pFireResist as i64, 16),
        ("_pLghtResist", player._pLghtResist as i64, 75),
        ("_pLvlVisited", player._pLvlVisited.iter().filter(|&&x| x).count() as i64, 0),
        ("_pSLvlVisited", player._pSLvlVisited.iter().filter(|&&x| x).count() as i64, 0),
    ];
    let more: [(&str, i64, i64); 25] = [
        ("_pNFrames", player._pNFrames as i64, 20),
        ("_pWFrames", player._pWFrames as i64, 8),
        ("_pAFrames", player._pAFrames as i64, 0),
        ("_pAFNum", player._pAFNum as i64, 0),
        ("_pSFrames", player._pSFrames as i64, 16),
        ("_pSFNum", player._pSFNum as i64, 12),
        ("_pHFrames", player._pHFrames as i64, 0),
        ("_pDFrames", player._pDFrames as i64, 20),
        ("_pBFrames", player._pBFrames as i64, 0),
        ("_pIMinDam", player._pIMinDam as i64, 1),
        ("_pIMaxDam", player._pIMaxDam as i64, 14),
        ("_pIAC", player._pIAC as i64, 115),
        ("_pIBonusDam", player._pIBonusDam as i64, 0),
        ("_pIBonusToHit", player._pIBonusToHit as i64, 0),
        ("_pIBonusAC", player._pIBonusAC as i64, 0),
        ("_pIBonusDamMod", player._pIBonusDamMod as i64, 0),
        ("_pISpells", player._pISpells as i64, 0),
        ("_pIFlags", player._pIFlags.0 as i64, 0),
        ("_pIGetHit", player._pIGetHit as i64, 0),
        ("_pISplLvlAdd", player._pISplLvlAdd as i64, 0),
        ("_pIEnAc", player._pIEnAc as i64, 0),
        ("_pIFMinDam", player._pIFMinDam as i64, 0),
        ("_pIFMaxDam", player._pIFMaxDam as i64, 0),
        ("_pILMinDam", player._pILMinDam as i64, 0),
        ("_pILMaxDam", player._pILMaxDam as i64, 0),
    ];
    let bad: Vec<_> = ints.iter().chain(more.iter()).filter(|(_, a, b)| a != b).map(|(f, a, b)| format!("{f}: got {a}, expected {b}")).collect();
    assert!(bad.is_empty(), "{bad:?}");
    assert_eq!(player.queuedSpell.spellId, SpellID::Invalid);
    assert_eq!(player.queuedSpell.spellType, SpellType::Invalid);
    assert_eq!(player.inventorySpell, SpellID::Null);
    assert_eq!(player._pRSpell, SpellID::Invalid);
    assert_eq!(player._pRSplType, SpellType::Invalid);
    assert_eq!(player._pSBkSpell, SpellID::Invalid);
    assert_eq!(player._pSpellFlags, SpellFlag::None_);
    assert!(player.uses_ranged_weapon());
    assert!(!player.pOriginalCathedral);
}

#[test]
fn writehero_pfile_write_hero() {
    let Some(data_dir) = std::env::var_os("DIABLO_DATA_DIR") else {
        eprintln!("skipped: set DIABLO_DATA_DIR to the folder with the game's MPQ files");
        return;
    };
    let work = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("writehero");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).unwrap();

    let mut ctx = Ctx::new(Platform::headless_from_env());
    let ctx = &mut ctx;
    ctx.paths.set_base_path(&format!("{}/", std::path::Path::new(&data_dir).display()));
    diablo1_rs::init::load_core_archives(ctx);
    diablo1_rs::init::load_game_archives(ctx);
    ctx.paths.set_pref_path(&format!("{}/", work.display()));

    ctx.init.gb_vanilla = true;
    ctx.init.gb_is_hellfire = false;
    ctx.init.gb_is_multiplayer = true;
    ctx.init.gb_is_hellfire_save_game = false;
    ctx.gendung.leveltype = diablo1_rs::levels::gendung::DungeonType::Town;
    ctx.loadsave.giNumberOfLevels = 17;

    ctx.players.Players.truncate(1);
    while ctx.players.Players.is_empty() {
        ctx.players.Players.push(Default::default());
    }
    ctx.players.MyPlayerId = 0;
    ctx.players.MyPlayer = Some(0);

    let mut info = diablo1_rs::pfile::UiHeroInfo { heroclass: HeroClass::Rogue, ..Default::default() };
    diablo1_rs::pfile::pfile_ui_save_create(ctx, &mut info);
    let pks = pack_player_test();
    unpack_player(ctx, &pks, 0);
    assert_player(&ctx.players.Players[0]);
    diablo1_rs::pfile::pfile_write_hero(ctx, false);

    let data = std::fs::read(work.join("multi_0.sv")).expect("multi_0.sv written");
    assert_eq!(sha256(&data), "a79367caae6192d54703168d82e0316aa289b2a33251255fad8abe34889c1d3a");
}
