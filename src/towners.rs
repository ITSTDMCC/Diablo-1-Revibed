//! `Source/towners.cpp`: the people (and cows) of Tristram.

use crate::ctx::Ctx;
use crate::effects_data::*;
use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList, ClxSpriteSheet};
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::enums::*;
use crate::inv::{has_inventory_item_with_id, has_inventory_or_belt_item, remove_inventory_item_by_id};
use crate::minitext::init_q_text_msg;
use crate::msg::net_send_cmd_quest;
use crate::quests::MAXQUESTS;
use crate::utils::language::tr;

pub const NUM_TOWNERS: usize = 16;
pub const NUM_TOWNER_TYPES: usize = TOWN_COWFARM as usize + 1;

/// `Towner::talk`: player index, towner index.
pub type TalkFn = fn(&mut Ctx, usize, usize);

/// `Towner`
#[derive(Clone)]
pub struct Towner {
    /// `ownedAnim`
    pub owned_anim: Option<ClxSpriteList>,
    /// `anim`
    pub anim: Option<ClxSpriteList>,
    /// `animOrder` (static data)
    pub anim_order: &'static [u8],
    pub talk: Option<TalkFn>,
    pub name: String,
    /// Tile position of NPC
    pub position: Point,
    /// Randomly chosen topic for discussion (picked when loading into town)
    pub gossip: _speech_id,
    pub _tAnimWidth: u16,
    pub _tAnimDelay: i16,
    pub _tAnimCnt: i16,
    pub _tAnimLen: u8,
    pub _tAnimFrame: u8,
    pub _tAnimFrameCnt: u8,
    /// `animOrderSize`
    pub anim_order_size: u8,
    pub _ttype: _talker_id,
}

impl Default for Towner {
    fn default() -> Self {
        Towner {
            owned_anim: None,
            anim: None,
            anim_order: &[],
            talk: None,
            name: String::new(),
            position: Point::default(),
            gossip: TEXT_NONE,
            _tAnimWidth: 0,
            _tAnimDelay: 0,
            _tAnimCnt: 0,
            _tAnimLen: 0,
            _tAnimFrame: 0,
            _tAnimFrameCnt: 0,
            anim_order_size: 0,
            _ttype: TOWN_SMITH,
        }
    }
}

impl Towner {
    /// `currentSprite`
    pub fn current_sprite(&self) -> ClxSprite {
        self.anim.as_ref().expect("towner anim").get(self._tAnimFrame as usize)
    }
}

/// Globals of towners.cpp.
pub struct TownersState {
    /// `Towners`
    pub towners: Vec<Towner>,
    /// `CowSprites`
    pub cow_sprites: Option<ClxSpriteSheet>,
    cow_msg: i32,
    cow_clicks: i32,
    /// `CowPlaying`
    cow_playing: SfxId,
    /// `QuestDialogTable`
    pub QuestDialogTable: [[_speech_id; MAXQUESTS]; NUM_TOWNER_TYPES],
}

impl Default for TownersState {
    fn default() -> Self {
        TownersState {
            towners: (0..NUM_TOWNERS).map(|_| Towner::default()).collect(),
            cow_sprites: None,
            cow_msg: 0,
            cow_clicks: 0,
            cow_playing: SFX_NONE,
            QuestDialogTable: QUEST_DIALOG_TABLE,
        }
    }
}

/// `TownerData`
struct TownerData {
    type_: _talker_id,
    position: Point,
    dir: Direction,
    init: fn(&mut Ctx, usize, &TownerData),
    talk: TalkFn,
}

/// Original: `NewTownerAnim` (towners.cpp).
// @port towners.cpp|devilution::NewTownerAnim(Towner &towner, ClxSpriteList sprites, uint8_t numFrames, int delay) sha=d11f27eff7a7
fn new_towner_anim(towner: &mut Towner, sprites: ClxSpriteList, num_frames: u8, delay: i32) {
    towner.anim = Some(sprites);
    towner._tAnimLen = num_frames;
    towner._tAnimFrame = 0;
    towner._tAnimCnt = 0;
    towner._tAnimDelay = delay as i16;
}

/// Original: `InitTownerInfo` (towners.cpp).
// @port towners.cpp|devilution::InitTownerInfo(int i, const TownerData &townerData) sha=27b84d8acc33
fn init_towner_info(ctx: &mut Ctx, i: usize, towner_data: &TownerData) {
    let towner = &mut ctx.towners.towners[i];
    towner._ttype = towner_data.type_;
    towner.position = towner_data.position;
    towner.talk = Some(towner_data.talk);
    let p = towner.position;
    ctx.gendung.dMonster[p.x as usize][p.y as usize] = (i + 1) as i16;
    (towner_data.init)(ctx, i, towner_data);
}

/// Original: `LoadTownerAnimations` (towners.cpp).
// @port towners.cpp|devilution::LoadTownerAnimations(Towner &towner, const char *path, int frames, int delay) sha=0906640b9307
fn load_towner_animations(ctx: &mut Ctx, t: usize, path: &str, frames: i32, delay: i32) {
    ctx.towners.towners[t].owned_anim = None;
    let w = ctx.towners.towners[t]._tAnimWidth;
    let cel = crate::engine::load_sprites::load_cel(ctx, path, w);
    let towner = &mut ctx.towners.towners[t];
    towner.owned_anim = Some(cel.clone());
    new_towner_anim(towner, cel, frames as u8, delay);
}

fn set_anim_order(ctx: &mut Ctx, t: usize, order: &'static [u8]) {
    let towner = &mut ctx.towners.towners[t];
    towner.anim_order = order;
    towner.anim_order_size = order.len() as u8;
}

#[rustfmt::skip]
static SMITH_ANIM_ORDER: &[u8] = &[
    4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4,
    4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4,
    4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4,
    4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4,
    4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4,
    4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3,
];

/// Original: `InitSmith` (towners.cpp).
// @port towners.cpp|devilution::InitSmith(Towner &towner, const TownerData &townerData) sha=d084d8ba024b
fn init_smith(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, SMITH_ANIM_ORDER);
    load_towner_animations(ctx, t, "towners\\smith\\smithn", 16, 3);
    ctx.towners.towners[t].name = tr("Griswold the Blacksmith");
    ctx.towners.towners[t].gossip = ctx.rng.pick_randomly_among(&[
        TEXT_GRISWOLD2, TEXT_GRISWOLD3, TEXT_GRISWOLD4, TEXT_GRISWOLD5, TEXT_GRISWOLD6, TEXT_GRISWOLD7, TEXT_GRISWOLD8, TEXT_GRISWOLD9, TEXT_GRISWOLD10,
        TEXT_GRISWOLD12, TEXT_GRISWOLD13,
    ]);
}

#[rustfmt::skip]
static BAR_OWNER_ANIM_ORDER: &[u8] = &[
    0, 1, 2, 2, 1, 0, 15, 14, 13, 13, 14, 15,
    0, 1, 2, 2, 1, 0, 15, 14, 13, 13, 14, 15,
    0, 1, 2, 2, 1, 0, 15, 14, 13, 13, 14, 15,
    0, 1, 2, 2, 1, 0, 15, 14, 13, 13, 14, 15,
    0, 1, 2, 2, 1, 0, 15, 14, 13, 13, 14, 15,
    0, 1, 2, 2, 1, 0, 15, 14, 13, 13, 14, 15,
    0, 1, 2, 2, 1, 0, 15, 14, 13, 13, 14, 15,
    0, 1, 2, 1, 0, 15, 14, 13, 13, 14, 15,
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
];

