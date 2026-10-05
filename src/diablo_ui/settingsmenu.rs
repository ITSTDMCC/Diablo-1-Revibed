//! `Source/DiabloUI/settingsmenu.cpp`: the settings menu.
//!
//! The original keeps `OptionCategoryBase *` / `OptionEntryBase *` pointers; the port refers to
//! categories by their index in `Options::GetCategories()` order and to entries with `OptRef`.

use std::cell::RefCell;
use std::rc::Rc;

use crate::controls::controller_buttons::{ControllerButton, ControllerButtonCombo};
use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{ui_add_background, ui_add_logo, ui_clear_screen, ui_init_list, ui_init_list_clear, ui_load_black_background, ui_poll_and_render};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiListItem, UiListItemRef};
use crate::engine::render::text_render::{get_line_width_fmt, word_wrap_string, DrawStringFormatArg, GameFontTables, UiFlags};
use crate::engine::surface::Rect;
use crate::options::{flags as of, OptionEntry, OptionEntryType, CATEGORY_ORDER};
use crate::platform::events::Event;
use crate::utils::language::tr;

const INDEX_KEY_OR_PAD_INPUT: usize = 1;
const INDEX_PAD_TIMER_TEXT: usize = 2;

/// `ShownMenuType`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum ShownMenuType {
    #[default]
    Categories,
    Settings,
    ListOption,
    KeyInput,
    PadInput,
}

/// `SpecialMenuEntry`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
enum SpecialMenuEntry {
    None = -1,
    PreviousMenu = -2,
    UnbindKey = -3,
    BindPadButton = -4,
    UnbindPadButton = -5,
}

/// An `OptionEntryBase *`: a plain entry (category index, entry index), or a key/pad mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OptRef {
    Plain(usize, usize),
    Key(usize),
    Pad(usize),
}

/// The globals of settingsmenu.cpp.
#[derive(Default)]
pub struct SettingsMenuState {
    end_menu: bool,
    back_to_main: bool,
    vec_dialog_items: Vec<UiListItemRef>,
    vec_dialog: Vec<UiItemRef>,
    vec_options: Vec<OptRef>,
    /// `selectedCategory` (index into `Options::GetCategories()`)
    selected_category: Option<usize>,
    selected_option: Option<OptRef>,
    shown_menu: ShownMenuType,
    /// `optionDescription`
    option_description: Rc<RefCell<String>>,
    /// `rectList`
    rect_list: Rect,
    /// `rectDescription`
    rect_description: Rect,
    pad_entry_combo: ControllerButtonCombo,
    pad_entry_start_time: u32,
    pad_entry_timer_text: String,
}

fn st(ctx: &mut Ctx) -> &mut SettingsMenuState {
    &mut ctx.diablo_ui.settingsmenu
}

// ---------------------------------------------------------------------------------------------
// `OptionEntryBase` / `OptionCategoryBase` access

fn with_plain<R>(ctx: &mut Ctx, cat: usize, i: usize, f: impl FnOnce(&mut dyn OptionEntry) -> R) -> R {
    let mut entries = ctx.options.entries_of(CATEGORY_ORDER[cat]);
    f(entries.swap_remove(i))
}

/// `GetCategories()[cat]->GetEntries()`
fn category_entries(ctx: &mut Ctx, cat: usize) -> Vec<OptRef> {
    match CATEGORY_ORDER[cat] {
        "Keymapping" => (0..ctx.options.keymapper.actions.len()).map(OptRef::Key).collect(),
        "Padmapping" => (0..ctx.options.padmapper.actions.len()).map(OptRef::Pad).collect(),
        key => (0..ctx.options.entries_of(key).len()).map(|i| OptRef::Plain(cat, i)).collect(),
    }
}

fn opt_flags(ctx: &mut Ctx, r: OptRef) -> u8 {
    match r {
        OptRef::Plain(c, i) => with_plain(ctx, c, i, |e| e.base().get_flags()),
        OptRef::Key(i) => ctx.options.keymapper.actions[i].base.get_flags(),
        OptRef::Pad(i) => ctx.options.padmapper.actions[i].base.get_flags(),
    }
}

