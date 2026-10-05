//! `Source/monster.cpp`: monsters.

use std::rc::Rc;

use crate::ctx::Ctx;
use crate::engine::actor_position::ActorPosition;
use crate::engine::animationinfo::AnimationInfo;
use crate::engine::clx_sprite::{ClxSpriteList, ClxSpriteListOrSheet};
use crate::engine::geometry::{Direction, Point};
use crate::utils::language::tr;
use crate::engine::sound::TSnd;
use crate::enums::*;
use crate::tables::monstdat::*;

pub const MaxMonsters: usize = 200;
pub const MaxLvlMTypes: usize = 24;

/// `AnimStruct`
#[derive(Clone, Debug, Default)]
pub struct AnimStruct {
    pub sprites: Option<ClxSpriteListOrSheet>,
    pub width: u16,
    pub frames: i8,
    pub rate: i8,
}

impl AnimStruct {
    /// `spritesForDirection`
    // @port monster.h|devilution::AnimStruct::spritesForDirection(Direction direction) sha=04be3b93f61e
    pub fn sprites_for_direction(&self, direction: Direction) -> Option<ClxSpriteList> {
        let s = self.sprites.as_ref()?;
        Some(if s.is_sheet() { s.sheet().get(direction as usize) } else { s.list().clone() })
    }
}

/// `CMonster`
#[derive(Default)]
pub struct CMonster {
    /// `animData`
    pub anim_data: Option<Rc<[u8]>>,
    pub anims: [AnimStruct; 6],
    pub sounds: [[Option<Box<TSnd>>; 2]; 4],
    /// `data`: index into `MonstersData`
    pub data: usize,
    pub type_: _monster_id,
    pub placeFlags: u8,
    pub corpseId: i8,
}

/// `Monster`
#[derive(Clone, Debug, Default)]
pub struct Monster {
    pub uniqueMonsterTRN: Option<Rc<[u8; 256]>>,
    pub animInfo: AnimationInfo,
    pub maxHitPoints: i32,
    pub hitPoints: i32,
    pub flags: u32,
    pub rndItemSeed: u32,
    pub aiSeed: u32,
    pub toHit: u16,
    pub resistance: u16,
    pub talkMsg: _speech_id,
    pub goalVar1: i16,
    pub goalVar2: i8,
    pub goalVar3: i8,
    pub var1: i16,
    pub var2: i16,
    pub var3: i8,
    pub position: ActorPosition,
    pub goal: MonsterGoal,
    pub enemyPosition: Point,
    pub levelType: u8,
    pub mode: MonsterMode,
    pub pathCount: u8,
    pub direction: Direction,
    pub enemy: u8,
    pub isInvalid: bool,
    pub ai: MonsterAIID,
    pub intelligence: u8,
    pub activeForTicks: u8,
    pub uniqueType: UniqueMonsterType,
    pub uniqTrans: u8,
    pub corpseId: i8,
    pub whoHit: i8,
    pub minDamage: u8,
    pub maxDamage: u8,
    pub minDamageSpecial: u8,
    pub maxDamageSpecial: u8,
    pub armorClass: u8,
    pub leader: u8,
    pub leaderRelation: LeaderRelation,
    pub packSize: u8,
    pub lightId: i8,
}

impl Monster {
    pub const NoLeader: u8 = 0xff;

    /// `isUnique`
    // @port monster.h|devilution::Monster::isUnique() sha=c0ec6629b59e
    pub fn is_unique(&self) -> bool {
        self.uniqueType != UniqueMonsterType::None
    }

    /// Original: `Monster::isPlayerMinion` (monster.cpp).
    // @port monster.cpp|devilution::Monster::isPlayerMinion() sha=b5440c18f430
    pub fn is_player_minion(&self) -> bool {
        (self.flags & MFLAG_GOLEM as u32) != 0 && (self.flags & MFLAG_BERSERK as u32) == 0
    }
}

/// Original: `IsMonsterModeMove` (monster.h).
// @port monster.h|devilution::IsMonsterModeMove(MonsterMode mode) sha=51749a2b4053
pub fn is_monster_mode_move(mode: MonsterMode) -> bool {
    matches!(mode, MonsterMode::MoveNorthwards | MonsterMode::MoveSouthwards | MonsterMode::MoveSideways)
}

/// Globals of monster.cpp.
pub struct MonsterState {
    pub LevelMonsterTypes: Vec<CMonster>,
    pub LevelMonsterTypeCount: usize,
    pub Monsters: Vec<Monster>,
    pub ActiveMonsters: [i32; MaxMonsters],
    pub ActiveMonsterCount: usize,
    pub MonsterKillCounts: Vec<i32>,
    /// `totalmonsters`
    pub totalmonsters: usize,
    /// `monstimgtot`
    pub monstimgtot: i32,
    /// `uniquetrans`
    pub uniquetrans: i32,
    /// `sgbSaveSoundOn`
    pub sgbSaveSoundOn: bool,
}

impl Default for MonsterState {
    fn default() -> Self {
        MonsterState {
            LevelMonsterTypes: (0..MaxLvlMTypes).map(|_| CMonster::default()).collect(),
            LevelMonsterTypeCount: 0,
            Monsters: vec![Monster::default(); MaxMonsters],
            ActiveMonsters: [0; MaxMonsters],
            ActiveMonsterCount: 0,
            MonsterKillCounts: vec![0; MonstersData.len()],
            totalmonsters: 0,
            monstimgtot: 0,
            uniquetrans: 0,
            sgbSaveSoundOn: false,
        }
    }
}

/// `Monster::type()`
// @port monster.h|devilution::Monster::type() sha=b49ad95ec04f
pub fn monster_type(ctx: &Ctx, m: usize) -> &CMonster {
    &ctx.monster.LevelMonsterTypes[ctx.monster.Monsters[m].levelType as usize]
}

/// `Monster::type().type`
pub fn monster_type_id(ctx: &Ctx, m: usize) -> _monster_id {
    monster_type(ctx, m).type_
}

/// `Monster::type().corpseId`
pub fn monster_corpse_id(ctx: &Ctx, m: usize) -> i8 {
    monster_type(ctx, m).corpseId
}

/// `Monster::data()`
// @port monster.h|devilution::Monster::data() sha=9e72b01d23f6
pub fn monster_data(ctx: &Ctx, m: usize) -> &'static MonsterData {
    &MonstersData[monster_type(ctx, m).data]
}

/// `Monster::level(difficulty)`
// @port monster.h|devilution::Monster::level(_difficulty difficulty) sha=a6bd9e214f21
pub fn monster_level(ctx: &Ctx, m: usize, difficulty: _difficulty) -> u32 {
    let mon = &ctx.monster.Monsters[m];
    let mut base_level = monster_data(ctx, m).level as u32;
    if mon.is_unique() {
        base_level = UniqueMonstersData[mon.uniqueType.0 as usize].mlevel as u32;
        if base_level != 0 {
            base_level *= 2;
        } else {
            base_level = monster_data(ctx, m).level as u32 + 5;
        }
    }
    if monster_type_id(ctx, m) == MT_DIABLO && !ctx.init.gb_is_hellfire {
        base_level = base_level.wrapping_sub(15);
    }
    if difficulty == DIFF_NIGHTMARE {
        base_level += 15;
    } else if difficulty == DIFF_HELL {
        base_level += 30;
    }
    base_level
}

const NightmareToHitBonus: u32 = 85;
const HellToHitBonus: u32 = 120;

/// Original: `Monster::getVisualMonsterMode` (monster.cpp).
// @port monster.cpp|devilution::Monster::getVisualMonsterMode() sha=1d03c63af0d2
pub fn get_visual_monster_mode(ctx: &Ctx, m: usize) -> MonsterMode {
    let mode = ctx.monster.Monsters[m].mode;
    if mode != MonsterMode::Petrified {
        return mode;
    }
    for missile in ctx.missiles.Missiles.iter() {
        if missile._mitype == MissileID::StoneCurse && missile.var2 as usize == m {
            return MonsterMode::from_raw(missile.var1 as u8);
        }
    }
    MonsterMode::Petrified
}

/// Original: `Monster::isWalking` (monster.cpp).
// @port monster.cpp|devilution::Monster::isWalking() sha=d84d130aeb7a
pub fn is_walking(ctx: &Ctx, m: usize) -> bool {
    matches!(get_visual_monster_mode(ctx, m), MonsterMode::MoveNorthwards | MonsterMode::MoveSouthwards | MonsterMode::MoveSideways)
}

/// Original: `Monster::toHitSpecial` (monster.cpp).
// @port monster.cpp|devilution::Monster::toHitSpecial(_difficulty difficulty) sha=cb47955b3224
pub fn to_hit_special(ctx: &Ctx, m: usize, difficulty: _difficulty) -> u32 {
    let mon = &ctx.monster.Monsters[m];
    let mut base_to_hit_special = monster_data(ctx, m).toHitSpecial as u32;
    if mon.is_unique() && UniqueMonstersData[mon.uniqueType.0 as usize].customToHit != 0 {
        base_to_hit_special = UniqueMonstersData[mon.uniqueType.0 as usize].customToHit as u32;
    }
    if difficulty == DIFF_NIGHTMARE {
        base_to_hit_special += NightmareToHitBonus;
    } else if difficulty == DIFF_HELL {
        base_to_hit_special += HellToHitBonus;
    }
    base_to_hit_special
}

/// Original: `Monster::exp` (monster.h).
// @port monster.h|devilution::Monster::exp(_difficulty difficulty) sha=cdc47f00d80e
pub fn monster_exp(ctx: &Ctx, m: usize, difficulty: _difficulty) -> u32 {
    let mut monster_exp = monster_data(ctx, m).exp as u32;
    if difficulty == DIFF_NIGHTMARE {
        monster_exp = 2 * (monster_exp + 1000);
    } else if difficulty == DIFF_HELL {
        monster_exp = 4 * (monster_exp + 1000);
    }
    if ctx.monster.Monsters[m].is_unique() {
        monster_exp *= 2;
    }
    monster_exp
}

pub use crate::diablo_game::is_diablo_alive;

/// Original: `devilution::FreeMonsters` (monster.cpp).
// @port monster.cpp|devilution::FreeMonsters() sha=924991731d90
pub fn free_monsters(ctx: &mut Ctx) {
    for monster_type in ctx.monster.LevelMonsterTypes.iter_mut() {
        monster_type.anim_data = None;
        for anim_data in monster_type.anims.iter_mut() {
            anim_data.sprites = None;
        }
        for variants in monster_type.sounds.iter_mut() {
            for sound in variants.iter_mut() {
                *sound = None;
            }
        }
    }
}


/// `GolemHoldingCell`
pub const GOLEM_HOLDING_CELL: Point = Point::new(1, 0);









/// Original: `devilution::Monster::name` (monster.h).
// @port monster.h|devilution::Monster::name() sha=4eb516b96c3e
pub fn monster_name(ctx: &crate::ctx::Ctx, m: usize) -> String {
    let mon = &ctx.monster.Monsters[m];
    if mon.uniqueType != UniqueMonsterType::None {
        return crate::utils::language::pgettext("monster", UniqueMonstersData[mon.uniqueType.0 as usize].mName);
    }
    crate::utils::language::pgettext("monster", monster_data(ctx, m).name)
}


/// Original: `ClearMVars` (monster.cpp).
// @port monster.cpp|devilution::ClearMVars(Monster &monster) sha=1679d123a80f
fn clear_m_vars(monster: &mut Monster) {
    monster.var1 = 0;
    monster.var2 = 0;
    monster.var3 = 0;
    monster.position.temp = Point::new(0, 0);
}

/// Original: `ClrAllMonsters` (monster.cpp).
// @port monster.cpp|devilution::ClrAllMonsters() sha=d88c1e03491f
fn clr_all_monsters(ctx: &mut Ctx) {
    for i in 0..ctx.monster.Monsters.len() {
        let dir = Direction::from_u8(ctx.rng.generate_rnd(8) as u8);
        let enemy = ctx.rng.generate_rnd(ctx.multi.gbActivePlayers as i32) as u8;
        let enemy_position = ctx.players.Players[enemy as usize].position.future;
        let monster = &mut ctx.monster.Monsters[i];
        clear_m_vars(monster);
        monster.goal = MonsterGoal::None;
        monster.mode = MonsterMode::Stand;
        monster.var1 = 0;
        monster.var2 = 0;
        monster.position.tile = Point::new(0, 0);
        monster.position.future = Point::new(0, 0);
        monster.position.old = Point::new(0, 0);
        monster.direction = dir;
        monster.animInfo = AnimationInfo::default();
        monster.flags = 0;
        monster.isInvalid = false;
        monster.enemy = enemy;
        monster.enemyPosition = enemy_position;
    }
}

/// Original: `DeleteMonster` (monster.cpp).
// @port monster.cpp|devilution::DeleteMonster(size_t activeIndex) sha=d0d21ac18a7d
fn delete_monster(ctx: &mut Ctx, active_index: usize) {
    let m = ctx.monster.ActiveMonsters[active_index] as usize;
    if (ctx.monster.Monsters[m].flags & MFLAG_BERSERK as u32) != 0 {
        let light_id = ctx.monster.Monsters[m].lightId as i32;
        crate::lighting::add_un_light(ctx, light_id);
    }
    ctx.monster.ActiveMonsterCount -= 1;
    // This ensures alive monsters are before ActiveMonsterCount in the array and any deleted monster after
    let n = ctx.monster.ActiveMonsterCount;
    ctx.monster.ActiveMonsters.swap(active_index, n);
}

/// Original: `devilution::InitLevelMonsters` (monster.cpp).
// @port monster.cpp|devilution::InitLevelMonsters() sha=9b9680018f07
pub fn init_level_monsters(ctx: &mut Ctx) {
    ctx.monster.LevelMonsterTypeCount = 0;
    ctx.monster.monstimgtot = 0;
    for level_monster_type in ctx.monster.LevelMonsterTypes.iter_mut() {
        level_monster_type.placeFlags = 0;
    }
    clr_all_monsters(ctx);
    ctx.monster.ActiveMonsterCount = 0;
    ctx.monster.totalmonsters = MaxMonsters;
    for i in 0..MaxMonsters {
        ctx.monster.ActiveMonsters[i] = i as i32;
    }
    ctx.monster.uniquetrans = 0;
}

/// Original: `devilution::DeleteMonsterList` (monster.cpp).
// @port monster.cpp|devilution::DeleteMonsterList() sha=84e47223f498
pub fn delete_monster_list(ctx: &mut Ctx) {
    for i in 0..crate::player::MAX_PLRS {
        let golem = &mut ctx.monster.Monsters[i];
        if !golem.isInvalid {
            continue;
        }
        golem.position.tile = GOLEM_HOLDING_CELL;
        golem.position.future = Point::new(0, 0);
        golem.position.old = Point::new(0, 0);
        golem.isInvalid = false;
    }
    let mut i = crate::player::MAX_PLRS;
    while i < ctx.monster.ActiveMonsterCount {
        let m = ctx.monster.ActiveMonsters[i];
        if ctx.monster.Monsters[m as usize].isInvalid {
            if ctx.cursor.pcursmonst as i32 == m {
                // Unselect monster if player highlighted it
                ctx.cursor.pcursmonst = -1;
            }
            delete_monster(ctx, i);
        } else {
            i += 1;
        }
    }
}


/// Original: `devilution::ProcessMonsters` (monster.cpp).
// @port monster.cpp|devilution::ProcessMonsters() sha=a1d03653ab26
pub fn process_monsters(ctx: &mut Ctx) {
    delete_monster_list(ctx);
    assert!(ctx.monster.ActiveMonsterCount <= MaxMonsters);
    let difficulty = ctx.multi.sgGameInitInfo.nDifficulty;
    let mut i = 0;
    while i < ctx.monster.ActiveMonsterCount {
        let m = ctx.monster.ActiveMonsters[i] as usize;
        follow_the_leader(ctx, m);
        if ctx.init.gb_is_multiplayer {
            let seed = ctx.monster.Monsters[m].aiSeed;
            ctx.rng.set_rnd_seed(seed);
            ctx.monster.Monsters[m].aiSeed = ctx.rng.advance_rnd_seed() as u32;
        }
        let level = monster_level(ctx, m, difficulty) as i32;
        {
            let monster = &mut ctx.monster.Monsters[m];
            if monster.hitPoints < monster.maxHitPoints && monster.hitPoints >> 6 > 0 {
                if level > 1 {
                    monster.hitPoints += level / 2;
                } else {
                    monster.hitPoints += level;
                }
                // prevent going over max HP with part of a single regen tick
                monster.hitPoints = monster.hitPoints.min(monster.maxHitPoints);
            }
        }
        let tile = ctx.monster.Monsters[m].position.tile;
        if crate::levels::gendung::is_tile_visible(ctx, tile) && ctx.monster.Monsters[m].activeForTicks == 0 {
            let t = monster_type_id(ctx, m);
            if t == MT_CLEAVER {
                crate::effects::play_sfx(ctx, crate::effects::USFX_CLEAVER);
            }
            if t == MT_NAKRUL {
                if ctx.multi.sgGameInitInfo.bCowQuest != 0 {
                    crate::effects::play_sfx(ctx, crate::effects::USFX_NAKRUL6);
                } else if ctx.crypt.IsUberRoomOpened {
                    crate::effects::play_sfx(ctx, crate::effects::USFX_NAKRUL4);
                } else {
                    crate::effects::play_sfx(ctx, crate::effects::USFX_NAKRUL5);
                }
            }
            if t == MT_DEFILER {
                crate::effects::play_sfx(ctx, crate::effects::USFX_DEFILER8);
            }
            update_enemy(ctx, m);
        }
        if (ctx.monster.Monsters[m].flags & MFLAG_TARGETS_MONSTER as u32) != 0 {
            let enemy = ctx.monster.Monsters[m].enemy as usize;
            assert!(enemy < MaxMonsters);
            // BUGFIX: enemy target may be dead at time of access, thus reading garbage data from `Monsters[monster.enemy].position.future`.
            let f = ctx.monster.Monsters[enemy].position.future;
            let monster = &mut ctx.monster.Monsters[m];
            monster.position.last = f;
            monster.enemyPosition = monster.position.last;
        } else {
            let enemy = ctx.monster.Monsters[m].enemy as usize;
            assert!(enemy < crate::player::MAX_PLRS);
            let future = ctx.players.Players[enemy].position.future;
            let visible = crate::levels::gendung::is_tile_visible(ctx, tile);
            let is_diablo = monster_type_id(ctx, m) == MT_DIABLO;
            let monster = &mut ctx.monster.Monsters[m];
            monster.enemyPosition = future;
            if visible {
                monster.activeForTicks = u8::MAX;
                monster.position.last = future;
            } else if monster.activeForTicks != 0 && !is_diablo {
                monster.activeForTicks -= 1;
            }
        }
        loop {
            if (ctx.monster.Monsters[m].flags & MFLAG_SEARCH as u32) == 0 || !ai_plan_path(ctx, m) {
                ai_proc(ctx, m);
            }
            if !update_mode_stance(ctx, m) {
                break;
            }
            group_unity(ctx, m);
        }
        let monster = &mut ctx.monster.Monsters[m];
        if monster.mode != MonsterMode::Petrified && (monster.flags & MFLAG_ALLOW_SPECIAL as u32) == 0 {
            let lock = (monster.flags & MFLAG_LOCK_ANIMATION as u32) != 0;
            monster.animInfo.process_animation(lock);
        }
        i += 1;
    }
    delete_monster_list(ctx);
}

// ---------------------------------------------------------------------------------------------
// monster.cpp, part 1: placement, animation helpers, movement starts, attacks.
// ---------------------------------------------------------------------------------------------

use crate::engine::geometry::{left, opposite, right, Displacement};
use crate::levels::gendung::{in_dungeon_bounds, is_tile_visible, DungeonType};
use crate::lighting::NO_LIGHT;

const NightmareAcBonus: i32 = 50;
const HellAcBonus: i32 = 80;

/// `SkeletonTypes`
const SKELETON_TYPES: [_monster_id; 12] = [MT_WSKELAX, MT_TSKELAX, MT_RSKELAX, MT_XSKELAX, MT_WSKELBW, MT_TSKELBW, MT_RSKELBW, MT_XSKELBW, MT_WSKELSD, MT_TSKELSD, MT_RSKELSD, MT_XSKELSD];

/// `Animletter`: maps from monster action to monster animation letter.
const ANIMLETTER: [u8; 6] = *b"nwahds";

fn rnd(ctx: &mut Ctx, v: i32) -> i32 {
    ctx.rng.generate_rnd(v)
}

/// `CMonster::getAnimData`
// @port monster.h|devilution::CMonster::getAnimData(MonsterGraphic graphic) sha=a2d364e455b7
pub fn get_anim_data(ctx: &Ctx, m: usize, graphic: MonsterGraphic) -> &AnimStruct {
    &monster_type(ctx, m).anims[graphic as usize]
}

/// Original: `Monster::changeAnimationData(MonsterGraphic, Direction)` (monster.h).
// @port monster.h|devilution::Monster::changeAnimationData(MonsterGraphic graphic, Direction desiredDirection) sha=99df4863abbf
pub fn change_animation_data_dir(ctx: &mut Ctx, m: usize, graphic: MonsterGraphic, desired_direction: Direction) {
    let a = get_anim_data(ctx, m, graphic);
    let (sprites, frames, rate) = (a.sprites_for_direction(desired_direction), a.frames, a.rate);
    ctx.monster.Monsters[m].animInfo.change_animation_data(sprites, frames, rate);
}

/// Original: `Monster::changeAnimationData(MonsterGraphic)` (monster.h).
// @port monster.h|devilution::Monster::changeAnimationData(MonsterGraphic graphic) sha=c9957ecdb019
pub fn change_animation_data(ctx: &mut Ctx, m: usize, graphic: MonsterGraphic) {
    let d = ctx.monster.Monsters[m].direction;
    change_animation_data_dir(ctx, m, graphic, d);
}

/// Original: `GetNumAnims` (monster.cpp).
// @port monster.cpp|devilution::GetNumAnims(const MonsterData &monsterData) sha=d53071f06e90
fn get_num_anims(monster_data: &MonsterData) -> usize {
    if monster_data.hasSpecial {
        6
    } else {
        5
    }
}

/// Original: `InitMonsterTRN` (monster.cpp).
// @port monster.cpp|devilution::InitMonsterTRN(CMonster &monst) sha=603bb740fe65
fn init_monster_trn(ctx: &mut Ctx, type_index: usize) {
    let data = &MonstersData[ctx.monster.LevelMonsterTypes[type_index].data];
    let path = format!("monsters\\{}.trn", data.trnFile.unwrap_or(""));
    let mut color_translations = [0u8; 256];
    crate::engine::load_file::load_file_in_mem_exact(ctx, &path, &mut color_translations);
    for c in color_translations.iter_mut() {
        if *c == 255 {
            *c = 0;
        }
    }
    let num_anims = get_num_anims(data);
    let mtype = ctx.monster.LevelMonsterTypes[type_index].type_;
    for i in 0..num_anims {
        if i == 1 && matches!(mtype, MT_COUNSLR | MT_MAGISTR | MT_CABALIST | MT_ADVOCATE) {
            continue;
        }
        let anim = &mut ctx.monster.LevelMonsterTypes[type_index].anims[i];
        if let Some(sprites) = anim.sprites.as_mut() {
            match sprites {
                ClxSpriteListOrSheet::Sheet(sheet) => crate::engine::render::clx_render::clx_apply_trans_sheet(sheet, &color_translations),
                ClxSpriteListOrSheet::List(list) => crate::engine::render::clx_render::clx_apply_trans_list(list, &color_translations),
            }
        }
    }
}

/// Original: `InitMonster` (monster.cpp).
// @port monster.cpp|devilution::InitMonster(Monster &monster, Direction rd, size_t typeIndex, Point position) sha=ef7aa1795a00
fn init_monster(ctx: &mut Ctx, m: usize, rd: Direction, type_index: usize, position: Point) {
    {
        let monster = &mut ctx.monster.Monsters[m];
        monster.direction = rd;
        monster.position.tile = position;
        monster.position.future = position;
        monster.position.old = position;
        monster.levelType = type_index as u8;
        monster.mode = MonsterMode::Stand;
        monster.animInfo = AnimationInfo::default();
    }
    change_animation_data(ctx, m, MonsterGraphic::Stand);
    let tpf = ctx.monster.Monsters[m].animInfo.ticksPerFrame as i32;
    ctx.monster.Monsters[m].animInfo.tickCounterOfCurrentFrame = rnd(ctx, tpf - 1) as i8;
    let nof = ctx.monster.Monsters[m].animInfo.numberOfFrames as i32;
    ctx.monster.Monsters[m].animInfo.currentFrame = rnd(ctx, nof - 1) as i8;

    let data = monster_data(ctx, m);
    let mut maxhp = data.hitPointsMinimum as i32 + rnd(ctx, data.hitPointsMaximum as i32 - data.hitPointsMinimum as i32 + 1);
    if monster_type_id(ctx, m) == MT_DIABLO && !ctx.init.gb_is_hellfire {
        maxhp /= 2;
    }
    let mut max_hit_points = maxhp << 6;
    if !ctx.init.gb_is_multiplayer {
        max_hit_points = (max_hit_points / 2).max(64);
    }
    let rnd_item_seed = ctx.rng.advance_rnd_seed() as u32;
    let ai_seed = ctx.rng.advance_rnd_seed() as u32;
    {
        let monster = &mut ctx.monster.Monsters[m];
        monster.maxHitPoints = max_hit_points;
        monster.hitPoints = monster.maxHitPoints;
        monster.ai = data.ai;
        monster.intelligence = data.intelligence;
        monster.goal = MonsterGoal::Normal;
        monster.goalVar1 = 0;
        monster.goalVar2 = 0;
        monster.goalVar3 = 0;
        monster.pathCount = 0;
        monster.isInvalid = false;
        monster.uniqueType = UniqueMonsterType::None;
        monster.activeForTicks = 0;
        monster.lightId = NO_LIGHT as i8;
        monster.rndItemSeed = rnd_item_seed;
        monster.aiSeed = ai_seed;
        monster.whoHit = 0;
        monster.toHit = data.toHit as u16;
        monster.minDamage = data.minDamage;
        monster.maxDamage = data.maxDamage;
        monster.minDamageSpecial = data.minDamageSpecial;
        monster.maxDamageSpecial = data.maxDamageSpecial;
        monster.armorClass = data.armorClass;
        monster.resistance = data.resistance as u16;
        monster.leader = Monster::NoLeader;
        monster.leaderRelation = LeaderRelation::None;
        monster.flags = data.abilityFlags as u32;
        monster.talkMsg = TEXT_NONE;
    }
    if ctx.monster.Monsters[m].ai == MonsterAIID::Gargoyle {
        change_animation_data(ctx, m, MonsterGraphic::Special);
        let monster = &mut ctx.monster.Monsters[m];
        monster.animInfo.currentFrame = 0;
        monster.flags |= MFLAG_ALLOW_SPECIAL as u32;
        monster.mode = MonsterMode::SpecialMeleeAttack;
    }
    let (hf, mp) = (ctx.init.gb_is_hellfire, ctx.init.gb_is_multiplayer);
    let difficulty = ctx.multi.sgGameInitInfo.nDifficulty;
    let monster = &mut ctx.monster.Monsters[m];
    if difficulty == DIFF_NIGHTMARE {
        monster.maxHitPoints *= 3;
        if hf {
            monster.maxHitPoints += (if mp { 100 } else { 50 }) << 6;
        } else {
            monster.maxHitPoints += 100 << 6;
        }
        monster.hitPoints = monster.maxHitPoints;
        monster.toHit += NightmareToHitBonus as u16;
        monster.minDamage = 2u8.wrapping_mul(monster.minDamage.wrapping_add(2));
        monster.maxDamage = 2u8.wrapping_mul(monster.maxDamage.wrapping_add(2));
        monster.minDamageSpecial = 2u8.wrapping_mul(monster.minDamageSpecial.wrapping_add(2));
        monster.maxDamageSpecial = 2u8.wrapping_mul(monster.maxDamageSpecial.wrapping_add(2));
        monster.armorClass = monster.armorClass.wrapping_add(NightmareAcBonus as u8);
    } else if difficulty == DIFF_HELL {
        monster.maxHitPoints *= 4;
        if hf {
            monster.maxHitPoints += (if mp { 200 } else { 100 }) << 6;
        } else {
            monster.maxHitPoints += 200 << 6;
        }
        monster.hitPoints = monster.maxHitPoints;
        monster.toHit += HellToHitBonus as u16;
        monster.minDamage = 4u8.wrapping_mul(monster.minDamage).wrapping_add(6);
        monster.maxDamage = 4u8.wrapping_mul(monster.maxDamage).wrapping_add(6);
        monster.minDamageSpecial = 4u8.wrapping_mul(monster.minDamageSpecial).wrapping_add(6);
        monster.maxDamageSpecial = 4u8.wrapping_mul(monster.maxDamageSpecial).wrapping_add(6);
        monster.armorClass = monster.armorClass.wrapping_add(HellAcBonus as u8);
        monster.resistance = data.resistanceHell as u16;
    }
}

/// Original: `CanPlaceMonster` (monster.cpp).
// @port monster.cpp|devilution::CanPlaceMonster(Point position) sha=33d5318776e7
fn can_place_monster(ctx: &Ctx, position: Point) -> bool {
    in_dungeon_bounds(position)
        && ctx.gendung.dMonster[position.x as usize][position.y as usize] == 0
        && ctx.gendung.dPlayer[position.x as usize][position.y as usize] == 0
        && !is_tile_visible(ctx, position)
        && !crate::levels::gendung::tile_contains_set_piece(ctx, position)
        && !crate::engine::path::is_tile_occupied(ctx, position)
}

/// Original: `PlaceMonster` (monster.cpp).
// @port monster.cpp|devilution::PlaceMonster(int i, size_t typeIndex, Point position) sha=3b470661faee
fn place_monster(ctx: &mut Ctx, i: usize, type_index: usize, position: Point) {
    if ctx.monster.LevelMonsterTypes[type_index].type_ == MT_NAKRUL {
        for j in 0..ctx.monster.ActiveMonsterCount {
            if ctx.monster.Monsters[j].levelType as usize == type_index {
                return;
            }
        }
    }
    ctx.gendung.dMonster[position.x as usize][position.y as usize] = (i + 1) as i16;
    let rd = Direction::from_u8(rnd(ctx, 8) as u8);
    init_monster(ctx, i, rd, type_index, position);
}

