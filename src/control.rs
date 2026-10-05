//! `Source/control.cpp`: the control panel and side panels.

use crate::ctx::Ctx;
use crate::engine::backbuffer_state::{redraw_component, redraw_everything, PanelDrawComponent};
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::geometry::{Displacement, Point, Rectangle, Size};
use crate::engine::render::clx_render::clx_draw;
use crate::engine::render::text_render::{draw_string, draw_string_at, get_line_width, render_clx_sprite, GameFontTables, UiFlags};
use crate::engine::surface::{OwnedSurface, Rect, Surface};
use crate::enums::*;
use crate::platform::events::keys::*;
use crate::utils::language::{ngettext, pgettext, tr};

/// `MAX_SEND_STR_LEN` (msg.h)
const MAX_SEND_STR_LEN: usize = 80;

/// `SidePanelSize` (control.h)
pub const SIDE_PANEL_SIZE: (i32, i32) = (320, 352);

/// `InfoBoxTopLeft` (control.h)
pub const INFO_BOX_TOP_LEFT: crate::engine::geometry::Displacement = crate::engine::geometry::Displacement::new(177, 46);
/// `InfoBoxSize` (control.h)
pub const INFO_BOX_SIZE: crate::engine::geometry::Size = crate::engine::geometry::Size::new(288, 64);

#[derive(Default)]
pub struct ControlState {
    pub drop_gold_flag: bool,
    pub talkflag: bool,
    pub spselflag: bool,
    /// `MainPanel`, `LeftPanel`, `RightPanel`
    pub main_panel: Rect,
    pub left_panel: Rect,
    pub right_panel: Rect,
    /// `InfoString`
    pub info_string: String,
    /// `InfoColor`
    pub info_color: crate::engine::render::text_render::UiFlags,
    pub sbookflag: bool,
    pub chrflag: bool,
    pub panelflag: bool,
    /// `dropGoldValue`
    pub drop_gold_value: i32,
    /// `initialDropGoldIndex`
    pub initial_drop_gold_index: i8,
    /// `initialDropGoldValue`
    pub initial_drop_gold_value: i32,
    /// `sgbPlrTalkTbl`
    pub sgb_plr_talk_tbl: i32,
    /// `lvlbtndown`
    pub lvlbtndown: bool,
    /// `panbtndown`
    pub panbtndown: bool,
    /// `chrbtnactive`
    pub chrbtnactive: bool,
    /// `chrbtn`
    pub chrbtn: [bool; 4],
    /// `sbooktab`
    pub sbooktab: i32,
    /// `pBtmBuff`
    pub p_btm_buff: Option<OwnedSurface>,
    /// `pGBoxBuff`
    pub p_g_box_buff: Option<ClxSpriteList>,
    /// `pLifeBuff`
    p_life_buff: Option<OwnedSurface>,
    /// `pManaBuff`
    p_mana_buff: Option<OwnedSurface>,
    /// `talkButtons`
    talk_buttons: Option<ClxSpriteList>,
    /// `pDurIcons`
    p_dur_icons: Option<ClxSpriteList>,
    /// `multiButtons`
    multi_buttons: Option<ClxSpriteList>,
    /// `pPanelButtons`
    p_panel_buttons: Option<ClxSpriteList>,
    /// `PanelButtons`
    panel_buttons: [bool; 8],
    /// `PanelButtonIndex`
    panel_button_index: i32,
    /// `TalkSave` (NUL-terminated buffers of `MAX_SEND_STR_LEN` bytes in the original)
    talk_save: [String; 8],
    /// `TalkSaveIndex`
    talk_save_index: u8,
    /// `NextTalkSave`
    next_talk_save: u8,
    /// `TalkMessage`
    talk_message: String,
    /// `TalkButtonsDown`
    talk_buttons_down: [bool; 3],
    /// `WhisperList`
    whisper_list: [bool; crate::player::MAX_PLRS],
}

/// Original: `devilution::IsLeftPanelOpen` (control.cpp).
// @port control.cpp|devilution::IsLeftPanelOpen() sha=40740597782d
pub fn is_left_panel_open(ctx: &Ctx) -> bool {
    ctx.control.chrflag || ctx.quests.QuestLogIsOpen || ctx.stash.IsStashOpen
}

/// Original: `devilution::IsRightPanelOpen` (control.cpp).
// @port control.cpp|devilution::IsRightPanelOpen() sha=2aecc12b395b
pub fn is_right_panel_open(ctx: &Ctx) -> bool {
    ctx.inv.invflag || ctx.control.sbookflag
}

/// Original: `devilution::AddPanelString` (control.cpp).
// @port control.cpp|devilution::AddPanelString(string_view str) sha=5826f0478b09
// @port control.cpp|devilution::AddPanelString(std::string &&str) sha=c54db3f9b3a2
pub fn add_panel_string(ctx: &mut Ctx, s: &str) {
    if ctx.control.info_string.is_empty() {
        ctx.control.info_string = s.to_string();
    } else {
        ctx.control.info_string.push('\n');
        ctx.control.info_string.push_str(s);
    }
}

/// Original: `devilution::GetMainPanel` (control.cpp).
// @port control.cpp|devilution::GetMainPanel() sha=0c6770c24c5d
pub fn get_main_panel(ctx: &Ctx) -> Rect {
    ctx.control.main_panel
}

/// Original: `devilution::GetLeftPanel` (control.cpp).
// @port control.cpp|devilution::GetLeftPanel() sha=725b92e42ae4
pub fn get_left_panel(ctx: &Ctx) -> Rect {
    ctx.control.left_panel
}

/// Original: `devilution::GetRightPanel` (control.cpp).
// @port control.cpp|devilution::GetRightPanel() sha=5b7deb81ede7
pub fn get_right_panel(ctx: &Ctx) -> Rect {
    ctx.control.right_panel
}

/// Original: `devilution::CalculatePanelAreas` (control.cpp).
// @port control.cpp|devilution::CalculatePanelAreas() sha=d8704acf252e
pub fn calculate_panel_areas(ctx: &mut Ctx) {
    let (sw, sh) = (ctx.dx.gn_screen_width, ctx.dx.gn_screen_height);
    let main = Rect::new((sw - 640) / 2, sh - 128, 640, 128);
    let mut left = Rect::new(0, 0, SIDE_PANEL_SIZE.0, SIDE_PANEL_SIZE.1);
    let mut right = Rect::new(0, 0, SIDE_PANEL_SIZE.0, SIDE_PANEL_SIZE.1);
    let virtual_gamepad = ctx.controls.control_mode == crate::controls::ControlTypes::VirtualGamepad;
    if virtual_gamepad {
        left.x = sw / 2 - left.w;
    } else if sw - left.w - right.w > main.w {
        left.x = (sw - left.w - right.w - main.w) / 2;
    }
    left.y = (sh - left.h - main.h) / 2;
    if virtual_gamepad {
        right.x = sw / 2;
    } else {
        right.x = sw - right.w - left.x;
    }
    right.y = left.y;
    ctx.control.main_panel = main;
    ctx.control.left_panel = left;
    ctx.control.right_panel = right;
    ctx.dx.gn_viewport_height = sh;
    if sw <= main.w {
        // Part of the screen is fully obscured by the UI
        ctx.dx.gn_viewport_height -= main.h;
    }
}

crate::pending_fn!(pub fn quick_spell_hotkey(ctx: &mut Ctx, i: usize), "diablo.cpp|devilution::InitKeymapActions() QuickSpell lambda");

/// Original: `devilution::OpenCharPanel` (control.cpp).
// @port control.cpp|devilution::OpenCharPanel() sha=1737c78aa2aa
pub fn open_char_panel(ctx: &mut Ctx) {
    ctx.quests.QuestLogIsOpen = false;
    crate::qol::stash::close_gold_withdraw(ctx);
    crate::inv::close_stash(ctx);
    ctx.control.chrflag = true;
}

/// Original: `devilution::CloseCharPanel` (control.cpp).
// @port control.cpp|devilution::CloseCharPanel() sha=2678e67973cc
pub fn close_char_panel(ctx: &mut Ctx) {
    ctx.control.chrflag = false;
    if crate::player::is_inspecting_player(ctx) {
        ctx.players.InspectPlayer = ctx.players.MyPlayer;
        crate::engine::backbuffer_state::redraw_everything(ctx);
        crate::error::init_diablo_msg(ctx, &crate::utils::language::tr("Stopped inspecting players."), 3500);
    }
}

/// Original: `devilution::ToggleCharPanel` (control.cpp).
// @port control.cpp|devilution::ToggleCharPanel() sha=0cfde65ef72d
pub fn toggle_char_panel(ctx: &mut Ctx) {
    if ctx.control.chrflag {
        close_char_panel(ctx);
    } else {
        open_char_panel(ctx);
    }
}

/// Original: `devilution::GetPanelPosition` (control.cpp).
// @port control.cpp|devilution::GetPanelPosition(UiPanels panel, Point offset) sha=43bb3227f945
pub fn get_panel_position(ctx: &Ctx, panel: crate::enums::UiPanels, offset: crate::engine::geometry::Point) -> crate::engine::geometry::Point {
    use crate::enums::UiPanels;
    let r = match panel {
        UiPanels::Main => get_main_panel(ctx),
        UiPanels::Quest | UiPanels::Character | UiPanels::Stash => get_left_panel(ctx),
        UiPanels::Spell | UiPanels::Inventory => get_right_panel(ctx),
        _ => get_main_panel(ctx),
    };
    crate::engine::geometry::Point::new(r.x + offset.x, r.y + offset.y)
}

