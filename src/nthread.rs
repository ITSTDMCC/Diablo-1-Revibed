//! `Source/nthread.cpp`: game ticks and network turns.
//!
//! In multiplayer the original runs `NthreadHandler` on a second thread that keeps sending and
//! receiving turns while the main thread has released `MemCrit` (`nthread_ignore_mutex(true)`:
//! loading a level, fading). The port's game state is single-threaded, so the handler runs
//! cooperatively instead: [`nthread_pump`] runs its loop iterations that are due, and is called
//! whenever a frame is presented (the loading screen, fades and the progress dialog all present
//! frames).

use crate::ctx::Ctx;
use crate::engine::animationinfo::AnimationInfo;
use crate::multi::MAX_PLRS;
use crate::storm::storm_net::*;

/// Globals of nthread.cpp.
#[derive(Default)]
pub struct NthreadState {
    pub sgbNetUpdateRate: u8,
    pub gdwMsgLenTbl: [usize; MAX_PLRS],
    pub gdwTurnsInTransit: u32,
    /// `glpMsgTbl`: the turn data received for each player
    pub glpMsgTbl: [Option<Vec<u8>>; MAX_PLRS],
    pub gdwLargestMsgSize: u32,
    pub gdwNormalMsgSize: u32,
    pub last_tick: i32,
    /// `ProgressToNextGameTick`: fraction (of `AnimationInfo::baseValueFraction`) of the next tick.
    pub ProgressToNextGameTick: u8,
    nthread_should_run: bool,
    sgbSyncCountdown: i8,
    turn_upper_bit: u32,
    sgbTicsOutOfSync: bool,
    sgbPacketCountdown: i8,
    sgbThreadIsRunning: bool,
    /// When the turn handler's next iteration is due (its `SDL_Delay`).
    handler_next_run: u32,
}

/// Original: `devilution::nthread_terminate_game` (nthread.cpp).
// @port nthread.cpp|devilution::nthread_terminate_game(const char *pszFcn) sha=e69774bcc3a5
pub fn nthread_terminate_game(ctx: &mut Ctx, fcn: &str) {
    let s_err = serr_get_last_error(ctx);
    if s_err == STORM_ERROR_INVALID_PLAYER {
        return;
    }
    if s_err != STORM_ERROR_GAME_TERMINATED && s_err != STORM_ERROR_NOT_IN_GAME {
        crate::appfat::app_fatal(ctx, &format!("{fcn}:\n{fcn}"));
    }
    ctx.multi.gbGameDestroyed = true;
}

/// Original: `devilution::nthread_send_and_recv_turn` (nthread.cpp).
// @port nthread.cpp|devilution::nthread_send_and_recv_turn(uint32_t curTurn, int turnDelta) sha=2cdac457ffc3
pub fn nthread_send_and_recv_turn(ctx: &mut Ctx, mut cur_turn: u32, turn_delta: i32) -> u32 {
    let mut cur_turns_in_transit = 0u32;
    if !snet_get_turns_in_transit(ctx, &mut cur_turns_in_transit) {
        nthread_terminate_game(ctx, "SNetGetTurnsInTransit");
        return 0;
    }
    loop {
        let more = cur_turns_in_transit < ctx.nthread.gdwTurnsInTransit;
        cur_turns_in_transit = cur_turns_in_transit.wrapping_add(1);
        if !more {
            break;
        }
        let turn_tmp = ctx.nthread.turn_upper_bit | (cur_turn & 0x7FFFFFFF);
        ctx.nthread.turn_upper_bit = 0;
        let turn = turn_tmp;
        if !snet_send_turn(ctx, &turn.to_le_bytes()) {
            nthread_terminate_game(ctx, "SNetSendTurn");
            return 0;
        }
        cur_turn = cur_turn.wrapping_add(turn_delta as u32);
        if cur_turn >= 0x7FFFFFFF {
            cur_turn &= 0xFFFF;
        }
    }
    cur_turn
}

