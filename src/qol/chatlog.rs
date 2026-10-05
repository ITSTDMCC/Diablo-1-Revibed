//! `Source/qol/chatlog` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn toggle_chat_log(ctx: &mut Ctx), "qol/chatlog.cpp|devilution::ToggleChatLog()");

crate::pending_fn!(pub fn clear_chat_log_flag(ctx: &mut Ctx), "qol/chatlog.cpp|devilution::ChatLogFlag");