/// Original: `devilution::control_reset_talk` (control.cpp).
// @port control.cpp|devilution::control_reset_talk() sha=f568004b99f3
pub fn control_reset_talk(ctx: &mut Ctx) {
    ctx.control.talkflag = false;
    ctx.platform.stop_text_input();
    ctx.control.sgb_plr_talk_tbl = 0;
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

/// Original: `devilution::CloseGoldDrop` (control.cpp).
// @port control.cpp|devilution::CloseGoldDrop() sha=6b17ce552799
pub fn close_gold_drop(ctx: &mut Ctx) {
    if !ctx.control.drop_gold_flag {
        return;
    }
    ctx.control.drop_gold_flag = false;
    ctx.platform.stop_text_input();
}

/// Original: `devilution::CanPanelsCoverView` (control.h).
// @port control.h|devilution::CanPanelsCoverView() sha=eb369c6b977f
pub fn can_panels_cover_view(ctx: &Ctx) -> bool {
    let main_panel = get_main_panel(ctx);
    crate::utils::display::get_screen_width(ctx) <= main_panel.w && crate::utils::display::get_screen_height(ctx) <= SIDE_PANEL_SIZE.1 + main_panel.h
}

/// `IncrementAttributeButtonSize`
const INCREMENT_ATTRIBUTE_BUTTON_SIZE: Size = Size::new(41, 22);
/// `ChrBtnsRect`: maps from attribute_id to the rectangle on screen used for attribute increment buttons.
pub const CHR_BTNS_RECT: [Rectangle; 4] = [
    Rectangle::new(Point::new(137, 138), INCREMENT_ATTRIBUTE_BUTTON_SIZE),
    Rectangle::new(Point::new(137, 166), INCREMENT_ATTRIBUTE_BUTTON_SIZE),
    Rectangle::new(Point::new(137, 195), INCREMENT_ATTRIBUTE_BUTTON_SIZE),
    Rectangle::new(Point::new(137, 223), INCREMENT_ATTRIBUTE_BUTTON_SIZE),
];

/// `PanBtnPos`: positions of panel buttons (x, y, w, h).
pub const PAN_BTN_POS: [Rect; 8] = [
    Rect::new(9, 9, 71, 19),    // char button
    Rect::new(9, 35, 71, 19),   // quests button
    Rect::new(9, 75, 71, 19),   // map button
    Rect::new(9, 101, 71, 19),  // menu button
    Rect::new(560, 9, 71, 19),  // inv button
    Rect::new(560, 35, 71, 19), // spells button
    Rect::new(87, 91, 33, 32),  // chat button
    Rect::new(527, 91, 33, 32), // friendly fire button
];

const PANEL_BUTTON_CHARINFO: usize = 0;
const PANEL_BUTTON_QLOG: usize = 1;
const PANEL_BUTTON_AUTOMAP: usize = 2;
const PANEL_BUTTON_MAINMENU: usize = 3;
const PANEL_BUTTON_INVENTORY: usize = 4;
const PANEL_BUTTON_SPELLBOOK: usize = 5;
const PANEL_BUTTON_SENDMSG: usize = 6;
const PANEL_BUTTON_FRIENDLY: usize = 7;

/// `PanBtnHotKey`: maps from panel_button_id to hotkey name.
const PAN_BTN_HOT_KEY: [Option<&str>; 8] = [Some("'c'"), Some("'q'"), Some("Tab"), Some("Esc"), Some("'i'"), Some("'b'"), Some("Enter"), None];
/// `PanBtnStr`: maps from panel_button_id to panel button description.
const PAN_BTN_STR: [&str; 8] = ["Character Information", "Quests log", "Automap", "Main Menu", "Inventory", "Spell book", "Send Message", ""];

fn my_player(ctx: &Ctx) -> usize {
    ctx.players.MyPlayer.expect("MyPlayer")
}

fn mouse(ctx: &Ctx) -> Point {
    Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1)
}

fn main_pos(ctx: &Ctx) -> Point {
    let r = get_main_panel(ctx);
    Point::new(r.x, r.y)
}

fn xy(p: Point) -> (i32, i32) {
    (p.x, p.y)
}

/// Replaces the first `{...}` placeholder of a fmt-style format string.
fn fmt1(fmt: &str, arg: &str) -> String {
    match (fmt.find('{'), fmt.find('}')) {
        (Some(a), Some(b)) if b > a => format!("{}{}{}", &fmt[..a], arg, &fmt[b + 1..]),
        _ => fmt.to_string(),
    }
}

/// A view of `pBtmBuff`.
fn btm_buff(ctx: &mut Ctx) -> Surface {
    Surface::of(ctx.control.p_btm_buff.as_mut().expect("pBtmBuff"))
}

/// `pGBoxBuff`
pub fn p_g_box_buff(ctx: &Ctx) -> &ClxSpriteList {
    ctx.control.p_g_box_buff.as_ref().expect("pGBoxBuff")
}

/// Original: `DrawFlaskTop` (control.cpp): draws a section of the empty flask cel on top of
/// the panel to create the illusion of the flask getting empty.
// @port control.cpp|devilution::DrawFlaskTop(const Surface &out, Point position, const Surface &celBuf, int y0, int y1) sha=3737dfe14956
fn draw_flask_top(out: &Surface, position: Point, cel_buf: &Surface, y0: i32, y1: i32) {
    out.blit_from(cel_buf, Rect::new(0, y0, cel_buf.w(), y1 - y0), xy(position));
}

/// Original: `DrawFlask` (control.cpp): draws the dome of the flask that protrudes above the
/// panel top line.
// @port control.cpp|devilution::DrawFlask(const Surface &out, const Surface &celBuf, Point sourcePosition, Point targetPosition, int h) sha=7ea0228579cc
fn draw_flask(out: &Surface, cel_buf: &Surface, source_position: Point, target_position: Point, h: i32) {
    const FLASK_WIDTH: i32 = 59;
    out.blit_from_skip_color_index_zero(cel_buf, Rect::new(source_position.x, source_position.y, FLASK_WIDTH, h), xy(target_position));
}

/// Original: `DrawFlaskUpper` (control.cpp): draws the part of the life/mana flasks
/// protruding above the bottom panel.
// @port control.cpp|devilution::DrawFlaskUpper(const Surface &out, const Surface &sourceBuffer, int offset, int fillPer) sha=2095b405bcda
fn draw_flask_upper(ctx: &mut Ctx, out: &Surface, source_buffer: &Surface, offset: i32, fill_per: i32) {
    // clamping because this function only draws the top 12% of the flask display
    let empty_portion = (80 - fill_per).clamp(0, 11) + 2; // +2 to account for the frame being included in the sprite
    let main = main_pos(ctx);
    // Draw the empty part of the flask
    draw_flask(out, source_buffer, Point::new(13, 3), main + Displacement::new(offset, -13), empty_portion);
    if empty_portion < 13 {
        // Draw the filled part of the flask
        let btm = btm_buff(ctx);
        draw_flask(out, &btm, Point::new(offset, empty_portion + 3), main + Displacement::new(offset, -13 + empty_portion), 13 - empty_portion);
    }
}

/// Original: `DrawFlaskLower` (control.cpp): draws the part of the life/mana flasks inside
/// the bottom panel.
// @port control.cpp|devilution::DrawFlaskLower(const Surface &out, const Surface &sourceBuffer, int offset, int fillPer) sha=a0432337c447
fn draw_flask_lower(ctx: &mut Ctx, out: &Surface, source_buffer: &Surface, offset: i32, fill_per: i32) {
    let filled = fill_per.clamp(0, 69);
    let main = main_pos(ctx);
    if filled < 69 {
        draw_flask_top(out, main + Displacement::new(offset, 0), source_buffer, 16, 85 - filled);
    }
    // It appears that the panel defaults to having a filled flask and DrawFlaskTop only overlays the appropriate amount of empty space.
    // This draw might not be necessary?
    if filled > 0 {
        draw_panel_box(ctx, out, Rect::new(offset, 85 - filled, 88, filled), main + Displacement::new(offset, 69 - filled));
    }
}

/// Original: `SetButtonStateDown` (control.cpp).
// @port control.cpp|devilution::SetButtonStateDown(int btnId) sha=6daaf7d3a9e2
fn set_button_state_down(ctx: &mut Ctx, btn_id: usize) {
    ctx.control.panel_buttons[btn_id] = true;
    redraw_component(ctx, PanelDrawComponent::ControlButtons);
    ctx.control.panbtndown = true;
}

/// Original: `PrintInfo` (control.cpp).
// @port control.cpp|devilution::PrintInfo(const Surface &out) sha=6da84bc28e0f
fn print_info(ctx: &mut Ctx, out: &Surface) {
    if ctx.control.talkflag {
        return;
    }
    const SPACE: [i32; 5] = [18, 12, 6, 3, 0];
    let main = main_pos(ctx);
    let mut info_area = Rect::new(main.x + INFO_BOX_TOP_LEFT.delta_x, main.y + INFO_BOX_TOP_LEFT.delta_y, INFO_BOX_SIZE.width, INFO_BOX_SIZE.height);
    let new_line_count = ctx.control.info_string.matches('\n').count() as i32;
    let space_index = 4.min(new_line_count) as usize;
    let spacing = SPACE[space_index];
    let line_height = 12 + spacing;
    // Adjusting the line height to add spacing between lines
    // will also add additional space beneath the last line
    // which throws off the vertical centering
    info_area.y += spacing / 2;
    let text = ctx.control.info_string.clone();
    let flags = ctx.control.info_color | UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::KERNING_FIT_SPACING;
    draw_string(ctx, out, &text, info_area, flags, 2, line_height);
}

/// Original: `CapStatPointsToAdd` (control.cpp).
// @port control.cpp|devilution::CapStatPointsToAdd(int remainingStatPoints, const Player &player, CharacterAttribute attribute) sha=4c38c0b34b10
fn cap_stat_points_to_add(remaining_stat_points: i32, player: &crate::player::Player, attribute: CharacterAttribute) -> i32 {
    let points_to_reach_cap = player.get_maximum_attribute_value(attribute) - player.get_base_attribute_value(attribute);
    remaining_stat_points.min(points_to_reach_cap)
}

/// Original: `DrawDurIcon4Item` (control.cpp).
// @port control.cpp|devilution::DrawDurIcon4Item(const Surface &out, Item &pItem, int x, int c) sha=a8c548269f67
fn draw_dur_icon_4_item(ctx: &Ctx, out: &Surface, item: &crate::items::Item, x: i32, mut c: usize) -> i32 {
    const DURABILITY_THRESHOLD_GOLD: i32 = 5;
    const DURABILITY_THRESHOLD_RED: i32 = 2;
    if item.is_empty() {
        return x;
    }
    if item._iDurability as i32 > DURABILITY_THRESHOLD_GOLD {
        return x;
    }
    if c == 0 {
        c = match item._itype {
            ItemType::Sword => 1,
            ItemType::Axe => 5,
            ItemType::Bow => 6,
            ItemType::Mace => 4,
            ItemType::Staff => 7,
            _ => 0,
        };
    }
    let icons = ctx.control.p_dur_icons.as_ref().expect("pDurIcons");
    // Calculate how much of the icon should be gold and red
    let height = icons.get(c).height() as i32; // Height of durability icon CEL
    let mut partition = 0;
    if item._iDurability as i32 > DURABILITY_THRESHOLD_RED {
        let current = item._iDurability as i32 - DURABILITY_THRESHOLD_RED;
        partition = (height * current) / (DURABILITY_THRESHOLD_GOLD - DURABILITY_THRESHOLD_RED);
    }
    // Draw icon
    let y = -17 + get_main_panel(ctx).y;
    if partition > 0 {
        let stenciled_buffer = out.subregion_y(y - partition, partition);
        clx_draw(&stenciled_buffer, (x, partition), &icons.get(c + 8)); // Gold icon
    }
    if partition != height {
        let stenciled_buffer = out.subregion_y(y - height, height - partition);
        clx_draw(&stenciled_buffer, (x, height), &icons.get(c)); // Red icon
    }
    x - icons.get(c).height() as i32 - 8 // Add in spacing for the next durability icon
}

