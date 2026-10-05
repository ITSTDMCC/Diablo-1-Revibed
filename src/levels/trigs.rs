//! `Source/levels/trigs.cpp`: level entrances and exits (stairs, town warps, quest portals).

use crate::ctx::Ctx;
use crate::engine::geometry::{Displacement, Point};
use crate::enums::*;
use crate::levels::gendung::{DungeonType, MAXDUNX, MAXDUNY};
use crate::utils::language::tr;

pub const MAXTRIGGERS: usize = 7;

/// `TriggerStruct`
#[derive(Clone, Copy, Debug, Default)]
pub struct TriggerStruct {
    pub position: Point,
    pub _tmsg: interface_mode,
    pub _tlvl: i32,
}

/// Globals of trigs.cpp.
#[derive(Default)]
pub struct TrigsState {
    pub trigflag: bool,
    pub numtrigs: i32,
    pub trigs: [TriggerStruct; MAXTRIGGERS],
    pub TWarpFrom: i32,
}

/// Specifies the dungeon piece IDs which constitute stairways leading down to the cathedral from town.
const TOWN_DOWN_LIST: [u16; 10] = [715, 714, 718, 719, 720, 722, 723, 724, 725, 726];
/// Specifies the dungeon piece IDs which constitute stairways leading down to the catacombs from town.
const TOWN_WARP1_LIST: [u16; 12] = [1170, 1171, 1172, 1173, 1174, 1175, 1176, 1177, 1178, 1180, 1182, 1184];
const TOWN_CRYPT_LIST: [u16; 8] = [1330, 1331, 1332, 1333, 1334, 1335, 1336, 1337];
const TOWN_HIVE_LIST: [u16; 4] = [1306, 1307, 1308, 1309];
const L1_UP_LIST: [u16; 11] = [126, 128, 129, 130, 131, 132, 134, 136, 137, 138, 139];
const L1_DOWN_LIST: [u16; 9] = [105, 106, 107, 108, 109, 111, 113, 114, 117];
const L2_UP_LIST: [u16; 2] = [265, 266];
const L2_DOWN_LIST: [u16; 4] = [268, 269, 270, 271];
const L2_TWARP_UP_LIST: [u16; 2] = [557, 558];
const L3_UP_LIST: [u16; 14] = [169, 170, 171, 172, 173, 174, 175, 176, 177, 178, 179, 180, 181, 182];
const L3_DOWN_LIST: [u16; 8] = [161, 162, 163, 164, 165, 166, 167, 168];
const L3_TWARP_UP_LIST: [u16; 14] = [181, 547, 548, 549, 550, 551, 552, 553, 554, 555, 556, 557, 558, 559];
const L4_UP_LIST: [u16; 3] = [81, 82, 89];
const L4_DOWN_LIST: [u16; 5] = [119, 129, 130, 131, 132];
const L4_TWARP_UP_LIST: [u16; 3] = [420, 421, 428];
#[rustfmt::skip]
const L4_PENTA_LIST: [u16; 32] = [352, 353, 354, 355, 356, 357, 358, 359, 360, 361, 362, 363, 364, 365, 366, 367, 368, 369, 370, 371, 372, 373, 374, 375, 376, 377, 378, 379, 380, 381, 382, 383];
const L5_TWARP_UP_LIST: [u16; 9] = [171, 172, 173, 174, 175, 176, 177, 178, 183];
const L5_UP_LIST: [u16; 10] = [148, 149, 150, 151, 152, 153, 154, 156, 157, 158];
const L5_DOWN_LIST: [u16; 9] = [124, 125, 128, 130, 131, 134, 135, 139, 141];
const L6_TWARP_UP_LIST: [u16; 14] = [78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91];
const L6_UP_LIST: [u16; 14] = [64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77];
const L6_DOWN_LIST: [u16; 8] = [56, 57, 58, 59, 60, 61, 62, 63];

