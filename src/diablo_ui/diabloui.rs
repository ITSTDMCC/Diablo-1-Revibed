//! `Source/DiabloUI/diabloui.cpp`: the front-end UI framework (lists, focus, rendering,
//! mouse/keyboard navigation, fades).

use std::cell::RefCell;
use std::rc::Rc;

use crate::controls::controller::{get_menu_actions, get_menu_held_up_down_action, MenuAction};
use crate::controls::ControlTypes;
use crate::ctx::Ctx;
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiKind, UiType};
use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList};
use crate::engine::palette::PaletteRef;
use crate::engine::render::text_render::{draw_string, draw_string_with_colors, render_clx_sprite, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::platform::events::{keys::*, Event, BUTTON_LEFT, KMOD_CTRL};

/// `_artFocus`
pub const FOCUS_SMALL: usize = 0;
pub const FOCUS_MED: usize = 1;
pub const FOCUS_BIG: usize = 2;

/// `_mainmenu_selections`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum MainmenuSelections {
    None,
    SinglePlayer,
    Multiplayer,
    ShowSupport,
    Settings,
    ShowCredits,
    ExitDiablo,
    AttractMode,
}

impl MainmenuSelections {
    pub fn from_i32(v: i32) -> MainmenuSelections {
        use MainmenuSelections::*;
        [None, SinglePlayer, Multiplayer, ShowSupport, Settings, ShowCredits, ExitDiablo, AttractMode][v as usize]
    }
}

/// Number of hero classes (`enum_size<HeroClass>`).
pub const NUM_HERO_CLASSES: usize = 6;

pub type FocusFn = fn(&mut Ctx, i32);
pub type SelectFn = fn(&mut Ctx, i32);
pub type EscFn = fn(&mut Ctx);
pub type FullscreenFn = fn(&mut Ctx);
pub type YesNoFn = fn(&mut Ctx) -> bool;

#[derive(Default, Clone, Copy)]
struct ScrollBarState {
    up_arrow_pressed: bool,
    down_arrow_pressed: bool,
}

/// Globals of diabloui.cpp.
pub struct DiabloUiState {
    pub art_logo: Option<ClxSpriteList>,
    pub difficulty_indicator: Option<ClxSpriteList>,
    pub art_focus: [Option<ClxSpriteList>; 3],
    pub art_background_widescreen: Option<ClxSpriteList>,
    pub art_background: Option<ClxSpriteList>,
    pub art_cursor: Option<ClxSpriteList>,
    pub text_input_active: bool,
    pub selected_item: usize,
    art_hero: Option<ClxSpriteList>,
    art_hero_portrait_order: [u8; NUM_HERO_CLASSES + 1],
    art_hero_overrides: [Option<ClxSpriteList>; NUM_HERO_CLASSES + 1],
    selected_item_max: usize,
    list_viewport_size: usize,
    list_offset: usize,
    gfn_list_focus: Option<FocusFn>,
    gfn_list_select: Option<SelectFn>,
    gfn_list_esc: Option<EscFn>,
    gfn_fullscreen: Option<FullscreenFn>,
    gfn_list_yes_no: Option<YesNoFn>,
    g_ui_items: Vec<UiItemRef>,
    g_ui_list: Option<UiItemRef>,
    ui_items_wraps: bool,
    ui_text_input: Option<Rc<RefCell<String>>>,
    ui_text_input_len: usize,
    allow_empty_text_input: bool,
    fade_tc: u32,
    pub fade_value: i32,
    scroll_bar_state: ScrollBarState,
    /// `DiabloTitleLogo` (title.cpp)
    pub diablo_title_logo: Option<ClxSpriteList>,
    /// `ButtonSprites` (button.cpp)
    pub button_sprites: Option<ClxSpriteList>,
    /// scrollbar.cpp
    pub scrollbar: crate::diablo_ui::scrollbar::ScrollBarArt,
    /// selstart.cpp
    pub selstart: crate::diablo_ui::selstart::SelStartState,
    /// mainmenu.cpp
    pub mainmenu: crate::diablo_ui::mainmenu::MainMenuState,
    /// hero/selhero.cpp
    pub selhero: crate::diablo_ui::selhero::SelHeroState,
    /// selok.cpp
    pub selok: crate::diablo_ui::selok::SelOkState,
    /// selyesno.cpp
    pub selyesno: crate::diablo_ui::selyesno::SelYesNoState,
    /// multi/selgame.cpp
    pub selgame: crate::diablo_ui::selgame::SelGameState,
    /// settingsmenu.cpp
    pub settingsmenu: crate::diablo_ui::settingsmenu::SettingsMenuState,
}

impl Default for DiabloUiState {
    fn default() -> Self {
        DiabloUiState {
            art_logo: None,
            difficulty_indicator: None,
            art_focus: [None, None, None],
            art_background_widescreen: None,
            art_background: None,
            art_cursor: None,
            text_input_active: true,
            selected_item: 0,
            art_hero: None,
            art_hero_portrait_order: [0; NUM_HERO_CLASSES + 1],
            art_hero_overrides: Default::default(),
            selected_item_max: 0,
            list_viewport_size: 1,
            list_offset: 0,
            gfn_list_focus: None,
            gfn_list_select: None,
            gfn_list_esc: None,
            gfn_fullscreen: None,
            gfn_list_yes_no: None,
            g_ui_items: Vec::new(),
            g_ui_list: None,
            ui_items_wraps: false,
            ui_text_input: None,
            ui_text_input_len: 0,
            allow_empty_text_input: false,
            fade_tc: 0,
            fade_value: 0,
            scroll_bar_state: ScrollBarState::default(),
            diablo_title_logo: None,
            button_sprites: None,
            scrollbar: Default::default(),
            selstart: Default::default(),
            mainmenu: Default::default(),
            selhero: Default::default(),
            selok: Default::default(),
            settingsmenu: Default::default(),
            selyesno: Default::default(),
            selgame: Default::default(),
        }
    }
}

/// `DiabloUiSurface()`: the back buffer.
pub fn diablo_ui_surface(ctx: &mut Ctx) -> Surface {
    match ctx.dx.pal_surface.as_mut() {
        Some(s) => Surface::of(s),
        None => Surface::default(),
    }
}

/// Original: `AdjustListOffset` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::AdjustListOffset(std::size_t itemIndex) sha=e078eac5709d
fn adjust_list_offset(ctx: &mut Ctx, item_index: usize) {
    let u = &mut ctx.diablo_ui;
    if item_index >= u.list_offset + u.list_viewport_size {
        u.list_offset = item_index - (u.list_viewport_size.wrapping_sub(1));
    }
    if item_index < u.list_offset {
        u.list_offset = item_index;
    }
}

