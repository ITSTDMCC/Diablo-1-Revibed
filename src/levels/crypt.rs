//! `Source/levels/crypt.cpp`: the Hellfire crypt levels.
#![allow(non_upper_case_globals)]

use crate::ctx::Ctx;
use crate::engine::geometry::{Displacement, Point, Rectangle, Size};
use crate::enums::*;
use crate::levels::drlg_l1::{miniset, place_mini_set_random, select_chamber};
use crate::levels::gendung::*;

/// Globals of levels/crypt.cpp.
#[derive(Default)]
pub struct CryptState {
    pub UberRow: i32,
    pub UberCol: i32,
    pub IsUberRoomOpened: bool,
    pub IsUberLeverActivated: bool,
    pub UberDiabloMonsterIndex: i32,
}

/// `(UberRow, UberCol)`
pub fn uber_row_col(ctx: &Ctx) -> (i32, i32) {
    (ctx.crypt.UberRow, ctx.crypt.UberCol)
}

/// `IsUberRoomOpened = v`
pub fn set_is_uber_room_opened(ctx: &mut Ctx, v: bool) {
    ctx.crypt.IsUberRoomOpened = v;
}

/// `L5STAIRSUP`: stairs up.
pub fn l5_stairs_up() -> Miniset {
    miniset(4, 4, &[&[22, 22, 22, 22], &[2, 2, 2, 2], &[13, 13, 13, 13], &[13, 13, 13, 13]], &[&[0, 66, 23, 0], &[63, 64, 65, 0], &[0, 67, 68, 0], &[0, 0, 0, 0]])
}

fn l5_stairs_up_hf() -> Miniset {
    miniset(4, 5, &[&[22, 22, 22, 22], &[22, 22, 22, 22], &[2, 2, 2, 2], &[13, 13, 13, 13], &[13, 13, 13, 13]], &[&[0, 54, 23, 0], &[0, 53, 18, 0], &[55, 56, 57, 0], &[58, 59, 60, 0], &[0, 0, 0, 0]])
}

fn l5_stairs_down() -> Miniset {
    miniset(4, 5, &[&[13, 13, 13, 13], &[13, 13, 13, 13], &[13, 13, 13, 13], &[13, 13, 13, 13], &[13, 13, 13, 13]], &[&[0, 0, 52, 0], &[0, 48, 51, 0], &[0, 47, 50, 0], &[45, 46, 49, 0], &[0, 0, 0, 0]])
}

fn l5_stairs_town() -> Miniset {
    miniset(4, 5, &[&[22, 22, 22, 22], &[22, 22, 22, 22], &[2, 2, 2, 2], &[13, 13, 13, 13], &[13, 13, 13, 13]], &[&[0, 62, 23, 0], &[0, 61, 18, 0], &[63, 64, 65, 0], &[66, 67, 68, 0], &[0, 0, 0, 0]])
}

fn vwall_section() -> Miniset {
    miniset(1, 3, &[&[1], &[1], &[1]], &[&[91], &[90], &[89]])
}

fn hwall_section() -> Miniset {
    miniset(3, 1, &[&[2, 2, 2]], &[&[94, 93, 92]])
}

/// The 3x3 floor patterns with a single centre replacement (`CryptFloorLave`, `CryptPillar1`..`5`, `CryptStar`).
fn floor3x3(centre: u8) -> Miniset {
    miniset(3, 3, &[&[13, 13, 13], &[13, 13, 13], &[13, 13, 13]], &[&[0, 0, 0], &[0, centre, 0], &[0, 0, 0]])
}

