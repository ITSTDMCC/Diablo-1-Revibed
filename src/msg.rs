//! `Source/msg.cpp`: game commands sent between players and the level delta state (pending).

use crate::ctx::Ctx;
use crate::engine::geometry::Point;
use crate::enums::*;

/// Globals of msg.cpp.
#[derive(Default)]
pub struct MsgState {
    /// `gbBufferMsgs`
    pub gbBufferMsgs: u8,
}

crate::pending_fn!(pub fn delta_add_item(ctx: &mut Ctx, ii: i32), "msg.cpp|devilution::DeltaAddItem(int ii)");
crate::pending_fn!(pub fn net_send_cmd(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id), "msg.cpp|devilution::NetSendCmd(bool bHiPri, _cmd_id bCmd)");
crate::pending_fn!(pub fn net_send_cmd_p_item(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, position: Point, ii: usize), "msg.cpp|devilution::NetSendCmdPItem(bool bHiPri, _cmd_id bCmd, Point position, const Item &item)");
crate::pending_fn!(pub fn net_send_cmd_quest(ctx: &mut Ctx, b_hi_pri: bool, q: usize), "msg.cpp|devilution::NetSendCmdQuest(bool bHiPri, const Quest &quest)");
crate::pending_fn!(pub fn net_send_cmd_param1(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, w_param1: u16), "msg.cpp|devilution::NetSendCmdParam1(bool bHiPri, _cmd_id bCmd, uint16_t wParam1)");
crate::pending_fn!(pub fn net_send_cmd_param2(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, w_param1: u16, w_param2: u16), "msg.cpp|devilution::NetSendCmdParam2(bool bHiPri, _cmd_id bCmd, uint16_t wParam1, uint16_t wParam2)");
crate::pending_fn!(pub fn net_send_cmd_param5(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, w_param1: u16, w_param2: u16, w_param3: u16, w_param4: u16, w_param5: u16), "msg.cpp|devilution::NetSendCmdParam5(bool bHiPri, _cmd_id bCmd, uint16_t wParam1, uint16_t wParam2, uint16_t wParam3, uint16_t wParam4, uint16_t wParam5)");
crate::pending_fn!(pub fn net_send_cmd_loc_param4(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, position: Point, w_param1: u16, w_param2: u16, w_param3: u16, w_param4: u16), "msg.cpp|devilution::NetSendCmdLocParam4(bool bHiPri, _cmd_id bCmd, Point position, uint16_t wParam1, uint16_t wParam2, uint16_t wParam3, uint16_t wParam4)");
crate::pending_fn!(pub fn net_send_cmd_loc_param5(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, position: Point, w_param1: u16, w_param2: u16, w_param3: u16, w_param4: u16, w_param5: u16), "msg.cpp|devilution::NetSendCmdLocParam5(bool bHiPri, _cmd_id bCmd, Point position, uint16_t wParam1, uint16_t wParam2, uint16_t wParam3, uint16_t wParam4, uint16_t wParam5)");
crate::pending_fn!(pub fn net_send_cmd_ch_item(ctx: &mut Ctx, b_hi_pri: bool, b_loc: u8), "msg.cpp|devilution::NetSendCmdChItem(bool bHiPri, uint8_t bLoc)");
crate::pending_fn!(pub fn net_sync_inv_item(ctx: &mut Ctx, pnum: usize, item: i32), "msg.cpp|devilution::NetSyncInvItem(const Player &player, int invListIndex)");
crate::pending_fn!(pub fn net_send_cmd_damage(ctx: &mut Ctx, b_hi_pri: bool, b_plr: u8, dw_dam: u32, damage_type: DamageType), "msg.cpp|devilution::NetSendCmdDamage(bool bHiPri, uint8_t bPlr, uint32_t dwDam, DamageType damageType)");
crate::pending_fn!(pub fn net_send_cmd_g_item(ctx: &mut Ctx, b_hi_pri: bool, b_cmd: _cmd_id, pnum: u8, ii: u8), "msg.cpp|devilution::NetSendCmdGItem(bool bHiPri, _cmd_id bCmd, uint8_t pnum, uint8_t ii)");