/// Original: `devilution::UiInitList` (DiabloUI/diabloui.cpp). Defaults in the original:
/// itemsWraps false, fnFullscreen/fnYesNo null, selectedItem 0.
// @port DiabloUI/diabloui.cpp|devilution::UiInitList(void (*fnFocus)(int value), void (*fnSelect)(int value), void (*fnEsc)(), const std::vector<std::unique_ptr<UiItemBase>> &items, bool itemsWraps, void (*fnFullscreen)(), bool (*fnYesNo)(), size_t selectedItem) sha=97026fdfef46
#[allow(clippy::too_many_arguments)]
pub fn ui_init_list(
    ctx: &mut Ctx,
    fn_focus: Option<FocusFn>,
    fn_select: Option<SelectFn>,
    fn_esc: Option<EscFn>,
    items: &[UiItemRef],
    items_wraps: bool,
    fn_fullscreen: Option<FullscreenFn>,
    fn_yes_no: Option<YesNoFn>,
    selected_item: usize,
) {
    {
        let u = &mut ctx.diablo_ui;
        u.selected_item = selected_item;
        u.selected_item_max = 0;
        u.list_viewport_size = 0;
        u.gfn_list_focus = fn_focus;
        u.gfn_list_select = fn_select;
        u.gfn_list_esc = fn_esc;
        u.gfn_fullscreen = fn_fullscreen;
        u.gfn_list_yes_no = fn_yes_no;
        u.g_ui_items = items.to_vec();
        u.ui_items_wraps = items_wraps;
        u.list_offset = 0;
    }
    if let Some(f) = fn_focus {
        f(ctx, selected_item as i32);
    }
    // SDL_StopTextInput(): text input is delivered regardless; textInputActive gates it.
    ctx.diablo_ui.text_input_active = false;
    let mut scrollbar: Option<UiItemRef> = None;
    for item in items {
        let ty = item.borrow().get_type();
        match ty {
            UiType::Edit => {
                let it = item.borrow();
                if let UiKind::Edit { value, max_length, allow_empty, .. } = &it.kind {
                    ctx.diablo_ui.text_input_active = true;
                    ctx.diablo_ui.allow_empty_text_input = *allow_empty;
                    ctx.diablo_ui.ui_text_input = Some(value.clone());
                    ctx.diablo_ui.ui_text_input_len = *max_length;
                }
            }
            UiType::List => {
                let (max, viewport, needs_next) = {
                    let it = item.borrow();
                    let l = it.as_list();
                    let max = l.m_vec_items.len().saturating_sub(1);
                    let needs_next = selected_item <= max
                        && !l.m_vec_items.is_empty()
                        && l.get_item(selected_item).borrow().ui_flags.has(UiFlags::NEEDS_NEXT_ELEMENT);
                    (max, l.viewport_size, needs_next)
                };
                ctx.diablo_ui.selected_item_max = max;
                ctx.diablo_ui.list_viewport_size = viewport;
                ctx.diablo_ui.g_ui_list = Some(item.clone());
                if needs_next {
                    adjust_list_offset(ctx, selected_item + 1);
                }
            }
            UiType::Scrollbar => scrollbar = Some(item.clone()),
            _ => {}
        }
    }
    adjust_list_offset(ctx, selected_item);
    if let Some(sb) = scrollbar {
        if ctx.diablo_ui.list_viewport_size >= ctx.diablo_ui.selected_item_max + 1 {
            sb.borrow_mut().hide();
        } else {
            sb.borrow_mut().show();
        }
    }
}

/// Original: `devilution::UiRenderListItems` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiRenderListItems() sha=94dcb3844002
pub fn ui_render_list_items(ctx: &mut Ctx) {
    let items = ctx.diablo_ui.g_ui_items.clone();
    ui_render_items(ctx, &items);
}

/// Original: `devilution::UiInitList_clear` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiInitList_clear() sha=4e14f1b1b426
pub fn ui_init_list_clear(ctx: &mut Ctx) {
    let u = &mut ctx.diablo_ui;
    u.selected_item = 0;
    u.selected_item_max = 0;
    u.list_viewport_size = 1;
    u.gfn_list_focus = None;
    u.gfn_list_select = None;
    u.gfn_list_esc = None;
    u.gfn_fullscreen = None;
    u.gfn_list_yes_no = None;
    u.g_ui_list = None;
    u.g_ui_items.clear();
    u.ui_items_wraps = false;
}

/// Original: `devilution::UiPlayMoveSound` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiPlayMoveSound() sha=341a19bae35e
pub fn ui_play_move_sound(ctx: &mut Ctx) {
    crate::effects::effects_play_sound(ctx, crate::effects::IS_TITLEMOV);
}

/// Original: `devilution::UiPlaySelectSound` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiPlaySelectSound() sha=e5e2137f1865
pub fn ui_play_select_sound(ctx: &mut Ctx) {
    crate::effects::effects_play_sound(ctx, crate::effects::IS_TITLSLCT);
}

fn list_item_flags(ctx: &Ctx, index: usize) -> UiFlags {
    let list = ctx.diablo_ui.g_ui_list.as_ref().expect("gUiList");
    let it = list.borrow();
    let flags = it.as_list().get_item(index).borrow().ui_flags;
    flags
}

/// Original: `UiFocus` (DiabloUI/diabloui.cpp). `ignore_items_wraps` defaults to false.
// @port DiabloUI/diabloui.cpp|devilution::UiFocus(std::size_t itemIndex, bool checkUp, bool ignoreItemsWraps = false) sha=cda207d4fe7d
fn ui_focus(ctx: &mut Ctx, mut item_index: usize, mut check_up: bool, ignore_items_wraps: bool) {
    if ctx.diablo_ui.selected_item == item_index {
        return;
    }
    adjust_list_offset(ctx, item_index);
    while list_item_flags(ctx, item_index).has(UiFlags::ELEMENT_HIDDEN | UiFlags::ELEMENT_DISABLED) {
        let (wraps, max) = (ctx.diablo_ui.ui_items_wraps, ctx.diablo_ui.selected_item_max);
        if check_up {
            if item_index > 0 {
                item_index -= 1;
            } else if wraps && !ignore_items_wraps {
                item_index = max;
            } else {
                check_up = false;
            }
        } else if item_index < max {
            item_index += 1;
        } else if wraps && !ignore_items_wraps {
            item_index = 0;
        } else {
            check_up = true;
        }
    }
    if list_item_flags(ctx, item_index).has(UiFlags::NEEDS_NEXT_ELEMENT) {
        adjust_list_offset(ctx, item_index + 1);
    }
    adjust_list_offset(ctx, item_index);
    ctx.diablo_ui.selected_item = item_index;
    ui_play_move_sound(ctx);
    if let Some(f) = ctx.diablo_ui.gfn_list_focus {
        f(ctx, item_index as i32);
    }
}

