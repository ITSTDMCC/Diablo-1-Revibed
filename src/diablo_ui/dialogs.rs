//! `Source/DiabloUI/dialogs.cpp`: OK/error dialogs.

use crate::ctx::Ctx;
use crate::diablo_ui::button::{DIALOG_BUTTON_HEIGHT, DIALOG_BUTTON_WIDTH};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef};
use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList};
use crate::engine::render::text_render::{word_wrap_string, GameFontTables, UiFlags};
use crate::engine::surface::Rect;
use crate::platform::log;
use crate::utils::language::tr;

#[derive(Default)]
pub struct DialogsState {
    /// `inDialog` (static in UiOkDialog)
    in_dialog: bool,
    /// `ownedDialogSprite`
    owned_dialog_sprite: Option<ClxSpriteList>,
    /// `dialogEnd`
    dialog_end: bool,
    /// `vecOkDialog`
    vec_ok_dialog: Vec<UiItemRef>,
}

/// Original: `DialogActionOK` (DiabloUI/dialogs.cpp).
// @port DiabloUI/dialogs.cpp|devilution::DialogActionOK() sha=e04d00ed1523
fn dialog_action_ok(ctx: &mut Ctx) {
    ctx.dialogs.dialog_end = true;
}

/// Original: `LoadDialogSprite` (DiabloUI/dialogs.cpp).
// @port DiabloUI/dialogs.cpp|devilution::LoadDialogSprite(bool hasCaption, bool isError) sha=ccdbc5edd12d
fn load_dialog_sprite(ctx: &mut Ctx, has_caption: bool, is_error: bool) -> Option<ClxSprite> {
    const TRANSPARENT_COLOR: u8 = 255;
    use crate::engine::load_sprites::{load_optional_clx, load_pcx};
    let sprite = if !has_caption {
        load_pcx(ctx, if is_error { "ui_art\\srpopup" } else { "ui_art\\spopup" }, Some(TRANSPARENT_COLOR), None, true)
    } else if is_error {
        match load_optional_clx(ctx, "ui_art\\dvl_lrpopup.clx") {
            Some(s) => Some(s),
            None => load_pcx(ctx, "ui_art\\lrpopup", Some(TRANSPARENT_COLOR), None, true),
        }
    } else {
        load_pcx(ctx, "ui_art\\lpopup", Some(TRANSPARENT_COLOR), None, true)
    };
    ctx.dialogs.owned_dialog_sprite = sprite;
    ctx.dialogs.owned_dialog_sprite.as_ref().map(|s| s.get(0))
}

/// Original: `Init` (DiabloUI/dialogs.cpp).
// @port DiabloUI/dialogs.cpp|devilution::Init(string_view caption, string_view text, bool error, bool renderBehind) sha=b82d12af3569
fn init(ctx: &mut Ctx, caption: &str, text: &str, error: bool, render_behind: bool) -> bool {
    if !render_behind && !crate::diablo_ui::diabloui::ui_load_black_background(ctx) {
        ctx.platform.show_cursor(true);
    }
    crate::diablo_ui::button::load_dialog_button_graphics(ctx);

    let Some(dialog_sprite) = load_dialog_sprite(ctx, !caption.is_empty(), error) else {
        return false;
    };

    let dialog_width = dialog_sprite.width() as i32;
    let text_width = dialog_width - 40;

    let wrapped_text = word_wrap_string(ctx, text, text_width as u32, GameFontTables::FontSizeDialog, 1);

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let (w, h) = (dialog_sprite.width() as i32, dialog_sprite.height() as i32);
    let (bw, bh) = (DIALOG_BUTTON_WIDTH, DIALOG_BUTTON_HEIGHT);
    let v = &mut ctx.dialogs.vec_ok_dialog;
    if caption.is_empty() {
        v.push(UiItem::image_clx(dialog_sprite, Rect::new(ui_position.x + 180, ui_position.y + 168, w, h), UiFlags::NONE));
        v.push(UiItem::text(&wrapped_text, Rect::new(ui_position.x + 200, ui_position.y + 211, text_width, 80), UiFlags::ALIGN_CENTER | UiFlags::COLOR_DIALOG_WHITE));
        v.push(UiItem::button(&tr("OK"), dialog_action_ok, Rect::new(ui_position.x + 265, ui_position.y + 265, bw, bh), UiFlags::NONE));
    } else {
        v.push(UiItem::image_clx(dialog_sprite, Rect::new(ui_position.x + 127, ui_position.y + 100, w, h), UiFlags::NONE));
        v.push(UiItem::text(caption, Rect::new(ui_position.x + 147, ui_position.y + 110, text_width, 20), UiFlags::ALIGN_CENTER | UiFlags::COLOR_YELLOW));
        v.push(UiItem::text(&wrapped_text, Rect::new(ui_position.x + 147, ui_position.y + 141, text_width, 190), UiFlags::ALIGN_CENTER | UiFlags::COLOR_DIALOG_WHITE));
        v.push(UiItem::button(&tr("OK"), dialog_action_ok, Rect::new(ui_position.x + 264, ui_position.y + 335, bw, bh), UiFlags::NONE));
    }
    true
}