/// `TextCmdItem`
struct TextCmdItem {
    text: &'static str,
    description: &'static str,
    required_parameter: &'static str,
    action_proc: fn(&mut Ctx, &str) -> String,
}

/// `TextCmdList`
const TEXT_CMD_LIST: [TextCmdItem; 5] = [
    TextCmdItem { text: "/help", description: "Prints help overview or help for a specific command.", required_parameter: "[command]", action_proc: text_cmd_help },
    TextCmdItem { text: "/arena", description: "Enter a PvP Arena.", required_parameter: "<arena-number>", action_proc: text_cmd_arena },
    TextCmdItem { text: "/arenapot", description: "Gives Arena Potions.", required_parameter: "<number>", action_proc: text_cmd_arena_pot },
    TextCmdItem { text: "/inspect", description: "Inspects stats and equipment of another player.", required_parameter: "<player name>", action_proc: text_cmd_inspect },
    TextCmdItem { text: "/seedinfo", description: "Show seed infos for current level.", required_parameter: "", action_proc: text_cmd_level_seed },
];

/// C `atoi`: optional leading whitespace and sign, then decimal digits.
fn atoi(s: &str) -> i32 {
    let s = s.trim_start();
    let (neg, digits) = match s.as_bytes().first() {
        Some(b'-') => (true, &s[1..]),
        Some(b'+') => (false, &s[1..]),
        _ => (false, s),
    };
    let mut v: i32 = 0;
    for b in digits.bytes() {
        if !b.is_ascii_digit() {
            break;
        }
        v = v.wrapping_mul(10).wrapping_add((b - b'0') as i32);
    }
    if neg {
        v.wrapping_neg()
    } else {
        v
    }
}

/// Original: `TextCmdHelp` (control.cpp).
// @port control.cpp|devilution::TextCmdHelp(const string_view parameter) sha=f2d3873e6584
fn text_cmd_help(_ctx: &mut Ctx, parameter: &str) -> String {
    if parameter.is_empty() {
        let mut ret = tr("Available Commands:");
        for text_cmd in TEXT_CMD_LIST.iter() {
            ret.push(' ');
            ret.push_str(&tr(text_cmd.text));
        }
        return ret;
    }
    let Some(text_cmd_item) = TEXT_CMD_LIST.iter().find(|elem| elem.text == parameter) else {
        return format!("{}{}{}", tr("Command "), parameter, tr(" is unkown."));
    };
    if text_cmd_item.required_parameter.is_empty() {
        return format!("{}{}{}", tr("Description: "), tr(text_cmd_item.description), tr("\nParameters: No additional parameter needed."));
    }
    format!("{}{}{}{}", tr("Description: "), tr(text_cmd_item.description), tr("\nParameters: "), tr(text_cmd_item.required_parameter))
}

/// Original: `AppendArenaOverview` (control.cpp).
// @port control.cpp|devilution::AppendArenaOverview(std::string &ret) sha=db23a0b438b6
fn append_arena_overview(ret: &mut String) {
    for arena in SL_FIRST_ARENA..=SL_LAST {
        ret.push_str(&format!("\n{} ({})", arena - SL_FIRST_ARENA + 1, crate::levels::setmaps::QUEST_LEVEL_NAMES[arena as usize]));
    }
}

/// `DungeonTypeForArena`
const DUNGEON_TYPE_FOR_ARENA: [crate::levels::gendung::DungeonType; 3] = [
    crate::levels::gendung::DungeonType::Cathedral, // SL_ARENA_CHURCH
    crate::levels::gendung::DungeonType::Hell,      // SL_ARENA_HELL
    crate::levels::gendung::DungeonType::Hell,      // SL_ARENA_CIRCLE_OF_LIFE
];

/// Original: `TextCmdArena` (control.cpp).
// @port control.cpp|devilution::TextCmdArena(const string_view parameter) sha=b02ca3922ca7
fn text_cmd_arena(ctx: &mut Ctx, parameter: &str) -> String {
    let mut ret = String::new();
    if !ctx.init.gb_is_multiplayer {
        ret.push_str(&tr("Arenas are only supported in multiplayer."));
        return ret;
    }
    if parameter.is_empty() {
        ret.push_str(&tr("What arena do you want to visit?"));
        append_arena_overview(&mut ret);
        return ret;
    }
    let arena_number = atoi(parameter);
    let arena_level = (arena_number - 1 + SL_FIRST_ARENA as i32) as _setlevels;
    if arena_number < 0 || !crate::levels::gendung::is_arena_level(arena_level) {
        ret.push_str(&tr("Invalid arena-number. Valid numbers are:"));
        append_arena_overview(&mut ret);
        return ret;
    }
    let me = my_player(ctx);
    if !ctx.players.Players[me].is_on_level(0) && !ctx.players.Players[me].is_on_arena_level() {
        ret.push_str(&tr("To enter a arena, you need to be in town or another arena."));
        return ret;
    }
    ctx.gendung.setlvltype = DUNGEON_TYPE_FOR_ARENA[(arena_level - SL_FIRST_ARENA) as usize];
    crate::player::start_new_lvl(ctx, me, WM_DIABSETLVL, arena_level as i32);
    ret
}

/// Original: `TextCmdArenaPot` (control.cpp).
// @port control.cpp|devilution::TextCmdArenaPot(const string_view parameter) sha=de71b0980ef8
fn text_cmd_arena_pot(ctx: &mut Ctx, parameter: &str) -> String {
    let ret = String::new();
    if !ctx.init.gb_is_multiplayer {
        return tr("Arenas are only supported in multiplayer.");
    }
    let me = my_player(ctx);
    let mut pot_number = 1.max(atoi(parameter));
    while pot_number > 0 {
        let mut item = crate::items::Item::default();
        crate::items::initialize_item(ctx, &mut item, IDI_ARENAPOT);
        crate::items::generate_new_seed(ctx, &mut item);
        item.update_required_stats_cache_for_player(&ctx.players.Players[me]);
        if !crate::inv::auto_place_item_in_belt(ctx, me, &item, true) && !crate::inv::auto_place_item_in_inventory(ctx, me, &item, true) {
            break; // inventory is full
        }
        pot_number -= 1;
    }
    ret
}

/// Original: `TextCmdInspect` (control.cpp).
// @port control.cpp|devilution::TextCmdInspect(const string_view parameter) sha=f82dfec5122e
fn text_cmd_inspect(ctx: &mut Ctx, parameter: &str) -> String {
    let mut ret = String::new();
    if !ctx.init.gb_is_multiplayer {
        ret.push_str(&tr("Inspecting only supported in multiplayer."));
        return ret;
    }
    if parameter.is_empty() {
        ret.push_str(&tr("Stopped inspecting players."));
        ctx.players.InspectPlayer = ctx.players.MyPlayer;
        return ret;
    }
    let param = parameter.to_ascii_lowercase();
    for i in 0..ctx.players.Players.len() {
        let player_name = ctx.players.Players[i]._pName.as_str().to_ascii_lowercase();
        if player_name.contains(&param) {
            ctx.players.InspectPlayer = Some(i);
            ret.push_str(&tr("Inspecting player: "));
            ret.push_str(&ctx.players.Players[i]._pName.as_str().to_string());
            open_char_panel(ctx);
            if !ctx.control.sbookflag {
                ctx.inv.invflag = true;
            }
            redraw_everything(ctx);
            return ret;
        }
    }
    ret.push_str(&tr("No players found with such a name"));
    ret
}

/// Original: `IsQuestEnabled` (control.cpp).
// @port control.cpp|devilution::IsQuestEnabled(const Quest &quest) sha=e8d8c1c749de
fn is_quest_enabled(ctx: &Ctx, quest: &crate::quests::Quest) -> bool {
    let hf = ctx.init.gb_is_hellfire;
    let info = &ctx.multi.sgGameInitInfo;
    match quest._qidx {
        Q_FARMER => hf && info.bCowQuest == 0,
        Q_JERSEY => hf && info.bCowQuest != 0,
        Q_GIRL => hf && info.bTheoQuest != 0,
        Q_CORNSTN => hf && !ctx.init.gb_is_multiplayer,
        Q_GRAVE | Q_DEFILER | Q_NAKRUL => hf,
        Q_TRADER => false,
        _ => quest._qactive != QUEST_NOTAVAIL,
    }
}

/// Original: `TextCmdLevelSeed` (control.cpp), release build (no `_DEBUG` seed lines).
// @port control.cpp|devilution::TextCmdLevelSeed(const string_view parameter) sha=87e2683cd338
fn text_cmd_level_seed(ctx: &mut Ctx, _parameter: &str) -> String {
    let level_type = if ctx.gendung.setlevel { "set level" } else { "dungeon level" };
    let pid = ctx.multi.sgGameInitInfo.programid;
    let game_id: String = [(pid >> 24) & 0xFF, (pid >> 16) & 0xFF, (pid >> 8) & 0xFF, pid & 0xFF].iter().map(|&b| b as u8 as char).collect();
    let mode = if ctx.init.gb_is_multiplayer { "MP" } else { "SP" };
    let quest_pool = if crate::quests::use_multiplayer_quests(ctx) { "MP" } else { "Full" };
    let mut quest_flags: u32 = 0;
    for quest in ctx.quests.Quests.iter() {
        quest_flags <<= 1;
        if is_quest_enabled(ctx, quest) {
            quest_flags |= 1;
        }
    }
    let currlevel = ctx.gendung.currlevel as usize;
    format!(
        "Seedinfo for {} {}\nseed: {}\n\n{} {}\n{} quests: {}\nStorybook: {}",
        level_type, currlevel, ctx.diablo.glSeedTbl[currlevel], game_id, mode, quest_pool, quest_flags, ctx.diablo.glSeedTbl[16]
    )
}

