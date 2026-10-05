//! `Source/gamemenu` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn gamemenu_save_game(ctx: &mut Ctx, b: bool), "gamemenu.cpp|devilution::gamemenu_save_game(bool bActivate)");

crate::pending_fn!(pub fn gamemenu_load_game(ctx: &mut Ctx, b: bool), "gamemenu.cpp|devilution::gamemenu_load_game(bool bActivate)");

crate::pending_fn!(pub fn gamemenu_quit_game(ctx: &mut Ctx, b: bool), "gamemenu.cpp|devilution::gamemenu_quit_game(bool bActivate)");

crate::pending_fn!(pub fn gamemenu_off(ctx: &mut Ctx), "gamemenu.cpp|devilution::gamemenu_off()");

crate::pending_fn!(pub fn gamemenu_on(ctx: &mut Ctx), "gamemenu.cpp|devilution::gamemenu_on()");
