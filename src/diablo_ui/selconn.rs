//! `Source/DiabloUI/multi/selconn.cpp`: choosing the multiplayer connection. ZeroTier is not
//! ported, so the list holds "Client-Server (TCP)" and "Offline", as in a DevilutionX build with
//! `DISABLE_ZERO_TIER`.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{
    load_background_art, ui_add_background, ui_add_logo, ui_clear_screen, ui_focus_navigation_esc, ui_focus_navigation_select, ui_init_list, ui_poll_and_render,
};
use crate::diablo_ui::selgame::{CONNECTION_NAMES, SELCONN_LOOPBACK, SELCONN_TCP, SELCONN_ZT};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiListItem, UiListItemRef};
use crate::engine::render::text_render::{word_wrap_string, GameFontTables, UiFlags};
use crate::engine::surface::Rect;
use crate::multi::MAX_PLRS;
use crate::utils::language::tr;

const DESCRIPTION_WIDTH: i32 = 205;

/// Globals of selconn.cpp (`provider` is in `SelGameState`).
#[derive(Default)]
pub struct SelConnState {
    selconn_max_players: Rc<RefCell<String>>,
    selconn_description: Rc<RefCell<String>>,
    selconn_gateway: Rc<RefCell<String>>,
    selconn_return_value: bool,
    selconn_end_menu: bool,
    vec_conn_items: Vec<UiListItemRef>,
    vec_sel_conn_dlg: Vec<UiItemRef>,
}

/// Original: `SelconnLoad` (DiabloUI/multi/selconn.cpp).
// @port DiabloUI/multi/selconn.cpp|devilution::SelconnLoad() sha=96cfe6f59dfb
fn selconn_load(ctx: &mut Ctx) {
    load_background_art(ctx, "ui_art\\selconn", 1);
    let s = &mut ctx.diablo_ui.selconn;
    s.vec_conn_items.push(UiListItem::new(&tr(CONNECTION_NAMES[SELCONN_TCP as usize]), SELCONN_TCP, UiFlags::NONE));
    s.vec_conn_items.push(UiListItem::new(&tr(CONNECTION_NAMES[SELCONN_LOOPBACK as usize]), SELCONN_LOOPBACK, UiFlags::NONE));

    let mut dlg = std::mem::take(&mut ctx.diablo_ui.selconn.vec_sel_conn_dlg);
    ui_add_background(ctx, &mut dlg);
    ui_add_logo(ctx, &mut dlg);

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let s = &ctx.diablo_ui.selconn;
    let (x, y) = (ui_position.x, ui_position.y);
    dlg.push(UiItem::art_text(&tr("Multi Player Game"), Rect::new(x + 24, y + 161, 590, 35), UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER, 3, -1));
    dlg.push(UiItem::art_text_dynamic(s.selconn_max_players.clone(), Rect::new(x + 35, y + 218, DESCRIPTION_WIDTH, 21), UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK, 1, -1));
    dlg.push(UiItem::art_text(&tr("Requirements:"), Rect::new(x + 35, y + 256, DESCRIPTION_WIDTH, 21), UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK, 1, -1));
    dlg.push(UiItem::art_text_dynamic(s.selconn_description.clone(), Rect::new(x + 35, y + 275, DESCRIPTION_WIDTH, 66), UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK, 1, 16));
    dlg.push(UiItem::art_text(&tr("no gateway needed"), Rect::new(x + 30, y + 356, 220, 31), UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_SILVER, 0, -1));
    dlg.push(UiItem::art_text_dynamic(s.selconn_gateway.clone(), Rect::new(x + 35, y + 393, DESCRIPTION_WIDTH, 21), UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK, 1, -1));
    dlg.push(UiItem::art_text(&tr("Select Connection"), Rect::new(x + 300, y + 211, 295, 33), UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER, 3, -1));
    fn no_action(_: &mut Ctx) {}
    dlg.push(UiItem::art_text_button(
        &tr("Change Gateway"),
        no_action,
        Rect::new(x + 16, y + 427, 250, 35),
        UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD | UiFlags::ELEMENT_HIDDEN,
    ));
    let items = s.vec_conn_items.clone();
    dlg.push(UiItem::list(
        &items,
        items.len(),
        x + 305,
        y + 256,
        285,
        26,
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_12 | UiFlags::VERTICAL_CENTER | UiFlags::COLOR_UI_GOLD_DARK,
        1,
    ));
    dlg.push(UiItem::art_text_button(&tr("OK"), ui_focus_navigation_select, Rect::new(x + 299, y + 427, 140, 35), UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD));
    dlg.push(UiItem::art_text_button(&tr("Cancel"), ui_focus_navigation_esc, Rect::new(x + 454, y + 427, 144, 35), UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD));

    ui_init_list(ctx, Some(selconn_focus), Some(selconn_select), Some(selconn_esc), &dlg, true, None, None, 0);
    ctx.diablo_ui.selconn.vec_sel_conn_dlg = dlg;
}

