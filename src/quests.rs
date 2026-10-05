//! `Source/quests.cpp`: quests, the quest log and quest set pieces.

use crate::ctx::Ctx;
use crate::effects_data::*;
use crate::engine::geometry::{Direction, Displacement, Point, Rectangle, Size};
use crate::engine::surface::Surface;
use crate::enums::*;
use crate::levels::gendung::DungeonType;
use crate::msg::net_send_cmd_quest;
use crate::utils::language::tr;

pub const MAXQUESTS: usize = 24;

/// `Quest`
#[derive(Clone, Copy, Debug)]
pub struct Quest {
    pub _qidx: quest_id,
    pub _qactive: quest_state,
    pub _qlevel: u8,
    pub position: Point,
    pub _qlvltype: DungeonType,
    pub _qslvl: _setlevels,
    pub _qlog: bool,
    pub _qmsg: _speech_id,
    pub _qvar1: u8,
    pub _qvar2: u8,
}

impl Default for Quest {
    fn default() -> Self {
        Quest {
            _qidx: 0,
            _qactive: QUEST_NOTAVAIL,
            _qlevel: 0,
            position: Point::default(),
            _qlvltype: DungeonType::Town,
            _qslvl: SL_NONE,
            _qlog: false,
            _qmsg: TEXT_NONE,
            _qvar1: 0,
            _qvar2: 0,
        }
    }
}

/// `QuestData`
pub struct QuestData {
    pub _qdlvl: u8,
    pub _qdmultlvl: i8,
    pub _qlvlt: DungeonType,
    pub questBookOrder: i8,
    pub _qdrnd: u8,
    pub _qslvl: _setlevels,
    pub isSinglePlayerOnly: bool,
    pub _qdmsg: _speech_id,
    pub _qlstr: &'static str,
}

const fn qd(
    _qdlvl: u8,
    _qdmultlvl: i8,
    _qlvlt: DungeonType,
    questBookOrder: i8,
    _qdrnd: u8,
    _qslvl: _setlevels,
    isSinglePlayerOnly: bool,
    _qdmsg: _speech_id,
    _qlstr: &'static str,
) -> QuestData {
    QuestData { _qdlvl, _qdmultlvl, _qlvlt, questBookOrder, _qdrnd, _qslvl, isSinglePlayerOnly, _qdmsg, _qlstr }
}

use DungeonType::{Catacombs as DTYPE_CATACOMBS, Cathedral as DTYPE_CATHEDRAL, Caves as DTYPE_CAVES, None as DTYPE_NONE};

/// `QuestsData`: the data related to each quest_id.
#[rustfmt::skip]
pub static QuestsData: [QuestData; MAXQUESTS] = [
    qd( 5, -1, DTYPE_NONE,       5, 100, SL_NONE,         true,  TEXT_INFRA5,   "The Magic Rock"),
    qd( 9, -1, DTYPE_NONE,      10, 100, SL_NONE,         true,  TEXT_MUSH8,    "Black Mushroom"),
    qd( 4, -1, DTYPE_NONE,       3, 100, SL_NONE,         true,  TEXT_GARBUD1,  "Gharbad The Weak"),
    qd( 8, -1, DTYPE_NONE,       9, 100, SL_NONE,         true,  TEXT_ZHAR1,    "Zhar the Mad"),
    qd(14, -1, DTYPE_NONE,      21, 100, SL_NONE,         true,  TEXT_VEIL9,    "Lachdanan"),
    qd(15, -1, DTYPE_NONE,      23, 100, SL_NONE,         false, TEXT_VILE3,    "Diablo"),
    qd( 2,  2, DTYPE_NONE,       0, 100, SL_NONE,         false, TEXT_BUTCH9,   "The Butcher"),
    qd( 4, -1, DTYPE_NONE,       4, 100, SL_NONE,         true,  TEXT_BANNER2,  "Ogden's Sign"),
    qd( 7, -1, DTYPE_NONE,       8, 100, SL_NONE,         true,  TEXT_BLINDING, "Halls of the Blind"),
    qd( 5, -1, DTYPE_NONE,       6, 100, SL_NONE,         true,  TEXT_BLOODY,   "Valor"),
    qd(10, -1, DTYPE_NONE,      11, 100, SL_NONE,         true,  TEXT_ANVIL5,   "Anvil of Fury"),
    qd(13, -1, DTYPE_NONE,      20, 100, SL_NONE,         true,  TEXT_BLOODWAR, "Warlord of Blood"),
    qd( 3,  3, DTYPE_CATHEDRAL,  2, 100, SL_SKELKING,     false, TEXT_KING2,    "The Curse of King Leoric"),
    qd( 2, -1, DTYPE_CAVES,      1, 100, SL_POISONWATER,  true,  TEXT_POISON3,  "Poisoned Water Supply"),
    qd( 6, -1, DTYPE_CATACOMBS,  7, 100, SL_BONECHAMB,    true,  TEXT_BONER,    "The Chamber of Bone"),
    qd(15, 15, DTYPE_CATHEDRAL, 22, 100, SL_VILEBETRAYER, false, TEXT_VILE1,    "Archbishop Lazarus"),
    qd(17, 17, DTYPE_NONE,      17, 100, SL_NONE,         false, TEXT_GRAVE7,   "Grave Matters"),
    qd( 9,  9, DTYPE_NONE,      12, 100, SL_NONE,         false, TEXT_FARMER1,  "Farmer's Orchard"),
    qd(17, -1, DTYPE_NONE,      14, 100, SL_NONE,         true,  TEXT_GIRL2,    "Little Girl"),
    qd(19, -1, DTYPE_NONE,      16, 100, SL_NONE,         true,  TEXT_TRADER,   "Wandering Trader"),
    qd(17, 17, DTYPE_NONE,      15, 100, SL_NONE,         false, TEXT_DEFILER1, "The Defiler"),
    qd(21, 21, DTYPE_NONE,      19, 100, SL_NONE,         false, TEXT_NAKRUL1,  "Na-Krul"),
    qd(21, -1, DTYPE_NONE,      18, 100, SL_NONE,         true,  TEXT_CORNSTN,  "Cornerstone of the World"),
    qd( 9,  9, DTYPE_NONE,      13, 100, SL_NONE,         false, TEXT_JERSEY4,  "The Jersey's Jersey"),
];

/// `QuestTriggerNames`
const QUEST_TRIGGER_NAMES: [&str; 5] = ["King Leoric's Tomb", "The Chamber of Bone", "Maze", "A Dark Passage", "Unholy Altar"];

