//! `Source/DiabloUI/mainmenu.cpp`: the main menu.

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{
    load_background_art, ui_add_background, ui_add_logo, ui_clear_screen, ui_init_list, ui_poll_and_render, MainmenuSelections,
};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiListItem, UiListItemRef};
use crate::engine::render::text_render::UiFlags;
use crate::engine::surface::Rect;
use crate::utils::language::tr;

/// Globals of mainmenu.cpp.
pub struct MainMenuState {
    /// `mainmenu_attract_time_out` (seconds)
    mainmenu_attract_time_out: i32,
    /// `dwAttractTicks`
    dw_attract_ticks: u32,
    vec_main_menu_dialog: Vec<UiItemRef>,
    vec_menu_items: Vec<UiListItemRef>,
    /// `MainMenuResult`
    main_menu_result: MainmenuSelections,
}

impl Default for MainMenuState {
    fn default() -> Self {
        MainMenuState {
            mainmenu_attract_time_out: 0,
            dw_attract_ticks: 0,
            vec_main_menu_dialog: Vec::new(),
            vec_menu_items: Vec::new(),
            main_menu_result: MainmenuSelections::None,
        }
    }
}

/// Original: `UiMainMenuSelect` (DiabloUI/mainmenu.cpp).
// @port DiabloUI/mainmenu.cpp|devilution::UiMainMenuSelect(int value) sha=a9b596fb2e7f
fn ui_main_menu_select(ctx: &mut Ctx, value: i32) {
    let v = ctx.diablo_ui.mainmenu.vec_menu_items[value as usize].borrow().m_value;
    ctx.diablo_ui.mainmenu.main_menu_result = MainmenuSelections::from_i32(v);
}

/// Original: `MainmenuEsc` (DiabloUI/mainmenu.cpp).
// @port DiabloUI/mainmenu.cpp|devilution::MainmenuEsc() sha=1da038c47057
fn mainmenu_esc(ctx: &mut Ctx) {
    let last = ctx.diablo_ui.mainmenu.vec_menu_items.len() - 1;
    if ctx.diablo_ui.selected_item == last {
        ui_main_menu_select(ctx, last as i32);
    } else {
        ctx.diablo_ui.selected_item = last;
    }
}

/// Original: `MainmenuLoad` (DiabloUI/mainmenu.cpp).
// @port DiabloUI/mainmenu.cpp|devilution::MainmenuLoad(const char *name) sha=70dda321c48d
fn mainmenu_load(ctx: &mut Ctx, name: &str) {
    use MainmenuSelections as M;
    let exit_text = if ctx.init.gb_is_hellfire { tr("Exit Hellfire") } else { tr("Exit Diablo") };
    let items = vec![
        UiListItem::new(&tr("Single Player"), M::SinglePlayer as i32, UiFlags::NONE),
        UiListItem::new(&tr("Multi Player"), M::Multiplayer as i32, UiFlags::NONE),
        UiListItem::new(&tr("Settings"), M::Settings as i32, UiFlags::NONE),
        UiListItem::new(&tr("Support"), M::ShowSupport as i32, UiFlags::NONE),
        UiListItem::new(&tr("Show Credits"), M::ShowCredits as i32, UiFlags::NONE),
        UiListItem::new(&exit_text, M::ExitDiablo as i32, UiFlags::NONE),
    ];
    ctx.diablo_ui.mainmenu.vec_menu_items.extend(items);

    if !ctx.init.gb_is_spawn || ctx.init.gb_is_hellfire {
        if ctx.init.gb_is_hellfire {
            ctx.diablo_ui.art_background_widescreen = crate::engine::load_sprites::load_optional_clx(ctx, "ui_art\\mainmenuw.clx");
        }
        load_background_art(ctx, "ui_art\\mainmenu", 1);
    } else {
        load_background_art(ctx, "ui_art\\swmmenu", 1);
    }

    let mut dialog = std::mem::take(&mut ctx.diablo_ui.mainmenu.vec_main_menu_dialog);
    ui_add_background(ctx, &mut dialog);
    ui_add_logo(ctx, &mut dialog);

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);

    if ctx.init.gb_is_spawn && ctx.init.gb_is_hellfire {
        let rect1 = Rect::new(ui_position.x, ui_position.y + 145, 640, 30);
        dialog.push(UiItem::art_text(&tr("Shareware"), rect1, UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER | UiFlags::ALIGN_CENTER, 8, -1));
    }

    let menu_items = ctx.diablo_ui.mainmenu.vec_menu_items.clone();
    dialog.push(UiItem::list(
        &menu_items,
        menu_items.len(),
        ui_position.x + 64,
        ui_position.y + 192,
        510,
        43,
        UiFlags::FONT_SIZE_42 | UiFlags::COLOR_UI_GOLD | UiFlags::ALIGN_CENTER,
        5,
    ));

    let rect2 = Rect::new(17, ctx.dx.gn_screen_height - 36, 605, 21);
    dialog.push(UiItem::art_text(name, rect2, UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK, 1, -1));

    ui_init_list(ctx, None, Some(ui_main_menu_select), Some(mainmenu_esc), &dialog, true, None, None, 0);
    ctx.diablo_ui.mainmenu.vec_main_menu_dialog = dialog;
}