fn push_trig(ctx: &mut Ctx, position: Point, msg: interface_mode, lvl: Option<i32>) {
    let n = ctx.trigs.numtrigs as usize;
    let t = &mut ctx.trigs.trigs[n];
    t.position = position;
    t._tmsg = msg;
    if let Some(l) = lvl {
        t._tlvl = l;
    }
    ctx.trigs.numtrigs += 1;
}

/// Original: `devilution::InitNoTriggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitNoTriggers() sha=a3adcb245be1
pub fn init_no_triggers(ctx: &mut Ctx) {
    ctx.trigs.numtrigs = 0;
    ctx.trigs.trigflag = false;
}

/// Original: `devilution::IsWarpOpen` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::IsWarpOpen(dungeon_type type) sha=7238668baba2
pub fn is_warp_open(ctx: &Ctx, type_: DungeonType) -> bool {
    if ctx.init.gb_is_spawn {
        return false;
    }
    if ctx.init.gb_is_multiplayer && type_ != DungeonType::Nest {
        // Opening the nest is part of in town quest
        return true;
    }
    let my_player = &ctx.players.Players[ctx.players.MyPlayer.expect("MyPlayer")];
    if type_ == DungeonType::Catacombs && (my_player.pTownWarps & 1) != 0 {
        return true;
    }
    if type_ == DungeonType::Caves && (my_player.pTownWarps & 2) != 0 {
        return true;
    }
    if type_ == DungeonType::Hell && (my_player.pTownWarps & 4) != 0 {
        return true;
    }
    if ctx.init.gb_is_hellfire {
        if type_ == DungeonType::Catacombs && my_player._pLevel >= 10 {
            return true;
        }
        if type_ == DungeonType::Caves && my_player._pLevel >= 15 {
            return true;
        }
        if type_ == DungeonType::Hell && my_player._pLevel >= 20 {
            return true;
        }
        if type_ == DungeonType::Nest && matches!(ctx.quests.Quests[Q_FARMER as usize]._qactive, QUEST_DONE | QUEST_HIVE_DONE) {
            return true;
        }
        if type_ == DungeonType::Crypt && ctx.quests.Quests[Q_GRAVE as usize]._qactive == QUEST_DONE {
            return true;
        }
    }
    false
}

/// Original: `devilution::InitTownTriggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitTownTriggers() sha=dd0214ad38ab
pub fn init_town_triggers(ctx: &mut Ctx) {
    ctx.trigs.numtrigs = 0;
    // Cathedral
    push_trig(ctx, Point::new(25, 29), WM_DIABNEXTLVL, None);
    if is_warp_open(ctx, DungeonType::Catacombs) {
        push_trig(ctx, Point::new(49, 21), WM_DIABTOWNWARP, Some(5));
    }
    if is_warp_open(ctx, DungeonType::Caves) {
        push_trig(ctx, Point::new(17, 69), WM_DIABTOWNWARP, Some(9));
    }
    if is_warp_open(ctx, DungeonType::Hell) {
        push_trig(ctx, Point::new(41, 80), WM_DIABTOWNWARP, Some(13));
    }
    if is_warp_open(ctx, DungeonType::Nest) {
        push_trig(ctx, Point::new(80, 62), WM_DIABTOWNWARP, Some(17));
    }
    if is_warp_open(ctx, DungeonType::Crypt) {
        push_trig(ctx, Point::new(36, 24), WM_DIABTOWNWARP, Some(21));
    }
    ctx.trigs.trigflag = false;
}

/// Scans `dPiece` in the original's order (`j` outer, `i` inner) and lets `f` add triggers.
fn scan_pieces(ctx: &mut Ctx, f: impl Fn(&mut Ctx, u16, Point)) {
    for j in 0..MAXDUNY {
        for i in 0..MAXDUNX {
            let piece = ctx.gendung.dPiece[i][j];
            f(ctx, piece, Point::new(i as i32, j as i32));
        }
    }
}