fn opt_type(ctx: &mut Ctx, r: OptRef) -> OptionEntryType {
    match r {
        OptRef::Plain(c, i) => with_plain(ctx, c, i, |e| e.get_type()),
        OptRef::Key(_) => OptionEntryType::Key,
        OptRef::Pad(_) => OptionEntryType::PadButton,
    }
}

fn opt_name(ctx: &mut Ctx, r: OptRef) -> String {
    match r {
        OptRef::Plain(c, i) => with_plain(ctx, c, i, |e| e.get_name()),
        OptRef::Key(i) => ctx.options.keymapper.actions[i].get_name(),
        OptRef::Pad(i) => ctx.options.padmapper.actions[i].get_name(),
    }
}

fn opt_description(ctx: &mut Ctx, r: OptRef) -> String {
    match r {
        OptRef::Plain(c, i) => with_plain(ctx, c, i, |e| e.base().get_description()),
        OptRef::Key(i) => ctx.options.keymapper.actions[i].base.get_description(),
        OptRef::Pad(i) => ctx.options.padmapper.actions[i].base.get_description(),
    }
}

fn opt_value_description(ctx: &mut Ctx, r: OptRef) -> String {
    match r {
        OptRef::Plain(c, i) => with_plain(ctx, c, i, |e| e.get_value_description()),
        OptRef::Key(i) => ctx.options.keymapper.action_value_description(i),
        OptRef::Pad(i) => ctx.options.padmapper.actions[i].value_description(ctx, false),
    }
}

fn opt_list_size(ctx: &mut Ctx, r: OptRef) -> usize {
    match r {
        OptRef::Plain(c, i) => with_plain(ctx, c, i, |e| e.get_list_size()),
        _ => 0,
    }
}

fn opt_list_description(ctx: &mut Ctx, r: OptRef, index: usize) -> String {
    match r {
        OptRef::Plain(c, i) => with_plain(ctx, c, i, |e| e.get_list_description(index)),
        _ => String::new(),
    }
}

fn opt_active_list_index(ctx: &mut Ctx, r: OptRef) -> usize {
    match r {
        OptRef::Plain(c, i) => with_plain(ctx, c, i, |e| e.get_active_list_index()),
        _ => 0,
    }
}

fn is_fullscreen_option(ctx: &mut Ctx, r: OptRef) -> bool {
    match r {
        OptRef::Plain(c, i) => CATEGORY_ORDER[c] == "Graphics" && with_plain(ctx, c, i, |e| e.base().key == "Fullscreen"),
        _ => false,
    }
}

// ---------------------------------------------------------------------------------------------

/// Original: `IsValidEntry` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::IsValidEntry(OptionEntryBase *pOptionEntry) sha=74438ff224c6
fn is_valid_entry(ctx: &mut Ctx, r: OptRef) -> bool {
    let flags = opt_flags(ctx, r);
    if flags & of::NEED_DIABLO_MPQ != 0 && !crate::init::have_diabdat(ctx) {
        return false;
    }
    if flags & of::NEED_HELLFIRE_MPQ != 0 && !crate::init::have_hellfire(ctx) {
        return false;
    }
    flags & (of::INVISIBLE | if ctx.init.gb_is_hellfire { of::ONLY_DIABLO } else { of::ONLY_HELLFIRE }) == 0
}

/// Original: `CreateDrawStringFormatArgForEntry` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::CreateDrawStringFormatArgForEntry(OptionEntryBase *pEntry) sha=78301f020170
fn create_draw_string_format_arg_for_entry(ctx: &mut Ctx, r: OptRef) -> Vec<DrawStringFormatArg> {
    vec![
        DrawStringFormatArg::string(&opt_name(ctx, r), UiFlags::COLOR_UI_GOLD),
        DrawStringFormatArg::string(&opt_value_description(ctx, r), UiFlags::COLOR_UI_SILVER),
    ]
}

/// Original: `NeedsTwoLinesToDisplayOption` (DiabloUI/settingsmenu.cpp): check if the option
/// text can't fit in one list line (list width minus drawn selector).
// @port DiabloUI/settingsmenu.cpp|devilution::NeedsTwoLinesToDisplayOption(std::vector<DrawStringFormatArg> &formatArgs) sha=0d950d203eb4
fn needs_two_lines_to_display_option(ctx: &mut Ctx, format_args: &mut [DrawStringFormatArg]) -> bool {
    let w = st(ctx).rect_list.w;
    get_line_width_fmt(ctx, "{}: {}", format_args, 0, GameFontTables::GameFont24, 1, None) >= (w - 90)
}