/// Original: `PlaceGroup` (monster.cpp).
// @port monster.cpp|devilution::PlaceGroup(size_t typeIndex, unsigned num, Monster *leader = nullptr, bool leashed = false) sha=256c3ac00bd7
fn place_group(ctx: &mut Ctx, type_index: usize, mut num: u32, leader: Option<usize>, leashed: bool) {
    let mut placed: u32 = 0;
    for _try1 in 0..10 {
        while placed != 0 {
            ctx.monster.ActiveMonsterCount -= 1;
            placed -= 1;
            let position = ctx.monster.Monsters[ctx.monster.ActiveMonsterCount].position.tile;
            ctx.gendung.dMonster[position.x as usize][position.y as usize] = 0;
        }
        let mut xp;
        let mut yp;
        if let Some(l) = leader {
            let offset = rnd(ctx, 8);
            let position = ctx.monster.Monsters[l].position.tile + Direction::from_u8(offset as u8);
            xp = position.x;
            yp = position.y;
        } else {
            loop {
                xp = rnd(ctx, 80) + 16;
                yp = rnd(ctx, 80) + 16;
                if can_place_monster(ctx, Point::new(xp, yp)) {
                    break;
                }
            }
        }
        let x1 = xp;
        let y1 = yp;
        if num as usize + ctx.monster.ActiveMonsterCount > ctx.monster.totalmonsters {
            num = (ctx.monster.totalmonsters - ctx.monster.ActiveMonsterCount) as u32;
        }
        let mut j = 0;
        let mut try2 = 0;
        while j < num && try2 < 100 {
            let ok = can_place_monster(ctx, Point::new(xp, yp))
                && ctx.gendung.dTransVal[xp as usize][yp as usize] == ctx.gendung.dTransVal[x1 as usize][y1 as usize]
                && !(leashed && ((xp - x1).abs() >= 4 || (yp - y1).abs() >= 4));
            if !ok {
                try2 += 1;
            } else {
                let amc = ctx.monster.ActiveMonsterCount;
                place_monster(ctx, amc, type_index, Point::new(xp, yp));
                if let Some(l) = leader {
                    let intelligence = ctx.monster.Monsters[l].intelligence;
                    {
                        let minion = &mut ctx.monster.Monsters[amc];
                        minion.maxHitPoints *= 2;
                        minion.hitPoints = minion.maxHitPoints;
                        minion.intelligence = intelligence;
                    }
                    if leashed {
                        set_leader(ctx, amc, Some(l));
                    }
                    if ctx.monster.Monsters[amc].ai != MonsterAIID::Gargoyle {
                        change_animation_data(ctx, amc, MonsterGraphic::Stand);
                        let nof = ctx.monster.Monsters[amc].animInfo.numberOfFrames as i32;
                        let f = rnd(ctx, nof - 1) as i8;
                        let minion = &mut ctx.monster.Monsters[amc];
                        minion.animInfo.currentFrame = f;
                        minion.flags &= !(MFLAG_ALLOW_SPECIAL as u32);
                        minion.mode = MonsterMode::Stand;
                    }
                }
                ctx.monster.ActiveMonsterCount += 1;
                placed += 1;
                j += 1;
            }
            // BUGFIX: `yp += Point.y` (the original adds deltaX twice)
            xp += Displacement::from(Direction::from_u8(rnd(ctx, 8) as u8)).delta_x;
            yp += Displacement::from(Direction::from_u8(rnd(ctx, 8) as u8)).delta_x;
        }
        if placed >= num {
            break;
        }
    }
    if leashed {
        ctx.monster.Monsters[leader.expect("leader")].packSize = placed as u8;
    }
}

/// Original: `GetMonsterTypeIndex` (monster.cpp).
// @port monster.cpp|devilution::GetMonsterTypeIndex(_monster_id type) sha=295065ba3843
fn get_monster_type_index(ctx: &Ctx, type_: _monster_id) -> usize {
    for i in 0..ctx.monster.LevelMonsterTypeCount {
        if ctx.monster.LevelMonsterTypes[i].type_ == type_ {
            return i;
        }
    }
    ctx.monster.LevelMonsterTypeCount
}

/// Original: `PlaceUniqueMonst` (monster.cpp).
// @port monster.cpp|devilution::PlaceUniqueMonst(UniqueMonsterType uniqindex, size_t minionType, int bosspacksize) sha=30c3cfe0ee7b
fn place_unique_monst(ctx: &mut Ctx, uniqindex: UniqueMonsterType, minion_type: usize, bosspacksize: i32) {
    let m = ctx.monster.ActiveMonsterCount;
    let unique_monster_data = &UniqueMonstersData[uniqindex.0 as usize];
    let mut count = 0;
    let mut position;
    loop {
        let rx = rnd(ctx, 80);
        let ry = rnd(ctx, 80);
        position = Point::new(rx, ry) + Displacement::new(16, 16);
        let mut count2 = 0;
        for x in position.x - 3..position.x + 3 {
            for y in position.y - 3..position.y + 3 {
                if in_dungeon_bounds(Point::new(x, y)) && can_place_monster(ctx, Point::new(x, y)) {
                    count2 += 1;
                }
            }
        }
        if count2 < 9 {
            count += 1;
            if count < 1000 {
                continue;
            }
        }
        if can_place_monster(ctx, position) {
            break;
        }
    }
    let set_piece = ctx.gendung.SetPiece.position.mega_to_world();
    if uniqindex == UniqueMonsterType::SnotSpill {
        position = set_piece + Displacement::new(8, 12);
    }
    if uniqindex == UniqueMonsterType::WarlordOfBlood {
        position = set_piece + Displacement::new(6, 7);
    }
    if uniqindex == UniqueMonsterType::Zhar {
        for i in 0..ctx.gendung.themeCount {
            if i == ctx.themes.zharlib {
                position = ctx.gendung.themeLoc[i as usize].room.position.mega_to_world() + Displacement::new(4, 4);
                break;
            }
        }
    }
    if ctx.gendung.setlevel {
        match uniqindex {
            UniqueMonsterType::Lazarus => position = Point::new(32, 46),
            UniqueMonsterType::RedVex => position = Point::new(40, 45),
            UniqueMonsterType::BlackJade => position = Point::new(38, 49),
            UniqueMonsterType::SkeletonKing => position = Point::new(35, 47),
            _ => {}
        }
    } else {
        match uniqindex {
            UniqueMonsterType::Lazarus => position = set_piece + Displacement::new(3, 6),
            UniqueMonsterType::RedVex => position = set_piece + Displacement::new(5, 3),
            UniqueMonsterType::BlackJade => position = set_piece + Displacement::new(5, 9),
            _ => {}
        }
    }
    if uniqindex == UniqueMonsterType::Butcher {
        position = set_piece + Displacement::new(4, 4);
    }
    if uniqindex == UniqueMonsterType::NaKrul {
        if ctx.crypt.UberRow == 0 || ctx.crypt.UberCol == 0 {
            ctx.crypt.UberDiabloMonsterIndex = -1;
            return;
        }
        position = Point::new(ctx.crypt.UberRow - 2, ctx.crypt.UberCol);
        ctx.crypt.UberDiabloMonsterIndex = ctx.monster.ActiveMonsterCount as i32;
    }
    let type_index = get_monster_type_index(ctx, unique_monster_data.mtype);
    let amc = ctx.monster.ActiveMonsterCount;
    place_monster(ctx, amc, type_index, position);
    ctx.monster.ActiveMonsterCount += 1;
    prepare_unique_monst(ctx, m, uniqindex, minion_type, bosspacksize, unique_monster_data);
}

/// Original: `AddMonsterType(_monster_id, placeflag)` (monster.cpp).
// @port monster.cpp|devilution::AddMonsterType(_monster_id type, placeflag placeflag) sha=6c09ce415ef6
fn add_monster_type(ctx: &mut Ctx, type_: _monster_id, placeflag: placeflag) -> usize {
    let type_index = get_monster_type_index(ctx, type_);
    if type_index == ctx.monster.LevelMonsterTypeCount {
        ctx.monster.LevelMonsterTypeCount += 1;
        ctx.monster.LevelMonsterTypes[type_index].type_ = type_;
        ctx.monster.LevelMonsterTypes[type_index].data = type_ as usize;
        ctx.monster.monstimgtot += MonstersData[type_ as usize].image as i32;
        init_monster_gfx(ctx, type_index);
        init_monster_snd(ctx, type_index);
    }
    ctx.monster.LevelMonsterTypes[type_index].placeFlags |= placeflag;
    type_index
}

/// Original: `AddMonsterType(UniqueMonsterType, placeflag)` (monster.cpp).
// @port monster.cpp|devilution::AddMonsterType(UniqueMonsterType uniqueType, placeflag placeflag) sha=3a9631404a4c
fn add_monster_type_unique(ctx: &mut Ctx, unique_type: UniqueMonsterType, placeflag: placeflag) -> usize {
    add_monster_type(ctx, UniqueMonstersData[unique_type.0 as usize].mtype, placeflag)
}

/// Original: `PlaceUniqueMonsters` (monster.cpp).
// @port monster.cpp|devilution::PlaceUniqueMonsters() sha=2c60f004dea6
fn place_unique_monsters(ctx: &mut Ctx) {
    let mut u = 0;
    while UniqueMonstersData[u].mtype != -1 {
        let data = &UniqueMonstersData[u];
        u += 1;
        if data.mlevel != ctx.gendung.currlevel {
            continue;
        }
        let minion_type = get_monster_type_index(ctx, data.mtype);
        if minion_type == ctx.monster.LevelMonsterTypeCount {
            continue;
        }
        let unique_type = UniqueMonsterType::from_repr((u - 1) as u8).expect("UniqueMonsterType");
        let q = &ctx.quests.Quests;
        if unique_type == UniqueMonsterType::Garbud && q[Q_GARBUD as usize]._qactive == QUEST_NOTAVAIL {
            continue;
        }
        if unique_type == UniqueMonsterType::Zhar && q[Q_ZHAR as usize]._qactive == QUEST_NOTAVAIL {
            continue;
        }
        if unique_type == UniqueMonsterType::SnotSpill && q[Q_LTBANNER as usize]._qactive == QUEST_NOTAVAIL {
            continue;
        }
        if unique_type == UniqueMonsterType::Lachdan && q[Q_VEIL as usize]._qactive == QUEST_NOTAVAIL {
            continue;
        }
        if unique_type == UniqueMonsterType::WarlordOfBlood && q[Q_WARLORD as usize]._qactive == QUEST_NOTAVAIL {
            continue;
        }
        place_unique_monst(ctx, unique_type, minion_type, 8);
    }
}

/// Loads a `.dun` file and places its monsters at `position` (the `SetMapMonsters(LoadFileInMem(...))` pattern).
fn set_map_monsters_from_file(ctx: &mut Ctx, path: &str, position: Point) {
    let bytes = crate::engine::load_file::load_file_in_mem(ctx, path).unwrap_or_default();
    let dun_data: Vec<u16> = bytes.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    set_map_monsters(ctx, &dun_data, position);
}

/// Original: `PlaceQuestMonsters` (monster.cpp).
// @port monster.cpp|devilution::PlaceQuestMonsters() sha=6154d6a0e7f8
fn place_quest_monsters(ctx: &mut Ctx) {
    use crate::quests::is_quest_available;
    if !ctx.gendung.setlevel {
        let set_piece = ctx.gendung.SetPiece.position.mega_to_world();
        if is_quest_available(ctx, Q_BUTCHER) {
            place_unique_monst(ctx, UniqueMonsterType::Butcher, 0, 0);
        }
        if ctx.gendung.currlevel == ctx.quests.Quests[Q_SKELKING as usize]._qlevel && crate::quests::use_multiplayer_quests(ctx) {
            for i in 0..ctx.monster.LevelMonsterTypeCount {
                if is_skel(ctx.monster.LevelMonsterTypes[i].type_) {
                    place_unique_monst(ctx, UniqueMonsterType::SkeletonKing, i, 30);
                    break;
                }
            }
        }
        if is_quest_available(ctx, Q_LTBANNER) {
            set_map_monsters_from_file(ctx, "levels\\l1data\\banner1.dun", set_piece);
        }
        if is_quest_available(ctx, Q_BLOOD) {
            set_map_monsters_from_file(ctx, "levels\\l2data\\blood2.dun", set_piece);
        }
        if is_quest_available(ctx, Q_BLIND) {
            set_map_monsters_from_file(ctx, "levels\\l2data\\blind2.dun", set_piece);
        }
        if is_quest_available(ctx, Q_ANVIL) {
            set_map_monsters_from_file(ctx, "levels\\l3data\\anvil.dun", set_piece + Displacement::new(2, 2));
        }
        if is_quest_available(ctx, Q_WARLORD) {
            set_map_monsters_from_file(ctx, "levels\\l4data\\warlord.dun", set_piece);
            add_monster_type_unique(ctx, UniqueMonsterType::WarlordOfBlood, PLACE_SCATTER);
        }
        if is_quest_available(ctx, Q_VEIL) {
            add_monster_type_unique(ctx, UniqueMonsterType::Lachdan, PLACE_SCATTER);
        }
        if is_quest_available(ctx, Q_ZHAR) && ctx.themes.zharlib == -1 {
            ctx.quests.Quests[Q_ZHAR as usize]._qactive = QUEST_NOTAVAIL;
        }
        if ctx.gendung.currlevel == ctx.quests.Quests[Q_BETRAYER as usize]._qlevel && crate::quests::use_multiplayer_quests(ctx) {
            add_monster_type_unique(ctx, UniqueMonsterType::Lazarus, PLACE_UNIQUE);
            add_monster_type_unique(ctx, UniqueMonsterType::RedVex, PLACE_UNIQUE);
            place_unique_monst(ctx, UniqueMonsterType::Lazarus, 0, 0);
            place_unique_monst(ctx, UniqueMonsterType::RedVex, 0, 0);
            place_unique_monst(ctx, UniqueMonsterType::BlackJade, 0, 0);
            set_map_monsters_from_file(ctx, "levels\\l4data\\vile1.dun", set_piece);
        }
        if ctx.gendung.currlevel == 24 {
            ctx.crypt.UberDiabloMonsterIndex = -1;
            let type_index = get_monster_type_index(ctx, MT_NAKRUL);
            if type_index < ctx.monster.LevelMonsterTypeCount {
                for i in 0..ctx.monster.ActiveMonsterCount {
                    let monster = &ctx.monster.Monsters[i];
                    if monster.is_unique() || monster.levelType as usize == type_index {
                        ctx.crypt.UberDiabloMonsterIndex = i as i32;
                        break;
                    }
                }
            }
            if ctx.crypt.UberDiabloMonsterIndex == -1 {
                place_unique_monst(ctx, UniqueMonsterType::NaKrul, 0, 0);
            }
        }
    } else if ctx.gendung.setlvlnum == SL_SKELKING {
        place_unique_monst(ctx, UniqueMonsterType::SkeletonKing, 0, 0);
    }
}

/// Original: `LoadDiabMonsts` (monster.cpp).
// @port monster.cpp|devilution::LoadDiabMonsts() sha=45666da821c5
fn load_diab_monsts(ctx: &mut Ctx) {
    let q = ctx.drlg_l4.diablo_quads;
    set_map_monsters_from_file(ctx, "levels\\l4data\\diab1.dun", q[0].mega_to_world());
    set_map_monsters_from_file(ctx, "levels\\l4data\\diab2a.dun", q[1].mega_to_world());
    set_map_monsters_from_file(ctx, "levels\\l4data\\diab3a.dun", q[2].mega_to_world());
    set_map_monsters_from_file(ctx, "levels\\l4data\\diab4a.dun", q[3].mega_to_world());
}

/// Original: `NewMonsterAnim` (monster.cpp).
// @port monster.cpp|devilution::NewMonsterAnim(Monster &monster, MonsterGraphic graphic, Direction md, AnimationDistributionFlags flags = AnimationDistributionFlags::None, int8_t numSkippedFrames = 0, int8_t distributeFramesBeforeFrame = 0) sha=1dc31ec71c59
fn new_monster_anim_full(ctx: &mut Ctx, m: usize, graphic: MonsterGraphic, md: Direction, flags: AnimationDistributionFlags, num_skipped_frames: i8, distribute_frames_before_frame: i8) {
    let a = get_anim_data(ctx, m, graphic);
    let (sprites, frames, rate) = (a.sprites_for_direction(md), a.frames, a.rate);
    let monster = &mut ctx.monster.Monsters[m];
    monster.animInfo.set_new_animation(sprites, frames, rate, flags, num_skipped_frames, distribute_frames_before_frame, 0);
    monster.flags &= !((MFLAG_LOCK_ANIMATION | MFLAG_ALLOW_SPECIAL) as u32);
    monster.direction = md;
}

fn new_monster_anim(ctx: &mut Ctx, m: usize, graphic: MonsterGraphic, md: Direction) {
    new_monster_anim_full(ctx, m, graphic, md, AnimationDistributionFlags::None, 0, 0);
}

/// Original: `StartMonsterGotHit` (monster.cpp).
// @port monster.cpp|devilution::StartMonsterGotHit(Monster &monster) sha=4d9389638bc7
fn start_monster_got_hit(ctx: &mut Ctx, m: usize) {
    let t = monster_type_id(ctx, m);
    if t != MT_GOLEM {
        let animation_flags = if ctx.diablo.g_game_logic_step < GameLogicStep::ProcessMonsters { AnimationDistributionFlags::ProcessAnimationPending } else { AnimationDistributionFlags::None };
        let num_skipped_frames = if ctx.init.gb_is_hellfire && t == MT_DIABLO { 4 } else { 0 };
        let d = ctx.monster.Monsters[m].direction;
        new_monster_anim_full(ctx, m, MonsterGraphic::GotHit, d, animation_flags, num_skipped_frames, 0);
        ctx.monster.Monsters[m].mode = MonsterMode::HitRecovery;
    }
    {
        let monster = &mut ctx.monster.Monsters[m];
        monster.position.tile = monster.position.old;
        monster.position.future = monster.position.old;
    }
    m_clear_squares(ctx, m);
    let tile = ctx.monster.Monsters[m].position.tile;
    ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = (m + 1) as i16;
}

/// Original: `IsRanged` (monster.cpp).
// @port monster.cpp|devilution::IsRanged(Monster &monster) sha=87e5df96090a
fn is_ranged(monster: &Monster) -> bool {
    matches!(monster.ai, MonsterAIID::SkeletonRanged | MonsterAIID::GoatRanged | MonsterAIID::Succubus | MonsterAIID::LazarusSuccubus)
}

/// Original: `UpdateEnemy` (monster.cpp).
// @port monster.cpp|devilution::UpdateEnemy(Monster &monster) sha=8b2517a07b72
fn update_enemy(ctx: &mut Ctx, m: usize) {
    let mut target = Point::default();
    let mut menemy: i32 = -1;
    let mut best_dist: i32 = -1;
    let mut bestsameroom = false;
    let position = ctx.monster.Monsters[m].position.tile;
    let is_player_minion = ctx.monster.Monsters[m].is_player_minion();
    let tv = |ctx: &Ctx, p: Point| ctx.gendung.dTransVal[p.x as usize][p.y as usize];
    if !is_player_minion {
        for pnum in 0..ctx.players.Players.len() {
            let player = &ctx.players.Players[pnum];
            if !player.plractive || !crate::player::is_on_active_level(ctx, pnum) || player._pLvlChanging || ((player._pHitPoints >> 6) == 0 && ctx.init.gb_is_multiplayer) {
                continue;
            }
            let sameroom = tv(ctx, position) == tv(ctx, player.position.tile);
            let dist = position.walking_distance(player.position.tile);
            if (sameroom && !bestsameroom) || ((sameroom || !bestsameroom) && dist < best_dist) || menemy == -1 {
                ctx.monster.Monsters[m].flags &= !(MFLAG_TARGETS_MONSTER as u32);
                menemy = pnum as i32;
                target = ctx.players.Players[pnum].position.future;
                best_dist = dist;
                bestsameroom = sameroom;
            }
        }
    }
    for i in 0..ctx.monster.ActiveMonsterCount {
        let monster_id = ctx.monster.ActiveMonsters[i] as usize;
        if monster_id == m {
            continue;
        }
        let other = &ctx.monster.Monsters[monster_id];
        if (other.hitPoints >> 6) <= 0 {
            continue;
        }
        if other.position.tile == GOLEM_HOLDING_CELL {
            continue;
        }
        if other.talkMsg != TEXT_NONE && m_talker(ctx, monster_id) {
            continue;
        }
        if is_player_minion && other.is_player_minion() {
            // prevent golems from fighting each other
            continue;
        }
        let dist = other.position.tile.walking_distance(position);
        let mflags = ctx.monster.Monsters[m].flags;
        let golem = (mflags & MFLAG_GOLEM as u32) != 0;
        let berserk = (mflags & MFLAG_BERSERK as u32) != 0;
        if (!golem && !berserk && dist >= 2 && !is_ranged(&ctx.monster.Monsters[m])) || (!golem && !berserk && (other.flags & MFLAG_GOLEM as u32) == 0) {
            continue;
        }
        let sameroom = tv(ctx, position) == tv(ctx, other.position.tile);
        if (sameroom && !bestsameroom) || ((sameroom || !bestsameroom) && dist < best_dist) || menemy == -1 {
            target = other.position.future;
            ctx.monster.Monsters[m].flags |= MFLAG_TARGETS_MONSTER as u32;
            menemy = monster_id as i32;
            best_dist = dist;
            bestsameroom = sameroom;
        }
    }
    let monster = &mut ctx.monster.Monsters[m];
    if menemy != -1 {
        monster.flags &= !(MFLAG_NO_ENEMY as u32);
        monster.enemy = menemy as u8;
        monster.enemyPosition = target;
    } else {
        monster.flags |= MFLAG_NO_ENEMY as u32;
    }
}

/// Original: `AiDelay` (monster.cpp): make the AI wait a bit before thinking again.
// @port monster.cpp|devilution::AiDelay(Monster &monster, int len) sha=491c9896ae01
fn ai_delay(ctx: &mut Ctx, m: usize, len: i32) {
    if len <= 0 {
        return;
    }
    let monster = &mut ctx.monster.Monsters[m];
    if monster.ai == MonsterAIID::Lazarus {
        return;
    }
    monster.var2 = len as i16;
    monster.mode = MonsterMode::Delay;
}

/// Original: `GetMonsterDirection` (monster.cpp): the direction from the monster to its current enemy.
// @port monster.cpp|devilution::GetMonsterDirection(Monster &monster) sha=c6dcfffa16a7
fn get_monster_direction(ctx: &Ctx, m: usize) -> Direction {
    let monster = &ctx.monster.Monsters[m];
    crate::engine::get_direction(monster.position.tile, monster.enemyPosition)
}

/// Original: `StartSpecialStand` (monster.cpp).
// @port monster.cpp|devilution::StartSpecialStand(Monster &monster, Direction md) sha=bd1a4be685b8
fn start_special_stand(ctx: &mut Ctx, m: usize, md: Direction) {
    new_monster_anim(ctx, m, MonsterGraphic::Special, md);
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::SpecialStand;
    monster.position.future = monster.position.tile;
    monster.position.old = monster.position.tile;
}

/// Original: `WalkNorthwards` (monster.cpp).
// @port monster.cpp|devilution::WalkNorthwards(Monster &monster, int xadd, int yadd, Direction endDir) sha=1dfe08ce7b16
fn walk_northwards(ctx: &mut Ctx, m: usize, xadd: i32, yadd: i32, end_dir: Direction) {
    let tile = ctx.monster.Monsters[m].position.tile;
    let fx = (xadd + tile.x) as u8 as i32;
    let fy = (yadd + tile.y) as u8 as i32;
    ctx.gendung.dMonster[fx as usize][fy as usize] = -((m + 1) as i16);
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::MoveNorthwards;
    monster.position.old = monster.position.tile;
    monster.position.future = Point::new(fx, fy);
    monster.var1 = xadd as i16;
    monster.var2 = yadd as i16;
    monster.var3 = end_dir as i8;
    new_monster_anim_full(ctx, m, MonsterGraphic::Walk, end_dir, AnimationDistributionFlags::ProcessAnimationPending, -1, 0);
}

/// Original: `WalkSouthwards` (monster.cpp).
// @port monster.cpp|devilution::WalkSouthwards(Monster &monster, int xoff, int yoff, int xadd, int yadd, Direction endDir) sha=6ce536cadabc
fn walk_southwards(ctx: &mut Ctx, m: usize, _xoff: i32, _yoff: i32, xadd: i32, yadd: i32, end_dir: Direction) {
    let tile = ctx.monster.Monsters[m].position.tile;
    let fx = (xadd + tile.x) as u8 as i32;
    let fy = (yadd + tile.y) as u8 as i32;
    ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = -((m + 1) as i16);
    {
        let monster = &mut ctx.monster.Monsters[m];
        monster.var1 = tile.x as i16;
        monster.var2 = tile.y as i16;
        monster.position.old = tile;
        monster.position.tile = Point::new(fx, fy);
        monster.position.future = Point::new(fx, fy);
    }
    ctx.gendung.dMonster[fx as usize][fy as usize] = (m + 1) as i16;
    let light_id = ctx.monster.Monsters[m].lightId as i32;
    if light_id != NO_LIGHT {
        crate::lighting::change_light_xy(ctx, light_id, Point::new(fx, fy));
    }
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::MoveSouthwards;
    monster.var3 = end_dir as i8;
    new_monster_anim_full(ctx, m, MonsterGraphic::Walk, end_dir, AnimationDistributionFlags::ProcessAnimationPending, -1, 0);
}

/// Original: `WalkSideways` (monster.cpp).
// @port monster.cpp|devilution::WalkSideways(Monster &monster, int xoff, int yoff, int xadd, int yadd, int mapx, int mapy, Direction endDir) sha=da0256c921e1
#[allow(clippy::too_many_arguments)]
fn walk_sideways(ctx: &mut Ctx, m: usize, _xoff: i32, _yoff: i32, xadd: i32, yadd: i32, mapx: i32, mapy: i32, end_dir: Direction) {
    let tile = ctx.monster.Monsters[m].position.tile;
    let fx = (xadd + tile.x) as u8 as i32;
    let fy = (yadd + tile.y) as u8 as i32;
    let x = (mapx + tile.x) as u8 as i32;
    let y = (mapy + tile.y) as u8 as i32;
    let light_id = ctx.monster.Monsters[m].lightId as i32;
    if light_id != NO_LIGHT {
        crate::lighting::change_light_xy(ctx, light_id, Point::new(x, y));
    }
    ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = -((m + 1) as i16);
    ctx.gendung.dMonster[fx as usize][fy as usize] = (m + 1) as i16;
    let monster = &mut ctx.monster.Monsters[m];
    monster.position.temp = Point::new(x, y);
    monster.position.old = tile;
    monster.position.future = Point::new(fx, fy);
    monster.mode = MonsterMode::MoveSideways;
    monster.var1 = fx as i16;
    monster.var2 = fy as i16;
    monster.var3 = end_dir as i8;
    new_monster_anim_full(ctx, m, MonsterGraphic::Walk, end_dir, AnimationDistributionFlags::ProcessAnimationPending, -1, 0);
}

/// Original: `StartAttack` (monster.cpp).
// @port monster.cpp|devilution::StartAttack(Monster &monster) sha=04c2fd9fa474
fn start_attack(ctx: &mut Ctx, m: usize) {
    let md = get_monster_direction(ctx, m);
    new_monster_anim_full(ctx, m, MonsterGraphic::Attack, md, AnimationDistributionFlags::ProcessAnimationPending, 0, 0);
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::MeleeAttack;
    monster.position.future = monster.position.tile;
    monster.position.old = monster.position.tile;
}

/// Original: `StartRangedAttack` (monster.cpp).
// @port monster.cpp|devilution::StartRangedAttack(Monster &monster, MissileID missileType, int dam) sha=1b8ee4957dc4
fn start_ranged_attack(ctx: &mut Ctx, m: usize, missile_type: MissileID, dam: i32) {
    let md = get_monster_direction(ctx, m);
    new_monster_anim_full(ctx, m, MonsterGraphic::Attack, md, AnimationDistributionFlags::ProcessAnimationPending, 0, 0);
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::RangedAttack;
    monster.var1 = missile_type as i8 as i16;
    monster.var2 = dam as i16;
    monster.position.future = monster.position.tile;
    monster.position.old = monster.position.tile;
}

/// Original: `StartRangedSpecialAttack` (monster.cpp).
// @port monster.cpp|devilution::StartRangedSpecialAttack(Monster &monster, MissileID missileType, int dam) sha=9a9e3e2fa42d
fn start_ranged_special_attack(ctx: &mut Ctx, m: usize, missile_type: MissileID, dam: i32) {
    let md = get_monster_direction(ctx, m);
    let mut distribute_frames_before_frame = 0;
    if ctx.monster.Monsters[m].ai == MonsterAIID::Mega {
        distribute_frames_before_frame = monster_data(ctx, m).animFrameNumSpecial;
    }
    new_monster_anim_full(ctx, m, MonsterGraphic::Special, md, AnimationDistributionFlags::ProcessAnimationPending, 0, distribute_frames_before_frame);
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::SpecialRangedAttack;
    monster.var1 = missile_type as i8 as i16;
    monster.var2 = 0;
    monster.var3 = dam as i8;
    monster.position.future = monster.position.tile;
    monster.position.old = monster.position.tile;
}

/// Original: `StartSpecialAttack` (monster.cpp).
// @port monster.cpp|devilution::StartSpecialAttack(Monster &monster) sha=501ba0116ede
fn start_special_attack(ctx: &mut Ctx, m: usize) {
    let md = get_monster_direction(ctx, m);
    new_monster_anim(ctx, m, MonsterGraphic::Special, md);
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::SpecialMeleeAttack;
    monster.position.future = monster.position.tile;
    monster.position.old = monster.position.tile;
}

/// Original: `StartEating` (monster.cpp).
// @port monster.cpp|devilution::StartEating(Monster &monster) sha=328fb3e352a3
fn start_eating(ctx: &mut Ctx, m: usize) {
    let d = ctx.monster.Monsters[m].direction;
    new_monster_anim(ctx, m, MonsterGraphic::Special, d);
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::SpecialMeleeAttack;
    monster.position.future = monster.position.tile;
    monster.position.old = monster.position.tile;
}

/// Original: `DiabloDeath` (monster.cpp).
// @port monster.cpp|devilution::DiabloDeath(Monster &diablo, bool sendmsg) sha=6a9c4e5082f6
fn diablo_death(ctx: &mut Ctx, diablo: usize, sendmsg: bool) {
    crate::effects::play_sfx(ctx, crate::effects::USFX_DIABLOD);
    ctx.quests.Quests[Q_DIABLO as usize]._qactive = QUEST_DONE;
    if sendmsg {
        crate::msg::net_send_cmd_quest(ctx, true, Q_DIABLO as usize);
    }
    ctx.monster.sgbSaveSoundOn = ctx.sound.gb_sound_on;
    ctx.diablo.gb_process_players = false;
    for i in 0..ctx.monster.ActiveMonsterCount {
        let monster_id = ctx.monster.ActiveMonsters[i] as usize;
        if monster_type_id(ctx, monster_id) == MT_DIABLO || ctx.monster.Monsters[diablo].activeForTicks == 0 {
            continue;
        }
        let d = ctx.monster.Monsters[monster_id].direction;
        new_monster_anim(ctx, monster_id, MonsterGraphic::Death, d);
        {
            let monster = &mut ctx.monster.Monsters[monster_id];
            monster.mode = MonsterMode::Death;
            monster.var1 = 0;
            monster.position.tile = monster.position.old;
            monster.position.future = monster.position.tile;
        }
        m_clear_squares(ctx, monster_id);
        let t = ctx.monster.Monsters[monster_id].position.tile;
        ctx.gendung.dMonster[t.x as usize][t.y as usize] = (monster_id + 1) as i16;
    }
    let dtile = ctx.monster.Monsters[diablo].position.tile;
    crate::lighting::add_light(ctx, dtile, 8);
    crate::lighting::do_vision(ctx, dtile, 8, crate::lighting::MAP_EXP_NONE, true);
    let view = ctx.gendung.ViewPosition;
    let mut dist = dtile.walking_distance(view);
    if dist > 20 {
        dist = 20;
    }
    {
        let d = &mut ctx.monster.Monsters[diablo];
        // var3 is an int8_t in the original, which truncates `ViewPosition.x << 16` to 0.
        d.var3 = (view.x << 16) as i8;
        d.position.temp.x = view.y << 16;
        d.position.temp.y = ((d.var3 as i32 - (d.position.tile.x << 16)) as f64 / dist as f64) as i32;
    }
    if !ctx.init.gb_is_multiplayer {
        let me = ctx.players.MyPlayer.expect("MyPlayer");
        let lvl = (ctx.multi.sgGameInitInfo.nDifficulty as u8).wrapping_add(1);
        let p = &mut ctx.players.Players[me];
        p.pDiabloKillLevel = p.pDiabloKillLevel.max(lvl);
    }
}