// `Tile`
const VWall: u8 = 1;
const HWall: u8 = 2;
const Corner: u8 = 3;
const DWall: u8 = 4;
const DArch: u8 = 5;
const VWallEnd: u8 = 6;
const HWallEnd: u8 = 7;
const HArchEnd: u8 = 8;
const VArchEnd: u8 = 9;
const HArchVWall: u8 = 10;
const VArch: u8 = 11;
const HArch: u8 = 12;
const Floor: u8 = 13;
const HWallVArch: u8 = 14;
const Pillar: u8 = 15;
const Pillar1: u8 = 16;
const Pillar2: u8 = 17;
const DirtHwall: u8 = 18;
const DirtVwall: u8 = 19;
const DirtCorner: u8 = 21;
const DirtHWallEnd: u8 = 23;
const DirtVWallEnd: u8 = 24;
const DirtHWall2: u8 = 82;
const DirtVWall2: u8 = 83;
const DirtCorner2: u8 = 85;
const DirtHWallEnd2: u8 = 87;
const DirtVWallEnd2: u8 = 88;
const VWall5: u8 = 89;
const VWall6: u8 = 90;
const VWall7: u8 = 91;
const HWall5: u8 = 92;
const VArch5: u8 = 95;
const HArch5: u8 = 96;
const Floor6: u8 = 97;
const Floor7: u8 = 98;
const Floor8: u8 = 99;
const Floor9: u8 = 100;
const Floor10: u8 = 101;
const VWall2: u8 = 112;
const HWall2: u8 = 113;
const Corner2: u8 = 114;
const DWall2: u8 = 115;
const DArch2: u8 = 116;
const VWallEnd2: u8 = 117;
const HWallEnd2: u8 = 118;
const HArchEnd2: u8 = 119;
const VArchEnd2: u8 = 120;
const HArchVWall2: u8 = 121;
const VArch2: u8 = 122;
const HArch2: u8 = 123;
const Floor2: u8 = 124;
const HWallVArch2: u8 = 125;
const Pillar3: u8 = 126;
const Pillar4: u8 = 127;
const Pillar5: u8 = 128;
const VWall3: u8 = 129;
const HWall3: u8 = 130;
const Corner3: u8 = 131;
const DWall3: u8 = 132;
const DArch3: u8 = 133;
const VWallEnd3: u8 = 134;
const HWallEnd3: u8 = 135;
const HArchEnd3: u8 = 136;
const VArchEnd3: u8 = 137;
const HArchVWall3: u8 = 138;
const VArch3: u8 = 139;
const HArch3: u8 = 140;
const Floor3: u8 = 141;
const HWallVArch3: u8 = 142;
const Pillar6: u8 = 143;
const Pillar7: u8 = 144;
const Pillar8: u8 = 145;
const VWall4: u8 = 146;
const HWall4: u8 = 147;
const Corner4: u8 = 148;
const DWall4: u8 = 149;
const DArch4: u8 = 150;
const VWallEnd4: u8 = 151;
const HWallEnd4: u8 = 152;
const HArchEnd4: u8 = 153;
const VArchEnd4: u8 = 154;
const HArchVWall4: u8 = 155;
const VArch4: u8 = 156;
const HArch4: u8 = 157;
const Floor4: u8 = 158;
const HWallVArch4: u8 = 159;
const Pillar9: u8 = 160;
const Pillar10: u8 = 161;
const Pillar11: u8 = 162;
const Floor11: u8 = 163;
const Floor12: u8 = 164;
const Floor13: u8 = 165;
const Floor14: u8 = 166;
const PillarHalf: u8 = 167;
const VWall8: u8 = 173;
const VWall9: u8 = 174;
const VWall10: u8 = 175;
const VWall11: u8 = 176;
const VWall12: u8 = 177;
const VWall13: u8 = 178;
const HWall8: u8 = 179;
const HWall9: u8 = 180;
const HWall10: u8 = 181;
const HWall11: u8 = 182;
const HWall12: u8 = 183;
const HWall13: u8 = 184;
const VArch6: u8 = 185;
const VArch7: u8 = 186;
const HArch6: u8 = 187;
const HArch7: u8 = 188;
const Floor15: u8 = 189;
const Floor16: u8 = 190;
const Floor17: u8 = 191;
const Pillar12: u8 = 192;
const Floor18: u8 = 193;
const Floor19: u8 = 194;
const Floor20: u8 = 195;
const Floor21: u8 = 196;
const Floor22: u8 = 197;
const Floor23: u8 = 198;
const VDemon: u8 = 199;
const HDemon: u8 = 200;
const VSuccubus: u8 = 201;
const HSuccubus: u8 = 202;
const Shadow1: u8 = 203;
const Shadow2: u8 = 204;
const Shadow3: u8 = 205;
const Shadow4: u8 = 206;
const Shadow5: u8 = 207;
const Shadow6: u8 = 208;
const Shadow7: u8 = 209;
const Shadow8: u8 = 210;
const Shadow9: u8 = 211;
const Shadow10: u8 = 212;
const Shadow11: u8 = 213;
const Shadow12: u8 = 214;
const Shadow13: u8 = 215;
const Shadow14: u8 = 216;
const Shadow15: u8 = 217;