/// Original: `UiFocusUp` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiFocusUp() sha=08ac076f1355
fn ui_focus_up(ctx: &mut Ctx) {
    let u = &ctx.diablo_ui;
    if u.selected_item > 0 {
        let i = u.selected_item - 1;
        ui_focus(ctx, i, true, false);
    } else if u.ui_items_wraps {
        let m = u.selected_item_max;
        ui_focus(ctx, m, true, false);
    }
}

/// Original: `UiFocusDown` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiFocusDown() sha=7e0f1b0dd59c
fn ui_focus_down(ctx: &mut Ctx) {
    let u = &ctx.diablo_ui;
    if u.selected_item < u.selected_item_max {
        let i = u.selected_item + 1;
        ui_focus(ctx, i, false, false);
    } else if u.ui_items_wraps {
        ui_focus(ctx, 0, false, false);
    }
}

/// Original: `UiFocusPageUp` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiFocusPageUp() sha=7fd87bb8f66c
fn ui_focus_page_up(ctx: &mut Ctx) {
    if ctx.diablo_ui.list_offset == 0 {
        ui_focus(ctx, 0, true, true);
    } else {
        let u = &ctx.diablo_ui;
        let relpos = u.selected_item.wrapping_sub(u.list_offset);
        let mut prev_page_start = u.selected_item.wrapping_sub(relpos);
        if prev_page_start >= u.list_viewport_size {
            prev_page_start -= u.list_viewport_size;
        } else {
            prev_page_start = 0;
        }
        adjust_list_offset(ctx, prev_page_start);
        let i = ctx.diablo_ui.list_offset + relpos;
        ui_focus(ctx, i, true, true);
    }
}

/// Original: `UiFocusPageDown` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiFocusPageDown() sha=234500638afb
fn ui_focus_page_down(ctx: &mut Ctx) {
    let u = &ctx.diablo_ui;
    if u.list_offset + u.list_viewport_size > u.selected_item_max {
        let m = u.selected_item_max;
        ui_focus(ctx, m, false, true);
    } else {
        let relpos = u.selected_item - u.list_offset;
        let mut next_page_end = u.selected_item + (u.list_viewport_size - relpos - 1);
        if next_page_end + u.list_viewport_size <= u.selected_item_max {
            next_page_end += u.list_viewport_size;
        } else {
            next_page_end = u.selected_item_max;
        }
        adjust_list_offset(ctx, next_page_end);
        let i = ctx.diablo_ui.list_offset + relpos;
        ui_focus(ctx, i, false, true);
    }
}

/// Original: `SelheroCatToName` (DiabloUI/diabloui.cpp): appends to the text field, truncated
/// to the field size like `CopyUtf8`.
// @port DiabloUI/diabloui.cpp|devilution::SelheroCatToName(const char *inBuf, char *outBuf, int cnt) sha=b2f4084b3b16
fn selhero_cat_to_name(in_buf: &str, out: &mut String, cnt: usize) {
    let dest_count = cnt.saturating_sub(out.len());
    out.push_str(&crate::utils::utf8::copy_utf8(in_buf, dest_count));
}

/// Original: `HandleMenuAction` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::HandleMenuAction(MenuAction menuAction) sha=76a6302a5383
fn handle_menu_action(ctx: &mut Ctx, action: MenuAction) -> bool {
    match action {
        MenuAction::Select => {
            ui_focus_navigation_select(ctx);
            true
        }
        MenuAction::Up => {
            ui_focus_up(ctx);
            true
        }
        MenuAction::Down => {
            ui_focus_down(ctx);
            true
        }
        MenuAction::PageUp => {
            ui_focus_page_up(ctx);
            true
        }
        MenuAction::PageDown => {
            ui_focus_page_down(ctx);
            true
        }
        MenuAction::Delete => {
            ui_focus_navigation_yes_no(ctx);
            true
        }
        MenuAction::Back => {
            if ctx.diablo_ui.gfn_list_esc.is_none() {
                return false;
            }
            ui_focus_navigation_esc(ctx);
            true
        }
        _ => false,
    }
}

/// Original: `UiOnBackgroundChange` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiOnBackgroundChange() sha=ced27983d053
fn ui_on_background_change(ctx: &mut Ctx) {
    ctx.diablo_ui.fade_tc = 0;
    ctx.diablo_ui.fade_value = 0;
    crate::engine::palette::black_palette(ctx);
    if crate::hwcursor::is_hardware_cursor_enabled(ctx)
        && ctx.diablo_ui.art_cursor.is_some()
        && ctx.controls.control_device == ControlTypes::KeyboardAndMouse
        && crate::hwcursor::get_current_cursor_info(ctx).cursor_type() != crate::hwcursor::CursorType::UserInterface
    {
        crate::hwcursor::set_hardware_cursor(ctx, crate::hwcursor::CursorInfo::user_interface_cursor());
    }
    if let Some(s) = ctx.dx.pal_surface.as_mut() {
        s.pixels.fill(0);
    }
    crate::engine::dx::blt_fast(ctx, None, None);
    crate::engine::dx::render_present(ctx);
}

