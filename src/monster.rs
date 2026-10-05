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
crate::pending_fn!(pub fn delete_monster_list(ctx: &mut Ctx), "monster.cpp|devilution::DeleteMonsterList()");