/// Original: `CleanUpSettingsUI` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::CleanUpSettingsUI() sha=709b22c0ecf6
fn clean_up_settings_ui(ctx: &mut Ctx) {
    ui_init_list_clear(ctx);

    let s = st(ctx);
    s.vec_dialog_items.clear();
    s.vec_dialog.clear();
    s.vec_options.clear();

    ctx.diablo_ui.art_background = None;
    ctx.diablo_ui.art_background_widescreen = None;
    crate::diablo_ui::scrollbar::unload_scroll_bar(ctx);
}

/// Original: `GoBackOneMenuLevel` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::GoBackOneMenuLevel() sha=3e2cc4a37159
fn go_back_one_menu_level(ctx: &mut Ctx) {
    let s = st(ctx);
    s.end_menu = true;
    match s.shown_menu {
        ShownMenuType::Categories => s.back_to_main = true,
        ShownMenuType::Settings => s.shown_menu = ShownMenuType::Categories,
        _ => s.shown_menu = ShownMenuType::Settings,
    }
}

/// Original: `StartPadEntryTimer` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::StartPadEntryTimer() sha=f78aa9d06504
fn start_pad_entry_timer(ctx: &mut Ctx) {
    let ticks = ctx.platform.ticks();
    let s = st(ctx);
    s.pad_entry_combo = ControllerButtonCombo::single(ControllerButton::None);
    s.pad_entry_start_time = ticks;
    if s.pad_entry_start_time == 0 {
        s.pad_entry_start_time += 1;
    }
    // Removes access to these dialog items while entering bindings
    for item in s.vec_dialog_items.iter().skip(INDEX_PAD_TIMER_TEXT + 1) {
        item.borrow_mut().ui_flags |= UiFlags::ELEMENT_HIDDEN;
    }
}

/// Original: `StopPadEntryTimer` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::StopPadEntryTimer() sha=b0a2fc9c8e38
fn stop_pad_entry_timer(ctx: &mut Ctx) {
    let s = st(ctx);
    s.pad_entry_combo = ControllerButtonCombo::single(ControllerButton::None);
    s.pad_entry_start_time = 0;
    s.pad_entry_timer_text = String::new();
    s.vec_dialog_items[INDEX_PAD_TIMER_TEXT].borrow_mut().m_text = s.pad_entry_timer_text.clone();
    // Restores access to these dialog items after binding is complete
    for item in s.vec_dialog_items.iter().skip(INDEX_PAD_TIMER_TEXT + 1) {
        item.borrow_mut().ui_flags &= !UiFlags::ELEMENT_HIDDEN;
    }
}

/// Original: `UpdatePadEntryTimerText` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::UpdatePadEntryTimerText() sha=0e6d910b1a10
fn update_pad_entry_timer_text(ctx: &mut Ctx) {
    if st(ctx).shown_menu != ShownMenuType::PadInput {
        return;
    }
    let ticks = ctx.platform.ticks();
    let start = st(ctx).pad_entry_start_time;
    let elapsed = ticks.wrapping_sub(start);
    if start == 0 || elapsed > 10000 {
        stop_pad_entry_timer(ctx);
        return;
    }
    let text = format!("{} {}", tr("Press gamepad buttons to change."), 10 - elapsed / 1000);
    let s = st(ctx);
    s.pad_entry_timer_text = text;
    s.vec_dialog_items[INDEX_PAD_TIMER_TEXT].borrow_mut().m_text = s.pad_entry_timer_text.clone();
}

/// Original: `UpdateDescription(const OptionEntryBase &)` / `UpdateDescription(const
/// OptionCategoryBase &)` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::UpdateDescription(const OptionEntryBase &option) sha=7b2ead5b9971
// @port DiabloUI/settingsmenu.cpp|devilution::UpdateDescription(const OptionCategoryBase &category) sha=30e88378b420
fn update_description(ctx: &mut Ctx, description: &str) {
    let w = st(ctx).rect_description.w;
    let paragraphs = word_wrap_string(ctx, description, w as u32, GameFontTables::GameFont12, 1);
    *st(ctx).option_description.borrow_mut() = crate::utils::utf8::copy_utf8(&paragraphs, 512);
}

