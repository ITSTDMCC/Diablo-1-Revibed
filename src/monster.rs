//! `Source/monster.cpp`: monsters (data model; most logic is pending).

use std::rc::Rc;

use crate::ctx::Ctx;
use crate::engine::actor_position::ActorPosition;
use crate::engine::animationinfo::AnimationInfo;
use crate::engine::clx_sprite::{ClxSpriteList, ClxSpriteListOrSheet};
use crate::engine::geometry::{Direction, Point};
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
    pub fn is_unique(&self) -> bool {
        self.uniqueType != UniqueMonsterType::None
    }

    /// Original: `Monster::isPlayerMinion` (monster.cpp).
    // @port monster.cpp|devilution::Monster::isPlayerMinion() sha=b5440c18f430
    pub fn is_player_minion(&self) -> bool {
        (self.flags & MFLAG_GOLEM as u32) != 0 && (self.flags & MFLAG_BERSERK as u32) == 0
    }
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
        }
    }
}

/// `Monster::type()`
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
pub fn monster_data(ctx: &Ctx, m: usize) -> &'static MonsterData {
    &MonstersData[monster_type(ctx, m).data]
}

/// `Monster::level(difficulty)`
pub fn monster_level(ctx: &Ctx, m: usize, difficulty: _difficulty) -> u32 {
    let mon = &ctx.monster.Monsters[m];
    let mut base_level = monster_data(ctx, m).level as u32;
    if mon.is_unique() {
        base_level = UniqueMonstersData[mon.uniqueType as i8 as usize].mlevel as u32;
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
    if mon.is_unique() && UniqueMonstersData[mon.uniqueType as u8 as usize].customToHit != 0 {
        base_to_hit_special = UniqueMonstersData[mon.uniqueType as u8 as usize].customToHit as u32;
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

crate::pending_fn!(pub fn sync_monster_anim(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::SyncMonsterAnim(Monster &monster)");
crate::pending_fn!(pub fn is_diablo_alive(ctx: &mut Ctx, play_sfx: bool) -> bool, "monster.cpp|devilution::IsDiabloAlive(bool playSFX)");

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

crate::pending_fn!(pub fn set_map_monsters(ctx: &mut Ctx, dun_data: &[u16], start_position: Point), "monster.cpp|devilution::SetMapMonsters(const uint16_t *dunData, Point startPosition)");
crate::pending_fn!(pub fn find_monster_at_position(ctx: &Ctx, position: Point, ignore_movement_check: bool) -> Option<usize>, "monster.cpp|devilution::FindMonsterAtPosition(Point position, bool ignoreMovementCheck)");
crate::pending_fn!(pub fn can_talk_to_monst(ctx: &Ctx, m: usize) -> bool, "monster.cpp|devilution::CanTalkToMonst(const Monster &monster)");
crate::pending_fn!(pub fn talkto_monster(ctx: &mut Ctx, pnum: usize, m: usize), "monster.cpp|devilution::TalktoMonster(Player &player, Monster &monster)");
crate::pending_fn!(pub fn is_possible_to_hit(ctx: &Ctx, m: usize) -> bool, "monster.cpp|devilution::Monster::isPossibleToHit()");
crate::pending_fn!(pub fn try_lift_gargoyle(ctx: &mut Ctx, m: usize) -> bool, "monster.cpp|devilution::Monster::tryLiftGargoyle()");
crate::pending_fn!(pub fn apply_monster_damage(ctx: &mut Ctx, damage_type: DamageType, m: usize, damage: i32), "monster.cpp|devilution::ApplyMonsterDamage(DamageType damageType, Monster &monster, int damage)");
crate::pending_fn!(pub fn m_start_kill(ctx: &mut Ctx, m: usize, pnum: usize), "monster.cpp|devilution::M_StartKill(Monster &monster, const Player &player)");
crate::pending_fn!(pub fn m_start_hit(ctx: &mut Ctx, m: usize, pnum: usize, dam: i32), "monster.cpp|devilution::M_StartHit(Monster &monster, const Player &player, int dam)");
crate::pending_fn!(pub fn m_get_knockback(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::M_GetKnockback(Monster &monster)");
crate::pending_fn!(pub fn add_doppelganger(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::AddDoppelganger(Monster &monster)");
crate::pending_fn!(pub fn kill_my_golem(ctx: &mut Ctx), "monster.cpp|devilution::KillMyGolem()");

/// `GolemHoldingCell`
pub const GOLEM_HOLDING_CELL: Point = Point::new(1, 0);

crate::pending_fn!(pub fn m_talker(ctx: &Ctx, m: usize) -> bool, "monster.cpp|devilution::M_Talker(const Monster &monster)");
crate::pending_fn!(pub fn find_unique_monster(ctx: &Ctx, unique_type: UniqueMonsterType) -> Option<usize>, "monster.cpp|devilution::FindUniqueMonster(UniqueMonsterType monsterType)");
crate::pending_fn!(pub fn m_clear_squares(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::M_ClearSquares(const Monster &monster)");
crate::pending_fn!(pub fn decode_enemy(ctx: &mut Ctx, m: usize, enemy_id: i32), "monster.cpp|devilution::decode_enemy(Monster &monster, int enemyId)");
crate::pending_fn!(pub fn encode_enemy(ctx: &Ctx, m: usize) -> u8, "monster.cpp|devilution::encode_enemy(Monster &monster)");
crate::pending_fn!(pub fn m_start_stand(ctx: &mut Ctx, m: usize, md: Direction), "monster.cpp|devilution::M_StartStand(Monster &monster, Direction md)");
crate::pending_fn!(pub fn m_sync_start_kill(ctx: &mut Ctx, m: usize, position: Point, pnum: usize), "monster.cpp|devilution::M_SyncStartKill(Monster &monster, Point position, const Player &player)");
crate::pending_fn!(pub fn walk(ctx: &mut Ctx, m: usize, md: Direction) -> bool, "monster.cpp|devilution::Walk(Monster &monster, Direction md)");
crate::pending_fn!(pub fn dir_ok(ctx: &Ctx, m: usize, mdir: Direction) -> bool, "monster.cpp|devilution::DirOK(const Monster &monster, Direction mdir)");
crate::pending_fn!(pub fn tag(ctx: &mut Ctx, m: usize, pnum: usize), "monster.cpp|devilution::Monster::tag(const Player &tagger)");
crate::pending_fn!(pub fn golum_ai(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::GolumAi(Monster &golem)");
crate::pending_fn!(pub fn m_update_relations(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::M_UpdateRelations(const Monster &monster)");
crate::pending_fn!(pub fn weaken_na_krul(ctx: &mut Ctx), "monster.cpp|devilution::WeakenNaKrul()");

crate::pending_fn!(pub fn init_monsters(ctx: &mut crate::ctx::Ctx), "monster.cpp|devilution::InitMonsters()");

crate::pending_fn!(pub fn init_golems(ctx: &mut crate::ctx::Ctx), "monster.cpp|devilution::InitGolems()");

crate::pending_fn!(pub fn get_level_m_types(ctx: &mut crate::ctx::Ctx), "monster.cpp|devilution::GetLevelMTypes()");


crate::pending_fn!(pub fn prep_do_ending(ctx: &mut crate::ctx::Ctx), "monster.cpp|devilution::PrepDoEnding()");

crate::pending_fn!(pub fn do_ending(ctx: &mut crate::ctx::Ctx), "monster.cpp|devilution::DoEnding()");


/// Original: `devilution::Monster::name` (monster.h).
// @port monster.h|devilution::Monster::name() sha=4eb516b96c3e
pub fn monster_name(ctx: &crate::ctx::Ctx, m: usize) -> String {
    let mon = &ctx.monster.Monsters[m];
    if mon.uniqueType != UniqueMonsterType::None {
        return crate::utils::language::pgettext("monster", UniqueMonstersData[mon.uniqueType as i8 as usize].mName);
    }
    crate::utils::language::pgettext("monster", monster_data(ctx, m).name)
}

crate::pending_fn!(pub fn print_monst_history(ctx: &mut Ctx, mt: _monster_id), "monster.cpp|devilution::PrintMonstHistory(int mt)");
crate::pending_fn!(pub fn print_unique_history(ctx: &mut Ctx), "monster.cpp|devilution::PrintUniqueHistory()");

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

crate::pending_fn!(fn follow_the_leader(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::FollowTheLeader(Monster &monster)");
crate::pending_fn!(fn update_enemy(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::UpdateEnemy(Monster &monster)");
crate::pending_fn!(fn ai_plan_path(ctx: &mut Ctx, m: usize) -> bool, "monster.cpp|devilution::AiPlanPath(Monster &monster)");
crate::pending_fn!(fn ai_proc(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::AiProc");
crate::pending_fn!(fn update_mode_stance(ctx: &mut Ctx, m: usize) -> bool, "monster.cpp|devilution::UpdateModeStance(Monster &monster)");
crate::pending_fn!(fn group_unity(ctx: &mut Ctx, m: usize), "monster.cpp|devilution::GroupUnity(Monster &monster)");

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