/// Original: `devilution::InitL1Triggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitL1Triggers() sha=b48673bd083d
pub fn init_l1_triggers(ctx: &mut Ctx) {
    ctx.trigs.numtrigs = 0;
    scan_pieces(ctx, |ctx, piece, p| {
        if piece == 128 {
            push_trig(ctx, p, WM_DIABPREVLVL, None);
        }
        if piece == 114 {
            push_trig(ctx, p, WM_DIABNEXTLVL, None);
        }
    });
    ctx.trigs.trigflag = false;
}

/// Original: `devilution::InitL2Triggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitL2Triggers() sha=aabf1b332467
pub fn init_l2_triggers(ctx: &mut Ctx) {
    ctx.trigs.numtrigs = 0;
    scan_pieces(ctx, |ctx, piece, p| {
        if piece == 266 {
            let schamb = ctx.quests.Quests[Q_SCHAMB as usize].position;
            if !crate::quests::is_quest_available(ctx, Q_SCHAMB) || p.x != schamb.x || p.y != schamb.y {
                push_trig(ctx, p, WM_DIABPREVLVL, None);
            }
        }
        if piece == 558 {
            push_trig(ctx, p, WM_DIABTWARPUP, Some(0));
        }
        if piece == 270 {
            push_trig(ctx, p, WM_DIABNEXTLVL, None);
        }
    });
    ctx.trigs.trigflag = false;
}

/// Original: `devilution::InitL3Triggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitL3Triggers() sha=e2524b7cf295
pub fn init_l3_triggers(ctx: &mut Ctx) {
    ctx.trigs.numtrigs = 0;
    scan_pieces(ctx, |ctx, piece, p| {
        if piece == 170 {
            push_trig(ctx, p, WM_DIABPREVLVL, None);
        }
        if piece == 167 {
            push_trig(ctx, p, WM_DIABNEXTLVL, None);
        }
        if piece == 548 {
            push_trig(ctx, p, WM_DIABTWARPUP, None);
        }
    });
    ctx.trigs.trigflag = false;
}

/// Original: `devilution::InitL4Triggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitL4Triggers() sha=0fd6037cfac1
pub fn init_l4_triggers(ctx: &mut Ctx) {
    ctx.trigs.numtrigs = 0;
    scan_pieces(ctx, |ctx, piece, p| {
        if piece == 82 {
            push_trig(ctx, p, WM_DIABPREVLVL, None);
        }
        if piece == 421 {
            push_trig(ctx, p, WM_DIABTWARPUP, Some(0));
        }
        if piece == 119 {
            push_trig(ctx, p, WM_DIABNEXTLVL, None);
        }
    });
    scan_pieces(ctx, |ctx, piece, p| {
        if piece == 369 && ctx.quests.Quests[Q_BETRAYER as usize]._qactive == QUEST_DONE {
            push_trig(ctx, p, WM_DIABNEXTLVL, None);
        }
    });
    ctx.trigs.trigflag = false;
}

/// Original: `devilution::InitHiveTriggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitHiveTriggers() sha=d988c1ccab82
pub fn init_hive_triggers(ctx: &mut Ctx) {
    ctx.trigs.numtrigs = 0;
    scan_pieces(ctx, |ctx, piece, p| {
        if piece == 65 {
            push_trig(ctx, p, WM_DIABPREVLVL, None);
        }
        if piece == 62 {
            push_trig(ctx, p, WM_DIABNEXTLVL, None);
        }
        if piece == 79 {
            push_trig(ctx, p, WM_DIABTWARPUP, None);
        }
    });
    ctx.trigs.trigflag = false;
}