/// Original: `ItemFocused` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::ItemFocused(int value) sha=42714e5d98b3
fn item_focused(ctx: &mut Ctx, value: i32) {
    match st(ctx).shown_menu {
        ShownMenuType::Categories => {
            let v = st(ctx).vec_dialog_items[value as usize].borrow().m_value;
            st(ctx).option_description.borrow_mut().clear();
            if v < 0 {
                return;
            }
            let d = ctx.options.category(CATEGORY_ORDER[v as usize]).get_description();
            update_description(ctx, &d);
        }
        ShownMenuType::Settings => {
            let v = st(ctx).vec_dialog_items[value as usize].borrow().m_value;
            st(ctx).option_description.borrow_mut().clear();
            if v < 0 {
                return;
            }
            let r = st(ctx).vec_options[v as usize];
            let d = opt_description(ctx, r);
            update_description(ctx, &d);
        }
        _ => {}
    }
}

/// Original: `ChangeOptionValue` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::ChangeOptionValue(OptionEntryBase *pOption, size_t listIndex) sha=d48e25188e99
fn change_option_value(ctx: &mut Ctx, r: OptRef, list_index: usize) -> bool {
    let recreate_ui = opt_flags(ctx, r) & of::RECREATE_UI != 0;
    if recreate_ui {
        st(ctx).end_menu = true;
        // Clean up all UI related Data
        clean_up_settings_ui(ctx);
        crate::diablo_ui::diabloui::unload_ui_gfx(ctx);
        crate::items::free_item_gfx(ctx);
        st(ctx).selected_option = Some(r);
    }

    let callback = match (opt_type(ctx, r), r) {
        (OptionEntryType::Boolean, OptRef::Plain(c, i)) => with_plain(ctx, c, i, |e| e.toggle_boolean()),
        (OptionEntryType::List, OptRef::Plain(c, i)) => with_plain(ctx, c, i, |e| e.set_active_list_index(list_index)),
        _ => None,
    };
    if let Some(cb) = callback {
        crate::options::run_option_callback(ctx, cb);
    }

    if recreate_ui {
        // Reinitialize UI with changed settings (for example game mode, language or resolution)
        crate::diablo_ui::diabloui::ui_initialize(ctx);
        crate::items::init_item_gfx(ctx);
        crate::hwcursor::set_hardware_cursor(ctx, crate::hwcursor::CursorInfo::unknown_cursor());
        return false;
    }

    true
}