/// Original: `CheckTextCommand` (control.cpp).
// @port control.cpp|devilution::CheckTextCommand(const string_view text) sha=0ebba843f89e
fn check_text_command(ctx: &mut Ctx, text: &str) -> bool {
    if text.is_empty() || !text.starts_with('/') {
        return false;
    }
    let found = TEXT_CMD_LIST.iter().find(|elem| text.starts_with(elem.text) && (text.len() == elem.text.len() || text.as_bytes()[elem.text.len()] == b' '));
    let Some(text_cmd) = found else {
        crate::error::init_diablo_msg(ctx, &format!("{}{}\" is unknown.", tr("Command \""), text), 3500);
        return true;
    };
    let mut parameter = "";
    if text.len() > text_cmd.text.len() + 1 {
        parameter = &text[text_cmd.text.len() + 1..];
    }
    let result = (text_cmd.action_proc)(ctx, parameter);
    if !result.is_empty() {
        crate::error::init_diablo_msg(ctx, &result, 3500);
    }
    true
}

/// Original: `ResetTalkMsg` (control.cpp), release build (no debug text commands).
// @port control.cpp|devilution::ResetTalkMsg() sha=489dc05b998d
fn reset_talk_msg(ctx: &mut Ctx) {
    let msg = ctx.control.talk_message.clone();
    if check_text_command(ctx, &msg) {
        return;
    }
    let mut pmask: u32 = 0;
    for i in 0..ctx.players.Players.len() {
        if ctx.control.whisper_list[i] {
            pmask |= 1 << i;
        }
    }
    crate::msg::net_send_cmd_string(ctx, pmask, &msg);
}

/// Original: `ControlPressEnter` (control.cpp).
// @port control.cpp|devilution::ControlPressEnter() sha=28f860bff741
fn control_press_enter(ctx: &mut Ctx) {
    if !ctx.control.talk_message.is_empty() {
        reset_talk_msg(ctx);
        let c = &mut ctx.control;
        let mut i: u8 = 0;
        while i < 8 {
            if c.talk_save[i as usize] == c.talk_message {
                break;
            }
            i += 1;
        }
        if i >= 8 {
            c.talk_save[c.next_talk_save as usize] = c.talk_message.clone();
            c.next_talk_save += 1;
            c.next_talk_save &= 7;
        } else {
            let talk_save = c.next_talk_save.wrapping_sub(1) & 7;
            if i != talk_save {
                c.talk_save[i as usize] = c.talk_save[talk_save as usize].clone();
                c.talk_save[talk_save as usize] = c.talk_message.clone();
            }
        }
        c.talk_message.clear();
        c.talk_save_index = c.next_talk_save;
    }
    control_reset_talk(ctx);
}

/// Original: `ControlUpDown` (control.cpp).
// @port control.cpp|devilution::ControlUpDown(int v) sha=211f0dbe968f
fn control_up_down(ctx: &mut Ctx, v: i32) {
    let c = &mut ctx.control;
    for _ in 0..8 {
        c.talk_save_index = ((v + c.talk_save_index as i32) & 7) as u8;
        if !c.talk_save[c.talk_save_index as usize].is_empty() {
            c.talk_message = c.talk_save[c.talk_save_index as usize].clone();
            return;
        }
    }
}

/// Original: `RemoveGold` (control.cpp).
// @port control.cpp|devilution::RemoveGold(Player &player, int goldIndex) sha=e398dbe4c4a0
fn remove_gold(ctx: &mut Ctx, pnum: usize, gold_index: i32) {
    let gi = (gold_index - INVITEM_INV_FIRST as i32) as usize;
    let drop_gold_value = ctx.control.drop_gold_value;
    ctx.players.Players[pnum].InvList[gi]._ivalue -= drop_gold_value;
    if ctx.players.Players[pnum].InvList[gi]._ivalue > 0 {
        crate::items::set_plr_hand_gold_curs(&mut ctx.players.Players[pnum].InvList[gi]);
        crate::msg::net_sync_inv_item(ctx, pnum, gi as i32);
    } else {
        crate::player::remove_inv_item(ctx, pnum, gi as i32, true);
    }
    let mut hold = std::mem::take(&mut ctx.players.Players[pnum].HoldItem);
    crate::items::make_gold_stack(ctx, &mut hold, drop_gold_value);
    let (e, curs) = (hold.is_empty(), hold._iCurs);
    ctx.players.Players[pnum].HoldItem = hold;
    crate::cursor::new_cursor_item(ctx, e, curs);
    ctx.players.Players[pnum]._pGold = crate::inv::calculate_gold(ctx, pnum);
    ctx.control.drop_gold_value = 0;
}

/// Original: `IsLevelUpButtonVisible` (control.cpp).
// @port control.cpp|devilution::IsLevelUpButtonVisible() sha=61fa5e553945
fn is_level_up_button_visible(ctx: &Ctx) -> bool {
    if ctx.control.spselflag || ctx.control.chrflag || ctx.players.Players[my_player(ctx)]._pStatPts == 0 {
        return false;
    }
    if ctx.controls.control_mode == crate::controls::ControlTypes::VirtualGamepad {
        return false;
    }
    if !crate::stores::stextflag_is_none(ctx) || ctx.stash.IsStashOpen {
        return false;
    }
    if ctx.quests.QuestLogIsOpen && get_left_panel(ctx).contains(main_pos(ctx) + Displacement::new(0, -74)) {
        return false;
    }
    true
}

/// Original: `devilution::IsChatAvailable` (control.cpp), release build.
// @port control.cpp|devilution::IsChatAvailable() sha=c853595be6b4
pub fn is_chat_available(ctx: &Ctx) -> bool {
    ctx.init.gb_is_multiplayer
}

/// Original: `devilution::FocusOnCharInfo` (control.cpp).
// @port control.cpp|devilution::FocusOnCharInfo() sha=b475b9c5db9c
pub fn focus_on_char_info(ctx: &mut Ctx) {
    let me = my_player(ctx);
    let my_player = &ctx.players.Players[me];
    if ctx.inv.invflag || my_player._pStatPts <= 0 {
        return;
    }
    // Find the first incrementable stat.
    let mut stat: i32 = -1;
    for (i, attribute) in [CharacterAttribute::Strength, CharacterAttribute::Magic, CharacterAttribute::Dexterity, CharacterAttribute::Vitality].into_iter().enumerate() {
        if my_player.get_base_attribute_value(attribute) >= my_player.get_maximum_attribute_value(attribute) {
            continue;
        }
        stat = i as i32;
    }
    if stat == -1 {
        return;
    }
    let c = CHR_BTNS_RECT[stat as usize].center();
    crate::controls::set_cursor_pos(ctx, (c.x, c.y));
}

/// Original: `devilution::DrawPanelBox` (control.cpp).
// @port control.cpp|devilution::DrawPanelBox(const Surface &out, SDL_Rect srcRect, Point targetPosition) sha=70fb9c0cd081
pub fn draw_panel_box(ctx: &mut Ctx, out: &Surface, src_rect: Rect, target_position: Point) {
    let btm = btm_buff(ctx);
    out.blit_from(&btm, src_rect, xy(target_position));
}

/// Original: `devilution::DrawLifeFlaskUpper` (control.cpp).
// @port control.cpp|devilution::DrawLifeFlaskUpper(const Surface &out) sha=02cc54cf15ef
pub fn draw_life_flask_upper(ctx: &mut Ctx, out: &Surface) {
    const LIFE_FLASK_UPPER_OFFSET: i32 = 109;
    let src = Surface::of(ctx.control.p_life_buff.as_mut().expect("pLifeBuff"));
    let per = ctx.players.Players[my_player(ctx)]._pHPPer;
    draw_flask_upper(ctx, out, &src, LIFE_FLASK_UPPER_OFFSET, per);
}

/// Original: `devilution::DrawManaFlaskUpper` (control.cpp).
// @port control.cpp|devilution::DrawManaFlaskUpper(const Surface &out) sha=5b75ecc9d3ae
pub fn draw_mana_flask_upper(ctx: &mut Ctx, out: &Surface) {
    const MANA_FLASK_UPPER_OFFSET: i32 = 475;
    let src = Surface::of(ctx.control.p_mana_buff.as_mut().expect("pManaBuff"));
    let per = ctx.players.Players[my_player(ctx)]._pManaPer;
    draw_flask_upper(ctx, out, &src, MANA_FLASK_UPPER_OFFSET, per);
}

/// Original: `devilution::DrawLifeFlaskLower` (control.cpp).
// @port control.cpp|devilution::DrawLifeFlaskLower(const Surface &out) sha=ac6f35b4bec7
pub fn draw_life_flask_lower(ctx: &mut Ctx, out: &Surface) {
    const LIFE_FLASK_LOWER_OFFSET: i32 = 96;
    let src = Surface::of(ctx.control.p_life_buff.as_mut().expect("pLifeBuff"));
    let per = ctx.players.Players[my_player(ctx)]._pHPPer;
    draw_flask_lower(ctx, out, &src, LIFE_FLASK_LOWER_OFFSET, per);
}

/// Original: `devilution::DrawManaFlaskLower` (control.cpp).
// @port control.cpp|devilution::DrawManaFlaskLower(const Surface &out) sha=9b372fec9172
pub fn draw_mana_flask_lower(ctx: &mut Ctx, out: &Surface) {
    const MANA_FLASK_LOWER_OFFESET: i32 = 464;
    let src = Surface::of(ctx.control.p_mana_buff.as_mut().expect("pManaBuff"));
    let per = ctx.players.Players[my_player(ctx)]._pManaPer;
    draw_flask_lower(ctx, out, &src, MANA_FLASK_LOWER_OFFESET, per);
}

/// Original: `devilution::DrawFlaskValues` (control.cpp).
// @port control.cpp|devilution::DrawFlaskValues(const Surface &out, Point pos, int currValue, int maxValue) sha=985512b583e0
pub fn draw_flask_values(ctx: &mut Ctx, out: &Surface, pos: Point, curr_value: i32, max_value: i32) {
    let color = if curr_value > 0 {
        if curr_value == max_value {
            UiFlags::COLOR_GOLD
        } else {
            UiFlags::COLOR_WHITE
        }
    } else {
        UiFlags::COLOR_RED
    };
    let mut draw_string_with_shadow = |ctx: &mut Ctx, text: &str, pos: Point| {
        draw_string_at(ctx, out, text, xy(pos + Displacement::new(-1, -1)), UiFlags::COLOR_BLACK | UiFlags::KERNING_FIT_SPACING, 0, -1);
        draw_string_at(ctx, out, text, xy(pos), color | UiFlags::KERNING_FIT_SPACING, 0, -1);
    };
    let curr_text = curr_value.to_string();
    let w = get_line_width(ctx, &curr_text, GameFontTables::GameFont12, 1, None);
    draw_string_with_shadow(ctx, &curr_text, pos - Displacement::new(w + 1, 0));
    draw_string_with_shadow(ctx, "/", pos);
    let slash_w = get_line_width(ctx, "/", GameFontTables::GameFont12, 1, None);
    draw_string_with_shadow(ctx, &max_value.to_string(), pos + Displacement::new(slash_w + 1, 0));
}

