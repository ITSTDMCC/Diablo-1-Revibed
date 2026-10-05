//! `Source/DiabloUI/multi/selconn.cpp`: choosing the multiplayer connection (pending).

use crate::ctx::Ctx;

crate::pending_fn!(pub fn ui_select_provider(ctx: &mut Ctx) -> bool, "DiabloUI/multi/selconn.cpp|devilution::UiSelectProvider(GameData *gameData)");
