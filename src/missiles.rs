//! `Source/missiles.cpp` and `Source/misdat.cpp`: missiles.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::{ClxSpriteList, ClxSpriteListOrSheet};
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::enums::*;

pub const TARGET_MONSTERS: mienemy_type = 0;
pub const TARGET_PLAYERS: mienemy_type = 1;
pub const TARGET_BOTH: mienemy_type = 2;

/// `MissilePosition`
#[derive(Clone, Copy, Debug, Default)]
pub struct MissilePosition {
    pub tile: Point,
    pub start: Point,
    pub offset: Displacement,
    pub velocity: Displacement,
    pub traveled: Displacement,
    /// `tileForRendering`
    pub tileForRendering: Point,
    /// `offsetForRendering`
    pub offsetForRendering: Displacement,
}

/// `Missile`
#[derive(Clone, Debug, Default)]
pub struct Missile {
    pub _mitype: MissileID,
    pub position: MissilePosition,
    pub _mimfnum: i32,
    pub _mispllvl: i32,
    pub _miDelFlag: bool,
    pub _miAnimType: MissileGraphicID,
    pub _miAnimFlags: MissileGraphicsFlags,
    pub _miAnimData: Option<ClxSpriteList>,
    pub _miAnimDelay: i32,
    pub _miAnimLen: i32,
    pub _miAnimWidth: u16,
    pub _miAnimWidth2: i16,
    pub _miAnimCnt: i32,
    pub _miAnimAdd: i32,
    pub _miAnimFrame: i32,
    pub _miDrawFlag: bool,
    pub _miLightFlag: bool,
    pub _miPreFlag: bool,
    pub _miUniqTrans: u32,
    pub _mirange: i32,
    pub _misource: i32,
    pub _micaster: mienemy_type,
    pub _midam: i32,
    pub _miHitFlag: bool,
    pub _midist: i32,
    pub _mlid: i32,
    pub _mirnd: i32,
    pub var1: i32,
    pub var2: i32,
    pub var3: i32,
    pub var4: i32,
    pub var5: i32,
    pub var6: i32,
    pub var7: i32,
    pub limitReached: bool,
    pub lastCollisionTargetHash: i16,
}

/// `AddMissileParameter`
#[derive(Clone, Copy, Debug)]
pub struct AddMissileParameter {
    pub dst: Point,
    pub midir: Direction,
    /// `pParent` (index into `Missiles`)
    pub pParent: Option<usize>,
    pub spellFizzled: bool,
}

/// `MissileData::mAddProc`
pub type AddProc = fn(&mut Ctx, usize, &mut AddMissileParameter);
/// `MissileData::mProc`
pub type MissileProc = fn(&mut Ctx, usize);

/// `MissileData`
pub struct MissileData {
    pub mAddProc: Option<AddProc>,
    pub mProc: Option<MissileProc>,
    pub mlSFX: crate::effects_data::SfxId,
    pub miSFX: crate::effects_data::SfxId,
    pub mFileNum: MissileGraphicID,
    pub flags: MissileDataFlags,
    pub movementDistribution: MissileMovementDistribution,
}

impl MissileData {
    /// `isDrawn`
    // @port misdat.h|devilution::MissileData::isDrawn() sha=f0044e1f16f7
    pub fn is_drawn(&self) -> bool {
        (self.flags.0 & MissileDataFlags::Invisible.0) == 0
    }
    /// `isArrow`
    // @port misdat.h|devilution::MissileData::isArrow() sha=086b04386186
    pub fn is_arrow(&self) -> bool {
        (self.flags.0 & MissileDataFlags::Arrow.0) != 0
    }
    /// `damageType`
    // @port misdat.h|devilution::MissileData::damageType() sha=f300ad9de99c
    pub fn damage_type(&self) -> DamageType {
        DamageType::from_raw(self.flags.0 & 0b111)
    }
}

/// `GetMissileData` (misdat.h)
// @port misdat.h|devilution::GetMissileData(MissileID missileId) sha=08505e02f174
pub fn get_missile_data(missile_id: MissileID) -> &'static MissileData {
    &crate::tables::misdat::MissilesData[missile_id as usize]
}

/// `MissileFileData`: the loaded sprites; the static fields are `tables::misdat::MissileSpriteInfo`.
#[derive(Default)]
pub struct MissileFileData {
    pub sprites: Option<ClxSpriteListOrSheet>,
    /// `animWidth`, `animWidth2`, `animFAmt`, `flags` (copied from the table for convenience)
    pub animWidth: u16,
    pub animWidth2: i8,
    pub animFAmt: u8,
    pub flags: MissileGraphicsFlags,
    info_index: usize,
}

impl MissileFileData {
    /// Original: `MissileFileData::FreeGFX` (misdat.h).
    // @port misdat.h|devilution::MissileFileData::FreeGFX() sha=67f9ef1c4ce8
    pub fn free_gfx(&mut self) {
        self.sprites = None;
    }

    fn info(&self) -> &'static crate::tables::misdat::MissileFileInfo {
        &crate::tables::misdat::MissileSpriteInfo[self.info_index]
    }

    /// Original: `MissileFileData::animDelay` (misdat.cpp).
    // @port misdat.cpp|devilution::MissileFileData::animDelay(uint8_t dir) sha=b709c845852a
    pub fn anim_delay(&self, dir: u8) -> u8 {
        crate::tables::misdat::MissileAnimDelays[self.info().animDelayIdx as usize][dir as usize]
    }

    /// Original: `MissileFileData::animLen` (misdat.cpp).
    // @port misdat.cpp|devilution::MissileFileData::animLen(uint8_t dir) sha=7712f3171e06
    pub fn anim_len(&self, dir: u8) -> u8 {
        crate::tables::misdat::MissileAnimLengths[self.info().animLenIdx as usize][dir as usize]
    }

    /// `spritesForDirection`
    // @port misdat.h|devilution::MissileFileData::spritesForDirection(size_t direction) sha=489d2a40667e
    pub fn sprites_for_direction(&self, direction: usize) -> Option<ClxSpriteList> {
        let s = self.sprites.as_ref()?;
        Some(if s.is_sheet() { s.sheet().get(direction) } else { s.list().clone() })
    }
}

/// `MissileSpriteData` with every entry's static fields filled in.
pub fn new_missile_sprite_data() -> Vec<MissileFileData> {
    crate::tables::misdat::MissileSpriteInfo
        .iter()
        .enumerate()
        .map(|(i, info)| MissileFileData {
            sprites: None,
            animWidth: info.animWidth,
            animWidth2: info.animWidth2,
            animFAmt: info.animFAmt,
            flags: info.flags,
            info_index: i,
        })
        .collect()
}

/// Original: `MissileFileData::LoadGFX` (misdat.cpp).
// @port misdat.cpp|devilution::MissileFileData::LoadGFX() sha=c754f1d4c81e
pub fn missile_file_data_load_gfx(ctx: &mut Ctx, mi: usize) {
    if ctx.missiles.missile_sprite_data[mi].sprites.is_some() {
        return;
    }
    let info = &crate::tables::misdat::MissileSpriteInfo[mi];
    if info.name.is_empty() {
        return;
    }
    let sprites = if info.animFAmt == 1 {
        let path = format!("missiles\\{}", info.name);
        ClxSpriteListOrSheet::List(crate::engine::load_sprites::load_cl2(ctx, &path, info.animWidth))
    } else {
        let names: Vec<String> = (1..=info.animFAmt as usize).map(|i| format!("missiles\\{}{}.cl2", info.name, i)).collect();
        ClxSpriteListOrSheet::Sheet(crate::engine::load_sprites::load_multiple_cl2_sheet(ctx, &names, info.animWidth))
    };
    ctx.missiles.missile_sprite_data[mi].sprites = Some(sprites);
}

/// Original: `devilution::InitMissileGFX` (misdat.cpp).
// @port misdat.cpp|devilution::InitMissileGFX(bool loadHellfireGraphics) sha=7a1819e343e6
pub fn init_missile_gfx(ctx: &mut Ctx, load_hellfire_graphics: bool) {
    if ctx.diablo.headless_mode {
        return;
    }
    let mut mi = 0;
    while crate::tables::misdat::MissileSpriteInfo[mi].animFAmt != 0 {
        if !load_hellfire_graphics && mi > MissileGraphicID::BloodStarRedExplosion as usize {
            break;
        }
        if crate::tables::misdat::MissileSpriteInfo[mi].flags == MissileGraphicsFlags::MonsterOwned {
            mi += 1;
            continue;
        }
        missile_file_data_load_gfx(ctx, mi);
        mi += 1;
    }
}

/// `GetMissileSpriteData` (misdat.h)
// @port misdat.h|devilution::GetMissileSpriteData(MissileGraphicID graphicId) sha=3d7565a747ec
pub fn get_missile_sprite_data(ctx: &Ctx, graphic_id: MissileGraphicID) -> &MissileFileData {
    &ctx.missiles.missile_sprite_data[graphic_id as usize]
}

/// Globals of missiles.cpp / misdat.cpp.
pub struct MissilesState {
    /// `Missiles` (a `std::list` in the original; order is kept)
    pub Missiles: Vec<Missile>,
    pub MissilePreFlag: bool,
    /// `MissileSpriteData`
    pub missile_sprite_data: Vec<MissileFileData>,
}

impl Default for MissilesState {
    fn default() -> Self {
        MissilesState { Missiles: Vec::new(), MissilePreFlag: false, missile_sprite_data: new_missile_sprite_data() }
    }
}

/// Original: `devilution::FreeMissileGFX` (misdat.cpp).
// @port misdat.cpp|devilution::FreeMissileGFX() sha=dc3a9beaef6d
pub fn free_missile_gfx(ctx: &mut Ctx) {
    for missile_data in ctx.missiles.missile_sprite_data.iter_mut() {
        missile_data.free_gfx();
    }
}

/// `AddMissile` with the default `lSFX = std::nullopt`.
#[allow(clippy::too_many_arguments)]
pub fn add_missile(ctx: &mut Ctx, src: Point, dst: Point, midir: Direction, mitype: MissileID, micaster: mienemy_type, id: i32, midam: i32, spllvl: i32, parent: Option<usize>) -> Option<usize> {
    add_missile_sfx(ctx, src, dst, midir, mitype, micaster, id, midam, spllvl, parent, None)
}































































































































/// Original: `Missile::sourcePlayer` (missiles.h): the index of the player that cast the missile.
pub fn missile_source_player(missile: &Missile) -> Option<usize> {
    if (missile._micaster != TARGET_BOTH && missile._micaster != TARGET_MONSTERS) || missile._misource == -1 {
        return None;
    }
    Some(missile._misource as usize)
}

/// Original: `devilution::InitMissiles` (missiles.cpp).
// @port missiles.cpp|devilution::InitMissiles() sha=faf14ff9d9e1
pub fn init_missiles(ctx: &mut Ctx) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    ctx.scrollrt.AutoMapShowItems = false;
    ctx.players.Players[me]._pSpellFlags.0 &= !SpellFlag::Etherealize.0;
    if ctx.players.Players[me]._pInfraFlag {
        for i in 0..ctx.missiles.Missiles.len() {
            let missile = &ctx.missiles.Missiles[i];
            if missile._mitype == MissileID::Infravision && missile_source_player(missile) == Some(me) {
                crate::items::calc_plr_item_vals(ctx, me, true);
            }
        }
    }
    if ctx.players.Players[me]._pSpellFlags.has_any_of(SpellFlag::RageActive | SpellFlag::RageCooldown) {
        ctx.players.Players[me]._pSpellFlags.0 &= !SpellFlag::RageActive.0;
        ctx.players.Players[me]._pSpellFlags.0 &= !SpellFlag::RageCooldown.0;
        for i in 0..ctx.missiles.Missiles.len() {
            let missile = &ctx.missiles.Missiles[i];
            if missile._mitype == MissileID::Rage && missile_source_player(missile) == Some(me) {
                let var2 = missile.var2;
                let missing_hp = ctx.players.Players[me]._pMaxHP - ctx.players.Players[me]._pHitPoints;
                crate::items::calc_plr_item_vals(ctx, me, true);
                crate::player::apply_plr_damage(ctx, DamageType::Physical, me, 0, 1, missing_hp + var2, DeathReason::MonsterOrTrap);
            }
        }
    }
    ctx.missiles.Missiles.clear();
    let mask = !(DungeonFlag::Missile.0 | DungeonFlag::MissileFireWall.0 | DungeonFlag::MissileLightningWall.0);
    for j in 0..crate::levels::gendung::MAXDUNY {
        for i in 0..crate::levels::gendung::MAXDUNX {
            ctx.gendung.dFlags[i][j].0 &= mask;
        }
    }
}

/// Original: `DeleteMissiles` (missiles.cpp).
// @port missiles.cpp|devilution::DeleteMissiles() sha=26976cea4b07
fn delete_missiles(ctx: &mut Ctx) {
    ctx.missiles.Missiles.retain(|missile| !missile._miDelFlag);
}

/// Original: `ProcessManaShield` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessManaShield() sha=7fcd47992662
fn process_mana_shield(ctx: &mut Ctx) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let p = &mut ctx.players.Players[me];
    if p.pManaShield && p._pMana <= 0 {
        p.pManaShield = false;
        crate::msg::net_send_cmd(ctx, true, CMD_REMSHIELD);
    }
}

/// Original: `devilution::ProcessMissiles` (missiles.cpp). The original's `std::list` is a
/// `Vec` here; missiles added while processing are processed in the same pass, as there.
// @port missiles.cpp|devilution::ProcessMissiles() sha=4311b64c8ea9
pub fn process_missiles(ctx: &mut Ctx) {
    let mask = !(DungeonFlag::Missile.0 | DungeonFlag::MissileFireWall.0 | DungeonFlag::MissileLightningWall.0);
    for missile in ctx.missiles.Missiles.iter_mut() {
        let position = missile.position.tile;
        if crate::levels::gendung::in_dungeon_bounds(position) {
            ctx.gendung.dFlags[position.x as usize][position.y as usize].0 &= mask;
        } else {
            missile._miDelFlag = true;
        }
    }
    delete_missiles(ctx);
    ctx.missiles.MissilePreFlag = false;
    let mut i = 0;
    while i < ctx.missiles.Missiles.len() {
        let missile_data = get_missile_data(ctx.missiles.Missiles[i]._mitype);
        if let Some(proc_) = missile_data.mProc {
            proc_(ctx, i);
        }
        let missile = &mut ctx.missiles.Missiles[i];
        i += 1;
        if missile._miAnimFlags == MissileGraphicsFlags::NotAnimated {
            continue;
        }
        missile._miAnimCnt += 1;
        if missile._miAnimCnt < missile._miAnimDelay {
            continue;
        }
        missile._miAnimCnt = 0;
        missile._miAnimFrame += missile._miAnimAdd;
        if missile._miAnimFrame > missile._miAnimLen {
            missile._miAnimFrame = 1;
        } else if missile._miAnimFrame < 1 {
            missile._miAnimFrame = missile._miAnimLen;
        }
    }
    process_mana_shield(ctx);
    delete_missiles(ctx);
}

// ---------------------------------------------------------------------------------------------
// missiles.cpp, part 1: helpers, hit resolution and movement.
// ---------------------------------------------------------------------------------------------

use crate::levels::gendung::{in_dungeon_bounds, tile_has_any, DungeonType, MAXDUNX, MAXDUNY};
use crate::monster::{monster_type_id, MaxMonsters};

/// `MissileSource`
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MissileSource {
    Player,
    Monster,
    Trap,
}

impl Missile {
    /// `IsTrap`
    // @port missiles.h|devilution::Missile::IsTrap() sha=61aa47a68e1e
    pub fn is_trap(&self) -> bool {
        self._misource == -1
    }

    /// `sourceMonster` (index into `Monsters`)
    // @port missiles.h|devilution::Missile::sourceMonster() sha=e5598b74116a
    pub fn source_monster(&self) -> Option<usize> {
        if self._micaster != TARGET_PLAYERS || self._misource == -1 {
            return None;
        }
        Some(self._misource as usize)
    }

    /// `sourcePlayer` (index into `Players`)
    // @port missiles.h|devilution::Missile::sourcePlayer() sha=739ea88a7a8f
    pub fn source_player(&self) -> Option<usize> {
        missile_source_player(self)
    }

    /// `sourceType`
    // @port missiles.h|devilution::Missile::sourceType() sha=bfd684b81013
    pub fn source_type(&self) -> MissileSource {
        if self._misource == -1 {
            return MissileSource::Trap;
        }
        if self._micaster == TARGET_PLAYERS {
            return MissileSource::Monster;
        }
        MissileSource::Player
    }

    /// `isSameSource`
    // @port missiles.h|devilution::Missile::isSameSource(Missile &missile) sha=645817269f33
    pub fn is_same_source(&self, other: &Missile) -> bool {
        self.source_type() == other.source_type() && self._misource == other._misource
    }
}

impl MissilePosition {
    /// `StopMissile`
    // @port missiles.h|devilution::MissilePosition::StopMissile() sha=5aeff0e82b5d
    pub fn stop_missile(&mut self) {
        self.velocity = Displacement::default();
        if self.tileForRendering == self.tile {
            self.offset = self.offsetForRendering;
        }
    }
}

fn mis(ctx: &mut Ctx, mi: usize) -> &mut Missile {
    &mut ctx.missiles.Missiles[mi]
}

fn rnd(ctx: &mut Ctx, v: i32) -> i32 {
    ctx.rng.generate_rnd(v)
}

fn piece_at(ctx: &Ctx, p: Point) -> i32 {
    ctx.gendung.dPiece[p.x as usize][p.y as usize] as i32
}

fn difficulty(ctx: &Ctx) -> _difficulty {
    ctx.multi.sgGameInitInfo.nDifficulty
}

/// Original: `AddClassHealingBonus` (missiles.cpp).
// @port missiles.cpp|devilution::AddClassHealingBonus(int hp, HeroClass heroClass) sha=7778514e3c46
fn add_class_healing_bonus(hp: i32, hero_class: HeroClass) -> i32 {
    match hero_class {
        HeroClass::Warrior | HeroClass::Monk | HeroClass::Barbarian => hp * 2,
        HeroClass::Rogue | HeroClass::Bard => hp + hp / 2,
        _ => hp,
    }
}

/// Original: `ScaleSpellEffect` (missiles.cpp).
// @port missiles.cpp|devilution::ScaleSpellEffect(int base, int spellLevel) sha=4e6bf4c5f681
fn scale_spell_effect(mut base: i32, spell_level: i32) -> i32 {
    for _ in 0..spell_level {
        base += base / 8;
    }
    base
}

/// Original: `GenerateRndSum` (missiles.cpp).
// @port missiles.cpp|devilution::GenerateRndSum(int range, int iterations) sha=67de19eee941
fn generate_rnd_sum(ctx: &mut Ctx, range: i32, iterations: i32) -> i32 {
    let mut value = 0;
    for _ in 0..iterations {
        value += rnd(ctx, range);
    }
    value
}

/// Original: `CheckBlock` (missiles.cpp).
// @port missiles.cpp|devilution::CheckBlock(Point from, Point to) sha=145e15babff2
fn check_block(ctx: &Ctx, mut from: Point, to: Point) -> bool {
    while from != to {
        from += crate::engine::get_direction(from, to);
        if tile_has_any(ctx, piece_at(ctx, from), TileProperties::Solid) {
            return true;
        }
    }
    false
}

/// Original: `FindClosest` (missiles.cpp).
// @port missiles.cpp|devilution::FindClosest(Point source, int rad) sha=4c8d69d44409
fn find_closest(ctx: &Ctx, source: Point, rad: i32) -> Option<usize> {
    let monster_position = crate::engine::path::find_closest_valid_position(
        ctx,
        &|ctx: &Ctx, target: Point| {
            // search for a monster with clear line of sight
            in_dungeon_bounds(target) && ctx.gendung.dMonster[target.x as usize][target.y as usize] > 0 && !check_block(ctx, source, target)
        },
        source,
        1,
        rad as u32,
    )?;
    let mid = ctx.gendung.dMonster[monster_position.x as usize][monster_position.y as usize];
    Some((mid - 1) as usize)
}

/// Original: `Direction16Flip` (missiles.cpp).
// @port missiles.cpp|devilution::Direction16Flip(Direction16 x, Direction16 pivot) sha=9aef025b3f4b
const fn direction16_flip(x: Direction16, pivot: Direction16) -> Direction16 {
    let ret = (2 * pivot as u8 + 16 - x as u8) % 16;
    match Direction16::from_repr(ret) {
        Some(d) => d,
        None => Direction16::South,
    }
}

/// Original: `UpdateMissileVelocity` (missiles.cpp).
// @port missiles.cpp|devilution::UpdateMissileVelocity(Missile &missile, Point destination, int velocityInPixels) sha=9c7605842f1a
fn update_missile_velocity(ctx: &mut Ctx, mi: usize, destination: Point, velocity_in_pixels: i32) {
    let missile = mis(ctx, mi);
    missile.position.velocity = Displacement::new(0, 0);
    if missile.position.tile == destination {
        return;
    }
    // Get the normalized vector in isometric projection
    let fixed16_normal_vector = (missile.position.tile - destination).world_to_normal_screen();
    // Multiplying by the target velocity gives us a scaled velocity vector.
    missile.position.velocity = fixed16_normal_vector * velocity_in_pixels;
}

/// Original: `PutMissile` (missiles.cpp): adds the missile to the lookup tables.
// @port missiles.cpp|devilution::PutMissile(Missile &missile) sha=20cd3776af25
fn put_missile(ctx: &mut Ctx, mi: usize) {
    let position = ctx.missiles.Missiles[mi].position.tile;
    if !in_dungeon_bounds(position) {
        ctx.missiles.Missiles[mi]._miDelFlag = true;
    }
    let missile = &ctx.missiles.Missiles[mi];
    if missile._miDelFlag {
        return;
    }
    let flags = &mut ctx.gendung.dFlags[position.x as usize][position.y as usize];
    *flags |= DungeonFlag::Missile;
    if missile._mitype == MissileID::FireWall {
        *flags |= DungeonFlag::MissileFireWall;
    }
    if missile._mitype == MissileID::LightningWall {
        *flags |= DungeonFlag::MissileLightningWall;
    }
    if missile._miPreFlag {
        ctx.missiles.MissilePreFlag = true;
    }
}

/// Original: `UpdateMissilePos` (missiles.cpp).
// @port missiles.cpp|devilution::UpdateMissilePos(Missile &missile) sha=24da94efabe9
fn update_missile_pos(ctx: &mut Ctx, mi: usize) {
    let missile = mis(ctx, mi);
    let pixels_travelled = missile.position.traveled >> 16;
    let tile_offset = pixels_travelled.screen_to_missile();
    missile.position.tile = missile.position.start + tile_offset;
    missile.position.offset = pixels_travelled + tile_offset.world_to_screen();
    let absolute_light_offset = pixels_travelled.screen_to_light();
    let l = missile._mlid;
    crate::lighting::change_light_offset(ctx, l, absolute_light_offset - tile_offset * 8);
}

/// Original: `MoveMissilePos` (missiles.cpp): shifts charging monsters that are not facing north.
// @port missiles.cpp|devilution::MoveMissilePos(Missile &missile) sha=012c7eaf3204
fn move_missile_pos(ctx: &mut Ctx, mi: usize) {
    let move_direction = match Direction::from_u8(ctx.missiles.Missiles[mi]._mimfnum as u8) {
        Direction::East => Direction::SouthEast,
        Direction::West => Direction::SouthWest,
        Direction::South | Direction::SouthWest | Direction::SouthEast => Direction::South,
        _ => return,
    };
    let target = ctx.missiles.Missiles[mi].position.tile + move_direction;
    let m = ctx.missiles.Missiles[mi].source_monster().expect("source monster");
    if crate::monster::is_tile_available_for_monster(ctx, m, target) {
        let missile = mis(ctx, mi);
        missile.position.tile = target;
        missile.position.offset += Displacement::from(move_direction).world_to_screen();
    }
}

/// Original: `ProjectileMonsterDamage` (missiles.cpp).
// @port missiles.cpp|devilution::ProjectileMonsterDamage(Missile &missile) sha=f91634f6e54a
fn projectile_monster_damage(ctx: &mut Ctx, mi: usize) -> i32 {
    let m = ctx.missiles.Missiles[mi].source_monster().expect("source monster");
    let (min, max) = (ctx.monster.Monsters[m].minDamage as i32, ctx.monster.Monsters[m].maxDamage as i32);
    min + rnd(ctx, max - min + 1)
}

/// Original: `ProjectileTrapDamage` (missiles.cpp).
// @port missiles.cpp|devilution::ProjectileTrapDamage(Missile &missile) sha=81e0f5071536
fn projectile_trap_damage(ctx: &mut Ctx, _mi: usize) -> i32 {
    let currlevel = ctx.gendung.currlevel as i32;
    currlevel + rnd(ctx, 2 * currlevel)
}

