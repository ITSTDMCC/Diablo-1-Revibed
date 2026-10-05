//! `Source/doom` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn doom_close(ctx: &mut Ctx), "doom.cpp|devilution::doom_close()");

crate::pending_fn!(pub fn doom_init(ctx: &mut Ctx), "doom.cpp|devilution::doom_init()");