/// Original: `InitBarOwner` (towners.cpp).
// @port towners.cpp|devilution::InitBarOwner(Towner &towner, const TownerData &townerData) sha=6e3bae095bb5
fn init_bar_owner(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, BAR_OWNER_ANIM_ORDER);
    load_towner_animations(ctx, t, "towners\\twnf\\twnfn", 16, 3);
    ctx.towners.towners[t].name = tr("Ogden the Tavern owner");
    ctx.towners.towners[t].gossip =
        ctx.rng.pick_randomly_among(&[TEXT_OGDEN2, TEXT_OGDEN3, TEXT_OGDEN4, TEXT_OGDEN5, TEXT_OGDEN6, TEXT_OGDEN8, TEXT_OGDEN9, TEXT_OGDEN10]);
}

/// Original: `InitTownDead` (towners.cpp).
// @port towners.cpp|devilution::InitTownDead(Towner &towner, const TownerData &townerData) sha=379eeaf94fbf
fn init_town_dead(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, &[]);
    load_towner_animations(ctx, t, "towners\\butch\\deadguy", 8, 6);
    ctx.towners.towners[t].name = tr("Wounded Townsman");
}

#[rustfmt::skip]
static WITCH_ANIM_ORDER: &[u8] = &[
     3,  3,  3,  4,  5,  5,  5,  4,  3, 14, 13, 12, 12, 12, 13, 14, 3, 4, 5, 5, 5, 4,
     3,  3,  3,  4,  5,  5,  5,  4,  3, 14, 13, 12, 12, 12, 13, 14, 3, 4, 5, 5, 5, 4,
     3,  3,  3,  4,  5,  5,  5,  4,  3, 14, 13, 12, 12, 12, 13, 14, 3, 4, 5, 5, 5, 4,
     3,  2,  1,  0, 18, 17, 18,  0,  1,  0, 18, 17, 18,  0,  1,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14,
    14, 14, 13, 12, 12, 12, 12, 13, 14,
    14, 14, 13, 12, 11, 11, 11, 10,  9,  9,  9,  8,
     7,  8,  9,  9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
     0,  1,  0, 18, 17, 18,  0,  1,  0,  1,  2,
];

/// Original: `InitWitch` (towners.cpp).
// @port towners.cpp|devilution::InitWitch(Towner &towner, const TownerData &townerData) sha=6bb0908361e2
fn init_witch(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, WITCH_ANIM_ORDER);
    load_towner_animations(ctx, t, "towners\\townwmn1\\witch", 19, 6);
    ctx.towners.towners[t].name = tr("Adria the Witch");
    ctx.towners.towners[t].gossip = ctx.rng.pick_randomly_among(&[
        TEXT_ADRIA2, TEXT_ADRIA3, TEXT_ADRIA4, TEXT_ADRIA5, TEXT_ADRIA6, TEXT_ADRIA7, TEXT_ADRIA8, TEXT_ADRIA9, TEXT_ADRIA10, TEXT_ADRIA12, TEXT_ADRIA13,
    ]);
}

/// Original: `InitBarmaid` (towners.cpp).
// @port towners.cpp|devilution::InitBarmaid(Towner &towner, const TownerData &townerData) sha=a427ee308358
fn init_barmaid(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, &[]);
    load_towner_animations(ctx, t, "towners\\townwmn1\\wmnn", 18, 6);
    ctx.towners.towners[t].name = tr("Gillian the Barmaid");
    ctx.towners.towners[t].gossip = ctx.rng.pick_randomly_among(&[
        TEXT_GILLIAN2, TEXT_GILLIAN3, TEXT_GILLIAN4, TEXT_GILLIAN5, TEXT_GILLIAN6, TEXT_GILLIAN7, TEXT_GILLIAN9, TEXT_GILLIAN10,
    ]);
}

/// Original: `InitBoy` (towners.cpp).
// @port towners.cpp|devilution::InitBoy(Towner &towner, const TownerData &townerData) sha=d2882f63d4db
fn init_boy(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, &[]);
    load_towner_animations(ctx, t, "towners\\townboy\\pegkid1", 20, 6);
    ctx.towners.towners[t].name = tr("Wirt the Peg-legged boy");
    ctx.towners.towners[t].gossip = ctx.rng.pick_randomly_among(&[
        TEXT_WIRT2, TEXT_WIRT3, TEXT_WIRT4, TEXT_WIRT5, TEXT_WIRT6, TEXT_WIRT7, TEXT_WIRT8, TEXT_WIRT9, TEXT_WIRT11, TEXT_WIRT12,
    ]);
}

#[rustfmt::skip]
static HEALER_ANIM_ORDER: &[u8] = &[
     0,  1,  2,  2,  1,  0, 19, 18, 18, 19,
     0,  1,  2,  2,  1,  0, 19, 18, 18, 19,
     0,  1,  2,  2,  1,  0, 19, 18, 18, 19,
     0,  1,  2,  2,  1,  0, 19, 18, 18, 19,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
    14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,
     4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15,
    14, 13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,
     4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
];

/// Original: `InitHealer` (towners.cpp).
// @port towners.cpp|devilution::InitHealer(Towner &towner, const TownerData &townerData) sha=93cd71767705
fn init_healer(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, HEALER_ANIM_ORDER);
    load_towner_animations(ctx, t, "towners\\healer\\healer", 20, 6);
    ctx.towners.towners[t].name = tr("Pepin the Healer");
    ctx.towners.towners[t].gossip = ctx.rng.pick_randomly_among(&[
        TEXT_PEPIN2, TEXT_PEPIN3, TEXT_PEPIN4, TEXT_PEPIN5, TEXT_PEPIN6, TEXT_PEPIN7, TEXT_PEPIN9, TEXT_PEPIN10, TEXT_PEPIN11,
    ]);
}

#[rustfmt::skip]
static TELLER_ANIM_ORDER: &[u8] = &[
     0,  0, 24, 24, 23, 22, 21, 20, 19, 18, 17, 16, 15, 14,
    15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 24, 24,  0,  0,  0, 24,
     0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14,
    13, 12, 11, 10,  9,  8,  7,  6,  5,  4,  3,  2,  1,  0,
];

/// Original: `InitTeller` (towners.cpp).
// @port towners.cpp|devilution::InitTeller(Towner &towner, const TownerData &townerData) sha=b118c7005947
fn init_teller(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, TELLER_ANIM_ORDER);
    load_towner_animations(ctx, t, "towners\\strytell\\strytell", 25, 3);
    ctx.towners.towners[t].name = tr("Cain the Elder");
    ctx.towners.towners[t].gossip = ctx.rng.pick_randomly_among(&[
        TEXT_STORY2, TEXT_STORY3, TEXT_STORY4, TEXT_STORY5, TEXT_STORY6, TEXT_STORY7, TEXT_STORY9, TEXT_STORY10, TEXT_STORY11,
    ]);
}