/// Original: `MonsterMHit` (missiles.cpp).
// @port missiles.cpp|devilution::MonsterMHit(int pnum, int monsterId, int mindam, int maxdam, int dist, MissileID t, DamageType damageType, bool shift) sha=f422be56a363
#[allow(clippy::too_many_arguments)]
fn monster_m_hit(ctx: &mut Ctx, pnum: usize, monster_id: usize, mindam: i32, maxdam: i32, dist: i32, t: MissileID, damage_type: DamageType, shift: bool) -> bool {
    let m = monster_id;
    if !crate::monster::is_possible_to_hit(ctx, m) || crate::monster::is_immune(ctx, m, t, damage_type) {
        return false;
    }
    let mut hit = rnd(ctx, 100);
    let hf = ctx.init.gb_is_hellfire;
    let missile_data = get_missile_data(t);
    let player = &ctx.players.Players[pnum];
    let mut hper;
    if missile_data.is_arrow() {
        hper = player.get_ranged_piercing_to_hit(hf);
        hper -= player.calculate_armor_pierce(ctx.monster.Monsters[m].armorClass as i32, false, hf);
        hper -= (dist * dist) / 2;
    } else {
        hper = player.get_magic_to_hit() - (crate::monster::monster_level(ctx, m, difficulty(ctx)) as i32 * 2) - dist;
    }
    hper = hper.clamp(5, 95);
    if ctx.monster.Monsters[m].mode == MonsterMode::Petrified {
        hit = 0;
    }
    if crate::monster::try_lift_gargoyle(ctx, m) {
        return true;
    }
    if hit >= hper {
        return false;
    }
    let mut dam = if t == MissileID::BoneSpirit {
        ctx.monster.Monsters[m].hitPoints / 3 >> 6
    } else {
        mindam + rnd(ctx, maxdam - mindam + 1)
    };
    let player = &ctx.players.Players[pnum];
    if missile_data.is_arrow() && damage_type == DamageType::Physical {
        dam = player._pIBonusDamMod + dam * player._pIBonusDam / 100 + dam;
        if player._pClass == HeroClass::Rogue {
            dam += player._pDamageMod;
        } else {
            dam += player._pDamageMod / 2;
        }
        if crate::monster::monster_data(ctx, m).monsterClass == MonsterClass::Demon && player._pIFlags.has_any_of(ItemSpecialEffect::TripleDemonDamage) {
            dam *= 3;
        }
    }
    let knockback = player._pIFlags.has_any_of(ItemSpecialEffect::Knockback);
    let resist = crate::monster::is_resistant(ctx, m, t, damage_type);
    if !shift {
        dam <<= 6;
    }
    if resist {
        dam >>= 2;
    }
    if Some(pnum) == ctx.players.MyPlayer {
        crate::monster::apply_monster_damage(ctx, damage_type, m, dam);
    }
    if ctx.monster.Monsters[m].hitPoints >> 6 <= 0 {
        crate::monster::m_start_kill(ctx, m, pnum);
    } else if resist {
        crate::monster::tag(ctx, m, pnum);
        crate::monster::play_effect(ctx, m, MonsterSound::Hit);
    } else {
        if ctx.monster.Monsters[m].mode != MonsterMode::Petrified && missile_data.is_arrow() && knockback {
            crate::monster::m_get_knockback(ctx, m);
        }
        if monster_type_id(ctx, m) != MT_GOLEM {
            crate::monster::m_start_hit(ctx, m, pnum, dam);
        }
    }
    if ctx.monster.Monsters[m].activeForTicks == 0 {
        let t = ctx.players.Players[pnum].position.tile;
        let monster = &mut ctx.monster.Monsters[m];
        monster.activeForTicks = u8::MAX;
        monster.position.last = t;
    }
    true
}

fn resist_percent(player: &crate::player::Player, damage_type: DamageType) -> i32 {
    match damage_type {
        DamageType::Fire => player._pFireResist as i32,
        DamageType::Lightning => player._pLghtResist as i32,
        DamageType::Magic | DamageType::Acid => player._pMagResist as i32,
        _ => 0,
    }
}

/// Original: `Plr2PlrMHit` (missiles.cpp).
// @port missiles.cpp|devilution::Plr2PlrMHit(const Player &player, int p, int mindam, int maxdam, int dist, MissileID mtype, DamageType damageType, bool shift, bool *blocked) sha=2a18be85536a
#[allow(clippy::too_many_arguments)]
fn plr2plr_m_hit(ctx: &mut Ctx, pnum: usize, p: usize, mindam: i32, maxdam: i32, dist: i32, mtype: MissileID, damage_type: DamageType, shift: bool, blocked: &mut bool) -> bool {
    if ctx.multi.sgGameInitInfo.bFriendlyFire == 0 && ctx.players.Players[pnum].friendlyMode {
        return false;
    }
    *blocked = false;
    let target = &ctx.players.Players[p];
    if target.is_on_arena_level() && target._pmode == PM_WALK_SIDEWAYS {
        return false;
    }
    if target._pInvincible {
        return false;
    }
    if mtype == MissileID::HolyBolt {
        return false;
    }
    let missile_data = get_missile_data(mtype);
    if target._pSpellFlags.has_any_of(SpellFlag::Etherealize) && missile_data.is_arrow() {
        return false;
    }
    let resper = resist_percent(target, damage_type) as i8 as i32;
    let hper = rnd(ctx, 100);
    let player = &ctx.players.Players[pnum];
    let target = &ctx.players.Players[p];
    let mut hit = if missile_data.is_arrow() {
        player.get_ranged_to_hit() - (dist * dist / 2) - target.get_armor()
    } else {
        player.get_magic_to_hit() - (target._pLevel as i32 * 2) - dist
    };
    hit = hit.clamp(5, 95);
    if hper >= hit {
        return false;
    }
    let mut blkper = 100;
    if !shift && (target._pmode == PM_STAND || target._pmode == PM_ATTACK) && target._pBlockFlag {
        blkper = rnd(ctx, 100);
    }
    let player = &ctx.players.Players[pnum];
    let target = &ctx.players.Players[p];
    let mut blk = target.get_block_chance(true) - (player._pLevel as i32 * 2);
    blk = blk.clamp(0, 100);
    let mut dam;
    if mtype == MissileID::BoneSpirit {
        dam = target._pHitPoints / 3;
    } else {
        dam = mindam + rnd(ctx, maxdam - mindam + 1);
        let player = &ctx.players.Players[pnum];
        if missile_data.is_arrow() && damage_type == DamageType::Physical {
            dam += player._pIBonusDamMod + player._pDamageMod + dam * player._pIBonusDam / 100;
        }
        if !shift {
            dam <<= 6;
        }
    }
    if !missile_data.is_arrow() {
        dam /= 2;
    }
    let is_me = Some(pnum) == ctx.players.MyPlayer;
    if resper > 0 {
        dam -= (dam * resper) / 100;
        if is_me {
            crate::msg::net_send_cmd_damage(ctx, true, p as u8, dam as u32, damage_type);
        }
        crate::player::player_say(ctx, p, HeroSpeech::ArghClang);
        return true;
    }
    if blkper < blk {
        let d = crate::engine::get_direction(ctx.players.Players[p].position.tile, ctx.players.Players[pnum].position.tile);
        crate::player::start_plr_block(ctx, p, d);
        *blocked = true;
    } else {
        if is_me {
            crate::msg::net_send_cmd_damage(ctx, true, p as u8, dam as u32, damage_type);
        }
        crate::player::start_plr_hit(ctx, p, dam, false);
    }
    true
}

/// `TestRotateBlockedMissile` (missiles.cpp, built only with BUILD_TESTING): the unit tests' entry
/// point to `RotateBlockedMissile`.
pub fn test_rotate_blocked_missile(ctx: &mut Ctx, mi: usize) {
    rotate_blocked_missile(ctx, mi);
}

/// Original: `RotateBlockedMissile` (missiles.cpp).
// @port missiles.cpp|devilution::RotateBlockedMissile(Missile &missile) sha=06c9d4d9072e
fn rotate_blocked_missile(ctx: &mut Ctx, mi: usize) {
    let rotation = ctx.rng.pick_randomly_among(&[-1, 1]);
    let missile = mis(ctx, mi);
    if missile._miAnimType == MissileGraphicID::Arrow {
        let dir = missile._miAnimFrame + rotation;
        missile._miAnimFrame = (dir + 15) % 16 + 1;
        return;
    }
    let mut dir = missile._mimfnum + rotation;
    let m_anim_f_amt = get_missile_sprite_data(ctx, ctx.missiles.Missiles[mi]._miAnimType).animFAmt as i32;
    if dir < 0 {
        dir = m_anim_f_amt - 1;
    } else if dir >= m_anim_f_amt {
        dir = 0;
    }
    set_miss_dir(ctx, mi, dir);
}

/// Original: `CheckMissileCol` (missiles.cpp).
// @port missiles.cpp|devilution::CheckMissileCol(Missile &missile, DamageType damageType, int minDamage, int maxDamage, bool isDamageShifted, Point position, bool dontDeleteOnCollision) sha=5e85dda732fb
#[allow(clippy::too_many_arguments)]
fn check_missile_col(ctx: &mut Ctx, mi: usize, damage_type: DamageType, min_damage: i32, max_damage: i32, is_damage_shifted: bool, position: Point, dont_delete_on_collision: bool) {
    if !in_dungeon_bounds(position) {
        return;
    }
    let (mx, my) = (position.x as usize, position.y as usize);
    let mut is_monster_hit = false;
    let mut mid = ctx.gendung.dMonster[mx][my] as i32;
    if mid > 0 || (mid != 0 && ctx.monster.Monsters[(mid.abs() - 1) as usize].mode == MonsterMode::Petrified) {
        mid = mid.abs() - 1;
        let m = mid as usize;
        let missile = ctx.missiles.Missiles[mi].clone();
        let opposing = |ctx: &Ctx| {
            let src = missile._misource as usize;
            ctx.monster.Monsters[m].is_player_minion() != ctx.monster.Monsters[src].is_player_minion() // the monsters are on opposing factions
                || (ctx.monster.Monsters[src].flags & MFLAG_BERSERK as u32) != 0 // or the attacker is berserked
                || (ctx.monster.Monsters[m].flags & MFLAG_BERSERK as u32) != 0 // or the target is berserked
        };
        if missile.is_trap() || (missile._micaster == TARGET_PLAYERS && opposing(ctx)) {
            // then the missile can potentially hit this target
            is_monster_hit = monster_trap_hit(ctx, m, min_damage, max_damage, missile._midist, missile._mitype, damage_type, is_damage_shifted);
        } else if missile._micaster == TARGET_BOTH || missile._micaster == TARGET_MONSTERS {
            is_monster_hit = monster_m_hit(ctx, missile._misource as usize, m, min_damage, max_damage, missile._midist, missile._mitype, damage_type, is_damage_shifted);
        }
    }
    if is_monster_hit {
        let missile = mis(ctx, mi);
        if !dont_delete_on_collision {
            missile._mirange = 0;
        }
        missile._miHitFlag = true;
    }
    let mut is_player_hit = false;
    let mut blocked = false;
    let pid = ctx.gendung.dPlayer[mx][my];
    if pid > 0 {
        let missile = ctx.missiles.Missiles[mi].clone();
        let p = (pid - 1) as usize;
        if missile._micaster != TARGET_BOTH && !missile.is_trap() {
            if missile._micaster == TARGET_MONSTERS {
                if (pid - 1) as i32 != missile._misource {
                    is_player_hit = plr2plr_m_hit(ctx, missile._misource as usize, p, min_damage, max_damage, missile._midist, missile._mitype, damage_type, is_damage_shifted, &mut blocked);
                }
            } else {
                is_player_hit = player_m_hit(ctx, p, Some(missile._misource as usize), missile._midist, min_damage, max_damage, missile._mitype, damage_type, is_damage_shifted, DeathReason::MonsterOrTrap, &mut blocked);
            }
        } else {
            let death_reason = if !missile.is_trap() && (missile._miAnimType == MissileGraphicID::FireWall || missile._miAnimType == MissileGraphicID::Lightning) {
                DeathReason::Player
            } else {
                DeathReason::MonsterOrTrap
            };
            is_player_hit = player_m_hit(ctx, p, None, missile._midist, min_damage, max_damage, missile._mitype, damage_type, is_damage_shifted, death_reason, &mut blocked);
        }
    }
    if is_player_hit {
        if ctx.init.gb_is_hellfire && blocked {
            rotate_blocked_missile(ctx, mi);
        } else if !dont_delete_on_collision {
            mis(ctx, mi)._mirange = 0;
        }
        mis(ctx, mi)._miHitFlag = true;
    }
    if is_missile_blocked_by_tile(ctx, position) {
        if let Some(object) = crate::objects::find_object_at_position(ctx, position, true) {
            if ctx.objects.Objects[object].is_breakable() {
                let sp = ctx.missiles.Missiles[mi].source_player();
                crate::objects::break_object_missile(ctx, sp, object);
            }
        }
        let missile = mis(ctx, mi);
        if !dont_delete_on_collision {
            missile._mirange = 0;
        }
        missile._miHitFlag = false;
    }
    let missile_data = get_missile_data(ctx.missiles.Missiles[mi]._mitype);
    if ctx.missiles.Missiles[mi]._mirange == 0 && missile_data.miSFX != -1 {
        let t = ctx.missiles.Missiles[mi].position.tile;
        crate::effects::play_sfx_loc(ctx, missile_data.miSFX, t, true);
    }
}

/// Original: `MoveMissile` (missiles.cpp). `ifCheckTileFailsDontMoveToTile` defaults to false.
// @port missiles.cpp|devilution::MoveMissile(Missile &missile, tl::function_ref<bool(Point)> checkTile, bool ifCheckTileFailsDontMoveToTile = false) sha=de14db93c344
fn move_missile(ctx: &mut Ctx, mi: usize, check_tile: &mut dyn FnMut(&mut Ctx, usize, Point) -> bool, if_check_tile_fails_dont_move_to_tile: bool) -> bool {
    let mut prev_tile = ctx.missiles.Missiles[mi].position.tile;
    {
        let m = mis(ctx, mi);
        m.position.traveled += m.position.velocity;
    }
    update_missile_pos(ctx, mi);
    let pos = ctx.missiles.Missiles[mi].position;
    let possible_visit_tiles = if pos.velocity.delta_x == 0 || pos.velocity.delta_y == 0 { prev_tile.walking_distance(pos.tile) } else { prev_tile.manhattan_distance(pos.tile) };
    if possible_visit_tiles == 0 {
        return false;
    }
    // Did the missile skip a tile?
    if possible_visit_tiles > 1 {
        let speed = pos.velocity.abs();
        let denominator: f32 = if 2 * speed.delta_y >= speed.delta_x { (2 * speed.delta_y) as f32 } else { speed.delta_x as f32 };
        let inc_velocity = pos.velocity * ((32 << 16) as f32 / denominator);
        let mut traveled = pos.traveled - pos.velocity;
        // Adjust the traveled vector to start on the next smallest multiple of incVelocity
        if inc_velocity.delta_y != 0 {
            traveled.delta_y = (traveled.delta_y / inc_velocity.delta_y) * inc_velocity.delta_y;
        }
        if inc_velocity.delta_x != 0 {
            traveled.delta_x = (traveled.delta_x / inc_velocity.delta_x) * inc_velocity.delta_x;
        }
        loop {
            let cur_traveled = ctx.missiles.Missiles[mi].position.traveled;
            let initial_diff = cur_traveled - traveled;
            traveled += inc_velocity;
            let inc_diff = cur_traveled - traveled;
            // we are at the original calculated position => resume with normal logic
            if (initial_diff.delta_x < 0) != (inc_diff.delta_x < 0) {
                break;
            }
            if (initial_diff.delta_y < 0) != (inc_diff.delta_y < 0) {
                break;
            }
            // calculate in-between tile
            let pixels_traveled = traveled >> 16;
            let tile_offset = pixels_traveled.screen_to_missile();
            let tile = ctx.missiles.Missiles[mi].position.start + tile_offset;
            // we haven't quite reached the missile's current position,
            // but we can break early to avoid checking collisions in this tile twice
            if tile == ctx.missiles.Missiles[mi].position.tile {
                break;
            }
            // skip collision logic if the missile is on a corner between tiles
            if pixels_traveled.delta_y % 16 == 0 && pixels_traveled.delta_x % 32 == 0 && (pixels_traveled.delta_y / 16).abs() % 2 != (pixels_traveled.delta_x / 32).abs() % 2 {
                continue;
            }
            // don't call checkTile more than once for a tile
            if prev_tile == tile {
                continue;
            }
            prev_tile = tile;
            if !check_tile(ctx, mi, tile) {
                mis(ctx, mi).position.traveled = traveled;
                if if_check_tile_fails_dont_move_to_tile {
                    mis(ctx, mi).position.traveled -= inc_velocity;
                    update_missile_pos(ctx, mi);
                    mis(ctx, mi).position.stop_missile();
                } else {
                    update_missile_pos(ctx, mi);
                }
                return true;
            }
        }
    }
    let t = ctx.missiles.Missiles[mi].position.tile;
    if !check_tile(ctx, mi, t) && if_check_tile_fails_dont_move_to_tile {
        {
            let m = mis(ctx, mi);
            m.position.traveled -= m.position.velocity;
        }
        update_missile_pos(ctx, mi);
        mis(ctx, mi).position.stop_missile();
    }
    true
}

/// Original: `MoveMissileAndCheckMissileCol` (missiles.cpp).
// @port missiles.cpp|devilution::MoveMissileAndCheckMissileCol(Missile &missile, DamageType damageType, int mindam, int maxdam, bool ignoreStart, bool ifCollidesDontMoveToHitTile) sha=c77606509a7a
fn move_missile_and_check_missile_col(ctx: &mut Ctx, mi: usize, damage_type: DamageType, mindam: i32, maxdam: i32, ignore_start: bool, if_collides_dont_move_to_hit_tile: bool) {
    let mut check_tile = |ctx: &mut Ctx, mi: usize, tile: Point| -> bool {
        if ignore_start && ctx.missiles.Missiles[mi].position.start == tile {
            return true;
        }
        check_missile_col(ctx, mi, damage_type, mindam, maxdam, false, tile, false);
        // Did missile hit anything?
        let missile = &ctx.missiles.Missiles[mi];
        if missile._mirange != 0 {
            return true;
        }
        if missile._miHitFlag && get_missile_data(missile._mitype).movementDistribution == MissileMovementDistribution::Blockable {
            return false;
        }
        !is_missile_blocked_by_tile(ctx, tile)
    };
    let tile_changed = move_missile(ctx, mi, &mut check_tile, if_collides_dont_move_to_hit_tile);
    let t = ctx.missiles.Missiles[mi].position.tile;
    let tile_target_hash = ctx.gendung.dMonster[t.x as usize][t.y as usize] ^ (ctx.gendung.dPlayer[t.x as usize][t.y as usize] as i16);
    // missile didn't change the tile... check that we perform CheckMissileCol only once for any monster/player to avoid multiple hits for slow missiles
    if !tile_changed && ctx.missiles.Missiles[mi].lastCollisionTargetHash != tile_target_hash {
        check_missile_col(ctx, mi, damage_type, mindam, maxdam, false, t, false);
    }
    // remember what target CheckMissileCol was checked against
    mis(ctx, mi).lastCollisionTargetHash = tile_target_hash;
}

/// Original: `SetMissAnim` (missiles.cpp).
// @port missiles.cpp|devilution::SetMissAnim(Missile &missile, MissileGraphicID animtype) sha=8ac469917c6a
fn set_miss_anim(ctx: &mut Ctx, mi: usize, mut animtype: MissileGraphicID) {
    let dir = ctx.missiles.Missiles[mi]._mimfnum;
    if animtype as u8 > MissileGraphicID::None as u8 {
        animtype = MissileGraphicID::None;
    }
    let headless = ctx.diablo.headless_mode;
    let missile_data = get_missile_sprite_data(ctx, animtype);
    let flags = missile_data.flags;
    let data = if !headless { missile_data.sprites_for_direction(dir as usize) } else { None };
    let delay = missile_data.anim_delay(dir as u8) as i32;
    let len = missile_data.anim_len(dir as u8) as i32;
    let (w, w2) = (missile_data.animWidth, missile_data.animWidth2 as i16);
    let missile = mis(ctx, mi);
    missile._miAnimType = animtype;
    missile._miAnimFlags = flags;
    if !headless {
        missile._miAnimData = data;
    }
    missile._miAnimDelay = delay;
    missile._miAnimLen = len;
    missile._miAnimWidth = w;
    missile._miAnimWidth2 = w2;
    missile._miAnimCnt = 0;
    missile._miAnimFrame = 1;
}

/// Original: `AddRune` (missiles.cpp).
// @port missiles.cpp|devilution::AddRune(Missile &missile, Point dst, MissileID missileID) sha=9fe3483fd285
fn add_rune(ctx: &mut Ctx, mi: usize, dst: Point, missile_id: MissileID) {
    let start = ctx.missiles.Missiles[mi].position.start;
    if crate::monster::line_clear_missile(ctx, start, dst) {
        let rune_position = crate::engine::path::find_closest_valid_position(
            ctx,
            &|ctx: &Ctx, target: Point| {
                if !in_dungeon_bounds(target) {
                    return false;
                }
                if crate::objects::is_object_at_position(ctx, target) {
                    return false;
                }
                if crate::levels::gendung::tile_contains_missile(ctx, target) {
                    return false;
                }
                if tile_has_any(ctx, piece_at(ctx, target), TileProperties::Solid) {
                    return false;
                }
                true
            },
            dst,
            0,
            9,
        );
        if let Some(p) = rune_position {
            let missile = mis(ctx, mi);
            missile.position.tile = p;
            missile.var1 = missile_id as i8 as i32;
            mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, p, 8);
            return;
        }
    }
    mis(ctx, mi)._miDelFlag = true;
}

/// Original: `CheckIfTrig` (missiles.cpp).
// @port missiles.cpp|devilution::CheckIfTrig(Point position) sha=fbf43626ef92
fn check_if_trig(ctx: &Ctx, position: Point) -> bool {
    for i in 0..ctx.trigs.numtrigs as usize {
        if ctx.trigs.trigs[i].position.walking_distance(position) < 2 {
            return true;
        }
    }
    false
}

/// Original: `GuardianTryFireAt` (missiles.cpp).
// @port missiles.cpp|devilution::GuardianTryFireAt(Missile &missile, Point target) sha=3eada5a0c28d
fn guardian_try_fire_at(ctx: &mut Ctx, mi: usize, target: Point) -> bool {
    let position = ctx.missiles.Missiles[mi].position.tile;
    if !crate::monster::line_clear_missile(ctx, position, target) {
        return false;
    }
    let mid = ctx.gendung.dMonster[target.x as usize][target.y as usize] as i32 - 1;
    if mid < 0 {
        return false;
    }
    let monster = &ctx.monster.Monsters[mid as usize];
    if monster.is_player_minion() {
        return false;
    }
    if monster.hitPoints >> 6 <= 0 {
        return false;
    }
    let missile = ctx.missiles.Missiles[mi].clone();
    let plevel = ctx.players.Players[missile._misource as usize]._pLevel as i32;
    let mut dmg = rnd(ctx, 10) + (plevel / 2) + 1;
    dmg = scale_spell_effect(dmg, missile._mispllvl);
    let _ = dmg;
    let dir = crate::engine::get_direction(position, target);
    let sp = missile.source_player().expect("source player");
    let slvl = ctx.players.Players[sp].get_spell_level(SpellID::Guardian);
    add_missile(ctx, position, target, dir, MissileID::Firebolt, TARGET_MONSTERS, missile._misource, missile._midam, slvl, Some(mi));
    set_miss_dir(ctx, mi, 2);
    mis(ctx, mi).var2 = 3;
    true
}

/// Original: `GrowWall` (missiles.cpp).
// @port missiles.cpp|devilution::GrowWall(int playerId, Point position, Point target, MissileID type, int spellLevel, int damage) sha=705bc46bcfb1
fn grow_wall(ctx: &mut Ctx, player_id: i32, position: Point, target: Point, type_: MissileID, spell_level: i32, damage: i32) -> bool {
    let dp = piece_at(ctx, position);
    assert!((0..=crate::levels::gendung::MAXTILES as i32).contains(&dp));
    if tile_has_any(ctx, dp, TileProperties::BlockMissile) || !in_dungeon_bounds(target) {
        return false;
    }
    let d = ctx.players.Players[player_id as usize]._pdir;
    add_missile(ctx, position, position, d, type_, TARGET_BOTH, player_id, damage, spell_level, None);
    true
}

/// Original: `SyncPositionWithParent` (missiles.cpp): syncs the missile position with its parent missile.
// @port missiles.cpp|devilution::SyncPositionWithParent(Missile &missile, const AddMissileParameter &parameter) sha=514e7f258c87
fn sync_position_with_parent(ctx: &mut Ctx, mi: usize, parameter: &AddMissileParameter) {
    let Some(parent) = parameter.pParent else {
        return;
    };
    let pp = ctx.missiles.Missiles[parent].position;
    let missile = mis(ctx, mi);
    missile.position.offset = pp.offset;
    missile.position.traveled = pp.traveled;
}