/// Original: `devilution::UiFocusNavigation` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiFocusNavigation(SDL_Event *event) sha=5568ae9ac1b5
pub fn ui_focus_navigation(ctx: &mut Ctx, event: &Event) {
    if matches!(
        event,
        Event::KeyUp { .. } | Event::MouseButtonUp { .. } | Event::MouseMotion { .. } | Event::MouseWheel { .. } | Event::FocusGained | Event::FocusLost
    ) {
        crate::diablo_ui::mainmenu::mainmenu_restart_repintro(ctx);
    }
    let mut handled = false;
    for a in get_menu_actions(ctx, event) {
        handled |= handle_menu_action(ctx, a);
    }
    if handled {
        return;
    }
    if let Event::MouseWheel { y, .. } = event {
        if *y > 0 {
            ui_focus_up(ctx);
        } else if *y < 0 {
            ui_focus_down(ctx);
        }
        return;
    }
    if ctx.diablo_ui.text_input_active {
        match event {
            Event::KeyDown { key, .. } => match *key {
                k if k == b'v' as i32 => {
                    if ctx.platform.mod_state() & KMOD_CTRL != 0 {
                        // Clipboard paste: the platform has no clipboard access yet (logged).
                        crate::platform::log::info!("clipboard paste is not available");
                    }
                    return;
                }
                SDLK_BACKSPACE | SDLK_LEFT => {
                    if let Some(t) = &ctx.diablo_ui.ui_text_input {
                        let mut t = t.borrow_mut();
                        // FindLastUtf8Symbols: remove the last code point
                        t.pop();
                    }
                    return;
                }
                _ => {}
            },
            Event::TextInput(text) => {
                if let Some(t) = ctx.diablo_ui.ui_text_input.clone() {
                    let len = ctx.diablo_ui.ui_text_input_len;
                    selhero_cat_to_name(text, &mut t.borrow_mut(), len);
                }
                return;
            }
            _ => {}
        }
    }
    if matches!(event, Event::MouseButtonDown { .. } | Event::MouseButtonUp { .. }) {
        let items = ctx.diablo_ui.g_ui_items.clone();
        if ui_item_mouse_events(ctx, event, &items) {}
    }
}

/// Original: `devilution::UiHandleEvents` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiHandleEvents(SDL_Event *event) sha=fc17e9f799ec
pub fn ui_handle_events(ctx: &mut Ctx, event: &Event) {
    if let Event::MouseMotion { x, y } = event {
        ctx.diablo.mouse_position = (*x, *y);
        return;
    }
    if let Event::KeyDown { key: SDLK_RETURN, .. } = event {
        if ctx.platform.is_key_down(SDLK_LALT) || ctx.platform.is_key_down(SDLK_RALT) {
            let full = crate::utils::display::is_full_screen(ctx);
            if let Some(cb) = ctx.options.graphics.fullscreen.set_value(!full) {
                crate::options::run_option_callback(ctx, cb);
            }
            crate::options::save_options(ctx);
            if let Some(f) = ctx.diablo_ui.gfn_fullscreen {
                f(ctx);
            }
            return;
        }
    }
    if *event == Event::Quit {
        crate::diablo::diablo_quit(ctx, 0);
    }
    crate::controls::controller::handle_controller_added_or_removed_event(ctx, event);
    match event {
        Event::FocusLost => crate::engine::sound::music_mute(ctx),
        Event::FocusGained => crate::diablo::diablo_focus_unpause(ctx),
        _ => {}
    }
}

/// Original: `devilution::UiFocusNavigationSelect` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiFocusNavigationSelect() sha=b96b805a1ac8
pub fn ui_focus_navigation_select(ctx: &mut Ctx) {
    ui_play_select_sound(ctx);
    if ctx.diablo_ui.text_input_active {
        let empty = ctx.diablo_ui.ui_text_input.as_ref().is_none_or(|t| t.borrow().is_empty());
        if !ctx.diablo_ui.allow_empty_text_input && empty {
            return;
        }
        ctx.diablo_ui.ui_text_input = None;
        ctx.diablo_ui.ui_text_input_len = 0;
    }
    if let Some(f) = ctx.diablo_ui.gfn_list_select {
        let i = ctx.diablo_ui.selected_item as i32;
        f(ctx, i);
    }
}

/// Original: `devilution::UiFocusNavigationEsc` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiFocusNavigationEsc() sha=39b838acc190
pub fn ui_focus_navigation_esc(ctx: &mut Ctx) {
    ui_play_select_sound(ctx);
    if ctx.diablo_ui.text_input_active {
        ctx.diablo_ui.ui_text_input = None;
        ctx.diablo_ui.ui_text_input_len = 0;
    }
    if let Some(f) = ctx.diablo_ui.gfn_list_esc {
        f(ctx);
    }
}

/// Original: `devilution::UiFocusNavigationYesNo` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiFocusNavigationYesNo() sha=9b7ec5a8ce4d
pub fn ui_focus_navigation_yes_no(ctx: &mut Ctx) {
    let Some(f) = ctx.diablo_ui.gfn_list_yes_no else { return };
    if f(ctx) {
        ui_play_select_sound(ctx);
    }
}

/// Original: `IsInsideRect` (DiabloUI/diabloui.cpp): SDL_PointInRect.
// @port DiabloUI/diabloui.cpp|devilution::IsInsideRect(const SDL_Event &event, const SDL_Rect &rect) sha=2b414b246875
fn is_inside_rect(p: (i32, i32), r: Rect) -> bool {
    p.0 >= r.x && p.0 < r.x + r.w && p.1 >= r.y && p.1 < r.y + r.h
}

/// Original: `LoadHeros` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::LoadHeros() sha=86053d6cbaf5
fn load_heros(ctx: &mut Ctx) {
    const PORTRAIT_HEIGHT: i32 = 76;
    ctx.diablo_ui.art_hero = crate::engine::load_sprites::load_pcx_sprite_list(ctx, "ui_art\\heros", -PORTRAIT_HEIGHT, None, None, true);
    let Some(hero) = ctx.diablo_ui.art_hero.as_ref() else { return };
    let num_portraits = hero.num_sprites();
    // HeroClass: Warrior, Rogue, Sorcerer, Monk, Bard, Barbarian
    let mut order = [0u8, 1, 2, 2, 1, 0, 3];
    if num_portraits >= 6 {
        order[3] = 3; // Monk
        order[4] = 4; // Bard
        order[NUM_HERO_CLASSES] = 5;
    }
    if num_portraits >= 7 {
        order[5] = 6; // Barbarian
    }
    ctx.diablo_ui.art_hero_portrait_order = order;
    for i in 0..=NUM_HERO_CLASSES {
        let path = format!("ui_art\\hero{i}");
        ctx.diablo_ui.art_hero_overrides[i] = crate::engine::load_sprites::load_pcx(ctx, &path, None, None, false);
    }
}