#[rustfmt::skip]
static DRUNK_ANIM_ORDER: &[u8] = &[
    0, 0, 0,  1,  2,  3,  4,  5,  6,  7,  8, 9, 10, 10, 10, 10, 11, 12, 13, 14, 15, 16, 17, 17,
    0, 0, 0, 17, 16, 15, 14, 13, 12, 11, 10, 9, 10, 11, 12, 13, 14, 15, 16, 17,
    0, 1, 2,  3,  4,  4,  4,  3,  2,  1,
];

/// Original: `InitDrunk` (towners.cpp).
// @port towners.cpp|devilution::InitDrunk(Towner &towner, const TownerData &townerData) sha=4e8640981a71
fn init_drunk(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, DRUNK_ANIM_ORDER);
    load_towner_animations(ctx, t, "towners\\drunk\\twndrunk", 18, 3);
    ctx.towners.towners[t].name = tr("Farnham the Drunk");
    ctx.towners.towners[t].gossip = ctx.rng.pick_randomly_among(&[
        TEXT_FARNHAM2, TEXT_FARNHAM3, TEXT_FARNHAM4, TEXT_FARNHAM5, TEXT_FARNHAM6, TEXT_FARNHAM8, TEXT_FARNHAM9, TEXT_FARNHAM10, TEXT_FARNHAM11,
        TEXT_FARNHAM12, TEXT_FARNHAM13,
    ]);
}

/// Original: `InitCows` (towners.cpp).
// @port towners.cpp|devilution::InitCows(Towner &towner, const TownerData &townerData) sha=4ff56c1827d1
fn init_cows(ctx: &mut Ctx, t: usize, d: &TownerData) {
    let sprites = ctx.towners.cow_sprites.as_ref().expect("CowSprites").get(d.dir as usize);
    let frame = {
        let towner = &mut ctx.towners.towners[t];
        towner._tAnimWidth = 128;
        towner.anim_order = &[];
        towner.anim_order_size = 0;
        new_towner_anim(towner, sprites, 12, 3);
        ctx.rng.generate_rnd(11)
    };
    let towner = &mut ctx.towners.towners[t];
    towner._tAnimFrame = frame as u8;
    towner.name = tr("Cow");

    let position = d.position;
    let cow_id = ctx.gendung.dMonster[position.x as usize][position.y as usize];
    for dir in [Direction::NorthWest, Direction::NorthEast, Direction::North] {
        let offset = position + dir;
        ctx.gendung.dMonster[offset.x as usize][offset.y as usize] = -cow_id;
    }
}

/// Original: `InitFarmer` (towners.cpp).
// @port towners.cpp|devilution::InitFarmer(Towner &towner, const TownerData &townerData) sha=f44f47b6976f
fn init_farmer(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, &[]);
    load_towner_animations(ctx, t, "towners\\farmer\\farmrn2", 15, 3);
    ctx.towners.towners[t].name = tr("Lester the farmer");
}

/// Original: `InitCowFarmer` (towners.cpp).
// @port towners.cpp|devilution::InitCowFarmer(Towner &towner, const TownerData &townerData) sha=62cfbc81bdb9
fn init_cow_farmer(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    let mut cel_path = "towners\\farmer\\cfrmrn2";
    if ctx.quests.Quests[Q_JERSEY as usize]._qactive == QUEST_DONE {
        cel_path = "towners\\farmer\\mfrmrn2";
    }
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, &[]);
    load_towner_animations(ctx, t, cel_path, 15, 3);
    ctx.towners.towners[t].name = tr("Complete Nut");
}

/// Original: `InitGirl` (towners.cpp).
// @port towners.cpp|devilution::InitGirl(Towner &towner, const TownerData &townerData) sha=1e3d12eb8d7e
fn init_girl(ctx: &mut Ctx, t: usize, _d: &TownerData) {
    ctx.towners.towners[t]._tAnimWidth = 96;
    set_anim_order(ctx, t, &[]);
    load_towner_animations(ctx, t, "towners\\girl\\girlw1", 20, 6);
    ctx.towners.towners[t].name = tr("Celia");
}

/// Original: `TownDead` (towners.cpp).
// @port towners.cpp|devilution::TownDead(Towner &towner) sha=aada66fa81f6
fn town_dead(ctx: &mut Ctx, t: usize) {
    let butcher = ctx.quests.Quests[Q_BUTCHER as usize];
    if crate::minitext::qtextflag(ctx) {
        if butcher._qvar1 == 1 {
            ctx.towners.towners[t]._tAnimCnt = 0;
        }
        return;
    }
    let towner = &mut ctx.towners.towners[t];
    if (butcher._qactive == QUEST_DONE || butcher._qvar1 == 1) && towner._tAnimLen != 1 {
        towner._tAnimLen = 1;
        towner.name = tr("Slain Townsman");
    }
}

/// Original: `TownerTalk` (towners.cpp).
// @port towners.cpp|devilution::TownerTalk(_speech_id message) sha=34f710b42a0f
fn towner_talk(ctx: &mut Ctx, message: _speech_id) {
    ctx.towners.cow_clicks = 0;
    ctx.towners.cow_msg = 0;
    init_q_text_msg(ctx, message);
}

fn q(ctx: &mut Ctx, id: quest_id) -> &mut crate::quests::Quest {
    &mut ctx.quests.Quests[id as usize]
}

fn send_quest(ctx: &mut Ctx, id: quest_id) {
    net_send_cmd_quest(ctx, true, id as usize);
}

fn spawn_unique_at(ctx: &mut Ctx, uid: _unique_items, position: Point, level: u8) {
    crate::items::spawn_unique(ctx, uid, position, Some(level as i32), true, false);
}

/// Original: `TalkToBarOwner` (towners.cpp).
// @port towners.cpp|devilution::TalkToBarOwner(Player &player, Towner &barOwner) sha=aeca4bdb7152
fn talk_to_bar_owner(ctx: &mut Ctx, pnum: usize, t: usize) {
    let visited = ctx.players.Players[pnum]._pLvlVisited;
    if !visited[0] {
        init_q_text_msg(ctx, TEXT_INTRO);
        return;
    }
    if q(ctx, Q_SKELKING)._qactive != QUEST_NOTAVAIL && (visited[2] || visited[4]) {
        if q(ctx, Q_SKELKING)._qvar2 == 0 {
            let king = q(ctx, Q_SKELKING);
            king._qvar2 = 1;
            king._qlog = true;
            if king._qactive == QUEST_INIT {
                king._qactive = QUEST_ACTIVE;
                king._qvar1 = 1;
            }
            init_q_text_msg(ctx, TEXT_KING2);
            send_quest(ctx, Q_SKELKING);
            return;
        }
        if q(ctx, Q_SKELKING)._qactive == QUEST_DONE && q(ctx, Q_SKELKING)._qvar2 == 1 {
            let king = q(ctx, Q_SKELKING);
            king._qvar2 = 2;
            king._qvar1 = 2;
            init_q_text_msg(ctx, TEXT_KING4);
            send_quest(ctx, Q_SKELKING);
            return;
        }
    }
    if q(ctx, Q_LTBANNER)._qactive != QUEST_NOTAVAIL && (visited[3] || visited[4]) && q(ctx, Q_LTBANNER)._qactive != QUEST_DONE {
        if q(ctx, Q_LTBANNER)._qvar2 == 0 {
            let banner = q(ctx, Q_LTBANNER);
            banner._qvar2 = 1;
            if banner._qactive == QUEST_INIT {
                banner._qvar1 = 1;
                banner._qactive = QUEST_ACTIVE;
            }
            banner._qlog = true;
            send_quest(ctx, Q_LTBANNER);
            init_q_text_msg(ctx, TEXT_BANNER2);
            return;
        }
        if q(ctx, Q_LTBANNER)._qvar2 == 1 && remove_inventory_item_by_id(ctx, pnum, IDI_BANNER) {
            let banner = q(ctx, Q_LTBANNER);
            banner._qactive = QUEST_DONE;
            banner._qvar1 = 3;
            let lvl = banner._qlevel;
            send_quest(ctx, Q_LTBANNER);
            let pos = ctx.towners.towners[t].position + Direction::SouthWest;
            spawn_unique_at(ctx, UITEM_HARCREST, pos, lvl);
            init_q_text_msg(ctx, TEXT_BANNER3);
            return;
        }
    }
    towner_talk(ctx, TEXT_OGDEN1);
    crate::stores::start_store(ctx, crate::enums::TalkID::Tavern);
}