/// `ReplaceTile` pairs (search, replace).
const STATUES: [(u8, u8); 4] = [(VWall, VDemon), (VWall, VSuccubus), (HWall, HDemon), (HWall, HSuccubus)];

const CRACKED_TILES: [(u8, u8); 17] = [
    (VWall, VWall2),
    (HWall, HWall2),
    (Corner, Corner2),
    (DWall, DWall2),
    (DArch, DArch2),
    (VWallEnd, VWallEnd2),
    (HWallEnd, HWallEnd2),
    (HArchEnd, HArchEnd2),
    (VArchEnd, VArchEnd2),
    (HArchVWall, HArchVWall2),
    (VArch, VArch2),
    (HArch, HArch2),
    (Floor, Floor2),
    (HWallVArch, HWallVArch2),
    (Pillar, Pillar3),
    (Pillar1, Pillar4),
    (Pillar2, Pillar5),
];

const BROKEN_TILES: [(u8, u8); 17] = [
    (VWall, VWall3),
    (HWall, HWall3),
    (Corner, Corner3),
    (DWall, DWall3),
    (DArch, DArch3),
    (VWallEnd, VWallEnd3),
    (HWallEnd, HWallEnd3),
    (HArchEnd, HArchEnd3),
    (VArchEnd, VArchEnd3),
    (HArchVWall, HArchVWall3),
    (VArch, VArch3),
    (HArch, HArch3),
    (Floor, Floor3),
    (HWallVArch, HWallVArch3),
    (Pillar, Pillar6),
    (Pillar1, Pillar7),
    (Pillar2, Pillar8),
];

const LEAKING_TILES: [(u8, u8); 17] = [
    (VWall, VWall4),
    (HWall, HWall4),
    (Corner, Corner4),
    (DWall, DWall4),
    (DArch, DArch4),
    (VWallEnd, VWallEnd4),
    (HWallEnd, HWallEnd4),
    (HArchEnd, HArchEnd4),
    (VArchEnd, VArchEnd4),
    (HArchVWall, HArchVWall4),
    (VArch, VArch4),
    (HArch, HArch4),
    (Floor, Floor4),
    (HWallVArch, HWallVArch4),
    (Pillar, Pillar9),
    (Pillar1, Pillar10),
    (Pillar2, Pillar11),
];

const SUBSTITIONS1_TILES: [(u8, u8); 26] = [
    (VArch, VArch6),
    (HArch, HArch6),
    (VArch, VArch7),
    (HArch, HArch7),
    (VWall5, VWall8),
    (VWall5, VWall9),
    (VWall6, VWall10),
    (VWall6, VWall11),
    (VWall7, VWall12),
    (VWall7, VWall13),
    (HWall5, HWall8),
    (HWall5, HWall9),
    (HWall5, HWall10),
    (HWall5, HWall11),
    (HWall5, HWall12),
    (HWall5, HWall13),
    (Floor7, Floor15),
    (Floor7, Floor16),
    (Floor6, Floor17),
    (Pillar, Pillar12),
    (Floor8, Floor18),
    (Floor8, Floor19),
    (Floor9, Floor20),
    (Floor10, Floor21),
    (Floor10, Floor22),
    (Floor10, Floor23),
];

const SUBSTITION1_FLOOR: [(u8, u8); 4] = [(Floor, Floor11), (Floor, Floor12), (Floor, Floor13), (Floor, Floor14)];
const SUBSTITION2_FLOOR: [(u8, u8); 4] = [(Floor, Floor6), (Floor, Floor7), (Floor, Floor8), (Floor, Floor9)];