/// Original: `ItemSelected` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::ItemSelected(int value) sha=c86d05b9d6ed
fn item_selected(ctx: &mut Ctx, value: i32) {
    let index = value as usize;
    let vec_item = st(ctx).vec_dialog_items[index].clone();
    let vec_item_value = vec_item.borrow().m_value;
    if vec_item_value < 0 {
        match vec_item_value {
            v if v == SpecialMenuEntry::None as i32 => {}
            v if v == SpecialMenuEntry::PreviousMenu as i32 => go_back_one_menu_level(ctx),
            v if v == SpecialMenuEntry::UnbindKey as i32 => {
                if let Some(OptRef::Key(i)) = st(ctx).selected_option {
                    ctx.options.keymapper.set_action_value(i, crate::platform::events::keys::SDLK_UNKNOWN as u32);
                    let d = ctx.options.keymapper.action_value_description(i);
                    st(ctx).vec_dialog_items[INDEX_KEY_OR_PAD_INPUT].borrow_mut().m_text = d;
                }
            }
            v if v == SpecialMenuEntry::BindPadButton as i32 => start_pad_entry_timer(ctx),
            v if v == SpecialMenuEntry::UnbindPadButton as i32 => {
                if let Some(OptRef::Pad(i)) = st(ctx).selected_option {
                    ctx.options.padmapper.actions[i].set_value(ControllerButtonCombo::single(ControllerButton::None));
                    let d = ctx.options.padmapper.actions[i].value_description(ctx, false);
                    st(ctx).vec_dialog_items[INDEX_KEY_OR_PAD_INPUT].borrow_mut().m_text = d;
                }
            }
            _ => {}
        }
        return;
    }

    match st(ctx).shown_menu {
        ShownMenuType::Categories => {
            let s = st(ctx);
            s.selected_category = Some(vec_item_value as usize);
            s.end_menu = true;
            s.shown_menu = ShownMenuType::Settings;
        }
        ShownMenuType::Settings => {
            let p_option = st(ctx).vec_options[vec_item_value as usize];
            let mut update_value_description = false;
            match opt_type(ctx, p_option) {
                OptionEntryType::List => {
                    let list_size = opt_list_size(ctx, p_option);
                    if list_size > 2 {
                        let s = st(ctx);
                        s.selected_option = Some(p_option);
                        s.end_menu = true;
                        s.shown_menu = ShownMenuType::ListOption;
                    } else {
                        // If the list contains only two items, we don't show a submenu and instead change the option value instantly
                        let mut next_index = opt_active_list_index(ctx, p_option) + 1;
                        if next_index >= list_size {
                            next_index = 0;
                        }
                        update_value_description = change_option_value(ctx, p_option, next_index);
                    }
                }
                OptionEntryType::Key => {
                    let s = st(ctx);
                    s.selected_option = Some(p_option);
                    s.end_menu = true;
                    s.shown_menu = ShownMenuType::KeyInput;
                }
                OptionEntryType::PadButton => {
                    let s = st(ctx);
                    s.selected_option = Some(p_option);
                    s.end_menu = true;
                    s.shown_menu = ShownMenuType::PadInput;
                }
                OptionEntryType::Boolean => {
                    update_value_description = change_option_value(ctx, p_option, 0);
                }
            }
            if update_value_description {
                let mut args = create_draw_string_format_arg_for_entry(ctx, p_option);
                let option_uses_two_lines = {
                    let items = &st(ctx).vec_dialog_items;
                    (index + 1) < items.len() && items[index].borrow().m_value == items[index + 1].borrow().m_value
                };
                if needs_two_lines_to_display_option(ctx, &mut args) != option_uses_two_lines {
                    let s = st(ctx);
                    s.selected_option = Some(p_option);
                    s.end_menu = true;
                } else {
                    vec_item.borrow_mut().args = args;
                    if option_uses_two_lines {
                        let d = opt_value_description(ctx, p_option);
                        st(ctx).vec_dialog_items[index + 1].borrow_mut().m_text = d;
                    }
                }
            }
        }
        ShownMenuType::ListOption => {
            let r = st(ctx).selected_option.expect("selectedOption");
            change_option_value(ctx, r, vec_item_value as usize);
            go_back_one_menu_level(ctx);
        }
        ShownMenuType::KeyInput | ShownMenuType::PadInput => {}
    }
}

/// Original: `EscPressed` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::EscPressed() sha=5fa336019a97
fn esc_pressed(ctx: &mut Ctx) {
    go_back_one_menu_level(ctx);
}

/// Original: `FullscreenChanged` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::FullscreenChanged() sha=ad80ab73d43d
fn fullscreen_changed(ctx: &mut Ctx) {
    let items = st(ctx).vec_dialog_items.clone();
    for vec_item in items {
        let vec_item_value = vec_item.borrow().m_value;
        if vec_item_value < 0 || vec_item_value as usize >= st(ctx).vec_options.len() {
            continue;
        }

        let p_option = st(ctx).vec_options[vec_item_value as usize];
        if !is_fullscreen_option(ctx, p_option) {
            continue;
        }

        vec_item.borrow_mut().args = create_draw_string_format_arg_for_entry(ctx, p_option);
        break;
    }
}