/// Original: `TalkToDeadguy` (towners.cpp).
// @port towners.cpp|devilution::TalkToDeadguy(Player &player, Towner &) sha=90adaf3b6758
fn talk_to_deadguy(ctx: &mut Ctx, pnum: usize, _t: usize) {
    let quest = q(ctx, Q_BUTCHER);
    if quest._qactive == QUEST_DONE {
        return;
    }
    if quest._qvar1 == 1 {
        crate::player::player_say_specific(ctx, pnum, HeroSpeech::YourDeathWillBeAvenged);
        return;
    }
    quest._qactive = QUEST_ACTIVE;
    quest._qlog = true;
    quest._qmsg = TEXT_BUTCH9;
    quest._qvar1 = 1;
    init_q_text_msg(ctx, TEXT_BUTCH9);
    send_quest(ctx, Q_BUTCHER);
}

/// Original: `TalkToBlackSmith` (towners.cpp).
// @port towners.cpp|devilution::TalkToBlackSmith(Player &player, Towner &blackSmith) sha=a869a1064ccf
fn talk_to_black_smith(ctx: &mut Ctx, pnum: usize, t: usize) {
    let visited = ctx.players.Players[pnum]._pLvlVisited;
    let pos = ctx.towners.towners[t].position + Direction::SouthWest;
    if q(ctx, Q_ROCK)._qactive != QUEST_NOTAVAIL && (visited[4] || visited[5]) && q(ctx, Q_ROCK)._qactive != QUEST_DONE {
        if q(ctx, Q_ROCK)._qvar2 == 0 {
            let rock = q(ctx, Q_ROCK);
            rock._qvar2 = 1;
            rock._qlog = true;
            if rock._qactive == QUEST_INIT {
                rock._qactive = QUEST_ACTIVE;
            }
            send_quest(ctx, Q_ROCK);
            init_q_text_msg(ctx, TEXT_INFRA5);
            return;
        }
        if q(ctx, Q_ROCK)._qvar2 == 1 && remove_inventory_item_by_id(ctx, pnum, IDI_ROCK) {
            q(ctx, Q_ROCK)._qactive = QUEST_DONE;
            send_quest(ctx, Q_ROCK);
            let lvl = q(ctx, Q_ROCK)._qlevel;
            spawn_unique_at(ctx, UITEM_INFRARING, pos, lvl);
            init_q_text_msg(ctx, TEXT_INFRA7);
            return;
        }
    }
    if !matches!(q(ctx, Q_ANVIL)._qactive, QUEST_NOTAVAIL | QUEST_DONE) {
        if (visited[9] || visited[10]) && q(ctx, Q_ANVIL)._qvar2 == 0 {
            let anvil = q(ctx, Q_ANVIL);
            anvil._qvar2 = 1;
            anvil._qlog = true;
            if anvil._qactive == QUEST_INIT {
                anvil._qactive = QUEST_ACTIVE;
            }
            send_quest(ctx, Q_ANVIL);
            init_q_text_msg(ctx, TEXT_ANVIL5);
            return;
        }
        if q(ctx, Q_ANVIL)._qvar2 == 1 && remove_inventory_item_by_id(ctx, pnum, IDI_ANVIL) {
            q(ctx, Q_ANVIL)._qactive = QUEST_DONE;
            send_quest(ctx, Q_ANVIL);
            let lvl = q(ctx, Q_ANVIL)._qlevel;
            spawn_unique_at(ctx, UITEM_GRISWOLD, pos, lvl);
            init_q_text_msg(ctx, TEXT_ANVIL7);
            return;
        }
    }
    towner_talk(ctx, TEXT_GRISWOLD1);
    crate::stores::start_store(ctx, crate::enums::TalkID::Smith);
}

/// Original: `TalkToWitch` (towners.cpp).
// @port towners.cpp|devilution::TalkToWitch(Player &player, Towner &) sha=1b7db13cd168
fn talk_to_witch(ctx: &mut Ctx, pnum: usize, _t: usize) {
    if q(ctx, Q_MUSHROOM)._qactive != QUEST_NOTAVAIL {
        if q(ctx, Q_MUSHROOM)._qactive == QUEST_INIT && remove_inventory_item_by_id(ctx, pnum, IDI_FUNGALTM) {
            let m = q(ctx, Q_MUSHROOM);
            m._qactive = QUEST_ACTIVE;
            m._qlog = true;
            m._qvar1 = QS_TOMEGIVEN as u8;
            send_quest(ctx, Q_MUSHROOM);
            init_q_text_msg(ctx, TEXT_MUSH8);
            return;
        }
        if q(ctx, Q_MUSHROOM)._qactive == QUEST_ACTIVE {
            let v1 = q(ctx, Q_MUSHROOM)._qvar1 as i32;
            if v1 >= QS_TOMEGIVEN && v1 < QS_MUSHGIVEN {
                if remove_inventory_item_by_id(ctx, pnum, IDI_MUSHROOM) {
                    q(ctx, Q_MUSHROOM)._qvar1 = QS_MUSHGIVEN as u8;
                    ctx.towners.QuestDialogTable[TOWN_HEALER as usize][Q_MUSHROOM as usize] = TEXT_MUSH3;
                    ctx.towners.QuestDialogTable[TOWN_WITCH as usize][Q_MUSHROOM as usize] = TEXT_NONE;
                    q(ctx, Q_MUSHROOM)._qmsg = TEXT_MUSH10;
                    send_quest(ctx, Q_MUSHROOM);
                    init_q_text_msg(ctx, TEXT_MUSH10);
                    return;
                }
                if q(ctx, Q_MUSHROOM)._qmsg != TEXT_MUSH9 {
                    q(ctx, Q_MUSHROOM)._qmsg = TEXT_MUSH9;
                    send_quest(ctx, Q_MUSHROOM);
                    init_q_text_msg(ctx, TEXT_MUSH9);
                    return;
                }
            }
            if q(ctx, Q_MUSHROOM)._qvar1 as i32 >= QS_MUSHGIVEN {
                if has_inventory_item_with_id(ctx, pnum, IDI_BRAIN) {
                    q(ctx, Q_MUSHROOM)._qmsg = TEXT_MUSH11;
                    send_quest(ctx, Q_MUSHROOM);
                    init_q_text_msg(ctx, TEXT_MUSH11);
                    return;
                }
                if has_inventory_or_belt_item(ctx, pnum, |i| i.IDidx == IDI_SPECELIX) {
                    q(ctx, Q_MUSHROOM)._qactive = QUEST_DONE;
                    send_quest(ctx, Q_MUSHROOM);
                    init_q_text_msg(ctx, TEXT_MUSH12);
                    return;
                }
            }
        }
    }
    towner_talk(ctx, TEXT_ADRIA1);
    crate::stores::start_store(ctx, crate::enums::TalkID::Witch);
}