/// Original: `SpawnLightning` (missiles.cpp).
// @port missiles.cpp|devilution::SpawnLightning(Missile &missile, int dam) sha=484430dce260
fn spawn_lightning(ctx: &mut Ctx, mi: usize, dam: i32) {
    mis(ctx, mi)._mirange -= 1;
    move_missile(
        ctx,
        mi,
        &mut |ctx: &mut Ctx, mi: usize, tile: Point| {
            assert!(in_dungeon_bounds(tile));
            let pn = piece_at(ctx, tile);
            assert!((0..=crate::levels::gendung::MAXTILES as i32).contains(&pn));
            let missile = &ctx.missiles.Missiles[mi];
            if (!missile.is_trap() || tile != missile.position.start) && tile_has_any(ctx, pn, TileProperties::BlockMissile) {
                mis(ctx, mi)._mirange = 0;
                return false;
            }
            true
        },
        false,
    );
    let missile = ctx.missiles.Missiles[mi].clone();
    let position = missile.position.tile;
    let pn = piece_at(ctx, position);
    if !tile_has_any(ctx, pn, TileProperties::BlockMissile) && position != Point::new(missile.var1, missile.var2) && in_dungeon_bounds(position) {
        let mut type_ = MissileID::Lightning;
        if missile.source_type() == MissileSource::Monster && matches!(monster_type_id(ctx, missile.source_monster().unwrap()), MT_STORM | MT_RSTORM | MT_STORML | MT_MAEL) {
            type_ = MissileID::ThinLightning;
        }
        add_missile(ctx, position, missile.position.start, Direction::South, type_, missile._micaster, missile._misource, dam, missile._mispllvl, Some(mi));
        let m = mis(ctx, mi);
        m.var1 = position.x;
        m.var2 = position.y;
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
    }
}

/// Original: `devilution::IsMissileBlockedByTile` (missiles.cpp).
// @port missiles.cpp|devilution::IsMissileBlockedByTile(Point tile) sha=4445096dd84a
pub fn is_missile_blocked_by_tile(ctx: &Ctx, tile: Point) -> bool {
    if !in_dungeon_bounds(tile) {
        return true;
    }
    if tile_has_any(ctx, piece_at(ctx, tile), TileProperties::BlockMissile) {
        return true;
    }
    // _oMissFlag is true if the object allows missiles to pass through so we need to invert the check here...
    match crate::objects::find_object_at_position(ctx, tile, true) {
        Some(o) => !ctx.objects.Objects[o]._oMissFlag,
        None => false,
    }
}

/// Original: `devilution::GetDamageAmt` (missiles.cpp): returns `(mind, maxd)`.
// @port missiles.cpp|devilution::GetDamageAmt(SpellID i, int *mind, int *maxd) sha=3697038672d2
pub fn get_damage_amt(ctx: &Ctx, i: SpellID) -> (i32, i32) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let p = &ctx.players.Players[me];
    let sl = p.get_spell_level(i);
    let lvl = p._pLevel as i32;
    let mag = p._pMagic;
    use SpellID as S;
    match i {
        S::Firebolt => {
            let mind = (mag / 8) + sl + 1;
            (mind, mind + 9)
        }
        S::Healing | S::HealOther => {
            // BUGFIX: healing calculation is unused
            (add_class_healing_bonus(lvl + sl + 1, p._pClass) - 1, add_class_healing_bonus((4 * lvl) + (6 * sl) + 10, p._pClass) - 1)
        }
        S::RuneOfLight | S::Lightning => (2, 2 + lvl),
        S::Flash => {
            let mut mind = scale_spell_effect(lvl, sl);
            mind += mind / 2;
            (mind, mind * 2)
        }
        S::Identify | S::TownPortal | S::StoneCurse | S::Infravision | S::Phasing | S::ManaShield | S::DoomSerpents | S::BloodRitual | S::Invisibility | S::Rage | S::Teleport | S::Etherealize | S::ItemRepair | S::StaffRecharge | S::TrapDisarm | S::Resurrect | S::Telekinesis | S::BoneSpirit | S::Warp | S::Reflect | S::Berserk | S::Search | S::RuneOfStone => (-1, -1),
        S::FireWall | S::LightningWall | S::RingOfFire => {
            let mind = 2 * lvl + 4;
            (mind, mind + 36)
        }
        S::Fireball | S::RuneOfFire => {
            let base = (2 * lvl) + 4;
            (scale_spell_effect(base, sl), scale_spell_effect(base + 36, sl))
        }
        S::Guardian => {
            let base = (lvl / 2) + 1;
            (scale_spell_effect(base, sl), scale_spell_effect(base + 9, sl))
        }
        S::ChainLightning => (4, 4 + (2 * lvl)),
        S::FlameWave => {
            let mind = 6 * (lvl + 1);
            (mind, mind + 54)
        }
        S::Nova | S::Immolation | S::RuneOfImmolation | S::RuneOfNova => (scale_spell_effect((lvl + 5) / 2, sl) * 5, scale_spell_effect((lvl + 30) / 2, sl) * 5),
        S::Inferno => {
            let mut maxd = lvl + 4;
            maxd += maxd / 2;
            (3, maxd)
        }
        S::Golem => (11, 17),
        S::Apocalypse => (lvl, lvl * 6),
        // BUGFIX: add '/ 2' to both values
        S::Elemental => (scale_spell_effect(2 * lvl + 4, sl), scale_spell_effect(2 * lvl + 40, sl)),
        S::ChargedBolt => (1, 1 + (mag / 4)),
        S::HolyBolt => (lvl + 9, lvl + 18),
        S::BloodStar => {
            let mind = (mag / 2) + 3 * sl - (mag / 8);
            (mind, mind)
        }
        // the original leaves the outputs untouched; every caller initialises them to 0 first
        _ => (0, 0),
    }
}

/// Original: `devilution::GetDirection16` (missiles.cpp).
// @port missiles.cpp|devilution::GetDirection16(Point p1, Point p2) sha=dad943e1186d
pub fn get_direction16(p1: Point, p2: Point) -> Direction16 {
    let offset = p2 - p1;
    let mut absolute = offset.abs();
    let flip_y = offset.delta_x != absolute.delta_x;
    let flip_x = offset.delta_y != absolute.delta_y;
    let mut flip_median = false;
    if absolute.delta_x > absolute.delta_y {
        std::mem::swap(&mut absolute.delta_x, &mut absolute.delta_y);
        flip_median = true;
    }
    let mut ret = Direction16::South;
    if 3 * absolute.delta_x <= (absolute.delta_y * 2) {
        // mx/my <= 2/3, approximation of tan(33.75)
        if 5 * absolute.delta_x < absolute.delta_y {
            // mx/my < 0.2, approximation of tan(11.25)
            ret = Direction16::SouthWest;
        } else {
            ret = Direction16::South_SouthWest;
        }
    }
    let mut median_pivot = Direction16::South;
    if flip_y {
        ret = direction16_flip(ret, Direction16::SouthWest);
        median_pivot = direction16_flip(median_pivot, Direction16::SouthWest);
    }
    if flip_x {
        ret = direction16_flip(ret, Direction16::SouthEast);
        median_pivot = direction16_flip(median_pivot, Direction16::SouthEast);
    }
    if flip_median {
        ret = direction16_flip(ret, median_pivot);
    }
    ret
}

/// Original: `devilution::MonsterTrapHit` (missiles.cpp).
// @port missiles.cpp|devilution::MonsterTrapHit(int monsterId, int mindam, int maxdam, int dist, MissileID t, DamageType damageType, bool shift) sha=271dbf05b470
#[allow(clippy::too_many_arguments)]
pub fn monster_trap_hit(ctx: &mut Ctx, monster_id: usize, mindam: i32, maxdam: i32, dist: i32, t: MissileID, damage_type: DamageType, shift: bool) -> bool {
    let m = monster_id;
    if !crate::monster::is_possible_to_hit(ctx, m) || crate::monster::is_immune(ctx, m, t, damage_type) {
        return false;
    }
    let hit = rnd(ctx, 100);
    let mut hper = 90 - ctx.monster.Monsters[m].armorClass as i32 - dist;
    hper = hper.clamp(5, 95);
    if crate::monster::try_lift_gargoyle(ctx, m) {
        return true;
    }
    if hit >= hper && ctx.monster.Monsters[m].mode != MonsterMode::Petrified {
        return false;
    }
    let resist = crate::monster::is_resistant(ctx, m, t, damage_type);
    let mut dam = mindam + rnd(ctx, maxdam - mindam + 1);
    if !shift {
        dam <<= 6;
    }
    if resist {
        dam /= 4;
    }
    crate::monster::apply_monster_damage(ctx, damage_type, m, dam);
    if ctx.monster.Monsters[m].hitPoints >> 6 <= 0 {
        let d = ctx.monster.Monsters[m].direction;
        crate::monster::monster_death_dir(ctx, m, d, true);
    } else if resist {
        crate::monster::play_effect(ctx, m, MonsterSound::Hit);
    } else if monster_type_id(ctx, m) != MT_GOLEM {
        crate::monster::m_start_hit_dam(ctx, m, dam);
    }
    true
}

/// Original: `devilution::PlayerMHit` (missiles.cpp).
// @port missiles.cpp|devilution::PlayerMHit(int pnum, Monster *monster, int dist, int mind, int maxd, MissileID mtype, DamageType damageType, bool shift, DeathReason deathReason, bool *blocked) sha=3ded5607fd4e
#[allow(clippy::too_many_arguments)]
pub fn player_m_hit(ctx: &mut Ctx, pnum: usize, monster: Option<usize>, dist: i32, mind: i32, maxd: i32, mtype: MissileID, damage_type: DamageType, shift: bool, death_reason: DeathReason, blocked: &mut bool) -> bool {
    *blocked = false;
    let player = &ctx.players.Players[pnum];
    if player._pHitPoints >> 6 <= 0 {
        return false;
    }
    if player._pInvincible {
        return false;
    }
    let missile_data = get_missile_data(mtype);
    if player._pSpellFlags.has_any_of(SpellFlag::Etherealize) && missile_data.is_arrow() {
        return false;
    }
    let hit = rnd(ctx, 100);
    let diff = difficulty(ctx);
    let player = &ctx.players.Players[pnum];
    let plevel = player._pLevel as i32;
    let mut hper = 40;
    if missile_data.is_arrow() {
        let tac = player.get_armor();
        if let Some(m) = monster {
            hper = ctx.monster.Monsters[m].toHit as i32 + ((crate::monster::monster_level(ctx, m, diff) as i32 - plevel) * 2) + 30 - (dist * 2) - tac;
        } else {
            hper = 100 - (tac / 2) - (dist * 2);
        }
    } else if let Some(m) = monster {
        hper += (crate::monster::monster_level(ctx, m, diff) as i32 * 2) - (plevel * 2) - (dist * 2);
    }
    let currlevel = ctx.gendung.currlevel;
    let mut minhit = 10;
    if currlevel == 14 {
        minhit = 20;
    }
    if currlevel == 15 {
        minhit = 25;
    }
    if currlevel == 16 {
        minhit = 30;
    }
    hper = hper.max(minhit);
    let mut blk = 100;
    let player = &ctx.players.Players[pnum];
    if (player._pmode == PM_STAND || player._pmode == PM_ATTACK) && player._pBlockFlag {
        blk = rnd(ctx, 100);
    }
    if shift {
        blk = 100;
    }
    if mtype == MissileID::AcidPuddle {
        blk = 100;
    }
    let player = &ctx.players.Players[pnum];
    let mut blkper = player.get_block_chance(false);
    if let Some(m) = monster {
        blkper -= (crate::monster::monster_level(ctx, m, diff) as i32 - plevel) * 2;
    }
    blkper = blkper.clamp(0, 100);
    let resper = resist_percent(player, damage_type) as i8 as i32;
    if hit >= hper {
        return false;
    }
    let mut dam;
    if mtype == MissileID::BoneSpirit {
        dam = player._pHitPoints / 3;
    } else {
        let half_trap = player._pIFlags.has_any_of(ItemSpecialEffect::HalfTrapDamage);
        let get_hit = player._pIGetHit;
        if !shift {
            dam = (mind << 6) + rnd(ctx, ((maxd - mind) << 6) + 1);
            if monster.is_none() && half_trap {
                dam /= 2;
            }
            dam += get_hit * 64;
        } else {
            dam = mind + rnd(ctx, maxd - mind + 1);
            if monster.is_none() && half_trap {
                dam /= 2;
            }
            dam += get_hit;
        }
        dam = dam.max(64);
    }
    if (resper <= 0 || ctx.init.gb_is_hellfire) && blk < blkper {
        let mut dir = ctx.players.Players[pnum]._pdir;
        if let Some(m) = monster {
            dir = crate::engine::get_direction(ctx.players.Players[pnum].position.tile, ctx.monster.Monsters[m].position.tile);
        }
        *blocked = true;
        crate::player::start_plr_block(ctx, pnum, dir);
        return true;
    }
    let is_me = Some(pnum) == ctx.players.MyPlayer;
    if resper > 0 {
        dam -= dam * resper / 100;
        if is_me {
            crate::player::apply_plr_damage(ctx, damage_type, pnum, 0, 0, dam, death_reason);
        }
        if ctx.players.Players[pnum]._pHitPoints >> 6 > 0 {
            crate::player::player_say(ctx, pnum, HeroSpeech::ArghClang);
        }
        return true;
    }
    if is_me {
        crate::player::apply_plr_damage(ctx, damage_type, pnum, 0, 0, dam, death_reason);
    }
    if ctx.players.Players[pnum]._pHitPoints >> 6 > 0 {
        crate::player::start_plr_hit(ctx, pnum, dam, false);
    }
    true
}

/// Original: `devilution::SetMissDir` (missiles.cpp).
// @port missiles.h|devilution::SetMissDir(Missile &missile, Direction16 dir) sha=56ec96b7471e
// @port missiles.h|devilution::SetMissDir(Missile &missile, Direction dir) sha=85d99e7ff129
// @port missiles.cpp|devilution::SetMissDir(Missile &missile, int dir) sha=917f5c38d488
pub fn set_miss_dir(ctx: &mut Ctx, mi: usize, dir: i32) {
    ctx.missiles.Missiles[mi]._mimfnum = dir;
    let t = ctx.missiles.Missiles[mi]._miAnimType;
    set_miss_anim(ctx, mi, t);
}

// ---------------------------------------------------------------------------------------------
// missiles.cpp, part 2: the AddXxx initialisers (first half).
// ---------------------------------------------------------------------------------------------

type P<'a> = &'a mut AddMissileParameter;

/// Original: `devilution::AddOpenNest` (missiles.cpp).
// @port missiles.cpp|devilution::AddOpenNest(Missile &missile, AddMissileParameter &parameter) sha=182f1568ad2f
pub fn add_open_nest(ctx: &mut Ctx, mi: usize, parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    for x in [80, 81] {
        for y in [62, 63] {
            add_missile(ctx, Point::new(x, y), Point::new(80, 62), parameter.midir, MissileID::BigExplosion, m._micaster, m._misource, m._midam, 0, None);
        }
    }
    mis(ctx, mi)._miDelFlag = true;
}

/// Original: `devilution::AddRuneOfFire` (missiles.cpp).
// @port missiles.cpp|devilution::AddRuneOfFire(Missile &missile, AddMissileParameter &parameter) sha=1b75f3bc16f7
pub fn add_rune_of_fire(ctx: &mut Ctx, mi: usize, parameter: P) {
    add_rune(ctx, mi, parameter.dst, MissileID::BigExplosion);
}

/// Original: `devilution::AddRuneOfLight` (missiles.cpp).
// @port missiles.cpp|devilution::AddRuneOfLight(Missile &missile, AddMissileParameter &parameter) sha=1452676141a0
pub fn add_rune_of_light(ctx: &mut Ctx, mi: usize, parameter: P) {
    let m = &ctx.missiles.Missiles[mi];
    let lvl = if m.source_type() == MissileSource::Player { ctx.players.Players[m.source_player().unwrap()]._pLevel as i32 } else { 0 };
    let dmg = 16 * (generate_rnd_sum(ctx, 10, 2) + lvl + 2);
    mis(ctx, mi)._midam = dmg;
    add_rune(ctx, mi, parameter.dst, MissileID::LightningWall);
}

/// Original: `devilution::AddRuneOfNova` (missiles.cpp).
// @port missiles.cpp|devilution::AddRuneOfNova(Missile &missile, AddMissileParameter &parameter) sha=dc6f8912a7d8
pub fn add_rune_of_nova(ctx: &mut Ctx, mi: usize, parameter: P) {
    add_rune(ctx, mi, parameter.dst, MissileID::Nova);
}

/// Original: `devilution::AddRuneOfImmolation` (missiles.cpp).
// @port missiles.cpp|devilution::AddRuneOfImmolation(Missile &missile, AddMissileParameter &parameter) sha=749b84c8e11f
pub fn add_rune_of_immolation(ctx: &mut Ctx, mi: usize, parameter: P) {
    add_rune(ctx, mi, parameter.dst, MissileID::Immolation);
}

/// Original: `devilution::AddRuneOfStone` (missiles.cpp).
// @port missiles.cpp|devilution::AddRuneOfStone(Missile &missile, AddMissileParameter &parameter) sha=06e8cfa7397f
pub fn add_rune_of_stone(ctx: &mut Ctx, mi: usize, parameter: P) {
    add_rune(ctx, mi, parameter.dst, MissileID::StoneCurse);
}

/// Original: `devilution::AddReflect` (missiles.cpp).
// @port missiles.cpp|devilution::AddReflect(Missile &missile, AddMissileParameter &) sha=2681e31f095a
pub fn add_reflect(ctx: &mut Ctx, mi: usize, _parameter: P) {
    mis(ctx, mi)._miDelFlag = true;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.source_type() != MissileSource::Player {
        return;
    }
    let pnum = m.source_player().unwrap();
    let player = &mut ctx.players.Players[pnum];
    let mut add = (if m._mispllvl != 0 { m._mispllvl } else { 2 }) * player._pLevel as i32;
    if player.wReflections as i32 + add >= u16::MAX as i32 {
        add = 0;
    }
    player.wReflections = (player.wReflections as i32 + add) as u16;
    let r = player.wReflections;
    if Some(pnum) == ctx.players.MyPlayer {
        crate::msg::net_send_cmd_param1(ctx, true, CMD_SETREFLECT, r);
    }
}

/// Original: `devilution::AddBerserk` (missiles.cpp).
// @port missiles.cpp|devilution::AddBerserk(Missile &missile, AddMissileParameter &parameter) sha=44b55a7c384a
pub fn add_berserk(ctx: &mut Ctx, mi: usize, parameter: P) {
    mis(ctx, mi)._miDelFlag = true;
    parameter.spellFizzled = true;
    if ctx.missiles.Missiles[mi].source_type() == MissileSource::Trap {
        return;
    }
    // the position check below consumes randomness (FlipCoin), so it runs on a copy of the RNG
    // state that is written back afterwards, as the original's lambda shares the global state
    let rng = std::cell::RefCell::new(ctx.rng.clone());
    let target_monster_position = crate::engine::path::find_closest_valid_position(
        ctx,
        &|ctx: &Ctx, target: Point| {
            if !in_dungeon_bounds(target) {
                return false;
            }
            let monster_id = (ctx.gendung.dMonster[target.x as usize][target.y as usize] as i32).abs() - 1;
            if monster_id < 0 {
                return false;
            }
            let monster = &ctx.monster.Monsters[monster_id as usize];
            if monster.is_player_minion() {
                return false;
            }
            if (monster.flags & MFLAG_BERSERK as u32) != 0 {
                return false;
            }
            if monster.is_unique() || monster.ai == MonsterAIID::Diablo {
                return false;
            }
            if matches!(monster.mode, MonsterMode::FadeIn | MonsterMode::FadeOut | MonsterMode::Charge) {
                return false;
            }
            if (monster.resistance & IMMUNE_MAGIC as u16) != 0 {
                return false;
            }
            if (monster.resistance & RESIST_MAGIC as u16) != 0 && ((monster.resistance & RESIST_MAGIC as u16) != 1 || !rng.borrow_mut().flip_coin(2)) {
                return false;
            }
            true
        },
        parameter.dst,
        0,
        5,
    );
    ctx.rng = rng.into_inner();
    if let Some(p) = target_monster_position {
        let m = ((ctx.gendung.dMonster[p.x as usize][p.y as usize] as i32).abs() - 1) as usize;
        let pnum = ctx.missiles.Missiles[mi].source_player().unwrap();
        let slvl = ctx.players.Players[pnum].get_spell_level(SpellID::Berserk);
        ctx.monster.Monsters[m].flags |= (MFLAG_BERSERK | MFLAG_GOLEM) as u32;
        let r1 = rnd(ctx, 10);
        let r2 = rnd(ctx, 10);
        let r3 = rnd(ctx, 10);
        let r4 = rnd(ctx, 10);
        let monster = &mut ctx.monster.Monsters[m];
        monster.minDamage = ((r1 + 120) * monster.minDamage as i32 / 100 + slvl) as u8;
        monster.maxDamage = ((r2 + 120) * monster.maxDamage as i32 / 100 + slvl) as u8;
        monster.minDamageSpecial = ((r3 + 120) * monster.minDamageSpecial as i32 / 100 + slvl) as u8;
        monster.maxDamageSpecial = ((r4 + 120) * monster.maxDamageSpecial as i32 / 100 + slvl) as u8;
        let light_radius = if ctx.gendung.leveltype == DungeonType::Nest { 9 } else { 3 };
        let t = ctx.monster.Monsters[m].position.tile;
        ctx.monster.Monsters[m].lightId = crate::lighting::add_light(ctx, t, light_radius) as i8;
        parameter.spellFizzled = false;
    }
}

/// Original: `devilution::AddHorkSpawn` (missiles.cpp).
// @port missiles.cpp|devilution::AddHorkSpawn(Missile &missile, AddMissileParameter &parameter) sha=0e7789e6e0a1
pub fn add_hork_spawn(ctx: &mut Ctx, mi: usize, parameter: P) {
    update_missile_velocity(ctx, mi, parameter.dst, 8);
    let m = mis(ctx, mi);
    m._mirange = 9;
    m.var1 = parameter.midir as i32;
    put_missile(ctx, mi);
}

/// Original: `devilution::AddJester` (missiles.cpp).
// @port missiles.cpp|devilution::AddJester(Missile &missile, AddMissileParameter &parameter) sha=c3438117ee0d
pub fn add_jester(ctx: &mut Ctx, mi: usize, parameter: P) {
    let spell = match rnd(ctx, 10) {
        0 | 1 => MissileID::Firebolt,
        2 => MissileID::Fireball,
        3 => MissileID::FireWallControl,
        4 => MissileID::Guardian,
        5 => MissileID::ChainLightning,
        6 => MissileID::TownPortal,
        7 => MissileID::Teleport,
        8 => MissileID::Apocalypse,
        9 => MissileID::StoneCurse,
        _ => MissileID::Firebolt,
    };
    let m = ctx.missiles.Missiles[mi].clone();
    let random_missile = add_missile(ctx, m.position.start, parameter.dst, parameter.midir, spell, m._micaster, m._misource, 0, m._mispllvl, None);
    parameter.spellFizzled = random_missile.is_none();
    mis(ctx, mi)._miDelFlag = true;
}

/// Original: `devilution::AddStealPotions` (missiles.cpp).
// @port missiles.cpp|devilution::AddStealPotions(Missile &missile, AddMissileParameter &) sha=1845310e4a36
pub fn add_steal_potions(ctx: &mut Ctx, mi: usize, _parameter: P) {
    use crate::objects::item_misc_id_idx;
    let start = ctx.missiles.Missiles[mi].position.start;
    crate::lighting::do_crawl_range(0, 2, &mut |displacement| {
        let target = start + displacement;
        if !in_dungeon_bounds(target) {
            return false;
        }
        let pnum = ctx.gendung.dPlayer[target.x as usize][target.y as usize];
        if pnum == 0 {
            return false;
        }
        let p = ((pnum as i32).abs() - 1) as usize;
        let mut has_played_sfx = false;
        for si in 0..crate::player::MaxBeltItems {
            let mut ii = IDI_NONE;
            if ctx.players.Players[p].SpdList[si]._itype == ItemType::Misc {
                if ctx.rng.flip_coin(2) {
                    continue;
                }
                match ctx.players.Players[p].SpdList[si]._iMiscId {
                    IMISC_FULLHEAL => ii = item_misc_id_idx(IMISC_HEAL),
                    IMISC_HEAL | IMISC_MANA => crate::player::remove_spd_bar_item(ctx, p, si as i32),
                    IMISC_FULLMANA => ii = item_misc_id_idx(IMISC_MANA),
                    IMISC_REJUV => {
                        let m = ctx.rng.pick_randomly_among(&[IMISC_HEAL, IMISC_MANA]);
                        ii = item_misc_id_idx(m);
                    }
                    IMISC_FULLREJUV => {
                        ii = match ctx.rng.generate_rnd(3) {
                            0 => item_misc_id_idx(IMISC_FULLMANA),
                            1 => item_misc_id_idx(IMISC_FULLHEAL),
                            _ => item_misc_id_idx(IMISC_REJUV),
                        };
                    }
                    _ => continue,
                }
            }
            if ii != IDI_NONE {
                let mut belt_item = ctx.players.Players[p].SpdList[si].clone();
                let seed = belt_item._iSeed;
                crate::items::initialize_item(ctx, &mut belt_item, ii);
                belt_item._iSeed = seed;
                belt_item._iStatFlag = true;
                ctx.players.Players[p].SpdList[si] = belt_item;
            }
            if !has_played_sfx {
                crate::effects::play_sfx_loc(ctx, crate::effects_data::IS_POPPOP2, target, true);
                has_played_sfx = true;
            }
        }
        crate::engine::backbuffer_state::redraw_everything(ctx);
        false
    });
    mis(ctx, mi)._miDelFlag = true;
}

