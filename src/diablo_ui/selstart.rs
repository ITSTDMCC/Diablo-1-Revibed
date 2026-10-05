//! `Source/DiabloUI/selstart.cpp`: asks whether to start Hellfire or Diablo.

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{load_background_art, ui_add_background, ui_add_logo, ui_clear_screen, ui_init_list, ui_poll_and_render, ui_render_items};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiListItem, UiListItemRef};
use crate::engine::render::text_render::UiFlags;
use crate::options::StartUpGameMode;
use crate::utils::language::tr;

/// Globals of selstart.cpp.
#[derive(Default)]
pub struct SelStartState {
    /// `endMenu`
    end_menu: bool,
    /// `vecDialogItems` values (the callbacks only need the item values).
    item_values: Vec<i32>,
}

/// Original: `ItemSelected` (DiabloUI/selstart.cpp).
// @port DiabloUI/selstart.cpp|devilution::ItemSelected(int value) sha=7c0b6c37a884
fn item_selected(ctx: &mut Ctx, value: i32) {
    let option = ctx.diablo_ui.selstart.item_values[value as usize];
    if let Some(cb) = ctx.options.start_up.game_mode.set_raw(option) {
        crate::options::run_option_callback(ctx, cb);
    }
    crate::options::save_options(ctx);
    ctx.diablo_ui.selstart.end_menu = true;
}

/// Original: `EscPressed` (DiabloUI/selstart.cpp).
// @port DiabloUI/selstart.cpp|devilution::EscPressed() sha=08905afde5c5
fn esc_pressed(ctx: &mut Ctx) {
    ctx.diablo_ui.selstart.end_menu = true;
}

/// Original: `devilution::UiSelStartUpGameOption` (DiabloUI/selstart.cpp).
// @port DiabloUI/selstart.cpp|devilution::UiSelStartUpGameOption() sha=e1b793308af0
pub fn ui_sel_start_up_game_option(ctx: &mut Ctx) {
    ctx.diablo_ui.art_background_widescreen = crate::engine::load_sprites::load_optional_clx(ctx, "ui_art\\mainmenuw.clx");
    load_background_art(ctx, "ui_art\\mainmenu", 1);
    let mut vec_dialog: Vec<UiItemRef> = Vec::new();
    ui_add_background(ctx, &mut vec_dialog);
    ui_add_logo(ctx, &mut vec_dialog);

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let values = vec![StartUpGameMode::Hellfire as i32, StartUpGameMode::Diablo as i32];
    let vec_dialog_items: Vec<UiListItemRef> = vec![
        UiListItem::new(&tr("Enter Hellfire"), values[0], UiFlags::NONE),
        UiListItem::new(&tr("Switch to Diablo"), values[1], UiFlags::NONE),
    ];
    ctx.diablo_ui.selstart.item_values = values;
    vec_dialog.push(UiItem::list(
        &vec_dialog_items,
        vec_dialog_items.len(),
        ui_position.x + 64,
        ui_position.y + 240,
        510,
        43,
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_42 | UiFlags::COLOR_UI_GOLD,
        5,
    ));

    ui_init_list(ctx, None, Some(item_selected), Some(esc_pressed), &vec_dialog, true, None, None, 0);

    ctx.diablo_ui.selstart.end_menu = false;
    while !ctx.diablo_ui.selstart.end_menu {
        ui_clear_screen(ctx);
        ui_render_items(ctx, &vec_dialog);
        ui_poll_and_render(ctx, None);
    }

    ctx.diablo_ui.art_background = None;
    ctx.diablo_ui.art_background_widescreen = None;
    ctx.diablo_ui.selstart.item_values.clear();
}