/// Original: `TalkToBarmaid` (towners.cpp).
// @port towners.cpp|devilution::TalkToBarmaid(Player &player, Towner &) sha=6e60b22a2708
fn talk_to_barmaid(ctx: &mut Ctx, pnum: usize, _t: usize) {
    if !ctx.players.Players[pnum]._pLvlVisited[21] && has_inventory_item_with_id(ctx, pnum, IDI_MAPOFDOOM) && q(ctx, Q_GRAVE)._qmsg != TEXT_GRAVE8 {
        let g = q(ctx, Q_GRAVE);
        g._qactive = QUEST_ACTIVE;
        g._qlog = true;
        g._qmsg = TEXT_GRAVE8;
        send_quest(ctx, Q_GRAVE);
        init_q_text_msg(ctx, TEXT_GRAVE8);
        return;
    }
    towner_talk(ctx, TEXT_GILLIAN1);
    crate::stores::start_store(ctx, crate::enums::TalkID::Barmaid);
}

/// Original: `TalkToDrunk` (towners.cpp).
// @port towners.cpp|devilution::TalkToDrunk(Player & , Towner &) sha=a40ba890cd33
fn talk_to_drunk(ctx: &mut Ctx, _pnum: usize, _t: usize) {
    towner_talk(ctx, TEXT_FARNHAM1);
    crate::stores::start_store(ctx, crate::enums::TalkID::Drunk);
}

/// Original: `TalkToHealer` (towners.cpp).
// @port towners.cpp|devilution::TalkToHealer(Player &player, Towner &healer) sha=0d503dd7eb0c
fn talk_to_healer(ctx: &mut Ctx, pnum: usize, t: usize) {
    let visited = ctx.players.Players[pnum]._pLvlVisited;
    let pw = *q(ctx, Q_PWATER);
    if pw._qactive != QUEST_NOTAVAIL {
        if (pw._qactive == QUEST_INIT && (visited[1] || visited[5])) || (pw._qactive == QUEST_ACTIVE && !pw._qlog) {
            let p = q(ctx, Q_PWATER);
            p._qactive = QUEST_ACTIVE;
            p._qlog = true;
            p._qmsg = TEXT_POISON3;
            init_q_text_msg(ctx, TEXT_POISON3);
            send_quest(ctx, Q_PWATER);
            return;
        }
        if pw._qactive == QUEST_DONE && pw._qvar1 != 2 {
            q(ctx, Q_PWATER)._qvar1 = 2;
            init_q_text_msg(ctx, TEXT_POISON5);
            let pos = ctx.towners.towners[t].position + Direction::SouthWest;
            spawn_unique_at(ctx, UITEM_TRING, pos, pw._qlevel);
            send_quest(ctx, Q_PWATER);
            return;
        }
    }
    if q(ctx, Q_MUSHROOM)._qactive == QUEST_ACTIVE {
        let v1 = q(ctx, Q_MUSHROOM)._qvar1 as i32;
        if v1 >= QS_MUSHGIVEN && v1 < QS_BRAINGIVEN && remove_inventory_item_by_id(ctx, pnum, IDI_BRAIN) {
            let pos = ctx.towners.towners[t].position + Displacement::new(0, 1);
            crate::items::spawn_quest_item(ctx, IDI_SPECELIX, pos, 0, 0, true);
            init_q_text_msg(ctx, TEXT_MUSH4);
            q(ctx, Q_MUSHROOM)._qvar1 = QS_BRAINGIVEN as u8;
            ctx.towners.QuestDialogTable[TOWN_HEALER as usize][Q_MUSHROOM as usize] = TEXT_NONE;
            send_quest(ctx, Q_MUSHROOM);
            return;
        }
    }
    towner_talk(ctx, TEXT_PEPIN1);
    crate::stores::start_store(ctx, crate::enums::TalkID::Healer);
}

/// Original: `TalkToBoy` (towners.cpp).
// @port towners.cpp|devilution::TalkToBoy(Player & , Towner &) sha=e6a9792c891b
fn talk_to_boy(ctx: &mut Ctx, _pnum: usize, _t: usize) {
    towner_talk(ctx, TEXT_WIRT1);
    crate::stores::start_store(ctx, crate::enums::TalkID::Boy);
}

/// Original: `TalkToStoryteller` (towners.cpp).
// @port towners.cpp|devilution::TalkToStoryteller(Player &player, Towner &) sha=776f6bb0639e
fn talk_to_storyteller(ctx: &mut Ctx, pnum: usize, _t: usize) {
    if !crate::quests::use_multiplayer_quests(ctx) {
        if q(ctx, Q_BETRAYER)._qactive == QUEST_INIT && remove_inventory_item_by_id(ctx, pnum, IDI_LAZSTAFF) {
            init_q_text_msg(ctx, TEXT_VILE1);
            let b = q(ctx, Q_BETRAYER);
            b._qlog = true;
            b._qactive = QUEST_ACTIVE;
            b._qvar1 = 2;
            send_quest(ctx, Q_BETRAYER);
            return;
        }
    } else if q(ctx, Q_BETRAYER)._qactive == QUEST_ACTIVE && !q(ctx, Q_BETRAYER)._qlog {
        init_q_text_msg(ctx, TEXT_VILE1);
        q(ctx, Q_BETRAYER)._qlog = true;
        send_quest(ctx, Q_BETRAYER);
        return;
    }
    if q(ctx, Q_BETRAYER)._qactive == QUEST_DONE && q(ctx, Q_BETRAYER)._qvar1 == 7 {
        q(ctx, Q_BETRAYER)._qvar1 = 8;
        init_q_text_msg(ctx, TEXT_VILE3);
        q(ctx, Q_DIABLO)._qlog = true;
        if ctx.init.gb_is_multiplayer {
            send_quest(ctx, Q_BETRAYER);
            send_quest(ctx, Q_DIABLO);
        }
        return;
    }
    towner_talk(ctx, TEXT_STORY1);
    crate::stores::start_store(ctx, crate::enums::TalkID::Storyteller);
}