/// Original: `devilution::control_update_life_mana` (control.cpp).
// @port control.cpp|devilution::control_update_life_mana() sha=e550efa7ec72
pub fn control_update_life_mana(ctx: &mut Ctx) {
    let me = my_player(ctx);
    ctx.players.Players[me].update_mana_percentage();
    ctx.players.Players[me].update_hit_point_percentage();
}

/// Original: `devilution::InitControlPan` (control.cpp).
// @port control.cpp|devilution::InitControlPan() sha=97dcd0f325f3
pub fn init_control_pan(ctx: &mut Ctx) {
    let headless = ctx.diablo.headless_mode;
    let main_size = (get_main_panel(ctx).w, get_main_panel(ctx).h);
    if !headless {
        let chat = if is_chat_available(ctx) { 2 } else { 1 };
        ctx.control.p_btm_buff = Some(OwnedSurface::new(main_size.0, (main_size.1 + 16) * chat));
        ctx.control.p_mana_buff = Some(OwnedSurface::new(88, 88));
        ctx.control.p_life_buff = Some(OwnedSurface::new(88, 88));
        crate::panels::charpanel::load_char_panel(ctx);
        crate::panels::spell_icons::load_large_spell_icons(ctx);
        {
            let sprite = crate::engine::load_sprites::load_cel(ctx, "ctrlpan\\panel8", main_size.0 as u16);
            let btm = btm_buff(ctx);
            clx_draw(&btm, (0, (main_size.1 + 16) - 1), &sprite.get(0));
        }
        {
            let bulbs_position = (0, 87);
            let status_panel = crate::engine::load_sprites::load_cel(ctx, "ctrlpan\\p8bulbs", 88);
            let life = Surface::of(ctx.control.p_life_buff.as_mut().unwrap());
            clx_draw(&life, bulbs_position, &status_panel.get(0));
            let mana = Surface::of(ctx.control.p_mana_buff.as_mut().unwrap());
            clx_draw(&mana, bulbs_position, &status_panel.get(1));
        }
    }
    ctx.control.talkflag = false;
    if is_chat_available(ctx) {
        if !headless {
            {
                let sprite = crate::engine::load_sprites::load_cel(ctx, "ctrlpan\\talkpanl", main_size.0 as u16);
                let btm = btm_buff(ctx);
                clx_draw(&btm, (0, (main_size.1 + 16) * 2 - 1), &sprite.get(0));
            }
            ctx.control.multi_buttons = Some(crate::engine::load_sprites::load_cel(ctx, "ctrlpan\\p8but2", 33));
            ctx.control.talk_buttons = Some(crate::engine::load_sprites::load_cel(ctx, "ctrlpan\\talkbutt", 61));
        }
        ctx.control.sgb_plr_talk_tbl = 0;
        ctx.control.talk_message.clear();
        ctx.control.whisper_list = [true; crate::player::MAX_PLRS];
        ctx.control.talk_buttons_down = [false; 3];
    }
    ctx.control.panelflag = false;
    ctx.control.lvlbtndown = false;
    if !headless {
        crate::panels::mainpanel::load_main_panel(ctx);
        ctx.control.p_panel_buttons = Some(crate::engine::load_sprites::load_cel(ctx, "ctrlpan\\panel8bu", 71));
        const CHAR_BUTTONS_FRAME_WIDTHS: [u16; 9] = [95, 41, 41, 41, 41, 41, 41, 41, 41];
        ctx.panels.p_chr_buttons = Some(crate::engine::load_sprites::load_cel_widths(ctx, "data\\charbut", &CHAR_BUTTONS_FRAME_WIDTHS));
    }
    clear_pan_btn(ctx);
    ctx.control.panel_button_index = if !is_chat_available(ctx) { 6 } else { 8 };
    if !headless {
        ctx.control.p_dur_icons = Some(crate::engine::load_sprites::load_cel(ctx, "items\\duricons", 32));
    }
    ctx.control.chrbtn = [false; 4];
    ctx.control.chrbtnactive = false;
    ctx.control.info_string.clear();
    redraw_component(ctx, PanelDrawComponent::Health);
    redraw_component(ctx, PanelDrawComponent::Mana);
    close_char_panel(ctx);
    ctx.control.spselflag = false;
    ctx.control.sbooktab = 0;
    ctx.control.sbookflag = false;
    if !headless {
        crate::panels::spell_book::init_spell_book(ctx);
        ctx.quests.p_q_log_cel = Some(crate::engine::load_sprites::load_cel(ctx, "data\\quest", SIDE_PANEL_SIZE.0 as u16));
        ctx.control.p_g_box_buff = Some(crate::engine::load_sprites::load_cel(ctx, "ctrlpan\\golddrop", 261));
    }
    close_gold_drop(ctx);
    ctx.control.drop_gold_value = 0;
    ctx.control.initial_drop_gold_value = 0;
    ctx.control.initial_drop_gold_index = 0;
    calculate_panel_areas(ctx);
    if !headless {
        crate::controls::modifier_hints::init_modifier_hints(ctx);
    }
}

/// Original: `devilution::DrawCtrlPan` (control.cpp).
// @port control.cpp|devilution::DrawCtrlPan(const Surface &out) sha=4b66f7bb3bc2
pub fn draw_ctrl_pan(ctx: &mut Ctx, out: &Surface) {
    let main = get_main_panel(ctx);
    let tbl = ctx.control.sgb_plr_talk_tbl;
    draw_panel_box(ctx, out, Rect::new(0, tbl + 16, main.w, main.h), Point::new(main.x, main.y));
    draw_info_box(ctx, out);
}

/// Original: `devilution::DrawCtrlBtns` (control.cpp).
// @port control.cpp|devilution::DrawCtrlBtns(const Surface &out) sha=66e337f30f01
pub fn draw_ctrl_btns(ctx: &mut Ctx, out: &Surface) {
    let main_panel_position = main_pos(ctx);
    for i in 0..6 {
        let b = PAN_BTN_POS[i];
        if !ctx.control.panel_buttons[i] {
            draw_panel_box(ctx, out, Rect::new(b.x, b.y + 16, 71, 20), main_panel_position + Displacement::new(b.x, b.y));
        } else {
            let position = main_panel_position + Displacement::new(b.x, b.y + 18);
            clx_draw(out, xy(position), &ctx.control.p_panel_buttons.as_ref().expect("pPanelButtons").get(i));
            render_clx_sprite(out, &ctx.panels.panel_button_down.as_ref().expect("PanelButtonDown").get(i), xy(position + Displacement::new(4, -18)));
        }
    }
    if ctx.control.panel_button_index == 8 {
        let mb = ctx.control.multi_buttons.as_ref().expect("multiButtons");
        clx_draw(out, xy(main_panel_position + Displacement::new(87, 122)), &mb.get(if ctx.control.panel_buttons[6] { 1 } else { 0 }));
        if ctx.players.Players[my_player(ctx)].friendlyMode {
            clx_draw(out, xy(main_panel_position + Displacement::new(527, 122)), &mb.get(if ctx.control.panel_buttons[7] { 3 } else { 2 }));
        } else {
            clx_draw(out, xy(main_panel_position + Displacement::new(527, 122)), &mb.get(if ctx.control.panel_buttons[7] { 5 } else { 4 }));
        }
    }
}

/// Original: `devilution::ClearPanBtn` (control.cpp).
// @port control.cpp|devilution::ClearPanBtn() sha=54b40678d909
pub fn clear_pan_btn(ctx: &mut Ctx) {
    ctx.control.panel_buttons = [false; 8];
    redraw_component(ctx, PanelDrawComponent::ControlButtons);
    ctx.control.panbtndown = false;
}

/// Whether the mouse is inside `PanBtnPos[i]` (inclusive bounds, as the original compares).
fn mouse_in_pan_btn(ctx: &Ctx, i: usize) -> bool {
    let m = mouse(ctx);
    let p = main_pos(ctx);
    let b = PAN_BTN_POS[i];
    m.x >= b.x + p.x && m.x <= b.x + p.x + b.w && m.y >= b.y + p.y && m.y <= b.y + p.y + b.h
}

/// Whether the mouse is over the current spell button.
fn mouse_in_spell_button(ctx: &Ctx) -> bool {
    let m = mouse(ctx);
    let p = main_pos(ctx);
    m.x >= 565 + p.x && m.x < 621 + p.x && m.y >= 64 + p.y && m.y < 120 + p.y
}

/// Original: `devilution::DoPanBtn` (control.cpp).
// @port control.cpp|devilution::DoPanBtn() sha=55d4cd4fb645
pub fn do_pan_btn(ctx: &mut Ctx) {
    for i in 0..ctx.control.panel_button_index as usize {
        if mouse_in_pan_btn(ctx, i) {
            ctx.control.panel_buttons[i] = true;
            redraw_component(ctx, PanelDrawComponent::ControlButtons);
            ctx.control.panbtndown = true;
        }
    }
    if !ctx.control.spselflag && mouse_in_spell_button(ctx) {
        if (ctx.platform.mod_state() & crate::platform::events::KMOD_SHIFT) != 0 {
            let me = my_player(ctx);
            ctx.players.Players[me]._pRSpell = SpellID::Invalid;
            ctx.players.Players[me]._pRSplType = SpellType::Invalid;
            redraw_everything(ctx);
            return;
        }
        crate::panels::spell_list::do_speed_book(ctx);
        crate::gamemenu::gamemenu_off(ctx);
    }
}

/// Original: `devilution::control_check_btn_press` (control.cpp).
// @port control.cpp|devilution::control_check_btn_press() sha=b08f3de65372
pub fn control_check_btn_press(ctx: &mut Ctx) {
    if mouse_in_pan_btn(ctx, 3) {
        set_button_state_down(ctx, 3);
    }
    if mouse_in_pan_btn(ctx, 6) {
        set_button_state_down(ctx, 6);
    }
}

