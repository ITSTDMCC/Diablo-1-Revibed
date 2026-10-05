//! `Source/multi.cpp`: game sessions (single and multiplayer).

use crate::ctx::Ctx;

/// `GameData`
#[derive(Clone, Copy, Debug, Default)]
pub struct GameData {
    pub size: i32,
    pub dwSeed: u32,
    pub programid: u32,
    pub versionMajor: u8,
    pub versionMinor: u8,
    pub versionPatch: u8,
    pub nDifficulty: crate::enums::_difficulty,
    pub nTickRate: u8,
    pub bRunInTown: u8,
    pub bTheoQuest: u8,
    pub bCowQuest: u8,
    pub bFriendlyFire: u8,
    pub fullQuests: u8,
}

/// Globals of multi.cpp.
#[derive(Default)]
pub struct MultiState {
    /// `sgGameInitInfo`
    pub sgGameInitInfo: GameData,
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
        ctx.players.MyPlayer = None;
    }
}

crate::pending_fn!(fn nthread_cleanup(ctx: &mut Ctx), "nthread.cpp|devilution::nthread_cleanup()");
crate::pending_fn!(fn tmsg_cleanup(ctx: &mut Ctx), "tmsg.cpp|devilution::tmsg_cleanup()");
crate::pending_fn!(fn unregister_net_event_handlers(ctx: &mut Ctx), "multi.cpp|devilution::UnregisterNetEventHandlers()");
