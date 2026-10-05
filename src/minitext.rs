//! `Source/minitext` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn qtextflag(ctx: &Ctx) -> bool, "minitext.cpp|devilution::qtextflag");

crate::pending_fn!(pub fn set_qtextflag(ctx: &mut Ctx, v: bool), "minitext.cpp|devilution::qtextflag");