/// Original: `devilution::DoAutoMap` (control.cpp).
// @port control.cpp|devilution::DoAutoMap() sha=1664a7c740c3
pub fn do_auto_map(ctx: &mut Ctx) {
    if !crate::automap::automap_active(ctx) {
        crate::automap::start_automap(ctx);
    } else {
        crate::automap::set_automap_active(ctx, false);
    }
}

/// Original: `devilution::CheckPanelInfo` (control.cpp).
// @port control.cpp|devilution::CheckPanelInfo() sha=07f34fa32956
pub fn check_panel_info(ctx: &mut Ctx) {
    ctx.control.panelflag = false;
    ctx.control.info_string.clear();
    for i in 0..ctx.control.panel_button_index as usize {
        if mouse_in_pan_btn(ctx, i) {
            if i != 7 {
                ctx.control.info_string = tr(PAN_BTN_STR[i]);
            } else if ctx.players.Players[my_player(ctx)].friendlyMode {
                ctx.control.info_string = tr("Player friendly");
            } else {
                ctx.control.info_string = tr("Player attack");
            }
            if let Some(hk) = PAN_BTN_HOT_KEY[i] {
                add_panel_string(ctx, &fmt1(&tr("Hotkey: {:s}"), &tr(hk)));
            }
            ctx.control.info_color = UiFlags::COLOR_WHITE;
            ctx.control.panelflag = true;
        }
    }
    if !ctx.control.spselflag && mouse_in_spell_button(ctx) {
        ctx.control.info_string = tr("Select current spell button");
        ctx.control.info_color = UiFlags::COLOR_WHITE;
        ctx.control.panelflag = true;
        add_panel_string(ctx, &tr("Hotkey: 's'"));
        let me = my_player(ctx);
        let spell_id = ctx.players.Players[me]._pRSpell;
        if crate::spells::is_valid_spell(ctx, spell_id) {
            let name = pgettext("spell", crate::items::get_spell_data(spell_id).sNameText);
            match ctx.players.Players[me]._pRSplType {
                SpellType::Skill => add_panel_string(ctx, &fmt1(&tr("{:s} Skill"), &name)),
                SpellType::Spell => {
                    add_panel_string(ctx, &fmt1(&tr("{:s} Spell"), &name));
                    let spell_level = ctx.players.Players[me].get_spell_level(spell_id);
                    let s = if spell_level == 0 { tr("Spell Level 0 - Unusable") } else { fmt1(&tr("Spell Level {:d}"), &spell_level.to_string()) };
                    add_panel_string(ctx, &s);
                }
                SpellType::Scroll => {
                    add_panel_string(ctx, &fmt1(&tr("Scroll of {:s}"), &name));
                    let scroll_count = count_scrolls_of(ctx, me, spell_id);
                    add_panel_string(ctx, &fmt1(&ngettext("{:d} Scroll", "{:d} Scrolls", scroll_count), &scroll_count.to_string()));
                }
                SpellType::Charges => {
                    add_panel_string(ctx, &fmt1(&tr("Staff of {:s}"), &name));
                    let charges = ctx.players.Players[me].InvBody[INVLOC_HAND_LEFT as usize]._iCharges;
                    add_panel_string(ctx, &fmt1(&ngettext("{:d} Charge", "{:d} Charges", charges), &charges.to_string()));
                }
                SpellType::Invalid => {}
            }
        }
    }
    let m = mouse(ctx);
    let p = main_pos(ctx);
    if m.x > 190 + p.x && m.x < 437 + p.x && m.y > 4 + p.y && m.y < 33 + p.y {
        ctx.cursor.pcursinvitem = crate::inv::check_inv_h_light(ctx);
    }
    if crate::qol::xpbar::check_xp_bar_info(ctx) {
        ctx.control.panelflag = true;
    }
}

/// `std::count_if` over `InventoryAndBeltPlayerItemsRange` for `isScrollOf(spellId)`.
pub fn count_scrolls_of(ctx: &Ctx, pnum: usize, spell_id: SpellID) -> i32 {
    let p = &ctx.players.Players[pnum];
    let n = p._pNumInv as usize;
    p.InvList[..n].iter().chain(p.SpdList.iter()).filter(|item| !item.is_empty() && item.is_scroll_of(spell_id)).count() as i32
}

/// Original: `devilution::CheckBtnUp` (control.cpp).
// @port control.cpp|devilution::CheckBtnUp() sha=fd7c02cb94b7
pub fn check_btn_up(ctx: &mut Ctx) {
    let mut gamemenu_off = true;
    redraw_component(ctx, PanelDrawComponent::ControlButtons);
    ctx.control.panbtndown = false;
    for i in 0..8 {
        if !ctx.control.panel_buttons[i] {
            continue;
        }
        ctx.control.panel_buttons[i] = false;
        if !mouse_in_pan_btn(ctx, i) {
            continue;
        }
        match i {
            PANEL_BUTTON_CHARINFO => toggle_char_panel(ctx),
            PANEL_BUTTON_QLOG => {
                close_char_panel(ctx);
                crate::qol::stash::close_gold_withdraw(ctx);
                crate::inv::close_stash(ctx);
                if !ctx.quests.QuestLogIsOpen {
                    crate::quests::start_questlog(ctx);
                } else {
                    ctx.quests.QuestLogIsOpen = false;
                }
            }
            PANEL_BUTTON_AUTOMAP => do_auto_map(ctx),
            PANEL_BUTTON_MAINMENU => {
                crate::minitext::set_qtextflag(ctx, false);
                crate::gamemenu::gamemenu_handle_previous(ctx);
                gamemenu_off = false;
            }
            PANEL_BUTTON_INVENTORY => {
                ctx.control.sbookflag = false;
                crate::qol::stash::close_gold_withdraw(ctx);
                crate::inv::close_stash(ctx);
                ctx.inv.invflag = !ctx.inv.invflag;
                if ctx.control.drop_gold_flag {
                    close_gold_drop(ctx);
                    ctx.control.drop_gold_value = 0;
                }
            }
            PANEL_BUTTON_SPELLBOOK => {
                crate::inv::close_inventory(ctx);
                if ctx.control.drop_gold_flag {
                    close_gold_drop(ctx);
                    ctx.control.drop_gold_value = 0;
                }
                ctx.control.sbookflag = !ctx.control.sbookflag;
            }
            PANEL_BUTTON_SENDMSG => {
                if ctx.control.talkflag {
                    control_reset_talk(ctx);
                } else {
                    control_type_message(ctx);
                }
            }
            PANEL_BUTTON_FRIENDLY => {
                // Toggle friendly Mode
                crate::msg::net_send_cmd(ctx, true, CMD_FRIENDLYMODE);
            }
            _ => {}
        }
    }
    if gamemenu_off {
        crate::gamemenu::gamemenu_off(ctx);
    }
}

/// Original: `devilution::FreeControlPan` (control.cpp).
// @port control.cpp|devilution::FreeControlPan() sha=8b1335a963ad
pub fn free_control_pan(ctx: &mut Ctx) {
    ctx.control.p_btm_buff = None;
    ctx.control.p_mana_buff = None;
    ctx.control.p_life_buff = None;
    crate::panels::spell_icons::free_large_spell_icons(ctx);
    crate::panels::spell_book::free_spell_book(ctx);
    ctx.control.p_panel_buttons = None;
    ctx.control.multi_buttons = None;
    ctx.control.talk_buttons = None;
    ctx.panels.p_chr_buttons = None;
    ctx.control.p_dur_icons = None;
    ctx.quests.p_q_log_cel = None;
    ctx.control.p_g_box_buff = None;
    crate::panels::mainpanel::free_main_panel(ctx);
    crate::panels::charpanel::free_char_panel(ctx);
    crate::controls::modifier_hints::free_modifier_hints(ctx);
}

/// Original: `devilution::DrawInfoBox` (control.cpp).
// @port control.cpp|devilution::DrawInfoBox(const Surface &out) sha=330e3e8e91b6
pub fn draw_info_box(ctx: &mut Ctx, out: &Surface) {
    let main = main_pos(ctx);
    draw_panel_box(ctx, out, Rect::new(177, 62, INFO_BOX_SIZE.width, INFO_BOX_SIZE.height), main + INFO_BOX_TOP_LEFT);
    if !ctx.control.panelflag
        && !ctx.trigs.trigflag
        && ctx.cursor.pcursinvitem == -1
        && ctx.cursor.pcursstashitem == crate::qol::stash::StashStruct::EmptyCell
        && !ctx.control.spselflag
    {
        ctx.control.info_string.clear();
        ctx.control.info_color = UiFlags::COLOR_WHITE;
    }
    let me = my_player(ctx);
    if ctx.control.spselflag || ctx.trigs.trigflag {
        ctx.control.info_color = UiFlags::COLOR_WHITE;
    } else if !ctx.players.Players[me].HoldItem.is_empty() {
        let hold = ctx.players.Players[me].HoldItem.clone();
        if hold._itype == ItemType::Gold {
            let n_gold = hold._ivalue;
            ctx.control.info_string = fmt1(&ngettext("{:s} gold piece", "{:s} gold pieces", n_gold), &crate::utils::format_int::format_integer(n_gold));
        } else if !ctx.players.Players[me].can_use_item(&hold) {
            ctx.control.info_string = tr("Requirements not met");
        } else {
            ctx.control.info_string = hold.get_name(ctx);
            ctx.control.info_color = hold.get_text_color();
        }
    } else {
        if ctx.cursor.pcursitem != -1 {
            let item = ctx.items.Items[ctx.cursor.pcursitem as usize].clone();
            crate::items::get_item_str(ctx, &item);
        } else if let Some(obj) = ctx.cursor.ObjectUnderCursor {
            crate::objects::get_object_str(ctx, obj);
        }
        if ctx.cursor.pcursmonst != -1 {
            let m = ctx.cursor.pcursmonst as usize;
            if ctx.gendung.leveltype != crate::levels::gendung::DungeonType::Town {
                ctx.control.info_color = UiFlags::COLOR_WHITE;
                ctx.control.info_string = crate::monster::monster_name(ctx, m);
                if ctx.monster.Monsters[m].is_unique() {
                    ctx.control.info_color = UiFlags::COLOR_WHITEGOLD;
                    crate::monster::print_unique_history(ctx);
                } else {
                    let t = crate::monster::monster_type_id(ctx, m);
                    crate::monster::print_monst_history(ctx, t);
                }
            } else if ctx.cursor.pcursitem == -1 {
                ctx.control.info_string = ctx.towners.towners[m].name.clone();
            }
        }
        if ctx.cursor.pcursplr != -1 {
            ctx.control.info_color = UiFlags::COLOR_WHITEGOLD;
            let target = &ctx.players.Players[ctx.cursor.pcursplr as usize];
            let name = target._pName.as_str().to_string();
            let class_name = tr(crate::tables::playerdat::PlayersData[target._pClass as usize].className);
            let level = target._pLevel;
            let (hp, max_hp) = (target._pHitPoints >> 6, target._pMaxHP >> 6);
            ctx.control.info_string = name;
            add_panel_string(ctx, &fmt1(&fmt1(&tr("{:s}, Level: {:d}"), &class_name), &level.to_string()));
            add_panel_string(ctx, &fmt1(&fmt1(&tr("Hit Points {:d} of {:d}"), &hp.to_string()), &max_hp.to_string()));
        }
    }
    if !ctx.control.info_string.is_empty() {
        print_info(ctx, out);
    }
}

