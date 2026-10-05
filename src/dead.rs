//! `Source/dead.cpp`: corpses (pending).

use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Point};

crate::pending_fn!(pub fn add_corpse(ctx: &mut Ctx, tile: Point, d_type: i8, dir: Direction), "dead.cpp|devilution::AddCorpse(Point tile, int8_t dType, Direction dir)");

crate::pending_fn!(pub fn move_lights_to_corpses(ctx: &mut Ctx), "dead.cpp|devilution::MoveLightsToCorpses()");
