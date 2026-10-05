//! `Source/qol/xpbar` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn init_xp_bar(ctx: &mut Ctx), "qol/xpbar.cpp|devilution::InitXPBar()");

crate::pending_fn!(pub fn free_xp_bar(ctx: &mut Ctx), "qol/xpbar.cpp|devilution::FreeXPBar()");