/// `QuestGroup1..4`
const QUEST_GROUP1: [quest_id; 3] = [Q_BUTCHER, Q_LTBANNER, Q_GARBUD];
const QUEST_GROUP2: [quest_id; 3] = [Q_BLIND, Q_ROCK, Q_BLOOD];
const QUEST_GROUP3: [quest_id; 3] = [Q_MUSHROOM, Q_ZHAR, Q_ANVIL];
const QUEST_GROUP4: [quest_id; 2] = [Q_VEIL, Q_WARLORD];

/// `InnerPanel`
const INNER_PANEL: Rectangle = Rectangle::new(Point::new(32, 26), Size::new(280, 300));
const LINE_HEIGHT: i32 = 12;
const MAX_SPACING: i32 = LINE_HEIGHT * 2;

/// Globals of quests.cpp.
pub struct QuestsState {
    pub QuestLogIsOpen: bool,
    /// `pQLogCel`
    pub p_q_log_cel: Option<crate::engine::clx_sprite::ClxSpriteList>,
    pub Quests: [Quest; MAXQUESTS],
    pub ReturnLvlPosition: Point,
    pub ReturnLevelType: DungeonType,
    pub ReturnLevel: i32,
    water_done: i32,
    encountered_quests: [quest_id; MAXQUESTS],
    encountered_quest_count: i32,
    first_finished_quest: i32,
    selected_quest: i32,
    list_y_offset: i32,
    line_spacing: i32,
    finished_quest_offset: i32,
}

impl Default for QuestsState {
    fn default() -> Self {
        QuestsState {
            QuestLogIsOpen: false,
            p_q_log_cel: None,
            Quests: [Quest::default(); MAXQUESTS],
            ReturnLvlPosition: Point::default(),
            ReturnLevelType: DungeonType::Town,
            ReturnLevel: 0,
            water_done: 0,
            encountered_quests: [0; MAXQUESTS],
            encountered_quest_count: 0,
            first_finished_quest: 0,
            selected_quest: 0,
            list_y_offset: 0,
            line_spacing: 0,
            finished_quest_offset: 0,
        }
    }
}

/// `QuestsData[qidx].isSinglePlayerOnly`
pub fn quest_data_is_single_player_only(qidx: quest_id) -> bool {
    QuestsData[qidx as usize].isSinglePlayerOnly
}

fn load_dun(ctx: &mut Ctx, path: &str) -> Vec<u16> {
    let bytes = crate::engine::load_file::load_file_in_mem(ctx, path).unwrap_or_else(|| panic!("missing {path}"));
    bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect()
}

/// Original: `DrawButcher` (quests.cpp).
// @port quests.cpp|devilution::DrawButcher() sha=c78565bb503a
fn draw_butcher(ctx: &mut Ctx) {
    let position = ctx.gendung.SetPiece.position.mega_to_world() + Displacement::new(3, 3);
    crate::levels::gendung::drlg_rect_trans(ctx, Rectangle::new(position, Size::new(7, 7)));
}

/// Original: `DrawSkelKing` (quests.cpp).
// @port quests.cpp|devilution::DrawSkelKing(quest_id q, Point position) sha=97842a227c6f
fn draw_skel_king(ctx: &mut Ctx, q: quest_id, position: Point) {
    ctx.quests.Quests[q as usize].position = position.mega_to_world() + Displacement::new(12, 7);
}

/// Original: `DrawWarLord` (quests.cpp).
// @port quests.cpp|devilution::DrawWarLord(Point position) sha=9bfb1fce308e
fn draw_war_lord(ctx: &mut Ctx, position: Point) {
    let dun_data = load_dun(ctx, "levels\\l4data\\warlord2.dun");
    ctx.gendung.SetPiece = Rectangle::new(position, Size::new(dun_data[0] as i32, dun_data[1] as i32));
    crate::levels::gendung::place_dun_tiles(ctx, &dun_data, position, 6);
}

/// Original: `DrawSChamber` (quests.cpp).
// @port quests.cpp|devilution::DrawSChamber(quest_id q, Point position) sha=c8fb04e75c8e
fn draw_s_chamber(ctx: &mut Ctx, q: quest_id, position: Point) {
    let dun_data = load_dun(ctx, "levels\\l2data\\bonestr1.dun");
    ctx.gendung.SetPiece = Rectangle::new(position, Size::new(dun_data[0] as i32, dun_data[1] as i32));
    crate::levels::gendung::place_dun_tiles(ctx, &dun_data, position, 3);
    ctx.quests.Quests[q as usize].position = position.mega_to_world() + Displacement::new(6, 7);
}

/// Original: `DrawLTBanner` (quests.cpp).
// @port quests.cpp|devilution::DrawLTBanner(Point position) sha=0e0f38aa3a30
fn draw_lt_banner(ctx: &mut Ctx, position: Point) {
    let dun_data = load_dun(ctx, "levels\\l1data\\banner1.dun");
    let width = dun_data[0] as i32;
    let height = dun_data[1] as i32;
    ctx.gendung.SetPiece = Rectangle::new(position, Size::new(width, height));
    let tile_layer = &dun_data[2..];
    for j in 0..height {
        for i in 0..width {
            let tile_id = tile_layer[(j * width + i) as usize] as u8;
            if tile_id != 0 {
                ctx.gendung.pdungeon[(position.x + i) as usize][(position.y + j) as usize] = tile_id;
            }
        }
    }
}

/// Original: `DrawBlind` (quests.cpp): closes the outer wall.
// @port quests.cpp|devilution::DrawBlind(Point position) sha=01bb7116a1ff
fn draw_blind(ctx: &mut Ctx, position: Point) {
    ctx.gendung.dungeon[position.x as usize][(position.y + 1) as usize] = 154;
    ctx.gendung.dungeon[(position.x + 10) as usize][(position.y + 8) as usize] = 154;
}

/// Original: `DrawBlood` (quests.cpp).
// @port quests.cpp|devilution::DrawBlood(Point position) sha=34723433b399
fn draw_blood(ctx: &mut Ctx, position: Point) {
    let dun_data = load_dun(ctx, "levels\\l2data\\blood2.dun");
    ctx.gendung.SetPiece = Rectangle::new(position, Size::new(dun_data[0] as i32, dun_data[1] as i32));
    crate::levels::gendung::place_dun_tiles(ctx, &dun_data, position, 0);
}

