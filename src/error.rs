//! `Source/error.cpp`: in-game messages (the box in the middle of the screen).

use std::collections::VecDeque;

use crate::ctx::Ctx;
use crate::engine::render::text_render::{word_wrap_string, GameFontTables};
use crate::utils::language::tr;

/// `MAX_SEND_STR_LEN` (msg.h)
const MAX_SEND_STR_LEN: usize = 80;
const LINE_WIDTH: u32 = 418;
pub const LINE_HEIGHT: i32 = 12;

/// `MessageEntry`
struct MessageEntry {
    text: String,
    /// Duration in milliseconds
    #[allow(dead_code)] // read by DrawDiabloMsg (pending)
    duration: u32,
}

/// Globals of error.cpp.
pub struct ErrorState {
    diablo_messages: VecDeque<MessageEntry>,
    msg_start_time: u32,
    pub text_lines: Vec<String>,
    pub error_window_height: i32,
}

impl Default for ErrorState {
    fn default() -> Self {
        ErrorState { diablo_messages: VecDeque::new(), msg_start_time: 0, text_lines: Vec::new(), error_window_height: 54 }
    }
}

/// `MsgStrings`: maps from `diablo_message` to the message.
pub const MSG_STRINGS: [&str; NUM_MSG] = [
    "",
    "Game saved",
    "No multiplayer functions in demo",
    "Direct Sound Creation Failed",
    "Not available in shareware version",
    "Not enough space to save",
    "No Pause in town",
    "Copying to a hard disk is recommended",
    "Multiplayer sync problem",
    "No pause in multiplayer",
    "Loading...",
    "Saving...",
    "Some are weakened as one grows strong",
    "New strength is forged through destruction",
    "Those who defend seldom attack",
    "The sword of justice is swift and sharp",
    "While the spirit is vigilant the body thrives",
    "The powers of mana refocused renews",
    "Time cannot diminish the power of steel",
    "Magic is not always what it seems to be",
    "What once was opened now is closed",
    "Intensity comes at the cost of wisdom",
    "Arcane power brings destruction",
    "That which cannot be held cannot be harmed",
    "Crimson and Azure become as the sun",
    "Knowledge and wisdom at the cost of self",
    "Drink and be refreshed",
    "Wherever you go, there you are",
    "Energy comes at the cost of wisdom",
    "Riches abound when least expected",
    "Where avarice fails, patience gains reward",
    "Blessed by a benevolent companion!",
    "The hands of men may be guided by fate",
    "Strength is bolstered by heavenly faith",
    "The essence of life flows from within",
    "The way is made clear when viewed from above",
    "Salvation comes at the cost of wisdom",
    "Mysteries are revealed in the light of reason",
    "Those who are last may yet be first",
    "Generosity brings its own rewards",
    "You must be at least level 8 to use this.",
    "You must be at least level 13 to use this.",
    "You must be at least level 17 to use this.",
    "Arcane knowledge gained!",
    "That which does not kill you...",
    "Knowledge is power.",
    "Give and you shall receive.",
    "Some experience is gained by touch.",
    "There's no place like home.",
    "Spiritual energy is restored.",
    "You feel more agile.",
    "You feel stronger.",
    "You feel wiser.",
    "You feel refreshed.",
    "That which can break will.",
];
pub const NUM_MSG: usize = 55;

/// Original: `InitNextLines` (error.cpp).
// @port error.cpp|devilution::InitNextLines() sha=b8b6f0cc83ae
fn init_next_lines(ctx: &mut Ctx) {
    ctx.error.text_lines.clear();
    let front = ctx.error.diablo_messages.front().expect("message").text.clone();
    let paragraphs = word_wrap_string(ctx, &front, LINE_WIDTH, GameFontTables::GameFont12, 1);
    ctx.error.text_lines = paragraphs.split('\n').map(str::to_string).collect();
    ctx.error.error_window_height = 54.max(ctx.error.text_lines.len() as i32 * LINE_HEIGHT + 42);
}

/// Original: `devilution::InitDiabloMsg(diablo_message e, uint32_t duration = 3500)` (error.cpp).
// @port error.cpp|devilution::InitDiabloMsg(diablo_message e, uint32_t duration) sha=52ffe072d69c
pub fn init_diablo_msg_id(ctx: &mut Ctx, e: usize, duration: u32) {
    let msg = tr(MSG_STRINGS[e]);
    init_diablo_msg(ctx, &msg, duration);
}

/// Original: `devilution::InitDiabloMsg(string_view msg, uint32_t duration = 3500)` (error.cpp).
// @port error.cpp|devilution::InitDiabloMsg(string_view msg, uint32_t duration) sha=1e0a3e5c9b17
pub fn init_diablo_msg(ctx: &mut Ctx, msg: &str, duration: u32) {
    if ctx.error.diablo_messages.len() >= MAX_SEND_STR_LEN {
        return;
    }
    if ctx.error.diablo_messages.iter().any(|entry| entry.text == msg) {
        return;
    }
    ctx.error.diablo_messages.push_back(MessageEntry { text: msg.to_string(), duration });
    if ctx.error.diablo_messages.len() == 1 {
        init_next_lines(ctx);
        ctx.error.msg_start_time = ctx.platform.ticks();
    }
}

/// Original: `devilution::IsDiabloMsgAvailable` (error.cpp).
// @port error.cpp|devilution::IsDiabloMsgAvailable() sha=11996ccecad9
pub fn is_diablo_msg_available(ctx: &Ctx) -> bool {
    !ctx.error.diablo_messages.is_empty()
}

/// Original: `devilution::CancelCurrentDiabloMsg` (error.cpp).
// @port error.cpp|devilution::CancelCurrentDiabloMsg() sha=4c86e6f52f4d
pub fn cancel_current_diablo_msg(ctx: &mut Ctx) {
    if ctx.error.diablo_messages.pop_front().is_some() && !ctx.error.diablo_messages.is_empty() {
        init_next_lines(ctx);
        ctx.error.msg_start_time = ctx.platform.ticks();
    }
}

/// Original: `devilution::ClrDiabloMsg` (error.cpp).
// @port error.cpp|devilution::ClrDiabloMsg() sha=460743a08a5f
pub fn clr_diablo_msg(ctx: &mut Ctx) {
    ctx.error.diablo_messages.clear();
}

crate::pending_fn!(pub fn draw_diablo_msg(ctx: &mut Ctx, out: &crate::engine::surface::Surface), "error.cpp|devilution::DrawDiabloMsg(const Surface &out)");