/// Original: `devilution::InitCryptTriggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitCryptTriggers() sha=a53b8db87255
pub fn init_crypt_triggers(ctx: &mut Ctx) {
    ctx.trigs.numtrigs = 0;
    scan_pieces(ctx, |ctx, piece, p| {
        if piece == 183 {
            push_trig(ctx, p, WM_DIABTWARPUP, Some(0));
        }
        if piece == 157 {
            push_trig(ctx, p, WM_DIABPREVLVL, None);
        }
        if piece == 125 {
            push_trig(ctx, p, WM_DIABNEXTLVL, None);
        }
    });
    ctx.trigs.trigflag = false;
}

fn init_single_return_trigger(ctx: &mut Ctx, position: Point) {
    ctx.trigs.trigflag = false;
    ctx.trigs.numtrigs = 1;
    ctx.trigs.trigs[0].position = position;
    ctx.trigs.trigs[0]._tmsg = WM_DIABRTNLVL;
}

/// Original: `devilution::InitSKingTriggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitSKingTriggers() sha=a4f7893479a2
pub fn init_s_king_triggers(ctx: &mut Ctx) {
    init_single_return_trigger(ctx, Point::new(82, 42));
}

/// Original: `devilution::InitSChambTriggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitSChambTriggers() sha=1c8d29f53c71
pub fn init_s_chamb_triggers(ctx: &mut Ctx) {
    init_single_return_trigger(ctx, Point::new(70, 39));
}

/// Original: `devilution::InitPWaterTriggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitPWaterTriggers() sha=d5ac8f60a5b4
pub fn init_p_water_triggers(ctx: &mut Ctx) {
    init_single_return_trigger(ctx, Point::new(30, 83));
}

/// Original: `devilution::InitVPTriggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::InitVPTriggers() sha=5c304caf38e6
pub fn init_vp_triggers(ctx: &mut Ctx) {
    init_single_return_trigger(ctx, Point::new(35, 32));
}

fn piece_at_cursor(ctx: &Ctx, dx: i32) -> u16 {
    let c = ctx.cursor.cursPosition;
    ctx.gendung.dPiece[(c.x + dx) as usize][c.y as usize]
}

/// Moves the cursor onto the first trigger with `msg` (optionally within 4 tiles); true when found.
fn snap_to_trigger(ctx: &mut Ctx, msg: interface_mode, near: bool) -> bool {
    let c = ctx.cursor.cursPosition;
    for j in 0..ctx.trigs.numtrigs as usize {
        let t = ctx.trigs.trigs[j];
        if t._tmsg == msg {
            if near {
                let dx = (t.position.x - c.x).abs();
                let dy = (t.position.y - c.y).abs();
                if !(dx < 4 && dy < 4) {
                    continue;
                }
            }
            ctx.cursor.cursPosition = t.position;
            return true;
        }
    }
    false
}

fn fmt_d(s: &str, v: i32) -> String {
    tr(s).replacen("{:d}", &v.to_string(), 1)
}

/// Original: `ForceTownTrig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceTownTrig() sha=eae844793fde
fn force_town_trig(ctx: &mut Ctx) -> bool {
    let piece = piece_at_cursor(ctx, 0);
    if TOWN_DOWN_LIST.contains(&piece) {
        ctx.control.info_string = tr("Down to dungeon");
        ctx.cursor.cursPosition = Point::new(25, 29);
        return true;
    }
    if is_warp_open(ctx, DungeonType::Catacombs) && TOWN_WARP1_LIST.contains(&piece) {
        ctx.control.info_string = tr("Down to catacombs");
        ctx.cursor.cursPosition = Point::new(49, 21);
        return true;
    }
    if is_warp_open(ctx, DungeonType::Caves) && (1198..=1219).contains(&piece) {
        ctx.control.info_string = tr("Down to caves");
        ctx.cursor.cursPosition = Point::new(17, 69);
        return true;
    }
    if is_warp_open(ctx, DungeonType::Hell) && (1239..=1254).contains(&piece) {
        ctx.control.info_string = tr("Down to hell");
        ctx.cursor.cursPosition = Point::new(41, 80);
        return true;
    }
    if is_warp_open(ctx, DungeonType::Nest) && TOWN_HIVE_LIST.contains(&piece) {
        ctx.control.info_string = tr("Down to Hive");
        ctx.cursor.cursPosition = Point::new(80, 62);
        return true;
    }
    if is_warp_open(ctx, DungeonType::Crypt) && TOWN_CRYPT_LIST.contains(&piece) {
        ctx.control.info_string = tr("Down to Crypt");
        ctx.cursor.cursPosition = Point::new(36, 24);
        return true;
    }
    false
}