/// Whether the mouse is over the level up button.
fn mouse_in_lvl_btn(ctx: &Ctx) -> bool {
    let m = mouse(ctx);
    let p = main_pos(ctx);
    m.x >= 40 + p.x && m.x <= 81 + p.x && m.y >= -39 + p.y && m.y <= -17 + p.y
}

/// Original: `devilution::CheckLvlBtn` (control.cpp).
// @port control.cpp|devilution::CheckLvlBtn() sha=f193f9279ca8
pub fn check_lvl_btn(ctx: &mut Ctx) {
    if !is_level_up_button_visible(ctx) {
        return;
    }
    if !ctx.control.lvlbtndown && mouse_in_lvl_btn(ctx) {
        ctx.control.lvlbtndown = true;
    }
}

/// Original: `devilution::ReleaseLvlBtn` (control.cpp).
// @port control.cpp|devilution::ReleaseLvlBtn() sha=7aa07615d10c
pub fn release_lvl_btn(ctx: &mut Ctx) {
    if mouse_in_lvl_btn(ctx) {
        open_char_panel(ctx);
    }
    ctx.control.lvlbtndown = false;
}

/// Original: `devilution::DrawLevelUpIcon` (control.cpp).
// @port control.cpp|devilution::DrawLevelUpIcon(const Surface &out) sha=d660d5be15b0
pub fn draw_level_up_icon(ctx: &mut Ctx, out: &Surface) {
    if is_level_up_button_visible(ctx) {
        let n_cel = if ctx.control.lvlbtndown { 2 } else { 1 };
        let main = main_pos(ctx);
        draw_string(ctx, out, &tr("Level Up"), Rect::new(main.x, main.y - 62, 120, 0), UiFlags::COLOR_WHITE | UiFlags::ALIGN_CENTER, 1, -1);
        clx_draw(out, xy(main + Displacement::new(40, -17)), &ctx.panels.p_chr_buttons.as_ref().expect("pChrButtons").get(n_cel));
    }
}

const ATTRIBUTES: [CharacterAttribute; 4] = [CharacterAttribute::Strength, CharacterAttribute::Magic, CharacterAttribute::Dexterity, CharacterAttribute::Vitality];

/// Original: `devilution::CheckChrBtns` (control.cpp).
// @port control.cpp|devilution::CheckChrBtns() sha=7ac5105843f3
pub fn check_chr_btns(ctx: &mut Ctx) {
    let me = my_player(ctx);
    if ctx.control.chrbtnactive || ctx.players.Players[me]._pStatPts == 0 {
        return;
    }
    for attribute in ATTRIBUTES {
        let p = &ctx.players.Players[me];
        if p.get_base_attribute_value(attribute) >= p.get_maximum_attribute_value(attribute) {
            continue;
        }
        let button_id = attribute as usize;
        let mut button = CHR_BTNS_RECT[button_id];
        button.position = get_panel_position(ctx, UiPanels::Character, button.position);
        if button.contains(mouse(ctx)) {
            ctx.control.chrbtn[button_id] = true;
            ctx.control.chrbtnactive = true;
        }
    }
}

/// Original: `devilution::ReleaseChrBtns` (control.cpp).
// @port control.cpp|devilution::ReleaseChrBtns(bool addAllStatPoints) sha=f20bab8eae33
pub fn release_chr_btns(ctx: &mut Ctx, add_all_stat_points: bool) {
    ctx.control.chrbtnactive = false;
    for attribute in ATTRIBUTES {
        let button_id = attribute as usize;
        if !ctx.control.chrbtn[button_id] {
            continue;
        }
        ctx.control.chrbtn[button_id] = false;
        let mut button = CHR_BTNS_RECT[button_id];
        button.position = get_panel_position(ctx, UiPanels::Character, button.position);
        if button.contains(mouse(ctx)) {
            let me = my_player(ctx);
            let mut stat_points_to_add = 1;
            if add_all_stat_points {
                let p = &ctx.players.Players[me];
                stat_points_to_add = cap_stat_points_to_add(p._pStatPts, p, attribute);
            }
            let cmd = match attribute {
                CharacterAttribute::Strength => CMD_ADDSTR,
                CharacterAttribute::Magic => CMD_ADDMAG,
                CharacterAttribute::Dexterity => CMD_ADDDEX,
                CharacterAttribute::Vitality => CMD_ADDVIT,
            };
            crate::msg::net_send_cmd_param1(ctx, true, cmd, stat_points_to_add as u16);
            ctx.players.Players[me]._pStatPts -= stat_points_to_add;
        }
    }
}

/// Original: `devilution::DrawDurIcon` (control.cpp).
// @port control.cpp|devilution::DrawDurIcon(const Surface &out) sha=82e6eb2b608d
pub fn draw_dur_icon(ctx: &mut Ctx, out: &Surface) {
    let (main, left, right) = (get_main_panel(ctx), get_left_panel(ctx), get_right_panel(ctx));
    let has_room_between_panels = right.x - (left.x + left.w) >= 16 + (32 + 8 + 32 + 8 + 32 + 8 + 32) + 16;
    let has_room_under_panels = main.y - (right.y + right.h) >= 16 + 32 + 16;
    if !has_room_between_panels && !has_room_under_panels && is_left_panel_open(ctx) && is_right_panel_open(ctx) {
        return;
    }
    let mut x = main.x + main.w - 32 - 16;
    if !has_room_under_panels && is_right_panel_open(ctx) && main.x + main.w > right.x {
        x -= main.x + main.w - right.x;
    }
    let me = my_player(ctx);
    let body = ctx.players.Players[me].InvBody.clone();
    x = draw_dur_icon_4_item(ctx, out, &body[INVLOC_HEAD as usize], x, 3);
    x = draw_dur_icon_4_item(ctx, out, &body[INVLOC_CHEST as usize], x, 2);
    x = draw_dur_icon_4_item(ctx, out, &body[INVLOC_HAND_LEFT as usize], x, 0);
    draw_dur_icon_4_item(ctx, out, &body[INVLOC_HAND_RIGHT as usize], x, 0);
}

/// Original: `devilution::RedBack` (control.cpp).
// @port control.cpp|devilution::RedBack(const Surface &out) sha=71e795d56c54
pub fn red_back(ctx: &mut Ctx, out: &Surface) {
    let tbl = *crate::engine::trn::get_pause_trn(ctx);
    let hell = ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Hell;
    for y in 0..ctx.dx.gn_viewport_height {
        for x in 0..ctx.dx.gn_screen_width {
            let v = out.get(x, y);
            if !hell || v >= 32 {
                out.put(x, y, tbl[v as usize]);
            }
        }
    }
}

/// Original: `devilution::DrawGoldSplit` (control.cpp).
// @port control.cpp|devilution::DrawGoldSplit(const Surface &out, int amount) sha=0c809d36b9e7
pub fn draw_gold_split(ctx: &mut Ctx, out: &Surface, amount: i32) {
    const DIALOG_X: i32 = 30;
    let pos = get_panel_position(ctx, UiPanels::Inventory, Point::new(DIALOG_X, 178));
    clx_draw(out, xy(pos), &p_g_box_buff(ctx).get(0));
    let initial = ctx.control.initial_drop_gold_value;
    let description = fmt1(
        &ngettext("You have {:s} gold piece. How many do you want to remove?", "You have {:s} gold pieces. How many do you want to remove?", initial),
        &crate::utils::format_int::format_integer(initial),
    );
    // Pre-wrap the string at spaces, otherwise DrawString would hard wrap in the middle of words
    let wrapped = crate::engine::render::text_render::word_wrap_string(ctx, &description, 200, GameFontTables::GameFont12, 1);
    // The split gold dialog is roughly 4 lines high, but we need at least one line for the player to input an amount.
    // Using a clipping region 50 units high (approx 3 lines with a lineheight of 17) to ensure there is enough room left
    //  for the text entered by the player.
    let p = get_panel_position(ctx, UiPanels::Inventory, Point::new(DIALOG_X + 31, 75));
    draw_string(ctx, out, &wrapped, Rect::new(p.x, p.y, 200, 50), UiFlags::COLOR_WHITEGOLD | UiFlags::ALIGN_CENTER, 1, 17);
    let value = if amount > 0 { amount.to_string() } else { String::new() };
    // Even a ten digit amount of gold only takes up about half a line. There's no need to wrap or clip text here so we
    // use the Point form of DrawString.
    let p = get_panel_position(ctx, UiPanels::Inventory, Point::new(DIALOG_X + 37, 128));
    draw_string_at(ctx, out, &value, xy(p), UiFlags::COLOR_WHITE | UiFlags::PENTA_CURSOR, 1, -1);
}

/// Original: `devilution::control_drop_gold` (control.cpp).
// @port control.cpp|devilution::control_drop_gold(SDL_Keycode vkey) sha=ffdd56324bc3
pub fn control_drop_gold(ctx: &mut Ctx, vkey: i32) {
    let me = my_player(ctx);
    if ctx.players.Players[me]._pHitPoints >> 6 <= 0 {
        close_gold_drop(ctx);
        ctx.control.drop_gold_value = 0;
        return;
    }
    if vkey == SDLK_RETURN || vkey == SDLK_KP_ENTER {
        if ctx.control.drop_gold_value > 0 {
            let idx = ctx.control.initial_drop_gold_index as i32;
            remove_gold(ctx, me, idx);
        }
        close_gold_drop(ctx);
    } else if vkey == SDLK_ESCAPE {
        close_gold_drop(ctx);
        ctx.control.drop_gold_value = 0;
    } else if vkey == SDLK_BACKSPACE {
        ctx.control.drop_gold_value /= 10;
    }
}