/// Original: `TalkToCow` (towners.cpp).
// @port towners.cpp|devilution::TalkToCow(Player &player, Towner &cow) sha=e7c0e9df83ca
fn talk_to_cow(ctx: &mut Ctx, pnum: usize, t: usize) {
    if ctx.towners.cow_playing != SFX_NONE && crate::effects::effect_is_playing(ctx, ctx.towners.cow_playing as i32) {
        return;
    }
    ctx.towners.cow_clicks += 1;
    ctx.towners.cow_playing = TSFX_COW1;
    let spawn = ctx.init.gb_is_spawn;
    if ctx.towners.cow_clicks == 4 {
        if spawn {
            ctx.towners.cow_clicks = 0;
        }
        ctx.towners.cow_playing = TSFX_COW2;
    } else if ctx.towners.cow_clicks >= 8 && !spawn {
        ctx.towners.cow_clicks = 4;
        const SN_SFX: [HeroSpeech; 3] = [HeroSpeech::YepThatsACowAlright, HeroSpeech::ImNotThirsty, HeroSpeech::ImNoMilkmaid];
        let m = ctx.towners.cow_msg as usize;
        crate::player::player_say_specific(ctx, pnum, SN_SFX[m]);
        ctx.towners.cow_msg += 1;
        if ctx.towners.cow_msg >= 3 {
            ctx.towners.cow_msg = 0;
        }
    }
    let pos = ctx.towners.towners[t].position;
    crate::effects::play_sfx_loc(ctx, ctx.towners.cow_playing, pos, true);
}

/// Original: `TalkToFarmer` (towners.cpp).
// @port towners.cpp|devilution::TalkToFarmer(Player &player, Towner &farmer) sha=03a92e10d853
fn talk_to_farmer(ctx: &mut Ctx, pnum: usize, t: usize) {
    let mp = ctx.init.gb_is_multiplayer;
    let pos = ctx.towners.towners[t].position + Displacement::new(1, 0);
    match q(ctx, Q_FARMER)._qactive {
        QUEST_NOTAVAIL | QUEST_INIT => {
            if has_inventory_item_with_id(ctx, pnum, IDI_RUNEBOMB) {
                init_q_text_msg(ctx, TEXT_FARMER2);
                let f = q(ctx, Q_FARMER);
                f._qactive = QUEST_ACTIVE;
                f._qvar1 = 1;
                f._qmsg = TEXT_FARMER1;
                f._qlog = true;
                if mp {
                    send_quest(ctx, Q_FARMER);
                }
                return;
            }
            let p = &ctx.players.Players[pnum];
            if !p._pLvlVisited[9] && p._pLevel < 15 {
                let mut qt = TEXT_FARMER8;
                if p._pLvlVisited[2] {
                    qt = TEXT_FARMER5;
                }
                if p._pLvlVisited[5] {
                    qt = TEXT_FARMER7;
                }
                if p._pLvlVisited[7] {
                    qt = TEXT_FARMER9;
                }
                init_q_text_msg(ctx, qt);
                return;
            }
            init_q_text_msg(ctx, TEXT_FARMER1);
            let f = q(ctx, Q_FARMER);
            f._qactive = QUEST_ACTIVE;
            f._qvar1 = 1;
            f._qlog = true;
            f._qmsg = TEXT_FARMER1;
            crate::items::spawn_rune_bomb(ctx, pos, true);
            if mp {
                send_quest(ctx, Q_FARMER);
            }
        }
        QUEST_ACTIVE => {
            let m = if has_inventory_item_with_id(ctx, pnum, IDI_RUNEBOMB) { TEXT_FARMER2 } else { TEXT_FARMER3 };
            init_q_text_msg(ctx, m);
        }
        QUEST_DONE => {
            init_q_text_msg(ctx, TEXT_FARMER4);
            crate::items::spawn_reward_item(ctx, IDI_AURIC, pos, true);
            q(ctx, Q_FARMER)._qactive = QUEST_HIVE_DONE;
            if mp {
                send_quest(ctx, Q_FARMER);
            }
        }
        QUEST_HIVE_DONE => {}
        _ => init_q_text_msg(ctx, TEXT_FARMER4),
    }
}

/// Original: `TalkToCowFarmer` (towners.cpp).
// @port towners.cpp|devilution::TalkToCowFarmer(Player &player, Towner &cowFarmer) sha=e8f133c94a93
fn talk_to_cow_farmer(ctx: &mut Ctx, pnum: usize, t: usize) {
    if remove_inventory_item_by_id(ctx, pnum, IDI_GREYSUIT) {
        init_q_text_msg(ctx, TEXT_JERSEY7);
        return;
    }
    let tpos = ctx.towners.towners[t].position;
    if remove_inventory_item_by_id(ctx, pnum, IDI_BROWNSUIT) {
        let lvl = q(ctx, Q_JERSEY)._qlevel;
        spawn_unique_at(ctx, UITEM_BOVINE, tpos + Direction::SouthEast, lvl);
        init_q_text_msg(ctx, TEXT_JERSEY8);
        q(ctx, Q_JERSEY)._qactive = QUEST_DONE;
        update_cow_farmer_anim_after_quest_complete(ctx);
        send_quest(ctx, Q_JERSEY);
        return;
    }
    if has_inventory_item_with_id(ctx, pnum, IDI_RUNEBOMB) {
        init_q_text_msg(ctx, TEXT_JERSEY5);
        let j = q(ctx, Q_JERSEY);
        j._qactive = QUEST_ACTIVE;
        j._qvar1 = 1;
        j._qmsg = TEXT_JERSEY4;
        j._qlog = true;
        send_quest(ctx, Q_JERSEY);
        return;
    }
    let mp = ctx.init.gb_is_multiplayer;
    match q(ctx, Q_JERSEY)._qactive {
        QUEST_NOTAVAIL | QUEST_INIT => {
            init_q_text_msg(ctx, TEXT_JERSEY1);
            q(ctx, Q_JERSEY)._qactive = QUEST_HIVE_TEASE1;
            if mp {
                send_quest(ctx, Q_JERSEY);
            }
        }
        QUEST_DONE => init_q_text_msg(ctx, TEXT_JERSEY1),
        QUEST_HIVE_TEASE1 => {
            init_q_text_msg(ctx, TEXT_JERSEY2);
            q(ctx, Q_JERSEY)._qactive = QUEST_HIVE_TEASE2;
            if mp {
                send_quest(ctx, Q_JERSEY);
            }
        }
        QUEST_HIVE_TEASE2 => {
            init_q_text_msg(ctx, TEXT_JERSEY3);
            q(ctx, Q_JERSEY)._qactive = QUEST_HIVE_ACTIVE;
            if mp {
                send_quest(ctx, Q_JERSEY);
            }
        }
        QUEST_HIVE_ACTIVE => {
            let p = &ctx.players.Players[pnum];
            if !p._pLvlVisited[9] && p._pLevel < 15 {
                let qt = match ctx.rng.generate_rnd(4) {
                    0 => TEXT_JERSEY9,
                    1 => TEXT_JERSEY10,
                    2 => TEXT_JERSEY11,
                    _ => TEXT_JERSEY12,
                };
                init_q_text_msg(ctx, qt);
                return;
            }
            init_q_text_msg(ctx, TEXT_JERSEY4);
            let j = q(ctx, Q_JERSEY);
            j._qactive = QUEST_ACTIVE;
            j._qvar1 = 1;
            j._qmsg = TEXT_JERSEY4;
            j._qlog = true;
            crate::items::spawn_rune_bomb(ctx, tpos + Displacement::new(1, 0), true);
            if mp {
                send_quest(ctx, Q_JERSEY);
            }
        }
        _ => init_q_text_msg(ctx, TEXT_JERSEY5),
    }
}