/// Original: `LoadUiGFX` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::LoadUiGFX() sha=6920452a0eb3
fn load_ui_gfx(ctx: &mut Ctx) {
    use crate::engine::load_sprites::{load_pcx, load_pcx_sprite_list};
    ctx.diablo_ui.art_logo = if ctx.init.gb_is_hellfire {
        load_pcx_sprite_list(ctx, "ui_art\\hf_logo2", 16, Some(0), None, true)
    } else {
        load_pcx_sprite_list(ctx, "ui_art\\smlogo", 15, Some(250), None, true)
    };
    ctx.diablo_ui.difficulty_indicator = load_pcx(ctx, "ui_art\\r1_gry", Some(0), None, true);
    ctx.diablo_ui.art_focus[FOCUS_SMALL] = load_pcx_sprite_list(ctx, "ui_art\\focus16", 8, Some(250), None, true);
    ctx.diablo_ui.art_focus[FOCUS_MED] = load_pcx_sprite_list(ctx, "ui_art\\focus", 8, Some(250), None, true);
    ctx.diablo_ui.art_focus[FOCUS_BIG] = load_pcx_sprite_list(ctx, "ui_art\\focus42", 8, Some(250), None, true);
    ctx.diablo_ui.art_cursor = load_pcx(ctx, "ui_art\\cursor", Some(0), None, true);
    load_heros(ctx);
}

/// Original: `devilution::UiGetHeroDialogSprite` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiGetHeroDialogSprite(size_t heroClassIndex) sha=ea1b730c9968
pub fn ui_get_hero_dialog_sprite(ctx: &Ctx, hero_class_index: usize) -> ClxSprite {
    match &ctx.diablo_ui.art_hero_overrides[hero_class_index] {
        Some(o) => o.get(0),
        None => ctx.diablo_ui.art_hero.as_ref().expect("ArtHero").get(ctx.diablo_ui.art_hero_portrait_order[hero_class_index] as usize),
    }
}

/// Original: `devilution::UnloadUiGFX` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UnloadUiGFX() sha=a7ad910a7b08
pub fn unload_ui_gfx(ctx: &mut Ctx) {
    let u = &mut ctx.diablo_ui;
    u.art_hero = None;
    for o in u.art_hero_overrides.iter_mut() {
        *o = None;
    }
    u.art_cursor = None;
    for a in u.art_focus.iter_mut() {
        *a = None;
    }
    u.art_logo = None;
    u.difficulty_indicator = None;
}

/// Original: `devilution::UiInitialize` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiInitialize() sha=ed5f3dcafd6c
pub fn ui_initialize(ctx: &mut Ctx) {
    load_ui_gfx(ctx);
    if ctx.diablo_ui.art_cursor.is_some() {
        ctx.platform.show_cursor(false);
    }
}

/// Original: `devilution::UiDestroy` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiDestroy() sha=623a7ec18b73
pub fn ui_destroy(ctx: &mut Ctx) {
    crate::engine::render::text_render::unload_fonts(ctx);
    unload_ui_gfx(ctx);
}

/// Original: `devilution::UiValidPlayerName` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiValidPlayerName(string_view name) sha=2d5ce3875ffa
pub fn ui_valid_player_name(name: &str) -> bool {
    const PLAYER_NAME_LENGTH: usize = 32;
    if name.is_empty() || name.len() > PLAYER_NAME_LENGTH {
        return false;
    }
    if name.contains([',', '<', '>', '%', '&', '\\', '"', '?', '*', '#', '/', ':', ' ']) {
        return false;
    }
    // IsBasicLatin: printable ASCII 0x20..=0x7E
    if !name.bytes().all(|b| (0x20..=0x7e).contains(&b)) {
        return false;
    }
    let banned = ["gvdl", "dvou", "tiju", "cjudi", "bttipmf", "ojhhfs", "cmj{{bse", "benjo"];
    let shifted: String = name.bytes().map(|b| (b.wrapping_add(1)) as char).collect();
    !banned.iter().any(|b| shifted.contains(b))
}

/// Original: `devilution::GetCenterOffset` (DiabloUI/diabloui.cpp). `bw` 0 means the screen width.
// @port DiabloUI/diabloui.cpp|devilution::GetCenterOffset(Sint16 w, Sint16 bw) sha=5f44bec8d639
pub fn get_center_offset(ctx: &Ctx, w: i32, bw: i32) -> i32 {
    let bw = if bw == 0 { ctx.dx.gn_screen_width } else { bw };
    (bw - w) / 2
}

/// Original: `devilution::UiLoadDefaultPalette` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiLoadDefaultPalette() sha=53f52faf3771
pub fn ui_load_default_palette(ctx: &mut Ctx) {
    let name = if ctx.init.gb_is_hellfire { "ui_art\\hellfire.pal" } else { "ui_art\\diablo.pal" };
    crate::engine::palette::load_palette(ctx, name, false);
    crate::engine::palette::apply_gamma(ctx, PaletteRef::Logical, PaletteRef::Orig, 256);
}

/// Original: `devilution::UiLoadBlackBackground` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiLoadBlackBackground() sha=f7d3863c155e
pub fn ui_load_black_background(ctx: &mut Ctx) -> bool {
    ctx.diablo_ui.art_background = None;
    ui_load_default_palette(ctx);
    ui_on_background_change(ctx);
    true
}

/// Original: `devilution::LoadBackgroundArt` (DiabloUI/diabloui.cpp). `frames` defaults to 1.
// @port DiabloUI/diabloui.cpp|devilution::LoadBackgroundArt(const char *pszFile, int frames) sha=cc0b1a8f26be
pub fn load_background_art(ctx: &mut Ctx, file: &str, frames: i32) {
    ctx.diablo_ui.art_background = None;
    let mut pal = [[0u8; 3]; 256];
    ctx.diablo_ui.art_background = crate::engine::load_sprites::load_pcx_sprite_list(ctx, file, frames as u16 as i32, None, Some(&mut pal), true);
    if ctx.diablo_ui.art_background.is_none() {
        return;
    }
    load_pal_in_mem(ctx, &pal);
    crate::engine::palette::apply_gamma(ctx, PaletteRef::Logical, PaletteRef::Orig, 256);
    ui_on_background_change(ctx);
}

/// Original: `devilution::UiAddBackground` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiAddBackground(std::vector<std::unique_ptr<UiItemBase>> *vecDialog) sha=770264b91fc3
pub fn ui_add_background(ctx: &mut Ctx, vec_dialog: &mut Vec<UiItemRef>) {
    let rect = Rect::new(0, crate::utils::display::get_ui_rectangle(ctx).y, 0, 0);
    if let Some(w) = &ctx.diablo_ui.art_background_widescreen {
        vec_dialog.push(UiItem::image_clx(w.get(0), rect, UiFlags::ALIGN_CENTER));
    }
    if let Some(b) = &ctx.diablo_ui.art_background {
        vec_dialog.push(UiItem::image_clx(b.get(0), rect, UiFlags::ALIGN_CENTER));
    }
}

