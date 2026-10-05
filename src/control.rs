//! `Source/control.cpp`: the control panel and side panels.

use crate::ctx::Ctx;
use crate::engine::surface::Rect;

/// `SidePanelSize` (control.h)
pub const SIDE_PANEL_SIZE: (i32, i32) = (320, 352);

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

crate::pending_fn!(pub fn display_spells_key_pressed(ctx: &mut Ctx), "diablo.cpp|devilution::DisplaySpellsKeyPressed()");

crate::pending_fn!(pub fn inventory_key_pressed(ctx: &mut Ctx), "diablo.cpp|devilution::InventoryKeyPressed()");

crate::pending_fn!(pub fn character_sheet_key_pressed(ctx: &mut Ctx), "diablo.cpp|devilution::CharacterSheetKeyPressed()");

crate::pending_fn!(pub fn quest_log_key_pressed(ctx: &mut Ctx), "diablo.cpp|devilution::QuestLogKeyPressed()");

crate::pending_fn!(pub fn spell_book_key_pressed(ctx: &mut Ctx), "diablo.cpp|devilution::SpellBookKeyPressed()");

crate::pending_fn!(pub fn diablo_hotkey_msg(ctx: &mut Ctx, msg: u32), "control.cpp|devilution::DiabloHotkeyMsg(uint32_t dwMsg)");

crate::pending_fn!(pub fn close_panels(ctx: &mut Ctx), "diablo.cpp|devilution::ClosePanels()");