/// Original: `ForceL1Trig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceL1Trig() sha=c1d228784b4d
fn force_l1_trig(ctx: &mut Ctx) -> bool {
    let piece = piece_at_cursor(ctx, 0);
    let cur = ctx.gendung.currlevel as i32;
    if L1_UP_LIST.contains(&piece) {
        ctx.control.info_string = if cur > 1 { fmt_d("Up to level {:d}", cur - 1) } else { tr("Up to town") };
        if snap_to_trigger(ctx, WM_DIABPREVLVL, false) {
            return true;
        }
    }
    if L1_DOWN_LIST.contains(&piece) {
        ctx.control.info_string = fmt_d("Down to level {:d}", cur + 1);
        if snap_to_trigger(ctx, WM_DIABNEXTLVL, false) {
            return true;
        }
    }
    false
}

/// Original: `ForceL2Trig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceL2Trig() sha=a5bdf7fe9a68
fn force_l2_trig(ctx: &mut Ctx) -> bool {
    let piece = piece_at_cursor(ctx, 0);
    let cur = ctx.gendung.currlevel as i32;
    if L2_UP_LIST.contains(&piece) {
        let before = ctx.cursor.cursPosition;
        if snap_to_trigger(ctx, WM_DIABPREVLVL, true) {
            let _ = before;
            ctx.control.info_string = fmt_d("Up to level {:d}", cur - 1);
            return true;
        }
    }
    if L2_DOWN_LIST.contains(&piece) {
        ctx.control.info_string = fmt_d("Down to level {:d}", cur + 1);
        if snap_to_trigger(ctx, WM_DIABNEXTLVL, false) {
            return true;
        }
    }
    if cur == 5 && L2_TWARP_UP_LIST.contains(&piece) && snap_to_trigger(ctx, WM_DIABTWARPUP, true) {
        ctx.control.info_string = tr("Up to town");
        return true;
    }
    false
}

/// Original: `ForceL3Trig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceL3Trig() sha=7e7eff1cc9e3
fn force_l3_trig(ctx: &mut Ctx) -> bool {
    let piece = piece_at_cursor(ctx, 0);
    let cur = ctx.gendung.currlevel as i32;
    if L3_UP_LIST.contains(&piece) {
        ctx.control.info_string = fmt_d("Up to level {:d}", cur - 1);
        if snap_to_trigger(ctx, WM_DIABPREVLVL, true) {
            return true;
        }
    }
    for &tile_id in L3_DOWN_LIST.iter() {
        if piece_at_cursor(ctx, 0) == tile_id || piece_at_cursor(ctx, 1) == tile_id || piece_at_cursor(ctx, 2) == tile_id {
            ctx.control.info_string = fmt_d("Down to level {:d}", cur + 1);
            if snap_to_trigger(ctx, WM_DIABNEXTLVL, false) {
                return true;
            }
        }
    }
    if cur == 9 && L3_TWARP_UP_LIST.contains(&piece) && snap_to_trigger(ctx, WM_DIABTWARPUP, true) {
        ctx.control.info_string = tr("Up to town");
        return true;
    }
    false
}