/// The `KeyInput` event handler lambda of `UiSettingsMenu`.
fn key_input_event_handler(ctx: &mut Ctx, event: &Event) -> bool {
    use crate::platform::events::keys::SDLK_UNKNOWN;
    use crate::platform::events::{BUTTON_MIDDLE, BUTTON_X1, BUTTON_X2};
    if ctx.diablo_ui.selected_item != INDEX_KEY_OR_PAD_INPUT {
        return false;
    }
    let mut key = SDLK_UNKNOWN as u32;
    match event {
        Event::KeyDown { key: sym, .. } => {
            let keycode = crate::controls::remap_keyboard_key(*sym);
            key = keycode as u32;
            if key >= b'a' as u32 && key <= b'z' as u32 {
                key -= (b'a' - b'A') as u32;
            }
        }
        Event::MouseButtonDown { button, .. } => {
            if [BUTTON_MIDDLE, BUTTON_X1, BUTTON_X2].contains(button) {
                key = *button as u32 | crate::options::KEYMAPPER_MOUSE_BUTTON_MASK;
            }
        }
        _ => {}
    }
    // Ignore unknown keys
    if key == SDLK_UNKNOWN as u32 {
        return false;
    }
    let Some(OptRef::Key(i)) = st(ctx).selected_option else { return false };
    if !ctx.options.keymapper.set_action_value(i, key) {
        return false;
    }
    let d = ctx.options.keymapper.action_value_description(i);
    st(ctx).vec_dialog_items[INDEX_KEY_OR_PAD_INPUT].borrow_mut().m_text = d;
    true
}

/// The `PadInput` event handler lambda of `UiSettingsMenu`.
fn pad_input_event_handler(ctx: &mut Ctx, event: &Event) -> bool {
    use crate::controls::controller::{is_controller_button_pressed, is_controller_motion, to_controller_button_events};
    use crate::platform::events::keys::SDLK_ESCAPE;
    if st(ctx).pad_entry_start_time == 0 {
        return false;
    }

    let ctrl_events = to_controller_button_events(ctx, event);
    for ctrl_event in ctrl_events {
        let is_gamepad_motion = is_controller_motion(event);
        crate::controls::plrctrls::detect_input_method(ctx, event, ctrl_event);
        if matches!(event, Event::KeyUp { key, .. } if *key == SDLK_ESCAPE) {
            stop_pad_entry_timer(ctx);
            return true;
        }
        if is_gamepad_motion || matches!(ctrl_event.button, ControllerButton::None | ControllerButton::Ignore) {
            continue;
        }

        let combo = st(ctx).pad_entry_combo;
        let modifier_pressed = combo.modifier != ControllerButton::None && is_controller_button_pressed(ctx, combo.modifier);
        let button_pressed = combo.button != ControllerButton::None && is_controller_button_pressed(ctx, combo.button);
        if ctrl_event.up {
            // When the player has released all relevant inputs, assume the binding is finished and stop the timer
            if combo.button != ControllerButton::None && !modifier_pressed && !button_pressed {
                stop_pad_entry_timer(ctx);
                return true;
            }
            continue;
        }

        let Some(OptRef::Pad(i)) = st(ctx).selected_option else { return true };
        let s = st(ctx);
        if !modifier_pressed && button_pressed {
            s.pad_entry_combo.modifier = s.pad_entry_combo.button;
        }
        s.pad_entry_combo.button = ctrl_event.button;
        let combo = s.pad_entry_combo;
        if ctx.options.padmapper.actions[i].set_value(combo) {
            let d = ctx.options.padmapper.actions[i].value_description(ctx, false);
            st(ctx).vec_dialog_items[INDEX_KEY_OR_PAD_INPUT].borrow_mut().m_text = d;
        }
    }
    true
}

