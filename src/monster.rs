//! `Source/monster.cpp`: monsters (only the level's monster-type graphics so far).

use std::rc::Rc;

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteListOrSheet;
use crate::engine::sound::TSnd;

pub const MAX_LVL_M_TYPES: usize = 24;

/// `AnimStruct` (only the sprites so far).
#[derive(Default)]
pub struct AnimStruct {
    pub sprites: Option<ClxSpriteListOrSheet>,
}

/// `CMonster` (only the loaded resources so far).
#[derive(Default)]
pub struct CMonster {
    /// `animData`
    pub anim_data: Option<Rc<[u8]>>,
    pub anims: [AnimStruct; 6],
    pub sounds: [[Option<Box<TSnd>>; 2]; 4],
}

/// Globals of monster.cpp.
pub struct MonsterState {
    /// `LevelMonsterTypes`
    pub level_monster_types: Vec<CMonster>,
}

impl Default for MonsterState {
    fn default() -> Self {
        MonsterState { level_monster_types: (0..MAX_LVL_M_TYPES).map(|_| CMonster::default()).collect() }
    }
}

/// Original: `devilution::FreeMonsters` (monster.cpp).
// @port monster.cpp|devilution::FreeMonsters() sha=924991731d90
pub fn free_monsters(ctx: &mut Ctx) {
    for monster_type in ctx.monster.level_monster_types.iter_mut() {
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