/// Original: `ForceL4Trig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceL4Trig() sha=e67d919b383e
fn force_l4_trig(ctx: &mut Ctx) -> bool {
    let piece = piece_at_cursor(ctx, 0);
    let cur = ctx.gendung.currlevel as i32;
    if L4_UP_LIST.contains(&piece) {
        ctx.control.info_string = fmt_d("Up to level {:d}", cur - 1);
        if snap_to_trigger(ctx, WM_DIABPREVLVL, false) {
            return true;
        }
    }
    if L4_DOWN_LIST.contains(&piece) {
        ctx.control.info_string = fmt_d("Down to level {:d}", cur + 1);
        if snap_to_trigger(ctx, WM_DIABNEXTLVL, false) {
            return true;
        }
    }
    if cur == 13 && L4_TWARP_UP_LIST.contains(&piece) && snap_to_trigger(ctx, WM_DIABTWARPUP, true) {
        ctx.control.info_string = tr("Up to town");
        return true;
    }
    if cur == 15 && L4_PENTA_LIST.contains(&piece) {
        ctx.control.info_string = tr("Down to Diablo");
        if snap_to_trigger(ctx, WM_DIABNEXTLVL, false) {
            return true;
        }
    }
    false
}

/// Original: `ForceHiveTrig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceHiveTrig() sha=e2f343675634
fn force_hive_trig(ctx: &mut Ctx) -> bool {
    let piece = piece_at_cursor(ctx, 0);
    let cur = ctx.gendung.currlevel as i32;
    if L6_UP_LIST.contains(&piece) {
        ctx.control.info_string = fmt_d("Up to Nest level {:d}", cur - 17);
        if snap_to_trigger(ctx, WM_DIABPREVLVL, false) {
            return true;
        }
    }
    for &tile_id in L6_DOWN_LIST.iter() {
        if piece_at_cursor(ctx, 0) == tile_id || piece_at_cursor(ctx, 1) == tile_id || piece_at_cursor(ctx, 2) == tile_id {
            ctx.control.info_string = fmt_d("Down to level {:d}", cur - 15);
            if snap_to_trigger(ctx, WM_DIABNEXTLVL, false) {
                return true;
            }
        }
    }
    if cur == 17 && L6_TWARP_UP_LIST.contains(&piece) && snap_to_trigger(ctx, WM_DIABTWARPUP, true) {
        ctx.control.info_string = tr("Up to town");
        return true;
    }
    false
}

/// Original: `ForceCryptTrig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceCryptTrig() sha=dcf986c0c44c
fn force_crypt_trig(ctx: &mut Ctx) -> bool {
    let piece = piece_at_cursor(ctx, 0);
    let cur = ctx.gendung.currlevel as i32;
    if L5_UP_LIST.contains(&piece) {
        ctx.control.info_string = fmt_d("Up to Crypt level {:d}", cur - 21);
        if snap_to_trigger(ctx, WM_DIABPREVLVL, false) {
            return true;
        }
    }
    if piece == 316 {
        ctx.control.info_string = tr("Cornerstone of the World");
        return true;
    }
    if L5_DOWN_LIST.contains(&piece) {
        ctx.control.info_string = fmt_d("Down to Crypt level {:d}", cur - 19);
        if snap_to_trigger(ctx, WM_DIABNEXTLVL, false) {
            return true;
        }
    }
    if cur == 21 && L5_TWARP_UP_LIST.contains(&piece) && snap_to_trigger(ctx, WM_DIABTWARPUP, true) {
        ctx.control.info_string = tr("Up to town");
        return true;
    }
    false
}

/// Original: `devilution::Freeupstairs` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::Freeupstairs() sha=92ef336676ec
pub fn freeupstairs(ctx: &mut Ctx) {
    for i in 0..ctx.trigs.numtrigs as usize {
        let tx = ctx.trigs.trigs[i].position.x;
        let ty = ctx.trigs.trigs[i].position.y;
        for yy in -2..=2 {
            for xx in -2..=2 {
                ctx.gendung.dFlags[(tx + xx) as usize][(ty + yy) as usize] |= DungeonFlag::Populated;
            }
        }
    }
}