/// Original: `QuestLogMouseToEntry` (quests.cpp).
// @port quests.cpp|devilution::QuestLogMouseToEntry() sha=003cc4796432
fn quest_log_mouse_to_entry(ctx: &Ctx) -> i32 {
    let lp = crate::control::get_left_panel(ctx);
    let mut inner_area = INNER_PANEL;
    inner_area.position = inner_area.position + Displacement::new(lp.x, lp.y);
    let mouse = Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1);
    let s = &ctx.quests;
    if !inner_area.contains(mouse) || s.encountered_quest_count == 0 {
        return -1;
    }
    let y = mouse.y - inner_area.position.y;
    for i in 0..s.first_finished_quest {
        if y >= s.list_y_offset + i * s.line_spacing && y < s.list_y_offset + i * s.line_spacing + LINE_HEIGHT {
            return i;
        }
    }
    -1
}

crate::pending_fn!(fn print_ql_string(ctx: &mut Ctx, out: &Surface, x: i32, y: i32, s: &str, marked: bool, disabled: bool), "quests.cpp|devilution::PrintQLString(const Surface &out, int x, int y, string_view str, bool marked, bool disabled)");

/// Original: `StartPWaterPurify` (quests.cpp).
// @port quests.cpp|devilution::StartPWaterPurify() sha=2fe95736e182
fn start_p_water_purify(ctx: &mut Ctx) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let tile = ctx.players.Players[me].position.tile;
    crate::effects::play_sfx_loc(ctx, IS_QUESTDN, tile, true);
    crate::engine::palette::load_palette(ctx, "levels\\l3data\\l3pwater.pal", false);
    update_p_water_palette(ctx);
    ctx.quests.water_done = 32;
}

/// Original: `devilution::InitQuests` (quests.cpp).
// @port quests.cpp|devilution::InitQuests() sha=47b3cebb2c29
pub fn init_quests(ctx: &mut Ctx) {
    ctx.towners.QuestDialogTable[TOWN_HEALER as usize][Q_MUSHROOM as usize] = TEXT_NONE;
    ctx.towners.QuestDialogTable[TOWN_WITCH as usize][Q_MUSHROOM as usize] = TEXT_MUSH9;
    ctx.quests.QuestLogIsOpen = false;
    ctx.quests.water_done = 0;

    let use_mp = use_multiplayer_quests(ctx);
    for (q, quest) in ctx.quests.Quests.iter_mut().enumerate() {
        quest._qidx = q as quest_id;
        let quest_data = &QuestsData[q];
        quest._qactive = QUEST_NOTAVAIL;
        quest.position = Point::new(0, 0);
        quest._qlvltype = quest_data._qlvlt;
        quest._qslvl = quest_data._qslvl;
        quest._qvar1 = 0;
        quest._qvar2 = 0;
        quest._qlog = false;
        quest._qmsg = quest_data._qdmsg;
        if !use_mp {
            quest._qlevel = quest_data._qdlvl;
            quest._qactive = QUEST_INIT;
        } else if !quest_data.isSinglePlayerOnly {
            quest._qlevel = quest_data._qdmultlvl as u8;
            quest._qactive = QUEST_INIT;
        }
    }

    if !use_mp && ctx.options.gameplay.randomize_quests.get() {
        // Quests are set from the seed used to generate level 16.
        let seed = ctx.diablo.glSeedTbl[15];
        let mut quests = ctx.quests.Quests;
        initialise_quest_pools(ctx, seed, &mut quests);
        ctx.quests.Quests = quests;
    }

    if ctx.init.gb_is_spawn {
        for quest in ctx.quests.Quests.iter_mut() {
            quest._qactive = QUEST_NOTAVAIL;
        }
    }

    let qs = &mut ctx.quests.Quests;
    if qs[Q_SKELKING as usize]._qactive == QUEST_NOTAVAIL {
        qs[Q_SKELKING as usize]._qvar2 = 2;
    }
    if qs[Q_ROCK as usize]._qactive == QUEST_NOTAVAIL {
        qs[Q_ROCK as usize]._qvar2 = 2;
    }
    qs[Q_LTBANNER as usize]._qvar1 = 1;
    if use_mp {
        qs[Q_BETRAYER as usize]._qvar1 = 2;
    }
    // In multiplayer items spawn during level generation to avoid desyncs
    if ctx.init.gb_is_multiplayer && ctx.quests.Quests[Q_MUSHROOM as usize]._qactive == QUEST_INIT {
        ctx.quests.Quests[Q_MUSHROOM as usize]._qvar1 = QS_TOMESPAWNED as u8;
    }
}

/// Original: `devilution::InitialiseQuestPools` (quests.cpp).
// @port quests.cpp|devilution::InitialiseQuestPools(uint32_t seed, Quest quests[]) sha=f232b0fff2a2
pub fn initialise_quest_pools(ctx: &mut Ctx, seed: u32, quests: &mut [Quest; MAXQUESTS]) {
    ctx.rng.set_rnd_seed(seed);
    let q = ctx.rng.pick_randomly_among(&[Q_SKELKING, Q_PWATER]);
    quests[q as usize]._qactive = QUEST_NOTAVAIL;

    let random_index = ctx.rng.generate_rnd(QUEST_GROUP1.len() as i32);
    if random_index >= 0 {
        quests[QUEST_GROUP1[random_index as usize] as usize]._qactive = QUEST_NOTAVAIL;
    }
    let random_index = ctx.rng.generate_rnd(QUEST_GROUP2.len() as i32);
    if random_index >= 0 {
        quests[QUEST_GROUP2[random_index as usize] as usize]._qactive = QUEST_NOTAVAIL;
    }
    let random_index = ctx.rng.generate_rnd(QUEST_GROUP3.len() as i32);
    if random_index >= 0 {
        quests[QUEST_GROUP3[random_index as usize] as usize]._qactive = QUEST_NOTAVAIL;
    }
    let random_index = ctx.rng.generate_rnd(QUEST_GROUP4.len() as i32);
    // always true, QuestGroup4 has two members
    if random_index >= 0 {
        quests[QUEST_GROUP4[random_index as usize] as usize]._qactive = QUEST_NOTAVAIL;
    }
}