/// Original: `devilution::AddStealMana` (missiles.cpp).
// @port missiles.cpp|devilution::AddStealMana(Missile &missile, AddMissileParameter &) sha=654aa6584018
pub fn add_steal_mana(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let start = ctx.missiles.Missiles[mi].position.start;
    let trapped = crate::engine::path::find_closest_valid_position(ctx, &|ctx: &Ctx, target: Point| in_dungeon_bounds(target) && ctx.gendung.dPlayer[target.x as usize][target.y as usize] != 0, start, 0, 2);
    if let Some(tp) = trapped {
        let p = ((ctx.gendung.dPlayer[tp.x as usize][tp.y as usize] as i32).abs() - 1) as usize;
        let player = &mut ctx.players.Players[p];
        player._pMana = 0;
        player._pManaBase = player._pMana + player._pMaxManaBase - player._pMaxMana;
        crate::items::calc_plr_inv(ctx, p, false);
        crate::engine::backbuffer_state::redraw_component(ctx, crate::engine::backbuffer_state::PanelDrawComponent::Mana);
        crate::effects::play_sfx_loc(ctx, crate::effects_data::TSFX_COW7, tp, true);
    }
    mis(ctx, mi)._miDelFlag = true;
}

/// Bonus to arrow velocity from the player's attack speed affixes.
fn attack_speed_bonus(player: &crate::player::Player) -> i32 {
    let mut av = 0;
    if player._pIFlags.has_any_of(ItemSpecialEffect::QuickAttack) {
        av += 1;
    }
    if player._pIFlags.has_any_of(ItemSpecialEffect::FastAttack) {
        av += 2;
    }
    if player._pIFlags.has_any_of(ItemSpecialEffect::FasterAttack) {
        av += 4;
    }
    if player._pIFlags.has_any_of(ItemSpecialEffect::FastestAttack) {
        av += 8;
    }
    av
}

/// Original: `devilution::AddSpectralArrow` (missiles.cpp).
// @port missiles.cpp|devilution::AddSpectralArrow(Missile &missile, AddMissileParameter &parameter) sha=62a379e27f21
pub fn add_spectral_arrow(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut av = 0;
    let m = &ctx.missiles.Missiles[mi];
    if m.source_type() == MissileSource::Player {
        let player = &ctx.players.Players[m.source_player().unwrap()];
        let lvl = player._pLevel as i32;
        if player._pClass == HeroClass::Rogue {
            av += (lvl - 1) / 4;
        } else if player._pClass == HeroClass::Warrior || player._pClass == HeroClass::Bard {
            av += (lvl - 1) / 8;
        }
        av += attack_speed_bonus(player);
    }
    let m = mis(ctx, mi);
    m._mirange = 1;
    m.var1 = parameter.dst.x;
    m.var2 = parameter.dst.y;
    m.var3 = av;
}

/// Original: `devilution::AddWarp` (missiles.cpp).
// @port missiles.cpp|devilution::AddWarp(Missile &missile, AddMissileParameter &parameter) sha=60eecf6b7704
pub fn add_warp(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut min_distance_sq = i32::MAX;
    let id = ctx.missiles.Missiles[mi]._misource as usize;
    let ptile = ctx.players.Players[id].position.tile;
    let mut tile = ptile;
    let mut i = 0;
    while i < ctx.trigs.numtrigs as usize && i < crate::levels::trigs::MAXTRIGGERS {
        let trg = ctx.trigs.trigs[i];
        i += 1;
        if !matches!(trg._tmsg, WM_DIABTWARPUP | WM_DIABPREVLVL | WM_DIABNEXTLVL | WM_DIABRTNLVL) {
            continue;
        }
        let up = matches!(trg._tmsg, WM_DIABTWARPUP | WM_DIABPREVLVL);
        let up_or_rtn = matches!(trg._tmsg, WM_DIABTWARPUP | WM_DIABPREVLVL | WM_DIABRTNLVL);
        let trigger_offset = match ctx.gendung.leveltype {
            DungeonType::Cathedral => {
                if ctx.gendung.setlevel && ctx.gendung.setlvlnum == SL_VILEBETRAYER {
                    Displacement::new(1, 1) // Portal
                } else if up_or_rtn {
                    Displacement::new(1, 2)
                } else {
                    Displacement::new(0, 1) // WM_DIABNEXTLVL
                }
            }
            DungeonType::Catacombs => {
                if up {
                    Displacement::new(1, 1)
                } else {
                    Displacement::new(0, 1) // WM_DIABRTNLVL, WM_DIABNEXTLVL
                }
            }
            DungeonType::Caves => {
                if up {
                    Displacement::new(0, 1)
                } else {
                    Displacement::new(1, 0) // WM_DIABRTNLVL, WM_DIABNEXTLVL
                }
            }
            DungeonType::Hell => Displacement::new(1, 0),
            DungeonType::Nest => {
                if up_or_rtn {
                    Displacement::new(0, 1)
                } else {
                    Displacement::new(1, 0) // WM_DIABNEXTLVL
                }
            }
            DungeonType::Crypt => {
                if up_or_rtn {
                    Displacement::new(1, 1)
                } else {
                    Displacement::new(0, 1) // WM_DIABNEXTLVL
                }
            }
            DungeonType::Town => crate::appfat::app_fatal(ctx, "invalid leveltype: DTYPE_TOWN"),
            DungeonType::None => crate::appfat::app_fatal(ctx, "leveltype not set"),
        };
        let candidate = trg.position + trigger_offset;
        let off = ptile - candidate;
        let distance_sq = off.delta_y * off.delta_y + off.delta_x * off.delta_x;
        if distance_sq < min_distance_sq {
            min_distance_sq = distance_sq;
            tile = candidate;
        }
    }
    mis(ctx, mi)._mirange = 2;
    let teleport_destination = crate::engine::path::find_closest_valid_position(
        ctx,
        &|ctx: &Ctx, target: Point| {
            for i in 0..ctx.trigs.numtrigs as usize {
                if ctx.trigs.trigs[i].position == target {
                    return false;
                }
            }
            crate::player::pos_ok_player(ctx, id, target)
        },
        tile,
        0,
        5,
    );
    match teleport_destination {
        Some(t) => mis(ctx, mi).position.tile = t,
        None => {
            // No valid teleport destination found
            mis(ctx, mi)._miDelFlag = true;
            parameter.spellFizzled = true;
        }
    }
}

/// Original: `devilution::AddLightningWall` (missiles.cpp).
// @port missiles.cpp|devilution::AddLightningWall(Missile &missile, AddMissileParameter &parameter) sha=816bff6259b7
pub fn add_lightning_wall(ctx: &mut Ctx, mi: usize, parameter: P) {
    update_missile_velocity(ctx, mi, parameter.dst, 16);
    let f = rnd(ctx, 8) + 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let (v1, v2) = match m.source_type() {
        MissileSource::Trap => (m.position.start.x, m.position.start.y),
        MissileSource::Player => {
            let t = ctx.players.Players[m.source_player().unwrap()].position.tile;
            (t.x, t.y)
        }
        MissileSource::Monster => {
            assert!(m.source_type() != MissileSource::Monster);
            (m.var1, m.var2)
        }
    };
    let m = mis(ctx, mi);
    m._miAnimFrame = f;
    m._mirange = 255 * (m._mispllvl + 1);
    m.var1 = v1;
    m.var2 = v2;
}

/// Original: `devilution::AddBigExplosion` (missiles.cpp).
// @port missiles.cpp|devilution::AddBigExplosion(Missile &missile, AddMissileParameter &) sha=da49c753e196
pub fn add_big_explosion(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    if m.source_type() == MissileSource::Player {
        let lvl = ctx.players.Players[m.source_player().unwrap()]._pLevel as i32;
        let mut dmg = 2 * (lvl + generate_rnd_sum(ctx, 10, 2)) + 4;
        dmg = scale_spell_effect(dmg, m._mispllvl);
        mis(ctx, mi)._midam = dmg;
        let damage_type = get_missile_data(m._mitype).damage_type();
        for position in crate::engine::geometry::points_in_rectangle_col_major(crate::engine::geometry::Rectangle::from_center(m.position.tile, 1)) {
            check_missile_col(ctx, mi, damage_type, dmg, dmg, false, position, true);
        }
    }
    let start = ctx.missiles.Missiles[mi].position.start;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, start, 8);
    set_miss_dir(ctx, mi, 0);
    let m = mis(ctx, mi);
    m._mirange = m._miAnimLen - 1;
}

/// Original: `devilution::AddImmolation` (missiles.cpp).
// @port missiles.cpp|devilution::AddImmolation(Missile &missile, AddMissileParameter &parameter) sha=05e074de097b
pub fn add_immolation(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.start == parameter.dst {
        dst += parameter.midir;
    }
    let mut sp = 16;
    if m._micaster == TARGET_MONSTERS {
        sp += m._mispllvl.min(34);
    }
    update_missile_velocity(ctx, mi, dst, sp);
    set_miss_dir(ctx, mi, get_direction16(m.position.start, dst) as i32);
    mis(ctx, mi)._mirange = 256;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.start, 8);
}

/// Original: `devilution::AddLightningBow` (missiles.cpp).
// @port missiles.cpp|devilution::AddLightningBow(Missile &missile, AddMissileParameter &parameter) sha=c5f22368ae2e
pub fn add_lightning_bow(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.start == parameter.dst {
        dst += parameter.midir;
    }
    update_missile_velocity(ctx, mi, dst, 32);
    let f = rnd(ctx, 8) + 1;
    let v = if m._misource < 0 { m.position.start } else { ctx.players.Players[m._misource as usize].position.tile };
    let m = mis(ctx, mi);
    m._miAnimFrame = f;
    m._mirange = 255;
    m.var1 = v.x;
    m.var2 = v.y;
    m._midam <<= 6;
}

/// Original: `devilution::AddMana` (missiles.cpp).
// @port missiles.cpp|devilution::AddMana(Missile &missile, AddMissileParameter &) sha=c64fbd08a4ee
pub fn add_mana(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    let p = m._misource as usize;
    let mut mana_amount = (rnd(ctx, 10) + 1) << 6;
    for _ in 0..ctx.players.Players[p]._pLevel {
        mana_amount += (rnd(ctx, 4) + 1) << 6;
    }
    for _ in 0..m._mispllvl {
        mana_amount += (rnd(ctx, 6) + 1) << 6;
    }
    let player = &mut ctx.players.Players[p];
    if player._pClass == HeroClass::Sorcerer {
        mana_amount *= 2;
    }
    if player._pClass == HeroClass::Rogue || player._pClass == HeroClass::Bard {
        mana_amount += mana_amount / 2;
    }
    player._pMana += mana_amount;
    if player._pMana > player._pMaxMana {
        player._pMana = player._pMaxMana;
    }
    player._pManaBase += mana_amount;
    if player._pManaBase > player._pMaxManaBase {
        player._pManaBase = player._pMaxManaBase;
    }
    mis(ctx, mi)._miDelFlag = true;
    crate::engine::backbuffer_state::redraw_component(ctx, crate::engine::backbuffer_state::PanelDrawComponent::Mana);
}

/// Original: `devilution::AddMagi` (missiles.cpp).
// @port missiles.cpp|devilution::AddMagi(Missile &missile, AddMissileParameter &) sha=5d3a852edab0
pub fn add_magi(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    let player = &mut ctx.players.Players[p];
    player._pMana = player._pMaxMana;
    player._pManaBase = player._pMaxManaBase;
    mis(ctx, mi)._miDelFlag = true;
    crate::engine::backbuffer_state::redraw_component(ctx, crate::engine::backbuffer_state::PanelDrawComponent::Mana);
}

/// Original: `devilution::AddRingOfFire` (missiles.cpp).
// @port missiles.cpp|devilution::AddRingOfFire(Missile &missile, AddMissileParameter &) sha=acdfed8af06c
pub fn add_ring_of_fire(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let m = mis(ctx, mi);
    m.var1 = m.position.start.x;
    m.var2 = m.position.start.y;
    m._mirange = 7;
}

/// Original: `devilution::AddSearch` (missiles.cpp).
// @port missiles.cpp|devilution::AddSearch(Missile &missile, AddMissileParameter &) sha=4383702713f3
pub fn add_search(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    let p = m._misource as usize;
    if Some(p) == ctx.players.MyPlayer {
        ctx.scrollrt.AutoMapShowItems = true;
    }
    let mut lvl = 2;
    if m._misource >= 0 {
        lvl = ctx.players.Players[p]._pLevel as i32 * 2;
    }
    mis(ctx, mi)._mirange = lvl + 10 * m._mispllvl + 245;
    for o in 0..ctx.missiles.Missiles.len() {
        let me = ctx.missiles.Missiles[mi].clone();
        let other = &ctx.missiles.Missiles[o];
        if o != mi && me.is_same_source(other) && other._mitype == MissileID::Search {
            let r1 = me._mirange;
            let r2 = other._mirange;
            if r2 < i32::MAX - r1 {
                ctx.missiles.Missiles[o]._mirange = r1 + r2;
            }
            mis(ctx, mi)._miDelFlag = true;
            break;
        }
    }
}

/// Original: `devilution::AddChargedBoltBow` (missiles.cpp).
// @port missiles.cpp|devilution::AddChargedBoltBow(Missile &missile, AddMissileParameter &parameter) sha=8d01b37e300b
pub fn add_charged_bolt_bow(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    mis(ctx, mi)._mirnd = rnd(ctx, 15) + 1;
    if ctx.missiles.Missiles[mi]._micaster != TARGET_MONSTERS {
        mis(ctx, mi)._midam = 15;
    }
    let start = ctx.missiles.Missiles[mi].position.start;
    if start == dst {
        dst += parameter.midir;
    }
    mis(ctx, mi)._miAnimFrame = rnd(ctx, 8) + 1;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, start, 5);
    update_missile_velocity(ctx, mi, dst, 8);
    let m = mis(ctx, mi);
    m.var1 = 5;
    m.var2 = parameter.midir as i32;
    m._mirange = 256;
}

/// Original: `devilution::AddElementalArrow` (missiles.cpp).
// @port missiles.cpp|devilution::AddElementalArrow(Missile &missile, AddMissileParameter &parameter) sha=dea68575c1fe
pub fn add_elemental_arrow(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.start == dst {
        dst += parameter.midir;
    }
    let mut av = 32;
    if m._micaster == TARGET_MONSTERS {
        let player = &ctx.players.Players[m._misource as usize];
        let lvl = player._pLevel as i32;
        if player._pClass == HeroClass::Rogue {
            av += lvl / 4;
        } else if matches!(player._pClass, HeroClass::Warrior | HeroClass::Bard) {
            av += lvl / 8;
        }
        if ctx.init.gb_is_hellfire {
            av += attack_speed_bonus(player);
        } else if matches!(player._pClass, HeroClass::Rogue | HeroClass::Warrior | HeroClass::Bard) {
            av -= 1;
        }
    }
    update_missile_velocity(ctx, mi, dst, av);
    set_miss_dir(ctx, mi, get_direction16(m.position.start, dst) as i32);
    let mm = mis(ctx, mi);
    mm._mirange = 256;
    mm.var1 = m.position.start.x;
    mm.var2 = m.position.start.y;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.start, 5);
}

/// Original: `devilution::AddArrow` (missiles.cpp).
// @port missiles.cpp|devilution::AddArrow(Missile &missile, AddMissileParameter &parameter) sha=db4f796d015b
pub fn add_arrow(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.start == dst {
        dst += parameter.midir;
    }
    let mut av = 32;
    if m._micaster == TARGET_MONSTERS {
        let p = m._misource as usize;
        if ctx.players.Players[p]._pIFlags.has_any_of(ItemSpecialEffect::RandomArrowVelocity) {
            av = rnd(ctx, 32) + 16;
        }
        let player = &ctx.players.Players[p];
        let lvl = player._pLevel as i32;
        if player._pClass == HeroClass::Rogue {
            av += (lvl - 1) / 4;
        } else if player._pClass == HeroClass::Warrior || player._pClass == HeroClass::Bard {
            av += (lvl - 1) / 8;
        }
        if ctx.init.gb_is_hellfire {
            av += attack_speed_bonus(player);
        }
    }
    update_missile_velocity(ctx, mi, dst, av);
    let mm = mis(ctx, mi);
    mm._miAnimFrame = get_direction16(m.position.start, dst) as i32 + 1;
    mm._mirange = 256;
}

/// Original: `UpdateVileMissPos` (missiles.cpp).
// @port missiles.cpp|devilution::UpdateVileMissPos(Missile &missile, Point dst) sha=36cded5772c1
fn update_vile_miss_pos(ctx: &mut Ctx, mi: usize, dst: Point) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    for k in 1..50 {
        for j in -k..=k {
            let yy = j + dst.y;
            for i in -k..=k {
                let xx = i + dst.x;
                if crate::player::pos_ok_player(ctx, me, Point::new(xx, yy)) {
                    mis(ctx, mi).position.tile = Point::new(xx, yy);
                    return;
                }
            }
        }
    }
}

/// Original: `devilution::AddPhasing` (missiles.cpp).
// @port missiles.cpp|devilution::AddPhasing(Missile &missile, AddMissileParameter &parameter) sha=724a949f8e37
pub fn add_phasing(ctx: &mut Ctx, mi: usize, parameter: P) {
    mis(ctx, mi)._mirange = 2;
    let m = ctx.missiles.Missiles[mi].clone();
    let p = m._misource as usize;
    if m._micaster == TARGET_BOTH {
        mis(ctx, mi).position.tile = parameter.dst;
        if !crate::player::pos_ok_player(ctx, p, parameter.dst) {
            update_vile_miss_pos(ctx, mi, parameter.dst);
        }
        return;
    }
    let mut targets: Vec<Point> = Vec::with_capacity(36);
    for y in -6..=6 {
        for x in -6..=6 {
            if (-3..=3).contains(&x) || (-3..=3).contains(&y) {
                continue; // Skip center
            }
            let target = m.position.start + Displacement::new(x, y);
            if !crate::player::pos_ok_player(ctx, p, target) {
                continue;
            }
            targets.push(target);
        }
    }
    if targets.is_empty() {
        mis(ctx, mi)._miDelFlag = true;
        return;
    }
    let r = rnd(ctx, targets.len() as i32).max(0) as usize;
    mis(ctx, mi).position.tile = targets[r];
}

fn set_projectile_damage(ctx: &mut Ctx, mi: usize, player_damage: Option<fn(&mut Ctx, usize) -> i32>) {
    if ctx.missiles.Missiles[mi]._midam != 0 {
        return;
    }
    let dam = match ctx.missiles.Missiles[mi].source_type() {
        MissileSource::Player => match player_damage {
            Some(f) => f(ctx, mi),
            None => return,
        },
        MissileSource::Monster => projectile_monster_damage(ctx, mi),
        MissileSource::Trap => projectile_trap_damage(ctx, mi),
    };
    mis(ctx, mi)._midam = dam;
}

/// Original: `devilution::AddFirebolt` (missiles.cpp).
// @port missiles.cpp|devilution::AddFirebolt(Missile &missile, AddMissileParameter &parameter) sha=ddc833c07def
pub fn add_firebolt(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.start == dst {
        dst += parameter.midir;
    }
    let mut sp = 26;
    if m._micaster == TARGET_MONSTERS {
        sp = 16;
        if !m.is_trap() {
            sp += (m._mispllvl * 2).min(47);
        }
    }
    update_missile_velocity(ctx, mi, dst, sp);
    set_miss_dir(ctx, mi, get_direction16(m.position.start, dst) as i32);
    let mm = mis(ctx, mi);
    mm._mirange = 256;
    mm.var1 = m.position.start.x;
    mm.var2 = m.position.start.y;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.start, 8);
    set_projectile_damage(
        ctx,
        mi,
        Some(|ctx: &mut Ctx, mi: usize| {
            let m = ctx.missiles.Missiles[mi].clone();
            let mag = ctx.players.Players[m.source_player().unwrap()]._pMagic;
            rnd(ctx, 10) + (mag / 8) + m._mispllvl + 1
        }),
    );
}

/// Original: `devilution::AddMagmaBall` (missiles.cpp).
// @port missiles.cpp|devilution::AddMagmaBall(Missile &missile, AddMissileParameter &parameter) sha=dad03426d9f2
pub fn add_magma_ball(ctx: &mut Ctx, mi: usize, parameter: P) {
    update_missile_velocity(ctx, mi, parameter.dst, 16);
    {
        let m = mis(ctx, mi);
        m.position.traveled.delta_x += 3 * m.position.velocity.delta_x;
        m.position.traveled.delta_y += 3 * m.position.velocity.delta_y;
    }
    update_missile_pos(ctx, mi);
    let hf = ctx.init.gb_is_hellfire;
    let m = mis(ctx, mi);
    if !hf || (m.position.velocity.delta_x as u32 & 0xFFFF0000) != 0 || (m.position.velocity.delta_y as u32 & 0xFFFF0000) != 0 {
        m._mirange = 256;
    } else {
        m._mirange = 1;
    }
    m.var1 = m.position.start.x;
    m.var2 = m.position.start.y;
    let start = m.position.start;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, start, 8);
    // Not typically created by Players
    set_projectile_damage(ctx, mi, None);
}

/// Original: `devilution::AddTeleport` (missiles.cpp).
// @port missiles.cpp|devilution::AddTeleport(Missile &missile, AddMissileParameter &parameter) sha=98f50763320f
pub fn add_teleport(ctx: &mut Ctx, mi: usize, parameter: P) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    let dest = crate::engine::path::find_closest_valid_position(ctx, &|ctx: &Ctx, target: Point| crate::player::pos_ok_player(ctx, p, target), parameter.dst, 0, 5);
    match dest {
        Some(t) => {
            let m = mis(ctx, mi);
            m.position.tile = t;
            m.position.start = t;
            m._mirange = 2;
        }
        None => {
            mis(ctx, mi)._miDelFlag = true;
            parameter.spellFizzled = true;
        }
    }
}

/// Original: `devilution::AddNovaBall` (missiles.cpp).
// @port missiles.cpp|devilution::AddNovaBall(Missile &missile, AddMissileParameter &parameter) sha=ace8dec0fe0c
pub fn add_nova_ball(ctx: &mut Ctx, mi: usize, parameter: P) {
    update_missile_velocity(ctx, mi, parameter.dst, 16);
    let f = rnd(ctx, 8) + 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let position = if m._misource < 0 { m.position.start } else { ctx.players.Players[m._misource as usize].position.tile };
    let mm = mis(ctx, mi);
    mm._miAnimFrame = f;
    mm._mirange = 255;
    mm.var1 = position.x;
    mm.var2 = position.y;
}

/// Original: `devilution::AddFireWall` (missiles.cpp).
// @port missiles.cpp|devilution::AddFireWall(Missile &missile, AddMissileParameter &parameter) sha=0fbe02cea79c
pub fn add_fire_wall(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dam = generate_rnd_sum(ctx, 10, 2) + 2;
    let m = ctx.missiles.Missiles[mi].clone();
    let currlevel = ctx.gendung.currlevel as i32;
    dam += if m._misource >= 0 { ctx.players.Players[m._misource as usize]._pLevel as i32 } else { currlevel }; // BUGFIX: missing parenthesis around ternary (fixed)
    dam <<= 3;
    mis(ctx, mi)._midam = dam;
    update_missile_velocity(ctx, mi, parameter.dst, 16);
    let mm = mis(ctx, mi);
    let i = mm._mispllvl;
    mm._mirange = 10;
    if i > 0 {
        mm._mirange *= i + 1;
    }
    if mm._micaster == TARGET_PLAYERS || mm._misource < 0 {
        mm._mirange += currlevel;
    }
    mm._mirange *= 16;
    mm.var1 = mm._mirange - mm._miAnimLen;
}

/// Original: `devilution::AddFireball` (missiles.cpp).
// @port missiles.cpp|devilution::AddFireball(Missile &missile, AddMissileParameter &parameter) sha=87566a209caa
pub fn add_fireball(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.start == dst {
        dst += parameter.midir;
    }
    let mut sp = 16;
    if m._micaster == TARGET_MONSTERS {
        sp += (m._mispllvl * 2).min(34);
        let lvl = ctx.players.Players[m._misource as usize]._pLevel as i32;
        let dmg = 2 * (lvl + generate_rnd_sum(ctx, 10, 2)) + 4;
        mis(ctx, mi)._midam = scale_spell_effect(dmg, m._mispllvl);
    }
    update_missile_velocity(ctx, mi, dst, sp);
    set_miss_dir(ctx, mi, get_direction16(m.position.start, dst) as i32);
    let mm = mis(ctx, mi);
    mm._mirange = 256;
    mm.var1 = m.position.start.x;
    mm.var2 = m.position.start.y;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.start, 8);
}

