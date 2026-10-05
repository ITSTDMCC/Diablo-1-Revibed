//! `Source/minitext` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn qtextflag(ctx: &Ctx) -> bool, "minitext.cpp|devilution::qtextflag");

crate::pending_fn!(pub fn set_qtextflag(ctx: &mut Ctx, v: bool), "minitext.cpp|devilution::qtextflag");

crate::pending_fn!(pub fn init_q_text_msg(ctx: &mut Ctx, m: crate::enums::_speech_id), "minitext.cpp|devilution::InitQTextMsg(_speech_id m)");