/// Original: `SpawnLoot` (monster.cpp).
// @port monster.cpp|devilution::SpawnLoot(Monster &monster, bool sendmsg) sha=12c1d04ca7b0
fn spawn_loot(ctx: &mut Ctx, m: usize, sendmsg: bool) {
    let t = monster_type_id(ctx, m);
    if t == MT_HORKSPWN {
        return;
    }
    let tile = ctx.monster.Monsters[m].position.tile;
    let unique = ctx.monster.Monsters[m].uniqueType;
    if crate::quests::is_quest_available(ctx, Q_GARBUD) && unique == UniqueMonsterType::Garbud {
        crate::items::create_type_item(ctx, tile + Displacement::new(1, 1), true, ItemType::Mace, IMISC_NONE as i32, sendmsg, false, false);
    } else if unique == UniqueMonsterType::Defiler {
        if crate::effects::effect_is_playing(ctx, crate::effects::USFX_DEFILER8 as i32) {
            crate::effects::stream_stop(ctx);
        }
        crate::items::spawn_map_of_doom(ctx, tile, sendmsg);
        ctx.quests.Quests[Q_DEFILER as usize]._qactive = QUEST_DONE;
        crate::msg::net_send_cmd_quest(ctx, true, Q_DEFILER as usize);
    } else if unique == UniqueMonsterType::HorkDemon {
        if ctx.multi.sgGameInitInfo.bTheoQuest != 0 {
            crate::items::spawn_theodore(ctx, tile, sendmsg);
        } else {
            crate::items::create_amulet(ctx, tile, 13, sendmsg, false, false);
        }
    } else if t == MT_NAKRUL {
        let mut n_sfx = if ctx.crypt.IsUberRoomOpened { crate::effects::USFX_NAKRUL4 } else { crate::effects::USFX_NAKRUL5 };
        if ctx.multi.sgGameInitInfo.bCowQuest != 0 {
            n_sfx = crate::effects::USFX_NAKRUL6;
        }
        if crate::effects::effect_is_playing(ctx, n_sfx as i32) {
            crate::effects::stream_stop(ctx);
        }
        ctx.crypt.UberDiabloMonsterIndex = -2;
        crate::items::create_magic_weapon(ctx, tile, ItemType::Sword, ICURS_GREAT_SWORD as i32, sendmsg, false);
        crate::items::create_magic_weapon(ctx, tile, ItemType::Staff, ICURS_WAR_STAFF as i32, sendmsg, false);
        crate::items::create_magic_weapon(ctx, tile, ItemType::Bow, ICURS_LONG_WAR_BOW as i32, sendmsg, false);
        crate::items::create_spell_book(ctx, tile, SpellID::Apocalypse, sendmsg, false);
    } else if !ctx.monster.Monsters[m].is_player_minion() {
        crate::items::spawn_item(ctx, m, tile, sendmsg, false);
    }
}

/// Original: `GetTeleportTile` (monster.cpp).
// @port monster.cpp|devilution::GetTeleportTile(const Monster &monster) sha=3af978e998fc
fn get_teleport_tile(ctx: &mut Ctx, m: usize) -> Option<Point> {
    let (mx, my) = (ctx.monster.Monsters[m].enemyPosition.x, ctx.monster.Monsters[m].enemyPosition.y);
    let rx = ctx.rng.pick_randomly_among(&[-1, 1]);
    let ry = ctx.rng.pick_randomly_among(&[-1, 1]);
    let tile = ctx.monster.Monsters[m].position.tile;
    for j in -1..=1 {
        for k in -1..1 {
            if j == 0 && k == 0 {
                continue;
            }
            let x = mx + rx * j;
            let y = my + ry * k;
            if !in_dungeon_bounds(Point::new(x, y)) || x == tile.x || y == tile.y {
                continue;
            }
            if is_tile_available_for_monster(ctx, m, Point::new(x, y)) {
                return Some(Point::new(x, y));
            }
        }
    }
    None
}

/// Original: `Teleport` (monster.cpp).
// @port monster.cpp|devilution::Teleport(Monster &monster) sha=867c30cae994
fn teleport(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode == MonsterMode::Petrified {
        return;
    }
    let Some(position) = get_teleport_tile(ctx, m) else {
        return;
    };
    m_clear_squares(ctx, m);
    let tile = ctx.monster.Monsters[m].position.tile;
    ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = 0;
    ctx.gendung.dMonster[position.x as usize][position.y as usize] = (m + 1) as i16;
    ctx.monster.Monsters[m].position.old = position;
    ctx.monster.Monsters[m].direction = get_monster_direction(ctx, m);
    let light_id = ctx.monster.Monsters[m].lightId as i32;
    if light_id != NO_LIGHT {
        crate::lighting::change_light_xy(ctx, light_id, position);
    }
}

/// Original: `IsHardHit` (monster.cpp).
// @port monster.cpp|devilution::IsHardHit(Monster &target, unsigned dam) sha=c943ece9eb6f
fn is_hard_hit(ctx: &Ctx, target: usize, dam: u32) -> bool {
    match monster_type_id(ctx, target) {
        MT_SNEAK | MT_STALKER | MT_UNSEEN | MT_ILLWEAV => true,
        _ => (dam >> 6) >= monster_level(ctx, target, ctx.multi.sgGameInitInfo.nDifficulty) + 3,
    }
}

/// Original: `MonsterHitMonster` (monster.cpp).
// @port monster.cpp|devilution::MonsterHitMonster(Monster &attacker, Monster &target, int dam) sha=ee7f7653fd4b
fn monster_hit_monster(ctx: &mut Ctx, attacker: usize, target: usize, dam: i32) {
    if is_hard_hit(ctx, target, dam as u32) {
        ctx.monster.Monsters[target].direction = opposite(ctx.monster.Monsters[attacker].direction);
    }
    m_start_hit_dam(ctx, target, dam);
}

/// Original: `StartDeathFromMonster` (monster.cpp).
// @port monster.cpp|devilution::StartDeathFromMonster(Monster &attacker, Monster &target) sha=9cd9f9cce23d
fn start_death_from_monster(ctx: &mut Ctx, attacker: usize, target: usize) {
    let md = crate::engine::get_direction(ctx.monster.Monsters[target].position.tile, ctx.monster.Monsters[attacker].position.tile);
    monster_death_dir(ctx, target, md, true);
    if ctx.init.gb_is_hellfire {
        let d = ctx.monster.Monsters[attacker].direction;
        m_start_stand(ctx, attacker, d);
    }
}

/// Original: `StartFadein` (monster.cpp).
// @port monster.cpp|devilution::StartFadein(Monster &monster, Direction md, bool backwards) sha=f9f36ac32593
fn start_fadein(ctx: &mut Ctx, m: usize, md: Direction, backwards: bool) {
    new_monster_anim(ctx, m, MonsterGraphic::Special, md);
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::FadeIn;
    monster.position.future = monster.position.tile;
    monster.position.old = monster.position.tile;
    monster.flags &= !(MFLAG_HIDDEN as u32);
    if backwards {
        monster.flags |= MFLAG_LOCK_ANIMATION as u32;
        monster.animInfo.currentFrame = monster.animInfo.numberOfFrames - 1;
    }
}

/// Original: `StartFadeout` (monster.cpp).
// @port monster.cpp|devilution::StartFadeout(Monster &monster, Direction md, bool backwards) sha=e3f0a828622c
fn start_fadeout(ctx: &mut Ctx, m: usize, md: Direction, backwards: bool) {
    new_monster_anim(ctx, m, MonsterGraphic::Special, md);
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::FadeOut;
    monster.position.future = monster.position.tile;
    monster.position.old = monster.position.tile;
    if backwards {
        monster.flags |= MFLAG_LOCK_ANIMATION as u32;
        monster.animInfo.currentFrame = monster.animInfo.numberOfFrames - 1;
    }
}

/// Original: `StartHeal` (monster.cpp): starts the monster healing procedure (gargoyles only).
// @port monster.cpp|devilution::StartHeal(Monster &monster) sha=a40734ecfbff
fn start_heal(ctx: &mut Ctx, m: usize) {
    change_animation_data(ctx, m, MonsterGraphic::Special);
    let frames = get_anim_data(ctx, m, MonsterGraphic::Special).frames;
    let r = rnd(ctx, 5);
    let monster = &mut ctx.monster.Monsters[m];
    monster.animInfo.currentFrame = frames - 1;
    monster.flags |= MFLAG_LOCK_ANIMATION as u32;
    monster.mode = MonsterMode::Heal;
    monster.var1 = (monster.maxHitPoints / (16 * (r + 4))) as i16;
}

/// Original: `SyncLightPosition` (monster.cpp).
// @port monster.cpp|devilution::SyncLightPosition(Monster &monster) sha=72140cde3e43
fn sync_light_position(ctx: &mut Ctx, m: usize) {
    let light_id = ctx.monster.Monsters[m].lightId as i32;
    if light_id == NO_LIGHT {
        return;
    }
    let offset = if is_walking(ctx, m) {
        let monster = &ctx.monster.Monsters[m];
        monster.position.calculate_walking_offset(monster.direction, &monster.animInfo, ctx.nthread.ProgressToNextGameTick)
    } else {
        Displacement::default()
    };
    crate::lighting::change_light_offset(ctx, light_id, offset.screen_to_light());
}

/// Original: `MonsterIdle` (monster.cpp).
// @port monster.cpp|devilution::MonsterIdle(Monster &monster) sha=0d36eacb78d8
fn monster_idle(ctx: &mut Ctx, m: usize) {
    if monster_type_id(ctx, m) == MT_GOLEM {
        change_animation_data(ctx, m, MonsterGraphic::Walk);
    } else {
        change_animation_data(ctx, m, MonsterGraphic::Stand);
    }
    if ctx.monster.Monsters[m].animInfo.is_last_frame() {
        update_enemy(ctx, m);
    }
    let monster = &mut ctx.monster.Monsters[m];
    if monster.var2 < i16::MAX {
        monster.var2 += 1;
    }
}

/// Original: `MonsterWalk` (monster.cpp): continue movement towards new tile.
// @port monster.cpp|devilution::MonsterWalk(Monster &monster, MonsterMode variant) sha=cdafc4565f46
fn monster_walk(ctx: &mut Ctx, m: usize, variant: MonsterMode) -> bool {
    // Check if we reached new tile
    let is_animation_end = ctx.monster.Monsters[m].animInfo.is_last_frame();
    if is_animation_end {
        let mon = ctx.monster.Monsters[m].clone();
        match variant {
            MonsterMode::MoveNorthwards => {
                ctx.gendung.dMonster[mon.position.tile.x as usize][mon.position.tile.y as usize] = 0;
                let t = Point::new(mon.position.tile.x + mon.var1 as i32, mon.position.tile.y + mon.var2 as i32);
                ctx.monster.Monsters[m].position.tile = t;
                ctx.gendung.dMonster[t.x as usize][t.y as usize] = (m + 1) as i16;
            }
            MonsterMode::MoveSouthwards => {
                ctx.gendung.dMonster[mon.var1 as usize][mon.var2 as usize] = 0;
            }
            MonsterMode::MoveSideways => {
                ctx.gendung.dMonster[mon.position.tile.x as usize][mon.position.tile.y as usize] = 0;
                let t = Point::new(mon.var1 as u8 as i32, mon.var2 as u8 as i32);
                ctx.monster.Monsters[m].position.tile = t;
                // dMonster is set here for backwards comparability, without it the monster would be invisible if loaded from a vanilla save.
                ctx.gendung.dMonster[t.x as usize][t.y as usize] = (m + 1) as i16;
            }
            _ => {}
        }
        let light_id = ctx.monster.Monsters[m].lightId as i32;
        if light_id != NO_LIGHT {
            let t = ctx.monster.Monsters[m].position.tile;
            crate::lighting::change_light_xy(ctx, light_id, t);
        }
        let d = ctx.monster.Monsters[m].direction;
        m_start_stand(ctx, m, d);
    } else {
        // We didn't reach new tile so update monster's "sub-tile" position
        let a = &ctx.monster.Monsters[m].animInfo;
        if a.tickCounterOfCurrentFrame == 0 && a.currentFrame == 0 && monster_type_id(ctx, m) == MT_FLESTHNG {
            play_effect(ctx, m, MonsterSound::Special);
        }
    }
    if ctx.monster.Monsters[m].lightId as i32 != NO_LIGHT {
        sync_light_position(ctx, m);
    }
    is_animation_end
}

/// Original: `MonsterAttackMonster` (monster.cpp).
// @port monster.cpp|devilution::MonsterAttackMonster(Monster &attacker, Monster &target, int hper, int mind, int maxd) sha=b1f74ab454cd
fn monster_attack_monster(ctx: &mut Ctx, attacker: usize, target: usize, hper: i32, mind: i32, maxd: i32) {
    if !is_possible_to_hit(ctx, target) {
        return;
    }
    let mut hit = rnd(ctx, 100);
    if ctx.monster.Monsters[target].mode == MonsterMode::Petrified {
        hit = 0;
    }
    if try_lift_gargoyle(ctx, target) {
        return;
    }
    if hit >= hper {
        return;
    }
    let dam = (mind + rnd(ctx, maxd - mind + 1)) << 6;
    apply_monster_damage(ctx, DamageType::Physical, target, dam);
    if ctx.monster.Monsters[attacker].is_player_minion() {
        let player_id = attacker;
        tag(ctx, target, player_id);
    }
    if ctx.monster.Monsters[target].hitPoints >> 6 <= 0 {
        start_death_from_monster(ctx, attacker, target);
    } else {
        monster_hit_monster(ctx, attacker, target, dam);
    }
    if ctx.monster.Monsters[target].activeForTicks == 0 {
        let at = ctx.monster.Monsters[attacker].position.tile;
        let t = &mut ctx.monster.Monsters[target];
        t.activeForTicks = u8::MAX;
        t.position.last = at;
    }
}

/// Original: `CheckReflect` (monster.cpp).
// @port monster.cpp|devilution::CheckReflect(Monster &monster, Player &player, int dam) sha=ed2df343e47f
fn check_reflect(ctx: &mut Ctx, m: usize, pnum: usize, dam: i32) -> i32 {
    ctx.players.Players[pnum].wReflections = ctx.players.Players[pnum].wReflections.wrapping_sub(1);
    if ctx.players.Players[pnum].wReflections as i16 <= 0 {
        crate::msg::net_send_cmd_param1(ctx, true, CMD_SETREFLECT, 0);
    }
    // reflects 20-30% damage
    let mdam = dam * ctx.rng.random_int_between(20, 30, true) / 100;
    apply_monster_damage(ctx, DamageType::Physical, m, mdam);
    if ctx.monster.Monsters[m].hitPoints >> 6 <= 0 {
        m_start_kill(ctx, m, pnum);
    } else {
        m_start_hit(ctx, m, pnum, mdam);
    }
    mdam
}

/// Original: `GetMinHit` (monster.cpp).
// @port monster.cpp|devilution::GetMinHit() sha=409ef5617b44
fn get_min_hit(ctx: &Ctx) -> i32 {
    match ctx.gendung.currlevel {
        16 => 30,
        15 => 25,
        14 => 20,
        _ => 15,
    }
}

/// Original: `MonsterAttackPlayer` (monster.cpp), release build (no god mode).
// @port monster.cpp|devilution::MonsterAttackPlayer(Monster &monster, Player &player, int hit, int minDam, int maxDam) sha=0a1981c8d50d
fn monster_attack_player(ctx: &mut Ctx, m: usize, pnum: usize, mut hit: i32, min_dam: i32, max_dam: i32) {
    {
        let player = &ctx.players.Players[pnum];
        if player._pHitPoints >> 6 <= 0 || player._pInvincible || player._pSpellFlags.has_any_of(SpellFlag::Etherealize) {
            return;
        }
        if ctx.monster.Monsters[m].position.tile.walking_distance(player.position.tile) >= 2 {
            return;
        }
    }
    let hper = rnd(ctx, 100);
    let difficulty = ctx.multi.sgGameInitInfo.nDifficulty;
    let mlevel = monster_level(ctx, m, difficulty) as i32;
    let mclass = monster_data(ctx, m).monsterClass;
    let player = &ctx.players.Players[pnum];
    let mut ac = player.get_armor();
    if player.pDamAcFlags.has_any_of(ItemSpecialEffectHf::ACAgainstDemons) && mclass == MonsterClass::Demon {
        ac += 40;
    }
    if player.pDamAcFlags.has_any_of(ItemSpecialEffectHf::ACAgainstUndead) && mclass == MonsterClass::Undead {
        ac += 20;
    }
    hit += 2 * (mlevel - player._pLevel as i32) + 30 - ac;
    let minhit = get_min_hit(ctx);
    hit = hit.max(minhit);
    let mut blkper = 100;
    let (pmode, block_flag) = (player._pmode, player._pBlockFlag);
    if (pmode == PM_STAND || pmode == PM_ATTACK) && block_flag {
        blkper = rnd(ctx, 100);
    }
    let player = &ctx.players.Players[pnum];
    let mut blk = player.get_block_chance(true) - (mlevel * 2);
    blk = blk.clamp(0, 100);
    if hper >= hit {
        return;
    }
    let is_me = Some(pnum) == ctx.players.MyPlayer;
    if blkper < blk {
        let dir = crate::engine::get_direction(player.position.tile, ctx.monster.Monsters[m].position.tile);
        crate::player::start_plr_block(ctx, pnum, dir);
        if is_me && ctx.players.Players[pnum].wReflections > 0 {
            let dam = rnd(ctx, ((max_dam - min_dam) << 6) + 1) + (min_dam << 6);
            let dam = (dam + (ctx.players.Players[pnum]._pIGetHit << 6)).max(64);
            check_reflect(ctx, m, pnum, dam);
        }
        return;
    }
    if monster_type_id(ctx, m) == MT_YZOMBIE && is_me {
        let player = &mut ctx.players.Players[pnum];
        if player._pMaxHP > 64 && player._pMaxHPBase > 64 {
            player._pMaxHP -= 64;
            if player._pHitPoints > player._pMaxHP {
                player._pHitPoints = player._pMaxHP;
            }
            player._pMaxHPBase -= 64;
            if player._pHPBase > player._pMaxHPBase {
                player._pHPBase = player._pMaxHPBase;
            }
        }
    }
    let mut dam = (min_dam << 6) + rnd(ctx, ((max_dam - min_dam) << 6) + 1);
    dam = (dam + (ctx.players.Players[pnum]._pIGetHit << 6)).max(64);
    if is_me {
        if ctx.players.Players[pnum].wReflections > 0 {
            let reflected_damage = check_reflect(ctx, m, pnum, dam);
            dam = (dam - reflected_damage).max(0);
        }
        crate::player::apply_plr_damage(ctx, DamageType::Physical, pnum, 0, 0, dam, DeathReason::MonsterOrTrap);
    }
    // Reflect can also kill a monster, so make sure the monster is still alive
    if ctx.players.Players[pnum]._pIFlags.has_any_of(ItemSpecialEffect::Thorns) && ctx.monster.Monsters[m].mode != MonsterMode::Death {
        let mdam = (rnd(ctx, 3) + 1) << 6;
        apply_monster_damage(ctx, DamageType::Physical, m, mdam);
        if ctx.monster.Monsters[m].hitPoints >> 6 <= 0 {
            m_start_kill(ctx, m, pnum);
        } else {
            m_start_hit(ctx, m, pnum, mdam);
        }
    }
    if (ctx.monster.Monsters[m].flags & MFLAG_NOLIFESTEAL as u32) == 0 && monster_type_id(ctx, m) == MT_SKING && ctx.init.gb_is_multiplayer {
        ctx.monster.Monsters[m].hitPoints += dam;
    }
    if ctx.players.Players[pnum]._pHitPoints >> 6 <= 0 {
        if ctx.init.gb_is_hellfire {
            let d = ctx.monster.Monsters[m].direction;
            m_start_stand(ctx, m, d);
        }
        return;
    }
    crate::player::start_plr_hit(ctx, pnum, dam, false);
    if (ctx.monster.Monsters[m].flags & MFLAG_KNOCKBACK as u32) != 0 {
        if ctx.players.Players[pnum]._pmode != PM_GOTHIT {
            crate::player::start_plr_hit(ctx, pnum, 0, true);
        }
        let new_position = ctx.players.Players[pnum].position.tile + ctx.monster.Monsters[m].direction;
        if crate::player::pos_ok_player(ctx, pnum, new_position) {
            ctx.players.Players[pnum].position.tile = new_position;
            let pdir = ctx.players.Players[pnum]._pdir;
            crate::player::fix_player_location(ctx, pnum, pdir);
            crate::player::fix_plr_walk_tags(ctx, pnum);
            ctx.gendung.dPlayer[new_position.x as usize][new_position.y as usize] = (pnum + 1) as i8;
            crate::player::set_player_old(ctx, pnum);
        }
    }
}

/// Original: `MonsterAttackEnemy` (monster.cpp).
// @port monster.cpp|devilution::MonsterAttackEnemy(Monster &monster, int hit, int minDam, int maxDam) sha=34228de71f8d
fn monster_attack_enemy(ctx: &mut Ctx, m: usize, hit: i32, min_dam: i32, max_dam: i32) {
    let enemy = ctx.monster.Monsters[m].enemy as usize;
    if (ctx.monster.Monsters[m].flags & MFLAG_TARGETS_MONSTER as u32) != 0 {
        monster_attack_monster(ctx, m, enemy, hit, min_dam, max_dam);
    } else {
        monster_attack_player(ctx, m, enemy, hit, min_dam, max_dam);
    }
}

/// Original: `MonsterAttack` (monster.cpp).
// @port monster.cpp|devilution::MonsterAttack(Monster &monster) sha=3ec44dad9d5a
fn monster_attack(ctx: &mut Ctx, m: usize) -> bool {
    let frame = ctx.monster.Monsters[m].animInfo.currentFrame as i32;
    let (to_hit, min_d, max_d, ai) = {
        let mon = &ctx.monster.Monsters[m];
        (mon.toHit as i32, mon.minDamage as i32, mon.maxDamage as i32, mon.ai)
    };
    if frame == monster_data(ctx, m).animFrameNum as i32 - 1 {
        monster_attack_enemy(ctx, m, to_hit, min_d, max_d);
        if ai != MonsterAIID::Snake {
            play_effect(ctx, m, MonsterSound::Attack);
        }
    }
    let t = monster_type_id(ctx, m);
    let frame = ctx.monster.Monsters[m].animInfo.currentFrame as i32;
    if matches!(t, MT_NMAGMA | MT_YMAGMA | MT_BMAGMA | MT_WMAGMA) && frame == 8 {
        monster_attack_enemy(ctx, m, to_hit + 10, min_d - 2, max_d - 2);
        play_effect(ctx, m, MonsterSound::Attack);
    }
    let frame = ctx.monster.Monsters[m].animInfo.currentFrame as i32;
    if matches!(t, MT_STORM | MT_RSTORM | MT_STORML | MT_MAEL) && frame == 12 {
        monster_attack_enemy(ctx, m, to_hit - 20, min_d + 4, max_d + 4);
        play_effect(ctx, m, MonsterSound::Attack);
    }
    if ai == MonsterAIID::Snake && ctx.monster.Monsters[m].animInfo.currentFrame == 0 {
        play_effect(ctx, m, MonsterSound::Attack);
    }
    if ctx.monster.Monsters[m].animInfo.is_last_frame() {
        let d = ctx.monster.Monsters[m].direction;
        m_start_stand(ctx, m, d);
        return true;
    }
    false
}


// ---------------------------------------------------------------------------------------------
// monster.cpp, part 2: mode handlers and the first AI routines.
// ---------------------------------------------------------------------------------------------

/// Original: `MonsterRangedAttack` (monster.cpp).
// @port monster.cpp|devilution::MonsterRangedAttack(Monster &monster) sha=9feaaa115c30
fn monster_ranged_attack(ctx: &mut Ctx, m: usize) -> bool {
    if ctx.monster.Monsters[m].animInfo.currentFrame as i32 == monster_data(ctx, m).animFrameNum as i32 - 1 {
        let mon = ctx.monster.Monsters[m].clone();
        let missile_type = MissileID::from_repr(mon.var1 as i8).unwrap_or(MissileID::Null);
        if missile_type != MissileID::Null {
            let multimissiles = if missile_type == MissileID::ChargedBolt { 3 } else { 1 };
            for _ in 0..multimissiles {
                crate::missiles::add_missile(ctx, mon.position.tile, mon.enemyPosition, mon.direction, missile_type, TARGET_PLAYERS, m as i32, mon.var2 as i32, 0, None);
            }
        }
        play_effect(ctx, m, MonsterSound::Attack);
    }
    if ctx.monster.Monsters[m].animInfo.is_last_frame() {
        let d = ctx.monster.Monsters[m].direction;
        m_start_stand(ctx, m, d);
        return true;
    }
    false
}

/// Original: `MonsterRangedSpecialAttack` (monster.cpp).
// @port monster.cpp|devilution::MonsterRangedSpecialAttack(Monster &monster) sha=f6a46d32524f
fn monster_ranged_special_attack(ctx: &mut Ctx, m: usize) -> bool {
    let special = monster_data(ctx, m).animFrameNumSpecial as i32;
    let mon = ctx.monster.Monsters[m].clone();
    if mon.animInfo.currentFrame as i32 == special - 1 && mon.animInfo.tickCounterOfCurrentFrame == 0 && (mon.ai != MonsterAIID::Mega || mon.var2 == 0) {
        let missile_type = MissileID::from_repr(mon.var1 as i8).unwrap_or(MissileID::Null);
        if crate::missiles::add_missile(ctx, mon.position.tile, mon.enemyPosition, mon.direction, missile_type, TARGET_PLAYERS, m as i32, mon.var3 as i32, 0, None).is_some() {
            play_effect(ctx, m, MonsterSound::Special);
        }
    }
    {
        let monster = &mut ctx.monster.Monsters[m];
        if monster.ai == MonsterAIID::Mega && monster.animInfo.currentFrame as i32 == special - 1 {
            let v = monster.var2;
            monster.var2 += 1;
            if v == 0 {
                monster.flags |= MFLAG_ALLOW_SPECIAL as u32;
            } else if monster.var2 == 15 {
                monster.flags &= !(MFLAG_ALLOW_SPECIAL as u32);
            }
        }
    }
    if ctx.monster.Monsters[m].animInfo.is_last_frame() {
        let d = ctx.monster.Monsters[m].direction;
        m_start_stand(ctx, m, d);
        return true;
    }
    false
}

/// Original: `MonsterSpecialAttack` (monster.cpp).
// @port monster.cpp|devilution::MonsterSpecialAttack(Monster &monster) sha=02e55b4dc90e
fn monster_special_attack(ctx: &mut Ctx, m: usize) -> bool {
    if ctx.monster.Monsters[m].animInfo.currentFrame as i32 == monster_data(ctx, m).animFrameNumSpecial as i32 - 1 {
        let hit = to_hit_special(ctx, m, ctx.multi.sgGameInitInfo.nDifficulty) as i32;
        let (mind, maxd) = (ctx.monster.Monsters[m].minDamageSpecial as i32, ctx.monster.Monsters[m].maxDamageSpecial as i32);
        monster_attack_enemy(ctx, m, hit, mind, maxd);
    }
    if ctx.monster.Monsters[m].animInfo.is_last_frame() {
        let d = ctx.monster.Monsters[m].direction;
        m_start_stand(ctx, m, d);
        return true;
    }
    false
}

/// Whether a fade animation reached its end (shared by `MonsterFadein` and `MonsterFadeout`).
fn fade_reached_end(monster: &Monster) -> bool {
    let lock = (monster.flags & MFLAG_LOCK_ANIMATION as u32) != 0;
    !((!lock || monster.animInfo.currentFrame != 0) && (lock || monster.animInfo.currentFrame != monster.animInfo.numberOfFrames - 1))
}

/// Original: `MonsterFadein` (monster.cpp).
// @port monster.cpp|devilution::MonsterFadein(Monster &monster) sha=5d8a749cf038
fn monster_fadein(ctx: &mut Ctx, m: usize) -> bool {
    if !fade_reached_end(&ctx.monster.Monsters[m]) {
        return false;
    }
    let d = ctx.monster.Monsters[m].direction;
    m_start_stand(ctx, m, d);
    ctx.monster.Monsters[m].flags &= !(MFLAG_LOCK_ANIMATION as u32);
    true
}

/// Original: `MonsterFadeout` (monster.cpp).
// @port monster.cpp|devilution::MonsterFadeout(Monster &monster) sha=d516fe5a07cf
fn monster_fadeout(ctx: &mut Ctx, m: usize) -> bool {
    if !fade_reached_end(&ctx.monster.Monsters[m]) {
        return false;
    }
    {
        let monster = &mut ctx.monster.Monsters[m];
        monster.flags &= !(MFLAG_LOCK_ANIMATION as u32);
        monster.flags |= MFLAG_HIDDEN as u32;
    }
    let d = ctx.monster.Monsters[m].direction;
    m_start_stand(ctx, m, d);
    true
}

/// Original: `MonsterHeal` (monster.cpp): applies the healing effect started by `StartHeal`.
// @port monster.cpp|devilution::MonsterHeal(Monster &monster) sha=2591fe1ae1ec
fn monster_heal(ctx: &mut Ctx, m: usize) {
    let monster = &mut ctx.monster.Monsters[m];
    if monster.animInfo.currentFrame == 0 {
        monster.flags &= !(MFLAG_LOCK_ANIMATION as u32);
        monster.flags |= MFLAG_ALLOW_SPECIAL as u32;
        if monster.var1 as i32 + monster.hitPoints < monster.maxHitPoints {
            monster.hitPoints += monster.var1 as i32;
        } else {
            monster.hitPoints = monster.maxHitPoints;
            monster.flags &= !(MFLAG_ALLOW_SPECIAL as u32);
            monster.mode = MonsterMode::SpecialMeleeAttack;
        }
    }
}

