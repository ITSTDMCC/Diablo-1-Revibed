//! `Source/DiabloUI/selok.cpp`: a message with an OK button.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{
    load_background_art, ui_add_background, ui_add_logo, ui_clear_screen, ui_init_list, ui_load_black_background, ui_poll_and_render, ui_render_items,
};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiListItem, UiListItemRef};
use crate::engine::render::text_render::{word_wrap_string, GameFontTables, UiFlags};
use crate::engine::surface::Rect;
use crate::utils::language::tr;

const MESSAGE_WIDTH: u32 = 400;

/// Globals of selok.cpp.
#[derive(Default)]
pub struct SelOkState {
    /// `dialogText`
    dialog_text: Rc<RefCell<String>>,
    pub selok_end_menu: bool,
    vec_sel_ok_dialog_items: Vec<UiListItemRef>,
    vec_sel_ok_dialog: Vec<UiItemRef>,
}

/// Original: `devilution::selok_Free` (DiabloUI/selok.cpp).
// @port DiabloUI/selok.cpp|devilution::selok_Free() sha=cf476a353402
pub fn selok_free(ctx: &mut Ctx) {
    ctx.diablo_ui.art_background = None;
    ctx.diablo_ui.selok.vec_sel_ok_dialog_items.clear();
    ctx.diablo_ui.selok.vec_sel_ok_dialog.clear();
}

/// Original: `devilution::selok_Select` (DiabloUI/selok.cpp).
// @port DiabloUI/selok.cpp|devilution::selok_Select(int) sha=90cb1229712f
pub fn selok_select(ctx: &mut Ctx, _value: i32) {
    ctx.diablo_ui.selok.selok_end_menu = true;
}

/// Original: `devilution::selok_Esc` (DiabloUI/selok.cpp).
// @port DiabloUI/selok.cpp|devilution::selok_Esc() sha=127129d752b1
pub fn selok_esc(ctx: &mut Ctx) {
    ctx.diablo_ui.selok.selok_end_menu = true;
}

/// Original: `devilution::UiSelOkDialog` (DiabloUI/selok.cpp).
// @port DiabloUI/selok.cpp|devilution::UiSelOkDialog(const char *title, const char *body, bool background) sha=8d04497af9bc
pub fn ui_sel_ok_dialog(ctx: &mut Ctx, title: Option<&str>, body: &str, background: bool) {
    if !background {
        ui_load_black_background(ctx);
    } else if !ctx.init.gb_is_spawn {
        load_background_art(ctx, "ui_art\\mainmenu", 1);
    } else {
        load_background_art(ctx, "ui_art\\swmmenu", 1);
    }

    let mut dialog = std::mem::take(&mut ctx.diablo_ui.selok.vec_sel_ok_dialog);
    ui_add_background(ctx, &mut dialog);
    ui_add_logo(ctx, &mut dialog);

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let text = ctx.diablo_ui.selok.dialog_text.clone();

    if let Some(title) = title {
        let rect1 = Rect::new(ui_position.x + 24, ui_position.y + 161, 590, 35);
        dialog.push(UiItem::art_text(title, rect1, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER, 3, -1));
        let rect2 = Rect::new(ui_position.x + 140, ui_position.y + 210, 560, 168);
        dialog.push(UiItem::art_text_dynamic(text.clone(), rect2, UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_SILVER, 1, -1));
    } else {
        let rect1 = Rect::new(ui_position.x + 140, ui_position.y + 197, 560, 168);
        dialog.push(UiItem::art_text_dynamic(text.clone(), rect1, UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_SILVER, 1, -1));
    }

    let item = UiListItem::new(&tr("OK"), 0, UiFlags::NONE);
    ctx.diablo_ui.selok.vec_sel_ok_dialog_items.push(item);
    let items = ctx.diablo_ui.selok.vec_sel_ok_dialog_items.clone();
    dialog.push(UiItem::list(
        &items,
        1,
        ui_position.x + 230,
        ui_position.y + 390,
        180,
        35,
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
        1,
    ));

    let wrapped = word_wrap_string(ctx, body, MESSAGE_WIDTH, GameFontTables::GameFont24, 1);
    *text.borrow_mut() = crate::utils::utf8::copy_utf8(&wrapped, 256);

    ui_init_list(ctx, None, Some(selok_select), Some(selok_esc), &dialog, false, None, None, 0);
    ctx.diablo_ui.selok.vec_sel_ok_dialog = dialog.clone();

    ctx.diablo_ui.selok.selok_end_menu = false;
    while !ctx.diablo_ui.selok.selok_end_menu {
        ui_clear_screen(ctx);
        ui_render_items(ctx, &dialog);
        ui_poll_and_render(ctx, None);
    }

    selok_free(ctx);
}
