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

/// `MissileFileData` (only the loaded sprites so far; the static fields come with misdat).
#[derive(Default)]
pub struct MissileFileData {
    pub sprites: Option<ClxSpriteListOrSheet>,
}

impl MissileFileData {
    /// Original: `MissileFileData::FreeGFX` (misdat.h).
    // @port misdat.h|devilution::MissileFileData::FreeGFX() sha=67f9ef1c4ce8
    pub fn free_gfx(&mut self) {
        self.sprites = None;
    }
}

/// Globals of missiles.cpp / misdat.cpp.
#[derive(Default)]
pub struct MissilesState {
    /// `Missiles` (a `std::list` in the original; order is kept)
    pub Missiles: Vec<Missile>,
    pub MissilePreFlag: bool,
    /// `MissileSpriteData`
    pub missile_sprite_data: Vec<MissileFileData>,
}

/// Original: `devilution::FreeMissileGFX` (misdat.cpp).
// @port misdat.cpp|devilution::FreeMissileGFX() sha=dc3a9beaef6d
pub fn free_missile_gfx(ctx: &mut Ctx) {
    for missile_data in ctx.missiles.missile_sprite_data.iter_mut() {
        missile_data.free_gfx();
    }
}

crate::pending_fn!(
    pub fn add_missile(ctx: &mut Ctx, src: Point, dst: Point, midir: Direction, mitype: MissileID, micaster: mienemy_type, id: i32, midam: i32, spllvl: i32, parent: Option<usize>) -> Option<usize>,
    "missiles.cpp|devilution::AddMissile(Point src, Point dst, Direction midir, MissileID mitype, mienemy_type micaster, int id, int midam, int spllvl, Missile *parent, std::optional<_sfx_id> lSFX)"
);