/// Original: `ApplyCryptShadowsPatterns` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::ApplyCryptShadowsPatterns() sha=63a24b12d5c3
fn apply_crypt_shadows_patterns(ctx: &mut Ctx) {
    let d = &mut ctx.gendung.dungeon;
    for j in 1..DMAXY {
        for i in 1..DMAXX {
            let tile = d[i][j];
            let mut set = |dx: usize, dy: usize, v: u8| {
                if d[i - dx][j - dy] == Floor {
                    d[i - dx][j - dy] = v;
                }
            };
            match tile {
                DArch | DArch2 | DArch3 => {
                    set(1, 0, Shadow1);
                    set(1, 1, Shadow2);
                    set(0, 1, Shadow3);
                }
                HWallEnd | HWallEnd2 | HWallEnd3 | HWallEnd4 | Pillar | Pillar2 | Pillar3 | Pillar5 | Pillar9 => {
                    set(1, 0, Shadow4);
                    set(1, 1, Shadow5);
                }
                HArchEnd | HArchEnd2 | HArchEnd3 | HArchEnd4 | HWallVArch | HWallVArch2 | HWallVArch3 | HWallVArch4 | VArch | VArch4 | VArch5 | VArch6 | VArch7 => {
                    set(1, 0, Shadow1);
                    set(1, 1, Shadow2);
                }
                VArchEnd | VArchEnd2 | VArchEnd4 => {
                    set(1, 0, Shadow4);
                    set(1, 1, Shadow5);
                    set(0, 1, Shadow3);
                }
                HArch | HArch2 | HArchVWall | HArchVWall2 | HArchVWall3 | HArchVWall4 => {
                    set(0, 1, Shadow3);
                }
                HArch5 | HArch6 => {
                    set(0, 1, Shadow6);
                }
                VArch2 => {
                    set(1, 0, Shadow9);
                    set(1, 1, Shadow10);
                }
                VArchEnd3 => {
                    set(1, 0, Shadow11);
                    set(1, 1, Shadow12);
                    set(0, 1, Shadow3);
                }
                VArch3 => {
                    set(1, 0, Shadow13);
                    set(1, 1, Shadow14);
                }
                HArch3 | HArch4 => {
                    set(0, 1, Shadow15);
                }
                Pillar6 | Pillar8 => {
                    set(1, 0, Shadow11);
                    set(1, 1, Shadow12);
                }
                DArch4 => {
                    set(1, 0, Shadow1);
                    set(1, 1, Shadow2);
                    set(0, 1, Shadow15);
                }
                Pillar11 | Pillar12 | PillarHalf => {
                    set(1, 0, Shadow7);
                    set(1, 1, Shadow8);
                }
                _ => {}
            }
        }
    }
}

/// Original: `PlaceMiniSetRandom1x1` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::PlaceMiniSetRandom1x1(uint8_t search, uint8_t replace, int rndper) sha=6a343fa7f345
fn place_mini_set_random_1x1(ctx: &mut Ctx, search: u8, replace: u8, rndper: i32) {
    place_mini_set_random(ctx, &miniset(1, 1, &[&[search]], &[&[replace]]), rndper);
}

fn place_pairs(ctx: &mut Ctx, pairs: &[(u8, u8)], rndper: i32) {
    for &(s, r) in pairs {
        place_mini_set_random_1x1(ctx, s, r, rndper);
    }
}

/// Original: `CryptCracked` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::CryptCracked(int rndper) sha=79098732cd6a
fn crypt_cracked(ctx: &mut Ctx, rndper: i32) {
    place_pairs(ctx, &CRACKED_TILES, rndper);
}

/// Original: `CryptBroken` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::CryptBroken(int rndper) sha=2fb3931cad09
fn crypt_broken(ctx: &mut Ctx, rndper: i32) {
    place_pairs(ctx, &BROKEN_TILES, rndper);
}

/// Original: `CryptLeaking` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::CryptLeaking(int rndper) sha=0a984f7e9509
fn crypt_leaking(ctx: &mut Ctx, rndper: i32) {
    place_pairs(ctx, &LEAKING_TILES, rndper);
}

/// Original: `CryptSubstitions1` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::CryptSubstitions1(int rndper) sha=5047c302ee20
fn crypt_substitions1(ctx: &mut Ctx, rndper: i32) {
    place_pairs(ctx, &SUBSTITIONS1_TILES, rndper);
}

