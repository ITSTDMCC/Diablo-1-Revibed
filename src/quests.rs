//! `Source/quests.cpp`: quest state (the logic is pending).

use crate::ctx::Ctx;
use crate::engine::geometry::Point;
use crate::enums::*;
use crate::levels::gendung::DungeonType;

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

/// Globals of quests.cpp.
pub struct QuestsState {
    pub QuestLogIsOpen: bool,
    pub Quests: [Quest; MAXQUESTS],
    pub ReturnLvlPosition: Point,
    pub ReturnLevelType: DungeonType,
    pub ReturnLevel: i32,
}

impl Default for QuestsState {
    fn default() -> Self {
        QuestsState { QuestLogIsOpen: false, Quests: [Quest::default(); MAXQUESTS], ReturnLvlPosition: Point::default(), ReturnLevelType: DungeonType::Town, ReturnLevel: 0 }
    }
}

crate::pending_fn!(pub fn is_quest_available(ctx: &Ctx, q: quest_id) -> bool, "quests.cpp|devilution::Quest::IsAvailable()");
crate::pending_fn!(pub fn use_multiplayer_quests(ctx: &Ctx) -> bool, "quests.cpp|devilution::UseMultiplayerQuests()");
crate::pending_fn!(pub fn opens_hive(ctx: &Ctx, position: Point) -> bool, "items.cpp|devilution::OpensHive(Point position)");
crate::pending_fn!(pub fn opens_grave(ctx: &Ctx, position: Point) -> bool, "items.cpp|devilution::OpensGrave(Point position)");
crate::pending_fn!(pub fn init_quests(ctx: &mut Ctx), "quests.cpp|devilution::InitQuests()");

crate::pending_fn!(pub fn resync_quests(ctx: &mut Ctx), "quests.cpp|devilution::ResyncQuests()");
