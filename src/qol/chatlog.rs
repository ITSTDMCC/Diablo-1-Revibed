//! `Source/qol/chatlog.cpp`: the in-game chat log.

use crate::ctx::Ctx;
use crate::engine::render::text_render::{draw_string, draw_string_with_colors, DrawStringFormatArg, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::utils::language::tr;

/// `ColoredText`
#[derive(Clone, Debug, Default)]
struct ColoredText {
    text: String,
    color: UiFlags,
}

/// `MultiColoredText`
#[derive(Clone, Debug, Default)]
struct MultiColoredText {
    text: String,
    colors: Vec<ColoredText>,
    offset: i32,
}

/// Globals of chatlog.cpp.
#[derive(Default)]
pub struct ChatLogState {
    unread_flag: bool,
    skip_lines: u32,
    message_counter: u32,
    chat_log_lines: Vec<MultiColoredText>,
    pub ChatLogFlag: bool,
}

const PaddingTop: i32 = 32;
const PaddingLeft: i32 = 32;
const PanelHeight: i32 = 297;
const ContentTextWidth: i32 = 577;

/// Original: `LineHeight` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::LineHeight() sha=146a08880dcb
fn line_height(ctx: &Ctx) -> i32 {
    if crate::utils::language::is_small_font_tall(ctx) {
        18
    } else {
        14
    }
}

/// Original: `BlankLineHeight` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::BlankLineHeight() sha=7eb85db69677
fn blank_line_height() -> i32 {
    12
}

/// Original: `DividerLineMarginY` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::DividerLineMarginY() sha=759262e39b0f
fn divider_line_margin_y() -> i32 {
    blank_line_height() / 2
}

/// Original: `HeaderHeight` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::HeaderHeight() sha=ad90c7dcb33a
fn header_height(ctx: &Ctx) -> i32 {
    PaddingTop + line_height(ctx) + 2 * blank_line_height() + divider_line_margin_y()
}

/// Original: `ContentPaddingY` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::ContentPaddingY() sha=6326feeaccde
fn content_padding_y() -> i32 {
    blank_line_height()
}

/// Original: `ContentsTextHeight` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::ContentsTextHeight() sha=fe2e80b10f64
fn contents_text_height(ctx: &Ctx) -> i32 {
    PanelHeight - header_height(ctx) - divider_line_margin_y() - 2 * content_padding_y() - blank_line_height()
}

/// Original: `NumVisibleLines` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::NumVisibleLines() sha=b6aed970e150
fn num_visible_lines(ctx: &Ctx) -> i32 {
    (contents_text_height(ctx) - 1) / line_height(ctx) + 1
}

/// Original: `devilution::ToggleChatLog` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::ToggleChatLog() sha=383eb34ba398
pub fn toggle_chat_log(ctx: &mut Ctx) {
    if ctx.chatlog.ChatLogFlag {
        ctx.chatlog.ChatLogFlag = false;
    } else {
        crate::stores::set_stextflag_none(ctx);
        crate::inv::close_inventory(ctx);
        crate::control::close_char_panel(ctx);
        ctx.control.sbookflag = false;
        ctx.control.spselflag = false;
        if crate::minitext::qtextflag(ctx) && ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town {
            crate::minitext::set_qtextflag(ctx, false);
            crate::effects::stream_stop(ctx);
        }
        ctx.quests.QuestLogIsOpen = false;
        ctx.help.HelpFlag = false;
        crate::error::cancel_current_diablo_msg(ctx);
        crate::gamemenu::gamemenu_off(ctx);
        ctx.chatlog.skip_lines = 0;
        ctx.chatlog.ChatLogFlag = true;
        crate::doom::doom_close(ctx);
    }
}

/// `ChatLogFlag = false`
pub fn clear_chat_log_flag(ctx: &mut Ctx) {
    ctx.chatlog.ChatLogFlag = false;
}

/// Original: `devilution::AddMessageToChatLog` (qol/chatlog.cpp). `player` is a player index.
// @port qol/chatlog.cpp|devilution::AddMessageToChatLog(string_view message, Player *player, UiFlags flags) sha=aa17b5487a02
pub fn add_message_to_chat_log(ctx: &mut Ctx, message: &str, player: Option<usize>, flags: UiFlags) {
    ctx.chatlog.message_counter += 1;
    let counter = ctx.chatlog.message_counter;
    let timestamp = match crate::platform::win32::localtime_hms(ctx.platform.time()) {
        Some((h, m, s)) => format!("[#{counter}] {h:02}:{m:02}:{s:02}"),
        None => format!("[#{counter}] "),
    };
    let old_size = ctx.chatlog.chat_log_lines.len();
    ctx.chatlog.chat_log_lines.push(MultiColoredText { text: String::new(), colors: vec![ColoredText::default()], offset: 0 });
    match player {
        None => {
            ctx.chatlog.chat_log_lines.push(MultiColoredText {
                text: "{0} {1}".to_string(),
                colors: vec![ColoredText { text: timestamp, color: UiFlags::COLOR_RED }, ColoredText { text: message.to_string(), color: flags }],
                offset: 0,
            });
        }
        Some(p) => {
            let pl = &ctx.players.Players[p];
            let player_info = tr("{:s} (lvl {:d}): ").replacen("{:s}", pl._pName.as_str(), 1).replacen("{:d}", &pl._pLevel.to_string(), 1);
            ctx.chatlog.chat_log_lines.push(MultiColoredText { text: message.to_string(), colors: vec![ColoredText::default()], offset: 20 });
            let name_color = if ctx.players.MyPlayer == Some(p) { UiFlags::COLOR_WHITEGOLD } else { UiFlags::COLOR_BLUE };
            ctx.chatlog.chat_log_lines.push(MultiColoredText {
                text: "{0} - {1}".to_string(),
                colors: vec![ColoredText { text: timestamp, color: UiFlags::COLOR_RED }, ColoredText { text: player_info, color: name_color }],
                offset: 0,
            });
        }
    }
    let diff = (ctx.chatlog.chat_log_lines.len() - old_size) as u32;
    if ctx.chatlog.skip_lines != 0 {
        ctx.chatlog.skip_lines += diff;
        ctx.chatlog.unread_flag = true;
    }
}

/// Original: `devilution::DrawChatLog` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::DrawChatLog(const Surface &out) sha=6e6e5492b19f
pub fn draw_chat_log(ctx: &mut Ctx, out: &Surface) {
    crate::stores::draw_s_text_help(ctx);
    crate::minitext::draw_q_text_back(ctx, out);
    if ctx.chatlog.skip_lines == 0 {
        ctx.chatlog.unread_flag = false;
    }
    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let lh = line_height(ctx);
    let blh = blank_line_height();
    let sx = ui_position.x + PaddingLeft;
    let sy = ui_position.y;

    let title = tr("Chat History (Messages: {:d})").replacen("{:d}", &ctx.chatlog.message_counter.to_string(), 1);
    let color = if ctx.chatlog.unread_flag { UiFlags::COLOR_RED } else { UiFlags::COLOR_WHITEGOLD };
    draw_string(ctx, out, &title, Rect::new(sx, sy + PaddingTop + blh, ContentTextWidth, lh), color | UiFlags::ALIGN_CENTER, 1, -1);

    if let Some((h, m, s)) = crate::platform::win32::localtime_hms(ctx.platform.time()) {
        let timestamp = format!("{h:02}:{m:02}:{s:02}");
        draw_string(ctx, out, &timestamp, Rect::new(sx, sy + PaddingTop + blh, ContentTextWidth, lh), UiFlags::COLOR_WHITEGOLD, 1, -1);
    }

    let title_bottom = sy + header_height(ctx);
    crate::stores::draw_s_line(ctx, out, title_bottom);

    let num_lines = num_visible_lines(ctx);
    let content_y = title_bottom + divider_line_margin_y() + content_padding_y();
    let n = ctx.chatlog.chat_log_lines.len();
    for i in 0..num_lines {
        let idx = i as usize + ctx.chatlog.skip_lines as usize;
        if idx >= n {
            break;
        }
        let text = ctx.chatlog.chat_log_lines[n - (idx + 1)].clone();
        let mut args: Vec<DrawStringFormatArg> = text.colors.iter().map(|x| DrawStringFormatArg::string(&x.text, x.color)).collect();
        draw_string_with_colors(
            ctx,
            out,
            &text.text,
            &mut args,
            Rect::new(sx + text.offset, content_y + i * lh, ContentTextWidth - text.offset * 2, lh),
            UiFlags::COLOR_WHITE,
            1,
            lh,
        );
    }

    draw_string(
        ctx,
        out,
        &tr("Press ESC to end or the arrow keys to scroll."),
        Rect::new(sx, content_y + contents_text_height(ctx) + content_padding_y() + blh, ContentTextWidth, lh),
        UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER,
        1,
        -1,
    );
}

/// Original: `devilution::ChatLogScrollUp` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::ChatLogScrollUp() sha=ed435a3509f3
pub fn chat_log_scroll_up(ctx: &mut Ctx) {
    if ctx.chatlog.skip_lines > 0 {
        ctx.chatlog.skip_lines -= 1;
    }
}

/// Original: `devilution::ChatLogScrollDown` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::ChatLogScrollDown() sha=64010c62ffa7
pub fn chat_log_scroll_down(ctx: &mut Ctx) {
    if (ctx.chatlog.skip_lines as i64 + num_visible_lines(ctx) as i64) < ctx.chatlog.chat_log_lines.len() as i64 {
        ctx.chatlog.skip_lines += 1;
    }
}

/// Original: `devilution::ChatLogScrollTop` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::ChatLogScrollTop() sha=b41200a35836
pub fn chat_log_scroll_top(ctx: &mut Ctx) {
    ctx.chatlog.skip_lines = 0;
}

/// Original: `devilution::ChatLogScrollBottom` (qol/chatlog.cpp).
// @port qol/chatlog.cpp|devilution::ChatLogScrollBottom() sha=b4754d341626
pub fn chat_log_scroll_bottom(ctx: &mut Ctx) {
    ctx.chatlog.skip_lines = (ctx.chatlog.chat_log_lines.len() as u32).wrapping_sub(num_visible_lines(ctx) as u32);
}
