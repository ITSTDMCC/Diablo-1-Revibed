//! `Source/multi.cpp`: game sessions (single and multiplayer).

use crate::ctx::Ctx;

/// Globals of multi.cpp.
#[derive(Default)]
pub struct MultiState {
    /// `sgbNetInited`
    pub sgb_net_inited: bool,
}

/// Original: `devilution::NetClose` (multi.cpp).
// @port multi.cpp|devilution::NetClose() sha=83e70aeb87f9
pub fn net_close(ctx: &mut Ctx) {
    if !ctx.multi.sgb_net_inited {
        return;
    }
    ctx.multi.sgb_net_inited = false;
    nthread_cleanup(ctx);
    tmsg_cleanup(ctx);
    unregister_net_event_handlers(ctx);
    crate::storm::storm_net::snet_leave_game(ctx, 3);
    if ctx.init.gb_is_multiplayer {
        ctx.platform.delay(2000);
    }
    if !crate::engine::demomode::is_running(ctx) {
        crate::player::clear_players(ctx);
        ctx.player.my_player = None;
    }
}

crate::pending_fn!(fn nthread_cleanup(ctx: &mut Ctx), "nthread.cpp|devilution::nthread_cleanup()");
crate::pending_fn!(fn tmsg_cleanup(ctx: &mut Ctx), "tmsg.cpp|devilution::tmsg_cleanup()");
crate::pending_fn!(fn unregister_net_event_handlers(ctx: &mut Ctx), "multi.cpp|devilution::UnregisterNetEventHandlers()");
