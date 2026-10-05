//! `Source/DiabloUI/selyesno.cpp`: a yes/no question.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{ui_add_background, ui_add_logo, ui_clear_screen, ui_init_list, ui_load_black_background, ui_poll_and_render, ui_render_items};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiListItem, UiListItemRef};
use crate::engine::render::text_render::{word_wrap_string, GameFontTables, UiFlags};
use crate::engine::surface::Rect;
use crate::utils::language::tr;

const MESSAGE_WIDTH: i32 = 400;

/// Globals of selyesno.cpp.
#[derive(Default)]
pub struct SelYesNoState {
    selyesno_end_menu: bool,
    selyesno_value: bool,
    selyesno_confirmation_message: Rc<RefCell<String>>,
    vec_sel_yes_no_dialog_items: Vec<UiListItemRef>,
    vec_sel_yes_no_dialog: Vec<UiItemRef>,
}

/// Original: `SelyesnoFree` (DiabloUI/selyesno.cpp).
// @port DiabloUI/selyesno.cpp|devilution::SelyesnoFree() sha=ccb5cf840f79
fn selyesno_free(ctx: &mut Ctx) {
    ctx.diablo_ui.art_background = None;
    ctx.diablo_ui.selyesno.vec_sel_yes_no_dialog_items.clear();
    ctx.diablo_ui.selyesno.vec_sel_yes_no_dialog.clear();
}

/// Original: `SelyesnoSelect` (DiabloUI/selyesno.cpp).
// @port DiabloUI/selyesno.cpp|devilution::SelyesnoSelect(int value) sha=52eed16dba51
fn selyesno_select(ctx: &mut Ctx, value: i32) {
    let v = ctx.diablo_ui.selyesno.vec_sel_yes_no_dialog_items[value as usize].borrow().m_value;
    ctx.diablo_ui.selyesno.selyesno_value = v == 0;
    ctx.diablo_ui.selyesno.selyesno_end_menu = true;
}

/// Original: `SelyesnoEsc` (DiabloUI/selyesno.cpp).
// @port DiabloUI/selyesno.cpp|devilution::SelyesnoEsc() sha=31a8190ff8c7
fn selyesno_esc(ctx: &mut Ctx) {
    ctx.diablo_ui.selyesno.selyesno_value = false;
    ctx.diablo_ui.selyesno.selyesno_end_menu = true;
}

/// Original: `devilution::UiSelHeroYesNoDialog` (DiabloUI/selyesno.cpp).
// @port DiabloUI/selyesno.cpp|devilution::UiSelHeroYesNoDialog(const char *title, const char *body) sha=362d3138a07d
pub fn ui_sel_hero_yes_no_dialog(ctx: &mut Ctx, title: &str, body: &str) -> bool {
    ui_load_black_background(ctx);
    let mut dialog = std::mem::take(&mut ctx.diablo_ui.selyesno.vec_sel_yes_no_dialog);
    ui_add_background(ctx, &mut dialog);
    ui_add_logo(ctx, &mut dialog);

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let message = ctx.diablo_ui.selyesno.selyesno_confirmation_message.clone();

    let rect1 = Rect::new(ui_position.x + 24, ui_position.y + 161, 590, 35);
    dialog.push(UiItem::art_text(title, rect1, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER, 3, -1));

    let rect2 = Rect::new(ui_position.x + 120, ui_position.y + 236, MESSAGE_WIDTH, 168);
    dialog.push(UiItem::art_text_dynamic(message.clone(), rect2, UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_SILVER, 1, -1));

    ctx.diablo_ui.selyesno.vec_sel_yes_no_dialog_items.push(UiListItem::new(&tr("Yes"), 0, UiFlags::NONE));
    ctx.diablo_ui.selyesno.vec_sel_yes_no_dialog_items.push(UiListItem::new(&tr("No"), 1, UiFlags::NONE));
    let items = ctx.diablo_ui.selyesno.vec_sel_yes_no_dialog_items.clone();
    dialog.push(UiItem::list(
        &items,
        items.len(),
        ui_position.x + 230,
        ui_position.y + 390,
        180,
        35,
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
        1,
    ));

    let wrapped = word_wrap_string(ctx, body, MESSAGE_WIDTH as u32, GameFontTables::GameFont24, 1);
    *message.borrow_mut() = crate::utils::utf8::copy_utf8(&wrapped, 256);

    ui_init_list(ctx, None, Some(selyesno_select), Some(selyesno_esc), &dialog, true, None, None, 0);
    ctx.diablo_ui.selyesno.vec_sel_yes_no_dialog = dialog.clone();

    ctx.diablo_ui.selyesno.selyesno_value = true;
    ctx.diablo_ui.selyesno.selyesno_end_menu = false;
    while !ctx.diablo_ui.selyesno.selyesno_end_menu {
        ui_clear_screen(ctx);
        ui_render_items(ctx, &dialog);
        ui_poll_and_render(ctx, None);
    }

    selyesno_free(ctx);
    ctx.diablo_ui.selyesno.selyesno_value
}
