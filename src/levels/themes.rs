//! `Source/levels/themes` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn init_themes(ctx: &mut crate::ctx::Ctx), "levels/themes.cpp|devilution::InitThemes()");

crate::pending_fn!(pub fn hold_theme_rooms(ctx: &mut crate::ctx::Ctx), "levels/themes.cpp|devilution::HoldThemeRooms()");

crate::pending_fn!(pub fn create_theme_rooms(ctx: &mut crate::ctx::Ctx), "levels/themes.cpp|devilution::CreateThemeRooms()");
