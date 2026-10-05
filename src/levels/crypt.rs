//! `Source/levels/crypt.cpp`: the Hellfire crypt levels (generation pending).

use crate::ctx::Ctx;

/// Globals of levels/crypt.cpp.
#[derive(Default)]
pub struct CryptState {
    pub UberRow: i32,
    pub UberCol: i32,
    pub IsUberRoomOpened: bool,
    pub IsUberLeverActivated: bool,
    pub UberDiabloMonsterIndex: i32,
}

/// `(UberRow, UberCol)`
pub fn uber_row_col(ctx: &Ctx) -> (i32, i32) {
    (ctx.crypt.UberRow, ctx.crypt.UberCol)
}

/// `IsUberRoomOpened = v`
pub fn set_is_uber_room_opened(ctx: &mut Ctx, v: bool) {
    ctx.crypt.IsUberRoomOpened = v;
}