/// Original: `devilution::CheckQuests` (quests.cpp).
// @port quests.cpp|devilution::CheckQuests() sha=e9daa968fc85
pub fn check_quests(ctx: &mut Ctx) {
    if ctx.init.gb_is_spawn {
        return;
    }
    let my_id = ctx.players.MyPlayerId as i32;
    let qb = Q_BETRAYER as usize;
    if is_quest_available(ctx, Q_BETRAYER) && use_multiplayer_quests(ctx) && ctx.quests.Quests[qb]._qvar1 == 2 {
        let p = ctx.gendung.SetPiece.position.mega_to_world() + Displacement::new(4, 6);
        crate::objects::add_object(ctx, OBJ_ALTBOY, p);
        ctx.quests.Quests[qb]._qvar1 = 3;
        net_send_cmd_quest(ctx, true, qb);
    }
    if use_multiplayer_quests(ctx) {
        return;
    }
    let quest = ctx.quests.Quests[qb];
    if ctx.gendung.currlevel == quest._qlevel
        && !ctx.gendung.setlevel
        && quest._qvar1 >= 2
        && (quest._qactive == QUEST_ACTIVE || quest._qactive == QUEST_DONE)
        && (quest._qvar2 == 0 || quest._qvar2 == 2)
    {
        // Spawn a portal at the quest trigger location
        crate::missiles::add_missile(
            ctx,
            quest.position,
            quest.position,
            Direction::South,
            MissileID::RedPortal,
            crate::missiles::TARGET_MONSTERS,
            my_id,
            0,
            0,
            None,
        );
        let q = &mut ctx.quests.Quests[qb];
        q._qvar2 = 1;
        if q._qactive == QUEST_ACTIVE && q._qvar1 == 2 {
            q._qvar1 = 3;
        }
    }
    let quest = ctx.quests.Quests[qb];
    if quest._qactive == QUEST_DONE && ctx.gendung.setlevel && ctx.gendung.setlvlnum == SL_VILEBETRAYER && quest._qvar2 == 4 {
        let portal_location = Point::new(35, 32);
        crate::missiles::add_missile(
            ctx,
            portal_location,
            portal_location,
            Direction::South,
            MissileID::RedPortal,
            crate::missiles::TARGET_MONSTERS,
            my_id,
            0,
            0,
            None,
        );
        ctx.quests.Quests[qb]._qvar2 = 3;
    }

    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.gendung.setlevel {
        let poison_water = ctx.quests.Quests[Q_PWATER as usize];
        if ctx.gendung.setlvlnum == poison_water._qslvl
            && poison_water._qactive != QUEST_INIT
            && ctx.gendung.leveltype == poison_water._qlvltype
            && ctx.monster.ActiveMonsterCount == 4
            && poison_water._qactive != QUEST_DONE
        {
            let pw = &mut ctx.quests.Quests[Q_PWATER as usize];
            pw._qactive = QUEST_DONE;
            pw._qlog = true;
            net_send_cmd_quest(ctx, true, Q_PWATER as usize);
            start_p_water_purify(ctx);
        }
    } else if ctx.players.Players[me]._pmode == PM_STAND {
        for i in 0..MAXQUESTS {
            let quest = ctx.quests.Quests[i];
            if ctx.gendung.currlevel == quest._qlevel
                && quest._qslvl != 0
                && quest._qactive != QUEST_NOTAVAIL
                && ctx.players.Players[me].position.tile == quest.position
                && (quest._qidx != Q_BETRAYER || quest._qvar1 >= 3)
            {
                if quest._qlvltype != DungeonType::None {
                    ctx.gendung.setlvltype = quest._qlvltype;
                }
                crate::player::start_new_lvl(ctx, me, WM_DIABSETLVL, quest._qslvl as i32);
            }
        }
    }
}

/// Original: `devilution::ForceQuests` (quests.cpp).
// @port quests.cpp|devilution::ForceQuests() sha=7e4435f93651
pub fn force_quests(ctx: &mut Ctx) -> bool {
    if ctx.init.gb_is_spawn {
        return false;
    }
    if use_multiplayer_quests(ctx) {
        return false;
    }
    for i in 0..MAXQUESTS {
        let quest = ctx.quests.Quests[i];
        if quest._qidx != Q_BETRAYER && ctx.gendung.currlevel == quest._qlevel && quest._qslvl != 0 {
            let ql = quest._qslvl as usize - 1;
            if crate::levels::trigs::entrance_boundary_contains(quest.position, ctx.cursor.cursPosition) {
                ctx.control.info_string = tr("To {:s}").replacen("{:s}", &tr(QUEST_TRIGGER_NAMES[ql]), 1);
                ctx.cursor.cursPosition = quest.position;
                return true;
            }
        }
    }
    false
}

