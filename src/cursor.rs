//! `Source/cursor.cpp`: the mouse cursor (software and hardware), and what it points at.

use crate::ctx::Ctx;

/// `cursor_id`
pub const CURSOR_NONE: i32 = 0;
pub const CURSOR_HAND: i32 = 1;
pub const CURSOR_IDENTIFY: i32 = 2;
pub const CURSOR_REPAIR: i32 = 3;
pub const CURSOR_RECHARGE: i32 = 4;
pub const CURSOR_DISARM: i32 = 5;
pub const CURSOR_OIL: i32 = 6;
pub const CURSOR_TELEKINESIS: i32 = 7;
pub const CURSOR_RESURRECT: i32 = 8;
pub const CURSOR_TELEPORT: i32 = 9;
pub const CURSOR_HEALOTHER: i32 = 10;
pub const CURSOR_HOURGLASS: i32 = 11;
pub const CURSOR_FIRSTITEM: i32 = 12;

pub struct CursorState {
    /// `pcurs`
    pub pcurs: i32,
    /// `pcursitem`
    pub pcursitem: i8,
    /// `pcursmonst`
    pub pcursmonst: i32,
    /// `pcursplr`
    pub pcursplr: i8,
    /// `ObjectUnderCursor` (index into `Objects`)
    pub ObjectUnderCursor: Option<usize>,
    /// `pcurstemp`
    pub pcurstemp: i32,
    /// `pcursinvitem`
    pub pcursinvitem: i8,
    /// `pcursstashitem`
    pub pcursstashitem: u16,
    /// `cursPosition`
    pub cursPosition: crate::engine::geometry::Point,
}

impl Default for CursorState {
    fn default() -> Self {
        CursorState {
            pcurs: 0,
            pcursitem: 0,
            pcursmonst: -1,
            pcursplr: 0,
            ObjectUnderCursor: None,
            pcurstemp: 0,
            pcursinvitem: 0,
            pcursstashitem: 0,
            cursPosition: Default::default(),
        }
    }
}

/// Original: `devilution::ResetCursor` (cursor.cpp).
// @port cursor.cpp|devilution::ResetCursor() sha=0028c9b9aa5b
pub fn reset_cursor(ctx: &mut Ctx) {
    let p = ctx.cursor.pcurs;
    new_cursor(ctx, p);
}

/// Original: `devilution::NewCursor(int cursId)` (cursor.cpp).
// @port cursor.cpp|devilution::NewCursor(int cursId) sha=c06610c5a462
pub fn new_cursor(ctx: &mut Ctx, curs_id: i32) {
    if ctx.cursor.pcurs >= CURSOR_FIRSTITEM && curs_id > CURSOR_HAND && curs_id < CURSOR_HOURGLASS && !crate::controls::plrctrls::try_drop_item(ctx) {
        return;
    }
    if curs_id < CURSOR_HOURGLASS && crate::player::my_player_exists(ctx) {
        crate::player::clear_my_hold_item(ctx);
    }
    ctx.cursor.pcurs = curs_id;
    if crate::hwcursor::is_hardware_cursor_enabled(ctx) && ctx.controls.control_device == crate::controls::ControlTypes::KeyboardAndMouse {
        let has_art_cursor = ctx.diablo_ui.art_cursor.is_some();
        if !has_art_cursor && curs_id == CURSOR_NONE {
            return;
        }
        let new_cursor =
            if has_art_cursor { crate::hwcursor::CursorInfo::user_interface_cursor() } else { crate::hwcursor::CursorInfo::game_cursor(curs_id) };
        if new_cursor != *crate::hwcursor::get_current_cursor_info(ctx) {
            crate::hwcursor::set_hardware_cursor(ctx, new_cursor);
        }
    }
}

crate::pending_fn!(pub fn set_hardware_cursor_from_sprite(ctx: &mut Ctx, pcurs: i32) -> bool, "hwcursor.cpp|devilution::SetHardwareCursorFromSprite(int pcurs)");

/// `InvItemWidth1`
#[rustfmt::skip]
static InvItemWidth1: [u16; 179] = [33, 32, 32, 32, 32, 32, 32, 32, 32, 32, 23, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56];
/// `InvItemWidth2`
#[rustfmt::skip]
static InvItemWidth2: [u16; 61] = [28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 56, 56, 28, 28, 28, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56];
/// `InvItemHeight1`
#[rustfmt::skip]
static InvItemHeight1: [u16; 179] = [29, 32, 32, 32, 32, 32, 32, 32, 32, 32, 35, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 56, 56, 56, 56, 56, 56, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84];
/// `InvItemHeight2`
#[rustfmt::skip]
static InvItemHeight2: [u16; 61] = [28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 56, 56, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84];

/// `InvItems1Size`
const InvItems1Size: i32 = InvItemWidth1.len() as i32;

/// Original: `devilution::GetInvItemSize` (cursor.cpp).
// @port cursor.cpp|devilution::GetInvItemSize(int cursId) sha=2b9e41fcd993
pub fn get_inv_item_size(curs_id: i32) -> crate::engine::geometry::Size {
    let i = curs_id - 1;
    if i >= InvItems1Size {
        let j = (i - InvItems1Size) as usize;
        return crate::engine::geometry::Size::new(InvItemWidth2[j] as i32, InvItemHeight2[j] as i32);
    }
    crate::engine::geometry::Size::new(InvItemWidth1[i as usize] as i32, InvItemHeight1[i as usize] as i32)
}

/// Original: `devilution::GetNumInvItems` (cursor.cpp).
// @port cursor.cpp|devilution::GetNumInvItems() sha=d99f2d240ccc
pub fn get_num_inv_items() -> usize {
    InvItemWidth1.len() + InvItemWidth2.len()
}

/// Original: `devilution::NewCursor(const Item &item)` (cursor.cpp).
// @port cursor.cpp|devilution::NewCursor(const Item &item) sha=e024d6d38be2
pub fn new_cursor_item(ctx: &mut Ctx, item_empty: bool, item_curs: u8) {
    if item_empty {
        new_cursor(ctx, CURSOR_HAND);
    } else {
        new_cursor(ctx, item_curs as i32 + CURSOR_FIRSTITEM);
    }
}
