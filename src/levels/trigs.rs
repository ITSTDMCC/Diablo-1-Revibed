//! `Source/levels/trigs.cpp`: level entrances and exits (pending).

use crate::engine::geometry::Point;
use crate::enums::interface_mode;

pub const MAXTRIGGERS: usize = 7;

/// `TriggerStruct`
#[derive(Clone, Copy, Debug, Default)]
pub struct TriggerStruct {
    pub position: Point,
    pub _tmsg: interface_mode,
    pub _tlvl: i32,
}

/// Globals of trigs.cpp.
#[derive(Default)]
pub struct TrigsState {
    pub trigflag: bool,
    pub numtrigs: i32,
    pub trigs: [TriggerStruct; MAXTRIGGERS],
    pub TWarpFrom: i32,
}