/// Original: `devilution::AddLightningControl` (missiles.cpp).
// @port missiles.cpp|devilution::AddLightningControl(Missile &missile, AddMissileParameter &parameter) sha=51a265ca5cb2
pub fn add_lightning_control(ctx: &mut Ctx, mi: usize, parameter: P) {
    {
        let m = mis(ctx, mi);
        m.var1 = m.position.start.x;
        m.var2 = m.position.start.y;
    }
    update_missile_velocity(ctx, mi, parameter.dst, 32);
    let f = rnd(ctx, 8) + 1;
    let m = mis(ctx, mi);
    m._miAnimFrame = f;
    m._mirange = 256;
}

/// Original: `devilution::AddLightning` (missiles.cpp).
// @port missiles.cpp|devilution::AddLightning(Missile &missile, AddMissileParameter &parameter) sha=46618df4e444
pub fn add_lightning(ctx: &mut Ctx, mi: usize, parameter: P) {
    mis(ctx, mi).position.start = parameter.dst;
    sync_position_with_parent(ctx, mi, parameter);
    mis(ctx, mi)._miAnimFrame = rnd(ctx, 8) + 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let range = if m._micaster == TARGET_PLAYERS || m.is_trap() {
        if m.is_trap() || monster_type_id(ctx, m._misource as usize) == MT_FAMILIAR {
            8
        } else {
            10
        }
    } else {
        (m._mispllvl / 2) + 6
    };
    mis(ctx, mi)._mirange = range;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.tile, 4);
}

/// Original: `devilution::AddMissileExplosion` (missiles.cpp).
// @port missiles.cpp|devilution::AddMissileExplosion(Missile &missile, AddMissileParameter &parameter) sha=83ebfcf1b975
pub fn add_missile_explosion(ctx: &mut Ctx, mi: usize, parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    if m._micaster != TARGET_MONSTERS && m._misource >= 0 {
        match monster_type_id(ctx, m._misource as usize) {
            MT_SUCCUBUS => set_miss_anim(ctx, mi, MissileGraphicID::BloodStarExplosion),
            MT_SNOWWICH => set_miss_anim(ctx, mi, MissileGraphicID::BloodStarBlueExplosion),
            MT_HLSPWN => set_miss_anim(ctx, mi, MissileGraphicID::BloodStarRedExplosion),
            MT_SOLBRNR => set_miss_anim(ctx, mi, MissileGraphicID::BloodStarYellowExplosion),
            _ => {}
        }
    }
    // AddMissileExplosion will always be called with a parent associated to the missile.
    let parent = parameter.pParent.expect("parent");
    let pp = ctx.missiles.Missiles[parent].position;
    let mm = mis(ctx, mi);
    mm.position.tile = pp.tile;
    mm.position.start = pp.start;
    mm.position.offset = pp.offset;
    mm.position.traveled = pp.traveled;
    mm._mirange = mm._miAnimLen;
}

/// Original: `devilution::AddWeaponExplosion` (missiles.cpp).
// @port missiles.cpp|devilution::AddWeaponExplosion(Missile &missile, AddMissileParameter &parameter) sha=5bfade68c97e
pub fn add_weapon_explosion(ctx: &mut Ctx, mi: usize, parameter: P) {
    mis(ctx, mi).var2 = parameter.dst.x;
    if parameter.dst.x == 1 {
        set_miss_anim(ctx, mi, MissileGraphicID::MagmaBallExplosion);
    } else {
        set_miss_anim(ctx, mi, MissileGraphicID::ChargedBolt);
    }
    let m = mis(ctx, mi);
    m._mirange = m._miAnimLen - 1;
}

// ---------------------------------------------------------------------------------------------
// missiles.cpp, part 3: the AddXxx initialisers (second half).
// ---------------------------------------------------------------------------------------------

fn is_my(ctx: &Ctx, pnum: usize) -> bool {
    Some(pnum) == ctx.players.MyPlayer
}

/// Original: `devilution::AddTownPortal` (missiles.cpp).
// @port missiles.cpp|devilution::AddTownPortal(Missile &missile, AddMissileParameter &parameter) sha=771ba22b5590
pub fn add_town_portal(ctx: &mut Ctx, mi: usize, parameter: P) {
    if ctx.gendung.leveltype == DungeonType::Town {
        let m = mis(ctx, mi);
        m.position.tile = parameter.dst;
        m.position.start = parameter.dst;
    } else {
        let target_position = crate::engine::path::find_closest_valid_position(
            ctx,
            &|ctx: &Ctx, target: Point| {
                if !in_dungeon_bounds(target) {
                    return false;
                }
                if crate::objects::is_object_at_position(ctx, target) {
                    return false;
                }
                if ctx.gendung.dPlayer[target.x as usize][target.y as usize] != 0 {
                    return false;
                }
                if crate::levels::gendung::tile_contains_missile(ctx, target) {
                    return false;
                }
                let dp = piece_at(ctx, target);
                if tile_has_any(ctx, dp, TileProperties::Solid | TileProperties::BlockMissile) {
                    return false;
                }
                !check_if_trig(ctx, target)
            },
            parameter.dst,
            0,
            5,
        );
        let m = mis(ctx, mi);
        match target_position {
            Some(t) => {
                m.position.tile = t;
                m.position.start = t;
                m._miDelFlag = false;
            }
            None => m._miDelFlag = true,
        }
    }
    {
        let m = mis(ctx, mi);
        m._mirange = 100;
        m.var1 = m._mirange - m._miAnimLen;
    }
    let me = ctx.missiles.Missiles[mi].clone();
    for o in 0..ctx.missiles.Missiles.len() {
        let other = &ctx.missiles.Missiles[o];
        if other._mitype == MissileID::TownPortal && o != mi && me.is_same_source(other) {
            ctx.missiles.Missiles[o]._mirange = 0;
        }
    }
    put_missile(ctx, mi);
    let m = ctx.missiles.Missiles[mi].clone();
    if m.source_player().is_some() && m.source_player() == ctx.players.MyPlayer && !m._miDelFlag && ctx.gendung.leveltype != DungeonType::Town {
        let lt = ctx.gendung.leveltype as i8 as u16;
        if !ctx.gendung.setlevel {
            let cl = ctx.gendung.currlevel as u16;
            crate::msg::net_send_cmd_loc_param3(ctx, true, CMD_ACTIVATEPORTAL, m.position.tile, cl, lt, 0);
        } else {
            let sl = ctx.gendung.setlvlnum as u16;
            crate::msg::net_send_cmd_loc_param3(ctx, true, CMD_ACTIVATEPORTAL, m.position.tile, sl, lt, 1);
        }
    }
}

/// Original: `devilution::AddFlashBottom` (missiles.cpp).
// @port missiles.cpp|devilution::AddFlashBottom(Missile &missile, AddMissileParameter &) sha=5bbc210b5ade
pub fn add_flash_bottom(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    let dam = match m.source_type() {
        MissileSource::Player => {
            let lvl = ctx.players.Players[m.source_player().unwrap()]._pLevel as i32;
            let dmg = generate_rnd_sum(ctx, 20, lvl + 1) + lvl + 1;
            let d = scale_spell_effect(dmg, m._mispllvl);
            d + d / 2
        }
        MissileSource::Monster => crate::monster::monster_level(ctx, m.source_monster().unwrap(), difficulty(ctx)) as i32 * 2,
        MissileSource::Trap => ctx.gendung.currlevel as i32 / 2,
    };
    let mm = mis(ctx, mi);
    mm._midam = dam;
    mm._mirange = 19;
}

/// Original: `devilution::AddFlashTop` (missiles.cpp).
// @port missiles.cpp|devilution::AddFlashTop(Missile &missile, AddMissileParameter &) sha=bba935b31fa6
pub fn add_flash_top(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    if m._micaster == TARGET_MONSTERS {
        if !m.is_trap() {
            let mut dmg = ctx.players.Players[m._misource as usize]._pLevel as i32 + 1;
            dmg += generate_rnd_sum(ctx, 20, dmg);
            let d = scale_spell_effect(dmg, m._mispllvl);
            mis(ctx, mi)._midam = d + d / 2;
        } else {
            mis(ctx, mi)._midam = ctx.gendung.currlevel as i32 / 2;
        }
    }
    let mm = mis(ctx, mi);
    mm._miPreFlag = true;
    mm._mirange = 19;
}

/// Original: `devilution::AddManaShield` (missiles.cpp).
// @port missiles.cpp|devilution::AddManaShield(Missile &missile, AddMissileParameter &parameter) sha=6286e55c8a41
pub fn add_mana_shield(ctx: &mut Ctx, mi: usize, parameter: P) {
    mis(ctx, mi)._miDelFlag = true;
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    if ctx.players.Players[p].pManaShield {
        parameter.spellFizzled = true;
        return;
    }
    ctx.players.Players[p].pManaShield = true;
    if is_my(ctx, p) {
        crate::msg::net_send_cmd(ctx, true, CMD_SETSHIELD);
    }
}

/// Original: `devilution::AddFlameWave` (missiles.cpp).
// @port missiles.cpp|devilution::AddFlameWave(Missile &missile, AddMissileParameter &parameter) sha=c49d3d51a1f3
pub fn add_flame_wave(ctx: &mut Ctx, mi: usize, parameter: P) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    let d = rnd(ctx, 10) + ctx.players.Players[p]._pLevel as i32 + 1;
    mis(ctx, mi)._midam = d;
    update_missile_velocity(ctx, mi, parameter.dst, 16);
    let m = mis(ctx, mi);
    m._mirange = 255;
    // Adjust missile's position for rendering
    m.position.tile += Direction::South;
    m.position.offset.delta_y -= 32;
}

/// Original: `devilution::AddGuardian` (missiles.cpp).
// @port missiles.cpp|devilution::AddGuardian(Missile &missile, AddMissileParameter &parameter) sha=323f4cc42338
pub fn add_guardian(ctx: &mut Ctx, mi: usize, parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    let start = m.position.start;
    let spawn_position = crate::engine::path::find_closest_valid_position(
        ctx,
        &|ctx: &Ctx, target: Point| {
            if !in_dungeon_bounds(target) {
                return false;
            }
            if ctx.gendung.dMonster[target.x as usize][target.y as usize] != 0 {
                return false;
            }
            if crate::objects::is_object_at_position(ctx, target) {
                return false;
            }
            if crate::levels::gendung::tile_contains_missile(ctx, target) {
                return false;
            }
            let dp = piece_at(ctx, target);
            if tile_has_any(ctx, dp, TileProperties::Solid | TileProperties::BlockMissile) {
                return false;
            }
            crate::monster::line_clear_missile(ctx, start, target)
        },
        parameter.dst,
        0,
        5,
    );
    let Some(sp) = spawn_position else {
        mis(ctx, mi)._miDelFlag = true;
        parameter.spellFizzled = true;
        return;
    };
    {
        let mm = mis(ctx, mi);
        mm._miDelFlag = false;
        mm.position.tile = sp;
        mm.position.start = sp;
    }
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, sp, 1);
    let plvl = ctx.players.Players[m._misource as usize]._pLevel as i32;
    let mm = mis(ctx, mi);
    mm._mirange = mm._mispllvl + (plvl / 2);
    if mm._mirange > 30 {
        mm._mirange = 30;
    }
    mm._mirange <<= 4;
    if mm._mirange < 30 {
        mm._mirange = 30;
    }
    mm.var1 = mm._mirange - mm._miAnimLen;
    mm.var3 = 1;
}

/// Original: `devilution::AddChainLightning` (missiles.cpp).
// @port missiles.cpp|devilution::AddChainLightning(Missile &missile, AddMissileParameter &parameter) sha=54b4504253e7
pub fn add_chain_lightning(ctx: &mut Ctx, mi: usize, parameter: P) {
    let m = mis(ctx, mi);
    m.var1 = parameter.dst.x;
    m.var2 = parameter.dst.y;
    m._mirange = 1;
}

/// Original: `InitMissileAnimationFromMonster` (missiles.cpp).
// @port missiles.cpp|devilution::InitMissileAnimationFromMonster(Missile &mis, Direction midir, const Monster &mon, MonsterGraphic graphic) sha=619a9d4fad2b
fn init_missile_animation_from_monster(ctx: &mut Ctx, mi: usize, midir: Direction, mon: usize, graphic: MonsterGraphic) {
    let anim = crate::monster::get_anim_data(ctx, mon, graphic).clone();
    let sprites = anim.sprites_for_direction(midir).expect("monster sprites");
    let width = sprites.get(0).width();
    let m = mis(ctx, mi);
    m._mimfnum = midir as i32;
    m._miAnimFlags = MissileGraphicsFlags::None;
    m._miAnimData = Some(sprites);
    m._miAnimDelay = anim.rate as i32;
    m._miAnimLen = anim.frames as i32;
    m._miAnimWidth = width;
    m._miAnimWidth2 = crate::engine::calculate_width2(width as i32) as i16;
    m._miAnimAdd = 1;
    m.var1 = 0;
    m.var2 = 0;
    m._miLightFlag = true;
    m._mirange = 256;
}