/// Original: `SelconnFree` (DiabloUI/multi/selconn.cpp).
// @port DiabloUI/multi/selconn.cpp|devilution::SelconnFree() sha=d1dca2604e84
fn selconn_free(ctx: &mut Ctx) {
    ctx.diablo_ui.art_background = None;
    ctx.diablo_ui.selconn.vec_conn_items.clear();
    ctx.diablo_ui.selconn.vec_sel_conn_dlg.clear();
}

/// Original: `SelconnEsc` (DiabloUI/multi/selconn.cpp).
// @port DiabloUI/multi/selconn.cpp|devilution::SelconnEsc() sha=c3d45215edb4
fn selconn_esc(ctx: &mut Ctx) {
    ctx.diablo_ui.selconn.selconn_return_value = false;
    ctx.diablo_ui.selconn.selconn_end_menu = true;
}

/// Original: `SelconnFocus` (DiabloUI/multi/selconn.cpp).
// @port DiabloUI/multi/selconn.cpp|devilution::SelconnFocus(int value) sha=965502a5fb5f
fn selconn_focus(ctx: &mut Ctx, value: i32) {
    let mut players = MAX_PLRS;
    let m_value = ctx.diablo_ui.selconn.vec_conn_items[value as usize].borrow().m_value;
    let mut description = String::new();
    match m_value {
        SELCONN_TCP => {
            description = tr("All computers must be connected to a TCP-compatible network.");
            players = MAX_PLRS;
        }
        SELCONN_ZT => {
            description = tr("All computers must be connected to the internet.");
            players = MAX_PLRS;
        }
        SELCONN_LOOPBACK => {
            description = tr("Play by yourself with no network exposure.");
            players = 1;
        }
        _ => {}
    }
    let max_players = tr("Players Supported: {:d}").replacen("{:d}", &players.to_string(), 1);
    *ctx.diablo_ui.selconn.selconn_max_players.borrow_mut() = crate::utils::utf8::copy_utf8(&max_players, 64);
    let description = crate::utils::utf8::copy_utf8(&description, 256);
    let wrapped = word_wrap_string(ctx, &description, DESCRIPTION_WIDTH as u32, GameFontTables::GameFont12, 1);
    *ctx.diablo_ui.selconn.selconn_description.borrow_mut() = crate::utils::utf8::copy_utf8(&wrapped, 256);
}

/// Original: `SelconnSelect` (DiabloUI/multi/selconn.cpp).
// @port DiabloUI/multi/selconn.cpp|devilution::SelconnSelect(int value) sha=92e7090165a6
fn selconn_select(ctx: &mut Ctx, value: i32) {
    let provider = ctx.diablo_ui.selconn.vec_conn_items[value as usize].borrow().m_value;
    ctx.diablo_ui.selgame.provider = provider;
    selconn_free(ctx);
    ctx.diablo_ui.selconn.selconn_end_menu = crate::storm::storm_net::snet_initialize_provider(ctx, provider as u32);
    selconn_load(ctx);
}

/// Original: `devilution::UiSelectProvider` (DiabloUI/multi/selconn.cpp). The game data the
/// original passes along is `sgGameInitInfo`, which the provider reads from `ctx`.
// @port DiabloUI/multi/selconn.cpp|devilution::UiSelectProvider(GameData *gameData) sha=3d02e30b055a
pub fn ui_select_provider(ctx: &mut Ctx) -> bool {
    selconn_load(ctx);
    ctx.diablo_ui.selconn.selconn_return_value = true;
    ctx.diablo_ui.selconn.selconn_end_menu = false;
    while !ctx.diablo_ui.selconn.selconn_end_menu {
        ui_clear_screen(ctx);
        ui_poll_and_render(ctx, None);
    }
    selconn_free(ctx);
    ctx.diablo_ui.selconn.selconn_return_value
}
