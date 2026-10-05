//! `Source/levels/setmaps` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn load_set_map(ctx: &mut crate::ctx::Ctx), "levels/setmaps.cpp|devilution::LoadSetMap()");

/// `QuestLevelNames`: maps from quest level to quest level names (untranslated keys).
pub const QUEST_LEVEL_NAMES: [&str; 9] = [
    "",
    "Skeleton King's Lair",
    "Chamber of Bone",
    "Maze",
    "Poisoned Water Supply",
    "Archbishop Lazarus' Lair",
    "Church Arena",
    "Hell Arena",
    "Circle of Life Arena",
];