/// Original: `CryptSubstitions2` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::CryptSubstitions2(int rndper) sha=b7d93666af77
fn crypt_substitions2(ctx: &mut Ctx, rndper: i32) {
    for centre in [167, 168, 169, 170, 171, 172] {
        // CryptPillar1..5, CryptStar
        place_mini_set_random(ctx, &floor3x3(centre), rndper);
    }
    place_pairs(ctx, &SUBSTITION1_FLOOR, rndper);
}

/// Original: `CryptFloor` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::CryptFloor(int rndper) sha=81f8344fa5da
fn crypt_floor(ctx: &mut Ctx, rndper: i32) {
    place_pairs(ctx, &SUBSTITION2_FLOOR, rndper);
}

/// Original: `devilution::InitCryptPieces` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::InitCryptPieces() sha=1c39f1ebd954
pub fn init_crypt_pieces(ctx: &mut Ctx) {
    let g = &mut ctx.gendung;
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            if g.dPiece[i][j] == 76 {
                g.dSpecial[i][j] = 1;
            } else if g.dPiece[i][j] == 79 {
                g.dSpecial[i][j] = 2;
            }
        }
    }
}

/// Original: `devilution::SetCryptRoom` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::SetCryptRoom() sha=bb37cbc9ad99
pub fn set_crypt_room(ctx: &mut Ctx) {
    let position = select_chamber(ctx);
    ctx.crypt.UberRow = 2 * position.x + 6;
    ctx.crypt.UberCol = 2 * position.y + 8;
    ctx.crypt.IsUberRoomOpened = false;
    ctx.crypt.IsUberLeverActivated = false;
    let dun_data = load_u16_file(ctx, "nlevels\\l5data\\uberroom.dun");
    ctx.gendung.SetPiece = Rectangle::new(position, Size::new(dun_data[0] as i32, dun_data[1] as i32));
    place_dun_tiles(ctx, &dun_data, position, 0);
}

/// Original: `devilution::SetCornerRoom` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::SetCornerRoom() sha=5462fcc33d9f
pub fn set_corner_room(ctx: &mut Ctx) {
    let position = select_chamber(ctx);
    let dun_data = load_u16_file(ctx, "nlevels\\l5data\\cornerstone.dun");
    ctx.gendung.SetPiece = Rectangle::new(position, Size::new(dun_data[0] as i32, dun_data[1] as i32));
    place_dun_tiles(ctx, &dun_data, position, 0);
}

/// Original: `devilution::FixCryptDirtTiles` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::FixCryptDirtTiles() sha=ff6841f4ad9e
pub fn fix_crypt_dirt_tiles(ctx: &mut Ctx) {
    let d = &mut ctx.gendung.dungeon;
    for j in 0..DMAXY - 1 {
        for i in 0..DMAXX - 1 {
            if d[i][j] == DirtVwall {
                d[i][j] = DirtVWall2;
            }
            if d[i][j] == DirtCorner {
                d[i][j] = DirtCorner2;
            }
            if d[i][j] == DirtHWallEnd {
                d[i][j] = DirtHWallEnd2;
            }
            if d[i][j] == DirtVWallEnd {
                d[i][j] = DirtVWallEnd2;
            }
            if d[i][j] == DirtHwall {
                d[i][j] = DirtHWall2;
            }
        }
    }
}

/// Original: `devilution::PlaceCryptStairs` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::PlaceCryptStairs(lvl_entry entry) sha=5ce0ec93a21b
pub fn place_crypt_stairs(ctx: &mut Ctx, entry: lvl_entry) -> bool {
    let mut success = true;
    let tries = (DMAXX * DMAXY) as i32;
    // Place stairs up
    let up = if ctx.gendung.currlevel != 21 { l5_stairs_up_hf() } else { l5_stairs_town() };
    match place_mini_set(ctx, &up, tries, true) {
        None => success = false,
        Some(position) => {
            if entry == ENTRY_MAIN || entry == ENTRY_TWARPDN {
                ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(3, 5);
            }
        }
    }
    // Place stairs down
    if ctx.gendung.currlevel != 24 {
        match place_mini_set(ctx, &l5_stairs_down(), tries, true) {
            None => success = false,
            Some(position) => {
                if entry == ENTRY_PREV {
                    ctx.gendung.ViewPosition = position.mega_to_world() + Displacement::new(3, 7);
                }
            }
        }
    }
    success
}