/// Original: `devilution::UiAddLogo` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiAddLogo(std::vector<std::unique_ptr<UiItemBase>> *vecDialog) sha=8bced781e346
pub fn ui_add_logo(ctx: &mut Ctx, vec_dialog: &mut Vec<UiItemRef>) {
    let logo = ctx.diablo_ui.art_logo.clone().expect("ArtLogo");
    let y = crate::utils::display::get_ui_rectangle(ctx).y;
    vec_dialog.push(UiItem::image_animated_clx(logo, Rect::new(0, y, 0, 0), UiFlags::ALIGN_CENTER));
}

/// Original: `devilution::UiFadeIn` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiFadeIn() sha=c57a7398d881
pub fn ui_fade_in(ctx: &mut Ctx) {
    if ctx.diablo_ui.fade_value < 256 {
        if ctx.diablo_ui.fade_value == 0 && ctx.diablo_ui.fade_tc == 0 {
            ctx.diablo_ui.fade_tc = ctx.platform.ticks();
        }
        let prev = ctx.diablo_ui.fade_value;
        ctx.diablo_ui.fade_value = (ctx.platform.ticks().wrapping_sub(ctx.diablo_ui.fade_tc) as f64 / 2.083) as i32;
        if ctx.diablo_ui.fade_value > 256 {
            ctx.diablo_ui.fade_value = 256;
            ctx.diablo_ui.fade_tc = 0;
        }
        if ctx.diablo_ui.fade_value != prev {
            let v = ctx.diablo_ui.fade_value;
            crate::engine::palette::set_fade_level(ctx, v, v != 0);
        }
    }
    crate::engine::dx::blt_fast(ctx, None, None);
    crate::engine::dx::render_present(ctx);
}

/// Original: `devilution::DrawSelector` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::DrawSelector(const SDL_Rect &rect) sha=7e57df345da2
pub fn draw_selector(ctx: &mut Ctx, rect: Rect) {
    let size = if rect.h >= 42 {
        FOCUS_BIG
    } else if rect.h >= 30 {
        FOCUS_MED
    } else {
        FOCUS_SMALL
    };
    let sprites = ctx.diablo_ui.art_focus[size].clone().expect("ArtFocus");
    let frame = crate::engine::get_animation_frame(ctx, sprites.num_sprites() as i32, 60);
    let sprite = sprites.get(frame as usize);
    let y = rect.y + (rect.h - sprite.height() as i32) / 2;
    let out = diablo_ui_surface(ctx);
    render_clx_sprite(&out, &sprite, (rect.x, y));
    render_clx_sprite(&out, &sprite, (rect.x + rect.w - sprite.width() as i32, y));
}

/// Original: `devilution::UiClearScreen` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiClearScreen() sha=d9b9b638559b
pub fn ui_clear_screen(ctx: &mut Ctx) {
    let clear = match &ctx.diablo_ui.art_background {
        None => true,
        Some(b) => {
            let s = b.get(0);
            ctx.dx.gn_screen_width > s.width() as i32 || ctx.dx.gn_screen_height > s.height() as i32
        }
    };
    if clear {
        if let Some(s) = ctx.dx.pal_surface.as_mut() {
            s.pixels.fill(0);
        }
    }
}

/// Original: `devilution::UiPollAndRender` (DiabloUI/diabloui.cpp). The optional event handler
/// returns true when it consumed the event.
// @port DiabloUI/diabloui.cpp|devilution::UiPollAndRender(std::optional<tl::function_ref<bool(SDL_Event &)>> eventHandler) sha=d13d1eeb2537
pub fn ui_poll_and_render(ctx: &mut Ctx, event_handler: Option<&mut dyn FnMut(&mut Ctx, &Event) -> bool>) {
    let mut handler = event_handler;
    while let Some(event) = crate::engine::events::poll_event(ctx) {
        if let Some(h) = handler.as_mut() {
            if h(ctx, &event) {
                continue;
            }
        }
        ui_focus_navigation(ctx, &event);
        ui_handle_events(ctx, &event);
    }
    let a = get_menu_held_up_down_action(ctx);
    handle_menu_action(ctx, a);
    ui_render_list_items(ctx);
    draw_mouse(ctx);
    ui_fade_in(ctx);
    if crate::hwcursor::is_hardware_cursor(ctx) && ctx.diablo_ui.fade_value != 0 {
        let v = ctx.controls.control_device == ControlTypes::KeyboardAndMouse;
        crate::hwcursor::set_hardware_cursor_visible(ctx, v);
    }
    // discord_manager::UpdateMenu: Discord is off in the port.
}