/// Original: `devilution::CheckQuestKill` (quests.cpp).
// @port quests.cpp|devilution::CheckQuestKill(const Monster &monster, bool sendmsg) sha=b2c05542f588
pub fn check_quest_kill(ctx: &mut Ctx, m: usize, sendmsg: bool) {
    if ctx.init.gb_is_spawn {
        return;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let mtype = crate::monster::monster_type_id(ctx, m);
    let unique_type = ctx.monster.Monsters[m].uniqueType;
    let say = |ctx: &mut Ctx, s: HeroSpeech| crate::player::player_say_delayed(ctx, me, s, 30);
    if mtype == MT_SKING {
        ctx.quests.Quests[Q_SKELKING as usize]._qactive = QUEST_DONE;
        say(ctx, HeroSpeech::RestWellLeoricIllFindYourSon);
        if sendmsg {
            net_send_cmd_quest(ctx, true, Q_SKELKING as usize);
        }
    } else if mtype == MT_CLEAVER {
        ctx.quests.Quests[Q_BUTCHER as usize]._qactive = QUEST_DONE;
        say(ctx, HeroSpeech::TheSpiritsOfTheDeadAreNowAvenged);
        if sendmsg {
            net_send_cmd_quest(ctx, true, Q_BUTCHER as usize);
        }
    } else if unique_type == UniqueMonsterType::Garbud {
        ctx.quests.Quests[Q_GARBUD as usize]._qactive = QUEST_DONE;
        net_send_cmd_quest(ctx, true, Q_GARBUD as usize);
        say(ctx, HeroSpeech::ImNotImpressed);
    } else if unique_type == UniqueMonsterType::Zhar {
        ctx.quests.Quests[Q_ZHAR as usize]._qactive = QUEST_DONE;
        net_send_cmd_quest(ctx, true, Q_ZHAR as usize);
        say(ctx, HeroSpeech::ImSorryDidIBreakYourConcentration);
    } else if unique_type == UniqueMonsterType::Lazarus {
        ctx.quests.Quests[Q_BETRAYER as usize]._qactive = QUEST_DONE;
        say(ctx, HeroSpeech::YourMadnessEndsHereBetrayer);
        ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 = 7;
        ctx.quests.Quests[Q_DIABLO as usize]._qactive = QUEST_ACTIVE;
        if use_multiplayer_quests(ctx) {
            for j in 0..crate::levels::gendung::MAXDUNY {
                for i in 0..crate::levels::gendung::MAXDUNX {
                    if ctx.gendung.dPiece[i][j] == 369 {
                        let n = ctx.trigs.numtrigs as usize;
                        ctx.trigs.trigs[n].position = Point::new(i as i32, j as i32);
                        ctx.trigs.trigs[n]._tmsg = WM_DIABNEXTLVL;
                        ctx.trigs.numtrigs += 1;
                    }
                }
            }
        } else {
            crate::levels::trigs::init_vp_triggers(ctx);
            ctx.quests.Quests[Q_BETRAYER as usize]._qvar2 = 4;
            let my_id = ctx.players.MyPlayerId as i32;
            crate::missiles::add_missile(
                ctx,
                Point::new(35, 32),
                Point::new(35, 32),
                Direction::South,
                MissileID::RedPortal,
                crate::missiles::TARGET_MONSTERS,
                my_id,
                0,
                0,
                None,
            );
        }
        if sendmsg {
            net_send_cmd_quest(ctx, true, Q_BETRAYER as usize);
            net_send_cmd_quest(ctx, true, Q_DIABLO as usize);
        }
    } else if unique_type == UniqueMonsterType::WarlordOfBlood {
        ctx.quests.Quests[Q_WARLORD as usize]._qactive = QUEST_DONE;
        net_send_cmd_quest(ctx, true, Q_WARLORD as usize);
        say(ctx, HeroSpeech::YourReignOfPainHasEnded);
    }
}

/// Original: `devilution::DRLG_CheckQuests` (quests.cpp).
// @port quests.cpp|devilution::DRLG_CheckQuests(Point position) sha=a69b3af0b6ee
pub fn drlg_check_quests(ctx: &mut Ctx, position: Point) {
    for i in 0..MAXQUESTS {
        let qidx = ctx.quests.Quests[i]._qidx;
        if is_quest_available(ctx, i as quest_id) {
            match qidx {
                Q_BUTCHER => draw_butcher(ctx),
                Q_LTBANNER => draw_lt_banner(ctx, position),
                Q_BLIND => draw_blind(ctx, position),
                Q_BLOOD => draw_blood(ctx, position),
                Q_WARLORD => draw_war_lord(ctx, position),
                Q_SKELKING => draw_skel_king(ctx, qidx, position),
                Q_SCHAMB => draw_s_chamber(ctx, qidx, position),
                _ => {}
            }
        }
    }
}

/// Original: `devilution::GetMapReturnLevel` (quests.cpp).
// @port quests.cpp|devilution::GetMapReturnLevel() sha=3a6fba6d2ab0
pub fn get_map_return_level(ctx: &Ctx) -> i32 {
    let q = &ctx.quests.Quests;
    match ctx.gendung.setlvlnum {
        SL_SKELKING => q[Q_SKELKING as usize]._qlevel as i32,
        SL_BONECHAMB => q[Q_SCHAMB as usize]._qlevel as i32,
        SL_POISONWATER => q[Q_PWATER as usize]._qlevel as i32,
        SL_VILEBETRAYER => q[Q_BETRAYER as usize]._qlevel as i32,
        _ => 0,
    }
}

/// Original: `devilution::GetMapReturnPosition` (quests.cpp).
// @port quests.cpp|devilution::GetMapReturnPosition() sha=c0c01a3b5c7d
pub fn get_map_return_position(ctx: &Ctx) -> Point {
    let q = &ctx.quests.Quests;
    match ctx.gendung.setlvlnum {
        SL_SKELKING => q[Q_SKELKING as usize].position + Direction::SouthEast,
        SL_BONECHAMB => q[Q_SCHAMB as usize].position + Direction::SouthEast,
        SL_POISONWATER => q[Q_PWATER as usize].position + Direction::SouthWest,
        SL_VILEBETRAYER => q[Q_BETRAYER as usize].position + Direction::South,
        _ => {
            let t = crate::towners::get_towner(ctx, TOWN_DRUNK).expect("drunk");
            ctx.towners.towners[t].position + Direction::SouthEast
        }
    }
}

/// Original: `devilution::LoadPWaterPalette` (quests.cpp).
// @port quests.cpp|devilution::LoadPWaterPalette() sha=852cd1bc46fa
pub fn load_p_water_palette(ctx: &mut Ctx) {
    let pw = ctx.quests.Quests[Q_PWATER as usize];
    if !ctx.gendung.setlevel || ctx.gendung.setlvlnum != pw._qslvl || pw._qactive == QUEST_INIT || ctx.gendung.leveltype != pw._qlvltype {
        return;
    }
    if pw._qactive == QUEST_DONE {
        crate::engine::palette::load_palette(ctx, "levels\\l3data\\l3pwater.pal", true);
    } else {
        crate::engine::palette::load_palette(ctx, "levels\\l3data\\l3pfoul.pal", true);
    }
}

/// Original: `devilution::UpdatePWaterPalette` (quests.cpp).
// @port quests.cpp|devilution::UpdatePWaterPalette() sha=e4d7d873ab4a
pub fn update_p_water_palette(ctx: &mut Ctx) {
    if ctx.quests.water_done > 0 {
        let n = ctx.quests.water_done;
        crate::engine::palette::palette_update_quest_palette(ctx, n);
        ctx.quests.water_done -= 1;
        return;
    }
    crate::engine::palette::palette_update_caves(ctx);
}

fn activate_if(ctx: &mut Ctx, q: quest_id, cond: impl Fn(&Quest, i32) -> bool) {
    let cur = ctx.gendung.currlevel as i32;
    let quest = ctx.quests.Quests[q as usize];
    if cond(&quest, cur) {
        ctx.quests.Quests[q as usize]._qactive = QUEST_ACTIVE;
        net_send_cmd_quest(ctx, true, q as usize);
    }
}

/// Original: `devilution::ResyncMPQuests` (quests.cpp).
// @port quests.cpp|devilution::ResyncMPQuests() sha=e60b7dbe3c0d
pub fn resync_mp_quests(ctx: &mut Ctx) {
    if ctx.init.gb_is_spawn {
        return;
    }
    let near = |q: &Quest, cur: i32| q._qactive == QUEST_INIT && cur >= q._qlevel as i32 - 1 && cur <= q._qlevel as i32 + 1;
    let before = |q: &Quest, cur: i32| q._qactive == QUEST_INIT && cur == q._qlevel as i32 - 1;
    activate_if(ctx, Q_SKELKING, near);
    activate_if(ctx, Q_BUTCHER, near);
    activate_if(ctx, Q_BETRAYER, before);
    if is_quest_available(ctx, Q_BETRAYER) {
        let p = ctx.gendung.SetPiece.position.mega_to_world() + Displacement::new(4, 6);
        crate::objects::add_object(ctx, OBJ_ALTBOY, p);
    }
    activate_if(ctx, Q_GRAVE, before);
    activate_if(ctx, Q_DEFILER, before);
    activate_if(ctx, Q_NAKRUL, before);
}

fn sync_all_object_anims(ctx: &mut Ctx) {
    for i in 0..ctx.objects.ActiveObjectCount as usize {
        let oi = ctx.objects.ActiveObjects[i] as usize;
        crate::objects::sync_object_anim(ctx, oi);
    }
}

fn banner_rect_trans(ctx: &mut Ctx) {
    let sp = ctx.gendung.SetPiece;
    let tren = ctx.gendung.TransVal;
    ctx.gendung.TransVal = 9;
    crate::levels::gendung::drlg_m_rect_trans(ctx, Rectangle::new(sp.position, Size::new(sp.size.width / 2 + 4, sp.size.height / 2)));
    ctx.gendung.TransVal = tren;
}

/// Original: `devilution::ResyncQuests` (quests.cpp).
// @port quests.cpp|devilution::ResyncQuests() sha=5673d549dbb5
pub fn resync_quests(ctx: &mut Ctx) {
    use crate::objects::obj_change_map_resync;
    if ctx.init.gb_is_spawn {
        return;
    }
    let mp = ctx.init.gb_is_multiplayer;
    ctx.objects.LoadingMapObjects = true;

    if is_quest_available(ctx, Q_LTBANNER) {
        let snot_spill = crate::monster::find_unique_monster(ctx, UniqueMonsterType::SnotSpill);
        let sp = ctx.gendung.SetPiece;
        let (px, py, w, h) = (sp.position.x, sp.position.y, sp.size.width, sp.size.height);
        let v1 = ctx.quests.Quests[Q_LTBANNER as usize]._qvar1;
        if v1 == 1 {
            obj_change_map_resync(ctx, px + w - 2, py + h - 2, px + w + 1, py + h + 1);
        }
        if v1 == 2 {
            obj_change_map_resync(ctx, px + w - 2, py + h - 2, px + w + 1, py + h + 1);
            obj_change_map_resync(ctx, px, py, px + (w / 2) + 2, py + (h / 2) - 2);
            sync_all_object_anims(ctx);
            banner_rect_trans(ctx);
            if let Some(s) = snot_spill {
                if mp && ctx.monster.Monsters[s].talkMsg != TEXT_BANNER12 {
                    let done = ctx.quests.Quests[Q_LTBANNER as usize]._qactive == QUEST_DONE;
                    let m = &mut ctx.monster.Monsters[s];
                    m.goal = MonsterGoal::Inquiring;
                    m.talkMsg = if done { TEXT_BANNER12 } else { TEXT_BANNER11 };
                    m.flags |= MFLAG_QUEST_COMPLETE as u32;
                }
            }
        }
        if v1 == 3 {
            obj_change_map_resync(ctx, px, py, px + w + 1, py + h + 1);
            sync_all_object_anims(ctx);
            banner_rect_trans(ctx);
            if let Some(s) = snot_spill {
                if mp {
                    let m = &mut ctx.monster.Monsters[s];
                    m.goal = MonsterGoal::Normal;
                    m.flags |= MFLAG_QUEST_COMPLETE as u32;
                    m.talkMsg = TEXT_NONE;
                    m.activeForTicks = u8::MAX;
                    crate::lighting::redo_player_vision(ctx);
                }
            }
        }
    }
    let mush = ctx.quests.Quests[Q_MUSHROOM as usize];
    if ctx.gendung.currlevel == mush._qlevel && !ctx.gendung.setlevel {
        if mush._qactive == QUEST_INIT && mush._qvar1 as i32 == QS_INIT {
            crate::items::spawn_quest_item(ctx, IDI_FUNGALTM, Point::new(0, 0), 5, 1, true);
            ctx.quests.Quests[Q_MUSHROOM as usize]._qvar1 = QS_TOMESPAWNED as u8;
            net_send_cmd_quest(ctx, true, Q_MUSHROOM as usize);
        } else if mush._qactive == QUEST_ACTIVE {
            let t = &mut ctx.towners.QuestDialogTable;
            if mush._qvar1 as i32 >= QS_MUSHGIVEN {
                t[TOWN_WITCH as usize][Q_MUSHROOM as usize] = TEXT_NONE;
                t[TOWN_HEALER as usize][Q_MUSHROOM as usize] = TEXT_MUSH3;
            } else if mush._qvar1 as i32 >= QS_BRAINGIVEN {
                t[TOWN_HEALER as usize][Q_MUSHROOM as usize] = TEXT_NONE;
            }
        }
    }
    let veil = ctx.quests.Quests[Q_VEIL as usize];
    if ctx.gendung.currlevel as i32 == veil._qlevel as i32 + 1 && veil._qactive == QUEST_ACTIVE && veil._qvar1 == 0 && !mp {
        ctx.quests.Quests[Q_VEIL as usize]._qvar1 = 1;
        crate::items::spawn_quest_item(ctx, IDI_GLDNELIX, Point::new(0, 0), 5, 1, true);
        net_send_cmd_quest(ctx, true, Q_VEIL as usize);
    }
    if ctx.gendung.setlevel && ctx.gendung.setlvlnum == SL_VILEBETRAYER {
        let v1 = ctx.quests.Quests[Q_BETRAYER as usize]._qvar1;
        if v1 >= 4 {
            obj_change_map_resync(ctx, 1, 11, 20, 18);
        }
        if v1 >= 6 {
            obj_change_map_resync(ctx, 1, 18, 20, 24);
            if mp {
                if let Some(l) = crate::monster::find_unique_monster(ctx, UniqueMonsterType::Lazarus) {
                    let m = &mut ctx.monster.Monsters[l];
                    m.goal = MonsterGoal::Normal;
                    m.talkMsg = TEXT_NONE;
                }
            }
        }
        if v1 >= 7 {
            crate::levels::trigs::init_vp_triggers(ctx);
        }
        sync_all_object_anims(ctx);
    }
    let b = ctx.quests.Quests[Q_BETRAYER as usize];
    if ctx.gendung.currlevel == b._qlevel
        && !ctx.gendung.setlevel
        && (b._qvar2 == 1 || b._qvar2 >= 3)
        && (b._qactive == QUEST_ACTIVE || b._qactive == QUEST_DONE)
    {
        ctx.quests.Quests[Q_BETRAYER as usize]._qvar2 = 2;
        net_send_cmd_quest(ctx, true, Q_BETRAYER as usize);
    }
    let d = ctx.quests.Quests[Q_DIABLO as usize];
    if ctx.gendung.currlevel == d._qlevel && !ctx.gendung.setlevel && d._qactive == QUEST_ACTIVE && mp {
        let pos = d.position;
        obj_change_map_resync(ctx, pos.x, pos.y, pos.x + 5, pos.y + 5);
        crate::levels::trigs::init_l4_triggers(ctx);
    }
    if ctx.gendung.currlevel == 0 && ctx.quests.Quests[Q_PWATER as usize]._qactive == QUEST_DONE && mp {
        crate::levels::town::clean_town_fountain(ctx);
    }
    if is_quest_available(ctx, Q_GARBUD) && mp {
        if let Some(g) = crate::monster::find_unique_monster(ctx, UniqueMonsterType::Garbud) {
            let v1 = ctx.quests.Quests[Q_GARBUD as usize]._qvar1 as i32;
            if v1 != QS_GHARBAD_INIT {
                let m = &mut ctx.monster.Monsters[g];
                match v1 {
                    QS_GHARBAD_FIRST_ITEM_READY => m.goal = MonsterGoal::Inquiring,
                    QS_GHARBAD_FIRST_ITEM_SPAWNED => {
                        m.talkMsg = TEXT_GARBUD2;
                        m.flags |= MFLAG_QUEST_COMPLETE as u32;
                        m.goal = MonsterGoal::Talking;
                    }
                    QS_GHARBAD_SECOND_ITEM_NEARLY_DONE => {
                        m.talkMsg = TEXT_GARBUD3;
                        m.flags |= MFLAG_QUEST_COMPLETE as u32;
                        m.goal = MonsterGoal::Inquiring;
                    }
                    QS_GHARBAD_SECOND_ITEM_READY => {
                        m.talkMsg = TEXT_GARBUD4;
                        m.flags |= MFLAG_QUEST_COMPLETE as u32;
                        m.goal = MonsterGoal::Inquiring;
                    }
                    QS_GHARBAD_ATTACKING => {
                        m.talkMsg = TEXT_NONE;
                        m.flags |= MFLAG_QUEST_COMPLETE as u32;
                        m.goal = MonsterGoal::Normal;
                        m.activeForTicks = u8::MAX;
                    }
                    _ => {}
                }
            }
        }
    }
    if is_quest_available(ctx, Q_ZHAR) && mp {
        if let Some(z) = crate::monster::find_unique_monster(ctx, UniqueMonsterType::Zhar) {
            let v1 = ctx.quests.Quests[Q_ZHAR as usize]._qvar1 as i32;
            if v1 != QS_ZHAR_INIT {
                let m = &mut ctx.monster.Monsters[z];
                m.flags |= MFLAG_QUEST_COMPLETE as u32;
                match v1 {
                    QS_ZHAR_ITEM_SPAWNED => m.goal = MonsterGoal::Talking,
                    QS_ZHAR_ANGRY => {
                        m.talkMsg = TEXT_ZHAR2;
                        m.goal = MonsterGoal::Inquiring;
                    }
                    QS_ZHAR_ATTACKING => {
                        m.talkMsg = TEXT_NONE;
                        m.goal = MonsterGoal::Normal;
                        m.activeForTicks = u8::MAX;
                    }
                    _ => {}
                }
            }
        }
    }
    if is_quest_available(ctx, Q_WARLORD) && mp {
        if let Some(w) = crate::monster::find_unique_monster(ctx, UniqueMonsterType::WarlordOfBlood) {
            if ctx.quests.Quests[Q_WARLORD as usize]._qvar1 as i32 == QS_WARLORD_ATTACKING {
                let m = &mut ctx.monster.Monsters[w];
                m.activeForTicks = u8::MAX;
                m.talkMsg = TEXT_NONE;
                m.goal = MonsterGoal::Normal;
            }
        }
    }
    if is_quest_available(ctx, Q_VEIL) && mp {
        if let Some(l) = crate::monster::find_unique_monster(ctx, UniqueMonsterType::Lachdan) {
            let v2 = ctx.quests.Quests[Q_VEIL as usize]._qvar2 as i32;
            let m = &mut ctx.monster.Monsters[l];
            match v2 {
                QS_VEIL_EARLY_RETURN => {
                    m.talkMsg = TEXT_VEIL10;
                    m.goal = MonsterGoal::Inquiring;
                }
                QS_VEIL_ITEM_SPAWNED => {
                    if m.talkMsg != TEXT_VEIL11 {
                        m.talkMsg = TEXT_VEIL11;
                        m.flags |= MFLAG_QUEST_COMPLETE as u32;
                        m.goal = MonsterGoal::Inquiring;
                    }
                }
                _ => {}
            }
        }
    }
    ctx.objects.LoadingMapObjects = false;
}

crate::pending_fn!(pub fn draw_quest_log(ctx: &mut Ctx, out: &Surface), "quests.cpp|devilution::DrawQuestLog(const Surface &out)");

/// Original: `devilution::StartQuestlog` (quests.cpp).
// @port quests.cpp|devilution::StartQuestlog() sha=b03b9ff9ec07
pub fn start_questlog(ctx: &mut Ctx) {
    let s = &mut ctx.quests;
    s.encountered_quest_count = 0;
    for quest in s.Quests.iter() {
        if quest._qactive == QUEST_ACTIVE && quest._qlog {
            s.encountered_quests[s.encountered_quest_count as usize] = quest._qidx;
            s.encountered_quest_count += 1;
        }
    }
    s.first_finished_quest = s.encountered_quest_count;
    for quest in s.Quests.iter() {
        if quest._qactive == QUEST_DONE || quest._qactive == QUEST_HIVE_DONE {
            s.encountered_quests[s.encountered_quest_count as usize] = quest._qidx;
            s.encountered_quest_count += 1;
        }
    }
    let ff = s.first_finished_quest as usize;
    let n = s.encountered_quest_count as usize;
    let less = |a: &quest_id, b: &quest_id| QuestsData[*a as usize].questBookOrder < QuestsData[*b as usize].questBookOrder;
    crate::utils::stdsort::sort_by(&mut s.encountered_quests[..ff], less);
    crate::utils::stdsort::sort_by(&mut s.encountered_quests[ff..n], less);

    let two_blocks = s.first_finished_quest != 0 && s.first_finished_quest < s.encountered_quest_count;
    s.list_y_offset = 0;
    s.finished_quest_offset = if !two_blocks { 0 } else { LINE_HEIGHT / 2 };
    let overall_min_height = s.encountered_quest_count * LINE_HEIGHT + s.finished_quest_offset;
    let space = INNER_PANEL.size.height;
    if s.encountered_quest_count > 0 {
        let additional_space = space - overall_min_height;
        let mut add_line_spacing = additional_space / s.encountered_quest_count;
        add_line_spacing = (MAX_SPACING - LINE_HEIGHT).min(add_line_spacing);
        s.line_spacing = LINE_HEIGHT + add_line_spacing;
        if two_blocks {
            let mut additional_sep_space = additional_space - (add_line_spacing * s.encountered_quest_count);
            additional_sep_space = LINE_HEIGHT.min(additional_sep_space);
            s.finished_quest_offset = 4.max(additional_sep_space);
        }
        let overall_height = s.encountered_quest_count * s.line_spacing + s.finished_quest_offset;
        s.list_y_offset += (space - overall_height) / 2;
    }
    s.selected_quest = if s.first_finished_quest == 0 { -1 } else { 0 };
    s.QuestLogIsOpen = true;
}

/// Original: `devilution::QuestlogUp` (quests.cpp).
// @port quests.cpp|devilution::QuestlogUp() sha=2727be178b1a
pub fn questlog_up(ctx: &mut Ctx) {
    let s = &mut ctx.quests;
    if s.first_finished_quest == 0 {
        s.selected_quest = -1;
    } else {
        s.selected_quest -= 1;
        if s.selected_quest < 0 {
            s.selected_quest = s.first_finished_quest - 1;
        }
        crate::effects::play_sfx(ctx, IS_TITLEMOV);
    }
}

/// Original: `devilution::QuestlogDown` (quests.cpp).
// @port quests.cpp|devilution::QuestlogDown() sha=74c3c3e52106
pub fn questlog_down(ctx: &mut Ctx) {
    let s = &mut ctx.quests;
    if s.first_finished_quest == 0 {
        s.selected_quest = -1;
    } else {
        s.selected_quest += 1;
        if s.selected_quest == s.first_finished_quest {
            s.selected_quest = 0;
        }
        crate::effects::play_sfx(ctx, IS_TITLEMOV);
    }
}

/// Original: `devilution::QuestlogEnter` (quests.cpp).
// @port quests.cpp|devilution::QuestlogEnter() sha=e662aee986db
pub fn questlog_enter(ctx: &mut Ctx) {
    crate::effects::play_sfx(ctx, IS_TITLSLCT);
    let s = &ctx.quests;
    if s.encountered_quest_count != 0 && s.selected_quest >= 0 && s.selected_quest < s.first_finished_quest {
        let msg = s.Quests[s.encountered_quests[s.selected_quest as usize] as usize]._qmsg;
        crate::minitext::init_q_text_msg(ctx, msg);
    }
    ctx.quests.QuestLogIsOpen = false;
}

/// Original: `devilution::QuestlogESC` (quests.cpp).
// @port quests.cpp|devilution::QuestlogESC() sha=b162cbafbc3b
pub fn questlog_esc(ctx: &mut Ctx) {
    let l = quest_log_mouse_to_entry(ctx);
    if l != -1 {
        questlog_enter(ctx);
    }
}

/// Original: `devilution::SetMultiQuest` (quests.cpp).
// @port quests.cpp|devilution::SetMultiQuest(int q, quest_state s, bool log, int v1, int v2, int16_t qmsg) sha=b90c7ef4e82b
pub fn set_multi_quest(ctx: &mut Ctx, q: i32, s: quest_state, log: bool, v1: i32, v2: i32, qmsg: i32) {
    if ctx.init.gb_is_spawn {
        return;
    }
    let quest = &mut ctx.quests.Quests[q as usize];
    let old_quest_state = quest._qactive;
    if quest._qactive != QUEST_DONE {
        if s > quest._qactive
            || (matches!(s, QUEST_ACTIVE | QUEST_DONE) && matches!(quest._qactive, QUEST_HIVE_TEASE1 | QUEST_HIVE_TEASE2 | QUEST_HIVE_ACTIVE))
        {
            quest._qactive = s;
        }
        if log {
            quest._qlog = true;
        }
    }
    if v1 > quest._qvar1 as i32 {
        quest._qvar1 = v1 as u8;
    }
    quest._qvar2 = v2 as u8;
    quest._qmsg = qmsg as i16 as _speech_id;
    if !use_multiplayer_quests(ctx) {
        // Ensure that changes on another client is also updated on our own
        resync_quests(ctx);
        let quest = ctx.quests.Quests[q as usize];
        let quest_got_completed = old_quest_state != QUEST_DONE && quest._qactive == QUEST_DONE;
        let me = ctx.players.MyPlayer.expect("MyPlayer");
        if quest._qidx == Q_PWATER && quest_got_completed && ctx.players.Players[me].is_on_set_level(quest._qslvl) {
            start_p_water_purify(ctx);
        }
        if quest._qidx == Q_GIRL && quest_got_completed && ctx.players.Players[me].is_on_level(0) {
            crate::towners::update_girl_anim_after_quest_complete(ctx);
        }
        if quest._qidx == Q_JERSEY && quest_got_completed && ctx.players.Players[me].is_on_level(0) {
            crate::towners::update_cow_farmer_anim_after_quest_complete(ctx);
        }
    }
}

/// Original: `devilution::UseMultiplayerQuests` (quests.cpp).
// @port quests.cpp|devilution::UseMultiplayerQuests() sha=40835358513f
pub fn use_multiplayer_quests(ctx: &Ctx) -> bool {
    ctx.multi.sgGameInitInfo.fullQuests == 0
}

/// Original: `devilution::Quest::IsAvailable` (quests.cpp).
// @port quests.cpp|devilution::Quest::IsAvailable() sha=da37519d3513
pub fn is_quest_available(ctx: &Ctx, q: quest_id) -> bool {
    let quest = &ctx.quests.Quests[q as usize];
    if ctx.gendung.setlevel {
        return false;
    }
    if ctx.gendung.currlevel != quest._qlevel {
        return false;
    }
    if quest._qactive == QUEST_NOTAVAIL {
        return false;
    }
    if QuestsData[quest._qidx as usize].isSinglePlayerOnly && use_multiplayer_quests(ctx) {
        return false;
    }
    true
}