/// Original: `MonsterTalk` (monster.cpp).
// @port monster.cpp|devilution::MonsterTalk(Monster &monster) sha=1fc73b7b8e87
fn monster_talk(ctx: &mut Ctx, m: usize) {
    let d = ctx.monster.Monsters[m].direction;
    m_start_stand(ctx, m, d);
    ctx.monster.Monsters[m].goal = MonsterGoal::Talking;
    let talk_msg = ctx.monster.Monsters[m].talkMsg;
    if crate::effects::effect_is_playing(ctx, crate::tables::textdat::Speeches[talk_msg as usize].sfxnr as i32) {
        return;
    }
    crate::minitext::init_q_text_msg(ctx, talk_msg);
    let unique = ctx.monster.Monsters[m].uniqueType;
    if unique == UniqueMonsterType::SnotSpill {
        if talk_msg == TEXT_BANNER10 && (ctx.monster.Monsters[m].flags & MFLAG_QUEST_COMPLETE as u32) == 0 {
            let sp = ctx.gendung.SetPiece;
            crate::objects::obj_change_map(ctx, sp.position.x, sp.position.y, sp.position.x + (sp.size.width / 2) + 2, sp.position.y + (sp.size.height / 2) - 2);
            let tren = ctx.gendung.TransVal;
            ctx.gendung.TransVal = 9;
            crate::levels::gendung::drlg_m_rect_trans(ctx, crate::engine::geometry::Rectangle::new(sp.position, crate::engine::geometry::Size::new(sp.size.width / 2 + 4, sp.size.height / 2)));
            ctx.gendung.TransVal = tren;
            ctx.quests.Quests[Q_LTBANNER as usize]._qvar1 = 2;
            if ctx.quests.Quests[Q_LTBANNER as usize]._qactive == QUEST_INIT {
                ctx.quests.Quests[Q_LTBANNER as usize]._qactive = QUEST_ACTIVE;
            }
            ctx.monster.Monsters[m].flags |= MFLAG_QUEST_COMPLETE as u32;
            crate::msg::net_send_cmd_quest(ctx, true, Q_LTBANNER as usize);
        }
        if ctx.quests.Quests[Q_LTBANNER as usize]._qvar1 < 2 {
            let flags = ctx.monster.Monsters[m].flags;
            crate::appfat::app_fatal(ctx, &format!("SS Talk = {}, Flags = {}", talk_msg as i32, flags));
        }
    }
    if unique == UniqueMonsterType::Lachdan && talk_msg == TEXT_VEIL9 {
        ctx.quests.Quests[Q_VEIL as usize]._qactive = QUEST_ACTIVE;
        ctx.quests.Quests[Q_VEIL as usize]._qlog = true;
        crate::msg::net_send_cmd_quest(ctx, true, Q_VEIL as usize);
    }
    if unique == UniqueMonsterType::WarlordOfBlood {
        ctx.quests.Quests[Q_WARLORD as usize]._qvar1 = QS_WARLORD_TALKING as u8;
        crate::msg::net_send_cmd_quest(ctx, true, Q_WARLORD as usize);
    }
    if unique == UniqueMonsterType::Lazarus && crate::quests::use_multiplayer_quests(ctx) {
        ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 = 6;
        let monster = &mut ctx.monster.Monsters[m];
        monster.goal = MonsterGoal::Normal;
        monster.activeForTicks = u8::MAX;
        monster.talkMsg = TEXT_NONE;
    }
}

/// Original: `MonsterGotHit` (monster.cpp).
// @port monster.cpp|devilution::MonsterGotHit(Monster &monster) sha=910d0aee7e9c
fn monster_got_hit(ctx: &mut Ctx, m: usize) -> bool {
    if ctx.monster.Monsters[m].animInfo.is_last_frame() {
        let d = ctx.monster.Monsters[m].direction;
        m_start_stand(ctx, m, d);
        return true;
    }
    false
}

/// Original: `ReleaseMinions` (monster.cpp).
// @port monster.cpp|devilution::ReleaseMinions(const Monster &leader) sha=188e31418a42
fn release_minions(ctx: &mut Ctx, leader: usize) {
    for i in 0..ctx.monster.ActiveMonsterCount {
        let mi = ctx.monster.ActiveMonsters[i] as usize;
        if ctx.monster.Monsters[mi].leaderRelation == LeaderRelation::Leashed && get_leader(ctx, mi) == Some(leader) {
            set_leader(ctx, mi, None);
        }
    }
}

/// Original: `ShrinkLeaderPacksize` (monster.cpp).
// @port monster.cpp|devilution::ShrinkLeaderPacksize(const Monster &monster) sha=ce48c5594dfc
fn shrink_leader_packsize(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].leaderRelation == LeaderRelation::Leashed {
        let l = get_leader(ctx, m).expect("leader");
        ctx.monster.Monsters[l].packSize = ctx.monster.Monsters[l].packSize.wrapping_sub(1);
    }
}

/// Original: `MonsterDeath(Monster &)` (monster.cpp): the death mode handler.
// @port monster.cpp|devilution::MonsterDeath(Monster &monster) sha=74534602a765
fn monster_death_mode(ctx: &mut Ctx, m: usize) {
    ctx.monster.Monsters[m].var1 += 1;
    if monster_type_id(ctx, m) == MT_DIABLO {
        let tile = ctx.monster.Monsters[m].position.tile;
        let view = &mut ctx.gendung.ViewPosition;
        if tile.x < view.x {
            view.x -= 1;
        } else if tile.x > view.x {
            view.x += 1;
        }
        if tile.y < view.y {
            view.y -= 1;
        } else if tile.y > view.y {
            view.y += 1;
        }
        if ctx.monster.Monsters[m].var1 == 140 {
            prep_do_ending(ctx);
        }
    } else if ctx.monster.Monsters[m].animInfo.is_last_frame() {
        let mon = &ctx.monster.Monsters[m];
        let corpse = if mon.is_unique() { mon.corpseId } else { monster_type(ctx, m).corpseId };
        let (tile, dir) = (mon.position.tile, mon.direction);
        crate::dead::add_corpse(ctx, tile, corpse, dir);
        ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = 0;
        ctx.monster.Monsters[m].isInvalid = true;
        m_update_relations(ctx, m);
    }
}

/// Original: `MonsterSpecialStand` (monster.cpp).
// @port monster.cpp|devilution::MonsterSpecialStand(Monster &monster) sha=d8a2fed4fcd2
fn monster_special_stand(ctx: &mut Ctx, m: usize) -> bool {
    if ctx.monster.Monsters[m].animInfo.currentFrame as i32 == monster_data(ctx, m).animFrameNumSpecial as i32 - 1 {
        play_effect(ctx, m, MonsterSound::Special);
    }
    if ctx.monster.Monsters[m].animInfo.is_last_frame() {
        let d = ctx.monster.Monsters[m].direction;
        m_start_stand(ctx, m, d);
        return true;
    }
    false
}

/// Original: `MonsterDelay` (monster.cpp).
// @port monster.cpp|devilution::MonsterDelay(Monster &monster) sha=7755eb6e6314
fn monster_delay(ctx: &mut Ctx, m: usize) -> bool {
    let md = get_monster_direction(ctx, m);
    change_animation_data_dir(ctx, m, MonsterGraphic::Stand, md);
    {
        let monster = &mut ctx.monster.Monsters[m];
        if monster.ai == MonsterAIID::Lazarus && (monster.var2 > 8 || monster.var2 < 0) {
            monster.var2 = 8;
        }
    }
    let v = ctx.monster.Monsters[m].var2;
    ctx.monster.Monsters[m].var2 -= 1;
    if v == 0 {
        let o_frame = ctx.monster.Monsters[m].animInfo.currentFrame;
        let d = ctx.monster.Monsters[m].direction;
        m_start_stand(ctx, m, d);
        ctx.monster.Monsters[m].animInfo.currentFrame = o_frame;
        return true;
    }
    false
}

/// Original: `MonsterPetrified` (monster.cpp).
// @port monster.cpp|devilution::MonsterPetrified(Monster &monster) sha=ca9c7cf5422f
fn monster_petrified(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].hitPoints <= 0 {
        let t = ctx.monster.Monsters[m].position.tile;
        ctx.gendung.dMonster[t.x as usize][t.y as usize] = 0;
        ctx.monster.Monsters[m].isInvalid = true;
    }
}

/// Original: `AddSkeleton` (monster.cpp).
// @port monster.cpp|devilution::AddSkeleton(Point position, Direction dir, bool inMap) sha=7163a7424b81
fn add_skeleton(ctx: &mut Ctx, position: Point, dir: Direction, in_map: bool) -> Option<usize> {
    let mut skeleton_indexes: Vec<usize> = Vec::with_capacity(SKELETON_TYPES.len());
    for i in 0..ctx.monster.LevelMonsterTypeCount {
        if is_skel(ctx.monster.LevelMonsterTypes[i].type_) {
            skeleton_indexes.push(i);
        }
    }
    if skeleton_indexes.is_empty() {
        return None;
    }
    let type_index = skeleton_indexes[rnd(ctx, skeleton_indexes.len() as i32) as usize];
    add_monster(ctx, position, dir, type_index, in_map)
}

/// Original: `SpawnSkeleton` (monster.cpp).
// @port monster.cpp|devilution::SpawnSkeleton(Point position, Direction dir) sha=468129c5c050
fn spawn_skeleton(ctx: &mut Ctx, position: Point, dir: Direction) {
    if let Some(skeleton) = add_skeleton(ctx, position, dir, true) {
        start_special_stand(ctx, skeleton, dir);
    }
}

/// Original: `IsLineNotSolid` (monster.cpp).
// @port monster.cpp|devilution::IsLineNotSolid(Point startPoint, Point endPoint) sha=a39b7c426cd9
fn is_line_not_solid(ctx: &Ctx, start_point: Point, end_point: Point) -> bool {
    line_clear(ctx, &|ctx: &Ctx, p: Point| crate::engine::path::is_tile_not_solid(ctx, p), start_point, end_point)
}

/// Original: `FollowTheLeader` (monster.cpp).
// @port monster.cpp|devilution::FollowTheLeader(Monster &monster) sha=184b8905d4d6
fn follow_the_leader(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].leaderRelation != LeaderRelation::Leashed {
        return;
    }
    let Some(leader) = get_leader(ctx, m) else {
        return;
    };
    let (l_active, l_tile) = (ctx.monster.Monsters[leader].activeForTicks, ctx.monster.Monsters[leader].position.tile);
    let monster = &mut ctx.monster.Monsters[m];
    if monster.activeForTicks >= l_active {
        return;
    }
    monster.position.last = l_tile;
    monster.activeForTicks = l_active - 1;
}

/// Original: `GroupUnity` (monster.cpp).
// @port monster.cpp|devilution::GroupUnity(Monster &monster) sha=ef9cad4c452a
fn group_unity(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].leaderRelation == LeaderRelation::None {
        return;
    }
    // No unique monster would be a minion of someone else!
    assert!(!ctx.monster.Monsters[m].is_unique());
    // Someone with a leaderRelation should have a leader, if we end up trying to access a nullptr then the relation was already broken...
    let leader = get_leader(ctx, m).expect("leader");
    let tile = ctx.monster.Monsters[m].position.tile;
    let lfuture = ctx.monster.Monsters[leader].position.future;
    if is_line_not_solid(ctx, tile, lfuture) {
        if ctx.monster.Monsters[m].leaderRelation == LeaderRelation::Separated && tile.walking_distance(lfuture) < 4 {
            // Reunite the separated monster with the pack
            ctx.monster.Monsters[leader].packSize += 1;
            ctx.monster.Monsters[m].leaderRelation = LeaderRelation::Leashed;
        }
    } else if ctx.monster.Monsters[m].leaderRelation == LeaderRelation::Leashed {
        ctx.monster.Monsters[leader].packSize = ctx.monster.Monsters[leader].packSize.wrapping_sub(1);
        ctx.monster.Monsters[m].leaderRelation = LeaderRelation::Separated;
    }
    if ctx.monster.Monsters[m].leaderRelation == LeaderRelation::Leashed {
        let active = ctx.monster.Monsters[m].activeForTicks;
        let l = &mut ctx.monster.Monsters[leader];
        if active > l.activeForTicks {
            l.position.last = tile;
            l.activeForTicks = active - 1;
        }
        if l.ai == MonsterAIID::Gargoyle && (l.flags & MFLAG_ALLOW_SPECIAL as u32) != 0 {
            l.flags &= !(MFLAG_ALLOW_SPECIAL as u32);
            l.mode = MonsterMode::SpecialMeleeAttack;
        }
    }
}

/// Original: `RandomWalk` (monster.cpp).
// @port monster.cpp|devilution::RandomWalk(Monster &monster, Direction md) sha=01e2b8629a10
fn random_walk(ctx: &mut Ctx, m: usize, md0: Direction) -> bool {
    let mdtemp = md0;
    let mut md = md0;
    let mut ok = dir_ok(ctx, m, md);
    if ctx.rng.flip_coin(2) {
        ok = ok || {
            md = right(mdtemp);
            dir_ok(ctx, m, md)
        } || {
            md = left(mdtemp);
            dir_ok(ctx, m, md)
        };
    } else {
        ok = ok || {
            md = left(mdtemp);
            dir_ok(ctx, m, md)
        } || {
            md = right(mdtemp);
            dir_ok(ctx, m, md)
        };
    }
    if ctx.rng.flip_coin(2) {
        ok = ok || {
            md = left(left(mdtemp));
            dir_ok(ctx, m, md)
        } || {
            md = right(right(mdtemp));
            dir_ok(ctx, m, md)
        };
    } else {
        ok = ok || {
            md = right(right(mdtemp));
            dir_ok(ctx, m, md)
        } || {
            md = left(left(mdtemp));
            dir_ok(ctx, m, md)
        };
    }
    if ok {
        walk(ctx, m, md);
    }
    ok
}

/// Original: `RandomWalk2` (monster.cpp).
// @port monster.cpp|devilution::RandomWalk2(Monster &monster, Direction md) sha=521c2c037ad9
fn random_walk2(ctx: &mut Ctx, m: usize, md: Direction) -> bool {
    let mut mdtemp = md;
    let mut ok = dir_ok(ctx, m, md); // Can we continue in the same direction
    // Randomly go left or right
    if ctx.rng.flip_coin(2) {
        ok = ok || {
            mdtemp = right(md);
            dir_ok(ctx, m, right(md))
        } || {
            mdtemp = left(md);
            dir_ok(ctx, m, left(md))
        };
    } else {
        ok = ok || {
            mdtemp = left(md);
            dir_ok(ctx, m, left(md))
        } || {
            mdtemp = right(md);
            dir_ok(ctx, m, right(md))
        };
    }
    if ok {
        walk(ctx, m, mdtemp);
    }
    ok
}

/// Original: `IsTileSafe` (monster.cpp): whether a tile is not affected by a spell we are vulnerable to.
// @port monster.cpp|devilution::IsTileSafe(const Monster &monster, Point position) sha=ef170d6fdbdb
fn is_tile_safe(ctx: &Ctx, m: usize, position: Point) -> bool {
    if !in_dungeon_bounds(position) {
        return false;
    }
    let monster = &ctx.monster.Monsters[m];
    let is_diablo = monster_type_id(ctx, m) == MT_DIABLO;
    let fears_fire = (monster.resistance & IMMUNE_FIRE as u16) == 0 || is_diablo;
    let fears_lightning = (monster.resistance & IMMUNE_LIGHTNING as u16) == 0 || is_diablo;
    let f = ctx.gendung.dFlags[position.x as usize][position.y as usize];
    !(fears_fire && f.has_any_of(DungeonFlag::MissileFireWall)) && !(fears_lightning && f.has_any_of(DungeonFlag::MissileLightningWall))
}

/// Original: `IsTileAvailable(Point)` (monster.cpp): whether the given tile is not currently blocked.
// @port monster.cpp|devilution::IsTileAvailable(Point position) sha=60f6068f485a
fn is_tile_available(ctx: &Ctx, position: Point) -> bool {
    if ctx.gendung.dPlayer[position.x as usize][position.y as usize] != 0 || ctx.gendung.dMonster[position.x as usize][position.y as usize] != 0 {
        return false;
    }
    if !crate::engine::path::is_tile_walkable(ctx, position, false) {
        return false;
    }
    true
}

/// Original: `IsTileAccessible` (monster.cpp): whether a monster can access the given tile (possibly by opening a door).
// @port monster.cpp|devilution::IsTileAccessible(const Monster &monster, Point position) sha=9b55753aaba8
fn is_tile_accessible(ctx: &Ctx, m: usize, position: Point) -> bool {
    if ctx.gendung.dPlayer[position.x as usize][position.y as usize] != 0 || ctx.gendung.dMonster[position.x as usize][position.y as usize] != 0 {
        return false;
    }
    if !crate::engine::path::is_tile_walkable(ctx, position, (ctx.monster.Monsters[m].flags & MFLAG_CAN_OPEN_DOOR as u32) != 0) {
        return false;
    }
    is_tile_safe(ctx, m, position)
}

/// Original: `AiPlanWalk` (monster.cpp).
// @port monster.cpp|devilution::AiPlanWalk(Monster &monster) sha=09e5bd3d7403
fn ai_plan_walk(ctx: &mut Ctx, m: usize) -> bool {
    let mut path = [0i8; crate::engine::path::MaxPathLength];
    // Maps from walking path step to facing direction.
    const PLR2MONST: [Direction; 9] = [
        Direction::South,
        Direction::NorthEast,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::SouthWest,
        Direction::North,
        Direction::East,
        Direction::South,
        Direction::West,
    ];
    let (start, dest) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
    if crate::engine::path::find_path(ctx, &|ctx: &Ctx, p: Point| is_tile_accessible(ctx, m, p), start, dest, &mut path) == 0 {
        return false;
    }
    random_walk(ctx, m, PLR2MONST[path[0] as usize]);
    true
}

/// Original: `Turn` (monster.cpp).
// @port monster.cpp|devilution::Turn(Direction direction, bool turnLeft) sha=d29116210c26
fn turn(direction: Direction, turn_left: bool) -> Direction {
    if turn_left {
        left(direction)
    } else {
        right(direction)
    }
}

/// Original: `RoundWalk` (monster.cpp); `dir` is `monster.goalVar2`.
// @port monster.cpp|devilution::RoundWalk(Monster &monster, Direction direction, int8_t *dir) sha=e67ee9744330
fn round_walk(ctx: &mut Ctx, m: usize, direction: Direction) -> bool {
    let dir = ctx.monster.Monsters[m].goalVar2;
    let turn45deg = turn(direction, dir != 0);
    let turn90deg = turn(turn45deg, dir != 0);
    // Turn 90 degrees
    if walk(ctx, m, turn90deg) {
        return true;
    }
    // Only do a small turn
    if walk(ctx, m, turn45deg) {
        return true;
    }
    // Continue straight
    if walk(ctx, m, direction) {
        return true;
    }
    // Try 90 degrees in the opposite than desired direction
    let g = &mut ctx.monster.Monsters[m].goalVar2;
    *g = if *g == 0 { 1 } else { 0 };
    random_walk(ctx, m, opposite(turn90deg))
}

/// Original: `AiPlanPath` (monster.cpp).
// @port monster.cpp|devilution::AiPlanPath(Monster &monster) sha=bd9fa6a76315
fn ai_plan_path(ctx: &mut Ctx, m: usize) -> bool {
    let is_golem = monster_type_id(ctx, m) == MT_GOLEM;
    if !is_golem {
        let mon = &ctx.monster.Monsters[m];
        if mon.activeForTicks == 0 {
            return false;
        }
        if mon.mode != MonsterMode::Stand {
            return false;
        }
        if !matches!(mon.goal, MonsterGoal::Normal | MonsterGoal::Move | MonsterGoal::Attack) {
            return false;
        }
        if mon.position.tile == GOLEM_HOLDING_CELL {
            return false;
        }
    }
    let (start, dest) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
    let clear = line_clear(ctx, &|ctx: &Ctx, p: Point| is_tile_available_for_monster(ctx, m, p), start, dest);
    let path_count = ctx.monster.Monsters[m].pathCount;
    if !clear || (5..8).contains(&path_count) {
        if (ctx.monster.Monsters[m].flags & MFLAG_CAN_OPEN_DOOR as u32) != 0 {
            crate::objects::monst_check_doors(ctx, m);
        }
        ctx.monster.Monsters[m].pathCount += 1;
        if ctx.monster.Monsters[m].pathCount < 5 {
            return false;
        }
        if ai_plan_walk(ctx, m) {
            return true;
        }
    }
    if !is_golem {
        ctx.monster.Monsters[m].pathCount = 0;
    }
    false
}

/// `monster.var1` interpreted as a `MonsterMode` (the previous mode).
fn var1_mode(monster: &Monster) -> MonsterMode {
    MonsterMode::from_repr(monster.var1 as u8).unwrap_or(MonsterMode::Stand)
}

/// `dTransVal` of the monster's tile equals that of its enemy's position.
fn same_room_as_enemy(ctx: &Ctx, m: usize) -> bool {
    let mon = &ctx.monster.Monsters[m];
    ctx.gendung.dTransVal[mon.position.tile.x as usize][mon.position.tile.y as usize] == ctx.gendung.dTransVal[mon.enemyPosition.x as usize][mon.enemyPosition.y as usize]
}

/// `monster.goalVar1++ >= limit` (post-increment comparison).
fn goal_var1_post_inc_ge(ctx: &mut Ctx, m: usize, limit: i32) -> bool {
    let g = &mut ctx.monster.Monsters[m].goalVar1;
    let v = *g;
    *g += 1;
    v as i32 >= limit
}