/// Original: `devilution::UiSettingsMenu` (DiabloUI/settingsmenu.cpp).
// @port DiabloUI/settingsmenu.cpp|devilution::UiSettingsMenu() sha=5d9910f90ef3
pub fn ui_settings_menu(ctx: &mut Ctx) {
    {
        let s = st(ctx);
        s.back_to_main = false;
        s.shown_menu = ShownMenuType::Categories;
        s.selected_category = None;
        s.selected_option = None;
    }

    loop {
        st(ctx).end_menu = false;

        ui_load_black_background(ctx);
        crate::diablo_ui::scrollbar::load_scroll_bar(ctx);
        let mut vec_dialog = std::mem::take(&mut st(ctx).vec_dialog);
        ui_add_background(ctx, &mut vec_dialog);
        ui_add_logo(ctx, &mut vec_dialog);

        let ui_rectangle = crate::utils::display::get_ui_rectangle(ctx);

        let small_font_tall = crate::utils::language::is_small_font_tall(ctx);
        let description_line_height = if small_font_tall { 20 } else { 18 };
        let description_margin_top = if small_font_tall { 10 } else { 16 };
        let rect_list = Rect::new(ui_rectangle.x + 50, ui_rectangle.y + 204, 540, 208);
        let rect_description = Rect::new(rect_list.x - 26, rect_list.y + rect_list.h + description_margin_top, 590, 80 - description_margin_top);
        st(ctx).rect_list = rect_list;
        st(ctx).rect_description = rect_description;

        st(ctx).option_description.borrow_mut().clear();

        let title_text = match st(ctx).shown_menu {
            ShownMenuType::Categories => tr("Settings"),
            ShownMenuType::Settings => {
                let c = st(ctx).selected_category.expect("selectedCategory");
                ctx.options.category(CATEGORY_ORDER[c]).get_name()
            }
            _ => {
                let r = st(ctx).selected_option.expect("selectedOption");
                opt_name(ctx, r)
            }
        };
        vec_dialog.push(UiItem::art_text(
            &title_text,
            Rect::new(ui_rectangle.x, ui_rectangle.y + 161, ui_rectangle.w, 35),
            UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER | UiFlags::ALIGN_CENTER,
            8,
            -1,
        ));
        {
            let sb = &ctx.diablo_ui.scrollbar;
            let bg = sb.art_scroll_bar_background.as_ref().expect("ArtScrollBarBackground").get(0);
            let thumb = sb.art_scroll_bar_thumb.as_ref().expect("ArtScrollBarThumb").get(0);
            let arrow = sb.art_scroll_bar_arrow.clone().expect("ArtScrollBarArrow");
            vec_dialog.push(UiItem::scrollbar(bg, thumb, arrow, Rect::new(rect_list.x + rect_list.w + 5, rect_list.y, 25, rect_list.h), UiFlags::NONE));
        }
        let description = st(ctx).option_description.clone();
        vec_dialog.push(UiItem::art_text_dynamic(
            description,
            rect_description,
            UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK | UiFlags::ALIGN_CENTER,
            1,
            description_line_height,
        ));

        let mut item_to_select = 0usize;
        let mut event_handler: Option<fn(&mut Ctx, &Event) -> bool> = None;
        let mut items: Vec<UiListItemRef> = Vec::new();

        match st(ctx).shown_menu {
            ShownMenuType::Categories => {
                let selected_category = st(ctx).selected_category;
                for cat_index in 0..CATEGORY_ORDER.len() {
                    for entry in category_entries(ctx, cat_index) {
                        if !is_valid_entry(ctx, entry) {
                            continue;
                        }
                        if selected_category == Some(cat_index) {
                            item_to_select = items.len();
                        }
                        let name = ctx.options.category(CATEGORY_ORDER[cat_index]).get_name();
                        items.push(UiListItem::new(&name, cat_index as i32, UiFlags::COLOR_UI_GOLD));
                        break;
                    }
                }
            }
            ShownMenuType::Settings => {
                let c = st(ctx).selected_category.expect("selectedCategory");
                let selected_option = st(ctx).selected_option;
                for entry in category_entries(ctx, c) {
                    if !is_valid_entry(ctx, entry) {
                        continue;
                    }
                    if selected_option == Some(entry) {
                        item_to_select = items.len();
                    }
                    let mut format_args = create_draw_string_format_arg_for_entry(ctx, entry);
                    let n = st(ctx).vec_options.len() as i32;
                    if needs_two_lines_to_display_option(ctx, &mut format_args) {
                        items.push(UiListItem::with_args("{}:", format_args, n, UiFlags::COLOR_UI_GOLD | UiFlags::NEEDS_NEXT_ELEMENT));
                        let d = opt_value_description(ctx, entry);
                        items.push(UiListItem::new(&d, n, UiFlags::COLOR_UI_SILVER | UiFlags::ELEMENT_DISABLED));
                    } else {
                        items.push(UiListItem::with_args("{}: {}", format_args, n, UiFlags::COLOR_UI_GOLD));
                    }
                    st(ctx).vec_options.push(entry);
                }
            }
            ShownMenuType::ListOption => {
                let r = st(ctx).selected_option.expect("selectedOption");
                for i in 0..opt_list_size(ctx, r) {
                    let d = opt_list_description(ctx, r, i);
                    items.push(UiListItem::new(&d, i as i32, UiFlags::COLOR_UI_GOLD));
                }
                item_to_select = opt_active_list_index(ctx, r);
                let d = opt_description(ctx, r);
                update_description(ctx, &d);
            }
            ShownMenuType::KeyInput => {
                let r = st(ctx).selected_option.expect("selectedOption");
                items.push(UiListItem::new(&tr("Bound key:"), SpecialMenuEntry::None as i32, UiFlags::COLOR_WHITEGOLD | UiFlags::ELEMENT_DISABLED));
                let d = opt_value_description(ctx, r);
                items.push(UiListItem::new(&d, SpecialMenuEntry::None as i32, UiFlags::COLOR_UI_GOLD));
                assert_eq!(INDEX_KEY_OR_PAD_INPUT, items.len() - 1);
                item_to_select = INDEX_KEY_OR_PAD_INPUT;
                event_handler = Some(key_input_event_handler);
                items.push(UiListItem::new(&tr("Press any key to change."), SpecialMenuEntry::None as i32, UiFlags::COLOR_UI_SILVER | UiFlags::ELEMENT_DISABLED));
                items.push(UiListItem::new("", SpecialMenuEntry::None as i32, UiFlags::ELEMENT_DISABLED));
                items.push(UiListItem::new(&tr("Unbind key"), SpecialMenuEntry::UnbindKey as i32, UiFlags::COLOR_UI_GOLD));
                let d = opt_description(ctx, r);
                update_description(ctx, &d);
            }
            ShownMenuType::PadInput => {
                let r = st(ctx).selected_option.expect("selectedOption");
                items.push(UiListItem::new(&tr("Bound button combo:"), SpecialMenuEntry::None as i32, UiFlags::COLOR_WHITEGOLD | UiFlags::ELEMENT_DISABLED));
                let d = opt_value_description(ctx, r);
                items.push(UiListItem::new(&d, SpecialMenuEntry::BindPadButton as i32, UiFlags::COLOR_UI_GOLD));
                assert_eq!(INDEX_KEY_OR_PAD_INPUT, items.len() - 1);
                item_to_select = INDEX_KEY_OR_PAD_INPUT;

                let timer_text = st(ctx).pad_entry_timer_text.clone();
                items.push(UiListItem::new(&timer_text, SpecialMenuEntry::None as i32, UiFlags::COLOR_UI_SILVER | UiFlags::ELEMENT_DISABLED));
                assert_eq!(INDEX_PAD_TIMER_TEXT, items.len() - 1);

                items.push(UiListItem::new("", SpecialMenuEntry::None as i32, UiFlags::ELEMENT_DISABLED));
                items.push(UiListItem::new(&tr("Unbind button combo"), SpecialMenuEntry::UnbindPadButton as i32, UiFlags::COLOR_UI_GOLD));

                st(ctx).pad_entry_start_time = 0;
                event_handler = Some(pad_input_event_handler);
                let d = opt_description(ctx, r);
                update_description(ctx, &d);
            }
        }

        items.push(UiListItem::new("", SpecialMenuEntry::None as i32, UiFlags::ELEMENT_DISABLED));
        items.push(UiListItem::new(&tr("Previous Menu"), SpecialMenuEntry::PreviousMenu as i32, UiFlags::COLOR_UI_GOLD));
        st(ctx).vec_dialog_items = items.clone();

        vec_dialog.push(UiItem::list(
            &items,
            (rect_list.h / 26) as usize,
            rect_list.x,
            rect_list.y,
            rect_list.w,
            26,
            UiFlags::FONT_SIZE_24 | UiFlags::ALIGN_CENTER,
            1,
        ));
        st(ctx).vec_dialog = vec_dialog.clone();

        ui_init_list(ctx, Some(item_focused), Some(item_selected), Some(esc_pressed), &vec_dialog, true, Some(fullscreen_changed), None, item_to_select);

        while !st(ctx).end_menu {
            ui_clear_screen(ctx);
            update_pad_entry_timer_text(ctx);
            match event_handler {
                Some(h) => {
                    let mut h = h;
                    ui_poll_and_render(ctx, Some(&mut h));
                }
                None => ui_poll_and_render(ctx, None),
            }
        }

        clean_up_settings_ui(ctx);
        if st(ctx).back_to_main {
            break;
        }
    }

    crate::options::save_options(ctx);
}
