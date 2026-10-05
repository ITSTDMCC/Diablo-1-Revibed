//! `Source/missiles.cpp` and `Source/misdat.cpp`: missiles (data model; logic pending).

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
    pub fn is_drawn(&self) -> bool {
        (self.flags.0 & MissileDataFlags::Invisible.0) == 0
    }
    /// `isArrow`
    pub fn is_arrow(&self) -> bool {
        (self.flags.0 & MissileDataFlags::Arrow.0) != 0
    }
    /// `damageType`
    pub fn damage_type(&self) -> DamageType {
        DamageType::from_raw(self.flags.0 & 0b111)
    }
}

/// `GetMissileData` (misdat.h)
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

crate::pending_fn!(
    pub fn add_missile_sfx(ctx: &mut Ctx, src: Point, dst: Point, midir: Direction, mitype: MissileID, micaster: mienemy_type, id: i32, midam: i32, spllvl: i32, parent: Option<usize>, l_sfx: Option<crate::effects_data::SfxId>) -> Option<usize>,
    "missiles.cpp|devilution::AddMissile(Point src, Point dst, Direction midir, MissileID mitype, mienemy_type micaster, int id, int midam, int spllvl, Missile *parent, std::optional<_sfx_id> lSFX)"
);
crate::pending_fn!(pub fn set_miss_dir(ctx: &mut Ctx, mi: usize, dir: i32), "missiles.cpp|devilution::SetMissDir(Missile &missile, int dir)");

crate::pending_fn!(pub fn redo_missile_flags(ctx: &mut Ctx), "missiles.cpp|devilution::RedoMissileFlags()");
crate::pending_fn!(pub fn missiles_process_charge(ctx: &mut Ctx), "missiles.cpp|devilution::missiles_process_charge()");
crate::pending_fn!(pub fn tile_contains_missile(ctx: &Ctx, position: Point) -> bool, "missiles.cpp|devilution::TileContainsMissile(Point position)");




crate::pending_fn!(pub fn is_missile_blocked_by_tile(ctx: &crate::ctx::Ctx, tile: crate::engine::geometry::Point) -> bool, "missiles.cpp|devilution::IsMissileBlockedByTile(Point tile)");

