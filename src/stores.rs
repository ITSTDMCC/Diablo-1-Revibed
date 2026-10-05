//! `Source/stores` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;
use crate::items::Item;

pub const WITCH_ITEMS: usize = 25;
pub const SMITH_ITEMS: usize = 25;
pub const SMITH_PREMIUM_ITEMS: usize = 15;
pub const STORE_LINES: usize = 104;

/// Globals of stores.cpp (the store UI state comes with the stores port).
pub struct StoresState {
    pub smithitem: Vec<Item>,
    pub numpremium: i32,
    pub premiumlevel: i32,
    pub premiumitems: Vec<Item>,
    pub healitem: Vec<Item>,
    pub witchitem: Vec<Item>,
    pub boylevel: i32,
    pub boyitem: Item,
}

impl Default for StoresState {
    fn default() -> Self {
        StoresState {
            smithitem: vec![Item::default(); SMITH_ITEMS],
            numpremium: 0,
            premiumlevel: 0,
            premiumitems: vec![Item::default(); SMITH_PREMIUM_ITEMS],
            healitem: vec![Item::default(); 20],
            witchitem: vec![Item::default(); WITCH_ITEMS],
            boylevel: 0,
            boyitem: Item::default(),
        }
    }
}

crate::pending_fn!(pub fn stextflag_is_none(ctx: &Ctx) -> bool, "stores.cpp|devilution::stextflag");
