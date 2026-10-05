//! `Source/levels/setmaps` (DevilutionX 1.5.3): loading the fixed quest ("set") levels.

use crate::ctx::Ctx;
use crate::engine::geometry::{Point, Rectangle, Size};
use crate::enums::*;
use crate::levels::gendung::DungeonType;

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

fn init_loaded(ctx: &mut Ctx, at: (i32, i32), range: ((i32, i32), (i32, i32)), lever_id: i32) {
    let oi = crate::objects::object_at_position(ctx, Point::new(at.0, at.1));
    let rect = Rectangle::new(Point::new(range.0 .0, range.0 .1), Size::new(range.1 .0, range.1 .1));
    ctx.objects.Objects[oi].initialize_loaded_object(rect, lever_id);
}

/// Original: `AddSKingObjs` (levels/setmaps.cpp).
// @port levels/setmaps.cpp|devilution::AddSKingObjs() sha=478114886257
fn add_s_king_objs(ctx: &mut Ctx) {
    let small_secret_room = ((20, 7), (3, 3));
    init_loaded(ctx, (64, 34), small_secret_room, 1);

    let gate = ((20, 14), (1, 2));
    init_loaded(ctx, (64, 59), gate, 2);

    let large_secret_room = ((8, 1), (7, 10));
    init_loaded(ctx, (27, 37), large_secret_room, 3);
    init_loaded(ctx, (46, 35), large_secret_room, 3);
    init_loaded(ctx, (49, 53), large_secret_room, 3);
    init_loaded(ctx, (27, 53), large_secret_room, 3);
}

/// Original: `AddSChamObjs` (levels/setmaps.cpp).
// @port levels/setmaps.cpp|devilution::AddSChamObjs() sha=d2f9cf532abb
fn add_s_cham_objs(ctx: &mut Ctx) {
    init_loaded(ctx, (37, 30), ((17, 0), (4, 5)), 1);
    init_loaded(ctx, (37, 46), ((13, 0), (3, 5)), 2);
}

/// Original: `AddVileObjs` (levels/setmaps.cpp).
// @port levels/setmaps.cpp|devilution::AddVileObjs() sha=341e0bcd6e6d
fn add_vile_objs(ctx: &mut Ctx) {
    init_loaded(ctx, (26, 45), ((1, 1), (8, 9)), 1);
    init_loaded(ctx, (45, 46), ((11, 1), (9, 9)), 2);
    init_loaded(ctx, (35, 36), ((7, 11), (6, 7)), 3);
}

/// Original: `SetMapTransparency` (levels/setmaps.cpp).
// @port levels/setmaps.cpp|devilution::SetMapTransparency(const char *path) sha=dc049a9efb51
fn set_map_transparency(ctx: &mut Ctx, path: &str) {
    let dun_data = crate::levels::gendung::load_u16_file(ctx, path);
    crate::levels::gendung::load_transparency(ctx, &dun_data);
}

/// Original: `LoadCustomMap` (levels/setmaps.cpp).
// @port levels/setmaps.cpp|devilution::LoadCustomMap(const char *path, Point viewPosition) sha=7a9d78514767
fn load_custom_map(ctx: &mut Ctx, path: &str, view_position: Point) {
    let setlvltype = ctx.gendung.setlvltype;
    match setlvltype {
        DungeonType::Cathedral | DungeonType::Crypt => crate::levels::drlg_l1::load_l1_dungeon(ctx, path, view_position),
        DungeonType::Catacombs => crate::levels::drlg_l2::load_l2_dungeon(ctx, path, view_position),
        DungeonType::Caves | DungeonType::Nest => crate::levels::drlg_l3::load_l3_dungeon(ctx, path, view_position),
        DungeonType::Hell => crate::levels::drlg_l4::load_l4_dungeon(ctx, path, view_position),
        DungeonType::Town | DungeonType::None => {}
    }
    crate::engine::palette::load_rnd_lvl_pal(ctx, setlvltype);
}