/// `Render(const UiItem &)` for each item type (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::Render(const UiText &uiText) sha=26535d8009b8
// @port DiabloUI/diabloui.cpp|devilution::Render(const UiArtText &uiArtText) sha=2fee21d46c0a
// @port DiabloUI/diabloui.cpp|devilution::Render(const UiImageClx &uiImage) sha=a75dd9b04483
// @port DiabloUI/diabloui.cpp|devilution::Render(const UiImageAnimatedClx &uiImage) sha=8f95d3c24163
// @port DiabloUI/diabloui.cpp|devilution::Render(const UiArtTextButton &uiButton) sha=9164b2c4740e
// @port DiabloUI/diabloui.cpp|devilution::Render(const UiList &uiList) sha=538650c55f87
// @port DiabloUI/diabloui.cpp|devilution::Render(const UiScrollbar &uiSb) sha=74f730457700
// @port DiabloUI/diabloui.cpp|devilution::Render(const UiEdit &uiEdit) sha=7b4f80d0fefa
fn render_item(ctx: &mut Ctx, item: &UiItem) {
    let out = diablo_ui_surface(ctx);
    match &item.kind {
        UiKind::Text { text } => {
            draw_string(ctx, &out, text, item.m_rect, item.get_flags() | UiFlags::FONT_SIZE_DIALOG, 1, -1);
        }
        UiKind::ArtText { text, spacing, line_height } => {
            let t = text.get();
            draw_string(ctx, &out, &t, item.m_rect, item.get_flags(), *spacing, *line_height);
        }
        UiKind::ImageClx { sprite } => {
            let mut x = item.m_rect.x;
            if item.is_centered() {
                x += get_center_offset(ctx, sprite.width() as i32, item.m_rect.w);
            }
            render_clx_sprite(&out, sprite, (x, item.m_rect.y));
        }
        UiKind::ImageAnimatedClx { list } => {
            let frame = crate::engine::get_animation_frame(ctx, list.num_sprites() as i32, 60);
            let sprite = list.get(frame as usize);
            let mut x = item.m_rect.x;
            if item.is_centered() {
                x += get_center_offset(ctx, sprite.width() as i32, item.m_rect.w);
            }
            render_clx_sprite(&out, &sprite, (x, item.m_rect.y));
        }
        UiKind::ArtTextButton { text, .. } => {
            draw_string(ctx, &out, text, item.m_rect, item.get_flags(), 1, -1);
        }
        UiKind::Button { .. } => crate::diablo_ui::button::render_button(ctx, item),
        UiKind::List(list) => {
            let (offset, viewport, selected) = (ctx.diablo_ui.list_offset, ctx.diablo_ui.list_viewport_size, ctx.diablo_ui.selected_item);
            let mut i = offset;
            while i < list.m_vec_items.len() && (i - offset) < viewport {
                let rect = list.item_rect((i - offset) as i32);
                let li = list.get_item(i);
                if i == selected {
                    draw_selector(ctx, rect);
                }
                let li = li.borrow().clone();
                if li.args.is_empty() {
                    draw_string(ctx, &out, &li.m_text, rect, item.get_flags() | li.ui_flags, list.get_spacing(), -1);
                } else {
                    let mut args = li.args.clone();
                    draw_string_with_colors(ctx, &out, &li.m_text, &mut args, rect, item.get_flags() | li.ui_flags, list.get_spacing(), -1);
                }
                i += 1;
            }
        }
        UiKind::Scrollbar { bg, thumb, arrow } => {
            use crate::diablo_ui::scrollbar::*;
            {
                let bg_y = item.m_rect.y + arrow.get(0).height() as i32;
                let bg_h = down_arrow_rect(item.m_rect, arrow).y - bg_y;
                let background_out = out.subregion(item.m_rect.x, bg_y, SCROLL_BAR_BG_WIDTH, bg_h);
                let mut y = 0;
                while y < bg_h {
                    render_clx_sprite(&background_out, bg, (0, y));
                    y += bg.height() as i32;
                }
            }
            let sbs = ctx.diablo_ui.scroll_bar_state;
            {
                let rect = up_arrow_rect(item.m_rect, arrow);
                let frame = if sbs.up_arrow_pressed { SCROLL_BAR_ARROW_FRAME_UP_ACTIVE } else { SCROLL_BAR_ARROW_FRAME_UP };
                render_clx_sprite(&out.subregion(rect.x, 0, SCROLL_BAR_ARROW_WIDTH, out.h()), &arrow.get(frame), (0, rect.y));
            }
            {
                let rect = down_arrow_rect(item.m_rect, arrow);
                let frame = if sbs.down_arrow_pressed { SCROLL_BAR_ARROW_FRAME_DOWN_ACTIVE } else { SCROLL_BAR_ARROW_FRAME_DOWN };
                render_clx_sprite(&out.subregion(rect.x, 0, SCROLL_BAR_ARROW_WIDTH, out.h()), &arrow.get(frame), (0, rect.y));
            }
            if ctx.diablo_ui.selected_item_max > 0 {
                let rect = thumb_rect(item.m_rect, arrow, thumb, ctx.diablo_ui.selected_item, ctx.diablo_ui.selected_item_max + 1);
                render_clx_sprite(&out, thumb, (rect.x, rect.y));
            }
        }
        UiKind::Edit { value, .. } => {
            draw_selector(ctx, item.m_rect);
            let r = item.m_rect;
            // MakeRectangle(...).inset({ 43, 1 })
            let rect = Rect::new(r.x + 43, r.y + 1, r.w - 86, r.h - 2);
            let v = value.borrow().clone();
            draw_string(ctx, &out, &v, rect, item.get_flags() | UiFlags::TEXT_CURSOR, 1, -1);
        }
    }
}

/// Original: `HandleMouseEventArtTextButton` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::HandleMouseEventArtTextButton(const SDL_Event &event, const UiArtTextButton *uiButton) sha=9b9021d5fca4
fn handle_mouse_event_art_text_button(ctx: &mut Ctx, event: &Event, action: fn(&mut Ctx)) -> bool {
    if !matches!(event, Event::MouseButtonUp { button: BUTTON_LEFT, .. }) {
        return false;
    }
    action(ctx);
    true
}

/// Original: `HandleMouseEventList` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::HandleMouseEventList(const SDL_Event &event, UiList *uiList) sha=4709238d990e
fn handle_mouse_event_list(ctx: &mut Ctx, event: &Event, item: &UiItemRef) -> bool {
    let (button, y, clicks, down) = match event {
        Event::MouseButtonDown { button, y, clicks, .. } => (*button, *y, *clicks, true),
        Event::MouseButtonUp { button, y, clicks, .. } => (*button, *y, *clicks, false),
        _ => return false,
    };
    if button != BUTTON_LEFT {
        return false;
    }
    let mut index = {
        let it = item.borrow();
        it.as_list().index_at(it.m_rect, y)
    };
    if down {
        item.borrow_mut().as_list_mut().press(index);
        return true;
    }
    if !item.borrow().as_list().is_pressed(index) {
        return false;
    }
    index += ctx.diablo_ui.list_offset;
    if ctx.diablo_ui.gfn_list_focus.is_some() && ctx.diablo_ui.selected_item != index {
        ui_focus(ctx, index, true, false);
    } else if ctx.diablo_ui.gfn_list_focus.is_none() || clicks >= 2 {
        let flags = item.borrow().as_list().get_item(index).borrow().ui_flags;
        if flags.has(UiFlags::ELEMENT_HIDDEN | UiFlags::ELEMENT_DISABLED) {
            return false;
        }
        ctx.diablo_ui.selected_item = index;
        ui_focus_navigation_select(ctx);
    }
    true
}