crate::pending_fn!(pub fn add_acid(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddAcid(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_acid_puddle(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddAcidPuddle(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_apocalypse(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddApocalypse(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_apocalypse_boom(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddApocalypseBoom(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_arrow(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddArrow(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_berserk(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddBerserk(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_big_explosion(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddBigExplosion(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_bone_spirit(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddBoneSpirit(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_chain_lightning(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddChainLightning(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_charged_bolt(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddChargedBolt(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_charged_bolt_bow(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddChargedBoltBow(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_diablo_apocalypse(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddDiabloApocalypse(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_elemental(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddElemental(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_elemental_arrow(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddElementalArrow(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_fire_wall(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddFireWall(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_fire_wall_control(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddFireWallControl(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_fireball(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddFireball(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_firebolt(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddFirebolt(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_flame_wave(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddFlameWave(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_flame_wave_control(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddFlameWaveControl(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_flash_bottom(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddFlashBottom(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_flash_top(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddFlashTop(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_generic_magic_missile(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddGenericMagicMissile(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_golem(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddGolem(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_guardian(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddGuardian(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_heal_other(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddHealOther(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_healing(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddHealing(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_holy_bolt(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddHolyBolt(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_hork_spawn(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddHorkSpawn(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_identify(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddIdentify(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_immolation(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddImmolation(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_inferno(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddInferno(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_inferno_control(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddInfernoControl(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_infravision(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddInfravision(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_item_repair(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddItemRepair(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_jester(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddJester(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_lightning(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddLightning(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_lightning_bow(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddLightningBow(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_lightning_control(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddLightningControl(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_lightning_wall(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddLightningWall(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_magi(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddMagi(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_magma_ball(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddMagmaBall(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_mana(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddMana(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_mana_shield(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddManaShield(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_missile_explosion(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddMissileExplosion(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_nova(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddNova(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_nova_ball(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddNovaBall(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_open_nest(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddOpenNest(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_phasing(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddPhasing(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_rage(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddRage(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_red_portal(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddRedPortal(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_reflect(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddReflect(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_resurrect(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddResurrect(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_resurrect_beam(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddResurrectBeam(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_rhino(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddRhino(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_ring_of_fire(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddRingOfFire(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_rune_of_fire(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddRuneOfFire(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_rune_of_immolation(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddRuneOfImmolation(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_rune_of_light(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddRuneOfLight(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_rune_of_nova(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddRuneOfNova(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_rune_of_stone(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddRuneOfStone(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_search(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddSearch(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_spectral_arrow(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddSpectralArrow(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_staff_recharge(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddStaffRecharge(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_steal_mana(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddStealMana(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_steal_potions(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddStealPotions(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_stone_curse(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddStoneCurse(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_telekinesis(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddTelekinesis(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_teleport(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddTeleport(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_town_portal(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddTownPortal(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_trap_disarm(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddTrapDisarm(Missile &missile, AddMissileParameter &)");

crate::pending_fn!(pub fn add_warp(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddWarp(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn add_weapon_explosion(ctx: &mut crate::ctx::Ctx, mi: usize, parameter: &mut crate::missiles::AddMissileParameter), "missiles.cpp|devilution::AddWeaponExplosion(Missile &missile, AddMissileParameter &parameter)");

crate::pending_fn!(pub fn process_acid_puddle(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessAcidPuddle(Missile &missile)");

crate::pending_fn!(pub fn process_acid_splate(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessAcidSplate(Missile &missile)");

crate::pending_fn!(pub fn process_apocalypse(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessApocalypse(Missile &missile)");

crate::pending_fn!(pub fn process_apocalypse_boom(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessApocalypseBoom(Missile &missile)");

crate::pending_fn!(pub fn process_arrow(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessArrow(Missile &missile)");

crate::pending_fn!(pub fn process_big_explosion(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessBigExplosion(Missile &missile)");

crate::pending_fn!(pub fn process_bone_spirit(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessBoneSpirit(Missile &missile)");

crate::pending_fn!(pub fn process_chain_lightning(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessChainLightning(Missile &missile)");

crate::pending_fn!(pub fn process_charged_bolt(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessChargedBolt(Missile &missile)");

crate::pending_fn!(pub fn process_elemental(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessElemental(Missile &missile)");

crate::pending_fn!(pub fn process_elemental_arrow(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessElementalArrow(Missile &missile)");

crate::pending_fn!(pub fn process_fire_wall(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessFireWall(Missile &missile)");

crate::pending_fn!(pub fn process_fire_wall_control(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessFireWallControl(Missile &missile)");

crate::pending_fn!(pub fn process_fireball(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessFireball(Missile &missile)");

crate::pending_fn!(pub fn process_flame_wave(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessFlameWave(Missile &missile)");

crate::pending_fn!(pub fn process_flame_wave_control(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessFlameWaveControl(Missile &missile)");

crate::pending_fn!(pub fn process_flash_bottom(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessFlashBottom(Missile &missile)");

crate::pending_fn!(pub fn process_flash_top(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessFlashTop(Missile &missile)");

crate::pending_fn!(pub fn process_generic_projectile(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessGenericProjectile(Missile &missile)");

crate::pending_fn!(pub fn process_guardian(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessGuardian(Missile &missile)");

crate::pending_fn!(pub fn process_holy_bolt(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessHolyBolt(Missile &missile)");

crate::pending_fn!(pub fn process_hork_spawn(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessHorkSpawn(Missile &missile)");

crate::pending_fn!(pub fn process_immolation(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessImmolation(Missile &missile)");

crate::pending_fn!(pub fn process_inferno(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessInferno(Missile &missile)");

crate::pending_fn!(pub fn process_inferno_control(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessInfernoControl(Missile &missile)");

crate::pending_fn!(pub fn process_infravision(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessInfravision(Missile &missile)");

crate::pending_fn!(pub fn process_lightning(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessLightning(Missile &missile)");

crate::pending_fn!(pub fn process_lightning_bow(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessLightningBow(Missile &missile)");

crate::pending_fn!(pub fn process_lightning_control(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessLightningControl(Missile &missile)");

crate::pending_fn!(pub fn process_lightning_wall(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessLightningWall(Missile &missile)");

crate::pending_fn!(pub fn process_lightning_wall_control(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessLightningWallControl(Missile &missile)");

crate::pending_fn!(pub fn process_missile_explosion(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessMissileExplosion(Missile &missile)");

crate::pending_fn!(pub fn process_nova(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessNova(Missile &missile)");

crate::pending_fn!(pub fn process_nova_ball(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessNovaBall(Missile &missile)");

crate::pending_fn!(pub fn process_rage(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessRage(Missile &missile)");

crate::pending_fn!(pub fn process_red_portal(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessRedPortal(Missile &missile)");

crate::pending_fn!(pub fn process_resurrect_beam(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessResurrectBeam(Missile &missile)");

crate::pending_fn!(pub fn process_rhino(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessRhino(Missile &missile)");

crate::pending_fn!(pub fn process_ring_of_fire(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessRingOfFire(Missile &missile)");

crate::pending_fn!(pub fn process_rune(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessRune(Missile &missile)");

crate::pending_fn!(pub fn process_search(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessSearch(Missile &missile)");

crate::pending_fn!(pub fn process_spectral_arrow(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessSpectralArrow(Missile &missile)");

crate::pending_fn!(pub fn process_stone_curse(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessStoneCurse(Missile &missile)");

crate::pending_fn!(pub fn process_teleport(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessTeleport(Missile &missile)");

crate::pending_fn!(pub fn process_town_portal(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessTownPortal(Missile &missile)");

crate::pending_fn!(pub fn process_weapon_explosion(ctx: &mut crate::ctx::Ctx, mi: usize), "missiles.cpp|devilution::ProcessWeaponExplosion(Missile &missile)");

crate::pending_fn!(pub fn get_damage_amt(ctx: &Ctx, i: SpellID) -> (i32, i32), "missiles.cpp|devilution::GetDamageAmt(SpellID i, int *mind, int *maxd)");

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