fn force_return_trig(ctx: &mut Ctx, list: &[u16], q: quest_id) -> bool {
    if list.contains(&piece_at_cursor(ctx, 0)) {
        let lvl = ctx.quests.Quests[q as usize]._qlevel as i32;
        ctx.control.info_string = fmt_d("Back to Level {:d}", lvl);
        ctx.cursor.cursPosition = ctx.trigs.trigs[0].position;
        return true;
    }
    false
}

/// Original: `ForceSKingTrig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceSKingTrig() sha=00410e56f77c
fn force_s_king_trig(ctx: &mut Ctx) -> bool {
    force_return_trig(ctx, &L1_UP_LIST, Q_SKELKING)
}

/// Original: `ForceSChambTrig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceSChambTrig() sha=4ef4a7259905
fn force_s_chamb_trig(ctx: &mut Ctx) -> bool {
    force_return_trig(ctx, &L2_DOWN_LIST, Q_SCHAMB)
}

/// Original: `ForcePWaterTrig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForcePWaterTrig() sha=cefabfb6b8b2
fn force_p_water_trig(ctx: &mut Ctx) -> bool {
    force_return_trig(ctx, &L3_DOWN_LIST, Q_PWATER)
}

/// Original: `ForceArenaTrig` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::ForceArenaTrig() sha=e31ae0907295
fn force_arena_trig(ctx: &mut Ctx) -> bool {
    let check_list: &[u16] = match ctx.gendung.setlvltype {
        DungeonType::Town => &TOWN_WARP1_LIST,
        DungeonType::Cathedral => &L1_UP_LIST,
        DungeonType::Catacombs => &L2_TWARP_UP_LIST,
        DungeonType::Caves => &L3_TWARP_UP_LIST,
        DungeonType::Hell => &L4_TWARP_UP_LIST,
        DungeonType::Nest => &L5_TWARP_UP_LIST,
        DungeonType::Crypt => &L6_TWARP_UP_LIST,
        _ => return false,
    };
    if check_list.contains(&piece_at_cursor(ctx, 0)) {
        ctx.control.info_string = tr("Up to town");
        ctx.cursor.cursPosition = ctx.trigs.trigs[0].position;
        return true;
    }
    false
}

/// Original: `devilution::CheckTrigForce` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::CheckTrigForce() sha=f36a0ca7cfe7
pub fn check_trig_force(ctx: &mut Ctx) {
    ctx.trigs.trigflag = false;
    let mouse = crate::engine::geometry::Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1);
    if ctx.controls.control_mode == crate::controls::ControlTypes::KeyboardAndMouse && crate::control::get_main_panel(ctx).contains(mouse) {
        return;
    }
    if !ctx.gendung.setlevel {
        let f = match ctx.gendung.leveltype {
            DungeonType::Town => force_town_trig(ctx),
            DungeonType::Cathedral => force_l1_trig(ctx),
            DungeonType::Catacombs => force_l2_trig(ctx),
            DungeonType::Caves => force_l3_trig(ctx),
            DungeonType::Hell => force_l4_trig(ctx),
            DungeonType::Nest => force_hive_trig(ctx),
            DungeonType::Crypt => force_crypt_trig(ctx),
            _ => false,
        };
        ctx.trigs.trigflag = f;
        if ctx.gendung.leveltype != DungeonType::Town && !ctx.trigs.trigflag {
            ctx.trigs.trigflag = crate::quests::force_quests(ctx);
        }
    } else {
        let f = match ctx.gendung.setlvlnum {
            SL_SKELKING => force_s_king_trig(ctx),
            SL_BONECHAMB => force_s_chamb_trig(ctx),
            SL_POISONWATER => force_p_water_trig(ctx),
            n => {
                if crate::levels::gendung::is_arena_level(n) {
                    force_arena_trig(ctx)
                } else {
                    ctx.trigs.trigflag
                }
            }
        };
        ctx.trigs.trigflag = f;
    }
}

