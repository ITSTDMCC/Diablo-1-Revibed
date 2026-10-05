//! `Source/help` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn help_key_pressed(ctx: &mut Ctx), "diablo.cpp|devilution::HelpKeyPressed()");

crate::pending_fn!(pub fn set_help_flag(ctx: &mut Ctx, v: bool), "help.cpp|devilution::HelpFlag");
