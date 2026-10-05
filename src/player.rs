//! `Source/player` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn stop_my_player(ctx: &mut Ctx), "diablo.cpp|devilution::InitKeymapActions() StopHero lambda");

/// Globals of player.cpp.
#[derive(Default)]
pub struct PlayerState {
    /// `MyPlayer`: index into `Players`, none before a game is set up.
    pub my_player: Option<usize>,
}

/// `MyPlayer != nullptr`
pub fn my_player_exists(ctx: &Ctx) -> bool {
    ctx.player.my_player.is_some()
}

crate::pending_fn!(pub fn clear_my_hold_item(ctx: &mut Ctx), "cursor.cpp|devilution::NewCursor(int cursId) MyPlayer->HoldItem.clear()");

crate::pending_fn!(pub fn clear_players(ctx: &mut Ctx), "multi.cpp|devilution::NetClose() Players.clear()");