/// Original: `MainmenuFree` (DiabloUI/mainmenu.cpp).
// @port DiabloUI/mainmenu.cpp|devilution::MainmenuFree() sha=7b3fdccee39b
fn mainmenu_free(ctx: &mut Ctx) {
    ctx.diablo_ui.art_background_widescreen = None;
    ctx.diablo_ui.art_background = None;
    ctx.diablo_ui.mainmenu.vec_main_menu_dialog.clear();
    ctx.diablo_ui.mainmenu.vec_menu_items.clear();
}

/// Original: `devilution::mainmenu_restart_repintro` (DiabloUI/mainmenu.cpp).
// @port DiabloUI/mainmenu.cpp|devilution::mainmenu_restart_repintro() sha=0f307ebaa3c4
pub fn mainmenu_restart_repintro(ctx: &mut Ctx) {
    let t = ctx.diablo_ui.mainmenu.mainmenu_attract_time_out;
    ctx.diablo_ui.mainmenu.dw_attract_ticks = ctx.platform.ticks().wrapping_add((t * 1000) as u32);
}

/// Original: `devilution::UiMainMenuDialog` (DiabloUI/mainmenu.cpp).
// @port DiabloUI/mainmenu.cpp|devilution::UiMainMenuDialog(const char *name, _mainmenu_selections *pdwResult, int attractTimeOut) sha=26f03f470964
pub fn ui_main_menu_dialog(ctx: &mut Ctx, name: &str, pdw_result: &mut MainmenuSelections, attract_time_out: i32) -> bool {
    ctx.diablo_ui.mainmenu.main_menu_result = MainmenuSelections::None;
    while ctx.diablo_ui.mainmenu.main_menu_result == MainmenuSelections::None {
        ctx.diablo_ui.mainmenu.mainmenu_attract_time_out = attract_time_out;
        mainmenu_load(ctx, name);

        mainmenu_restart_repintro(ctx); // for automatic starts

        while ctx.diablo_ui.mainmenu.main_menu_result == MainmenuSelections::None {
            ui_clear_screen(ctx);
            ui_poll_and_render(ctx, None);
            let have_data = ctx.init.archives.diabdat_mpq.is_some() || ctx.init.archives.hellfire_mpq.is_some();
            if ctx.platform.ticks() >= ctx.diablo_ui.mainmenu.dw_attract_ticks && have_data {
                ctx.diablo_ui.mainmenu.main_menu_result = MainmenuSelections::AttractMode;
            }
        }

        mainmenu_free(ctx);
    }
    *pdw_result = ctx.diablo_ui.mainmenu.main_menu_result;
    true
}