/// Original: `devilution::DrawTalkPan` (control.cpp).
// @port control.cpp|devilution::DrawTalkPan(const Surface &out) sha=f07b00675054
pub fn draw_talk_pan(ctx: &mut Ctx, out: &Surface) {
    if !ctx.control.talkflag {
        return;
    }
    let mp = main_pos(ctx);
    let tbl = ctx.control.sgb_plr_talk_tbl;
    draw_panel_box(ctx, out, Rect::new(175, tbl + 20, 294, 5), mp + Displacement::new(175, 4));
    let mut off = 0;
    let mut i = 293;
    while i > 283 {
        draw_panel_box(ctx, out, Rect::new((off / 2) + 175, tbl + off + 25, i, 1), mp + Displacement::new((off / 2) + 175, off + 9));
        off += 1;
        i -= 1;
    }
    draw_panel_box(ctx, out, Rect::new(185, tbl + 35, 274, 30), mp + Displacement::new(185, 19));
    draw_panel_box(ctx, out, Rect::new(180, tbl + 65, 284, 5), mp + Displacement::new(180, 49));
    for i in 0..10 {
        draw_panel_box(ctx, out, Rect::new(180, tbl + i + 70, i + 284, 1), mp + Displacement::new(180, i + 54));
    }
    draw_panel_box(ctx, out, Rect::new(170, tbl + 80, 310, 55), mp + Displacement::new(170, 64));
    let mut x = mp.x + 200;
    let y = mp.y + 10;
    let msg = ctx.control.talk_message.clone();
    let len = draw_string(ctx, out, &msg, Rect::new(x, y, 250, 39), UiFlags::COLOR_WHITE | UiFlags::PENTA_CURSOR, 1, 13) as usize;
    let mut cut = len.min(MAX_SEND_STR_LEN - 1).min(ctx.control.talk_message.len());
    while !ctx.control.talk_message.is_char_boundary(cut) {
        cut -= 1;
    }
    ctx.control.talk_message.truncate(cut);
    x += 46;
    let mut talk_btn = 0usize;
    let me = my_player(ctx);
    for i in 0..ctx.players.Players.len() {
        if i == me {
            continue;
        }
        let color = if ctx.players.Players[i].friendlyMode { UiFlags::COLOR_WHITEGOLD } else { UiFlags::COLOR_RED };
        let talk_pan_position = mp + Displacement::new(172, 84 + 18 * talk_btn as i32);
        let down = ctx.control.talk_buttons_down[talk_btn];
        let tb = ctx.control.talk_buttons.as_ref().expect("talkButtons");
        let tbt = ctx.panels.talk_button.as_ref().expect("TalkButton");
        if ctx.control.whisper_list[i] {
            // the normal (unpressed) voice button is pre-rendered on the panel, only need to draw over it when the button is held
            if down {
                let sprite_index = if talk_btn == 0 { 2 } else { 3 }; // the first button sprite includes a tip from the devils wing so is different to the rest.
                clx_draw(out, xy(talk_pan_position), &tb.get(sprite_index));
                // Draw the translated string over the top of the default (english) button. This graphic is inset to avoid overlapping the wingtip, letting
                // the first button be treated the same as the other two further down the panel.
                render_clx_sprite(out, &tbt.get(2), xy(talk_pan_position + Displacement::new(4, -15)));
            }
        } else {
            let mut sprite_index = if talk_btn == 0 { 0 } else { 1 }; // the first button sprite includes a tip from the devils wing so is different to the rest.
            if down {
                sprite_index += 4; // held button sprites are at index 4 and 5 (with and without wingtip respectively)
            }
            clx_draw(out, xy(talk_pan_position), &tb.get(sprite_index));
            // Draw the translated string over the top of the default (english) button. This graphic is inset to avoid overlapping the wingtip, letting
            // the first button be treated the same as the other two further down the panel.
            render_clx_sprite(out, &tbt.get(if down { 1 } else { 0 }), xy(talk_pan_position + Displacement::new(4, -15)));
        }
        if ctx.players.Players[i].plractive {
            let name = ctx.players.Players[i]._pName.as_str().to_string();
            draw_string(ctx, out, &name, Rect::new(x, y + 60 + talk_btn as i32 * 18, 204, 0), color, 1, -1);
        }
        talk_btn += 1;
    }
}

/// Whether the mouse is over the whisper buttons column.
fn mouse_in_talk_btns(ctx: &Ctx) -> bool {
    let m = mouse(ctx);
    let p = main_pos(ctx);
    !(m.x < 172 + p.x || m.y < 69 + p.y || m.x > 233 + p.x || m.y > 123 + p.y)
}

/// Original: `devilution::control_check_talk_btn` (control.cpp).
// @port control.cpp|devilution::control_check_talk_btn() sha=af9ef8fb11c5
pub fn control_check_talk_btn(ctx: &mut Ctx) -> bool {
    if !ctx.control.talkflag {
        return false;
    }
    if !mouse_in_talk_btns(ctx) {
        return false;
    }
    ctx.control.talk_buttons_down = [false; 3];
    let idx = ((mouse(ctx).y - (69 + main_pos(ctx).y)) / 18) as usize;
    ctx.control.talk_buttons_down[idx] = true;
    true
}

/// Original: `devilution::control_release_talk_btn` (control.cpp).
// @port control.cpp|devilution::control_release_talk_btn() sha=5ad79f2abfb1
pub fn control_release_talk_btn(ctx: &mut Ctx) {
    if !ctx.control.talkflag {
        return;
    }
    ctx.control.talk_buttons_down = [false; 3];
    if !mouse_in_talk_btns(ctx) {
        return;
    }
    let mut off = (mouse(ctx).y - (69 + main_pos(ctx).y)) / 18;
    let mut player_id = 0usize;
    while player_id < ctx.players.Players.len() && off != -1 {
        if player_id != ctx.players.MyPlayerId {
            off -= 1;
        }
        player_id += 1;
    }
    if player_id > 0 && player_id <= ctx.players.Players.len() {
        ctx.control.whisper_list[player_id - 1] = !ctx.control.whisper_list[player_id - 1];
    }
}

/// Original: `devilution::control_type_message` (control.cpp).
// @port control.cpp|devilution::control_type_message() sha=6a6e07b1193a
pub fn control_type_message(ctx: &mut Ctx) {
    if !is_chat_available(ctx) {
        return;
    }
    ctx.control.talkflag = true;
    let main = get_main_panel(ctx);
    ctx.platform.text_input_rect = (main.x + 200, main.y + 22, 0, 27);
    ctx.control.talk_message.clear();
    ctx.control.talk_buttons_down = [false; 3];
    ctx.control.sgb_plr_talk_tbl = main.h + 16;
    redraw_everything(ctx);
    ctx.control.talk_save_index = ctx.control.next_talk_save;
    ctx.platform.start_text_input();
}

/// Original: `devilution::IsTalkActive` (control.cpp).
// @port control.cpp|devilution::IsTalkActive() sha=f938d8395195
pub fn is_talk_active(ctx: &Ctx) -> bool {
    if !is_chat_available(ctx) {
        return false;
    }
    ctx.control.talkflag
}

/// Original: `devilution::control_new_text` (control.cpp): `strncat` into the
/// `MAX_SEND_STR_LEN` buffer (cut at a character boundary here).
// @port control.cpp|devilution::control_new_text(string_view text) sha=fab4d7d947d9
pub fn control_new_text(ctx: &mut Ctx, text: &str) {
    let room = MAX_SEND_STR_LEN - 1 - ctx.control.talk_message.len().min(MAX_SEND_STR_LEN - 1);
    let mut cut = room.min(text.len());
    while !text.is_char_boundary(cut) {
        cut -= 1;
    }
    ctx.control.talk_message.push_str(&text[..cut]);
}

/// Original: `devilution::control_presskeys` (control.cpp).
// @port control.cpp|devilution::control_presskeys(SDL_Keycode vkey) sha=0d1c6d78d111
pub fn control_presskeys(ctx: &mut Ctx, vkey: i32) -> bool {
    if !is_chat_available(ctx) {
        return false;
    }
    if !ctx.control.talkflag {
        return false;
    }
    match vkey {
        SDLK_ESCAPE => {
            control_reset_talk(ctx);
            true
        }
        SDLK_RETURN | SDLK_KP_ENTER => {
            control_press_enter(ctx);
            true
        }
        SDLK_BACKSPACE => {
            // FindLastUtf8Symbols: remove the last code point
            ctx.control.talk_message.pop();
            true
        }
        SDLK_DOWN => {
            control_up_down(ctx, 1);
            true
        }
        SDLK_UP => {
            control_up_down(ctx, -1);
            true
        }
        _ => vkey >= SDLK_SPACE && vkey <= b'z' as i32,
    }
}

/// Original: `devilution::DiabloHotkeyMsg` (control.cpp), release build.
// @port control.cpp|devilution::DiabloHotkeyMsg(uint32_t dwMsg) sha=dbe5e9fc028d
pub fn diablo_hotkey_msg(ctx: &mut Ctx, dw_msg: u32) {
    if !is_chat_available(ctx) {
        return;
    }
    assert!((dw_msg as usize) < crate::options::QUICK_MESSAGE_OPTIONS);
    let msgs = ctx.options.chat.sz_hot_key_msgs[dw_msg as usize].clone();
    for msg in msgs.iter() {
        if check_text_command(ctx, msg) {
            continue;
        }
        let char_msg = crate::utils::utf8::copy_utf8(msg, MAX_SEND_STR_LEN);
        crate::msg::net_send_cmd_string(ctx, 0xFFFFFF, &char_msg);
    }
}

/// Original: `devilution::GoldDropNewText` (control.cpp).
// @port control.cpp|devilution::GoldDropNewText(string_view text) sha=b811263329bf
pub fn gold_drop_new_text(ctx: &mut Ctx, text: &str) {
    for vkey in text.bytes() {
        let digit = vkey as i8 as i32 - b'0' as i32;
        if (0..=9).contains(&digit) {
            let mut new_gold_value = ctx.control.drop_gold_value.wrapping_mul(10);
            new_gold_value += digit;
            if new_gold_value <= ctx.control.initial_drop_gold_value {
                ctx.control.drop_gold_value = new_gold_value;
            }
        }
    }
}
