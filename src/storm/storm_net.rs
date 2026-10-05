//! `Source/storm/storm_net.cpp`: the network provider facade (dvlnet).

use crate::ctx::Ctx;

/// `net::abstract_net`: the network back-ends (loopback, TCP, ZeroTier) are not ported yet.
pub trait AbstractNet {
    fn snet_leave_game(&mut self, type_: i32) -> bool;
}

/// Globals of storm_net.cpp.
#[derive(Default)]
pub struct StormNetState {
    /// `dvlnet_inst`
    pub dvlnet_inst: Option<Box<dyn AbstractNet>>,
    /// `dwLastError`
    pub dw_last_error: u32,
}

/// Original: `devilution::SErrGetLastError` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SErrGetLastError() sha=0ef342a7ef94
pub fn serr_get_last_error(ctx: &Ctx) -> u32 {
    ctx.storm_net.dw_last_error
}

/// Original: `devilution::SErrSetLastError` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SErrSetLastError(uint32_t dwErrCode) sha=e5a7901276dc
pub fn serr_set_last_error(ctx: &mut Ctx, dw_err_code: u32) {
    ctx.storm_net.dw_last_error = dw_err_code;
}

/// Original: `devilution::SNetDestroy` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetDestroy() sha=a15dcd049354
pub fn snet_destroy(ctx: &mut Ctx) -> bool {
    ctx.storm_net.dvlnet_inst = None;
    true
}

/// Original: `devilution::SNetLeaveGame` (storm/storm_net.cpp).
// @port storm/storm_net.cpp|devilution::SNetLeaveGame(int type) sha=5fcf6897e3fd
pub fn snet_leave_game(ctx: &mut Ctx, type_: i32) -> bool {
    match ctx.storm_net.dvlnet_inst.as_mut() {
        None => true,
        Some(inst) => inst.snet_leave_game(type_),
    }
}