/// Original: `TalkToGirl` (towners.cpp).
// @port towners.cpp|devilution::TalkToGirl(Player &player, Towner &girl) sha=f572a954b5ef
fn talk_to_girl(ctx: &mut Ctx, pnum: usize, t: usize) {
    let mp = ctx.init.gb_is_multiplayer;
    if q(ctx, Q_GIRL)._qactive != QUEST_DONE && remove_inventory_item_by_id(ctx, pnum, IDI_THEODORE) {
        init_q_text_msg(ctx, TEXT_GIRL4);
        let pos = ctx.towners.towners[t].position;
        crate::items::create_amulet(ctx, pos, 13, false, false, true);
        q(ctx, Q_GIRL)._qactive = QUEST_DONE;
        update_girl_anim_after_quest_complete(ctx);
        if mp {
            send_quest(ctx, Q_GIRL);
        }
        return;
    }
    match q(ctx, Q_GIRL)._qactive {
        QUEST_NOTAVAIL | QUEST_INIT => {
            init_q_text_msg(ctx, TEXT_GIRL2);
            let g = q(ctx, Q_GIRL);
            g._qactive = QUEST_ACTIVE;
            g._qvar1 = 1;
            g._qlog = true;
            g._qmsg = TEXT_GIRL2;
            if mp {
                send_quest(ctx, Q_GIRL);
            }
        }
        QUEST_ACTIVE => init_q_text_msg(ctx, TEXT_GIRL3),
        _ => {}
    }
}

/// `TownersData`
#[rustfmt::skip]
static TOWNERS_DATA: [TownerData; 15] = [
    TownerData { type_: TOWN_SMITH,   position: Point::new(62, 63), dir: Direction::SouthWest, init: init_smith,      talk: talk_to_black_smith },
    TownerData { type_: TOWN_HEALER,  position: Point::new(55, 79), dir: Direction::SouthEast, init: init_healer,     talk: talk_to_healer },
    TownerData { type_: TOWN_DEADGUY, position: Point::new(24, 32), dir: Direction::North,     init: init_town_dead,  talk: talk_to_deadguy },
    TownerData { type_: TOWN_TAVERN,  position: Point::new(55, 62), dir: Direction::SouthWest, init: init_bar_owner,  talk: talk_to_bar_owner },
    TownerData { type_: TOWN_STORY,   position: Point::new(62, 71), dir: Direction::South,     init: init_teller,     talk: talk_to_storyteller },
    TownerData { type_: TOWN_DRUNK,   position: Point::new(71, 84), dir: Direction::South,     init: init_drunk,      talk: talk_to_drunk },
    TownerData { type_: TOWN_WITCH,   position: Point::new(80, 20), dir: Direction::South,     init: init_witch,      talk: talk_to_witch },
    TownerData { type_: TOWN_BMAID,   position: Point::new(43, 66), dir: Direction::South,     init: init_barmaid,    talk: talk_to_barmaid },
    TownerData { type_: TOWN_PEGBOY,  position: Point::new(11, 53), dir: Direction::South,     init: init_boy,        talk: talk_to_boy },
    TownerData { type_: TOWN_COW,     position: Point::new(58, 16), dir: Direction::SouthWest, init: init_cows,       talk: talk_to_cow },
    TownerData { type_: TOWN_COW,     position: Point::new(56, 14), dir: Direction::NorthWest, init: init_cows,       talk: talk_to_cow },
    TownerData { type_: TOWN_COW,     position: Point::new(59, 20), dir: Direction::North,     init: init_cows,       talk: talk_to_cow },
    TownerData { type_: TOWN_COWFARM, position: Point::new(61, 22), dir: Direction::SouthWest, init: init_cow_farmer, talk: talk_to_cow_farmer },
    TownerData { type_: TOWN_FARMER,  position: Point::new(62, 16), dir: Direction::South,     init: init_farmer,     talk: talk_to_farmer },
    TownerData { type_: TOWN_GIRL,    position: Point::new(77, 43), dir: Direction::South,     init: init_girl,       talk: talk_to_girl },
];

/// `QuestDialogTable` (initial contents): quest gossip for each towner ID.
#[rustfmt::skip]
const QUEST_DIALOG_TABLE: [[_speech_id; MAXQUESTS]; NUM_TOWNER_TYPES] = [
    /*TOWN_SMITH*/   [TEXT_INFRA6, TEXT_MUSH6, TEXT_NONE, TEXT_NONE, TEXT_VEIL5, TEXT_NONE, TEXT_BUTCH5, TEXT_BANNER6, TEXT_BLIND5, TEXT_BLOOD5, TEXT_ANVIL6, TEXT_WARLRD5, TEXT_KING7, TEXT_POISON7, TEXT_BONE5, TEXT_VILE9, TEXT_GRAVE2, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE],
    /*TOWN_HEALER*/  [TEXT_INFRA3, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_VEIL3, TEXT_NONE, TEXT_BUTCH3, TEXT_BANNER4, TEXT_BLIND3, TEXT_BLOOD3, TEXT_ANVIL3, TEXT_WARLRD3, TEXT_KING5, TEXT_POISON4, TEXT_BONE3, TEXT_VILE7, TEXT_GRAVE3, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE],
    /*TOWN_DEADGUY*/ [TEXT_NONE; MAXQUESTS],
    /*TOWN_TAVERN*/  [TEXT_INFRA2, TEXT_MUSH2, TEXT_NONE, TEXT_NONE, TEXT_VEIL2, TEXT_NONE, TEXT_BUTCH2, TEXT_NONE, TEXT_BLIND2, TEXT_BLOOD2, TEXT_ANVIL2, TEXT_WARLRD2, TEXT_KING3, TEXT_POISON2, TEXT_BONE2, TEXT_VILE4, TEXT_GRAVE5, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE],
    /*TOWN_STORY*/   [TEXT_INFRA1, TEXT_MUSH1, TEXT_NONE, TEXT_NONE, TEXT_VEIL1, TEXT_VILE3, TEXT_BUTCH1, TEXT_BANNER1, TEXT_BLIND1, TEXT_BLOOD1, TEXT_ANVIL1, TEXT_WARLRD1, TEXT_KING1, TEXT_POISON1, TEXT_BONE1, TEXT_VILE2, TEXT_GRAVE6, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE],
    /*TOWN_DRUNK*/   [TEXT_INFRA8, TEXT_MUSH7, TEXT_NONE, TEXT_NONE, TEXT_VEIL6, TEXT_NONE, TEXT_BUTCH6, TEXT_BANNER7, TEXT_BLIND6, TEXT_BLOOD6, TEXT_ANVIL8, TEXT_WARLRD6, TEXT_KING8, TEXT_POISON8, TEXT_BONE6, TEXT_VILE10, TEXT_GRAVE7, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE],
    /*TOWN_WITCH*/   [TEXT_INFRA9, TEXT_MUSH9, TEXT_NONE, TEXT_NONE, TEXT_VEIL7, TEXT_NONE, TEXT_BUTCH7, TEXT_BANNER8, TEXT_BLIND7, TEXT_BLOOD7, TEXT_ANVIL9, TEXT_WARLRD7, TEXT_KING9, TEXT_POISON9, TEXT_BONE7, TEXT_VILE11, TEXT_GRAVE1, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE],
    /*TOWN_BMAID*/   [TEXT_INFRA4, TEXT_MUSH5, TEXT_NONE, TEXT_NONE, TEXT_VEIL4, TEXT_NONE, TEXT_BUTCH4, TEXT_BANNER5, TEXT_BLIND4, TEXT_BLOOD4, TEXT_ANVIL4, TEXT_WARLRD4, TEXT_KING6, TEXT_POISON6, TEXT_BONE4, TEXT_VILE8, TEXT_GRAVE8, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE],
    /*TOWN_PEGBOY*/  [TEXT_INFRA10, TEXT_MUSH13, TEXT_NONE, TEXT_NONE, TEXT_VEIL8, TEXT_NONE, TEXT_BUTCH8, TEXT_BANNER9, TEXT_BLIND8, TEXT_BLOOD8, TEXT_ANVIL10, TEXT_WARLRD8, TEXT_KING10, TEXT_POISON10, TEXT_BONE8, TEXT_VILE12, TEXT_GRAVE9, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE, TEXT_NONE],
    /*TOWN_COW*/     [TEXT_NONE; MAXQUESTS],
    /*TOWN_FARMER*/  [TEXT_NONE; MAXQUESTS],
    /*TOWN_GIRL*/    [TEXT_NONE; MAXQUESTS],
    /*TOWN_COWFARM*/ [TEXT_NONE; MAXQUESTS],
];

