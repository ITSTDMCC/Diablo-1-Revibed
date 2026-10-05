//! `Source/levels/themes` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn init_themes(ctx: &mut crate::ctx::Ctx), "levels/themes.cpp|devilution::InitThemes()");

crate::pending_fn!(pub fn hold_theme_rooms(ctx: &mut crate::ctx::Ctx), "levels/themes.cpp|devilution::HoldThemeRooms()");

crate::pending_fn!(pub fn create_theme_rooms(ctx: &mut crate::ctx::Ctx), "levels/themes.cpp|devilution::CreateThemeRooms()");

/// Globals of levels/themes.cpp.
pub struct ThemesState {
    /// `zharlib`
    pub zharlib: i32,
}

impl Default for ThemesState {
    fn default() -> Self {
        ThemesState { zharlib: -1 }
    }
}
