//! `Source/stores` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn stextflag_is_none(ctx: &Ctx) -> bool, "stores.cpp|devilution::stextflag");
