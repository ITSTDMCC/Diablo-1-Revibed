//! `Source/plrmsg` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn event_plr_msg(ctx: &mut Ctx, text: &str, style: crate::engine::render::text_render::UiFlags), "plrmsg.cpp|devilution::EventPlrMsg(string_view text, UiFlags style)");
