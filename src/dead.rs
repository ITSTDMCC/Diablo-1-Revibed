//! `Source/dead.cpp`: corpses (pending).

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

crate::pending_fn!(pub fn add_corpse(ctx: &mut Ctx, tile: Point, d_type: i8, dir: Direction), "dead.cpp|devilution::AddCorpse(Point tile, int8_t dType, Direction dir)");

crate::pending_fn!(pub fn move_lights_to_corpses(ctx: &mut Ctx), "dead.cpp|devilution::MoveLightsToCorpses()");

crate::pending_fn!(pub fn init_corpses(ctx: &mut crate::ctx::Ctx), "dead.cpp|devilution::InitCorpses()");