/// Original: `devilution::nthread_recv_turns` (nthread.cpp).
// @port nthread.cpp|devilution::nthread_recv_turns(bool *pfSendAsync) sha=a28f9816f2b3
pub fn nthread_recv_turns(ctx: &mut Ctx, pf_send_async: Option<&mut bool>) -> bool {
    let mut send_async = false;
    let tick_delay = ctx.diablo.gn_tick_delay as i32;
    let n = &mut ctx.nthread;
    n.sgbPacketCountdown = n.sgbPacketCountdown.wrapping_sub(1);
    let result = 'r: {
        if n.sgbPacketCountdown > 0 {
            n.last_tick = n.last_tick.wrapping_add(tick_delay);
            break 'r true;
        }
        n.sgbSyncCountdown = n.sgbSyncCountdown.wrapping_sub(1);
        n.sgbPacketCountdown = n.sgbNetUpdateRate as i8;
        if n.sgbSyncCountdown != 0 {
            send_async = true;
            n.last_tick = n.last_tick.wrapping_add(tick_delay);
            break 'r true;
        }
        if !snet_receive_turns(ctx) {
            if serr_get_last_error(ctx) != STORM_ERROR_NO_MESSAGES_WAITING {
                nthread_terminate_game(ctx, "SNetReceiveTurns");
            }
            let n = &mut ctx.nthread;
            n.sgbTicsOutOfSync = false;
            n.sgbSyncCountdown = 1;
            n.sgbPacketCountdown = 1;
            break 'r false;
        }
        if !ctx.nthread.sgbTicsOutOfSync {
            ctx.nthread.sgbTicsOutOfSync = true;
            ctx.nthread.last_tick = ctx.platform.ticks() as i32;
        }
        ctx.nthread.sgbSyncCountdown = 4;
        crate::multi::multi_msg_countdown(ctx);
        send_async = true;
        ctx.nthread.last_tick = ctx.nthread.last_tick.wrapping_add(tick_delay);
        true
    };
    if let Some(p) = pf_send_async {
        *p = send_async;
    }
    result
}

/// Original: `devilution::nthread_set_turn_upper_bit` (nthread.cpp).
// @port nthread.cpp|devilution::nthread_set_turn_upper_bit() sha=bbc67f7cd81b
pub fn nthread_set_turn_upper_bit(ctx: &mut Ctx) {
    ctx.nthread.turn_upper_bit = 0x80000000;
}

/// Original: `devilution::nthread_start` (nthread.cpp).
// @port nthread.cpp|devilution::nthread_start(bool setTurnUpperBit) sha=4710542b7800
pub fn nthread_start(ctx: &mut Ctx, set_turn_upper_bit: bool) {
    ctx.nthread.last_tick = ctx.platform.ticks() as i32;
    ctx.nthread.sgbPacketCountdown = 1;
    ctx.nthread.sgbSyncCountdown = 1;
    ctx.nthread.sgbTicsOutOfSync = true;
    if set_turn_upper_bit {
        nthread_set_turn_upper_bit(ctx);
    } else {
        ctx.nthread.turn_upper_bit = 0;
    }
    let mut caps = SnetCaps { size: 36, ..Default::default() };
    snet_get_provider_caps(ctx, &mut caps);
    let n = &mut ctx.nthread;
    n.gdwTurnsInTransit = caps.defaultturnsintransit;
    if n.gdwTurnsInTransit == 0 {
        n.gdwTurnsInTransit = 1;
    }
    if caps.defaultturnssec <= 20 && caps.defaultturnssec != 0 {
        n.sgbNetUpdateRate = (20 / caps.defaultturnssec) as u8;
    } else {
        n.sgbNetUpdateRate = 1;
    }
    let mut largest_msg_size = 512u32;
    if caps.maxmessagesize < 0x200 {
        largest_msg_size = caps.maxmessagesize;
    }
    n.gdwLargestMsgSize = largest_msg_size;
    n.gdwNormalMsgSize = caps.bytessec.wrapping_mul(n.sgbNetUpdateRate as u32) / 20;
    n.gdwNormalMsgSize = n.gdwNormalMsgSize.wrapping_mul(3);
    n.gdwNormalMsgSize >>= 2;
    if caps.maxplayers > MAX_PLRS as u32 {
        caps.maxplayers = MAX_PLRS as u32;
    }
    n.gdwNormalMsgSize /= caps.maxplayers;
    while n.gdwNormalMsgSize < 0x80 {
        n.gdwNormalMsgSize *= 2;
        n.sgbNetUpdateRate = n.sgbNetUpdateRate.wrapping_mul(2);
    }
    if n.gdwNormalMsgSize > largest_msg_size {
        n.gdwNormalMsgSize = largest_msg_size;
    }
    if ctx.init.gb_is_multiplayer {
        ctx.nthread.sgbThreadIsRunning = false;
        // MemCrit.lock(); the handler waits for nthread_ignore_mutex(true)
        ctx.nthread.nthread_should_run = true;
    }
}

