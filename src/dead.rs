//! `Source/dead.cpp`: placing dead monsters.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::{ClxSpriteList, ClxSpriteListOrSheet};
use crate::engine::geometry::{Direction, Point};

/// `MaxCorpses`
pub const MaxCorpses: usize = 31;

/// `Corpse`
#[derive(Clone, Default)]
pub struct Corpse {
    pub sprites: Option<ClxSpriteListOrSheet>,
    pub frame: i32,
    pub width: u16,
    pub translationPaletteIndex: u8,
}

impl Corpse {
    /// `spritesForDirection`
    // @port dead.h|devilution::Corpse::spritesForDirection(Direction direction) sha=bfa2752e9df8
    pub fn sprites_for_direction(&self, direction: Direction) -> ClxSpriteList {
        let s = self.sprites.as_ref().expect("corpse sprites");
        if s.is_sheet() {
            s.sheet().get(direction as usize)
        } else {
            s.list().clone()
        }
    }
}

/// Globals of dead.cpp.
#[derive(Default)]
pub struct DeadState {
    /// `Corpses`
    pub Corpses: [Corpse; MaxCorpses],
    /// `stonendx`
    pub stonendx: i8,
}

/// Original: `InitDeadAnimationFromMonster` (dead.cpp).
// @port dead.cpp|devilution::InitDeadAnimationFromMonster(Corpse &corpse, const CMonster &mon) sha=06b25bf5b2b5
fn init_dead_animation_from_monster(ctx: &mut Ctx, nd: usize, type_index: usize) {
    let anim_data = ctx.monster.LevelMonsterTypes[type_index].anims[crate::enums::MonsterGraphic::Death as usize].clone();
    let corpse = &mut ctx.dead.Corpses[nd];
    corpse.sprites = anim_data.sprites;
    corpse.frame = anim_data.frames as i32 - 1;
    corpse.width = anim_data.width;
}

/// Original: `MoveLightToCorpse` (dead.cpp).
// @port dead.cpp|devilution::MoveLightToCorpse(Monster &monster) sha=1db8f6926c37
fn move_light_to_corpse(ctx: &mut Ctx, m: usize) {
    use crate::levels::gendung::{MAXDUNX, MAXDUNY};
    let (corpse_id, light_id) = (ctx.monster.Monsters[m].corpseId, ctx.monster.Monsters[m].lightId as i32);
    for dx in 0..MAXDUNX {
        for dy in 0..MAXDUNY {
            if (ctx.gendung.dCorpse[dx][dy] & 0x1F) == corpse_id {
                crate::lighting::change_light_xy(ctx, light_id, Point::new(dx as i32, dy as i32));
                return;
            }
        }
    }
    crate::lighting::add_un_light(ctx, light_id);
}

/// Original: `devilution::InitCorpses` (dead.cpp).
// @port dead.cpp|devilution::InitCorpses() sha=4301cd839094
pub fn init_corpses(ctx: &mut Ctx) {
    let mut mtypes = [0i8; crate::monster::MaxMonsters];
    let mut nd: i8 = 0;
    for i in 0..ctx.monster.LevelMonsterTypeCount {
        let t = ctx.monster.LevelMonsterTypes[i].type_ as usize;
        if mtypes[t] != 0 {
            continue;
        }
        init_dead_animation_from_monster(ctx, nd as usize, i);
        ctx.dead.Corpses[nd as usize].translationPaletteIndex = 0;
        nd += 1;
        ctx.monster.LevelMonsterTypes[i].corpseId = nd;
        mtypes[t] = nd;
    }
    nd += 1; // Unused blood spatter
    if !ctx.diablo.headless_mode {
        let sprites = crate::missiles::get_missile_sprite_data(ctx, crate::enums::MissileGraphicID::StoneCurseShatter).sprites.clone();
        ctx.dead.Corpses[nd as usize].sprites = sprites;
    }
    let c = &mut ctx.dead.Corpses[nd as usize];
    c.frame = 11;
    c.width = 128;
    c.translationPaletteIndex = 0;
    nd += 1;
    ctx.dead.stonendx = nd;
    for i in 0..ctx.monster.ActiveMonsterCount {
        let m = ctx.monster.ActiveMonsters[i] as usize;
        if ctx.monster.Monsters[m].is_unique() {
            let lt = ctx.monster.Monsters[m].levelType as usize;
            init_dead_animation_from_monster(ctx, nd as usize, lt);
            ctx.dead.Corpses[nd as usize].translationPaletteIndex = (m + 1) as u8;
            nd += 1;
            ctx.monster.Monsters[m].corpseId = nd;
        }
    }
    assert!(nd as usize <= MaxCorpses);
}

/// Original: `devilution::AddCorpse` (dead.cpp).
// @port dead.cpp|devilution::AddCorpse(Point tilePosition, int8_t dv, Direction ddir) sha=e657944d10d6
pub fn add_corpse(ctx: &mut Ctx, tile: Point, dv: i8, ddir: Direction) {
    ctx.gendung.dCorpse[tile.x as usize][tile.y as usize] = ((dv as i32 & 0x1F) + ((ddir as i32) << 5)) as i8;
}

/// Original: `devilution::MoveLightsToCorpses` (dead.cpp).
// @port dead.cpp|devilution::MoveLightsToCorpses() sha=71bf2b4f1212
pub fn move_lights_to_corpses(ctx: &mut Ctx) {
    for i in 0..ctx.monster.ActiveMonsterCount {
        let m = ctx.monster.ActiveMonsters[i] as usize;
        if !ctx.monster.Monsters[m].is_unique() {
            continue;
        }
        move_light_to_corpse(ctx, m);
    }
}