/// Original: `devilution::AddRhino` (missiles.cpp).
// @port missiles.cpp|devilution::AddRhino(Missile &missile, AddMissileParameter &parameter) sha=99238125ee26
pub fn add_rhino(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mon = ctx.missiles.Missiles[mi]._misource as usize;
    let t = monster_type_id(ctx, mon);
    let mut graphic = MonsterGraphic::Walk;
    if matches!(t, MT_HORNED | MT_MUDRUN | MT_FROSTC | MT_OBLORD) {
        graphic = MonsterGraphic::Special;
    } else if matches!(t, MT_NSNAKE | MT_RSNAKE | MT_BSNAKE | MT_GSNAKE) {
        graphic = MonsterGraphic::Attack;
    }
    update_missile_velocity(ctx, mi, parameter.dst, 18);
    if !ctx.diablo.headless_mode {
        init_missile_animation_from_monster(ctx, mi, parameter.midir, mon, graphic);
    } else {
        // headless runs have no sprites; keep the non-graphical part of the animation set-up
        let anim = crate::monster::get_anim_data(ctx, mon, graphic).clone();
        let m = mis(ctx, mi);
        m._mimfnum = parameter.midir as i32;
        m._miAnimFlags = MissileGraphicsFlags::None;
        m._miAnimDelay = anim.rate as i32;
        m._miAnimLen = anim.frames as i32;
        m._miAnimAdd = 1;
        m.var1 = 0;
        m.var2 = 0;
        m._miLightFlag = true;
        m._mirange = 256;
    }
    if matches!(t, MT_NSNAKE | MT_RSNAKE | MT_BSNAKE | MT_GSNAKE) {
        mis(ctx, mi)._miAnimFrame = 7;
    }
    if ctx.monster.Monsters[mon].is_unique() {
        mis(ctx, mi)._mlid = ctx.monster.Monsters[mon].lightId as i32;
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::AddGenericMagicMissile` (missiles.cpp).
// @port missiles.cpp|devilution::AddGenericMagicMissile(Missile &missile, AddMissileParameter &parameter) sha=fd0a0b595fd5
pub fn add_generic_magic_missile(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.start == dst {
        dst += parameter.midir;
    }
    update_missile_velocity(ctx, mi, dst, 16);
    {
        let mm = mis(ctx, mi);
        mm._mirange = 256;
        mm.var1 = m.position.start.x;
        mm.var2 = m.position.start.y;
    }
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.start, 8);
    if m._micaster != TARGET_MONSTERS && m._misource > 0 {
        let t = monster_type_id(ctx, m._misource as usize);
        if t == MT_SUCCUBUS {
            set_miss_anim(ctx, mi, MissileGraphicID::BloodStar);
        }
        if t == MT_SNOWWICH {
            set_miss_anim(ctx, mi, MissileGraphicID::BloodStarBlue);
        }
        if t == MT_HLSPWN {
            set_miss_anim(ctx, mi, MissileGraphicID::BloodStarRed);
        }
        if t == MT_SOLBRNR {
            set_miss_anim(ctx, mi, MissileGraphicID::BloodStarYellow);
        }
    }
    let anim_type = ctx.missiles.Missiles[mi]._miAnimType;
    if get_missile_sprite_data(ctx, anim_type).animFAmt == 16 {
        set_miss_dir(ctx, mi, get_direction16(m.position.start, dst) as i32);
    }
    set_projectile_damage(
        ctx,
        mi,
        Some(|ctx: &mut Ctx, mi: usize| {
            let m = &ctx.missiles.Missiles[mi];
            let mag = ctx.players.Players[m.source_player().unwrap()]._pMagic;
            3 * m._mispllvl - (mag / 8) + (mag / 2)
        }),
    );
}

/// Original: `devilution::AddAcid` (missiles.cpp).
// @port missiles.cpp|devilution::AddAcid(Missile &missile, AddMissileParameter &parameter) sha=18215281d4ac
pub fn add_acid(ctx: &mut Ctx, mi: usize, parameter: P) {
    update_missile_velocity(ctx, mi, parameter.dst, 16);
    let start = ctx.missiles.Missiles[mi].position.start;
    set_miss_dir(ctx, mi, get_direction16(start, parameter.dst) as i32);
    let hf = ctx.init.gb_is_hellfire;
    let m = ctx.missiles.Missiles[mi].clone();
    let range = if !hf || (m.position.velocity.delta_x as u32 & 0xFFFF0000) != 0 || (m.position.velocity.delta_y as u32 & 0xFFFF0000) != 0 {
        5 * (ctx.monster.Monsters[m._misource as usize].intelligence as i32 + 4)
    } else {
        1
    };
    let mm = mis(ctx, mi);
    mm._mirange = range;
    mm._mlid = crate::lighting::NO_LIGHT as i32;
    mm.var1 = start.x;
    mm.var2 = start.y;
    // Not typically created by Players
    set_projectile_damage(ctx, mi, None);
    put_missile(ctx, mi);
}

/// Original: `devilution::AddAcidPuddle` (missiles.cpp).
// @port missiles.cpp|devilution::AddAcidPuddle(Missile &missile, AddMissileParameter &) sha=cc3dae34ecdf
pub fn add_acid_puddle(ctx: &mut Ctx, mi: usize, _parameter: P) {
    mis(ctx, mi)._miLightFlag = true;
    let monst = ctx.missiles.Missiles[mi]._misource as usize;
    let r = rnd(ctx, 15) + 40 * (ctx.monster.Monsters[monst].intelligence as i32 + 1);
    let m = mis(ctx, mi);
    m._mirange = r;
    m._miPreFlag = true;
}

/// Original: `devilution::AddStoneCurse` (missiles.cpp).
// @port missiles.cpp|devilution::AddStoneCurse(Missile &missile, AddMissileParameter &parameter) sha=bfbc69663015
pub fn add_stone_curse(ctx: &mut Ctx, mi: usize, parameter: P) {
    let target = crate::engine::path::find_closest_valid_position(
        ctx,
        &|ctx: &Ctx, target: Point| {
            if !in_dungeon_bounds(target) {
                return false;
            }
            let monster_id = (ctx.gendung.dMonster[target.x as usize][target.y as usize] as i32).abs() - 1;
            if monster_id < 0 {
                return false;
            }
            let m = monster_id as usize;
            if matches!(monster_type_id(ctx, m), MT_GOLEM | MT_DIABLO | MT_NAKRUL) {
                return false;
            }
            if matches!(ctx.monster.Monsters[m].mode, MonsterMode::FadeIn | MonsterMode::FadeOut | MonsterMode::Charge) {
                return false;
            }
            true
        },
        parameter.dst,
        0,
        5,
    );
    let Some(tp) = target else {
        mis(ctx, mi)._miDelFlag = true;
        parameter.spellFizzled = true;
        return;
    };
    // Petrify the targeted monster
    let monster_id = ((ctx.gendung.dMonster[tp.x as usize][tp.y as usize] as i32).abs() - 1) as usize;
    if ctx.monster.Monsters[monster_id].mode == MonsterMode::Petrified {
        // Monster is already petrified and StoneCurse doesn't stack
        mis(ctx, mi)._miDelFlag = true;
        return;
    }
    let mode = ctx.monster.Monsters[monster_id].mode as i32;
    {
        let m = mis(ctx, mi);
        m.var1 = mode;
        m.var2 = monster_id as i32;
    }
    crate::monster::petrify(ctx, monster_id);
    // And set up the missile to unpetrify it in the future
    let m = mis(ctx, mi);
    m.position.tile = tp;
    m.position.start = tp;
    m._mirange = m._mispllvl + 6;
    if m._mirange > 15 {
        m._mirange = 15;
    }
    m._mirange <<= 4;
}

/// Original: `devilution::AddGolem` (missiles.cpp).
// @port missiles.cpp|devilution::AddGolem(Missile &missile, AddMissileParameter &parameter) sha=dfc71c0c3334
pub fn add_golem(ctx: &mut Ctx, mi: usize, parameter: P) {
    mis(ctx, mi)._miDelFlag = true;
    let player_id = ctx.missiles.Missiles[mi]._misource as usize;
    let golem = player_id;
    if ctx.monster.Monsters[golem].position.tile != crate::monster::GOLEM_HOLDING_CELL && is_my(ctx, player_id) {
        crate::monster::kill_my_golem(ctx);
    }
    if ctx.monster.Monsters[golem].position.tile == crate::monster::GOLEM_HOLDING_CELL {
        let start = ctx.missiles.Missiles[mi].position.start;
        let spawn_position = crate::engine::path::find_closest_valid_position(ctx, &|ctx: &Ctx, target: Point| !crate::engine::path::is_tile_occupied(ctx, target) && crate::monster::line_clear_missile(ctx, start, target), parameter.dst, 0, 5);
        if let Some(sp) = spawn_position {
            crate::monster::spawn_golem(ctx, player_id, golem, sp, mi);
        }
    }
}

/// Original: `devilution::AddApocalypseBoom` (missiles.cpp).
// @port missiles.cpp|devilution::AddApocalypseBoom(Missile &missile, AddMissileParameter &parameter) sha=1d0e52b4a8f3
pub fn add_apocalypse_boom(ctx: &mut Ctx, mi: usize, parameter: P) {
    let m = mis(ctx, mi);
    m.position.tile = parameter.dst;
    m.position.start = parameter.dst;
    m._mirange = m._miAnimLen;
}

/// Original: `devilution::AddHealing` (missiles.cpp).
// @port missiles.cpp|devilution::AddHealing(Missile &missile, AddMissileParameter &) sha=9190aa908e30
pub fn add_healing(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    let p = m._misource as usize;
    let lvl = ctx.players.Players[p]._pLevel as i32;
    let mut hp = rnd(ctx, 10) + 1;
    hp += generate_rnd_sum(ctx, 4, lvl) + lvl;
    hp += generate_rnd_sum(ctx, 6, m._mispllvl) + m._mispllvl;
    hp <<= 6;
    let player = &mut ctx.players.Players[p];
    if matches!(player._pClass, HeroClass::Warrior | HeroClass::Barbarian | HeroClass::Monk) {
        hp *= 2;
    } else if matches!(player._pClass, HeroClass::Rogue | HeroClass::Bard) {
        hp += hp / 2;
    }
    player._pHitPoints = (player._pHitPoints + hp).min(player._pMaxHP);
    player._pHPBase = (player._pHPBase + hp).min(player._pMaxHPBase);
    mis(ctx, mi)._miDelFlag = true;
    crate::engine::backbuffer_state::redraw_component(ctx, crate::engine::backbuffer_state::PanelDrawComponent::Health);
}

fn keyboard_and_mouse(ctx: &Ctx) -> bool {
    ctx.controls.control_mode == crate::controls::ControlTypes::KeyboardAndMouse
}

/// Original: `devilution::AddHealOther` (missiles.cpp).
// @port missiles.cpp|devilution::AddHealOther(Missile &missile, AddMissileParameter &) sha=350cb0190be1
pub fn add_heal_other(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    mis(ctx, mi)._miDelFlag = true;
    if is_my(ctx, p) {
        crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HEALOTHER);
        if !keyboard_and_mouse(ctx) {
            crate::diablo_game::try_icon_curs(ctx);
        }
    }
}

/// Original: `devilution::AddElemental` (missiles.cpp).
// @port missiles.cpp|devilution::AddElemental(Missile &missile, AddMissileParameter &parameter) sha=b35fdf436ad0
pub fn add_elemental(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.start == dst {
        dst += parameter.midir;
    }
    let lvl = ctx.players.Players[m._misource as usize]._pLevel as i32;
    let dmg = 2 * (lvl + generate_rnd_sum(ctx, 10, 2)) + 4;
    mis(ctx, mi)._midam = scale_spell_effect(dmg, m._mispllvl) / 2;
    update_missile_velocity(ctx, mi, dst, 16);
    set_miss_dir(ctx, mi, crate::engine::get_direction(m.position.start, dst) as i32);
    let mm = mis(ctx, mi);
    mm._mirange = 256;
    mm.var1 = m.position.start.x;
    mm.var2 = m.position.start.y;
    mm.var4 = dst.x;
    mm.var5 = dst.y;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.start, 8);
}

/// Shared body of `AddIdentify`, `AddItemRepair` and `AddStaffRecharge`.
fn open_inventory_with_cursor(ctx: &mut Ctx, mi: usize, cursor: i32) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    mis(ctx, mi)._miDelFlag = true;
    if is_my(ctx, p) {
        if ctx.control.sbookflag {
            ctx.control.sbookflag = false;
        }
        if !ctx.inv.invflag {
            ctx.inv.invflag = true;
            if !keyboard_and_mouse(ctx) {
                crate::controls::plrctrls::focus_on_inventory(ctx);
            }
        }
        crate::cursor::new_cursor(ctx, cursor);
    }
}

/// Original: `devilution::AddIdentify` (missiles.cpp).
// @port missiles.cpp|devilution::AddIdentify(Missile &missile, AddMissileParameter &) sha=f578197aba51
pub fn add_identify(ctx: &mut Ctx, mi: usize, _parameter: P) {
    open_inventory_with_cursor(ctx, mi, crate::cursor::CURSOR_IDENTIFY);
}

/// Original: `devilution::AddFireWallControl` (missiles.cpp).
// @port missiles.cpp|devilution::AddFireWallControl(Missile &missile, AddMissileParameter &parameter) sha=cfe4a5a3200d
pub fn add_fire_wall_control(ctx: &mut Ctx, mi: usize, parameter: P) {
    let start = ctx.missiles.Missiles[mi].position.start;
    let spread = crate::engine::path::find_closest_valid_position(
        ctx,
        &|ctx: &Ctx, target: Point| start != target && crate::engine::path::is_tile_not_solid(ctx, target) && !crate::objects::is_object_at_position(ctx, target) && crate::monster::line_clear_missile(ctx, start, target),
        parameter.dst,
        0,
        5,
    );
    let Some(sp) = spread else {
        mis(ctx, mi)._miDelFlag = true;
        parameter.spellFizzled = true;
        return;
    };
    use crate::engine::geometry::{left, right};
    let m = mis(ctx, mi);
    m._miDelFlag = false;
    m.var1 = sp.x;
    m.var2 = sp.y;
    m.var5 = sp.x;
    m.var6 = sp.y;
    m.var3 = left(left(parameter.midir)) as i32;
    m.var4 = right(right(parameter.midir)) as i32;
    m._mirange = 7;
}

/// Original: `devilution::AddInfravision` (missiles.cpp).
// @port missiles.cpp|devilution::AddInfravision(Missile &missile, AddMissileParameter &) sha=c42feb2c3417
pub fn add_infravision(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let m = mis(ctx, mi);
    m._mirange = scale_spell_effect(1584, m._mispllvl);
}

/// Original: `devilution::AddFlameWaveControl` (missiles.cpp).
// @port missiles.cpp|devilution::AddFlameWaveControl(Missile &missile, AddMissileParameter &parameter) sha=c8b01fa36940
pub fn add_flame_wave_control(ctx: &mut Ctx, mi: usize, parameter: P) {
    let m = mis(ctx, mi);
    m.var1 = parameter.dst.x;
    m.var2 = parameter.dst.y;
    m._mirange = 1;
    m._miAnimFrame = 4;
}

/// Original: `devilution::AddNova` (missiles.cpp).
// @port missiles.cpp|devilution::AddNova(Missile &missile, AddMissileParameter &parameter) sha=5936d1716ba3
pub fn add_nova(ctx: &mut Ctx, mi: usize, parameter: P) {
    {
        let m = mis(ctx, mi);
        m.var1 = parameter.dst.x;
        m.var2 = parameter.dst.y;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    let dam = if !m.is_trap() {
        let lvl = ctx.players.Players[m._misource as usize]._pLevel as i32;
        let dmg = generate_rnd_sum(ctx, 6, 5) + lvl + 5;
        scale_spell_effect(dmg / 2, m._mispllvl)
    } else {
        (ctx.gendung.currlevel as i32 / 2) + generate_rnd_sum(ctx, 3, 3)
    };
    let mm = mis(ctx, mi);
    mm._midam = dam;
    mm._mirange = 1;
}

/// Original: `devilution::AddRage` (missiles.cpp).
// @port missiles.cpp|devilution::AddRage(Missile &missile, AddMissileParameter &parameter) sha=2d8351b966d4
pub fn add_rage(ctx: &mut Ctx, mi: usize, parameter: P) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    let player = &ctx.players.Players[p];
    if player._pSpellFlags.has_any_of(SpellFlag::RageActive | SpellFlag::RageCooldown) || player._pHitPoints <= (player._pLevel as i32) << 6 {
        mis(ctx, mi)._miDelFlag = true;
        parameter.spellFizzled = true;
        return;
    }
    let lvl = player._pLevel as i32;
    let mut tmp = 3 * lvl;
    tmp <<= 7;
    ctx.players.Players[p]._pSpellFlags |= SpellFlag::RageActive;
    let m = mis(ctx, mi);
    m.var2 = tmp;
    m._mirange = lvl * 2 + 10 * m._mispllvl + 245;
    crate::items::calc_plr_item_vals(ctx, p, true);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    crate::player::player_say(ctx, p, HeroSpeech::Aaaaargh);
}

/// Original: `devilution::AddItemRepair` (missiles.cpp).
// @port missiles.cpp|devilution::AddItemRepair(Missile &missile, AddMissileParameter &) sha=855c8bbf035e
pub fn add_item_repair(ctx: &mut Ctx, mi: usize, _parameter: P) {
    open_inventory_with_cursor(ctx, mi, crate::cursor::CURSOR_REPAIR);
}

/// Original: `devilution::AddStaffRecharge` (missiles.cpp).
// @port missiles.cpp|devilution::AddStaffRecharge(Missile &missile, AddMissileParameter &) sha=b86f061ce662
pub fn add_staff_recharge(ctx: &mut Ctx, mi: usize, _parameter: P) {
    open_inventory_with_cursor(ctx, mi, crate::cursor::CURSOR_RECHARGE);
}

/// Original: `devilution::AddTrapDisarm` (missiles.cpp).
// @port missiles.cpp|devilution::AddTrapDisarm(Missile &missile, AddMissileParameter &) sha=957ed6a1b336
pub fn add_trap_disarm(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    mis(ctx, mi)._miDelFlag = true;
    if is_my(ctx, p) {
        crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_DISARM);
        if !keyboard_and_mouse(ctx) {
            if ctx.cursor.ObjectUnderCursor.is_some() {
                let (id, cp) = (ctx.players.MyPlayerId, ctx.cursor.cursPosition);
                crate::msg::net_send_cmd_loc(ctx, id, true, CMD_DISARMXY, cp);
            } else {
                crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HAND);
            }
        }
    }
}

/// Original: `devilution::AddApocalypse` (missiles.cpp).
// @port missiles.cpp|devilution::AddApocalypse(Missile &missile, AddMissileParameter &) sha=3ca93c0bbc6b
pub fn add_apocalypse(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    {
        let m = mis(ctx, mi);
        let s = m.position.start;
        m.var1 = 8;
        m.var2 = (s.y - 8).max(1);
        m.var3 = (s.y + 8).min(MAXDUNY as i32 - 1);
        m.var4 = (s.x - 8).max(1);
        m.var5 = (s.x + 8).min(MAXDUNX as i32 - 1);
        m.var6 = m.var4;
    }
    let player_level = ctx.players.Players[p]._pLevel as i32;
    let dam = generate_rnd_sum(ctx, 6, player_level) + player_level;
    let m = mis(ctx, mi);
    m._midam = dam;
    m._mirange = 255;
}

/// Original: `devilution::AddInferno` (missiles.cpp).
// @port missiles.cpp|devilution::AddInferno(Missile &missile, AddMissileParameter &parameter) sha=9baa823acb57
pub fn add_inferno(ctx: &mut Ctx, mi: usize, parameter: P) {
    {
        let m = mis(ctx, mi);
        m.var2 = 5 * m._midam;
        m.position.start = parameter.dst;
    }
    sync_position_with_parent(ctx, mi, parameter);
    {
        let m = mis(ctx, mi);
        m._mirange = m.var2 + 20;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.start, 1);
    let dam = if m._micaster == TARGET_MONSTERS {
        let lvl = ctx.players.Players[m._misource as usize]._pLevel as i32;
        let i = rnd(ctx, lvl) + rnd(ctx, 2);
        8 * i + 16 + ((8 * i + 16) / 2)
    } else {
        let mon = &ctx.monster.Monsters[m._misource as usize];
        let (min, max) = (mon.minDamage as i32, mon.maxDamage as i32);
        min + rnd(ctx, max - min + 1)
    };
    mis(ctx, mi)._midam = dam;
}

/// Original: `devilution::AddInfernoControl` (missiles.cpp).
// @port missiles.cpp|devilution::AddInfernoControl(Missile &missile, AddMissileParameter &parameter) sha=9903da8d708d
pub fn add_inferno_control(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let start = ctx.missiles.Missiles[mi].position.start;
    if start == parameter.dst {
        dst += parameter.midir;
    }
    update_missile_velocity(ctx, mi, dst, 32);
    let m = mis(ctx, mi);
    m.var1 = start.x;
    m.var2 = start.y;
    m._mirange = 256;
}

/// Original: `devilution::AddChargedBolt` (missiles.cpp).
// @port missiles.cpp|devilution::AddChargedBolt(Missile &missile, AddMissileParameter &parameter) sha=1b4025a2c348
pub fn add_charged_bolt(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    mis(ctx, mi)._mirnd = rnd(ctx, 15) + 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let dam = if m._micaster == TARGET_MONSTERS { rnd(ctx, ctx.players.Players[m._misource as usize]._pMagic / 4) + 1 } else { 15 };
    mis(ctx, mi)._midam = dam;
    if m.position.start == dst {
        dst += parameter.midir;
    }
    mis(ctx, mi)._miAnimFrame = rnd(ctx, 8) + 1;
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.start, 5);
    update_missile_velocity(ctx, mi, dst, 8);
    let mm = mis(ctx, mi);
    mm.var1 = 5;
    mm.var2 = parameter.midir as i32;
    mm._mirange = 256;
}

/// Original: `devilution::AddHolyBolt` (missiles.cpp).
// @port missiles.cpp|devilution::AddHolyBolt(Missile &missile, AddMissileParameter &parameter) sha=ee52bc500237
pub fn add_holy_bolt(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.start == dst {
        dst += parameter.midir;
    }
    let mut sp = 16;
    if !m.is_trap() {
        sp += (m._mispllvl * 2).min(47);
    }
    let lvl = ctx.players.Players[m._misource as usize]._pLevel as i32;
    update_missile_velocity(ctx, mi, dst, sp);
    set_miss_dir(ctx, mi, get_direction16(m.position.start, dst) as i32);
    {
        let mm = mis(ctx, mi);
        mm._mirange = 256;
        mm.var1 = m.position.start.x;
        mm.var2 = m.position.start.y;
    }
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.start, 8);
    mis(ctx, mi)._midam = rnd(ctx, 10) + lvl + 9;
}

/// Original: `devilution::AddResurrect` (missiles.cpp).
// @port missiles.cpp|devilution::AddResurrect(Missile &missile, AddMissileParameter &) sha=f4aa02ca4b65
pub fn add_resurrect(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    if is_my(ctx, p) {
        crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_RESURRECT);
        if !keyboard_and_mouse(ctx) {
            crate::diablo_game::try_icon_curs(ctx);
        }
    }
    mis(ctx, mi)._miDelFlag = true;
}

/// Original: `devilution::AddResurrectBeam` (missiles.cpp).
// @port missiles.cpp|devilution::AddResurrectBeam(Missile &missile, AddMissileParameter &parameter) sha=2911df54ca37
pub fn add_resurrect_beam(ctx: &mut Ctx, mi: usize, parameter: P) {
    let len = get_missile_sprite_data(ctx, MissileGraphicID::Resurrect).anim_len(0) as i32;
    let m = mis(ctx, mi);
    m.position.tile = parameter.dst;
    m.position.start = parameter.dst;
    m._mirange = len;
}

/// Original: `devilution::AddTelekinesis` (missiles.cpp).
// @port missiles.cpp|devilution::AddTelekinesis(Missile &missile, AddMissileParameter &) sha=b51571b8bc30
pub fn add_telekinesis(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    mis(ctx, mi)._miDelFlag = true;
    if is_my(ctx, p) {
        crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_TELEKINESIS);
    }
}

/// Original: `devilution::AddBoneSpirit` (missiles.cpp).
// @port missiles.cpp|devilution::AddBoneSpirit(Missile &missile, AddMissileParameter &parameter) sha=445d3d640c5c
pub fn add_bone_spirit(ctx: &mut Ctx, mi: usize, parameter: P) {
    let mut dst = parameter.dst;
    let start = ctx.missiles.Missiles[mi].position.start;
    if start == dst {
        dst += parameter.midir;
    }
    update_missile_velocity(ctx, mi, dst, 16);
    set_miss_dir(ctx, mi, crate::engine::get_direction(start, dst) as i32);
    {
        let m = mis(ctx, mi);
        m._mirange = 256;
        m.var1 = start.x;
        m.var2 = start.y;
        m.var4 = dst.x;
        m.var5 = dst.y;
    }
    mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, start, 8);
}

/// Original: `devilution::AddRedPortal` (missiles.cpp).
// @port missiles.cpp|devilution::AddRedPortal(Missile &missile, AddMissileParameter &) sha=94756a7979df
pub fn add_red_portal(ctx: &mut Ctx, mi: usize, _parameter: P) {
    {
        let m = mis(ctx, mi);
        m._mirange = 100;
        m.var1 = 100 - m._miAnimLen;
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::AddDiabloApocalypse` (missiles.cpp).
// @port missiles.cpp|devilution::AddDiabloApocalypse(Missile &missile, AddMissileParameter &) sha=683407ba8b46
pub fn add_diablo_apocalypse(ctx: &mut Ctx, mi: usize, _parameter: P) {
    let m = ctx.missiles.Missiles[mi].clone();
    for p in 0..ctx.players.Players.len() {
        let player = &ctx.players.Players[p];
        if !player.plractive {
            continue;
        }
        let future = player.position.future;
        if !crate::monster::line_clear_missile(ctx, m.position.start, future) {
            continue;
        }
        add_missile(ctx, Point::new(0, 0), future, Direction::South, MissileID::DiabloApocalypseBoom, m._micaster, m._misource, m._midam, 0, None);
    }
    mis(ctx, mi)._miDelFlag = true;
}

// ---------------------------------------------------------------------------------------------
// missiles.cpp, part 4: AddMissile and the first ProcessXxx handlers.
// ---------------------------------------------------------------------------------------------

fn change_light(ctx: &mut Ctx, l: i32, position: Point, radius: i32) {
    crate::lighting::change_light(ctx, l, position, radius as u8);
}

fn unlight(ctx: &mut Ctx, mi: usize) {
    let l = ctx.missiles.Missiles[mi]._mlid;
    crate::lighting::add_un_light(ctx, l);
}

/// Original: `devilution::AddMissile` (missiles.cpp).
// @port missiles.cpp|devilution::AddMissile(Point src, Point dst, Direction midir, MissileID mitype, mienemy_type micaster, int id, int midam, int spllvl, Missile *parent, std::optional<_sfx_id> lSFX) sha=47b7b237fcbd
#[allow(clippy::too_many_arguments)]
pub fn add_missile_sfx(ctx: &mut Ctx, src: Point, dst: Point, midir: Direction, mitype: MissileID, micaster: mienemy_type, id: i32, midam: i32, spllvl: i32, parent: Option<usize>, l_sfx: Option<crate::effects_data::SfxId>) -> Option<usize> {
    let mi = ctx.missiles.Missiles.len();
    ctx.missiles.Missiles.push(Missile::default());
    let missile_data = get_missile_data(mitype);
    {
        let missile = mis(ctx, mi);
        missile._mitype = mitype;
        missile._micaster = micaster;
        missile._misource = id;
        missile._midam = midam;
        missile._mispllvl = spllvl;
        missile.position.tile = src;
        missile.position.start = src;
        missile._miAnimAdd = 1;
        missile._miAnimType = missile_data.mFileNum;
        missile._miDrawFlag = missile_data.is_drawn();
        missile._mlid = crate::lighting::NO_LIGHT as i32;
        missile.lastCollisionTargetHash = 0;
    }
    if !ctx.missiles.Missiles[mi].is_trap() && micaster == TARGET_PLAYERS {
        let monster = &ctx.monster.Monsters[id as usize];
        if monster.is_unique() {
            let t = monster.uniqTrans as u32 + 1;
            mis(ctx, mi)._miUniqTrans = t;
        }
    }
    let anim_type = ctx.missiles.Missiles[mi]._miAnimType;
    if anim_type == MissileGraphicID::None || get_missile_sprite_data(ctx, anim_type).animFAmt < 8 {
        set_miss_dir(ctx, mi, 0);
    } else {
        set_miss_dir(ctx, mi, midir as i32);
    }
    let l_sfx = l_sfx.unwrap_or(missile_data.mlSFX);
    if l_sfx != crate::effects_data::SFX_NONE {
        crate::effects::play_sfx_loc(ctx, l_sfx, src, true);
    }
    let mut parameter = AddMissileParameter { dst, midir, pParent: parent, spellFizzled: false };
    (missile_data.mAddProc.expect("mAddProc"))(ctx, mi, &mut parameter);
    if parameter.spellFizzled {
        return None;
    }
    Some(mi)
}

/// Original: `devilution::ProcessElementalArrow` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessElementalArrow(Missile &missile) sha=af769ca545d1
pub fn process_elemental_arrow(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let currlevel = ctx.gendung.currlevel as i32;
    if m._miAnimType == MissileGraphicID::ChargedBolt || m._miAnimType == MissileGraphicID::MagmaBallExplosion {
        change_light(ctx, m._mlid, m.position.tile, m._miAnimFrame + 5);
    } else {
        let p = m._misource;
        mis(ctx, mi)._midist += 1;
        let (mind, maxd);
        if !m.is_trap() {
            if m._micaster == TARGET_MONSTERS {
                // BUGFIX: damage of missile should be encoded in missile struct; player can be dead/have left the game before missile arrives.
                let player = &ctx.players.Players[p as usize];
                mind = player._pIMinDam;
                maxd = player._pIMaxDam;
            } else {
                // BUGFIX: damage of missile should be encoded in missile struct; monster can be dead before missile arrives.
                let monster = &ctx.monster.Monsters[p as usize];
                mind = monster.minDamage as i32;
                maxd = monster.maxDamage as i32;
            }
        } else {
            mind = rnd(ctx, 10) + 1 + currlevel;
            maxd = rnd(ctx, 10) + 1 + currlevel * 2;
        }
        move_missile_and_check_missile_col(ctx, mi, DamageType::Physical, mind, maxd, true, false);
        if ctx.missiles.Missiles[mi]._mirange == 0 {
            {
                let mm = mis(ctx, mi);
                mm._mimfnum = 0;
                mm._mirange = mm._miAnimLen - 1;
                mm.position.stop_missile();
            }
            let (e_mind, e_maxd, e_anim, damage_type);
            match m._mitype {
                MissileID::LightningArrow => {
                    if !m.is_trap() {
                        // BUGFIX: damage of missile should be encoded in missile struct; player can be dead/have left the game before missile arrives.
                        let player = &ctx.players.Players[p as usize];
                        e_mind = player._pILMinDam;
                        e_maxd = player._pILMaxDam;
                    } else {
                        e_mind = rnd(ctx, 10) + 1 + currlevel;
                        e_maxd = rnd(ctx, 10) + 1 + currlevel * 2;
                    }
                    e_anim = MissileGraphicID::ChargedBolt;
                    damage_type = DamageType::Lightning;
                }
                MissileID::FireArrow => {
                    if !m.is_trap() {
                        // BUGFIX: damage of missile should be encoded in missile struct; player can be dead/have left the game before missile arrives.
                        let player = &ctx.players.Players[p as usize];
                        e_mind = player._pIFMinDam;
                        e_maxd = player._pIFMaxDam;
                    } else {
                        e_mind = rnd(ctx, 10) + 1 + currlevel;
                        e_maxd = rnd(ctx, 10) + 1 + currlevel * 2;
                    }
                    e_anim = MissileGraphicID::MagmaBallExplosion;
                    damage_type = DamageType::Fire;
                }
                t => crate::appfat::app_fatal(ctx, &format!("wrong missile ID {}", t as i8)),
            }
            set_miss_anim(ctx, mi, e_anim);
            let t = ctx.missiles.Missiles[mi].position.tile;
            check_missile_col(ctx, mi, damage_type, e_mind, e_maxd, false, t, true);
        } else {
            let mm = ctx.missiles.Missiles[mi].clone();
            if mm.position.tile != Point::new(mm.var1, mm.var2) {
                let x = mis(ctx, mi);
                x.var1 = mm.position.tile.x;
                x.var2 = mm.position.tile.y;
                change_light(ctx, mm._mlid, mm.position.tile, 5);
            }
        }
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessArrow` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessArrow(Missile &missile) sha=d58998aad449
pub fn process_arrow(ctx: &mut Ctx, mi: usize) {
    {
        let m = mis(ctx, mi);
        m._mirange -= 1;
        m._midist += 1;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    let currlevel = ctx.gendung.currlevel as i32;
    let (mind, maxd) = match m.source_type() {
        // BUGFIX: damage of missile should be encoded in missile struct; player can be dead/have left the game before missile arrives.
        MissileSource::Player => {
            let player = &ctx.players.Players[m.source_player().unwrap()];
            (player._pIMinDam, player._pIMaxDam)
        }
        // BUGFIX: damage of missile should be encoded in missile struct; monster can be dead before missile arrives.
        MissileSource::Monster => {
            let monster = &ctx.monster.Monsters[m.source_monster().unwrap()];
            (monster.minDamage as i32, monster.maxDamage as i32)
        }
        MissileSource::Trap => (currlevel, 2 * currlevel),
    };
    move_missile_and_check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), mind, maxd, true, false);
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessGenericProjectile` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessGenericProjectile(Missile &missile) sha=beb2720f19c9
pub fn process_generic_projectile(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    move_missile_and_check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, true, true);
    let m = ctx.missiles.Missiles[mi].clone();
    if m._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        let dst = Point::new(0, 0);
        let dir = Direction::from_u8(m._mimfnum as u8);
        let explosion = match m._mitype {
            MissileID::Firebolt | MissileID::MagmaBall => Some(MissileID::MagmaBallExplosion),
            MissileID::BloodStar => Some(MissileID::BloodStarExplosion),
            MissileID::Acid => Some(MissileID::AcidSplat),
            MissileID::OrangeFlare => Some(MissileID::OrangeExplosion),
            MissileID::BlueFlare => Some(MissileID::BlueExplosion),
            MissileID::RedFlare => Some(MissileID::RedExplosion),
            MissileID::YellowFlare => Some(MissileID::YellowExplosion),
            MissileID::BlueFlare2 => Some(MissileID::BlueExplosion2),
            _ => None,
        };
        if let Some(e) = explosion {
            add_missile(ctx, m.position.tile, dst, dir, e, m._micaster, m._misource, 0, 0, Some(mi));
        }
        if m._mlid != crate::lighting::NO_LIGHT as i32 {
            crate::lighting::add_un_light(ctx, m._mlid);
        }
        put_missile(ctx, mi);
    } else {
        if m.position.tile != Point::new(m.var1, m.var2) {
            let mm = mis(ctx, mi);
            mm.var1 = m.position.tile.x;
            mm.var2 = m.position.tile.y;
            if m._mlid != crate::lighting::NO_LIGHT as i32 {
                change_light(ctx, m._mlid, m.position.tile, 8);
            }
        }
        put_missile(ctx, mi);
    }
}

/// Original: `devilution::ProcessNovaBall` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessNovaBall(Missile &missile) sha=014c75ff499a
pub fn process_nova_ball(ctx: &mut Ctx, mi: usize) {
    let m = ctx.missiles.Missiles[mi].clone();
    let target_position = Point::new(m.var1, m.var2);
    mis(ctx, mi)._mirange -= 1;
    let j = ctx.missiles.Missiles[mi]._mirange;
    move_missile_and_check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, false, false);
    if ctx.missiles.Missiles[mi]._miHitFlag {
        mis(ctx, mi)._mirange = j;
    }
    if ctx.missiles.Missiles[mi].position.tile == target_position {
        if let Some(o) = crate::objects::find_object_at_position(ctx, target_position, true) {
            if ctx.objects.Objects[o].is_shrine() {
                mis(ctx, mi)._mirange = j;
            }
        }
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessAcidPuddle` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessAcidPuddle(Missile &missile) sha=93007f9c4865
pub fn process_acid_puddle(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let range = m._mirange;
    check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, true, m.position.tile, false);
    mis(ctx, mi)._mirange = range;
    if range == 0 {
        if ctx.missiles.Missiles[mi]._mimfnum != 0 {
            mis(ctx, mi)._miDelFlag = true;
        } else {
            set_miss_dir(ctx, mi, 1);
            let mm = mis(ctx, mi);
            mm._mirange = mm._miAnimLen;
        }
    }
    put_missile(ctx, mi);
}

/// `ExpLight` of `ProcessFireWall` and `ProcessFlameWave` (13 initialisers, 14 elements).
const EXP_LIGHT: [i32; 14] = [2, 3, 4, 5, 5, 6, 7, 8, 9, 10, 11, 12, 12, 0];

/// Original: `devilution::ProcessFireWall` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessFireWall(Missile &missile) sha=d82d6acfd8cc
pub fn process_fire_wall(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    if m._mirange == m.var1 {
        set_miss_dir(ctx, mi, 1);
        mis(ctx, mi)._miAnimFrame = rnd(ctx, 11) + 1;
    }
    if m._mirange == ctx.missiles.Missiles[mi]._miAnimLen - 1 {
        set_miss_dir(ctx, mi, 0);
        let mm = mis(ctx, mi);
        mm._miAnimFrame = 13;
        mm._miAnimAdd = -1;
    }
    check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, true, m.position.tile, true);
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if m._mimfnum != 0 && m._mirange != 0 && m._miAnimAdd != -1 && m.var2 < 12 {
        if m.var2 == 0 {
            mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.tile, EXP_LIGHT[0] as u8);
        }
        let l = ctx.missiles.Missiles[mi]._mlid;
        change_light(ctx, l, m.position.tile, EXP_LIGHT[m.var2 as usize]);
        mis(ctx, mi).var2 += 1;
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessFireball` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessFireball(Missile &missile) sha=035c3bbf3821
pub fn process_fireball(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    if m._miAnimType == MissileGraphicID::BigExplosion {
        if m._mirange == 0 {
            mis(ctx, mi)._miDelFlag = true;
            unlight(ctx, mi);
        }
    } else {
        let (mut min_dam, mut max_dam) = (m._midam, m._midam);
        if m._micaster != TARGET_MONSTERS {
            let monster = &ctx.monster.Monsters[m._misource as usize];
            min_dam = monster.minDamage as i32;
            max_dam = monster.maxDamage as i32;
        }
        let damage_type = get_missile_data(m._mitype).damage_type();
        move_missile_and_check_missile_col(ctx, mi, damage_type, min_dam, max_dam, true, false);
        let m = ctx.missiles.Missiles[mi].clone();
        if m._mirange == 0 {
            let missile_position = m.position.tile;
            change_light(ctx, m._mlid, m.position.tile, m._miAnimFrame);
            const OFFSETS: [Direction; 9] = [
                Direction::NoDirection,
                Direction::SouthWest,
                Direction::NorthEast,
                Direction::SouthEast,
                Direction::East,
                Direction::South,
                Direction::NorthWest,
                Direction::West,
                Direction::North,
            ];
            for offset in OFFSETS {
                if !check_block(ctx, m.position.start, missile_position + offset) {
                    check_missile_col(ctx, mi, damage_type, min_dam, max_dam, false, missile_position + offset, true);
                }
            }
            let g = &ctx.gendung;
            let tl = |x: i32, y: i32| g.TransList[g.dTransVal[x as usize][y as usize] as u8 as usize];
            let solid = |x: i32, y: i32| tile_has_any(ctx, g.dPiece[x as usize][y as usize] as i32, TileProperties::Solid);
            let (x, y) = (missile_position.x, missile_position.y);
            let vel = ctx.missiles.Missiles[mi].position.velocity;
            let shift_tile = !tl(x, y) || (vel.delta_x < 0 && ((tl(x, y + 1) && solid(x, y + 1)) || (tl(x, y - 1) && solid(x, y - 1))));
            let shift_y = vel.delta_y > 0 && ((tl(x + 1, y) && solid(x + 1, y)) || (tl(x - 1, y) && solid(x - 1, y)));
            let shift_x = vel.delta_x > 0 && ((tl(x, y + 1) && solid(x, y + 1)) || (tl(x, y - 1) && solid(x, y - 1)));
            {
                let mm = mis(ctx, mi);
                if shift_tile {
                    mm.position.tile += Displacement::new(1, 1);
                    mm.position.offset.delta_y -= 32;
                }
                if shift_y {
                    mm.position.offset.delta_y -= 32;
                }
                if shift_x {
                    mm.position.offset.delta_x -= 32;
                }
                mm._mimfnum = 0;
            }
            set_miss_anim(ctx, mi, MissileGraphicID::BigExplosion);
            let mm = mis(ctx, mi);
            mm._mirange = mm._miAnimLen - 1;
            mm.position.velocity = Displacement::default();
        } else if m.position.tile != Point::new(m.var1, m.var2) {
            let mm = mis(ctx, mi);
            mm.var1 = m.position.tile.x;
            mm.var2 = m.position.tile.y;
            change_light(ctx, m._mlid, m.position.tile, 8);
        }
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessHorkSpawn` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessHorkSpawn(Missile &missile) sha=307d41b5b3cf
pub fn process_hork_spawn(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), 0, 0, false, m.position.tile, false);
    if ctx.missiles.Missiles[mi]._mirange <= 0 {
        mis(ctx, mi)._miDelFlag = true;
        let t = ctx.missiles.Missiles[mi].position.tile;
        let spawn_position = crate::engine::path::find_closest_valid_position(ctx, &|ctx: &Ctx, target: Point| !crate::engine::path::is_tile_occupied(ctx, target), t, 0, 1);
        if let Some(sp) = spawn_position {
            let facing = Direction::from_u8(m.var1 as u8);
            if let Some(monster) = crate::monster::add_monster(ctx, sp, facing, 1, true) {
                crate::monster::m_start_stand(ctx, monster, facing);
            }
        }
    } else {
        {
            let mm = mis(ctx, mi);
            mm._midist += 1;
            mm.position.traveled += mm.position.velocity;
        }
        update_missile_pos(ctx, mi);
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessRune` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessRune(Missile &missile) sha=333dff0bbb51
pub fn process_rune(ctx: &mut Ctx, mi: usize) {
    let m = ctx.missiles.Missiles[mi].clone();
    let position = m.position.tile;
    let mid = ctx.gendung.dMonster[position.x as usize][position.y as usize] as i32;
    let pid = ctx.gendung.dPlayer[position.x as usize][position.y as usize] as i32;
    if mid != 0 || pid != 0 {
        let target_position = if mid != 0 { ctx.monster.Monsters[(mid.abs() - 1) as usize].position.tile } else { ctx.players.Players[(pid.abs() - 1) as usize].position.tile };
        let dir = crate::engine::get_direction(position, target_position);
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
        let t = MissileID::from_repr(m.var1 as i8).unwrap_or(MissileID::Null);
        add_missile(ctx, position, position, dir, t, TARGET_BOTH, m._misource, m._midam, m._mispllvl, None);
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessLightningWall` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessLightningWall(Missile &missile) sha=bf4825d0b39d
pub fn process_lightning_wall(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let range = m._mirange;
    check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, true, m.position.tile, false);
    if ctx.missiles.Missiles[mi]._miHitFlag {
        mis(ctx, mi)._mirange = range;
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessBigExplosion` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessBigExplosion(Missile &missile) sha=1616d9fe7bf2
pub fn process_big_explosion(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange <= 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessLightningBow` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessLightningBow(Missile &missile) sha=aecdba4ec449
pub fn process_lightning_bow(ctx: &mut Ctx, mi: usize) {
    let d = ctx.missiles.Missiles[mi]._midam;
    spawn_lightning(ctx, mi, d);
}

/// Original: `devilution::ProcessRingOfFire` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessRingOfFire(Missile &missile) sha=69752cc00356
pub fn process_ring_of_fire(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._miDelFlag = true;
    let m = ctx.missiles.Missiles[mi].clone();
    let src = m._misource as i8;
    let lvl = (if m._micaster == TARGET_MONSTERS { ctx.players.Players[src as usize]._pLevel as u8 } else { ctx.gendung.currlevel }) as i32;
    let dmg = 16 * (generate_rnd_sum(ctx, 10, 2) + lvl + 2) / 2;
    if m.limitReached {
        return;
    }
    let center = Point::new(m.var1, m.var2);
    crate::lighting::do_crawl(3, &mut |displacement| {
        let target = center + displacement;
        if !in_dungeon_bounds(target) {
            return false;
        }
        let dp = piece_at(ctx, target);
        if tile_has_any(ctx, dp, TileProperties::Solid) {
            return false;
        }
        if crate::objects::is_object_at_position(ctx, target) {
            return false;
        }
        if !crate::monster::line_clear_missile(ctx, m.position.tile, target) {
            return false;
        }
        if tile_has_any(ctx, dp, TileProperties::BlockMissile) {
            mis(ctx, mi).limitReached = true;
            return true;
        }
        add_missile(ctx, target, target, Direction::South, MissileID::FireWall, TARGET_BOTH, src as i32, dmg, m._mispllvl, None);
        false
    });
}

/// Original: `devilution::ProcessSearch` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessSearch(Missile &missile) sha=497e546e610a
pub fn process_search(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange != 0 {
        return;
    }
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    mis(ctx, mi)._miDelFlag = true;
    let t = ctx.players.Players[p].position.tile;
    crate::effects::play_sfx_loc(ctx, crate::effects_data::IS_CAST7, t, true);
    if is_my(ctx, p) {
        ctx.scrollrt.AutoMapShowItems = false;
    }
}

/// Original: `devilution::ProcessLightningWallControl` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessLightningWallControl(Missile &missile) sha=5355fb2a0261
pub fn process_lightning_wall_control(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        return;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    let id = m._misource;
    let lvl = if !m.is_trap() { ctx.players.Players[id as usize]._pLevel as i32 } else { 0 };
    let dmg = 16 * (generate_rnd_sum(ctx, 10, 2) + lvl + 2);
    {
        let position = Point::new(m.var1, m.var2);
        let target = position + Direction::from_u8(m.var3 as u8);
        if !m.limitReached && grow_wall(ctx, id, position, target, MissileID::LightningWall, m._mispllvl, dmg) {
            let mm = mis(ctx, mi);
            mm.var1 = target.x;
            mm.var2 = target.y;
        } else {
            mis(ctx, mi).limitReached = true;
        }
    }
    {
        let position = Point::new(m.var5, m.var6);
        let target = position + Direction::from_u8(m.var4 as u8);
        if m.var7 == 0 && grow_wall(ctx, id, position, target, MissileID::LightningWall, m._mispllvl, dmg) {
            let mm = mis(ctx, mi);
            mm.var5 = target.x;
            mm.var6 = target.y;
        } else {
            mis(ctx, mi).var7 = 1;
        }
    }
}

/// Original: `ProcessNovaCommon` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessNovaCommon(Missile &missile, MissileID projectileType) sha=34ab394d0ce3
fn process_nova_common(ctx: &mut Ctx, mi: usize, projectile_type: MissileID) {
    let m = ctx.missiles.Missiles[mi].clone();
    let id = m._misource;
    let dam = m._midam;
    let src = m.position.tile;
    let mut dir = Direction::South;
    let mut en = TARGET_PLAYERS;
    if !m.is_trap() {
        dir = ctx.players.Players[id as usize]._pdir;
        en = TARGET_MONSTERS;
    }
    const QUARTER_RADIUS: [(i32, i32); 9] = [(4, 0), (4, 1), (4, 2), (4, 3), (4, 4), (3, 4), (2, 4), (1, 4), (0, 4)];
    for (qx, qy) in QUARTER_RADIUS {
        // This ends up with two missiles targeting offsets 4,0, 0,4, -4,0, 0,-4.
        let q = Displacement::new(qx, qy);
        for offset in [q, q.flip_xy(), q.flip_x(), q.flip_y()] {
            add_missile(ctx, src, src + offset, dir, projectile_type, en, id, dam, m._mispllvl, None);
        }
    }
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
    }
}

/// Original: `devilution::ProcessImmolation` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessImmolation(Missile &missile) sha=7a013effad8a
pub fn process_immolation(ctx: &mut Ctx, mi: usize) {
    process_nova_common(ctx, mi, MissileID::FireballBow);
}

/// Original: `devilution::ProcessNova` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessNova(Missile &missile) sha=942d55394ba3
pub fn process_nova(ctx: &mut Ctx, mi: usize) {
    process_nova_common(ctx, mi, MissileID::NovaBall);
}

/// Original: `devilution::ProcessSpectralArrow` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessSpectralArrow(Missile &missile) sha=761d9febdf8b
pub fn process_spectral_arrow(ctx: &mut Ctx, mi: usize) {
    let m = ctx.missiles.Missiles[mi].clone();
    let id = m._misource;
    let dam = m._midam;
    let src = m.position.tile;
    let dst = Point::new(m.var1, m.var2);
    let spllvl = m.var3;
    let mut mitype = MissileID::Arrow;
    let mut dir = Direction::South;
    let mut micaster = TARGET_PLAYERS;
    if !m.is_trap() {
        let player = &ctx.players.Players[id as usize];
        dir = player._pdir;
        micaster = TARGET_MONSTERS;
        match player._pILMinDam {
            0 => mitype = MissileID::FireballBow,
            1 => mitype = MissileID::LightningBow,
            2 => mitype = MissileID::ChargedBoltBow,
            3 => mitype = MissileID::HolyBoltBow,
            _ => {}
        }
    }
    add_missile(ctx, src, dst, dir, mitype, micaster, id, dam, spllvl, None);
    if mitype == MissileID::ChargedBoltBow {
        add_missile(ctx, src, dst, dir, mitype, micaster, id, dam, spllvl, None);
        add_missile(ctx, src, dst, dir, mitype, micaster, id, dam, spllvl, None);
    }
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
    }
}

/// Original: `devilution::ProcessLightningControl` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessLightningControl(Missile &missile) sha=9f962fe2e748
pub fn process_lightning_control(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let currlevel = ctx.gendung.currlevel as i32;
    let dam = if m.is_trap() {
        // BUGFIX: damage of missile should be encoded in missile struct; monster can be dead before missile arrives.
        rnd(ctx, currlevel) + 2 * currlevel
    } else if m._micaster == TARGET_MONSTERS {
        // BUGFIX: damage of missile should be encoded in missile struct; player can be dead/have left the game before missile arrives.
        let lvl = ctx.players.Players[m._misource as usize]._pLevel as i32;
        (rnd(ctx, 2) + rnd(ctx, lvl) + 2) << 6
    } else {
        let monster = &ctx.monster.Monsters[m._misource as usize];
        let (min, max) = (monster.minDamage as i32, monster.maxDamage as i32);
        2 * (min + rnd(ctx, max - min + 1))
    };
    spawn_lightning(ctx, mi, dam);
}

/// Original: `devilution::ProcessLightning` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessLightning(Missile &missile) sha=697168342074
pub fn process_lightning(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let j = m._mirange;
    if m.position.tile != m.position.start {
        check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, true, m.position.tile, false);
    }
    if ctx.missiles.Missiles[mi]._miHitFlag {
        mis(ctx, mi)._mirange = j;
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessTownPortal` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessTownPortal(Missile &missile) sha=463dc7386b2b
pub fn process_town_portal(ctx: &mut Ctx, mi: usize) {
    const EXP_LIGHT_TP: [i32; 17] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 15, 15];
    {
        let m = mis(ctx, mi);
        if m._mirange > 1 {
            m._mirange -= 1;
        }
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if m._mirange == m.var1 {
        set_miss_dir(ctx, mi, 1);
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if ctx.gendung.leveltype != DungeonType::Town && m._mimfnum != 1 && m._mirange != 0 {
        if m.var2 == 0 {
            mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.tile, 1);
        }
        let l = ctx.missiles.Missiles[mi]._mlid;
        change_light(ctx, l, m.position.tile, EXP_LIGHT_TP[m.var2 as usize]);
        mis(ctx, mi).var2 += 1;
    }
    for p in 0..ctx.players.Players.len() {
        let player = &ctx.players.Players[p];
        if player.plractive && crate::player::is_on_active_level(ctx, p) && !player._pLvlChanging && player._pmode == PM_STAND && player.position.tile == m.position.tile {
            crate::player::clr_plr_path(ctx, p);
            if is_my(ctx, p) {
                crate::msg::net_send_cmd_param1(ctx, true, CMD_WARP, m._misource as u16);
                ctx.players.Players[p]._pmode = PM_NEWLVL;
            }
        }
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    put_missile(ctx, mi);
}

/// Shared body of `ProcessFlashBottom` and `ProcessFlashTop`.
fn process_flash(ctx: &mut Ctx, mi: usize, offsets: &[Direction]) {
    let m = ctx.missiles.Missiles[mi].clone();
    if m._micaster == TARGET_MONSTERS && !m.is_trap() {
        ctx.players.Players[m._misource as usize]._pInvincible = true;
    }
    mis(ctx, mi)._mirange -= 1;
    for &offset in offsets {
        let t = ctx.missiles.Missiles[mi].position.tile;
        check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, true, t + offset, true);
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        if m._micaster == TARGET_MONSTERS && !m.is_trap() {
            ctx.players.Players[m._misource as usize]._pInvincible = false;
        }
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessFlashBottom` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessFlashBottom(Missile &missile) sha=c309bbbddfad
pub fn process_flash_bottom(ctx: &mut Ctx, mi: usize) {
    process_flash(ctx, mi, &[Direction::NorthWest, Direction::NoDirection, Direction::SouthEast, Direction::West, Direction::SouthWest, Direction::South]);
}

/// Original: `devilution::ProcessFlashTop` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessFlashTop(Missile &missile) sha=85a6201bb74c
pub fn process_flash_top(ctx: &mut Ctx, mi: usize) {
    process_flash(ctx, mi, &[Direction::North, Direction::NorthEast, Direction::East]);
}

// ---------------------------------------------------------------------------------------------
// missiles.cpp, part 5: remaining ProcessXxx handlers.
// ---------------------------------------------------------------------------------------------

/// Original: `devilution::ProcessFlameWave` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessFlameWave(Missile &missile) sha=01ad1b769751
pub fn process_flame_wave(ctx: &mut Ctx, mi: usize) {
    {
        // Adjust missile's position for processing
        let m = mis(ctx, mi);
        m.position.tile += Direction::North;
        m.position.offset.delta_y += 32;
        m.var1 += 1;
    }
    if ctx.missiles.Missiles[mi].var1 == ctx.missiles.Missiles[mi]._miAnimLen {
        set_miss_dir(ctx, mi, 1);
        mis(ctx, mi)._miAnimFrame = rnd(ctx, 11) + 1;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    let j = m._mirange;
    move_missile_and_check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, false, false);
    if ctx.missiles.Missiles[mi]._miHitFlag {
        mis(ctx, mi)._mirange = j;
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if m._mimfnum != 0 || m._mirange == 0 {
        if m.position.tile != Point::new(m.var3, m.var4) {
            let mm = mis(ctx, mi);
            mm.var3 = m.position.tile.x;
            mm.var4 = m.position.tile.y;
            change_light(ctx, m._mlid, m.position.tile, 8);
        }
    } else {
        if m.var2 == 0 {
            mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.tile, EXP_LIGHT[0] as u8);
        }
        let l = ctx.missiles.Missiles[mi]._mlid;
        change_light(ctx, l, m.position.tile, EXP_LIGHT[m.var2 as usize]);
        mis(ctx, mi).var2 += 1;
    }
    {
        // Adjust missile's position for rendering
        let mm = mis(ctx, mi);
        mm.position.tile += Direction::South;
        mm.position.offset.delta_y -= 32;
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessGuardian` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessGuardian(Missile &missile) sha=f45c58f083bf
pub fn process_guardian(ctx: &mut Ctx, mi: usize) {
    {
        let m = mis(ctx, mi);
        m._mirange -= 1;
        if m.var2 > 0 {
            m.var2 -= 1;
        }
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if m._mirange == m.var1 || (m._mimfnum == 2 && m.var2 == 0) {
        set_miss_dir(ctx, mi, 1);
    }
    let position = m.position.tile;
    if (m._mirange % 16) == 0 {
        // Guardians pick a target by working backwards along lines originally based on VisionCrawlTable.
        // Because of their rather unique behaviour the points checked have been unrolled here
        const GUARDIAN_ARC: [(i32, i32); 48] = [
            (6, 0), (5, 0), (4, 0), (3, 0), (2, 0), (1, 0),
            (6, 1), (5, 1), (4, 1), (3, 1),
            (6, 2), (2, 1),
            (5, 2),
            (6, 3), (4, 2),
            (5, 3), (3, 2), (1, 1),
            (6, 4),
            (6, 5), (5, 4), (4, 3), (2, 2),
            (5, 5), (4, 4), (3, 3),
            (6, 6), (5, 6), (4, 5), (3, 4), (2, 3),
            (4, 6), (3, 5), (2, 4), (1, 2),
            (3, 6), (2, 5), (1, 3), (0, 1),
            (2, 6), (1, 4),
            (1, 5),
            (1, 6),
            (0, 2),
            (0, 3),
            (0, 6), (0, 5), (0, 4),
        ];
        for (x, y) in GUARDIAN_ARC {
            let offset = Displacement::new(x, y);
            if guardian_try_fire_at(ctx, mi, position + offset)
                || guardian_try_fire_at(ctx, mi, position + offset.flip_xy())
                || guardian_try_fire_at(ctx, mi, position + offset.flip_y())
                || guardian_try_fire_at(ctx, mi, position + offset.flip_x())
            {
                break;
            }
        }
    }
    if ctx.missiles.Missiles[mi]._mirange == 14 {
        set_miss_dir(ctx, mi, 0);
        let mm = mis(ctx, mi);
        mm._miAnimFrame = 15;
        mm._miAnimAdd = -1;
    }
    {
        let mm = mis(ctx, mi);
        mm.var3 += mm._miAnimAdd;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if m.var3 > 15 {
        mis(ctx, mi).var3 = 15;
    } else if m.var3 > 0 {
        change_light(ctx, m._mlid, position, m.var3);
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessChainLightning` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessChainLightning(Missile &missile) sha=a9ba4585d11b
pub fn process_chain_lightning(ctx: &mut Ctx, mi: usize) {
    let m = ctx.missiles.Missiles[mi].clone();
    let id = m._misource;
    let position = m.position.tile;
    let dst = Point::new(m.var1, m.var2);
    let dir = crate::engine::get_direction(position, dst);
    add_missile(ctx, position, dst, dir, MissileID::LightningControl, TARGET_MONSTERS, id, 1, m._mispllvl, None);
    let rad = (m._mispllvl + 3).min(crate::lighting::MaxCrawlRadius as i32);
    crate::lighting::do_crawl_range(1, rad as u32, &mut |displacement| {
        let target = position + displacement;
        if in_dungeon_bounds(target) && ctx.gendung.dMonster[target.x as usize][target.y as usize] > 0 {
            let dir = crate::engine::get_direction(position, target);
            add_missile(ctx, position, target, dir, MissileID::LightningControl, TARGET_MONSTERS, id, 1, m._mispllvl, None);
        }
        false
    });
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
    }
}

/// Original: `devilution::ProcessWeaponExplosion` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessWeaponExplosion(Missile &missile) sha=5798c2a3b1c5
pub fn process_weapon_explosion(ctx: &mut Ctx, mi: usize) {
    const EXP_LIGHT_W: [i32; 10] = [9, 10, 11, 12, 11, 10, 8, 6, 4, 2];
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let player = &ctx.players.Players[m._misource as usize];
    // BUGFIX: damage of missile should be encoded in missile struct; player can be dead/have left the game before missile arrives.
    let (mind, maxd, damage_type) = if m.var2 == 1 { (player._pIFMinDam, player._pIFMaxDam, DamageType::Fire) } else { (player._pILMinDam, player._pILMaxDam, DamageType::Lightning) };
    check_missile_col(ctx, mi, damage_type, mind, maxd, false, m.position.tile, false);
    let m = ctx.missiles.Missiles[mi].clone();
    if m.var1 == 0 {
        mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.tile, 9);
    } else if m._mirange != 0 {
        change_light(ctx, m._mlid, m.position.tile, EXP_LIGHT_W[m.var1 as usize]);
    }
    mis(ctx, mi).var1 += 1;
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    } else {
        put_missile(ctx, mi);
    }
}

/// Original: `devilution::ProcessMissileExplosion` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessMissileExplosion(Missile &missile) sha=a7d575fa51b9
pub fn process_missile_explosion(ctx: &mut Ctx, mi: usize) {
    const EXP_LIGHT_M: [i32; 15] = [9, 10, 11, 12, 11, 10, 8, 6, 4, 2, 1, 0, 0, 0, 0];
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    if m._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    } else {
        if m.var1 == 0 {
            mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.tile, 9);
        } else {
            change_light(ctx, m._mlid, m.position.tile, EXP_LIGHT_M[m.var1 as usize]);
        }
        mis(ctx, mi).var1 += 1;
        put_missile(ctx, mi);
    }
}

/// Original: `devilution::ProcessAcidSplate` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessAcidSplate(Missile &missile) sha=cfdbd9dc280a
pub fn process_acid_splate(ctx: &mut Ctx, mi: usize) {
    {
        let m = mis(ctx, mi);
        if m._mirange == m._miAnimLen {
            m.position.tile += Displacement::new(1, 1);
            m.position.offset.delta_y -= 32;
        }
        m._mirange -= 1;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if m._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        let monst = m._misource as usize;
        let dam = if crate::monster::monster_data(ctx, monst).level >= 2 { 2 } else { 1 };
        add_missile(ctx, m.position.tile, Point::new(0, 0), Direction::South, MissileID::AcidPuddle, TARGET_PLAYERS, monst as i32, dam, m._mispllvl, None);
    } else {
        put_missile(ctx, mi);
    }
}

/// Original: `devilution::ProcessTeleport` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessTeleport(Missile &missile) sha=c3576046312a
pub fn process_teleport(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange <= 0 {
        mis(ctx, mi)._miDelFlag = true;
        return;
    }
    let id = ctx.missiles.Missiles[mi]._misource as usize;
    let t = ctx.missiles.Missiles[mi].position.tile;
    let Some(dest) = crate::engine::path::find_closest_valid_position(ctx, &|ctx: &Ctx, target: Point| crate::player::pos_ok_player(ctx, id, target), t, 0, 5) else {
        return;
    };
    let old = ctx.players.Players[id].position.tile;
    ctx.gendung.dPlayer[old.x as usize][old.y as usize] = 0;
    crate::player::plr_clr_trans(ctx, old);
    {
        let p = &mut ctx.players.Players[id].position;
        p.tile = dest;
        p.future = dest;
        p.old = dest;
    }
    crate::player::plr_do_trans(ctx, dest);
    mis(ctx, mi).var1 = 1;
    ctx.gendung.dPlayer[dest.x as usize][dest.y as usize] = (id + 1) as i8;
    if ctx.gendung.leveltype != DungeonType::Town {
        let l = ctx.players.Players[id].lightId;
        crate::lighting::change_light_xy(ctx, l, dest);
        crate::lighting::change_vision_xy(ctx, id as i32, dest);
    }
    if is_my(ctx, id) {
        ctx.gendung.ViewPosition = dest;
    }
}

/// Original: `devilution::ProcessStoneCurse` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessStoneCurse(Missile &missile) sha=43c0b74c7677
pub fn process_stone_curse(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let mon = m.var2 as usize;
    if ctx.monster.Monsters[mon].hitPoints == 0 && m._miAnimType != MissileGraphicID::StoneCurseShatter {
        {
            let mm = mis(ctx, mi);
            mm._mimfnum = 0;
            mm._miDrawFlag = true;
        }
        set_miss_anim(ctx, mi, MissileGraphicID::StoneCurseShatter);
        mis(ctx, mi)._mirange = 11;
    }
    if ctx.monster.Monsters[mon].mode != MonsterMode::Petrified {
        mis(ctx, mi)._miDelFlag = true;
        return;
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        if ctx.monster.Monsters[mon].hitPoints > 0 {
            let monster = &mut ctx.monster.Monsters[mon];
            monster.mode = MonsterMode::from_raw(m.var1 as u8);
            monster.animInfo.isPetrified = false;
        } else {
            let (t, d) = (ctx.monster.Monsters[mon].position.tile, ctx.monster.Monsters[mon].direction);
            let s = ctx.dead.stonendx;
            crate::dead::add_corpse(ctx, t, s, d);
        }
    }
    if ctx.missiles.Missiles[mi]._miAnimType == MissileGraphicID::StoneCurseShatter {
        put_missile(ctx, mi);
    }
}

/// Original: `devilution::ProcessApocalypseBoom` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessApocalypseBoom(Missile &missile) sha=52f6732c0502
pub fn process_apocalypse_boom(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    if m.var1 == 0 {
        check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, false, m.position.tile, true);
    }
    if ctx.missiles.Missiles[mi]._miHitFlag {
        mis(ctx, mi).var1 = 1;
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessRhino` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessRhino(Missile &missile) sha=409d9846e314
pub fn process_rhino(ctx: &mut Ctx, mi: usize) {
    let monst = ctx.missiles.Missiles[mi]._misource as usize;
    if ctx.monster.Monsters[monst].mode != MonsterMode::Charge {
        mis(ctx, mi)._miDelFlag = true;
        return;
    }
    update_missile_pos(ctx, mi);
    let prev_pos = ctx.missiles.Missiles[mi].position.tile;
    let mut new_pos_snake = Point::default();
    ctx.gendung.dMonster[prev_pos.x as usize][prev_pos.y as usize] = 0;
    let is_snake = ctx.monster.Monsters[monst].ai == MonsterAIID::Snake;
    if is_snake {
        {
            let m = mis(ctx, mi);
            m.position.traveled += m.position.velocity * 2;
        }
        update_missile_pos(ctx, mi);
        new_pos_snake = ctx.missiles.Missiles[mi].position.tile;
        let m = mis(ctx, mi);
        m.position.traveled -= m.position.velocity;
    } else {
        let m = mis(ctx, mi);
        m.position.traveled += m.position.velocity;
    }
    update_missile_pos(ctx, mi);
    let new_pos = ctx.missiles.Missiles[mi].position.tile;
    if !crate::monster::is_tile_available_for_monster(ctx, monst, new_pos) || (is_snake && !crate::monster::is_tile_available_for_monster(ctx, monst, new_pos_snake)) {
        crate::monster::miss_to_monst(ctx, mi, prev_pos);
        mis(ctx, mi)._miDelFlag = true;
        return;
    }
    {
        let monster = &mut ctx.monster.Monsters[monst];
        monster.position.future = new_pos;
        monster.position.old = new_pos;
        monster.position.tile = new_pos;
    }
    ctx.gendung.dMonster[new_pos.x as usize][new_pos.y as usize] = -((monst + 1) as i16);
    if ctx.monster.Monsters[monst].is_unique() {
        let l = ctx.missiles.Missiles[mi]._mlid;
        crate::lighting::change_light_xy(ctx, l, new_pos);
    }
    move_missile_pos(ctx, mi);
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessFireWallControl` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessFireWallControl(Missile &missile) sha=9cd9e84bda82
pub fn process_fire_wall_control(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        return;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    let id = m._misource;
    {
        let position = Point::new(m.var1, m.var2);
        let target = position + Direction::from_u8(m.var3 as u8);
        if !m.limitReached && grow_wall(ctx, id, position, target, MissileID::FireWall, m._mispllvl, 0) {
            let mm = mis(ctx, mi);
            mm.var1 = target.x;
            mm.var2 = target.y;
        } else {
            mis(ctx, mi).limitReached = true;
        }
    }
    {
        let position = Point::new(m.var5, m.var6);
        let target = position + Direction::from_u8(m.var4 as u8);
        if m.var7 == 0 && grow_wall(ctx, id, position, target, MissileID::FireWall, m._mispllvl, 0) {
            let mm = mis(ctx, mi);
            mm.var5 = target.x;
            mm.var6 = target.y;
        } else {
            mis(ctx, mi).var7 = 1;
        }
    }
}

/// Original: `devilution::ProcessInfravision` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessInfravision(Missile &missile) sha=a5a2999dde96
pub fn process_infravision(ctx: &mut Ctx, mi: usize) {
    let p = ctx.missiles.Missiles[mi]._misource as usize;
    mis(ctx, mi)._mirange -= 1;
    ctx.players.Players[p]._pInfraFlag = true;
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        crate::items::calc_plr_item_vals(ctx, p, true);
    }
}

/// Original: `devilution::ProcessApocalypse` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessApocalypse(Missile &missile) sha=8d96cd8a519b
pub fn process_apocalypse(ctx: &mut Ctx, mi: usize) {
    let m = ctx.missiles.Missiles[mi].clone();
    let mut var4 = m.var4;
    for j in m.var2..m.var3 {
        for k in var4..m.var5 {
            let mid = ctx.gendung.dMonster[k as usize][j as usize] as i32 - 1;
            if mid < 0 {
                continue;
            }
            if ctx.monster.Monsters[mid as usize].is_player_minion() {
                continue;
            }
            if tile_has_any(ctx, ctx.gendung.dPiece[k as usize][j as usize] as i32, TileProperties::Solid) {
                continue;
            }
            if ctx.init.gb_is_hellfire && !crate::monster::line_clear_missile(ctx, m.position.tile, Point::new(k, j)) {
                continue;
            }
            let id = m._misource;
            let d = ctx.players.Players[id as usize]._pdir;
            add_missile(ctx, Point::new(k, j), Point::new(k, j), d, MissileID::ApocalypseBoom, TARGET_MONSTERS, id, m._midam, 0, None);
            let mm = mis(ctx, mi);
            mm.var2 = j;
            mm.var4 = k + 1;
            return;
        }
        var4 = m.var6;
        mis(ctx, mi).var4 = var4;
    }
    mis(ctx, mi)._miDelFlag = true;
}

/// Original: `devilution::ProcessFlameWaveControl` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessFlameWaveControl(Missile &missile) sha=7dd0a88bbb8e
pub fn process_flame_wave_control(ctx: &mut Ctx, mi: usize) {
    use crate::engine::geometry::{left, right};
    let mut f1 = false;
    let mut f2 = false;
    let m = ctx.missiles.Missiles[mi].clone();
    let id = m._misource;
    let src = m.position.tile;
    let sd = crate::engine::get_direction(src, Point::new(m.var1, m.var2));
    let dira = left(left(sd));
    let dirb = right(right(sd));
    let mut na = src + sd;
    let pn = piece_at(ctx, na);
    assert!((0..=crate::levels::gendung::MAXTILES as i32).contains(&pn));
    if !tile_has_any(ctx, pn, TileProperties::BlockMissile) {
        let pdir = ctx.players.Players[id as usize]._pdir;
        add_missile(ctx, na, na + sd, pdir, MissileID::FlameWave, TARGET_MONSTERS, id, 0, m._mispllvl, None);
        na += dira;
        let mut nb = src + sd + dirb;
        for _ in 0..(m._mispllvl / 2) + 2 {
            let pn = piece_at(ctx, na); // BUGFIX: dPiece is accessed before check against dungeon size and 0
            assert!((0..=crate::levels::gendung::MAXTILES as i32).contains(&pn));
            if tile_has_any(ctx, pn, TileProperties::BlockMissile) || f1 || !in_dungeon_bounds(na) {
                f1 = true;
            } else {
                add_missile(ctx, na, na + sd, pdir, MissileID::FlameWave, TARGET_MONSTERS, id, 0, m._mispllvl, None);
                na += dira;
            }
            let pn = piece_at(ctx, nb); // BUGFIX: dPiece is accessed before check against dungeon size and 0
            assert!((0..=crate::levels::gendung::MAXTILES as i32).contains(&pn));
            if tile_has_any(ctx, pn, TileProperties::BlockMissile) || f2 || !in_dungeon_bounds(nb) {
                f2 = true;
            } else {
                add_missile(ctx, nb, nb + sd, pdir, MissileID::FlameWave, TARGET_MONSTERS, id, 0, m._mispllvl, None);
                nb += dirb;
            }
        }
    }
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
    }
}

/// Original: `devilution::ProcessRage` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessRage(Missile &missile) sha=a78ee50235c7
pub fn process_rage(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    if ctx.missiles.Missiles[mi]._mirange != 0 {
        return;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    let p = m._misource as usize;
    let player = &mut ctx.players.Players[p];
    let mut hpdif = player._pMaxHP - player._pHitPoints;
    if player._pSpellFlags.has_any_of(SpellFlag::RageActive) {
        player._pSpellFlags.0 &= !SpellFlag::RageActive.0;
        player._pSpellFlags |= SpellFlag::RageCooldown;
        let lvl = player._pLevel as i32 * 2;
        mis(ctx, mi)._mirange = lvl + 10 * m._mispllvl + 245;
    } else {
        player._pSpellFlags.0 &= !SpellFlag::RageCooldown.0;
        mis(ctx, mi)._miDelFlag = true;
        hpdif += m.var2;
    }
    crate::items::calc_plr_item_vals(ctx, p, true);
    crate::player::apply_plr_damage(ctx, DamageType::Physical, p, 0, 1, hpdif, DeathReason::MonsterOrTrap);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    crate::player::player_say(ctx, p, HeroSpeech::HeavyBreathing);
}

/// Original: `devilution::ProcessInferno` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessInferno(Missile &missile) sha=f9e5ad114e91
pub fn process_inferno(ctx: &mut Ctx, mi: usize) {
    {
        let m = mis(ctx, mi);
        m._mirange -= 1;
        m.var2 -= 1;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    let mut k = m._mirange;
    check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, true, m.position.tile, false);
    if ctx.missiles.Missiles[mi]._mirange == 0 && ctx.missiles.Missiles[mi]._miHitFlag {
        mis(ctx, mi)._mirange = k;
    }
    if m.var2 == 0 {
        mis(ctx, mi)._miAnimFrame = 20;
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if m.var2 <= 0 {
        k = m._miAnimFrame;
        if k > 11 {
            k = 24 - k;
        }
        change_light(ctx, m._mlid, m.position.tile, k);
    }
    if m._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    if m.var2 <= 0 {
        put_missile(ctx, mi);
    }
}

/// Original: `devilution::ProcessInfernoControl` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessInfernoControl(Missile &missile) sha=483e5a546fb8
pub fn process_inferno_control(ctx: &mut Ctx, mi: usize) {
    {
        let m = mis(ctx, mi);
        m._mirange -= 1;
        m.position.traveled += m.position.velocity;
    }
    update_missile_pos(ctx, mi);
    let m = ctx.missiles.Missiles[mi].clone();
    if m.position.tile != Point::new(m.var1, m.var2) {
        let id = piece_at(ctx, m.position.tile);
        if !tile_has_any(ctx, id, TileProperties::BlockMissile) {
            add_missile(ctx, m.position.tile, m.position.start, Direction::South, MissileID::Inferno, m._micaster, m._misource, m.var3, m._mispllvl, Some(mi));
        } else {
            mis(ctx, mi)._mirange = 0;
        }
        let mm = mis(ctx, mi);
        mm.var1 = m.position.tile.x;
        mm.var2 = m.position.tile.y;
        mm.var3 += 1;
    }
    let m = mis(ctx, mi);
    if m._mirange == 0 || m.var3 == 3 {
        m._miDelFlag = true;
    }
}

/// Original: `devilution::ProcessChargedBolt` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessChargedBolt(Missile &missile) sha=b140e84b2b3e
pub fn process_charged_bolt(ctx: &mut Ctx, mi: usize) {
    use crate::engine::geometry::{left, right};
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    if m._miAnimType != MissileGraphicID::Lightning {
        if m.var3 == 0 {
            const BPATH: [i32; 16] = [-1, 0, 1, -1, 0, 1, -1, -1, 0, 0, 1, 1, 0, 1, -1, 0];
            let mut md = Direction::from_u8(m.var2 as u8);
            match BPATH[m._mirnd as usize] {
                -1 => md = left(md),
                1 => md = right(md),
                _ => {}
            }
            mis(ctx, mi)._mirnd = (m._mirnd + 1) & 0xF;
            update_missile_velocity(ctx, mi, m.position.tile + md, 8);
            mis(ctx, mi).var3 = 16;
        } else {
            mis(ctx, mi).var3 -= 1;
        }
        move_missile_and_check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), m._midam, m._midam, false, false);
        if ctx.missiles.Missiles[mi]._miHitFlag {
            {
                let mm = mis(ctx, mi);
                mm.var1 = 8;
                mm._mimfnum = 0;
                mm.position.offset = Displacement::new(0, 0);
                mm.position.velocity = Displacement::default();
            }
            set_miss_anim(ctx, mi, MissileGraphicID::Lightning);
            let mm = mis(ctx, mi);
            mm._mirange = mm._miAnimLen;
        }
        let mm = ctx.missiles.Missiles[mi].clone();
        change_light(ctx, mm._mlid, mm.position.tile, mm.var1);
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessHolyBolt` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessHolyBolt(Missile &missile) sha=8e0bd98904e2
pub fn process_holy_bolt(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    if m._miAnimType != MissileGraphicID::HolyBoltExplosion {
        let dam = m._midam;
        move_missile_and_check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), dam, dam, true, true);
        let m = ctx.missiles.Missiles[mi].clone();
        if m._mirange == 0 {
            mis(ctx, mi)._mimfnum = 0;
            set_miss_anim(ctx, mi, MissileGraphicID::HolyBoltExplosion);
            let mm = mis(ctx, mi);
            mm._mirange = mm._miAnimLen - 1;
            mm.position.stop_missile();
        } else if m.position.tile != Point::new(m.var1, m.var2) {
            let mm = mis(ctx, mi);
            mm.var1 = m.position.tile.x;
            mm.var2 = m.position.tile.y;
            change_light(ctx, m._mlid, m.position.tile, 8);
        }
    } else {
        change_light(ctx, m._mlid, m.position.tile, m._miAnimFrame + 7);
        if m._mirange == 0 {
            mis(ctx, mi)._miDelFlag = true;
            unlight(ctx, mi);
        }
    }
    put_missile(ctx, mi);
}

const EXPLOSION_OFFSETS: [Direction; 9] = [
    Direction::NoDirection,
    Direction::SouthWest,
    Direction::NorthEast,
    Direction::SouthEast,
    Direction::East,
    Direction::South,
    Direction::NorthWest,
    Direction::West,
    Direction::North,
];

/// Original: `devilution::ProcessElemental` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessElemental(Missile &missile) sha=9fcc56917155
pub fn process_elemental(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let dam = m._midam;
    let missile_position = m.position.tile;
    let damage_type = get_missile_data(m._mitype).damage_type();
    if m._miAnimType == MissileGraphicID::BigExplosion {
        change_light(ctx, m._mlid, m.position.tile, m._miAnimFrame);
        let start_point = if m.var3 == 2 { Point::new(m.var4, m.var5) } else { m.position.start };
        for offset in EXPLOSION_OFFSETS {
            if !check_block(ctx, start_point, missile_position + offset) {
                check_missile_col(ctx, mi, damage_type, dam, dam, true, missile_position + offset, true);
            }
        }
        if ctx.missiles.Missiles[mi]._mirange == 0 {
            mis(ctx, mi)._miDelFlag = true;
            unlight(ctx, mi);
        }
    } else {
        move_missile_and_check_missile_col(ctx, mi, damage_type, dam, dam, false, false);
        let m = ctx.missiles.Missiles[mi].clone();
        if m.var3 == 0 && missile_position == Point::new(m.var4, m.var5) {
            mis(ctx, mi).var3 = 1;
        }
        if ctx.missiles.Missiles[mi].var3 == 1 {
            {
                let mm = mis(ctx, mi);
                mm.var3 = 2;
                mm._mirange = 255;
            }
            match find_closest(ctx, missile_position, 19) {
                Some(next_monster) => {
                    let t = ctx.monster.Monsters[next_monster].position.tile;
                    let sd = crate::engine::get_direction(missile_position, t);
                    set_miss_dir(ctx, mi, sd as i32);
                    update_missile_velocity(ctx, mi, t, 16);
                }
                None => {
                    let sd = ctx.players.Players[m._misource as usize]._pdir;
                    set_miss_dir(ctx, mi, sd as i32);
                    update_missile_velocity(ctx, mi, missile_position + sd, 16);
                }
            }
        }
        let m = ctx.missiles.Missiles[mi].clone();
        if missile_position != Point::new(m.var1, m.var2) {
            let mm = mis(ctx, mi);
            mm.var1 = missile_position.x;
            mm.var2 = missile_position.y;
            change_light(ctx, m._mlid, missile_position, 8);
        }
        if ctx.missiles.Missiles[mi]._mirange == 0 {
            mis(ctx, mi)._mimfnum = 0;
            set_miss_anim(ctx, mi, MissileGraphicID::BigExplosion);
            let mm = mis(ctx, mi);
            mm._mirange = mm._miAnimLen - 1;
            mm.position.stop_missile();
        }
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessBoneSpirit` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessBoneSpirit(Missile &missile) sha=cb9866a38b56
pub fn process_bone_spirit(ctx: &mut Ctx, mi: usize) {
    mis(ctx, mi)._mirange -= 1;
    let m = ctx.missiles.Missiles[mi].clone();
    let dam = m._midam;
    if m._mimfnum == 8 {
        change_light(ctx, m._mlid, m.position.tile, m._miAnimFrame);
        if m._mirange == 0 {
            mis(ctx, mi)._miDelFlag = true;
            unlight(ctx, mi);
        }
        put_missile(ctx, mi);
    } else {
        move_missile_and_check_missile_col(ctx, mi, get_missile_data(m._mitype).damage_type(), dam, dam, false, false);
        let m = ctx.missiles.Missiles[mi].clone();
        let c = m.position.tile;
        if m.var3 == 0 && c == Point::new(m.var4, m.var5) {
            mis(ctx, mi).var3 = 1;
        }
        if ctx.missiles.Missiles[mi].var3 == 1 {
            {
                let mm = mis(ctx, mi);
                mm.var3 = 2;
                mm._mirange = 255;
            }
            match find_closest(ctx, c, 19) {
                Some(monster) => {
                    mis(ctx, mi)._midam = ctx.monster.Monsters[monster].hitPoints >> 7;
                    let t = ctx.monster.Monsters[monster].position.tile;
                    set_miss_dir(ctx, mi, crate::engine::get_direction(c, t) as i32);
                    update_missile_velocity(ctx, mi, t, 16);
                }
                None => {
                    let sd = ctx.players.Players[m._misource as usize]._pdir;
                    set_miss_dir(ctx, mi, sd as i32);
                    update_missile_velocity(ctx, mi, c + sd, 16);
                }
            }
        }
        let m = ctx.missiles.Missiles[mi].clone();
        if c != Point::new(m.var1, m.var2) {
            let mm = mis(ctx, mi);
            mm.var1 = c.x;
            mm.var2 = c.y;
            change_light(ctx, m._mlid, c, 8);
        }
        if ctx.missiles.Missiles[mi]._mirange == 0 {
            set_miss_dir(ctx, mi, 8);
            let mm = mis(ctx, mi);
            mm.position.velocity = Displacement::default();
            mm._mirange = 7;
        }
        put_missile(ctx, mi);
    }
}

/// Original: `devilution::ProcessResurrectBeam` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessResurrectBeam(Missile &missile) sha=0378eefd1784
pub fn process_resurrect_beam(ctx: &mut Ctx, mi: usize) {
    let m = mis(ctx, mi);
    m._mirange -= 1;
    if m._mirange == 0 {
        m._miDelFlag = true;
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::ProcessRedPortal` (missiles.cpp).
// @port missiles.cpp|devilution::ProcessRedPortal(Missile &missile) sha=bdc55dc79527
pub fn process_red_portal(ctx: &mut Ctx, mi: usize) {
    const EXP_LIGHT_RP: [i32; 17] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 15, 15];
    {
        let m = mis(ctx, mi);
        if m._mirange > 1 {
            m._mirange -= 1;
        }
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if m._mirange == m.var1 {
        set_miss_dir(ctx, mi, 1);
    }
    let m = ctx.missiles.Missiles[mi].clone();
    if ctx.gendung.leveltype != DungeonType::Town && m._mimfnum != 1 && m._mirange != 0 {
        if m.var2 == 0 {
            mis(ctx, mi)._mlid = crate::lighting::add_light(ctx, m.position.tile, 1);
        }
        let l = ctx.missiles.Missiles[mi]._mlid;
        change_light(ctx, l, m.position.tile, EXP_LIGHT_RP[m.var2 as usize]);
        mis(ctx, mi).var2 += 1;
    }
    if ctx.missiles.Missiles[mi]._mirange == 0 {
        mis(ctx, mi)._miDelFlag = true;
        unlight(ctx, mi);
    }
    put_missile(ctx, mi);
}

/// Original: `devilution::missiles_process_charge` (missiles.cpp).
// @port missiles.cpp|devilution::missiles_process_charge() sha=8233cb99dfb6
pub fn missiles_process_charge(ctx: &mut Ctx) {
    for mi in 0..ctx.missiles.Missiles.len() {
        let m = ctx.missiles.Missiles[mi].clone();
        let data = get_missile_sprite_data(ctx, m._miAnimType).sprites_for_direction(m._mimfnum as usize);
        mis(ctx, mi)._miAnimData = data;
        if m._mitype != MissileID::Rhino {
            continue;
        }
        let t = monster_type_id(ctx, m._misource as usize);
        let graphic = if matches!(t, MT_HORNED | MT_MUDRUN | MT_FROSTC | MT_OBLORD) {
            MonsterGraphic::Special
        } else if matches!(t, MT_NSNAKE | MT_RSNAKE | MT_BSNAKE | MT_GSNAKE) {
            MonsterGraphic::Attack
        } else {
            MonsterGraphic::Walk
        };
        let data = crate::monster::get_anim_data(ctx, m._misource as usize, graphic).sprites_for_direction(Direction::from_u8(m._mimfnum as u8));
        mis(ctx, mi)._miAnimData = data;
    }
}

/// Original: `devilution::RedoMissileFlags` (missiles.cpp).
// @port missiles.cpp|devilution::RedoMissileFlags() sha=daaf5645f635
pub fn redo_missile_flags(ctx: &mut Ctx) {
    for mi in 0..ctx.missiles.Missiles.len() {
        put_missile(ctx, mi);
    }
}