/// Original: `AiAvoidance` (monster.cpp).
// @port monster.cpp|devilution::AiAvoidance(Monster &monster) sha=1787b9170300
fn ai_avoidance(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    if ctx.monster.Monsters[m].activeForTicks < u8::MAX {
        crate::objects::monst_check_doors(ctx, m);
    }
    let v = rnd(ctx, 100);
    let distance_to_enemy = distance_to_enemy(ctx, m);
    if distance_to_enemy >= 2 && ctx.monster.Monsters[m].activeForTicks == u8::MAX && same_room_as_enemy(ctx, m) {
        if ctx.monster.Monsters[m].goal == MonsterGoal::Move || (distance_to_enemy >= 4 && ctx.rng.flip_coin(4)) {
            if ctx.monster.Monsters[m].goal != MonsterGoal::Move {
                let r = rnd(ctx, 2) as i8;
                ctx.monster.Monsters[m].goalVar1 = 0;
                ctx.monster.Monsters[m].goalVar2 = r;
            }
            ctx.monster.Monsters[m].goal = MonsterGoal::Move;
            if (goal_var1_post_inc_ge(ctx, m, 2 * distance_to_enemy as i32) && dir_ok(ctx, m, md)) || !same_room_as_enemy(ctx, m) {
                ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
            } else if !round_walk(ctx, m, md) {
                let r = rnd(ctx, 10) + 10;
                ai_delay(ctx, m, r);
            }
        }
    } else {
        ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Normal {
        let mon = ctx.monster.Monsters[m].clone();
        let intel = mon.intelligence as i32;
        if distance_to_enemy >= 2 {
            if (mon.var2 > 20 && v < 2 * intel + 28) || (is_monster_mode_move(var1_mode(&mon)) && mon.var2 == 0 && v < 2 * intel + 78) {
                random_walk(ctx, m, md);
            }
        } else if v < 2 * intel + 23 {
            ctx.monster.Monsters[m].direction = md;
            if matches!(mon.ai, MonsterAIID::GoatMelee | MonsterAIID::Gharbad) && mon.hitPoints < (mon.maxHitPoints / 2) && !ctx.rng.flip_coin(2) {
                start_special_attack(ctx, m);
            } else {
                start_attack(ctx, m);
            }
        }
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `GetMissileType` (monster.cpp).
// @port monster.cpp|devilution::GetMissileType(MonsterAIID ai) sha=f8ba49f2076f
fn get_missile_type(ai: MonsterAIID) -> MissileID {
    match ai {
        MonsterAIID::GoatMelee => MissileID::Arrow,
        MonsterAIID::Succubus | MonsterAIID::LazarusSuccubus => MissileID::BloodStar,
        MonsterAIID::Acid | MonsterAIID::AcidUnique => MissileID::Acid,
        MonsterAIID::FireBat => MissileID::Firebolt,
        MonsterAIID::Torchant => MissileID::Fireball,
        MonsterAIID::Lich => MissileID::OrangeFlare,
        MonsterAIID::ArchLich => MissileID::YellowFlare,
        MonsterAIID::Psychorb => MissileID::BlueFlare,
        MonsterAIID::Necromorb => MissileID::RedFlare,
        MonsterAIID::Magma => MissileID::MagmaBall,
        MonsterAIID::Storm => MissileID::ThinLightningControl,
        MonsterAIID::Diablo => MissileID::DiabloApocalypse,
        MonsterAIID::BoneDemon => MissileID::BlueFlare2,
        _ => MissileID::Arrow,
    }
}

/// Original: `AiRanged` (monster.cpp).
// @port monster.cpp|devilution::AiRanged(Monster &monster) sha=185e693ae9dd
fn ai_ranged(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    let mon = ctx.monster.Monsters[m].clone();
    if mon.activeForTicks == u8::MAX || (mon.flags & MFLAG_TARGETS_MONSTER as u32) != 0 {
        let md = get_monster_direction(ctx, m);
        if mon.activeForTicks < u8::MAX {
            crate::objects::monst_check_doors(ctx, m);
        }
        ctx.monster.Monsters[m].direction = md;
        if var1_mode(&mon) == MonsterMode::RangedAttack {
            let r = rnd(ctx, 20);
            ai_delay(ctx, m, r);
        } else if distance_to_enemy(ctx, m) < 4 && rnd(ctx, 100) < 10 * (mon.intelligence as i32 + 7) {
            random_walk(ctx, m, opposite(md));
        }
        if ctx.monster.Monsters[m].mode == MonsterMode::Stand {
            let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
            if line_clear_missile(ctx, tile, enemy_pos) {
                let missile_type = get_missile_type(mon.ai);
                if mon.ai == MonsterAIID::AcidUnique {
                    start_ranged_special_attack(ctx, m, missile_type, 0);
                } else {
                    start_ranged_attack(ctx, m, missile_type, 0);
                }
            } else {
                check_stand_animation_is_loaded(ctx, m, md);
            }
        }
        return;
    }
    if mon.activeForTicks != 0 {
        let md = crate::engine::get_direction(mon.position.tile, mon.position.last);
        random_walk(ctx, m, md);
    }
}

/// Original: `AiRangedAvoidance` (monster.cpp).
// @port monster.cpp|devilution::AiRangedAvoidance(Monster &monster) sha=489427ed455d
fn ai_ranged_avoidance(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    let ai = ctx.monster.Monsters[m].ai;
    if matches!(ai, MonsterAIID::Magma | MonsterAIID::Storm | MonsterAIID::BoneDemon) && ctx.monster.Monsters[m].activeForTicks < u8::MAX {
        crate::objects::monst_check_doors(ctx, m);
    }
    let lessmissiles = if ai == MonsterAIID::Acid { 1 } else { 0 };
    let dam = if ai == MonsterAIID::Diablo { 40 } else { 0 };
    let missile_type = get_missile_type(ai);
    let mut v = rnd(ctx, 10000);
    let distance_to_enemy = distance_to_enemy(ctx, m);
    let intel = ctx.monster.Monsters[m].intelligence as i32;
    if distance_to_enemy >= 2 && ctx.monster.Monsters[m].activeForTicks == u8::MAX && same_room_as_enemy(ctx, m) {
        if ctx.monster.Monsters[m].goal == MonsterGoal::Move || (distance_to_enemy >= 3 && ctx.rng.flip_coin(4 << lessmissiles)) {
            if ctx.monster.Monsters[m].goal != MonsterGoal::Move {
                let r = rnd(ctx, 2) as i8;
                ctx.monster.Monsters[m].goalVar1 = 0;
                ctx.monster.Monsters[m].goalVar2 = r;
            }
            ctx.monster.Monsters[m].goal = MonsterGoal::Move;
            let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
            if goal_var1_post_inc_ge(ctx, m, 2 * distance_to_enemy as i32) && dir_ok(ctx, m, md) {
                ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
            } else if v < (500 * (intel + 1) >> lessmissiles) && line_clear_missile(ctx, tile, enemy_pos) {
                start_ranged_special_attack(ctx, m, missile_type, dam);
            } else {
                round_walk(ctx, m, md);
            }
        }
    } else {
        ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Normal {
        let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
        if ((distance_to_enemy >= 3 && v < ((500 * (intel + 2)) >> lessmissiles)) || v < ((500 * (intel + 1)) >> lessmissiles)) && line_clear_missile(ctx, tile, enemy_pos) {
            start_ranged_special_attack(ctx, m, missile_type, dam);
        } else if distance_to_enemy >= 2 {
            v = rnd(ctx, 100);
            let mon = &ctx.monster.Monsters[m];
            if v < 1000 * (intel + 5) || (is_monster_mode_move(var1_mode(mon)) && mon.var2 == 0 && v < 1000 * (intel + 8)) {
                random_walk(ctx, m, md);
            }
        } else if v < 1000 * (intel + 6) {
            ctx.monster.Monsters[m].direction = md;
            start_attack(ctx, m);
        }
    }
    if ctx.monster.Monsters[m].mode == MonsterMode::Stand {
        let r = rnd(ctx, 10) + 5;
        ai_delay(ctx, m, r);
    }
}

/// Original: `ZombieAi` (monster.cpp).
// @port monster.cpp|devilution::ZombieAi(Monster &monster) sha=9d5f4fd1b6e2
fn zombie_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    if !is_tile_visible(ctx, ctx.monster.Monsters[m].position.tile) {
        return;
    }
    let intel = ctx.monster.Monsters[m].intelligence as i32;
    if rnd(ctx, 100) < 2 * intel + 10 {
        let mon = &ctx.monster.Monsters[m];
        let dist = mon.enemyPosition.walking_distance(mon.position.tile);
        if dist >= 2 {
            if dist >= 2 * intel + 4 {
                let mut md = ctx.monster.Monsters[m].direction;
                if rnd(ctx, 100) < 2 * intel + 20 {
                    md = Direction::from_u8(rnd(ctx, 8) as u8);
                }
                walk(ctx, m, md);
            } else {
                let d = get_monster_direction(ctx, m);
                random_walk(ctx, m, d);
            }
        } else {
            start_attack(ctx, m);
        }
    }
    let d = ctx.monster.Monsters[m].direction;
    check_stand_animation_is_loaded(ctx, m, d);
}

/// Original: `OverlordAi` (monster.cpp).
// @port monster.cpp|devilution::OverlordAi(Monster &monster) sha=6a3c8f0e9cd2
fn overlord_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = get_monster_direction(ctx, m);
    ctx.monster.Monsters[m].direction = md;
    let v = rnd(ctx, 100);
    let mon = ctx.monster.Monsters[m].clone();
    let intel = mon.intelligence as i32;
    if distance_to_enemy(ctx, m) >= 2 {
        if (mon.var2 > 20 && v < 4 * intel + 20) || (is_monster_mode_move(var1_mode(&mon)) && mon.var2 == 0 && v < 4 * intel + 70) {
            random_walk(ctx, m, md);
        }
    } else if v < 4 * intel + 15 {
        start_attack(ctx, m);
    } else if v < 4 * intel + 20 {
        start_special_attack(ctx, m);
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `SkeletonAi` (monster.cpp).
// @port monster.cpp|devilution::SkeletonAi(Monster &monster) sha=e0579c8b1870
fn skeleton_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    ctx.monster.Monsters[m].direction = md;
    let mon = ctx.monster.Monsters[m].clone();
    let intel = mon.intelligence as i32;
    if distance_to_enemy(ctx, m) >= 2 {
        if var1_mode(&mon) == MonsterMode::Delay || rnd(ctx, 100) >= 35 - 4 * intel {
            random_walk(ctx, m, md);
        } else {
            let r = 15 - 2 * intel + rnd(ctx, 10);
            ai_delay(ctx, m, r);
        }
    } else if var1_mode(&mon) == MonsterMode::Delay || rnd(ctx, 100) < 2 * intel + 20 {
        start_attack(ctx, m);
    } else {
        let r = 2 * (5 - intel) + rnd(ctx, 10);
        ai_delay(ctx, m, r);
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `SkeletonBowAi` (monster.cpp).
// @port monster.cpp|devilution::SkeletonBowAi(Monster &monster) sha=13486e06f233
fn skeleton_bow_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = get_monster_direction(ctx, m);
    ctx.monster.Monsters[m].direction = md;
    let v = rnd(ctx, 100);
    let mut walking = false;
    let mon = ctx.monster.Monsters[m].clone();
    let intel = mon.intelligence as i32;
    if distance_to_enemy(ctx, m) < 4 && ((mon.var2 > 20 && v < 2 * intel + 13) || (is_monster_mode_move(var1_mode(&mon)) && mon.var2 == 0 && v < 2 * intel + 63)) {
        walking = walk(ctx, m, opposite(md));
    }
    if !walking && rnd(ctx, 100) < 2 * intel + 3 {
        let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
        if line_clear_missile(ctx, tile, enemy_pos) {
            start_ranged_attack(ctx, m, MissileID::Arrow, 4);
        }
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `ScavengerFindCorpse` (monster.cpp).
// @port monster.cpp|devilution::ScavengerFindCorpse(const Monster &scavenger) sha=3bbabba5a0b8
fn scavenger_find_corpse(ctx: &mut Ctx, m: usize) -> Option<Point> {
    let reverse_search = ctx.rng.flip_coin(2);
    let first = if reverse_search { 4 } else { -4 };
    let last = if reverse_search { -4 } else { 4 };
    let increment = if reverse_search { -1 } else { 1 };
    let tile = ctx.monster.Monsters[m].position.tile;
    let mut y = first;
    // As in the original, the loops test `<= last` also when searching in reverse (so a
    // reverse search finds nothing).
    while y <= last {
        let mut x = first;
        while x <= last {
            let position = tile + Displacement::new(x, y);
            x += increment;
            if !in_dungeon_bounds(position) {
                continue;
            }
            if ctx.gendung.dCorpse[position.x as usize][position.y as usize] == 0 {
                continue;
            }
            if !is_line_not_solid(ctx, tile, position) {
                continue;
            }
            return Some(position);
        }
        y += increment;
    }
    None
}

/// Original: `ScavengerAi` (monster.cpp).
// @port monster.cpp|devilution::ScavengerAi(Monster &monster) sha=c55e42de196b
fn scavenger_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    {
        let mon = &ctx.monster.Monsters[m];
        if mon.hitPoints < (mon.maxHitPoints / 2) && mon.goal != MonsterGoal::Healing {
            if mon.leaderRelation != LeaderRelation::None {
                shrink_leader_packsize(ctx, m);
                ctx.monster.Monsters[m].leaderRelation = LeaderRelation::None;
            }
            ctx.monster.Monsters[m].goal = MonsterGoal::Healing;
            ctx.monster.Monsters[m].goalVar3 = 10;
        }
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Healing && ctx.monster.Monsters[m].goalVar3 != 0 {
        ctx.monster.Monsters[m].goalVar3 -= 1;
        let tile = ctx.monster.Monsters[m].position.tile;
        if ctx.gendung.dCorpse[tile.x as usize][tile.y as usize] != 0 {
            start_eating(ctx, m);
            let hf = ctx.init.gb_is_hellfire;
            let mon = &mut ctx.monster.Monsters[m];
            if hf {
                let m_max_hp = mon.maxHitPoints;
                mon.hitPoints += m_max_hp / 8;
                if mon.hitPoints > mon.maxHitPoints {
                    mon.hitPoints = mon.maxHitPoints;
                }
                if mon.goalVar3 <= 0 || mon.hitPoints == mon.maxHitPoints {
                    ctx.gendung.dCorpse[tile.x as usize][tile.y as usize] = 0;
                }
            } else {
                mon.hitPoints += 64;
            }
            let mon = &mut ctx.monster.Monsters[m];
            let mut target_health = mon.maxHitPoints;
            if !hf {
                target_health = (mon.maxHitPoints / 2) + (mon.maxHitPoints / 4);
            }
            if mon.hitPoints >= target_health {
                mon.goal = MonsterGoal::Normal;
                mon.goalVar1 = 0;
                mon.goalVar2 = 0;
            }
        } else {
            if ctx.monster.Monsters[m].goalVar1 == 0 {
                if let Some(position) = scavenger_find_corpse(ctx, m) {
                    ctx.monster.Monsters[m].goalVar1 = (position.x + 1) as i16;
                    ctx.monster.Monsters[m].goalVar2 = (position.y + 1) as i8;
                }
            }
            if ctx.monster.Monsters[m].goalVar1 != 0 {
                let x = ctx.monster.Monsters[m].goalVar1 as i32 - 1;
                let y = ctx.monster.Monsters[m].goalVar2 as i32 - 1;
                let d = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, Point::new(x, y));
                ctx.monster.Monsters[m].direction = d;
                random_walk(ctx, m, d);
            }
        }
    }
    if ctx.monster.Monsters[m].mode == MonsterMode::Stand {
        skeleton_ai(ctx, m);
    }
}

/// Original: `RhinoAi` (monster.cpp).
// @port monster.cpp|devilution::RhinoAi(Monster &monster) sha=6c848aaf609f
fn rhino_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    if ctx.monster.Monsters[m].activeForTicks < u8::MAX {
        crate::objects::monst_check_doors(ctx, m);
    }
    let mut v = rnd(ctx, 100);
    let distance_to_enemy = distance_to_enemy(ctx, m);
    if distance_to_enemy >= 2 {
        if ctx.monster.Monsters[m].goal == MonsterGoal::Move || (distance_to_enemy >= 5 && !ctx.rng.flip_coin(4)) {
            if ctx.monster.Monsters[m].goal != MonsterGoal::Move {
                let r = rnd(ctx, 2) as i8;
                ctx.monster.Monsters[m].goalVar1 = 0;
                ctx.monster.Monsters[m].goalVar2 = r;
            }
            ctx.monster.Monsters[m].goal = MonsterGoal::Move;
            if goal_var1_post_inc_ge(ctx, m, 2 * distance_to_enemy as i32) || !same_room_as_enemy(ctx, m) {
                ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
            } else if !round_walk(ctx, m, md) {
                let r = rnd(ctx, 10) + 10;
                ai_delay(ctx, m, r);
            }
        }
    } else {
        ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Normal {
        let intel = ctx.monster.Monsters[m].intelligence as i32;
        let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
        if distance_to_enemy >= 5 && v < 2 * intel + 43 && line_clear(ctx, &|ctx: &Ctx, p: Point| is_tile_available_for_monster(ctx, m, p), tile, enemy_pos) {
            if crate::missiles::add_missile(ctx, tile, enemy_pos, md, MissileID::Rhino, TARGET_PLAYERS, m as i32, 0, 0, None).is_some() {
                if monster_data(ctx, m).hasSpecialSound {
                    play_effect(ctx, m, MonsterSound::Special);
                }
                ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = -((m + 1) as i16);
                ctx.monster.Monsters[m].mode = MonsterMode::Charge;
            }
        } else if distance_to_enemy >= 2 {
            v = rnd(ctx, 100);
            let mon = &ctx.monster.Monsters[m];
            if v >= 2 * intel + 33 && (!matches!(var1_mode(mon), MonsterMode::MoveNorthwards | MonsterMode::MoveSouthwards | MonsterMode::MoveSideways) || mon.var2 != 0 || v >= 2 * intel + 83) {
                let r = rnd(ctx, 10) + 10;
                ai_delay(ctx, m, r);
            } else {
                random_walk(ctx, m, md);
            }
        } else if v < 2 * intel + 28 {
            ctx.monster.Monsters[m].direction = md;
            start_attack(ctx, m);
        }
    }
    let d = ctx.monster.Monsters[m].direction;
    check_stand_animation_is_loaded(ctx, m, d);
}

/// Original: `FallenAi` (monster.cpp).
// @port monster.cpp|devilution::FallenAi(Monster &monster) sha=4b9cd2322e00
fn fallen_ai(ctx: &mut Ctx, m: usize) {
    {
        let mon = &mut ctx.monster.Monsters[m];
        if mon.goal == MonsterGoal::Attack {
            if mon.goalVar1 != 0 {
                mon.goalVar1 -= 1;
            } else {
                mon.goal = MonsterGoal::Normal;
            }
        }
        if mon.mode != MonsterMode::Stand || mon.activeForTicks == 0 {
            return;
        }
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Retreat {
        let v = ctx.monster.Monsters[m].goalVar1;
        ctx.monster.Monsters[m].goalVar1 -= 1;
        if v == 0 {
            ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
            let d = opposite(Direction::from_u8(ctx.monster.Monsters[m].goalVar2 as u8));
            m_start_stand(ctx, m, d);
        }
    }
    if ctx.monster.Monsters[m].animInfo.is_last_frame() {
        if !ctx.rng.flip_coin(4) {
            return;
        }
        let d = ctx.monster.Monsters[m].direction;
        start_special_stand(ctx, m, d);
        let intel = ctx.monster.Monsters[m].intelligence as i32;
        {
            let mon = &mut ctx.monster.Monsters[m];
            if mon.maxHitPoints - (2 * intel + 2) >= mon.hitPoints {
                mon.hitPoints += 2 * intel + 2;
            } else {
                mon.hitPoints = mon.maxHitPoints;
            }
        }
        let rad = 2 * intel + 4;
        let tile = ctx.monster.Monsters[m].position.tile;
        for y in -rad..=rad {
            for x in -rad..=rad {
                let xpos = tile.x + x;
                let ypos = tile.y + y;
                if in_dungeon_bounds(Point::new(xpos, ypos)) {
                    let mi = ctx.gendung.dMonster[xpos as usize][ypos as usize] as i32;
                    if mi <= 0 {
                        continue;
                    }
                    let other = &mut ctx.monster.Monsters[(mi - 1) as usize];
                    if other.ai != MonsterAIID::Fallen {
                        continue;
                    }
                    other.goal = MonsterGoal::Attack;
                    other.goalVar1 = (30 * intel + 105) as i16;
                }
            }
        }
    } else if ctx.monster.Monsters[m].goal == MonsterGoal::Retreat {
        let d = Direction::from_u8(ctx.monster.Monsters[m].goalVar2 as u8);
        ctx.monster.Monsters[m].direction = d;
        random_walk(ctx, m, d);
    } else if ctx.monster.Monsters[m].goal == MonsterGoal::Attack {
        if distance_to_enemy(ctx, m) < 2 {
            start_attack(ctx, m);
        } else {
            let d = get_monster_direction(ctx, m);
            random_walk(ctx, m, d);
        }
    } else {
        skeleton_ai(ctx, m);
    }
}

/// Original: `LeoricAi` (monster.cpp).
// @port monster.cpp|devilution::LeoricAi(Monster &monster) sha=6d3318699ad0
fn leoric_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    if ctx.monster.Monsters[m].activeForTicks < u8::MAX {
        crate::objects::monst_check_doors(ctx, m);
    }
    let mut v = rnd(ctx, 100);
    let distance_to_enemy = distance_to_enemy(ctx, m);
    if distance_to_enemy >= 2 && ctx.monster.Monsters[m].activeForTicks == u8::MAX && same_room_as_enemy(ctx, m) {
        if ctx.monster.Monsters[m].goal == MonsterGoal::Move || (distance_to_enemy >= 3 && ctx.rng.flip_coin(4)) {
            if ctx.monster.Monsters[m].goal != MonsterGoal::Move {
                let r = rnd(ctx, 2) as i8;
                ctx.monster.Monsters[m].goalVar1 = 0;
                ctx.monster.Monsters[m].goalVar2 = r;
            }
            ctx.monster.Monsters[m].goal = MonsterGoal::Move;
            if (goal_var1_post_inc_ge(ctx, m, 2 * distance_to_enemy as i32) && dir_ok(ctx, m, md)) || !same_room_as_enemy(ctx, m) {
                ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
            } else if !round_walk(ctx, m, md) {
                let r = rnd(ctx, 10) + 10;
                ai_delay(ctx, m, r);
            }
        }
    } else {
        ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Normal {
        let intel = ctx.monster.Monsters[m].intelligence as i32;
        let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
        if !ctx.init.gb_is_multiplayer && ((distance_to_enemy >= 3 && v < 4 * intel + 35) || v < 6) && line_clear_missile(ctx, tile, enemy_pos) {
            let new_position = tile + md;
            if is_tile_available_for_monster(ctx, m, new_position) && ctx.monster.ActiveMonsterCount < MaxMonsters {
                spawn_skeleton(ctx, new_position, md);
                start_special_stand(ctx, m, md);
            }
        } else if distance_to_enemy >= 2 {
            v = rnd(ctx, 100);
            let mon = &ctx.monster.Monsters[m];
            if v >= intel + 25 && (!matches!(var1_mode(mon), MonsterMode::MoveNorthwards | MonsterMode::MoveSouthwards | MonsterMode::MoveSideways) || mon.var2 != 0 || v >= intel + 75) {
                let r = rnd(ctx, 10) + 10;
                ai_delay(ctx, m, r);
            } else {
                random_walk(ctx, m, md);
            }
        } else if v < intel + 20 {
            ctx.monster.Monsters[m].direction = md;
            start_attack(ctx, m);
        }
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `BatAi` (monster.cpp).
// @port monster.cpp|devilution::BatAi(Monster &monster) sha=5c9b7cddf342
fn bat_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    ctx.monster.Monsters[m].direction = md;
    let v = rnd(ctx, 100);
    if ctx.monster.Monsters[m].goal == MonsterGoal::Retreat {
        if ctx.monster.Monsters[m].goalVar1 == 0 {
            random_walk(ctx, m, opposite(md));
            ctx.monster.Monsters[m].goalVar1 += 1;
        } else {
            let d = ctx.rng.pick_randomly_among(&[right(md), left(md)]);
            random_walk(ctx, m, d);
            ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
        }
        return;
    }
    let distance_to_enemy = distance_to_enemy(ctx, m);
    let intel = ctx.monster.Monsters[m].intelligence as i32;
    let t = monster_type_id(ctx, m);
    let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
    if t == MT_GLOOM && distance_to_enemy >= 5 && v < 4 * intel + 33 && line_clear(ctx, &|ctx: &Ctx, p: Point| is_tile_available_for_monster(ctx, m, p), tile, enemy_pos) {
        if crate::missiles::add_missile(ctx, tile, enemy_pos, md, MissileID::Rhino, TARGET_PLAYERS, m as i32, 0, 0, None).is_some() {
            ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = -((m + 1) as i16);
            ctx.monster.Monsters[m].mode = MonsterMode::Charge;
        }
    } else if distance_to_enemy >= 2 {
        let mon = &ctx.monster.Monsters[m];
        if (mon.var2 > 20 && v < intel + 13) || (is_monster_mode_move(var1_mode(mon)) && mon.var2 == 0 && v < intel + 63) {
            random_walk(ctx, m, md);
        }
    } else if v < 4 * intel + 8 {
        start_attack(ctx, m);
        ctx.monster.Monsters[m].goal = MonsterGoal::Retreat;
        ctx.monster.Monsters[m].goalVar1 = 0;
        if t == MT_FAMILIAR {
            let ep = ctx.monster.Monsters[m].enemyPosition;
            let r = rnd(ctx, 10) + 1;
            crate::missiles::add_missile(ctx, ep, Point::new(ep.x + 1, 0), Direction::South, MissileID::Lightning, TARGET_PLAYERS, m as i32, r, 0, None);
        }
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `GargoyleAi` (monster.cpp).
// @port monster.cpp|devilution::GargoyleAi(Monster &monster) sha=4050f58057a3
fn gargoyle_ai(ctx: &mut Ctx, m: usize) {
    let md = get_monster_direction(ctx, m);
    let distance_to_enemy = distance_to_enemy(ctx, m);
    let intel = ctx.monster.Monsters[m].intelligence as u32;
    if ctx.monster.Monsters[m].activeForTicks != 0 && (ctx.monster.Monsters[m].flags & MFLAG_ALLOW_SPECIAL as u32) != 0 {
        update_enemy(ctx, m);
        if distance_to_enemy < intel + 2 {
            ctx.monster.Monsters[m].flags &= !(MFLAG_ALLOW_SPECIAL as u32);
        }
        return;
    }
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    {
        let mon = &mut ctx.monster.Monsters[m];
        if mon.hitPoints < (mon.maxHitPoints / 2) {
            mon.goal = MonsterGoal::Retreat;
        }
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Retreat {
        if distance_to_enemy >= intel + 2 {
            ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
            start_heal(ctx, m);
        } else if !random_walk(ctx, m, opposite(md)) {
            ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
        }
    }
    ai_avoidance(ctx, m);
}

/// Original: `ButcherAi` (monster.cpp).
// @port monster.cpp|devilution::ButcherAi(Monster &monster) sha=ee850a3a0bba
fn butcher_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    ctx.monster.Monsters[m].direction = md;
    if distance_to_enemy(ctx, m) >= 2 {
        random_walk(ctx, m, md);
    } else {
        start_attack(ctx, m);
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `SneakAi` (monster.cpp).
// @port monster.cpp|devilution::SneakAi(Monster &monster) sha=5508e89e3cd0
fn sneak_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    let tile = ctx.monster.Monsters[m].position.tile;
    if ctx.gendung.dLight[tile.x as usize][tile.y as usize] as i32 == crate::lighting::LightsMax as i32 {
        return;
    }
    let intel = ctx.monster.Monsters[m].intelligence as i32;
    let dist = (5 - intel) as u32;
    let distance_to_enemy = distance_to_enemy(ctx, m);
    {
        let mon = &mut ctx.monster.Monsters[m];
        if var1_mode(mon) == MonsterMode::HitRecovery {
            mon.goal = MonsterGoal::Retreat;
            mon.goalVar1 = 0;
        } else if distance_to_enemy >= dist + 3 || mon.goalVar1 > 8 {
            mon.goal = MonsterGoal::Normal;
            mon.goalVar1 = 0;
        }
    }
    let mut md = get_monster_direction(ctx, m);
    let mon = ctx.monster.Monsters[m].clone();
    if mon.goal == MonsterGoal::Retreat && (mon.flags & MFLAG_NO_ENEMY as u32) == 0 {
        if (mon.flags & MFLAG_TARGETS_MONSTER as u32) != 0 {
            md = crate::engine::get_direction(tile, ctx.monster.Monsters[mon.enemy as usize].position.tile);
        } else {
            md = crate::engine::get_direction(tile, ctx.players.Players[mon.enemy as usize].position.last);
        }
        md = opposite(md);
        if monster_type_id(ctx, m) == MT_UNSEEN {
            md = ctx.rng.pick_randomly_among(&[right(md), left(md)]);
        }
    }
    ctx.monster.Monsters[m].direction = md;
    let v = rnd(ctx, 100);
    let hidden = (ctx.monster.Monsters[m].flags & MFLAG_HIDDEN as u32) != 0;
    if distance_to_enemy < dist && hidden {
        start_fadein(ctx, m, md, false);
    } else if distance_to_enemy >= dist + 1 && !hidden {
        start_fadeout(ctx, m, md, true);
    } else {
        let mon = &ctx.monster.Monsters[m];
        if mon.goal == MonsterGoal::Retreat || (distance_to_enemy >= 2 && ((mon.var2 > 20 && v < 4 * intel + 14) || (is_monster_mode_move(var1_mode(mon)) && mon.var2 == 0 && v < 4 * intel + 64))) {
            ctx.monster.Monsters[m].goalVar1 += 1;
            random_walk(ctx, m, md);
        }
    }
    if ctx.monster.Monsters[m].mode == MonsterMode::Stand {
        if distance_to_enemy >= 2 || v >= 4 * intel + 10 {
            change_animation_data(ctx, m, MonsterGraphic::Stand);
        } else {
            start_attack(ctx, m);
        }
    }
}

/// Original: `GharbadAi` (monster.cpp).
// @port monster.cpp|devilution::GharbadAi(Monster &monster) sha=d48490bfdc99
fn gharbad_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    let md = get_monster_direction(ctx, m);
    let tile = ctx.monster.Monsters[m].position.tile;
    let talk = ctx.monster.Monsters[m].talkMsg;
    if talk >= TEXT_GARBUD1 && talk <= TEXT_GARBUD3 && !is_tile_visible(ctx, tile) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
        ctx.monster.Monsters[m].goal = MonsterGoal::Inquiring;
        let next = match talk {
            TEXT_GARBUD1 => Some((TEXT_GARBUD2, QS_GHARBAD_FIRST_ITEM_READY)),
            TEXT_GARBUD2 => Some((TEXT_GARBUD3, QS_GHARBAD_SECOND_ITEM_NEARLY_DONE)),
            TEXT_GARBUD3 => Some((TEXT_GARBUD4, QS_GHARBAD_SECOND_ITEM_READY)),
            _ => None,
        };
        if let Some((msg, qs)) = next {
            ctx.monster.Monsters[m].talkMsg = msg;
            ctx.quests.Quests[Q_GARBUD as usize]._qvar1 = qs as u8;
            crate::msg::net_send_cmd_quest(ctx, true, Q_GARBUD as usize);
        }
    }
    if is_tile_visible(ctx, tile) && ctx.monster.Monsters[m].talkMsg == TEXT_GARBUD4 && !crate::effects::effect_is_playing(ctx, crate::effects::USFX_GARBUD4 as i32) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
        let mon = &mut ctx.monster.Monsters[m];
        mon.goal = MonsterGoal::Normal;
        mon.activeForTicks = u8::MAX;
        mon.talkMsg = TEXT_NONE;
        ctx.quests.Quests[Q_GARBUD as usize]._qvar1 = QS_GHARBAD_ATTACKING as u8;
        crate::msg::net_send_cmd_quest(ctx, true, Q_GARBUD as usize);
    }
    if matches!(ctx.monster.Monsters[m].goal, MonsterGoal::Normal | MonsterGoal::Move) {
        ai_avoidance(ctx, m);
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `SnotSpilAi` (monster.cpp).
// @port monster.cpp|devilution::SnotSpilAi(Monster &monster) sha=6a2609931505
fn snot_spil_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    let md = get_monster_direction(ctx, m);
    let tile = ctx.monster.Monsters[m].position.tile;
    if ctx.monster.Monsters[m].talkMsg == TEXT_BANNER10 && !is_tile_visible(ctx, tile) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
        ctx.monster.Monsters[m].talkMsg = TEXT_BANNER11;
        ctx.monster.Monsters[m].goal = MonsterGoal::Inquiring;
    }
    if ctx.monster.Monsters[m].talkMsg == TEXT_BANNER11 && ctx.quests.Quests[Q_LTBANNER as usize]._qvar1 == 3 {
        ctx.monster.Monsters[m].talkMsg = TEXT_NONE;
        ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
    }
    if is_tile_visible(ctx, tile) {
        if ctx.monster.Monsters[m].talkMsg == TEXT_BANNER12 && !crate::effects::effect_is_playing(ctx, crate::effects::USFX_SNOT3 as i32) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
            let sp = ctx.gendung.SetPiece;
            crate::objects::obj_change_map(ctx, sp.position.x, sp.position.y, sp.position.x + sp.size.width + 1, sp.position.y + sp.size.height + 1);
            ctx.quests.Quests[Q_LTBANNER as usize]._qvar1 = 3;
            crate::msg::net_send_cmd_quest(ctx, true, Q_LTBANNER as usize);
            crate::player::redo_player_vision(ctx);
            let mon = &mut ctx.monster.Monsters[m];
            mon.activeForTicks = u8::MAX;
            mon.talkMsg = TEXT_NONE;
            mon.goal = MonsterGoal::Normal;
        }
        if ctx.quests.Quests[Q_LTBANNER as usize]._qvar1 == 3 && matches!(ctx.monster.Monsters[m].goal, MonsterGoal::Normal | MonsterGoal::Attack) {
            fallen_ai(ctx, m);
        }
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `SnakeAi` (monster.cpp).
// @port monster.cpp|devilution::SnakeAi(Monster &monster) sha=13467609fc48
fn snake_ai(ctx: &mut Ctx, m: usize) {
    const PATTERN: [i8; 6] = [1, 1, 0, -1, -1, 0];
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let mut md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    ctx.monster.Monsters[m].direction = md;
    let distance_to_enemy = distance_to_enemy(ctx, m);
    let intel = ctx.monster.Monsters[m].intelligence as i32;
    let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
    if distance_to_enemy >= 2 {
        let v1 = var1_mode(&ctx.monster.Monsters[m]);
        if distance_to_enemy < 3 && line_clear(ctx, &|ctx: &Ctx, p: Point| is_tile_available_for_monster(ctx, m, p), tile, enemy_pos) && v1 != MonsterMode::Charge {
            if crate::missiles::add_missile(ctx, tile, enemy_pos, md, MissileID::Rhino, TARGET_PLAYERS, m as i32, 0, 0, None).is_some() {
                play_effect(ctx, m, MonsterSound::Attack);
                ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = -((m + 1) as i16);
                ctx.monster.Monsters[m].mode = MonsterMode::Charge;
            }
        } else if v1 == MonsterMode::Delay || rnd(ctx, 100) >= 35 - 2 * intel {
            let mon = &mut ctx.monster.Monsters[m];
            if PATTERN[mon.goalVar1 as usize] == -1 {
                md = left(md);
            } else if PATTERN[mon.goalVar1 as usize] == 1 {
                md = right(md);
            }
            mon.goalVar1 += 1;
            if mon.goalVar1 > 5 {
                mon.goalVar1 = 0;
            }
            let target_direction = Direction::from_u8(mon.goalVar2 as u8);
            if md != target_direction {
                let mut drift = md as i32 - mon.goalVar2 as i32;
                if drift < 0 {
                    drift += 8;
                }
                if drift < 4 {
                    md = right(target_direction);
                } else if drift > 4 {
                    md = left(target_direction);
                }
                mon.goalVar2 = md as i8;
            }
            if !walk(ctx, m, md) {
                let d = ctx.monster.Monsters[m].direction;
                random_walk2(ctx, m, d);
            }
        } else {
            let r = 15 - intel + rnd(ctx, 10);
            ai_delay(ctx, m, r);
        }
    } else if matches!(var1_mode(&ctx.monster.Monsters[m]), MonsterMode::Delay | MonsterMode::Charge) || rnd(ctx, 100) < intel + 20 {
        start_attack(ctx, m);
    } else {
        let r = 10 - intel + rnd(ctx, 10);
        ai_delay(ctx, m, r);
    }
    let d = ctx.monster.Monsters[m].direction;
    check_stand_animation_is_loaded(ctx, m, d);
}

/// Original: `CounselorAi` (monster.cpp).
// @port monster.cpp|devilution::CounselorAi(Monster &monster) sha=880f483e4546
fn counselor_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    if ctx.monster.Monsters[m].activeForTicks < u8::MAX {
        crate::objects::monst_check_doors(ctx, m);
    }
    let v = rnd(ctx, 100);
    let distance_to_enemy = distance_to_enemy(ctx, m);
    let intel = ctx.monster.Monsters[m].intelligence as i32;
    match ctx.monster.Monsters[m].goal {
        MonsterGoal::Retreat => {
            let g = ctx.monster.Monsters[m].goalVar1;
            ctx.monster.Monsters[m].goalVar1 += 1;
            if g <= 3 {
                random_walk(ctx, m, opposite(md));
            } else {
                ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
                start_fadein(ctx, m, md, true);
            }
        }
        MonsterGoal::Move => {
            if distance_to_enemy >= 2 && ctx.monster.Monsters[m].activeForTicks == u8::MAX && same_room_as_enemy(ctx, m) {
                let g = ctx.monster.Monsters[m].goalVar1;
                ctx.monster.Monsters[m].goalVar1 += 1;
                if (g as i32) < 2 * distance_to_enemy as i32 || !dir_ok(ctx, m, md) {
                    round_walk(ctx, m, md);
                } else {
                    ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
                    start_fadein(ctx, m, md, true);
                }
            } else {
                ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
                start_fadein(ctx, m, md, true);
            }
        }
        MonsterGoal::Normal => {
            let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
            if distance_to_enemy >= 2 {
                if v < 5 * (intel + 10) && line_clear_missile(ctx, tile, enemy_pos) {
                    const MISSILE_TYPES: [MissileID; 4] = [MissileID::Firebolt, MissileID::ChargedBolt, MissileID::LightningControl, MissileID::Fireball];
                    let (mind, maxd) = (ctx.monster.Monsters[m].minDamage as i32, ctx.monster.Monsters[m].maxDamage as i32);
                    let dam = mind + rnd(ctx, maxd - mind + 1);
                    start_ranged_attack(ctx, m, MISSILE_TYPES[intel as usize], dam);
                } else if rnd(ctx, 100) < 30 {
                    ctx.monster.Monsters[m].goal = MonsterGoal::Move;
                    ctx.monster.Monsters[m].goalVar1 = 0;
                    start_fadeout(ctx, m, md, false);
                } else {
                    let r = rnd(ctx, 10) + 2 * (5 - intel);
                    ai_delay(ctx, m, r);
                }
            } else {
                ctx.monster.Monsters[m].direction = md;
                let mon = ctx.monster.Monsters[m].clone();
                if mon.hitPoints < (mon.maxHitPoints / 2) {
                    ctx.monster.Monsters[m].goal = MonsterGoal::Retreat;
                    ctx.monster.Monsters[m].goalVar1 = 0;
                    start_fadeout(ctx, m, md, false);
                } else if var1_mode(&mon) == MonsterMode::Delay || rnd(ctx, 100) < 2 * intel + 20 {
                    start_ranged_attack(ctx, m, MissileID::Null, 0);
                    let d = ctx.monster.Monsters[m].direction;
                    crate::missiles::add_missile(ctx, tile, Point::new(0, 0), d, MissileID::FlashBottom, TARGET_PLAYERS, m as i32, 4, 0, None);
                    crate::missiles::add_missile(ctx, tile, Point::new(0, 0), d, MissileID::FlashTop, TARGET_PLAYERS, m as i32, 4, 0, None);
                } else {
                    let r = rnd(ctx, 10) + 2 * (5 - intel);
                    ai_delay(ctx, m, r);
                }
            }
        }
        _ => {}
    }
    if ctx.monster.Monsters[m].mode == MonsterMode::Stand {
        let r = rnd(ctx, 10) + 5;
        ai_delay(ctx, m, r);
    }
}

// ---------------------------------------------------------------------------------------------
// monster.cpp, part 3: remaining AI routines, level setup and the public interface.
// ---------------------------------------------------------------------------------------------

/// Original: `ZharAi` (monster.cpp).
// @port monster.cpp|devilution::ZharAi(Monster &monster) sha=cb7cf68141ed
fn zhar_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    let md = get_monster_direction(ctx, m);
    let tile = ctx.monster.Monsters[m].position.tile;
    if ctx.monster.Monsters[m].talkMsg == TEXT_ZHAR1 && !is_tile_visible(ctx, tile) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
        ctx.monster.Monsters[m].talkMsg = TEXT_ZHAR2;
        ctx.monster.Monsters[m].goal = MonsterGoal::Inquiring;
        ctx.quests.Quests[Q_ZHAR as usize]._qvar1 = QS_ZHAR_ANGRY as u8;
        crate::msg::net_send_cmd_quest(ctx, true, Q_ZHAR as usize);
    }
    if is_tile_visible(ctx, tile) && ctx.monster.Monsters[m].talkMsg == TEXT_ZHAR2 && !crate::effects::effect_is_playing(ctx, crate::effects::USFX_ZHAR2 as i32) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
        let mon = &mut ctx.monster.Monsters[m];
        mon.activeForTicks = u8::MAX;
        mon.talkMsg = TEXT_NONE;
        mon.goal = MonsterGoal::Normal;
        ctx.quests.Quests[Q_ZHAR as usize]._qvar1 = QS_ZHAR_ATTACKING as u8;
        crate::msg::net_send_cmd_quest(ctx, true, Q_ZHAR as usize);
    }
    if matches!(ctx.monster.Monsters[m].goal, MonsterGoal::Normal | MonsterGoal::Retreat | MonsterGoal::Move) {
        counselor_ai(ctx, m);
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `MegaAi` (monster.cpp).
// @port monster.cpp|devilution::MegaAi(Monster &monster) sha=f35ce0436024
fn mega_ai(ctx: &mut Ctx, m: usize) {
    let distance_to_enemy = distance_to_enemy(ctx, m);
    if distance_to_enemy >= 5 {
        skeleton_ai(ctx, m);
        return;
    }
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    if ctx.monster.Monsters[m].activeForTicks < u8::MAX {
        crate::objects::monst_check_doors(ctx, m);
    }
    let mut v = rnd(ctx, 100);
    let intel = ctx.monster.Monsters[m].intelligence as i32;
    if distance_to_enemy >= 2 && ctx.monster.Monsters[m].activeForTicks == u8::MAX && same_room_as_enemy(ctx, m) {
        if ctx.monster.Monsters[m].goal == MonsterGoal::Move || distance_to_enemy >= 3 {
            if ctx.monster.Monsters[m].goal != MonsterGoal::Move {
                let r = rnd(ctx, 2) as i8;
                ctx.monster.Monsters[m].goalVar1 = 0;
                ctx.monster.Monsters[m].goalVar2 = r;
            }
            ctx.monster.Monsters[m].goal = MonsterGoal::Move;
            ctx.monster.Monsters[m].goalVar3 = 4;
            let g = ctx.monster.Monsters[m].goalVar1;
            ctx.monster.Monsters[m].goalVar1 += 1;
            if (g as i32) < 2 * distance_to_enemy as i32 || !dir_ok(ctx, m, md) {
                if v < 5 * (intel + 16) {
                    round_walk(ctx, m, md);
                }
            } else {
                ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
            }
        }
    } else {
        ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Normal {
        let (tile, enemy_pos) = (ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].enemyPosition);
        let gv3 = ctx.monster.Monsters[m].goalVar3;
        if ((distance_to_enemy >= 3 && v < 5 * (intel + 2)) || v < 5 * (intel + 1) || gv3 == 4) && line_clear_missile(ctx, tile, enemy_pos) {
            start_ranged_special_attack(ctx, m, MissileID::InfernoControl, 0);
        } else if distance_to_enemy >= 2 {
            v = rnd(ctx, 100);
            let mon = &ctx.monster.Monsters[m];
            if v < 2 * (5 * intel + 25) || (is_monster_mode_move(var1_mode(mon)) && mon.var2 == 0 && v < 2 * (5 * intel + 40)) {
                random_walk(ctx, m, md);
            }
        } else if rnd(ctx, 100) < 10 * (intel + 4) {
            ctx.monster.Monsters[m].direction = md;
            if ctx.rng.flip_coin(2) {
                start_ranged_special_attack(ctx, m, MissileID::InfernoControl, 0);
            } else {
                start_attack(ctx, m);
            }
        }
        ctx.monster.Monsters[m].goalVar3 = 1;
    }
    if ctx.monster.Monsters[m].mode == MonsterMode::Stand {
        let r = rnd(ctx, 10) + 5;
        ai_delay(ctx, m, r);
    }
}

/// Original: `LazarusAi` (monster.cpp).
// @port monster.cpp|devilution::LazarusAi(Monster &monster) sha=a69e303ea210
fn lazarus_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    let md = get_monster_direction(ctx, m);
    let tile = ctx.monster.Monsters[m].position.tile;
    let mpq = crate::quests::use_multiplayer_quests(ctx);
    if is_tile_visible(ctx, tile) {
        if !mpq {
            let me = ctx.players.MyPlayer.expect("MyPlayer");
            let my_tile = ctx.players.Players[me].position.tile;
            if ctx.monster.Monsters[m].talkMsg == TEXT_VILE13 && ctx.monster.Monsters[m].goal == MonsterGoal::Inquiring && my_tile == Point::new(35, 46) {
                if !ctx.init.gb_is_multiplayer {
                    // Playing ingame movies is currently not supported in multiplayer
                    crate::movie::play_in_game_movie(ctx, "gendata\\fprst3.smk");
                }
                ctx.monster.Monsters[m].mode = MonsterMode::Talk;
                ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 = 5;
                crate::msg::net_send_cmd_quest(ctx, true, Q_BETRAYER as usize);
            }
            if ctx.monster.Monsters[m].talkMsg == TEXT_VILE13 && !crate::effects::effect_is_playing(ctx, crate::effects::USFX_LAZ1 as i32) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
                crate::objects::obj_change_map(ctx, 1, 18, 20, 24);
                crate::player::redo_player_vision(ctx);
                ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 = 6;
                let mon = &mut ctx.monster.Monsters[m];
                mon.goal = MonsterGoal::Normal;
                mon.activeForTicks = u8::MAX;
                mon.talkMsg = TEXT_NONE;
                crate::msg::net_send_cmd_quest(ctx, true, Q_BETRAYER as usize);
            }
        }
        if mpq && ctx.monster.Monsters[m].talkMsg == TEXT_VILE13 && ctx.monster.Monsters[m].goal == MonsterGoal::Inquiring && ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 <= 3 {
            ctx.monster.Monsters[m].mode = MonsterMode::Talk;
        }
    }
    if matches!(ctx.monster.Monsters[m].goal, MonsterGoal::Normal | MonsterGoal::Retreat | MonsterGoal::Move) {
        if !mpq && ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 == 4 && ctx.monster.Monsters[m].talkMsg == TEXT_NONE {
            // Fix save games affected by teleport bug
            crate::objects::obj_change_map_resync(ctx, 1, 18, 20, 24);
            crate::player::redo_player_vision(ctx);
            ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 = 6;
        }
        ctx.monster.Monsters[m].talkMsg = TEXT_NONE;
        counselor_ai(ctx, m);
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `LazarusMinionAi` (monster.cpp).
// @port monster.cpp|devilution::LazarusMinionAi(Monster &monster) sha=08661161809e
fn lazarus_minion_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    let md = get_monster_direction(ctx, m);
    if is_tile_visible(ctx, ctx.monster.Monsters[m].position.tile) {
        if !crate::quests::use_multiplayer_quests(ctx) {
            if ctx.quests.Quests[Q_BETRAYER as usize]._qvar1 <= 5 {
                ctx.monster.Monsters[m].goal = MonsterGoal::Inquiring;
            } else {
                ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
                ctx.monster.Monsters[m].talkMsg = TEXT_NONE;
            }
        } else {
            ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
        }
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Normal {
        ai_ranged(ctx, m);
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `LachdananAi` (monster.cpp).
// @port monster.cpp|devilution::LachdananAi(Monster &monster) sha=68e0f44f3e82
fn lachdanan_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    let md = get_monster_direction(ctx, m);
    let tile = ctx.monster.Monsters[m].position.tile;
    if ctx.monster.Monsters[m].talkMsg == TEXT_VEIL9 && !is_tile_visible(ctx, tile) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
        ctx.monster.Monsters[m].talkMsg = TEXT_VEIL10;
        ctx.monster.Monsters[m].goal = MonsterGoal::Inquiring;
        ctx.quests.Quests[Q_VEIL as usize]._qvar2 = QS_VEIL_EARLY_RETURN as u8;
        crate::msg::net_send_cmd_quest(ctx, true, Q_VEIL as usize);
    }
    if is_tile_visible(ctx, tile) && ctx.monster.Monsters[m].talkMsg == TEXT_VEIL11 && !crate::effects::effect_is_playing(ctx, crate::effects::USFX_LACH3 as i32) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
        ctx.monster.Monsters[m].talkMsg = TEXT_NONE;
        ctx.quests.Quests[Q_VEIL as usize]._qactive = QUEST_DONE;
        crate::msg::net_send_cmd_quest(ctx, true, Q_VEIL as usize);
        let d = ctx.monster.Monsters[m].direction;
        monster_death_dir(ctx, m, d, true);
        let me = ctx.players.MyPlayer.expect("MyPlayer");
        let t = ctx.monster.Monsters[m].position.tile;
        crate::msg::delta_kill_monster(ctx, m, t, me);
        crate::msg::net_send_cmd_loc_param1(ctx, false, CMD_MONSTDEATH, t, m as u16);
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `WarlordAi` (monster.cpp).
// @port monster.cpp|devilution::WarlordAi(Monster &monster) sha=cfea4588caa4
fn warlord_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand {
        return;
    }
    let md = get_monster_direction(ctx, m);
    if is_tile_visible(ctx, ctx.monster.Monsters[m].position.tile) {
        if ctx.monster.Monsters[m].talkMsg == TEXT_WARLRD9 && ctx.monster.Monsters[m].goal == MonsterGoal::Inquiring {
            ctx.monster.Monsters[m].mode = MonsterMode::Talk;
        }
        if ctx.monster.Monsters[m].talkMsg == TEXT_WARLRD9 && !crate::effects::effect_is_playing(ctx, crate::effects::USFX_WARLRD1 as i32) && ctx.monster.Monsters[m].goal == MonsterGoal::Talking {
            let mon = &mut ctx.monster.Monsters[m];
            mon.activeForTicks = u8::MAX;
            mon.talkMsg = TEXT_NONE;
            mon.goal = MonsterGoal::Normal;
            ctx.quests.Quests[Q_WARLORD as usize]._qvar1 = QS_WARLORD_ATTACKING as u8;
            crate::msg::net_send_cmd_quest(ctx, true, Q_WARLORD as usize);
        }
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Normal {
        skeleton_ai(ctx, m);
    }
    check_stand_animation_is_loaded(ctx, m, md);
}

/// Original: `HorkDemonAi` (monster.cpp).
// @port monster.cpp|devilution::HorkDemonAi(Monster &monster) sha=5cf87f7997d9
fn hork_demon_ai(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].mode != MonsterMode::Stand || ctx.monster.Monsters[m].activeForTicks == 0 {
        return;
    }
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.monster.Monsters[m].position.last);
    if ctx.monster.Monsters[m].activeForTicks < 255 {
        crate::objects::monst_check_doors(ctx, m);
    }
    let mut v = rnd(ctx, 100);
    let distance_to_enemy = distance_to_enemy(ctx, m);
    if distance_to_enemy < 2 {
        ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
    } else if ctx.monster.Monsters[m].goal == MonsterGoal::Move || (distance_to_enemy >= 5 && !ctx.rng.flip_coin(4)) {
        if ctx.monster.Monsters[m].goal != MonsterGoal::Move {
            let r = rnd(ctx, 2) as i8;
            ctx.monster.Monsters[m].goalVar1 = 0;
            ctx.monster.Monsters[m].goalVar2 = r;
        }
        ctx.monster.Monsters[m].goal = MonsterGoal::Move;
        if goal_var1_post_inc_ge(ctx, m, 2 * distance_to_enemy as i32) || !same_room_as_enemy(ctx, m) {
            ctx.monster.Monsters[m].goal = MonsterGoal::Normal;
        } else if !round_walk(ctx, m, md) {
            let r = rnd(ctx, 10) + 10;
            ai_delay(ctx, m, r);
        }
    }
    if ctx.monster.Monsters[m].goal == MonsterGoal::Normal {
        let intel = ctx.monster.Monsters[m].intelligence as i32;
        if distance_to_enemy >= 3 && v < 2 * intel + 43 {
            let position = ctx.monster.Monsters[m].position.tile + ctx.monster.Monsters[m].direction;
            if is_tile_available_for_monster(ctx, m, position) && ctx.monster.ActiveMonsterCount < MaxMonsters {
                start_ranged_special_attack(ctx, m, MissileID::HorkSpawn, 0);
            }
        } else if distance_to_enemy < 2 {
            if v < 2 * intel + 28 {
                ctx.monster.Monsters[m].direction = md;
                start_attack(ctx, m);
            }
        } else {
            v = rnd(ctx, 100);
            let mon = &ctx.monster.Monsters[m];
            if v < 2 * intel + 33 || (is_monster_mode_move(var1_mode(mon)) && mon.var2 == 0 && v < 2 * intel + 83) {
                random_walk(ctx, m, md);
            } else {
                let r = rnd(ctx, 10) + 10;
                ai_delay(ctx, m, r);
            }
        }
    }
    let d = ctx.monster.Monsters[m].direction;
    check_stand_animation_is_loaded(ctx, m, d);
}

/// Original: `GetMonsterTypeText` (monster.cpp).
// @port monster.cpp|devilution::GetMonsterTypeText(const MonsterData &monsterData) sha=ba16707329fe
fn get_monster_type_text(monster_data: &MonsterData) -> String {
    match monster_data.monsterClass {
        MonsterClass::Animal => tr("Animal"),
        MonsterClass::Demon => tr("Demon"),
        MonsterClass::Undead => tr("Undead"),
    }
}

/// Original: `ActivateSpawn` (monster.cpp).
// @port monster.cpp|devilution::ActivateSpawn(Monster &monster, Point position, Direction dir) sha=a445a80748ab
fn activate_spawn(ctx: &mut Ctx, m: usize, position: Point, dir: Direction) {
    ctx.gendung.dMonster[position.x as usize][position.y as usize] = (m + 1) as i16;
    let monster = &mut ctx.monster.Monsters[m];
    monster.position.tile = position;
    monster.position.future = position;
    monster.position.old = position;
    start_special_stand(ctx, m, dir);
}

/// `AiProc`: maps from monster AI ID to monster AI function.
const AI_PROC: [Option<fn(&mut Ctx, usize)>; 40] = [
    Some(zombie_ai),          // Zombie
    Some(overlord_ai),        // Fat
    Some(skeleton_ai),        // SkeletonMelee
    Some(skeleton_bow_ai),    // SkeletonRanged
    Some(scavenger_ai),       // Scavenger
    Some(rhino_ai),           // Rhino
    Some(ai_avoidance),       // GoatMelee
    Some(ai_ranged),          // GoatRanged
    Some(fallen_ai),          // Fallen
    Some(ai_ranged_avoidance), // Magma
    Some(leoric_ai),          // SkeletonKing
    Some(bat_ai),             // Bat
    Some(gargoyle_ai),        // Gargoyle
    Some(butcher_ai),         // Butcher
    Some(ai_ranged),          // Succubus
    Some(sneak_ai),           // Sneak
    Some(ai_ranged_avoidance), // Storm
    None,                     // FireMan
    Some(gharbad_ai),         // Gharbad
    Some(ai_ranged_avoidance), // Acid
    Some(ai_ranged),          // AcidUnique
    Some(golum_ai),           // Golem
    Some(zhar_ai),            // Zhar
    Some(snot_spil_ai),       // Snotspill
    Some(snake_ai),           // Snake
    Some(counselor_ai),       // Counselor
    Some(mega_ai),            // Mega
    Some(ai_ranged_avoidance), // Diablo
    Some(lazarus_ai),         // Lazarus
    Some(lazarus_minion_ai),  // LazarusSuccubus
    Some(lachdanan_ai),       // Lachdanan
    Some(warlord_ai),         // Warlord
    Some(ai_ranged),          // FireBat
    Some(ai_ranged),          // Torchant
    Some(hork_demon_ai),      // HorkDemon
    Some(ai_ranged),          // Lich
    Some(ai_ranged),          // ArchLich
    Some(ai_ranged),          // Psychorb
    Some(ai_ranged),          // Necromorb
    Some(ai_ranged_avoidance), // BoneDemon
];

/// `AiProc[static_cast<int8_t>(monster.ai)](monster)`
fn ai_proc(ctx: &mut Ctx, m: usize) {
    let ai = ctx.monster.Monsters[m].ai;
    let f = AI_PROC[ai as i8 as usize].expect("AiProc");
    f(ctx, m);
}

/// Original: `IsRelativeMoveOK` (monster.cpp).
// @port monster.cpp|devilution::IsRelativeMoveOK(const Monster &monster, Point position, Direction mdir) sha=c97adb0ddd61
fn is_relative_move_ok(ctx: &Ctx, m: usize, position: Point, mdir: Direction) -> bool {
    use crate::engine::path::is_tile_solid;
    let future_position = position + mdir;
    if !in_dungeon_bounds(future_position) || !is_tile_available_for_monster(ctx, m, future_position) {
        return false;
    }
    match mdir {
        Direction::East => !is_tile_solid(ctx, position + Direction::SouthEast),
        Direction::West => !is_tile_solid(ctx, position + Direction::SouthWest),
        Direction::North => !(is_tile_solid(ctx, position + Direction::NorthEast) || is_tile_solid(ctx, position + Direction::NorthWest)),
        Direction::South => !(is_tile_solid(ctx, position + Direction::SouthWest) || is_tile_solid(ctx, position + Direction::SouthEast)),
        _ => true,
    }
}

/// Original: `IsMonsterAvalible` (monster.cpp).
// @port monster.cpp|devilution::IsMonsterAvalible(const MonsterData &monsterData) sha=6b9dd6935cd5
fn is_monster_avalible(ctx: &Ctx, monster_data: &MonsterData) -> bool {
    if monster_data.availability == MonsterAvailability::Never {
        return false;
    }
    if ctx.init.gb_is_spawn && monster_data.availability == MonsterAvailability::Retail {
        return false;
    }
    let currlevel = ctx.gendung.currlevel as i32;
    currlevel >= monster_data.minDunLvl as i32 && currlevel <= monster_data.maxDunLvl as i32
}

/// Original: `UpdateModeStance` (monster.cpp).
// @port monster.cpp|devilution::UpdateModeStance(Monster &monster) sha=807bfc8400e4
fn update_mode_stance(ctx: &mut Ctx, m: usize) -> bool {
    match ctx.monster.Monsters[m].mode {
        MonsterMode::Stand => {
            monster_idle(ctx, m);
            false
        }
        mode @ (MonsterMode::MoveNorthwards | MonsterMode::MoveSouthwards | MonsterMode::MoveSideways) => monster_walk(ctx, m, mode),
        MonsterMode::MeleeAttack => monster_attack(ctx, m),
        MonsterMode::HitRecovery => monster_got_hit(ctx, m),
        MonsterMode::Death => {
            monster_death_mode(ctx, m);
            false
        }
        MonsterMode::SpecialMeleeAttack => monster_special_attack(ctx, m),
        MonsterMode::FadeIn => monster_fadein(ctx, m),
        MonsterMode::FadeOut => monster_fadeout(ctx, m),
        MonsterMode::RangedAttack => monster_ranged_attack(ctx, m),
        MonsterMode::SpecialStand => monster_special_stand(ctx, m),
        MonsterMode::SpecialRangedAttack => monster_ranged_special_attack(ctx, m),
        MonsterMode::Delay => monster_delay(ctx, m),
        MonsterMode::Petrified => {
            monster_petrified(ctx, m);
            false
        }
        MonsterMode::Heal => {
            monster_heal(ctx, m);
            false
        }
        MonsterMode::Talk => {
            monster_talk(ctx, m);
            false
        }
        _ => false,
    }
}

/// Original: `devilution::InitTRNForUniqueMonster` (monster.cpp).
// @port monster.cpp|devilution::InitTRNForUniqueMonster(Monster &monster) sha=ccbf565e8f1e
pub fn init_trn_for_unique_monster(ctx: &mut Ctx, m: usize) {
    let name = UniqueMonstersData[ctx.monster.Monsters[m].uniqueType.0 as usize].mTrnName.unwrap_or("");
    let filestr = format!("monsters\\monsters\\{name}.trn");
    let data = crate::engine::load_file::load_file_in_mem(ctx, &filestr).unwrap_or_default();
    let mut trn = [0u8; 256];
    let n = data.len().min(256);
    trn[..n].copy_from_slice(&data[..n]);
    ctx.monster.Monsters[m].uniqueMonsterTRN = Some(Rc::new(trn));
}

/// Original: `devilution::PrepareUniqueMonst` (monster.cpp).
// @port monster.cpp|devilution::PrepareUniqueMonst(Monster &monster, UniqueMonsterType monsterType, size_t minionType, int bosspacksize, const UniqueMonsterData &uniqueMonsterData) sha=606c44241ea0
pub fn prepare_unique_monst(ctx: &mut Ctx, m: usize, monster_type: UniqueMonsterType, minion_type: usize, bosspacksize: i32, unique_monster_data: &UniqueMonsterData) {
    let (hf, mp) = (ctx.init.gb_is_hellfire, ctx.init.gb_is_multiplayer);
    let difficulty = ctx.multi.sgGameInitInfo.nDifficulty;
    {
        let monster = &mut ctx.monster.Monsters[m];
        monster.uniqueType = monster_type;
        monster.maxHitPoints = (unique_monster_data.mmaxhp as i32) << 6;
        if !mp {
            monster.maxHitPoints = (monster.maxHitPoints / 2).max(64);
        }
        monster.hitPoints = monster.maxHitPoints;
        monster.ai = unique_monster_data.mAi;
        monster.intelligence = unique_monster_data.mint;
        monster.minDamage = unique_monster_data.mMinDamage;
        monster.maxDamage = unique_monster_data.mMaxDamage;
        monster.minDamageSpecial = unique_monster_data.mMinDamage;
        monster.maxDamageSpecial = unique_monster_data.mMaxDamage;
        monster.resistance = unique_monster_data.mMagicRes;
        monster.talkMsg = unique_monster_data.mtalkmsg;
    }
    if monster_type == UniqueMonsterType::HorkDemon {
        ctx.monster.Monsters[m].lightId = NO_LIGHT as i8;
    } else {
        let t = ctx.monster.Monsters[m].position.tile;
        ctx.monster.Monsters[m].lightId = crate::lighting::add_light(ctx, t, 3) as i8;
    }
    let mpq = crate::quests::use_multiplayer_quests(ctx);
    let betrayer_var1 = ctx.quests.Quests[Q_BETRAYER as usize]._qvar1;
    {
        let monster = &mut ctx.monster.Monsters[m];
        if mpq {
            if monster.ai == MonsterAIID::LazarusSuccubus {
                monster.talkMsg = TEXT_NONE;
            }
            if monster.ai == MonsterAIID::Lazarus && betrayer_var1 > 3 {
                monster.goal = MonsterGoal::Normal;
            } else if monster.talkMsg != TEXT_NONE {
                monster.goal = MonsterGoal::Inquiring;
            }
        } else if monster.talkMsg != TEXT_NONE {
            monster.goal = MonsterGoal::Inquiring;
        }
        if difficulty == DIFF_NIGHTMARE {
            monster.maxHitPoints *= 3;
            if hf {
                monster.maxHitPoints += (if mp { 100 } else { 50 }) << 6;
            } else {
                monster.maxHitPoints += 100 << 6;
            }
            monster.hitPoints = monster.maxHitPoints;
            monster.minDamage = 2u8.wrapping_mul(monster.minDamage.wrapping_add(2));
            monster.maxDamage = 2u8.wrapping_mul(monster.maxDamage.wrapping_add(2));
            monster.minDamageSpecial = 2u8.wrapping_mul(monster.minDamageSpecial.wrapping_add(2));
            monster.maxDamageSpecial = 2u8.wrapping_mul(monster.maxDamageSpecial.wrapping_add(2));
        } else if difficulty == DIFF_HELL {
            monster.maxHitPoints *= 4;
            if hf {
                monster.maxHitPoints += (if mp { 200 } else { 100 }) << 6;
            } else {
                monster.maxHitPoints += 200 << 6;
            }
            monster.hitPoints = monster.maxHitPoints;
            monster.minDamage = 4u8.wrapping_mul(monster.minDamage).wrapping_add(6);
            monster.maxDamage = 4u8.wrapping_mul(monster.maxDamage).wrapping_add(6);
            monster.minDamageSpecial = 4u8.wrapping_mul(monster.minDamageSpecial).wrapping_add(6);
            monster.maxDamageSpecial = 4u8.wrapping_mul(monster.maxDamageSpecial).wrapping_add(6);
        }
    }
    init_trn_for_unique_monster(ctx, m);
    ctx.monster.Monsters[m].uniqTrans = ctx.monster.uniquetrans as u8;
    ctx.monster.uniquetrans += 1;
    {
        let monster = &mut ctx.monster.Monsters[m];
        if unique_monster_data.customToHit != 0 {
            monster.toHit = unique_monster_data.customToHit as u16;
            if difficulty == DIFF_NIGHTMARE {
                monster.toHit += NightmareToHitBonus as u16;
            } else if difficulty == DIFF_HELL {
                monster.toHit += HellToHitBonus as u16;
            }
        }
        if unique_monster_data.customArmorClass != 0 {
            monster.armorClass = unique_monster_data.customArmorClass;
            if difficulty == DIFF_NIGHTMARE {
                monster.armorClass = monster.armorClass.wrapping_add(NightmareAcBonus as u8);
            } else if difficulty == DIFF_HELL {
                monster.armorClass = monster.armorClass.wrapping_add(HellAcBonus as u8);
            }
        }
    }
    if unique_monster_data.monsterPack != UniqueMonsterPack::None {
        place_group(ctx, minion_type, bosspacksize as u32, Some(m), unique_monster_data.monsterPack == UniqueMonsterPack::Leashed);
    }
    if ctx.monster.Monsters[m].ai != MonsterAIID::Gargoyle {
        change_animation_data(ctx, m, MonsterGraphic::Stand);
        let nof = ctx.monster.Monsters[m].animInfo.numberOfFrames as i32;
        let f = rnd(ctx, nof - 1) as i8;
        let monster = &mut ctx.monster.Monsters[m];
        monster.animInfo.currentFrame = f;
        monster.flags &= !(MFLAG_ALLOW_SPECIAL as u32);
        monster.mode = MonsterMode::Stand;
    }
}

/// Original: `devilution::GetLevelMTypes` (monster.cpp).
// @port monster.cpp|devilution::GetLevelMTypes() sha=006942ab0895
pub fn get_level_m_types(ctx: &mut Ctx) {
    use crate::quests::is_quest_available;
    add_monster_type(ctx, MT_GOLEM, PLACE_SPECIAL);
    let currlevel = ctx.gendung.currlevel;
    if currlevel == 16 {
        add_monster_type(ctx, MT_ADVOCATE, PLACE_SCATTER);
        add_monster_type(ctx, MT_RBLACK, PLACE_SCATTER);
        add_monster_type(ctx, MT_DIABLO, PLACE_SPECIAL);
        return;
    }
    if currlevel == 18 {
        add_monster_type(ctx, MT_HORKSPWN, PLACE_SCATTER);
    }
    if currlevel == 19 {
        add_monster_type(ctx, MT_HORKSPWN, PLACE_SCATTER);
        add_monster_type(ctx, MT_HORKDMN, PLACE_UNIQUE);
    }
    if currlevel == 20 {
        add_monster_type(ctx, MT_DEFILER, PLACE_UNIQUE);
    }
    if currlevel == 24 {
        add_monster_type(ctx, MT_ARCHLICH, PLACE_SCATTER);
        add_monster_type(ctx, MT_NAKRUL, PLACE_SPECIAL);
    }
    if !ctx.gendung.setlevel {
        if is_quest_available(ctx, Q_BUTCHER) {
            add_monster_type(ctx, MT_CLEAVER, PLACE_SPECIAL);
        }
        if is_quest_available(ctx, Q_GARBUD) {
            add_monster_type_unique(ctx, UniqueMonsterType::Garbud, PLACE_UNIQUE);
        }
        if is_quest_available(ctx, Q_ZHAR) {
            add_monster_type_unique(ctx, UniqueMonsterType::Zhar, PLACE_UNIQUE);
        }
        if is_quest_available(ctx, Q_LTBANNER) {
            add_monster_type_unique(ctx, UniqueMonsterType::SnotSpill, PLACE_UNIQUE);
        }
        if is_quest_available(ctx, Q_VEIL) {
            add_monster_type_unique(ctx, UniqueMonsterType::Lachdan, PLACE_UNIQUE);
        }
        if is_quest_available(ctx, Q_WARLORD) {
            add_monster_type_unique(ctx, UniqueMonsterType::WarlordOfBlood, PLACE_UNIQUE);
        }
        if crate::quests::use_multiplayer_quests(ctx) && currlevel == ctx.quests.Quests[Q_SKELKING as usize]._qlevel {
            add_monster_type(ctx, MT_SKING, PLACE_UNIQUE);
            let mut skeltypes: Vec<_monster_id> = Vec::new();
            for skeleton_type in SKELETON_TYPES {
                if !is_monster_avalible(ctx, &MonstersData[skeleton_type as usize]) {
                    continue;
                }
                skeltypes.push(skeleton_type);
            }
            let i = rnd(ctx, skeltypes.len() as i32) as usize;
            add_monster_type(ctx, skeltypes[i], PLACE_SCATTER);
        }
        let mut typelist: Vec<_monster_id> = Vec::with_capacity(MaxMonsters);
        for i in MT_NZOMBIE as usize..MonstersData.len() {
            if !is_monster_avalible(ctx, &MonstersData[i]) {
                continue;
            }
            typelist.push(i as _monster_id);
        }
        let mut nt = typelist.len();
        while nt > 0 && ctx.monster.LevelMonsterTypeCount < MaxLvlMTypes && ctx.monster.monstimgtot < 4000 {
            let mut i = 0;
            while i < nt {
                if MonstersData[typelist[i] as usize].image as i32 > 4000 - ctx.monster.monstimgtot {
                    nt -= 1;
                    typelist[i] = typelist[nt];
                    continue;
                }
                i += 1;
            }
            if nt != 0 {
                let i = rnd(ctx, nt as i32) as usize;
                add_monster_type(ctx, typelist[i], PLACE_SCATTER);
                nt -= 1;
                typelist[i] = typelist[nt];
            }
        }
    } else if ctx.gendung.setlvlnum == SL_SKELKING {
        add_monster_type(ctx, MT_SKING, PLACE_UNIQUE);
    }
}

/// Original: `devilution::InitMonsterSND` (monster.cpp).
// @port monster.cpp|devilution::InitMonsterSND(CMonster &monsterType) sha=88ad6b8273c6
pub fn init_monster_snd(ctx: &mut Ctx, type_index: usize) {
    if !ctx.sound.gb_snd_inited {
        return;
    }
    const PREFIXES: [&str; 4] = ["a", "h", "d", "s"]; // Attack, Hit, Death, Special
    let data = &MonstersData[ctx.monster.LevelMonsterTypes[type_index].type_ as usize];
    let sound_suffix = data.soundSuffix.unwrap_or(data.assetsSuffix);
    for (i, prefix) in PREFIXES.iter().enumerate() {
        if *prefix == "s" && !data.hasSpecialSound {
            continue;
        }
        for j in 0..2 {
            let path = format!("monsters\\{}{}{}.wav", sound_suffix, prefix, j + 1);
            let snd = crate::engine::sound::sound_file_load(ctx, &path, false);
            ctx.monster.LevelMonsterTypes[type_index].sounds[i][j] = Some(snd);
        }
    }
}

/// Original: `devilution::InitMonsterGFX` (monster.cpp). The original loads all animations
/// into one buffer (`animData`); here each animation owns its converted CLX data.
// @port monster.cpp|devilution::InitMonsterGFX(CMonster &monsterType) sha=55ed694adfd0
pub fn init_monster_gfx(ctx: &mut Ctx, type_index: usize) {
    let mtype = ctx.monster.LevelMonsterTypes[type_index].type_;
    let monster_data = &MonstersData[mtype as usize];
    let num_anims = get_num_anims(monster_data);
    let headless = ctx.diablo.headless_mode;
    for i in 0..num_anims {
        let has_anim = monster_data.frames[i] != 0;
        if !has_anim {
            ctx.monster.LevelMonsterTypes[type_index].anims[i].frames = 0;
            continue;
        }
        let sprites = if !headless {
            let path = format!("monsters\\{}{}", monster_data.assetsSuffix, ANIMLETTER[i] as char);
            Some(crate::engine::load_sprites::load_cl2_list_or_sheet(ctx, &path, crate::utils::cel_to_clx::Widths::Value(monster_data.width)))
        } else {
            None
        };
        let anim = &mut ctx.monster.LevelMonsterTypes[type_index].anims[i];
        anim.frames = monster_data.frames[i];
        anim.rate = monster_data.rate[i];
        anim.width = monster_data.width;
        if sprites.is_some() {
            anim.sprites = sprites;
        }
    }
    ctx.monster.LevelMonsterTypes[type_index].data = mtype as usize;
    if headless {
        return;
    }
    if monster_data.trnFile.is_some() {
        init_monster_trn(ctx, type_index);
    }
    use crate::missiles::missile_file_data_load_gfx as load;
    use MissileGraphicID as G;
    if matches!(mtype, MT_NMAGMA | MT_YMAGMA | MT_BMAGMA | MT_WMAGMA) {
        load(ctx, G::MagmaBall as usize);
    }
    if matches!(mtype, MT_STORM | MT_RSTORM | MT_STORML | MT_MAEL) {
        load(ctx, G::ThinLightning as usize);
    }
    if mtype == MT_SNOWWICH {
        load(ctx, G::BloodStarBlue as usize);
        load(ctx, G::BloodStarBlueExplosion as usize);
    }
    if mtype == MT_HLSPWN {
        load(ctx, G::BloodStarRed as usize);
        load(ctx, G::BloodStarRedExplosion as usize);
    }
    if mtype == MT_SOLBRNR {
        load(ctx, G::BloodStarYellow as usize);
        load(ctx, G::BloodStarYellowExplosion as usize);
    }
    if matches!(mtype, MT_NACID | MT_RACID | MT_BACID | MT_XACID | MT_SPIDLORD) {
        load(ctx, G::Acid as usize);
        load(ctx, G::AcidSplat as usize);
        load(ctx, G::AcidPuddle as usize);
    }
    if mtype == MT_LICH {
        load(ctx, G::OrangeFlare as usize);
        load(ctx, G::OrangeFlareExplosion as usize);
    }
    if mtype == MT_ARCHLICH {
        load(ctx, G::YellowFlare as usize);
        load(ctx, G::YellowFlareExplosion as usize);
    }
    if matches!(mtype, MT_PSYCHORB | MT_BONEDEMN) {
        load(ctx, G::BlueFlare2 as usize);
    }
    if mtype == MT_NECRMORB {
        load(ctx, G::RedFlare as usize);
        load(ctx, G::RedFlareExplosion as usize);
    }
    if mtype == MT_PSYCHORB {
        load(ctx, G::BlueFlareExplosion as usize);
    }
    if mtype == MT_BONEDEMN {
        load(ctx, G::BlueFlareExplosion2 as usize);
    }
    if mtype == MT_DIABLO {
        load(ctx, G::DiabloApocalypseBoom as usize);
    }
}

/// Original: `devilution::WeakenNaKrul` (monster.cpp).
// @port monster.cpp|devilution::WeakenNaKrul() sha=5fdaa968323a
pub fn weaken_na_krul(ctx: &mut Ctx) {
    let idx = ctx.crypt.UberDiabloMonsterIndex;
    if ctx.gendung.currlevel != 24 || idx as usize >= ctx.monster.ActiveMonsterCount {
        return;
    }
    let m = idx as usize;
    play_effect(ctx, m, MonsterSound::Death);
    let monster = &mut ctx.monster.Monsters[m];
    monster.armorClass = monster.armorClass.wrapping_sub(50);
    let hp = monster.maxHitPoints / 2;
    monster.resistance = 0;
    monster.hitPoints = hp;
    monster.maxHitPoints = hp;
}

/// Original: `devilution::InitGolems` (monster.cpp).
// @port monster.cpp|devilution::InitGolems() sha=00db0fcabe00
pub fn init_golems(ctx: &mut Ctx) {
    if !ctx.gendung.setlevel {
        for _ in 0..crate::player::MAX_PLRS {
            add_monster(ctx, GOLEM_HOLDING_CELL, Direction::South, 0, false);
        }
    }
}

/// Original: `devilution::InitMonsters` (monster.cpp).
// @port monster.cpp|devilution::InitMonsters() sha=56a659bb0bce
pub fn init_monsters(ctx: &mut Ctx) {
    if !ctx.init.gb_is_spawn && !ctx.gendung.setlevel && ctx.gendung.currlevel == 16 {
        load_diab_monsts(ctx);
    }
    let mut nt = ctx.trigs.numtrigs;
    if ctx.gendung.currlevel == 15 {
        nt = 1;
    }
    for i in 0..nt as usize {
        for s in -2..2 {
            for t in -2..2 {
                let p = ctx.trigs.trigs[i].position + Displacement::new(s, t);
                crate::lighting::do_vision(ctx, p, 15, crate::lighting::MAP_EXP_NONE, false);
            }
        }
    }
    if !ctx.init.gb_is_spawn {
        place_quest_monsters(ctx);
    }
    if !ctx.gendung.setlevel {
        if !ctx.init.gb_is_spawn {
            place_unique_monsters(ctx);
        }
        let mut na = 0;
        for s in 16..96 {
            for t in 16..96 {
                if !crate::engine::path::is_tile_solid(ctx, Point::new(s, t)) {
                    na += 1;
                }
            }
        }
        let mut numplacemonsters = na / 30;
        if ctx.init.gb_is_multiplayer {
            numplacemonsters += numplacemonsters / 2;
        }
        if ctx.monster.ActiveMonsterCount + numplacemonsters > MaxMonsters - 10 {
            numplacemonsters = MaxMonsters - 10 - ctx.monster.ActiveMonsterCount;
        }
        ctx.monster.totalmonsters = ctx.monster.ActiveMonsterCount + numplacemonsters;
        let mut scattertypes: Vec<usize> = Vec::new();
        for i in 0..ctx.monster.LevelMonsterTypeCount {
            if (ctx.monster.LevelMonsterTypes[i].placeFlags & PLACE_SCATTER) != 0 {
                scattertypes.push(i);
            }
        }
        while ctx.monster.ActiveMonsterCount < ctx.monster.totalmonsters {
            let type_index = scattertypes[rnd(ctx, scattertypes.len() as i32) as usize];
            let na;
            if ctx.gendung.currlevel == 1 || ctx.rng.flip_coin(2) {
                na = 1;
            } else if ctx.gendung.currlevel == 2 || ctx.gendung.leveltype == DungeonType::Crypt {
                na = rnd(ctx, 2) + 2;
            } else {
                na = rnd(ctx, 3) + 3;
            }
            place_group(ctx, type_index, na as u32, None, false);
        }
    }
    for i in 0..nt as usize {
        for s in -2..2 {
            for t in -2..2 {
                let p = ctx.trigs.trigs[i].position + Displacement::new(s, t);
                crate::lighting::do_un_vision(ctx, p, 15);
            }
        }
    }
}

/// Original: `devilution::SetMapMonsters` (monster.cpp).
// @port monster.cpp|devilution::SetMapMonsters(const uint16_t *dunData, Point startPosition) sha=cea3518cd34e
pub fn set_map_monsters(ctx: &mut Ctx, dun_data: &[u16], start_position: Point) {
    add_monster_type(ctx, MT_GOLEM, PLACE_SPECIAL);
    if ctx.gendung.setlevel {
        for _ in 0..crate::player::MAX_PLRS {
            add_monster(ctx, GOLEM_HOLDING_CELL, Direction::South, 0, false);
        }
    }
    if ctx.gendung.setlevel && ctx.gendung.setlvlnum == SL_VILEBETRAYER {
        add_monster_type_unique(ctx, UniqueMonsterType::Lazarus, PLACE_UNIQUE);
        add_monster_type_unique(ctx, UniqueMonsterType::RedVex, PLACE_UNIQUE);
        add_monster_type_unique(ctx, UniqueMonsterType::BlackJade, PLACE_UNIQUE);
        place_unique_monst(ctx, UniqueMonsterType::Lazarus, 0, 0);
        place_unique_monst(ctx, UniqueMonsterType::RedVex, 0, 0);
        place_unique_monst(ctx, UniqueMonsterType::BlackJade, 0, 0);
    }
    let mut width = dun_data[0] as usize;
    let mut height = dun_data[1] as usize;
    let layer2_offset = 2 + width * height;
    // The rest of the layers are at dPiece scale
    width *= 2;
    height *= 2;
    let monster_layer = &dun_data[layer2_offset + width * height..];
    for j in 0..height {
        for i in 0..width {
            let monster_id = monster_layer[j * width + i] as u8;
            if monster_id != 0 {
                let type_index = add_monster_type(ctx, crate::tables::monstdat::MonstConvTbl[(monster_id - 1) as usize], PLACE_SPECIAL);
                let amc = ctx.monster.ActiveMonsterCount;
                ctx.monster.ActiveMonsterCount += 1;
                place_monster(ctx, amc, type_index, start_position + Displacement::new(i as i32, j as i32));
            }
        }
    }
}

/// Original: `devilution::AddMonster` (monster.cpp).
// @port monster.cpp|devilution::AddMonster(Point position, Direction dir, size_t typeIndex, bool inMap) sha=1437ff6edc62
pub fn add_monster(ctx: &mut Ctx, position: Point, dir: Direction, type_index: usize, in_map: bool) -> Option<usize> {
    if ctx.monster.ActiveMonsterCount < MaxMonsters {
        let m = ctx.monster.ActiveMonsters[ctx.monster.ActiveMonsterCount] as usize;
        ctx.monster.ActiveMonsterCount += 1;
        if in_map {
            ctx.gendung.dMonster[position.x as usize][position.y as usize] = (m + 1) as i16;
        }
        init_monster(ctx, m, dir, type_index, position);
        return Some(m);
    }
    None
}

/// Original: `devilution::AddDoppelganger` (monster.cpp).
// @port monster.cpp|devilution::AddDoppelganger(Monster &monster) sha=0c2dac99d3dc
pub fn add_doppelganger(ctx: &mut Ctx, m: usize) {
    let mut target = Point::new(0, 0);
    for d in 0..8 {
        let position = ctx.monster.Monsters[m].position.tile + Direction::from_u8(d);
        if !is_tile_available(ctx, position) {
            continue;
        }
        target = position;
    }
    if target != Point::new(0, 0) {
        let type_index = get_monster_type_index(ctx, monster_type_id(ctx, m));
        let d = ctx.monster.Monsters[m].direction;
        add_monster(ctx, target, d, type_index, true);
    }
}

/// Original: `devilution::ApplyMonsterDamage` (monster.cpp).
// @port monster.cpp|devilution::ApplyMonsterDamage(DamageType damageType, Monster &monster, int damage) sha=f193c0ff78ff
pub fn apply_monster_damage(ctx: &mut Ctx, damage_type: DamageType, m: usize, damage: i32) {
    crate::qol::floatingnumbers::add_floating_number_monster(ctx, damage_type, m, damage);
    ctx.monster.Monsters[m].hitPoints -= damage;
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let tile = ctx.monster.Monsters[m].position.tile;
    if ctx.monster.Monsters[m].hitPoints >> 6 <= 0 {
        crate::msg::delta_kill_monster(ctx, m, tile, me);
        crate::msg::net_send_cmd_loc_param1(ctx, false, CMD_MONSTDEATH, tile, m as u16);
        return;
    }
    crate::msg::delta_monster_hp(ctx, m, me);
    crate::msg::net_send_cmd_mon_dmg(ctx, false, m as u16, damage as u32);
}

/// Original: `devilution::M_Talker` (monster.cpp).
// @port monster.cpp|devilution::M_Talker(const Monster &monster) sha=724a334aea9a
pub fn m_talker(ctx: &Ctx, m: usize) -> bool {
    matches!(
        ctx.monster.Monsters[m].ai,
        MonsterAIID::Lazarus | MonsterAIID::Warlord | MonsterAIID::Gharbad | MonsterAIID::Zhar | MonsterAIID::Snotspill | MonsterAIID::Lachdanan | MonsterAIID::LazarusSuccubus
    )
}

/// Original: `devilution::M_StartStand` (monster.cpp).
// @port monster.cpp|devilution::M_StartStand(Monster &monster, Direction md) sha=28472ce683fd
pub fn m_start_stand(ctx: &mut Ctx, m: usize, md: Direction) {
    clear_m_vars(&mut ctx.monster.Monsters[m]);
    if monster_type_id(ctx, m) == MT_GOLEM {
        new_monster_anim(ctx, m, MonsterGraphic::Walk, md);
    } else {
        new_monster_anim(ctx, m, MonsterGraphic::Stand, md);
    }
    let monster = &mut ctx.monster.Monsters[m];
    monster.var1 = monster.mode as i16;
    monster.var2 = 0;
    monster.mode = MonsterMode::Stand;
    monster.position.future = monster.position.tile;
    monster.position.old = monster.position.tile;
    update_enemy(ctx, m);
}

/// Original: `devilution::M_ClearSquares` (monster.cpp).
// @port monster.cpp|devilution::M_ClearSquares(const Monster &monster) sha=25bbaa448ea0
pub fn m_clear_squares(ctx: &mut Ctx, m: usize) {
    let old = ctx.monster.Monsters[m].position.old;
    for search_tile in crate::engine::geometry::points_in_rectangle(crate::engine::geometry::Rectangle::from_center(old, 1)) {
        if find_monster_at_position(ctx, search_tile, false) == Some(m) {
            ctx.gendung.dMonster[search_tile.x as usize][search_tile.y as usize] = 0;
        }
    }
}

/// Original: `devilution::M_GetKnockback` (monster.cpp).
// @port monster.cpp|devilution::M_GetKnockback(Monster &monster) sha=cb4ab971a19c
pub fn m_get_knockback(ctx: &mut Ctx, m: usize) {
    let dir = opposite(ctx.monster.Monsters[m].direction);
    let old = ctx.monster.Monsters[m].position.old;
    if !is_relative_move_ok(ctx, m, old, dir) {
        return;
    }
    m_clear_squares(ctx, m);
    ctx.monster.Monsters[m].position.old = old + dir;
    start_monster_got_hit(ctx, m);
}

/// Original: `devilution::M_StartHit(Monster &, int)` (monster.cpp).
// @port monster.cpp|devilution::M_StartHit(Monster &monster, int dam) sha=4eecaea6a204
pub fn m_start_hit_dam(ctx: &mut Ctx, m: usize, dam: i32) {
    play_effect(ctx, m, MonsterSound::Hit);
    if is_hard_hit(ctx, m, dam as u32) {
        let t = monster_type_id(ctx, m);
        if t == MT_BLINK {
            teleport(ctx, m);
        } else if matches!(t, MT_NSCAV | MT_BSCAV | MT_WSCAV | MT_YSCAV | MT_GRAVEDIG) {
            let monster = &mut ctx.monster.Monsters[m];
            monster.goal = MonsterGoal::Normal;
            monster.goalVar1 = 0;
            monster.goalVar2 = 0;
        }
        if ctx.monster.Monsters[m].mode != MonsterMode::Petrified {
            start_monster_got_hit(ctx, m);
        }
    }
}

/// Original: `devilution::M_StartHit(Monster &, const Player &, int)` (monster.cpp).
// @port monster.cpp|devilution::M_StartHit(Monster &monster, const Player &player, int dam) sha=cde68394bc8f
pub fn m_start_hit(ctx: &mut Ctx, m: usize, pnum: usize, dam: i32) {
    tag(ctx, m, pnum);
    if is_hard_hit(ctx, m, dam as u32) {
        let future = ctx.players.Players[pnum].position.future;
        {
            let monster = &mut ctx.monster.Monsters[m];
            monster.enemy = pnum as u8;
            monster.enemyPosition = future;
            monster.flags &= !(MFLAG_TARGETS_MONSTER as u32);
        }
        if ctx.monster.Monsters[m].mode != MonsterMode::Petrified {
            ctx.monster.Monsters[m].direction = get_monster_direction(ctx, m);
        }
    }
    m_start_hit_dam(ctx, m, dam);
}

/// Original: `devilution::MonsterDeath(Monster &, Direction, bool)` (monster.cpp).
// @port monster.cpp|devilution::MonsterDeath(Monster &monster, Direction md, bool sendmsg) sha=1b1e2988c969
pub fn monster_death_dir(ctx: &mut Ctx, m: usize, mut md: Direction, sendmsg: bool) {
    let difficulty = ctx.multi.sgGameInitInfo.nDifficulty;
    if !ctx.monster.Monsters[m].is_player_minion() {
        let (lvl, exp, who) = (monster_level(ctx, m, difficulty) as i32, monster_exp(ctx, m, difficulty) as i32, ctx.monster.Monsters[m].whoHit);
        crate::player::add_plr_monst_exper(ctx, lvl, exp, who);
    }
    let t = monster_type_id(ctx, m);
    ctx.monster.MonsterKillCounts[t as usize] += 1;
    {
        let monster = &mut ctx.monster.Monsters[m];
        monster.hitPoints = 0;
        monster.flags &= !(MFLAG_HIDDEN as u32);
    }
    let seed = ctx.monster.Monsters[m].rndItemSeed;
    ctx.rng.set_rnd_seed(seed);
    spawn_loot(ctx, m, sendmsg);
    if t == MT_DIABLO {
        diablo_death(ctx, m, true);
    } else {
        play_effect(ctx, m, MonsterSound::Death);
    }
    if ctx.monster.Monsters[m].mode != MonsterMode::Petrified {
        if t == MT_GOLEM {
            md = Direction::South;
        }
        let flags = if ctx.diablo.g_game_logic_step < GameLogicStep::ProcessMonsters { AnimationDistributionFlags::ProcessAnimationPending } else { AnimationDistributionFlags::None };
        new_monster_anim_full(ctx, m, MonsterGraphic::Death, md, flags, 0, 0);
        ctx.monster.Monsters[m].mode = MonsterMode::Death;
    } else if ctx.monster.Monsters[m].is_unique() {
        let l = ctx.monster.Monsters[m].lightId as i32;
        crate::lighting::add_un_light(ctx, l);
    }
    {
        let monster = &mut ctx.monster.Monsters[m];
        monster.goal = MonsterGoal::None;
        monster.var1 = 0;
        monster.position.tile = monster.position.old;
        monster.position.future = monster.position.old;
    }
    m_clear_squares(ctx, m);
    let tile = ctx.monster.Monsters[m].position.tile;
    ctx.gendung.dMonster[tile.x as usize][tile.y as usize] = (m + 1) as i16;
    crate::quests::check_quest_kill(ctx, m, sendmsg);
    m_fallen_fear(ctx, tile);
    if matches!(t, MT_NACID | MT_RACID | MT_BACID | MT_XACID | MT_SPIDLORD) {
        let intel = ctx.monster.Monsters[m].intelligence as i32;
        crate::missiles::add_missile(ctx, tile, Point::new(0, 0), Direction::South, MissileID::AcidPuddle, TARGET_PLAYERS, m as i32, intel + 1, 0, None);
    }
}

/// Original: `devilution::StartMonsterDeath` (monster.cpp).
// @port monster.cpp|devilution::StartMonsterDeath(Monster &monster, const Player &player, bool sendmsg) sha=4d504bc47104
pub fn start_monster_death(ctx: &mut Ctx, m: usize, pnum: usize, sendmsg: bool) {
    tag(ctx, m, pnum);
    let md = crate::engine::get_direction(ctx.monster.Monsters[m].position.tile, ctx.players.Players[pnum].position.tile);
    monster_death_dir(ctx, m, md, sendmsg);
}

/// Original: `devilution::KillMyGolem` (monster.cpp).
// @port monster.cpp|devilution::KillMyGolem() sha=7dcdea95efed
pub fn kill_my_golem(ctx: &mut Ctx) {
    let id = ctx.players.MyPlayerId;
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let tile = ctx.monster.Monsters[id].position.tile;
    crate::msg::delta_kill_monster(ctx, id, tile, me);
    crate::msg::net_send_cmd_loc(ctx, id, false, CMD_KILLGOLEM, tile);
    m_start_kill(ctx, id, me);
}

/// Original: `devilution::M_StartKill` (monster.cpp).
// @port monster.cpp|devilution::M_StartKill(Monster &monster, const Player &player) sha=0ae95462c9bf
pub fn m_start_kill(ctx: &mut Ctx, m: usize, pnum: usize) {
    start_monster_death(ctx, m, pnum, true);
}

/// Original: `devilution::M_SyncStartKill` (monster.cpp).
// @port monster.cpp|devilution::M_SyncStartKill(Monster &monster, Point position, const Player &player) sha=1fee15739878
pub fn m_sync_start_kill(ctx: &mut Ctx, m: usize, position: Point, pnum: usize) {
    if ctx.monster.Monsters[m].hitPoints == 0 || ctx.monster.Monsters[m].mode == MonsterMode::Death {
        return;
    }
    if ctx.gendung.dMonster[position.x as usize][position.y as usize] == 0 {
        m_clear_squares(ctx, m);
        ctx.monster.Monsters[m].position.tile = position;
        ctx.monster.Monsters[m].position.old = position;
    }
    start_monster_death(ctx, m, pnum, false);
}

/// Original: `devilution::M_UpdateRelations` (monster.cpp).
// @port monster.cpp|devilution::M_UpdateRelations(const Monster &monster) sha=26eba119ee40
pub fn m_update_relations(ctx: &mut Ctx, m: usize) {
    if has_leashed_minions(ctx, m) {
        release_minions(ctx, m);
    }
    shrink_leader_packsize(ctx, m);
}

/// Original: `devilution::DoEnding` (monster.cpp).
// @port monster.cpp|devilution::DoEnding() sha=d67e21189601
pub fn do_ending(ctx: &mut Ctx) {
    if ctx.init.gb_is_multiplayer {
        crate::storm::storm_net::snet_leave_game(ctx, crate::storm::storm_net::LEAVE_ENDING);
    }
    crate::engine::sound::music_stop(ctx);
    if ctx.init.gb_is_multiplayer {
        ctx.platform.delay(1000);
    }
    if ctx.init.gb_is_spawn {
        return;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    match ctx.players.Players[me]._pClass {
        HeroClass::Sorcerer | HeroClass::Monk => crate::movie::play_movie(ctx, "gendata\\diabvic1.smk", false),
        HeroClass::Warrior | HeroClass::Barbarian => crate::movie::play_movie(ctx, "gendata\\diabvic2.smk", false),
        _ => crate::movie::play_movie(ctx, "gendata\\diabvic3.smk", false),
    }
    crate::movie::play_movie(ctx, "gendata\\diabend.smk", false);
    let b_music_on = ctx.sound.gb_music_on;
    ctx.sound.gb_music_on = true;
    let music_volume = crate::engine::sound::sound_get_or_set_music_volume(ctx, 1);
    crate::engine::sound::sound_get_or_set_music_volume(ctx, 0);
    crate::engine::sound::music_start(ctx, crate::engine::sound::TMUSIC_CATACOMBS as u8);
    ctx.movie.loop_movie = true;
    crate::movie::play_movie(ctx, "gendata\\loopdend.smk", true);
    ctx.movie.loop_movie = false;
    crate::engine::sound::music_stop(ctx);
    crate::engine::sound::sound_get_or_set_music_volume(ctx, music_volume);
    ctx.sound.gb_music_on = b_music_on;
}

/// Original: `devilution::PrepDoEnding` (monster.cpp).
// @port monster.cpp|devilution::PrepDoEnding() sha=7437c1d258dd
pub fn prep_do_ending(ctx: &mut Ctx) {
    ctx.sound.gb_sound_on = ctx.monster.sgbSaveSoundOn;
    ctx.diablo.gb_run_game = false;
    ctx.players.MyPlayerIsDead = false;
    ctx.diablo.cineflag = true;
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let lvl = (ctx.multi.sgGameInitInfo.nDifficulty as u8).wrapping_add(1);
    ctx.players.Players[me].pDiabloKillLevel = ctx.players.Players[me].pDiabloKillLevel.max(lvl);
    let mp = ctx.init.gb_is_multiplayer;
    for player in ctx.players.Players.iter_mut() {
        player._pmode = PM_QUIT;
        player._pInvincible = true;
        if mp {
            if player._pHitPoints >> 6 == 0 {
                player._pHitPoints = 64;
            }
            if player._pMana >> 6 == 0 {
                player._pMana = 64;
            }
        }
    }
}

/// Original: `devilution::Walk` (monster.cpp).
// @port monster.cpp|devilution::Walk(Monster &monster, Direction md) sha=16ae11f4edc9
pub fn walk(ctx: &mut Ctx, m: usize, md: Direction) -> bool {
    if !dir_ok(ctx, m, md) {
        return false;
    }
    match md {
        Direction::North => walk_northwards(ctx, m, -1, -1, Direction::North),
        Direction::NorthEast => walk_northwards(ctx, m, 0, -1, Direction::NorthEast),
        Direction::East => walk_sideways(ctx, m, -32, -16, 1, -1, 1, 0, Direction::East),
        Direction::SouthEast => walk_southwards(ctx, m, -32, -16, 1, 0, Direction::SouthEast),
        Direction::South => walk_southwards(ctx, m, 0, -32, 1, 1, Direction::South),
        Direction::SouthWest => walk_southwards(ctx, m, 32, -16, 0, 1, Direction::SouthWest),
        Direction::West => walk_sideways(ctx, m, 32, -16, -1, 1, 0, 1, Direction::West),
        Direction::NorthWest => walk_northwards(ctx, m, -1, 0, Direction::NorthWest),
        Direction::NoDirection => {}
    }
    true
}

/// Original: `devilution::GolumAi` (monster.cpp).
// @port monster.cpp|devilution::GolumAi(Monster &golem) sha=824a9d2d9e36
pub fn golum_ai(ctx: &mut Ctx, golem: usize) {
    let tile = ctx.monster.Monsters[golem].position.tile;
    if tile.x == 1 && tile.y == 0 {
        return;
    }
    if matches!(ctx.monster.Monsters[golem].mode, MonsterMode::Death | MonsterMode::SpecialStand) || is_walking(ctx, golem) {
        return;
    }
    if (ctx.monster.Monsters[golem].flags & MFLAG_TARGETS_MONSTER as u32) == 0 {
        update_enemy(ctx, golem);
    }
    if ctx.monster.Monsters[golem].mode == MonsterMode::MeleeAttack {
        return;
    }
    if (ctx.monster.Monsters[golem].flags & MFLAG_NO_ENEMY as u32) == 0 {
        let enemy = ctx.monster.Monsters[golem].enemy as usize;
        let (e_future, e_tile) = (ctx.monster.Monsters[enemy].position.future, ctx.monster.Monsters[enemy].position.tile);
        let tile = ctx.monster.Monsters[golem].position.tile;
        let mex = tile.x - e_future.x;
        let mey = tile.y - e_future.y;
        ctx.monster.Monsters[golem].direction = crate::engine::get_direction(tile, e_tile);
        if mex.abs() < 2 && mey.abs() < 2 {
            ctx.monster.Monsters[golem].enemyPosition = e_tile;
            if ctx.monster.Monsters[enemy].activeForTicks == 0 {
                ctx.monster.Monsters[enemy].activeForTicks = u8::MAX;
                ctx.monster.Monsters[enemy].position.last = tile;
                for j in 0..5 {
                    for k in 0..5 {
                        let mx = tile.x + k - 2;
                        let my = tile.y + j - 2;
                        if !in_dungeon_bounds(Point::new(mx, my)) {
                            continue;
                        }
                        let enemy_id = ctx.gendung.dMonster[mx as usize][my as usize] as i32;
                        if enemy_id > 0 {
                            ctx.monster.Monsters[(enemy_id - 1) as usize].activeForTicks = u8::MAX;
                        }
                    }
                }
            }
            start_attack(ctx, golem);
            return;
        }
        if ai_plan_path(ctx, golem) {
            return;
        }
    }
    {
        let g = &mut ctx.monster.Monsters[golem];
        g.pathCount += 1;
        if g.pathCount > 8 {
            g.pathCount = 5;
        }
    }
    let pdir = ctx.players.Players[golem]._pdir;
    if random_walk(ctx, golem, pdir) {
        return;
    }
    let mut md = left(ctx.monster.Monsters[golem].direction);
    for _ in 0..8 {
        md = right(md);
        if walk(ctx, golem, md) {
            break;
        }
    }
}

/// Original: `devilution::DirOK` (monster.cpp).
// @port monster.cpp|devilution::DirOK(const Monster &monster, Direction mdir) sha=b75745c2f65c
pub fn dir_ok(ctx: &Ctx, m: usize, mdir: Direction) -> bool {
    let position = ctx.monster.Monsters[m].position.tile;
    let future_position = position + mdir;
    if !is_relative_move_ok(ctx, m, position, mdir) {
        return false;
    }
    if ctx.monster.Monsters[m].leaderRelation == LeaderRelation::Leashed {
        let l = get_leader(ctx, m).expect("leader");
        return future_position.walking_distance(ctx.monster.Monsters[l].position.future) < 4;
    }
    if !has_leashed_minions(ctx, m) {
        return true;
    }
    let mut mcount = 0;
    for x in future_position.x - 3..=future_position.x + 3 {
        for y in future_position.y - 3..=future_position.y + 3 {
            if !in_dungeon_bounds(Point::new(x, y)) {
                continue;
            }
            let Some(minion) = find_monster_at_position(ctx, Point::new(x, y), true) else {
                continue;
            };
            if ctx.monster.Monsters[minion].leaderRelation == LeaderRelation::Leashed && get_leader(ctx, minion) == Some(m) {
                mcount += 1;
            }
        }
    }
    mcount == ctx.monster.Monsters[m].packSize as i32
}

/// Original: `devilution::PosOkMissile` (monster.cpp).
// @port monster.cpp|devilution::PosOkMissile(Point position) sha=b0a31b6eb0f1
pub fn pos_ok_missile(ctx: &Ctx, position: Point) -> bool {
    !crate::levels::gendung::tile_has_any(ctx, ctx.gendung.dPiece[position.x as usize][position.y as usize] as i32, TileProperties::BlockMissile)
}

/// Original: `devilution::LineClearMissile` (monster.cpp).
// @port monster.cpp|devilution::LineClearMissile(Point startPoint, Point endPoint) sha=1976c3defdbf
pub fn line_clear_missile(ctx: &Ctx, start_point: Point, end_point: Point) -> bool {
    line_clear(ctx, &|ctx: &Ctx, p: Point| pos_ok_missile(ctx, p), start_point, end_point)
}

/// Original: `devilution::LineClear` (monster.cpp).
// @port monster.cpp|devilution::LineClear(tl::function_ref<bool(Point)> clear, Point startPoint, Point endPoint) sha=d0d824b1895c
pub fn line_clear(ctx: &Ctx, clear: &dyn Fn(&Ctx, Point) -> bool, start_point: Point, mut end_point: Point) -> bool {
    let mut position = start_point;
    let mut dx = end_point.x - position.x;
    let mut dy = end_point.y - position.y;
    if dx.abs() > dy.abs() {
        if dx < 0 {
            std::mem::swap(&mut position, &mut end_point);
            dx = -dx;
            dy = -dy;
        }
        let (mut d, yinc_d, dinc_d, dinc_h);
        if dy > 0 {
            d = 2 * dy - dx;
            dinc_d = 2 * dy;
            dinc_h = 2 * (dy - dx);
            yinc_d = 1;
        } else {
            d = 2 * dy + dx;
            dinc_d = 2 * dy;
            dinc_h = 2 * (dx + dy);
            yinc_d = -1;
        }
        let mut done = false;
        while !done && position != end_point {
            if (d <= 0) ^ (yinc_d < 0) {
                d += dinc_d;
            } else {
                d += dinc_h;
                position.y += yinc_d;
            }
            position.x += 1;
            done = position != start_point && !clear(ctx, position);
        }
    } else {
        if dy < 0 {
            std::mem::swap(&mut position, &mut end_point);
            dy = -dy;
            dx = -dx;
        }
        let (mut d, xinc_d, dinc_d, dinc_h);
        if dx > 0 {
            d = 2 * dx - dy;
            dinc_d = 2 * dx;
            dinc_h = 2 * (dx - dy);
            xinc_d = 1;
        } else {
            d = 2 * dx + dy;
            dinc_d = 2 * dx;
            dinc_h = 2 * (dy + dx);
            xinc_d = -1;
        }
        let mut done = false;
        while !done && position != end_point {
            if (d <= 0) ^ (xinc_d < 0) {
                d += dinc_d;
            } else {
                d += dinc_h;
                position.x += xinc_d;
            }
            position.y += 1;
            done = position != start_point && !clear(ctx, position);
        }
    }
    position == end_point
}

/// Original: `devilution::SyncMonsterAnim` (monster.cpp), release build.
// @port monster.cpp|devilution::SyncMonsterAnim(Monster &monster) sha=497bbb536156
pub fn sync_monster_anim(ctx: &mut Ctx, m: usize) {
    if ctx.monster.Monsters[m].is_unique() {
        init_trn_for_unique_monster(ctx, m);
    }
    let mut graphic = MonsterGraphic::Stand;
    match get_visual_monster_mode(ctx, m) {
        MonsterMode::Stand | MonsterMode::Delay | MonsterMode::Talk => {}
        MonsterMode::MoveNorthwards | MonsterMode::MoveSouthwards | MonsterMode::MoveSideways => graphic = MonsterGraphic::Walk,
        MonsterMode::MeleeAttack | MonsterMode::RangedAttack => graphic = MonsterGraphic::Attack,
        MonsterMode::HitRecovery => graphic = MonsterGraphic::GotHit,
        MonsterMode::Death => graphic = MonsterGraphic::Death,
        MonsterMode::SpecialMeleeAttack | MonsterMode::FadeIn | MonsterMode::FadeOut | MonsterMode::SpecialStand | MonsterMode::SpecialRangedAttack | MonsterMode::Heal => graphic = MonsterGraphic::Special,
        MonsterMode::Charge => {
            graphic = MonsterGraphic::Attack;
            ctx.monster.Monsters[m].animInfo.currentFrame = 0;
        }
        _ => {
            ctx.monster.Monsters[m].animInfo.currentFrame = 0;
        }
    }
    change_animation_data(ctx, m, graphic);
}

/// Original: `devilution::M_FallenFear` (monster.cpp).
// @port monster.cpp|devilution::M_FallenFear(Point position) sha=0e01f19e0727
pub fn m_fallen_fear(ctx: &mut Ctx, position: Point) {
    let fear_area = crate::engine::geometry::Rectangle::from_center(position, 4);
    for tile in crate::engine::geometry::points_in_rectangle(fear_area) {
        if !in_dungeon_bounds(tile) {
            continue;
        }
        let mi = ctx.gendung.dMonster[tile.x as usize][tile.y as usize] as i32;
        if mi == 0 {
            continue;
        }
        let m = (mi.abs() - 1) as usize;
        let monster = &ctx.monster.Monsters[m];
        if monster.ai != MonsterAIID::Fallen || monster.hitPoints >> 6 <= 0 {
            continue;
        }
        let run_distance = (8 - monster_data(ctx, m).level as i32).max(2);
        let dir = crate::engine::get_direction(position, monster.position.tile);
        let monster = &mut ctx.monster.Monsters[m];
        monster.goal = MonsterGoal::Retreat;
        monster.goalVar1 = run_distance as i16;
        monster.goalVar2 = dir as i8;
    }
}

/// Original: `devilution::PrintMonstHistory` (monster.cpp).
// @port monster.cpp|devilution::PrintMonstHistory(int mt) sha=9e79eaf1a345
pub fn print_monst_history(ctx: &mut Ctx, mt: _monster_id) {
    use crate::control::add_panel_string;
    let mtu = mt as usize;
    let kills = ctx.monster.MonsterKillCounts[mtu];
    if ctx.options.gameplay.show_monster_type.get() {
        let s = tr("Type: {:s}  Kills: {:d}").replacen("{:s}", &get_monster_type_text(&MonstersData[mtu]), 1).replacen("{:d}", &kills.to_string(), 1);
        add_panel_string(ctx, &s);
    } else {
        add_panel_string(ctx, &tr("Total kills: {:d}").replacen("{:d}", &kills.to_string(), 1));
    }
    let (hf, mp) = (ctx.init.gb_is_hellfire, ctx.init.gb_is_multiplayer);
    let difficulty = ctx.multi.sgGameInitInfo.nDifficulty;
    if kills >= 30 {
        let mut min_hp = MonstersData[mtu].hitPointsMinimum as i32;
        let mut max_hp = MonstersData[mtu].hitPointsMaximum as i32;
        if !hf && mt == MT_DIABLO {
            min_hp /= 2;
            max_hp /= 2;
        }
        if !mp {
            min_hp /= 2;
            max_hp /= 2;
        }
        min_hp = min_hp.max(1);
        max_hp = max_hp.max(1);
        let mut hp_bonus_nightmare = 100;
        let mut hp_bonus_hell = 200;
        if hf {
            hp_bonus_nightmare = if !mp { 50 } else { 100 };
            hp_bonus_hell = if !mp { 100 } else { 200 };
        }
        if difficulty == DIFF_NIGHTMARE {
            min_hp = 3 * min_hp + hp_bonus_nightmare;
            max_hp = 3 * max_hp + hp_bonus_nightmare;
        } else if difficulty == DIFF_HELL {
            min_hp = 4 * min_hp + hp_bonus_hell;
            max_hp = 4 * max_hp + hp_bonus_hell;
        }
        let s = tr("Hit Points: {:d}-{:d}").replacen("{:d}", &min_hp.to_string(), 1).replacen("{:d}", &max_hp.to_string(), 1);
        add_panel_string(ctx, &s);
    }
    if kills >= 15 {
        let res = if difficulty != DIFF_HELL { MonstersData[mtu].resistance } else { MonstersData[mtu].resistanceHell } as i32;
        let (rm, rf, rl, im, ifi, il) = (RESIST_MAGIC as i32, RESIST_FIRE as i32, RESIST_LIGHTNING as i32, IMMUNE_MAGIC as i32, IMMUNE_FIRE as i32, IMMUNE_LIGHTNING as i32);
        if (res & (rm | rf | rl | im | ifi | il)) == 0 {
            add_panel_string(ctx, &tr("No magic resistance"));
        } else {
            if (res & (rm | rf | rl)) != 0 {
                let mut resists = tr("Resists:");
                if (res & rm) != 0 {
                    resists.push_str(&tr(" Magic"));
                }
                if (res & rf) != 0 {
                    resists.push_str(&tr(" Fire"));
                }
                if (res & rl) != 0 {
                    resists.push_str(&tr(" Lightning"));
                }
                add_panel_string(ctx, &resists);
            }
            if (res & (im | ifi | il)) != 0 {
                let mut immune = tr("Immune:");
                if (res & im) != 0 {
                    immune.push_str(&tr(" Magic"));
                }
                if (res & ifi) != 0 {
                    immune.push_str(&tr(" Fire"));
                }
                if (res & il) != 0 {
                    immune.push_str(&tr(" Lightning"));
                }
                add_panel_string(ctx, &immune);
            }
        }
    }
}

/// Original: `devilution::PrintUniqueHistory` (monster.cpp).
// @port monster.cpp|devilution::PrintUniqueHistory() sha=0596522c9d82
pub fn print_unique_history(ctx: &mut Ctx) {
    use crate::control::add_panel_string;
    let m = ctx.cursor.pcursmonst as usize;
    if ctx.options.gameplay.show_monster_type.get() {
        let s = tr("Type: {:s}").replacen("{:s}", &get_monster_type_text(monster_data(ctx, m)), 1);
        add_panel_string(ctx, &s);
    }
    let all = (RESIST_MAGIC | RESIST_FIRE | RESIST_LIGHTNING | IMMUNE_MAGIC | IMMUNE_FIRE | IMMUNE_LIGHTNING) as u16;
    let res = ctx.monster.Monsters[m].resistance & all;
    if res == 0 {
        add_panel_string(ctx, &tr("No resistances"));
        add_panel_string(ctx, &tr("No Immunities"));
    } else {
        if (res & (RESIST_MAGIC | RESIST_FIRE | RESIST_LIGHTNING) as u16) != 0 {
            add_panel_string(ctx, &tr("Some Magic Resistances"));
        } else {
            add_panel_string(ctx, &tr("No resistances"));
        }
        if (res & (IMMUNE_MAGIC | IMMUNE_FIRE | IMMUNE_LIGHTNING) as u16) != 0 {
            add_panel_string(ctx, &tr("Some Magic Immunities"));
        } else {
            add_panel_string(ctx, &tr("No Immunities"));
        }
    }
}

/// Original: `devilution::PlayEffect` (monster.cpp).
// @port monster.cpp|devilution::PlayEffect(Monster &monster, MonsterSound mode) sha=1fda96402f12
pub fn play_effect(ctx: &mut Ctx, m: usize, mode: MonsterSound) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.players.Players[me].pLvlLoad != 0 {
        return;
    }
    let snd_idx = rnd(ctx, 2) as usize;
    if !ctx.sound.gb_snd_inited || !ctx.sound.gb_sound_on || ctx.msg.gbBufferMsgs != 0 {
        return;
    }
    let level_type = ctx.monster.Monsters[m].levelType as usize;
    let playing = match ctx.monster.LevelMonsterTypes[level_type].sounds[mode as usize][snd_idx].as_ref() {
        None => return,
        Some(s) => s.is_playing(),
    };
    if playing {
        return;
    }
    let mut l_volume = 0;
    let mut l_pan = 0;
    let tile = ctx.monster.Monsters[m].position.tile;
    if !crate::engine::sound::calculate_sound_position(ctx, tile, &mut l_volume, &mut l_pan) {
        return;
    }
    let mut snd = ctx.monster.LevelMonsterTypes[level_type].sounds[mode as usize][snd_idx].take();
    crate::engine::sound::snd_play_snd(ctx, snd.as_deref_mut(), l_volume, l_pan);
    ctx.monster.LevelMonsterTypes[level_type].sounds[mode as usize][snd_idx] = snd;
}

/// Original: `devilution::MissToMonst` (monster.cpp).
// @port monster.cpp|devilution::MissToMonst(Missile &missile, Point position) sha=eaa1fc298beb
pub fn miss_to_monst(ctx: &mut Ctx, mi: usize, position: Point) {
    let monster_id = ctx.missiles.Missiles[mi]._misource as usize;
    assert!(monster_id < MaxMonsters);
    let m = monster_id;
    let old_position = ctx.missiles.Missiles[mi].position.tile;
    ctx.gendung.dMonster[position.x as usize][position.y as usize] = (monster_id + 1) as i16;
    ctx.monster.Monsters[m].direction = Direction::from_u8(ctx.missiles.Missiles[mi]._mimfnum as u8);
    ctx.monster.Monsters[m].position.tile = position;
    let d = ctx.monster.Monsters[m].direction;
    m_start_stand(ctx, m, d);
    m_start_hit_dam(ctx, m, 0);
    let t = monster_type_id(ctx, m);
    if t == MT_GLOOM {
        return;
    }
    let is_snake = matches!(t, MT_NSNAKE | MT_RSNAKE | MT_BSNAKE | MT_GSNAKE);
    let (mind, maxd) = (ctx.monster.Monsters[m].minDamageSpecial as i32, ctx.monster.Monsters[m].maxDamageSpecial as i32);
    if (ctx.monster.Monsters[m].flags & MFLAG_TARGETS_MONSTER as u32) == 0 {
        let dp = ctx.gendung.dPlayer[old_position.x as usize][old_position.y as usize];
        if dp <= 0 {
            return;
        }
        let pnum = (dp - 1) as usize;
        monster_attack_player(ctx, m, pnum, 500, mind, maxd);
        if is_snake {
            return;
        }
        let pmode = ctx.players.Players[pnum]._pmode;
        if pmode != PM_GOTHIT && pmode != PM_DEATH {
            crate::player::start_plr_hit(ctx, pnum, 0, true);
        }
        let new_position = old_position + ctx.monster.Monsters[m].direction;
        if crate::player::pos_ok_player(ctx, pnum, new_position) {
            ctx.players.Players[pnum].position.tile = new_position;
            let pdir = ctx.players.Players[pnum]._pdir;
            crate::player::fix_player_location(ctx, pnum, pdir);
            crate::player::fix_plr_walk_tags(ctx, pnum);
            ctx.gendung.dPlayer[new_position.x as usize][new_position.y as usize] = (pnum + 1) as i8;
            crate::player::set_player_old(ctx, pnum);
        }
        return;
    }
    let Some(target) = find_monster_at_position(ctx, old_position, true) else {
        return;
    };
    monster_attack_monster(ctx, m, target, 500, mind, maxd);
    if is_snake {
        return;
    }
    let new_position = old_position + ctx.monster.Monsters[m].direction;
    if is_tile_available_for_monster(ctx, target, new_position) {
        let monster_id = ctx.gendung.dMonster[old_position.x as usize][old_position.y as usize];
        ctx.gendung.dMonster[new_position.x as usize][new_position.y as usize] = monster_id;
        ctx.gendung.dMonster[old_position.x as usize][old_position.y as usize] = 0;
        let monster = &mut ctx.monster.Monsters[m];
        monster.position.tile = new_position;
        monster.position.future = new_position;
    }
}

/// Original: `devilution::FindMonsterAtPosition` (monster.cpp).
// @port monster.cpp|devilution::FindMonsterAtPosition(Point position, bool ignoreMovingMonsters) sha=a85a9d3efb5e
pub fn find_monster_at_position(ctx: &Ctx, position: Point, ignore_moving_monsters: bool) -> Option<usize> {
    if !in_dungeon_bounds(position) {
        return None;
    }
    let monster_id = ctx.gendung.dMonster[position.x as usize][position.y as usize];
    if monster_id == 0 || (ignore_moving_monsters && monster_id < 0) {
        // nothing at this position
        return None;
    }
    Some((monster_id as i32).unsigned_abs() as usize - 1)
}

/// Original: `devilution::FindUniqueMonster` (monster.cpp).
// @port monster.cpp|devilution::FindUniqueMonster(UniqueMonsterType monsterType) sha=b6ccb67ca31a
pub fn find_unique_monster(ctx: &Ctx, monster_type: UniqueMonsterType) -> Option<usize> {
    for i in 0..ctx.monster.ActiveMonsterCount {
        let monster_id = ctx.monster.ActiveMonsters[i] as usize;
        if ctx.monster.Monsters[monster_id].uniqueType == monster_type {
            return Some(monster_id);
        }
    }
    None
}

/// Original: `devilution::IsTileAvailable(const Monster &, Point)` (monster.cpp).
// @port monster.cpp|devilution::IsTileAvailable(const Monster &monster, Point position) sha=f34ce8d1fce4
pub fn is_tile_available_for_monster(ctx: &Ctx, m: usize, position: Point) -> bool {
    if !is_tile_available(ctx, position) {
        return false;
    }
    is_tile_safe(ctx, m, position)
}

/// Original: `devilution::IsSkel` (monster.cpp).
// @port monster.cpp|devilution::IsSkel(_monster_id mt) sha=eab3fc1ccba5
pub fn is_skel(mt: _monster_id) -> bool {
    SKELETON_TYPES.contains(&mt)
}

/// Original: `devilution::IsGoat` (monster.cpp).
// @port monster.cpp|devilution::IsGoat(_monster_id mt) sha=73df7ee6c5f2
pub fn is_goat(mt: _monster_id) -> bool {
    matches!(mt, MT_NGOATMC | MT_BGOATMC | MT_RGOATMC | MT_GGOATMC | MT_NGOATBW | MT_BGOATBW | MT_RGOATBW | MT_GGOATBW)
}

/// Original: `devilution::ActivateSkeleton` (monster.cpp).
// @port monster.cpp|devilution::ActivateSkeleton(Monster &monster, Point position) sha=a34b843f9ad6
pub fn activate_skeleton(ctx: &mut Ctx, m: usize, position: Point) {
    if is_tile_available(ctx, position) {
        activate_spawn(ctx, m, position, Direction::SouthWest);
        return;
    }
    const SPAWN_DIRECTIONS: [Direction; 8] = [
        Direction::North,
        Direction::NorthEast,
        Direction::East,
        Direction::NorthWest,
        Direction::SouthEast,
        Direction::West,
        Direction::SouthWest,
        Direction::South,
    ];
    let mut spawn_ok = [false; 8];
    for i in 0..SPAWN_DIRECTIONS.len() {
        if is_tile_available(ctx, position + SPAWN_DIRECTIONS[i]) {
            spawn_ok[i] = true;
        }
    }
    let count = spawn_ok.iter().filter(|&&b| b).count() as i32;
    if count == 0 {
        return;
    }
    // this is used in the following loop to find the nth set bit.
    let mut spawn_choice = rnd(ctx, 15) % count;
    for i in 0..spawn_ok.len() {
        if !spawn_ok[i] {
            continue;
        }
        if spawn_choice > 0 {
            spawn_choice -= 1;
            continue;
        }
        activate_spawn(ctx, m, position + SPAWN_DIRECTIONS[i], opposite(SPAWN_DIRECTIONS[i]));
        return;
    }
}

/// Original: `devilution::PreSpawnSkeleton` (monster.cpp).
// @port monster.cpp|devilution::PreSpawnSkeleton() sha=f5d491a0b3fb
pub fn pre_spawn_skeleton(ctx: &mut Ctx) -> Option<usize> {
    let skeleton = add_skeleton(ctx, Point::new(0, 0), Direction::South, false);
    if let Some(s) = skeleton {
        m_start_stand(ctx, s, Direction::South);
    }
    skeleton
}

/// Original: `devilution::TalktoMonster` (monster.cpp).
// @port monster.cpp|devilution::TalktoMonster(Player &player, Monster &monster) sha=65265d84b962
pub fn talkto_monster(ctx: &mut Ctx, pnum: usize, m: usize) {
    use crate::quests::is_quest_available;
    let is_me = Some(pnum) == ctx.players.MyPlayer;
    if is_me {
        ctx.monster.Monsters[m].mode = MonsterMode::Talk;
    }
    let unique = ctx.monster.Monsters[m].uniqueType;
    if unique == UniqueMonsterType::SnotSpill && is_quest_available(ctx, Q_LTBANNER) && ctx.quests.Quests[Q_LTBANNER as usize]._qvar1 == 2 && crate::inv::remove_inventory_item_by_id(ctx, pnum, IDI_BANNER) {
        ctx.quests.Quests[Q_LTBANNER as usize]._qactive = QUEST_DONE;
        ctx.monster.Monsters[m].talkMsg = TEXT_BANNER12;
        ctx.monster.Monsters[m].goal = MonsterGoal::Inquiring;
        crate::msg::net_send_cmd_quest(ctx, true, Q_LTBANNER as usize);
    }
    if unique == UniqueMonsterType::Lachdan
        && is_quest_available(ctx, Q_VEIL)
        && ctx.monster.Monsters[m].talkMsg >= TEXT_VEIL9
        && crate::inv::remove_inventory_item_by_id(ctx, pnum, IDI_GLDNELIX)
        && (ctx.monster.Monsters[m].flags & MFLAG_QUEST_COMPLETE as u32) == 0
    {
        {
            let monster = &mut ctx.monster.Monsters[m];
            monster.talkMsg = TEXT_VEIL11;
            monster.goal = MonsterGoal::Inquiring;
            monster.flags |= MFLAG_QUEST_COMPLETE as u32;
        }
        if is_me {
            let p = ctx.monster.Monsters[m].position.tile + Direction::South;
            crate::items::spawn_unique(ctx, UITEM_STEELVEIL, p, None, true, false);
            ctx.quests.Quests[Q_VEIL as usize]._qvar2 = QS_VEIL_ITEM_SPAWNED as u8;
            crate::msg::net_send_cmd_quest(ctx, true, Q_VEIL as usize);
        }
    }
    if unique == UniqueMonsterType::Zhar && ctx.monster.Monsters[m].talkMsg == TEXT_ZHAR1 && (ctx.monster.Monsters[m].flags & MFLAG_QUEST_COMPLETE as u32) == 0 && is_me {
        ctx.quests.Quests[Q_ZHAR as usize]._qactive = QUEST_ACTIVE;
        ctx.quests.Quests[Q_ZHAR as usize]._qlog = true;
        ctx.quests.Quests[Q_ZHAR as usize]._qvar1 = QS_ZHAR_ITEM_SPAWNED as u8;
        let p = ctx.monster.Monsters[m].position.tile + Displacement::new(1, 1);
        crate::items::create_type_item(ctx, p, false, ItemType::Misc, IMISC_BOOK as i32, false, false, true);
        ctx.monster.Monsters[m].flags |= MFLAG_QUEST_COMPLETE as u32;
        crate::msg::net_send_cmd_quest(ctx, true, Q_ZHAR as usize);
    }
    if unique == UniqueMonsterType::Garbud && is_me {
        if ctx.monster.Monsters[m].talkMsg == TEXT_GARBUD1 {
            ctx.quests.Quests[Q_GARBUD as usize]._qactive = QUEST_ACTIVE;
            ctx.quests.Quests[Q_GARBUD as usize]._qlog = true;
            crate::msg::net_send_cmd_quest(ctx, true, Q_GARBUD as usize);
        }
        if ctx.monster.Monsters[m].talkMsg == TEXT_GARBUD2 && (ctx.monster.Monsters[m].flags & MFLAG_QUEST_COMPLETE as u32) == 0 {
            let p = ctx.monster.Monsters[m].position.tile + Displacement::new(1, 1);
            crate::items::spawn_item(ctx, m, p, false, true);
            ctx.monster.Monsters[m].flags |= MFLAG_QUEST_COMPLETE as u32;
            ctx.quests.Quests[Q_GARBUD as usize]._qvar1 = QS_GHARBAD_FIRST_ITEM_SPAWNED as u8;
            crate::msg::net_send_cmd_quest(ctx, true, Q_GARBUD as usize);
        }
    }
}

/// Original: `devilution::SpawnGolem` (monster.cpp).
// @port monster.cpp|devilution::SpawnGolem(Player &player, Monster &golem, Point position, Missile &missile) sha=5969adc4635a
pub fn spawn_golem(ctx: &mut Ctx, pnum: usize, golem: usize, position: Point, mi: usize) {
    ctx.gendung.dMonster[position.x as usize][position.y as usize] = (golem + 1) as i16;
    let spllvl = ctx.missiles.Missiles[mi]._mispllvl;
    let (max_mana, plevel) = (ctx.players.Players[pnum]._pMaxMana, ctx.players.Players[pnum]._pLevel as i32);
    {
        let g = &mut ctx.monster.Monsters[golem];
        g.position.tile = position;
        g.position.future = position;
        g.position.old = position;
        g.pathCount = 0;
        g.maxHitPoints = 2 * (320 * spllvl + max_mana / 3);
        g.hitPoints = g.maxHitPoints;
        g.armorClass = 25;
        g.toHit = (5 * (spllvl + 8) + 2 * plevel) as u16;
        g.minDamage = (2 * (spllvl + 4)) as u8;
        g.maxDamage = (2 * (spllvl + 8)) as u8;
        g.flags |= MFLAG_GOLEM as u32;
    }
    start_special_stand(ctx, golem, Direction::South);
    update_enemy(ctx, golem);
    if Some(pnum) == ctx.players.MyPlayer {
        let g = ctx.monster.Monsters[golem].clone();
        let lvl = crate::msg::get_level_for_multiplayer(ctx, pnum);
        crate::msg::net_send_cmd_golem(ctx, g.position.tile.x as u8, g.position.tile.y as u8, g.direction, g.enemy, g.hitPoints, lvl);
    }
}

/// Original: `devilution::CanTalkToMonst` (monster.cpp).
// @port monster.cpp|devilution::CanTalkToMonst(const Monster &monster) sha=a46c52871dea
pub fn can_talk_to_monst(ctx: &Ctx, m: usize) -> bool {
    matches!(ctx.monster.Monsters[m].goal, MonsterGoal::Inquiring | MonsterGoal::Talking)
}

/// Original: `devilution::encode_enemy` (monster.cpp).
// @port monster.cpp|devilution::encode_enemy(Monster &monster) sha=aa8f40c66ae4
pub fn encode_enemy(ctx: &Ctx, m: usize) -> u8 {
    let monster = &ctx.monster.Monsters[m];
    if (monster.flags & MFLAG_TARGETS_MONSTER as u32) != 0 {
        return monster.enemy.wrapping_add(crate::player::MAX_PLRS as u8);
    }
    monster.enemy
}

/// Original: `devilution::decode_enemy` (monster.cpp).
// @port monster.cpp|devilution::decode_enemy(Monster &monster, int enemyId) sha=b91a439a5483
pub fn decode_enemy(ctx: &mut Ctx, m: usize, mut enemy_id: i32) {
    if enemy_id < crate::player::MAX_PLRS as i32 {
        let future = ctx.players.Players[enemy_id as usize].position.future;
        let monster = &mut ctx.monster.Monsters[m];
        monster.flags &= !(MFLAG_TARGETS_MONSTER as u32);
        monster.enemy = enemy_id as u8;
        monster.enemyPosition = future;
    } else {
        enemy_id -= crate::player::MAX_PLRS as i32;
        let future = ctx.monster.Monsters[enemy_id as usize].position.future;
        let monster = &mut ctx.monster.Monsters[m];
        monster.flags |= MFLAG_TARGETS_MONSTER as u32;
        monster.enemy = enemy_id as u8;
        monster.enemyPosition = future;
    }
}

/// Original: `Monster::getLeader` (monster.cpp).
// @port monster.cpp|devilution::Monster::getLeader() sha=f7c3e1303215
pub fn get_leader(ctx: &Ctx, m: usize) -> Option<usize> {
    let leader = ctx.monster.Monsters[m].leader;
    if leader == Monster::NoLeader {
        return None;
    }
    Some(leader as usize)
}

/// Original: `Monster::setLeader` (monster.cpp).
// @port monster.cpp|devilution::Monster::setLeader(const Monster *leader) sha=48c348ac4d1e
pub fn set_leader(ctx: &mut Ctx, m: usize, leader: Option<usize>) {
    let Some(l) = leader else {
        // really we should update this->leader to NoLeader to avoid leaving a dangling reference to a dead monster
        // when passed nullptr. So that buffed minions are drawn with a distinct colour in monhealthbar we leave the
        // reference and hope that no code tries to modify the leader through this instance later.
        ctx.monster.Monsters[m].leaderRelation = LeaderRelation::None;
        return;
    };
    let ai = ctx.monster.Monsters[l].ai;
    let monster = &mut ctx.monster.Monsters[m];
    monster.leader = l as u8;
    monster.leaderRelation = LeaderRelation::Leashed;
    monster.ai = ai;
}

/// Original: `Monster::hasLeashedMinions` (monster.h).
// @port monster.h|devilution::Monster::hasLeashedMinions() sha=d26ca0bad6b0
pub fn has_leashed_minions(ctx: &Ctx, m: usize) -> bool {
    let monster = &ctx.monster.Monsters[m];
    monster.is_unique() && UniqueMonstersData[monster.uniqueType.0 as usize].monsterPack == UniqueMonsterPack::Leashed
}

/// Original: `Monster::distanceToEnemy` (monster.cpp).
// @port monster.cpp|devilution::Monster::distanceToEnemy() sha=53293a04a39b
pub fn distance_to_enemy(ctx: &Ctx, m: usize) -> u32 {
    let monster = &ctx.monster.Monsters[m];
    let mx = monster.position.tile.x - monster.enemyPosition.x;
    let my = monster.position.tile.y - monster.enemyPosition.y;
    mx.abs().max(my.abs()) as u32
}

/// Original: `Monster::checkStandAnimationIsLoaded` (monster.cpp).
// @port monster.cpp|devilution::Monster::checkStandAnimationIsLoaded(Direction mdir) sha=d071eab33640
pub fn check_stand_animation_is_loaded(ctx: &mut Ctx, m: usize, mdir: Direction) {
    if matches!(ctx.monster.Monsters[m].mode, MonsterMode::Stand | MonsterMode::Talk) {
        ctx.monster.Monsters[m].direction = mdir;
        change_animation_data(ctx, m, MonsterGraphic::Stand);
    }
}

/// Original: `Monster::petrify` (monster.cpp).
// @port monster.cpp|devilution::Monster::petrify() sha=512d147d4754
pub fn petrify(ctx: &mut Ctx, m: usize) {
    let monster = &mut ctx.monster.Monsters[m];
    monster.mode = MonsterMode::Petrified;
    monster.animInfo.isPetrified = true;
}

/// Original: `Monster::isImmune` (monster.cpp).
// @port monster.cpp|devilution::Monster::isImmune(MissileID missileType, DamageType missileElement) sha=fe2c7616b581
pub fn is_immune(ctx: &Ctx, m: usize, missile_type: MissileID, missile_element: DamageType) -> bool {
    let res = ctx.monster.Monsters[m].resistance;
    if ((res & IMMUNE_MAGIC as u16) != 0 && missile_element == DamageType::Magic)
        || ((res & IMMUNE_FIRE as u16) != 0 && missile_element == DamageType::Fire)
        || ((res & IMMUNE_LIGHTNING as u16) != 0 && missile_element == DamageType::Lightning)
        || ((res & IMMUNE_ACID as u16) != 0 && missile_element == DamageType::Acid)
    {
        return true;
    }
    missile_type == MissileID::HolyBolt && monster_type_id(ctx, m) != MT_DIABLO && monster_data(ctx, m).monsterClass != MonsterClass::Undead
}

/// Original: `Monster::isResistant` (monster.cpp).
// @port monster.cpp|devilution::Monster::isResistant(MissileID missileType, DamageType missileElement) sha=6ca45d82fb5e
pub fn is_resistant(ctx: &Ctx, m: usize, missile_type: MissileID, missile_element: DamageType) -> bool {
    let res = ctx.monster.Monsters[m].resistance;
    if ((res & RESIST_MAGIC as u16) != 0 && missile_element == DamageType::Magic)
        || ((res & RESIST_FIRE as u16) != 0 && missile_element == DamageType::Fire)
        || ((res & RESIST_LIGHTNING as u16) != 0 && missile_element == DamageType::Lightning)
    {
        return true;
    }
    ctx.init.gb_is_hellfire && missile_type == MissileID::HolyBolt && matches!(monster_type_id(ctx, m), MT_DIABLO | MT_BONEDEMN)
}

/// Original: `Monster::isPossibleToHit` (monster.cpp).
// @port monster.cpp|devilution::Monster::isPossibleToHit() sha=fb0ccae93c9f
pub fn is_possible_to_hit(ctx: &Ctx, m: usize) -> bool {
    let monster = &ctx.monster.Monsters[m];
    let t = monster_type_id(ctx, m);
    !(monster.hitPoints >> 6 <= 0
        || monster.talkMsg != TEXT_NONE
        || (t == MT_ILLWEAV && monster.goal == MonsterGoal::Retreat)
        || monster.mode == MonsterMode::Charge
        || (matches!(t, MT_COUNSLR | MT_MAGISTR | MT_CABALIST | MT_ADVOCATE) && monster.goal != MonsterGoal::Normal))
}

/// Original: `Monster::tag` (monster.cpp).
// @port monster.cpp|devilution::Monster::tag(const Player &tagger) sha=0b970950defe
pub fn tag(ctx: &mut Ctx, m: usize, pnum: usize) {
    ctx.monster.Monsters[m].whoHit |= (1i32 << pnum) as i8;
}

/// Original: `Monster::tryLiftGargoyle` (monster.cpp).
// @port monster.cpp|devilution::Monster::tryLiftGargoyle() sha=57eeb5054176
pub fn try_lift_gargoyle(ctx: &mut Ctx, m: usize) -> bool {
    let monster = &mut ctx.monster.Monsters[m];
    if monster.ai == MonsterAIID::Gargoyle && (monster.flags & MFLAG_ALLOW_SPECIAL as u32) != 0 {
        monster.flags &= !(MFLAG_ALLOW_SPECIAL as u32);
        monster.mode = MonsterMode::SpecialMeleeAttack;
        return true;
    }
    false
}