/// Original: `HandleMouseEventScrollBar` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::HandleMouseEventScrollBar(const SDL_Event &event, const UiScrollbar *uiSb) sha=76fe75cb1c02
fn handle_mouse_event_scroll_bar(ctx: &mut Ctx, event: &Event, item: &UiItem) -> bool {
    use crate::diablo_ui::scrollbar::*;
    let UiKind::Scrollbar { thumb, arrow, .. } = &item.kind else { return false };
    match event {
        Event::MouseButtonUp { button: BUTTON_LEFT, x, y, .. } => {
            let p = (*x, *y);
            if ctx.diablo_ui.scroll_bar_state.up_arrow_pressed && is_inside_rect(p, up_arrow_rect(item.m_rect, arrow)) {
                ui_focus_up(ctx);
                return true;
            }
            if ctx.diablo_ui.scroll_bar_state.down_arrow_pressed && is_inside_rect(p, down_arrow_rect(item.m_rect, arrow)) {
                ui_focus_down(ctx);
                return true;
            }
            false
        }
        Event::MouseButtonDown { button: BUTTON_LEFT, x, y, .. } => {
            let p = (*x, *y);
            if is_inside_rect(p, bar_rect(item.m_rect, arrow)) {
                let tr = thumb_rect(item.m_rect, arrow, thumb, ctx.diablo_ui.selected_item, ctx.diablo_ui.selected_item_max + 1);
                if *y < tr.y {
                    ui_focus_page_up(ctx);
                } else if *y > tr.y + tr.h {
                    ui_focus_page_down(ctx);
                }
                return true;
            }
            if is_inside_rect(p, up_arrow_rect(item.m_rect, arrow)) {
                ctx.diablo_ui.scroll_bar_state.up_arrow_pressed = true;
                return true;
            }
            if is_inside_rect(p, down_arrow_rect(item.m_rect, arrow)) {
                ctx.diablo_ui.scroll_bar_state.down_arrow_pressed = true;
                return true;
            }
            false
        }
        _ => false,
    }
}

/// Original: `HandleMouseEvent` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::HandleMouseEvent(const SDL_Event &event, UiItemBase *item) sha=528785a644cc
fn handle_mouse_event(ctx: &mut Ctx, event: &Event, item: &UiItemRef) -> bool {
    let pos = match event {
        Event::MouseButtonDown { x, y, .. } | Event::MouseButtonUp { x, y, .. } => (*x, *y),
        _ => return false,
    };
    let (not_interactive, rect, ty) = {
        let it = item.borrow();
        (it.is_not_interactive(), it.m_rect, it.get_type())
    };
    if not_interactive || !is_inside_rect(pos, rect) {
        return false;
    }
    match ty {
        UiType::ArtTextButton => {
            let action = match &item.borrow().kind {
                UiKind::ArtTextButton { action, .. } => *action,
                _ => unreachable!(),
            };
            handle_mouse_event_art_text_button(ctx, event, action)
        }
        UiType::Button => crate::diablo_ui::button::handle_mouse_event_button(ctx, event, item),
        UiType::List => handle_mouse_event_list(ctx, event, item),
        UiType::Scrollbar => {
            let it = item.borrow().clone();
            handle_mouse_event_scroll_bar(ctx, event, &it)
        }
        _ => false,
    }
}

/// Original: `devilution::LoadPalInMem` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::LoadPalInMem(const SDL_Color *pPal) sha=05cafb7432d7
pub fn load_pal_in_mem(ctx: &mut Ctx, pal: &[crate::engine::dx::Color; 256]) {
    ctx.dx.pal.orig_palette = *pal;
}

/// Original: `devilution::UiRenderItem` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiRenderItem(const UiItemBase &item) sha=101cf7f40f12
pub fn ui_render_item(ctx: &mut Ctx, item: &UiItemRef) {
    let it = item.borrow().clone();
    if it.is_hidden() {
        return;
    }
    render_item(ctx, &it);
}

/// Original: `devilution::UiRenderItems` (both overloads, DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiRenderItems(const std::vector<UiItemBase *> &items) sha=f75aef2cc3d7
// @port DiabloUI/diabloui.cpp|devilution::UiRenderItems(const std::vector<std::unique_ptr<UiItemBase>> &items) sha=9c1fc5ac3e6b
pub fn ui_render_items(ctx: &mut Ctx, items: &[UiItemRef]) {
    for item in items {
        ui_render_item(ctx, item);
    }
}

/// Original: `devilution::UiItemMouseEvents` (both overloads, DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::UiItemMouseEvents(SDL_Event *event, const std::vector<UiItemBase *> &items) sha=2d6db47218ed
// @port DiabloUI/diabloui.cpp|devilution::UiItemMouseEvents(SDL_Event *event, const std::vector<std::unique_ptr<UiItemBase>> &items) sha=d363d14147ee
pub fn ui_item_mouse_events(ctx: &mut Ctx, event: &Event, items: &[UiItemRef]) -> bool {
    if items.is_empty() {
        return false;
    }
    let mut handled = false;
    for item in items {
        if handle_mouse_event(ctx, event, item) {
            handled = true;
            break;
        }
    }
    if let Event::MouseButtonUp { button: BUTTON_LEFT, .. } = event {
        ctx.diablo_ui.scroll_bar_state = ScrollBarState::default();
        for item in items {
            let ty = item.borrow().get_type();
            if ty == UiType::Button {
                crate::diablo_ui::button::handle_global_mouse_up_button(item);
            } else if ty == UiType::List {
                item.borrow_mut().as_list_mut().release();
            }
        }
    }
    handled
}

/// Original: `devilution::DrawMouse` (DiabloUI/diabloui.cpp).
// @port DiabloUI/diabloui.cpp|devilution::DrawMouse() sha=5adfe742069c
pub fn draw_mouse(ctx: &mut Ctx) {
    if ctx.controls.control_device != ControlTypes::KeyboardAndMouse || crate::hwcursor::is_hardware_cursor(ctx) {
        return;
    }
    let Some(cursor) = ctx.diablo_ui.art_cursor.clone() else { return };
    let out = diablo_ui_surface(ctx);
    render_clx_sprite(&out, &cursor.get(0), ctx.diablo.mouse_position);
}

/// Original: `devilution::CheckArchivesUpToDate` (diablo.cpp).
// @port diablo.cpp|devilution::CheckArchivesUpToDate() sha=4ad13bc701b1
pub fn check_archives_up_to_date(ctx: &mut Ctx) {
    use crate::utils::language::tr;
    let devilutionx_mpq_out_of_date = ctx
        .init
        .archives
        .devilutionx_mpq
        .as_ref()
        .is_some_and(|a| !a.has_file("data\\charbg.clx") || a.has_file("fonts\\12-00.bin"));
    let fonts_mpq_out_of_date = crate::init::are_extra_fonts_out_of_date(ctx);
    if devilutionx_mpq_out_of_date && fonts_mpq_out_of_date {
        crate::appfat::app_fatal(ctx, &tr("Please update devilutionx.mpq and fonts.mpq to the latest version"));
    } else if devilutionx_mpq_out_of_date {
        crate::appfat::app_fatal(
            ctx,
            &tr("Failed to load UI resources.\n\nMake sure devilutionx.mpq is in the game folder and that it is up to date."),
        );
    } else if fonts_mpq_out_of_date {
        crate::appfat::app_fatal(ctx, &tr("Please update fonts.mpq to the latest version"));
    }
}
