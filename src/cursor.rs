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

#[derive(Default)]
pub struct CursorState {
    /// `pcurs`
    pub pcurs: i32,
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
    if ctx.cursor.pcurs >= CURSOR_FIRSTITEM && curs_id > CURSOR_HAND && curs_id < CURSOR_HOURGLASS && !try_drop_item(ctx) {
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

crate::pending_fn!(fn try_drop_item(ctx: &mut Ctx) -> bool, "inv.cpp|devilution::TryDropItem()");
crate::pending_fn!(pub fn set_hardware_cursor_from_sprite(ctx: &mut Ctx, pcurs: i32) -> bool, "hwcursor.cpp|devilution::SetHardwareCursorFromSprite(int pcurs)");
