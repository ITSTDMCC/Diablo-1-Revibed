//! `Source/items.cpp`: items (only the drop graphics so far).

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;

/// `ITEMTYPES`
pub const ITEMTYPES: usize = 43;
/// `ItemAnimWidth`
pub const ITEM_ANIM_WIDTH: u16 = 96;

/// `ItemDropNames`: item type .cel file names.
pub const ITEM_DROP_NAMES: [&str; ITEMTYPES] = [
    "armor2", "axe", "fbttle", "bow", "goldflip", "helmut", "mace", "shield", "swrdflip", "rock", "cleaver", "staff", "ring", "crownf", "larmor", "wshield",
    "scroll", "fplatear", "fbook", "food", "fbttlebb", "fbttledy", "fbttleor", "fbttlebr", "fbttlebl", "fbttleby", "fbttlewh", "fbttledb", "fear", "fbrain",
    "fmush", "innsign", "bldstn", "fanvil", "flazstaf", "bombs1", "halfps1", "wholeps1", "runes1", "teddys1", "cows1", "donkys1", "mooses1",
];

/// Globals of items.cpp.
pub struct ItemsState {
    /// `itemanims`
    pub itemanims: Vec<Option<ClxSpriteList>>,
    /// `UniqueItemFlags`
    pub unique_item_flags: [bool; 128],
}

impl Default for ItemsState {
    fn default() -> Self {
        ItemsState { itemanims: (0..ITEMTYPES).map(|_| None).collect(), unique_item_flags: [false; 128] }
    }
}

/// Original: `devilution::InitItemGFX` (items.cpp).
// @port items.cpp|devilution::InitItemGFX() sha=4c1a0ab30db4
pub fn init_item_gfx(ctx: &mut Ctx) {
    let item_types = if ctx.init.gb_is_hellfire { ITEMTYPES } else { 35 };
    for i in 0..item_types {
        let arglist = format!("items\\{}", ITEM_DROP_NAMES[i]);
        ctx.items.itemanims[i] = Some(crate::engine::load_sprites::load_cel(ctx, &arglist, ITEM_ANIM_WIDTH));
    }
    ctx.items.unique_item_flags = [false; 128];
}

/// Original: `devilution::FreeItemGFX` (items.cpp).
// @port items.cpp|devilution::FreeItemGFX() sha=ff3c6c606a44
pub fn free_item_gfx(ctx: &mut Ctx) {
    for itemanim in ctx.items.itemanims.iter_mut() {
        *itemanim = None;
    }
}