/// Original: `IsTownerPresent` (towners.cpp).
// @port towners.cpp|devilution::IsTownerPresent(_talker_id npc) sha=b602501242ec
fn is_towner_present(ctx: &Ctx, npc: _talker_id) -> bool {
    let qa = |id: quest_id| ctx.quests.Quests[id as usize]._qactive;
    let hf = ctx.init.gb_is_hellfire;
    let gi = &ctx.multi.sgGameInitInfo;
    match npc {
        TOWN_DEADGUY => qa(Q_BUTCHER) != QUEST_NOTAVAIL && qa(Q_BUTCHER) != QUEST_DONE,
        TOWN_FARMER => hf && gi.bCowQuest == 0 && qa(Q_FARMER) != QUEST_HIVE_DONE,
        TOWN_COWFARM => hf && gi.bCowQuest != 0,
        TOWN_GIRL => {
            let me = ctx.players.MyPlayer.expect("MyPlayer");
            hf && gi.bTheoQuest != 0 && ctx.players.Players[me]._pLvlVisited[17] && qa(Q_GIRL) != QUEST_DONE
        }
        _ => true,
    }
}

/// Original: `devilution::GetTowner` (towners.cpp). Returns a `Towners` index.
// @port towners.cpp|devilution::GetTowner(_talker_id type) sha=20063c956ef1
pub fn get_towner(ctx: &Ctx, type_: _talker_id) -> Option<usize> {
    ctx.towners.towners.iter().position(|t| t._ttype == type_)
}

/// Original: `devilution::InitTowners` (towners.cpp).
// @port towners.cpp|devilution::InitTowners() sha=4bffb86bc439
pub fn init_towners(ctx: &mut Ctx) {
    assert!(ctx.towners.cow_sprites.is_none());
    ctx.towners.cow_sprites = Some(crate::engine::load_sprites::load_cel_sheet(ctx, "towners\\animals\\cow", 128));
    let mut i = 0;
    for towner_data in TOWNERS_DATA.iter() {
        if !is_towner_present(ctx, towner_data.type_) {
            continue;
        }
        init_towner_info(ctx, i, towner_data);
        i += 1;
    }
}

/// Original: `devilution::FreeTownerGFX` (towners.cpp).
// @port towners.cpp|devilution::FreeTownerGFX() sha=9849e64e50c3
pub fn free_towner_gfx(ctx: &mut Ctx) {
    for towner in ctx.towners.towners.iter_mut() {
        towner.owned_anim = None;
    }
    ctx.towners.cow_sprites = None;
}

/// Original: `devilution::ProcessTowners` (towners.cpp).
// @port towners.cpp|devilution::ProcessTowners() sha=aaac02fbb9ab
pub fn process_towners(ctx: &mut Ctx) {
    // BUGFIX: should be `i < numtowners`, was `i < NUM_TOWNERS`
    for t in 0..NUM_TOWNERS {
        if ctx.towners.towners[t]._ttype == TOWN_DEADGUY {
            town_dead(ctx, t);
        }
        let towner = &mut ctx.towners.towners[t];
        towner._tAnimCnt += 1;
        if towner._tAnimCnt < towner._tAnimDelay {
            continue;
        }
        towner._tAnimCnt = 0;
        if towner.anim_order_size > 0 {
            towner._tAnimFrameCnt = towner._tAnimFrameCnt.wrapping_add(1);
            if towner._tAnimFrameCnt > towner.anim_order_size - 1 {
                towner._tAnimFrameCnt = 0;
            }
            towner._tAnimFrame = towner.anim_order[towner._tAnimFrameCnt as usize];
            continue;
        }
        towner._tAnimFrame = towner._tAnimFrame.wrapping_add(1);
        if towner._tAnimFrame >= towner._tAnimLen {
            towner._tAnimFrame = 0;
        }
    }
}

/// Original: `devilution::TalkToTowner` (towners.cpp).
// @port towners.cpp|devilution::TalkToTowner(Player &player, int t) sha=02b33717bd15
pub fn talk_to_towner(ctx: &mut Ctx, pnum: usize, t: i32) {
    let t = t as usize;
    let towner_pos = ctx.towners.towners[t].position;
    let player = &ctx.players.Players[pnum];
    if player.position.tile.walking_distance(towner_pos) >= 2 {
        return;
    }
    if !player.HoldItem.is_empty() {
        return;
    }
    if let Some(talk) = ctx.towners.towners[t].talk {
        talk(ctx, pnum, t);
    }
}

/// Original: `devilution::UpdateGirlAnimAfterQuestComplete` (towners.cpp).
// @port towners.cpp|devilution::UpdateGirlAnimAfterQuestComplete() sha=5b816c34436d
pub fn update_girl_anim_after_quest_complete(ctx: &mut Ctx) {
    let Some(girl) = get_towner(ctx, TOWN_GIRL) else { return };
    if ctx.towners.towners[girl].owned_anim.is_none() {
        return;
    }
    let cur_frame = ctx.towners.towners[girl]._tAnimFrame;
    load_towner_animations(ctx, girl, "towners\\girl\\girls1", 20, 6);
    let g = &mut ctx.towners.towners[girl];
    g._tAnimFrame = cur_frame.min(g._tAnimLen.wrapping_sub(1));
}

/// Original: `devilution::UpdateCowFarmerAnimAfterQuestComplete` (towners.cpp).
// @port towners.cpp|devilution::UpdateCowFarmerAnimAfterQuestComplete() sha=8ac6c626c1db
pub fn update_cow_farmer_anim_after_quest_complete(ctx: &mut Ctx) {
    let cow_farmer = get_towner(ctx, TOWN_COWFARM).expect("cow farmer");
    let cur_frame = ctx.towners.towners[cow_farmer]._tAnimFrame;
    load_towner_animations(ctx, cow_farmer, "towners\\farmer\\mfrmrn2", 15, 3);
    let c = &mut ctx.towners.towners[cow_farmer];
    c._tAnimFrame = cur_frame.min(c._tAnimLen.wrapping_sub(1));
}