/// Original: `LoadArenaMap` (levels/setmaps.cpp).
// @port levels/setmaps.cpp|devilution::LoadArenaMap(const char *path, Point viewPosition, Point exitTrigger) sha=9c70b9855b8f
fn load_arena_map(ctx: &mut Ctx, path: &str, view_position: Point, exit_trigger: Point) {
    load_custom_map(ctx, path, view_position);
    ctx.trigs.trigflag = false;
    ctx.trigs.numtrigs = 1;
    ctx.trigs.trigs[0].position = exit_trigger;
    ctx.trigs.trigs[0]._tmsg = WM_DIABRTNLVL;
}

/// Original: `devilution::LoadSetMap` (levels/setmaps.cpp).
// @port levels/setmaps.cpp|devilution::LoadSetMap() sha=ecb3aab4113b
pub fn load_set_map(ctx: &mut Ctx) {
    use crate::levels::{drlg_l1, drlg_l2, drlg_l3, trigs};
    let pal = |ctx: &mut Ctx, p: &str| crate::engine::palette::load_palette(ctx, p, true);
    match ctx.gendung.setlvlnum {
        SL_SKELKING => {
            if ctx.quests.Quests[Q_SKELKING as usize]._qactive == QUEST_INIT {
                ctx.quests.Quests[Q_SKELKING as usize]._qactive = QUEST_ACTIVE;
                ctx.quests.Quests[Q_SKELKING as usize]._qvar1 = 1;
                crate::msg::net_send_cmd_quest(ctx, true, Q_SKELKING as usize);
            }
            drlg_l1::load_pre_l1_dungeon(ctx, "levels\\l1data\\sklkng1.dun");
            drlg_l1::load_l1_dungeon(ctx, "levels\\l1data\\sklkng2.dun", Point::new(83, 44));
            set_map_transparency(ctx, "levels\\l1data\\sklkngt.dun");
            pal(ctx, "levels\\l1data\\l1_2.pal");
            add_s_king_objs(ctx);
            trigs::init_s_king_triggers(ctx);
        }
        SL_BONECHAMB => {
            drlg_l2::load_pre_l2_dungeon(ctx, "levels\\l2data\\bonecha2.dun");
            drlg_l2::load_l2_dungeon(ctx, "levels\\l2data\\bonecha1.dun", Point::new(70, 40));
            set_map_transparency(ctx, "levels\\l2data\\bonechat.dun");
            pal(ctx, "levels\\l2data\\l2_2.pal");
            add_s_cham_objs(ctx);
            trigs::init_s_chamb_triggers(ctx);
        }
        SL_MAZE => {}
        SL_POISONWATER => {
            if ctx.quests.Quests[Q_PWATER as usize]._qactive == QUEST_INIT {
                ctx.quests.Quests[Q_PWATER as usize]._qactive = QUEST_ACTIVE;
            }
            drlg_l3::load_l3_dungeon(ctx, "levels\\l3data\\foulwatr.dun", Point::new(31, 83));
            pal(ctx, "levels\\l3data\\l3pfoul.pal");
            trigs::init_p_water_triggers(ctx);
        }
        SL_VILEBETRAYER => {
            let q = &mut ctx.quests.Quests[Q_BETRAYER as usize];
            if q._qactive == QUEST_DONE {
                q._qvar2 = 4;
            } else if q._qactive == QUEST_ACTIVE {
                q._qvar2 = 3;
            }
            drlg_l1::load_pre_l1_dungeon(ctx, "levels\\l1data\\vile1.dun");
            drlg_l1::load_l1_dungeon(ctx, "levels\\l1data\\vile2.dun", Point::new(35, 36));
            set_map_transparency(ctx, "levels\\l1data\\vile1.dun");
            pal(ctx, "levels\\l1data\\l1_2.pal");
            add_vile_objs(ctx);
            trigs::init_no_triggers(ctx);
        }
        SL_ARENA_CHURCH => load_arena_map(ctx, "arena\\church.dun", Point::new(29, 22), Point::new(28, 20)),
        SL_ARENA_HELL => load_arena_map(ctx, "arena\\hell.dun", Point::new(34, 26), Point::new(33, 26)),
        SL_ARENA_CIRCLE_OF_LIFE => load_arena_map(ctx, "arena\\circle_of_death.dun", Point::new(30, 26), Point::new(29, 26)),
        // SL_NONE: the debug-build test map is not part of the release game.
        _ => {}
    }
}