/// Original: `devilution::CryptSubstitution` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::CryptSubstitution() sha=38050a2a3353
pub fn crypt_substitution(ctx: &mut Ctx) {
    place_pairs(ctx, &STATUES, 10);
    place_mini_set_random_1x1(ctx, VArch, VArch5, 95);
    place_mini_set_random_1x1(ctx, HArch, HArch5, 95);
    place_mini_set_random(ctx, &vwall_section(), 100);
    place_mini_set_random(ctx, &hwall_section(), 100);
    place_mini_set_random(ctx, &floor3x3(101), 60); // CryptFloorLave
    apply_crypt_shadows_patterns(ctx);
    match ctx.gendung.currlevel {
        21 => {
            crypt_cracked(ctx, 30);
            crypt_broken(ctx, 15);
            crypt_leaking(ctx, 5);
            apply_crypt_shadows_patterns(ctx);
            crypt_floor(ctx, 10);
            crypt_substitions1(ctx, 5);
            crypt_substitions2(ctx, 20);
        }
        22 => {
            crypt_floor(ctx, 10);
            crypt_substitions1(ctx, 10);
            crypt_substitions2(ctx, 20);
            crypt_cracked(ctx, 30);
            crypt_broken(ctx, 20);
            crypt_leaking(ctx, 10);
            apply_crypt_shadows_patterns(ctx);
        }
        23 => {
            crypt_floor(ctx, 10);
            crypt_substitions1(ctx, 15);
            crypt_substitions2(ctx, 30);
            crypt_cracked(ctx, 30);
            crypt_broken(ctx, 20);
            crypt_leaking(ctx, 15);
            apply_crypt_shadows_patterns(ctx);
        }
        _ => {
            crypt_floor(ctx, 10);
            crypt_substitions1(ctx, 20);
            crypt_substitions2(ctx, 30);
            crypt_cracked(ctx, 30);
            crypt_broken(ctx, 20);
            crypt_leaking(ctx, 20);
            apply_crypt_shadows_patterns(ctx);
        }
    }
}

/// Original: `devilution::SetCryptSetPieceRoom` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::SetCryptSetPieceRoom() sha=6014b4350c63
pub fn set_crypt_set_piece_room(ctx: &mut Ctx) {
    let (minp, maxp) = (ctx.gendung.dminPosition, ctx.gendung.dmaxPosition);
    for j in minp.y..maxp.y {
        for i in minp.x..maxp.x {
            let p = ctx.gendung.dPiece[i as usize][j as usize];
            if p == 289 {
                ctx.crypt.UberRow = i;
                ctx.crypt.UberCol = j;
            }
            if p == 316 {
                ctx.items.CornerStone.position = Point::new(i, j);
            }
        }
    }
}

/// Original: `devilution::PlaceCryptLights` (levels/crypt.cpp).
// @port levels/crypt.cpp|devilution::PlaceCryptLights() sha=eb027cf47d11
pub fn place_crypt_lights(ctx: &mut Ctx) {
    const LAVA_TILES: [u16; 83] = [
        124, 128, 130, 132, 133, 134, 135, 139, 141, 143, 145, 156, 164, 166, 167, 168, 169, 170, 182, 190, 192, 195, 196, 199, 200, 254, 266, 273, 276, 281, 282, 283, 284, 285, 286, 287, 288, 290,
        302, 316, 434, 435, 436, 437, 445, 446, 447, 453, 457, 460, 466, 470, 477, 479, 484, 485, 486, 490, 507, 537, 557, 559, 561, 563, 564, 568, 569, 572, 578, 580, 584, 585, 589, 592, 593, 594,
        595, 596, 597, 598, 599, 600, 601,
    ];
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            if LAVA_TILES.contains(&ctx.gendung.dPiece[i][j]) {
                crate::lighting::do_lighting(ctx, Point::new(i as i32, j as i32), 3, Displacement::default());
            }
        }
    }
}
