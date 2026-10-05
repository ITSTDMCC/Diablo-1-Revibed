//! `Source/help.cpp`: the in-game help text.

use crate::ctx::Ctx;
use crate::engine::render::clx_render::clx_draw;
use crate::engine::render::text_render::{draw_string, word_wrap_string, GameFontTables, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::utils::language::{is_small_font_tall, tr};

/// Globals of help.cpp.
#[derive(Default)]
pub struct HelpState {
    pub HelpFlag: bool,
    /// `SkipLines`
    skip_lines: u32,
    /// `HelpTextLines`
    help_text_lines: Vec<String>,
    /// `Initialized` (function-local static of `InitHelp`)
    initialized: bool,
}

/// `HelpFlag = v`
pub fn set_help_flag(ctx: &mut Ctx, v: bool) {
    ctx.help.HelpFlag = v;
}

/// `HelpText`
const HELP_TEXT: &[&str] = &[
    "$Keyboard Shortcuts:",
    "F1:    Open Help Screen",
    "Esc:   Display Main Menu",
    "Tab:   Display Auto-map",
    "Space: Hide all info screens",
    "S: Open Speedbook",
    "B: Open Spellbook",
    "I: Open Inventory screen",
    "C: Open Character screen",
    "Q: Open Quest log",
    "F: Reduce screen brightness",
    "G: Increase screen brightness",
    "Z: Zoom Game Screen",
    "+ / -: Zoom Automap",
    "1 - 8: Use Belt item",
    "F5, F6, F7, F8:     Set hotkey for skill or spell",
    "Shift + Left Mouse Button: Attack without moving",
    "Shift + Left Mouse Button (on character screen): Assign all stat points",
    "Shift + Left Mouse Button (on inventory): Move item to belt or equip/unequip item",
    "Shift + Left Mouse Button (on belt): Move item to inventory",
    "",
    "$Movement:",
    "If you hold the mouse button down while moving, the character will continue to move in that direction.",
    "",
    "$Combat:",
    "Holding down the shift key and then left-clicking allows the character to attack without moving.",
    "",
    "$Auto-map:",
    "To access the auto-map, click the 'MAP' button on the Information Bar or press 'TAB' on the keyboard. Zooming in and out of the map is done with the + and - keys. Scrolling the map uses the arrow keys.",
    "",
    "$Picking up Objects:",
    "Useable items that are small in size, such as potions or scrolls, are automatically placed in your 'belt' located at the top of the Interface bar . When an item is placed in the belt, a small number appears in that box. Items may be used by either pressing the corresponding number or right-clicking on the item.",
    "",
    "$Gold:",
    "You can select a specific amount of gold to drop by right-clicking on a pile of gold in your inventory.",
    "",
    "$Skills & Spells:",
    "You can access your list of skills and spells by left-clicking on the 'SPELLS' button in the interface bar. Memorized spells and those available through staffs are listed here. Left-clicking on the spell you wish to cast will ready the spell. A readied spell may be cast by simply right-clicking in the play area.",
    "",
    "$Using the Speedbook for Spells:",
    "Left-clicking on the 'readied spell' button will open the 'Speedbook' which allows you to select a skill or spell for immediate use. To use a readied skill or spell, simply right-click in the main play area.",
    "Shift + Left-clicking on the 'select current spell' button will clear the readied spell.",
    "",
    "$Setting Spell Hotkeys:",
    "You can assign up to four Hotkeys for skills, spells or scrolls. Start by opening the 'speedbook' as described in the section above. Press the F5, F6, F7 or F8 keys after highlighting the spell you wish to assign.",
    "",
    "$Spell Books:",
    "Reading more than one book increases your knowledge of that spell, allowing you to cast the spell more effectively.",
];

const PADDING_TOP: i32 = 32;
const PADDING_LEFT: i32 = 32;
const PANEL_HEIGHT: i32 = 297;
const CONTENT_TEXT_WIDTH: i32 = 565;

// @port help.cpp|devilution::LineHeight() sha=146a08880dcb
fn line_height(ctx: &Ctx) -> i32 {
    if is_small_font_tall(ctx) {
        18
    } else {
        14
    }
}

// @port help.cpp|devilution::BlankLineHeight() sha=7eb85db69677
fn blank_line_height() -> i32 {
    12
}

// @port help.cpp|devilution::DividerLineMarginY() sha=759262e39b0f
fn divider_line_margin_y() -> i32 {
    blank_line_height() / 2
}

// @port help.cpp|devilution::HeaderHeight() sha=ad90c7dcb33a
fn header_height(ctx: &Ctx) -> i32 {
    PADDING_TOP + line_height(ctx) + 2 * blank_line_height() + divider_line_margin_y()
}

// @port help.cpp|devilution::ContentPaddingY() sha=6326feeaccde
fn content_padding_y() -> i32 {
    blank_line_height()
}

// @port help.cpp|devilution::ContentsTextHeight() sha=fe2e80b10f64
fn contents_text_height(ctx: &Ctx) -> i32 {
    PANEL_HEIGHT - header_height(ctx) - divider_line_margin_y() - 2 * content_padding_y() - blank_line_height()
}

// @port help.cpp|devilution::NumVisibleLines() sha=b6aed970e150
fn num_visible_lines(ctx: &Ctx) -> i32 {
    (contents_text_height(ctx) - 1) / line_height(ctx) + 1 // Ceil
}

/// Original: `DrawHelpSlider` (help.cpp).
// @port help.cpp|devilution::DrawHelpSlider(const Surface &out) sha=513229b1901d
fn draw_help_slider(ctx: &Ctx, out: &Surface) {
    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let cels = ctx.info_box.p_s_text_slid_cels.as_ref().expect("pSTextSlidCels");
    let slider_x_pos = CONTENT_TEXT_WIDTH + ui_position.x + 36;
    let mut slider_start = ui_position.y + header_height(ctx) + line_height(ctx) + 3;
    let slider_end = ui_position.y + PADDING_TOP + PANEL_HEIGHT - 12;
    clx_draw(out, (slider_x_pos, slider_start), &cels.get(11));
    slider_start += 12;
    let mut slider_current = slider_start;
    while slider_current < slider_end {
        clx_draw(out, (slider_x_pos, slider_current), &cels.get(13));
        slider_current += 12;
    }
    clx_draw(out, (slider_x_pos, slider_current), &cels.get(10));
    // Subtract visible lines from the total number of lines to get the actual
    // scroll range
    let scroll_range = ctx.help.help_text_lines.len() as i32 - num_visible_lines(ctx);
    // Subtract the size of the arrow buttons to get the length of the interior
    // part of the slider
    let slider_length = slider_current - 12 - slider_start;
    clx_draw(out, (slider_x_pos, slider_start + ((ctx.help.skip_lines as i32 * slider_length) / scroll_range)), &cels.get(12));
}

/// Original: `devilution::InitHelp` (help.cpp).
// @port help.cpp|devilution::InitHelp() sha=8abbbc5576be
pub fn init_help(ctx: &mut Ctx) {
    if ctx.help.initialized {
        return;
    }
    ctx.help.HelpFlag = false;
    for text in HELP_TEXT.iter() {
        let paragraph = word_wrap_string(ctx, &tr(text), CONTENT_TEXT_WIDTH as u32, GameFontTables::GameFont12, 1);
        for line in paragraph.split('\n') {
            ctx.help.help_text_lines.push(line.to_string());
        }
    }
    ctx.help.initialized = true;
}

/// Original: `devilution::DrawHelp` (help.cpp).
// @port help.cpp|devilution::DrawHelp(const Surface &out) sha=e916cc368805
pub fn draw_help(ctx: &mut Ctx, out: &Surface) {
    crate::stores::draw_s_text_help(ctx);
    crate::minitext::draw_q_text_back(ctx, out);

    let lh = line_height(ctx);
    let blh = blank_line_height();
    let title = if ctx.init.gb_is_hellfire {
        if ctx.init.gb_is_spawn {
            tr("Shareware Hellfire Help")
        } else {
            tr("Hellfire Help")
        }
    } else if ctx.init.gb_is_spawn {
        tr("Shareware Diablo Help")
    } else {
        tr("Diablo Help")
    };

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let sx = ui_position.x + PADDING_LEFT;
    let sy = ui_position.y;
    draw_string(ctx, out, &title, Rect::new(sx, sy + PADDING_TOP + blh, CONTENT_TEXT_WIDTH, lh), UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, 1, -1);

    let title_bottom = sy + header_height(ctx);
    crate::stores::draw_s_line(ctx, out, title_bottom);

    let num_lines = num_visible_lines(ctx);
    let content_y = title_bottom + divider_line_margin_y() + content_padding_y();
    for i in 0..num_lines {
        let line = ctx.help.help_text_lines[(i as u32 + ctx.help.skip_lines) as usize].clone();
        if line.is_empty() {
            continue;
        }
        let mut offset = 0;
        let mut style = UiFlags::COLOR_WHITE;
        if line.starts_with('$') {
            offset = 1;
            style = UiFlags::COLOR_BLUE;
        }
        draw_string(ctx, out, &line[offset..], Rect::new(sx, content_y + i * lh, CONTENT_TEXT_WIDTH, lh), style, 1, lh);
    }

    draw_string(
        ctx,
        out,
        &tr("Press ESC to end or the arrow keys to scroll."),
        Rect::new(sx, content_y + contents_text_height(ctx) + content_padding_y() + blh, CONTENT_TEXT_WIDTH, lh),
        UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER,
        1,
        -1,
    );

    draw_help_slider(ctx, out);
}

/// Original: `devilution::DisplayHelp` (help.cpp).
// @port help.cpp|devilution::DisplayHelp() sha=fa98ebca9c3e
pub fn display_help(ctx: &mut Ctx) {
    ctx.help.skip_lines = 0;
    ctx.help.HelpFlag = true;
    ctx.chatlog.ChatLogFlag = false;
}

/// Original: `devilution::HelpScrollUp` (help.cpp).
// @port help.cpp|devilution::HelpScrollUp() sha=5a9a4f87d444
pub fn help_scroll_up(ctx: &mut Ctx) {
    if ctx.help.skip_lines > 0 {
        ctx.help.skip_lines -= 1;
    }
}

/// Original: `devilution::HelpScrollDown` (help.cpp).
// @port help.cpp|devilution::HelpScrollDown() sha=c5ed939a2772
pub fn help_scroll_down(ctx: &mut Ctx) {
    if ctx.help.skip_lines as usize + (num_visible_lines(ctx) as usize) < ctx.help.help_text_lines.len() {
        ctx.help.skip_lines += 1;
    }
}