/// Original: `Deinit` (DiabloUI/dialogs.cpp).
// @port DiabloUI/dialogs.cpp|devilution::Deinit() sha=bccea50e087e
fn deinit(ctx: &mut Ctx) {
    ctx.dialogs.owned_dialog_sprite = None;
    ctx.dialogs.vec_ok_dialog.clear();
    crate::diablo_ui::button::free_dialog_button_graphics(ctx);
}

/// Original: `DialogLoop` (DiabloUI/dialogs.cpp).
// @port DiabloUI/dialogs.cpp|devilution::DialogLoop(const std::vector<std::unique_ptr<UiItemBase>> &items, const std::vector<std::unique_ptr<UiItemBase>> &renderBehind) sha=9ccf5d55f14d
fn dialog_loop(ctx: &mut Ctx, items: &[UiItemRef], render_behind: &[UiItemRef]) {
    use crate::controls::controller::MenuAction;
    use crate::diablo_ui::diabloui::*;
    use crate::platform::events::Event;
    ctx.dialogs.dialog_end = false;
    loop {
        while let Some(event) = crate::engine::events::poll_event(ctx) {
            match event {
                Event::MouseButtonDown { .. } | Event::MouseButtonUp { .. } => {
                    ui_item_mouse_events(ctx, &event, items);
                }
                _ => {
                    for menu_action in crate::controls::controller::get_menu_actions(ctx, &event) {
                        if !matches!(menu_action, MenuAction::Back | MenuAction::Select) {
                            continue;
                        }
                        ctx.dialogs.dialog_end = true;
                        break;
                    }
                }
            }
            ui_handle_events(ctx, &event);
        }

        ui_clear_screen(ctx);
        ui_render_items(ctx, render_behind);
        ui_render_list_items(ctx);
        ui_render_items(ctx, items);
        draw_mouse(ctx);
        ui_fade_in(ctx);
        if ctx.dialogs.dialog_end {
            break;
        }
    }
}

/// Original: `UiOkDialog` (DiabloUI/dialogs.cpp). Before the game window is active (or while
/// another dialog is open) the message goes to a system message box, as in the original.
// @port DiabloUI/dialogs.cpp|devilution::UiOkDialog(string_view caption, string_view text, bool error, const std::vector<std::unique_ptr<UiItemBase>> &renderBehind) sha=d809f0e6c0a3
fn ui_ok_dialog(ctx: &mut Ctx, caption: &str, text: &str, error: bool) {
    if !caption.is_empty() {
        log::error!("{}\n{}", caption, text);
    } else {
        log::error!("{}", text);
    }
    if !ctx.init.gb_active || ctx.dialogs.in_dialog {
        if !ctx.diablo.headless_mode {
            ctx.platform.show_cursor(true);
            show_message_box(ctx, caption, text);
        }
        return;
    }
    if crate::hwcursor::is_hardware_cursor(ctx) {
        ctx.platform.show_cursor(true);
    }

    let render_behind: &[UiItemRef] = &[];
    if !init(ctx, caption, text, error, !render_behind.is_empty()) {
        log::error!("{}
{}", caption, text);
        show_message_box(ctx, caption, text);
    }

    ctx.dialogs.in_dialog = true;
    // SDL_SetClipRect(DiabloUiSurface(), nullptr): the port's UI surface is never clipped.
    let items = ctx.dialogs.vec_ok_dialog.clone();
    dialog_loop(ctx, &items, render_behind);
    deinit(ctx);
    ctx.dialogs.in_dialog = false;
}

/// `SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, ...)`. Without a window (`DIABLO_HEADLESS`
/// test runs) the message only goes to the log, so an automated run never waits on a modal box.
fn show_message_box(ctx: &Ctx, caption: &str, text: &str) {
    if ctx.platform.headless {
        return;
    }
    crate::platform::win32::show_error_message_box(caption, text);
}

/// Original: `UiErrorOkDialog(string_view caption, string_view text, bool error)` (DiabloUI/dialogs.cpp).
// @port DiabloUI/dialogs.cpp|devilution::UiErrorOkDialog(string_view caption, string_view text, bool error) sha=5ece290ebb0e
pub fn ui_error_ok_dialog(ctx: &mut Ctx, caption: &str, text: &str, error: bool) {
    ui_ok_dialog(ctx, caption, text, error);
}
