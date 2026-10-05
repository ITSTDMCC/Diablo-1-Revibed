//! `Source/pfile` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn pfile_write_hero(ctx: &mut Ctx, write_game_data: bool), "pfile.cpp|devilution::pfile_write_hero(bool writeGameData)");

crate::pending_fn!(pub fn gb_valid_save_file(ctx: &Ctx) -> bool, "pfile.cpp|devilution::gbValidSaveFile");
