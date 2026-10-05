//! `Source/qol/monhealthbar` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn init_monster_health_bar(ctx: &mut Ctx), "qol/monhealthbar.cpp|devilution::InitMonsterHealthBar()");

crate::pending_fn!(pub fn free_monster_health_bar(ctx: &mut Ctx), "qol/monhealthbar.cpp|devilution::FreeMonsterHealthBar()");
