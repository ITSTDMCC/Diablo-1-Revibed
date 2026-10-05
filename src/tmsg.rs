//! `Source/tmsg.cpp`: delayed (timed) game messages.

use std::collections::VecDeque;

use crate::ctx::Ctx;

/// `TMsg`
struct TMsg {
    time: u32,
    body: Vec<u8>,
}

impl TMsg {
    /// Original: `TMsg::TMsg` (tmsg.cpp).
    // @port tmsg.cpp|devilution::TMsg::TMsg(uint32_t time, const byte *data, uint8_t len) sha=6fe24640d6c7
    fn new(time: u32, data: &[u8], len: u8) -> TMsg {
        TMsg { time, body: data[..len as usize].to_vec() }
    }
}

/// Globals of tmsg.cpp.
#[derive(Default)]
pub struct TmsgState {
    /// `TimedMsgList`
    timed_msg_list: VecDeque<TMsg>,
}

/// Original: `devilution::tmsg_get` (tmsg.cpp). Returns the message once its time has come.
// @port tmsg.cpp|devilution::tmsg_get(std::unique_ptr<byte[]> *msg) sha=19cc5bfaba5c
pub fn tmsg_get(ctx: &mut Ctx) -> Option<Vec<u8>> {
    let head = ctx.tmsg.timed_msg_list.front()?;
    if head.time.wrapping_sub(ctx.platform.ticks()) as i32 >= 0 {
        return None;
    }
    ctx.tmsg.timed_msg_list.pop_front().map(|m| m.body)
}

/// Original: `devilution::tmsg_add` (tmsg.cpp).
// @port tmsg.cpp|devilution::tmsg_add(const byte *msg, uint8_t len) sha=68c3df66a371
pub fn tmsg_add(ctx: &mut Ctx, msg: &[u8]) {
    let time = ctx.platform.ticks().wrapping_add(ctx.diablo.gn_tick_delay as u32 * 10);
    // `uint8_t len`
    let len = msg.len() as u8;
    ctx.tmsg.timed_msg_list.push_back(TMsg::new(time, msg, len));
}

/// Original: `devilution::tmsg_start` (tmsg.cpp).
// @port tmsg.cpp|devilution::tmsg_start() sha=72f275b4183c
pub fn tmsg_start(ctx: &mut Ctx) {
    assert!(ctx.tmsg.timed_msg_list.is_empty());
}

/// Original: `devilution::tmsg_cleanup` (tmsg.cpp).
// @port tmsg.cpp|devilution::tmsg_cleanup() sha=ecdbc6399625
pub fn tmsg_cleanup(ctx: &mut Ctx) {
    ctx.tmsg.timed_msg_list.clear();
}