/// Original: `NthreadHandler` (nthread.cpp): one pass of its loop. Returns the delay before the
/// next pass, or `None` when the handler stops.
// @port nthread.cpp|devilution::NthreadHandler() sha=54a48ea380ce
fn nthread_handler(ctx: &mut Ctx) -> Option<i32> {
    if !ctx.nthread.nthread_should_run {
        return None;
    }
    nthread_send_and_recv_turn(ctx, 0, 0);
    let mut delta = ctx.diablo.gn_tick_delay as i32;
    if nthread_recv_turns(ctx, None) {
        delta = ctx.nthread.last_tick.wrapping_sub(ctx.platform.ticks() as i32);
    }
    if !ctx.nthread.nthread_should_run {
        return None;
    }
    Some(delta)
}

/// Runs the turn handler's passes that are due while the main thread has the mutex released
/// (see the module comment).
pub fn nthread_pump(ctx: &mut Ctx) {
    for _ in 0..8 {
        if !ctx.nthread.nthread_should_run || !ctx.nthread.sgbThreadIsRunning {
            return;
        }
        let now = ctx.platform.ticks();
        if (now.wrapping_sub(ctx.nthread.handler_next_run) as i32) < 0 {
            return;
        }
        match nthread_handler(ctx) {
            Some(delta) => ctx.nthread.handler_next_run = now.wrapping_add(delta.max(0) as u32),
            None => return,
        }
    }
}

/// Original: `devilution::nthread_cleanup` (nthread.cpp).
// @port nthread.cpp|devilution::nthread_cleanup() sha=9bdc4b7ca5d2
pub fn nthread_cleanup(ctx: &mut Ctx) {
    let n = &mut ctx.nthread;
    n.nthread_should_run = false;
    n.gdwTurnsInTransit = 0;
    n.gdwNormalMsgSize = 0;
    n.gdwLargestMsgSize = 0;
}

/// Original: `devilution::nthread_ignore_mutex` (nthread.cpp). Without the turn thread
/// (single player) this does nothing, as in the original.
// @port nthread.cpp|devilution::nthread_ignore_mutex(bool bStart) sha=3b7295158945
pub fn nthread_ignore_mutex(ctx: &mut Ctx, b_start: bool) {
    if !ctx.nthread.nthread_should_run {
        return;
    }
    ctx.nthread.sgbThreadIsRunning = b_start;
    if b_start {
        ctx.nthread.handler_next_run = ctx.platform.ticks();
    }
}

/// Original: `devilution::nthread_has_500ms_passed` (nthread.cpp).
// @port nthread.cpp|devilution::nthread_has_500ms_passed(bool *drawGame) sha=2020296989de
pub fn nthread_has_500ms_passed(ctx: &mut Ctx, draw_game: Option<&mut bool>) -> bool {
    let current_tick_count = ctx.platform.ticks() as i32;
    let tick_delay = ctx.diablo.gn_tick_delay as i32;
    let mut ticks_elapsed = current_tick_count.wrapping_sub(ctx.nthread.last_tick);
    if ticks_elapsed > tick_delay * 10 {
        let mut reset_last_tick = true;
        if ctx.init.gb_is_multiplayer {
            for i in 0..ctx.players.Players.len() {
                if (ctx.multi.player_state[i] & PS_CONNECTED) != 0 && i != ctx.players.MyPlayerId {
                    reset_last_tick = false;
                    break;
                }
            }
        }
        if reset_last_tick {
            ctx.nthread.last_tick = current_tick_count;
            ticks_elapsed = 0;
        }
    }
    if let Some(d) = draw_game {
        *d = ticks_elapsed <= tick_delay;
    }
    ticks_elapsed >= 0
}

/// Original: `devilution::nthread_UpdateProgressToNextGameTick` (nthread.cpp).
// @port nthread.cpp|devilution::nthread_UpdateProgressToNextGameTick() sha=8cd47a9f9094
pub fn nthread_update_progress_to_next_game_tick(ctx: &mut Ctx) {
    if !ctx.diablo.gb_run_game
        || ctx.diablo.pause_mode != 0
        || (!ctx.init.gb_is_multiplayer && crate::gmenu::gmenu_is_active(ctx))
        || !ctx.diablo.gb_process_players
        || crate::engine::demomode::is_running(ctx)
    {
        return;
    }
    let current_tick_count = ctx.platform.ticks() as i32;
    let ticks_missing = ctx.nthread.last_tick.wrapping_sub(current_tick_count);
    let base = AnimationInfo::BASE_VALUE_FRACTION;
    if ticks_missing <= 0 {
        ctx.nthread.ProgressToNextGameTick = base as u8;
        return;
    }
    let tick_delay = ctx.diablo.gn_tick_delay as i32;
    let ticks_advanced = tick_delay - ticks_missing;
    let fraction = (ticks_advanced * base / tick_delay).clamp(0, base);
    ctx.nthread.ProgressToNextGameTick = fraction as u8;
}
