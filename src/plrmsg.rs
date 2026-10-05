//! `Source/plrmsg.cpp`: the in-game chat/event messages above the control panel.

use crate::ctx::Ctx;
use crate::engine::render::text_render::{draw_string, get_line_height, word_wrap_string, GameFontTables, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::utils::language::tr;

/// `PlayerMessage`
#[derive(Clone, Debug, Default)]
struct PlayerMessage {
    time: u32,
    style: UiFlags,
    text: String,
    /// length of the leading part rendered in gold (`from`)
    from_len: usize,
    line_height: i32,
}

/// Globals of plrmsg.cpp.
#[derive(Default)]
pub struct PlrMsgState {
    messages: [PlayerMessage; 8],
    /// `plrmsgTicks` (static in plrmsg_delay)
    plrmsg_ticks: u32,
}

/// Original: `CountLinesOfText` (plrmsg.cpp).
// @port plrmsg.cpp|devilution::CountLinesOfText(string_view text) sha=70e1dcdd73d7
fn count_lines_of_text(text: &str) -> i32 {
    1 + text.bytes().filter(|&b| b == b'\n').count() as i32
}

/// Original: `GetNextMessage` (plrmsg.cpp): shifts older messages back and returns the front.
// @port plrmsg.cpp|devilution::GetNextMessage() sha=aa2b394a3aa3
fn get_next_message(ctx: &mut Ctx) -> &mut PlayerMessage {
    ctx.plrmsg.messages.rotate_right(1);
    &mut ctx.plrmsg.messages[0]
}

/// Original: `devilution::plrmsg_delay` (plrmsg.cpp).
// @port plrmsg.cpp|devilution::plrmsg_delay(bool delay) sha=72430cb43e94
pub fn plrmsg_delay(ctx: &mut Ctx, delay: bool) {
    if delay {
        ctx.plrmsg.plrmsg_ticks = 0u32.wrapping_sub(ctx.platform.ticks());
        return;
    }
    ctx.plrmsg.plrmsg_ticks = ctx.plrmsg.plrmsg_ticks.wrapping_add(ctx.platform.ticks());
    let t = ctx.plrmsg.plrmsg_ticks;
    for message in ctx.plrmsg.messages.iter_mut() {
        message.time = message.time.wrapping_add(t);
    }
}

/// `EventPlrMsg(text)` with the default style (`UiFlags::ColorWhitegold`).
pub fn event_plr_msg(ctx: &mut Ctx, text: &str) {
    event_plr_msg_style(ctx, text, UiFlags::COLOR_WHITEGOLD);
}

/// Original: `devilution::EventPlrMsg` (plrmsg.cpp).
// @port plrmsg.cpp|devilution::EventPlrMsg(string_view text, UiFlags style) sha=9fa87dce31d0
pub fn event_plr_msg_style(ctx: &mut Ctx, text: &str, style: UiFlags) {
    let time = ctx.platform.ticks();
    let line_height = get_line_height(ctx, text, GameFontTables::GameFont12) + 3;
    let message = get_next_message(ctx);
    message.style = style;
    message.time = time;
    message.text = text.to_string();
    message.from_len = 0;
    message.line_height = line_height;
    crate::qol::chatlog::add_message_to_chat_log(ctx, text, None, UiFlags::COLOR_WHITE);
}

/// Original: `devilution::SendPlrMsg` (plrmsg.cpp).
// @port plrmsg.cpp|devilution::SendPlrMsg(Player &player, string_view text) sha=b0213e354190
pub fn send_plr_msg(ctx: &mut Ctx, pnum: usize, text: &str) {
    let player = &ctx.players.Players[pnum];
    let from = tr("{:s} (lvl {:d}): ").replacen("{:s}", player._pName.as_str(), 1).replacen("{:d}", &player._pLevel.to_string(), 1);
    let full = format!("{from}{text}");
    let time = ctx.platform.ticks();
    let line_height = get_line_height(ctx, &full, GameFontTables::GameFont12) + 3;
    let message = get_next_message(ctx);
    message.style = UiFlags::COLOR_WHITE;
    message.time = time;
    message.text = full;
    message.from_len = from.len();
    message.line_height = line_height;
    crate::qol::chatlog::add_message_to_chat_log(ctx, text, Some(pnum), UiFlags::COLOR_WHITE);
}

/// Original: `devilution::InitPlrMsg` (plrmsg.cpp).
// @port plrmsg.cpp|devilution::InitPlrMsg() sha=8e40599be8a8
pub fn init_plr_msg(ctx: &mut Ctx) {
    ctx.plrmsg.messages = Default::default();
}

/// Original: `devilution::DrawPlrMsg` (plrmsg.cpp).
// @port plrmsg.cpp|devilution::DrawPlrMsg(const Surface &out) sha=7e960e23ccca
pub fn draw_plr_msg(ctx: &mut Ctx, out: &Surface) {
    if ctx.chatlog.ChatLogFlag {
        return;
    }
    let screen_width = ctx.dx.gn_screen_width;
    let mut x = 10;
    let mut y = crate::control::get_main_panel(ctx).y - 13;
    let mut width = screen_width - 20;
    let talkflag = ctx.control.talkflag;
    if !talkflag && crate::control::is_left_panel_open(ctx) {
        let l = crate::control::get_left_panel(ctx);
        x += l.x + l.w;
        width -= l.w;
    }
    if !talkflag && crate::control::is_right_panel_open(ctx) {
        width -= screen_width - crate::control::get_right_panel(ctx).x;
    }
    if width < 300 {
        return;
    }
    width = width.min(540);

    let now = ctx.platform.ticks();
    let messages = ctx.plrmsg.messages.clone();
    for message in messages.iter() {
        if message.text.is_empty() {
            break;
        }
        if !talkflag && now.wrapping_sub(message.time) >= 10000 {
            break;
        }
        let text = word_wrap_string(ctx, &message.text, width as u32, GameFontTables::GameFont12, 1);
        let chatlines = count_lines_of_text(&text);
        y -= message.line_height * chatlines;
        crate::engine::draw_half_transparent_rect_to(ctx, out, x - 3, y, width + 6, message.line_height * chatlines);
        draw_string(ctx, out, &text, Rect::new(x, y, width, 0), message.style, 1, message.line_height);
        draw_string(ctx, out, &message.text[..message.from_len], Rect::new(x, y, width, 0), UiFlags::COLOR_WHITEGOLD, 1, message.line_height);
    }
}