/// Original: `devilution::CheckTriggers` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::CheckTriggers() sha=b7efb4836ef2
pub fn check_triggers(ctx: &mut Ctx) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let my_id = ctx.players.MyPlayerId;
    if ctx.players.Players[me]._pmode != PM_STAND {
        return;
    }
    for i in 0..ctx.trigs.numtrigs as usize {
        let trig = ctx.trigs.trigs[i];
        let tile = ctx.players.Players[me].position.tile;
        if tile != trig.position {
            continue;
        }
        let cur = ctx.gendung.currlevel as i32;
        match trig._tmsg {
            WM_DIABNEXTLVL => {
                if ctx.init.gb_is_spawn && cur >= 2 {
                    crate::msg::net_send_cmd_loc(ctx, my_id, true, CMD_WALKXY, Point::new(tile.x, tile.y + 1));
                    crate::player::player_say(ctx, me, HeroSpeech::NotAChance);
                    crate::error::init_diablo_msg_id(ctx, EMSG_NOT_IN_SHAREWARE as usize, 3500);
                } else {
                    crate::player::start_new_lvl(ctx, me, trig._tmsg, cur + 1);
                }
            }
            WM_DIABPREVLVL => crate::player::start_new_lvl(ctx, me, trig._tmsg, cur - 1),
            WM_DIABRTNLVL => {
                let l = crate::quests::get_map_return_level(ctx);
                crate::player::start_new_lvl(ctx, me, trig._tmsg, l);
            }
            WM_DIABTOWNWARP => {
                if ctx.init.gb_is_multiplayer {
                    let mut abort = false;
                    let mut abortflag = 0;
                    let mut position = tile;
                    let plvl = ctx.players.Players[me]._pLevel;
                    if trig._tlvl == 5 && plvl < 8 {
                        abort = true;
                        position.y += 1;
                        abortflag = EMSG_REQUIRES_LVL_8;
                    }
                    if matches!(trig._tlvl, 9 | 17) && plvl < 13 {
                        abort = true;
                        position.x += 1;
                        abortflag = EMSG_REQUIRES_LVL_13;
                    }
                    if matches!(trig._tlvl, 13 | 21) && plvl < 17 {
                        abort = true;
                        position.y += 1;
                        abortflag = EMSG_REQUIRES_LVL_17;
                    }
                    if abort {
                        crate::player::player_say(ctx, me, HeroSpeech::ICantGetThereFromHere);
                        crate::error::init_diablo_msg_id(ctx, abortflag as usize, 3500);
                        crate::msg::net_send_cmd_loc(ctx, my_id, true, CMD_WALKXY, position);
                        return;
                    }
                }
                crate::player::start_new_lvl(ctx, me, trig._tmsg, trig._tlvl);
            }
            WM_DIABTWARPUP => {
                ctx.trigs.TWarpFrom = cur;
                crate::player::start_new_lvl(ctx, me, trig._tmsg, 0);
            }
            _ => crate::appfat::app_fatal(ctx, "Unknown trigger msg"),
        }
    }
}

/// Original: `devilution::EntranceBoundaryContains` (levels/trigs.cpp).
// @port levels/trigs.cpp|devilution::EntranceBoundaryContains(Point entrance, Point position) sha=0aee7331171f
pub fn entrance_boundary_contains(entrance: Point, position: Point) -> bool {
    const ENTRANCE_OFFSETS: [(i32, i32); 7] = [(0, 0), (-1, 0), (0, -1), (-1, -1), (-2, -1), (-1, -2), (-2, -2)];
    ENTRANCE_OFFSETS.iter().any(|&(dx, dy)| entrance + Displacement::new(dx, dy) == position)
}
