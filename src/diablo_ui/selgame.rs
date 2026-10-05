//! `Source/DiabloUI/multi/selgame.cpp`: game creation/joining, difficulty and speed selection.
//! Single player uses the difficulty part through selhero's "dangerous hack".

use std::cell::RefCell;
use std::rc::Rc;

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{
    load_background_art, ui_add_background, ui_add_logo, ui_clear_screen, ui_focus_navigation_esc, ui_focus_navigation_select, ui_init_list, ui_init_list_clear,
    ui_poll_and_render,
};
use crate::diablo_ui::scrollbar::{load_scroll_bar, unload_scroll_bar};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiListItem, UiListItemRef};
use crate::engine::render::text_render::{word_wrap_string, GameFontTables, UiFlags};
use crate::engine::surface::Rect;
use crate::enums::*;
use crate::multi::{GameData, GameInfo};
use crate::utils::language::tr;

const DESCRIPTION_WIDTH: i32 = 205;

/// `SELCONN_ZT`, `SELCONN_TCP`, `SELCONN_LOOPBACK`
pub const SELCONN_ZT: i32 = 0;
pub const SELCONN_TCP: i32 = 1;
pub const SELCONN_LOOPBACK: i32 = 2;

/// `ConnectionNames` (selconn.cpp)
pub const CONNECTION_NAMES: [&str; 3] = ["ZeroTier", "Client-Server (TCP)", "Offline"];

/// Globals of selgame.cpp (and `provider` of selconn.cpp).
pub struct SelGameState {
    pub selgame_label: Rc<RefCell<String>>,
    pub selgame_ip: Rc<RefCell<String>>,
    pub selgame_password: Rc<RefCell<String>>,
    pub selgame_description: Rc<RefCell<String>>,
    pub selgame_title: String,
    pub selgame_entering_game: bool,
    pub selgame_selected_game: i32,
    pub selgame_end_menu: bool,
    pub gdw_player_id: i32,
    /// `nDifficulty`
    pub n_difficulty: _difficulty,
    pub n_tick_rate: i32,
    pub hero_level: i32,
    /// `m_game_data` points at `sgGameInitInfo` in every caller
    title: Rc<RefCell<String>>,
    vec_sel_game_dlg_items: Vec<UiListItemRef>,
    vec_sel_game_dialog: Vec<UiItemRef>,
    gamelist: Vec<GameInfo>,
    first_public_game_info_request_send: u32,
    highlighted_item: usize,
    /// statics of `RefreshGameList`
    last_request: u32,
    last_update: u32,
    /// `provider` (selconn.cpp)
    pub provider: i32,
}

impl Default for SelGameState {
    fn default() -> Self {
        SelGameState {
            selgame_label: Default::default(),
            selgame_ip: Default::default(),
            selgame_password: Default::default(),
            selgame_description: Default::default(),
            selgame_title: String::new(),
            selgame_entering_game: false,
            selgame_selected_game: 0,
            selgame_end_menu: false,
            gdw_player_id: 0,
            n_difficulty: DIFF_NORMAL,
            n_tick_rate: 0,
            hero_level: 0,
            title: Default::default(),
            vec_sel_game_dlg_items: Vec::new(),
            vec_sel_game_dialog: Vec::new(),
            gamelist: Vec::new(),
            first_public_game_info_request_send: 0,
            highlighted_item: 0,
            last_request: 0,
            last_update: 0,
            provider: 0,
        }
    }
}

fn set_text(t: &Rc<RefCell<String>>, s: &str, size: usize) {
    *t.borrow_mut() = crate::utils::utf8::copy_utf8(s, size);
}

/// Original: `selgame_FreeVectors` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_FreeVectors() sha=c5d60678be20
fn selgame_free_vectors(ctx: &mut Ctx) {
    ctx.diablo_ui.selgame.vec_sel_game_dlg_items.clear();
    ctx.diablo_ui.selgame.vec_sel_game_dialog.clear();
}

/// Original: `selgame_Init` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Init() sha=4fe38161db5a
fn selgame_init(ctx: &mut Ctx) {
    load_background_art(ctx, "ui_art\\selgame", 1);
    load_scroll_bar(ctx);
}

/// Original: `selgame_Free` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Free() sha=c46f237906df
fn selgame_free(ctx: &mut Ctx) {
    ctx.diablo_ui.art_background = None;
    unload_scroll_bar(ctx);
    selgame_free_vectors(ctx);
}

/// `GAME_ID` (diablo.h)
pub fn game_id(ctx: &Ctx) -> u32 {
    let be = |s: &[u8; 4]| u32::from_be_bytes(*s);
    match (ctx.init.gb_is_hellfire, ctx.init.gb_is_spawn) {
        (true, true) => be(b"HSHR"),
        (true, false) => be(b"HRTL"),
        (false, true) => be(b"DSHR"),
        (false, false) => be(b"DRTL"),
    }
}

/// Original: `IsGameCompatible` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::IsGameCompatible(const GameData &data) sha=f716d36994dc
fn is_game_compatible(ctx: &Ctx, data: &GameData) -> bool {
    let (major, minor, patch) = crate::multi::project_version();
    data.versionMajor == major && data.versionMinor == minor && data.versionPatch == patch && data.programid == game_id(ctx)
}

/// Original: `GetErrorMessageIncompatibility` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::GetErrorMessageIncompatibility(const GameData &data) sha=4ed54723da8a
fn get_error_message_incompatibility(ctx: &Ctx, data: &GameData) -> String {
    if data.programid != game_id(ctx) {
        let be = |s: &[u8; 4]| u32::from_be_bytes(*s);
        let game_mode = if data.programid == be(b"DRTL") {
            tr("Diablo")
        } else if data.programid == be(b"DSHR") {
            tr("Diablo Shareware")
        } else if data.programid == be(b"HRTL") {
            tr("Hellfire")
        } else if data.programid == be(b"HSHR") {
            tr("Hellfire Shareware")
        } else {
            return tr("The host is running a different game than you.");
        };
        return tr("The host is running a different game mode ({:s}) than you.").replacen("{:s}", &game_mode, 1);
    }
    tr("Your version {:s} does not match the host {:d}.{:d}.{:d}.")
        .replacen("{:s}", crate::multi::PROJECT_VERSION, 1)
        .replacen("{:d}", &data.versionMajor.to_string(), 1)
        .replacen("{:d}", &data.versionMinor.to_string(), 1)
        .replacen("{:d}", &data.versionPatch.to_string(), 1)
}

/// Original: `UiInitGameSelectionList` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::UiInitGameSelectionList(string_view search) sha=1916cf39bc5c
fn ui_init_game_selection_list(ctx: &mut Ctx, search: &str) {
    ctx.diablo_ui.selgame.selgame_entering_game = false;
    ctx.diablo_ui.selgame.selgame_selected_game = 0;

    let provider = ctx.diablo_ui.selgame.provider;
    if provider == SELCONN_LOOPBACK {
        ctx.diablo_ui.selgame.selgame_entering_game = true;
        selgame_game_selection_select(ctx, 0);
        return;
    }

    let previous = if provider == SELCONN_ZT {
        ctx.options.network.sz_previous_zt_game.clone()
    } else {
        ctx.options.network.sz_previous_host.clone()
    };
    set_text(&ctx.diablo_ui.selgame.selgame_ip, &previous, 129);

    selgame_free_vectors(ctx);

    let mut dialog: Vec<UiItemRef> = Vec::new();
    ui_add_background(ctx, &mut dialog);
    ui_add_logo(ctx, &mut dialog);

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);

    {
        let sb = &ctx.diablo_ui.scrollbar;
        let bg = sb.art_scroll_bar_background.as_ref().expect("ArtScrollBarBackground").get(0);
        let thumb = sb.art_scroll_bar_thumb.as_ref().expect("ArtScrollBarThumb").get(0);
        let arrow = sb.art_scroll_bar_arrow.clone().expect("ArtScrollBarArrow");
        dialog.push(UiItem::scrollbar(bg, thumb, arrow, Rect::new(ui_position.x + 590, ui_position.y + 244, 25, 178), UiFlags::NONE));
    }

    let conn_name = if provider == SELCONN_ZT { CONNECTION_NAMES[provider as usize].to_string() } else { tr(CONNECTION_NAMES[provider as usize]) };
    dialog.push(UiItem::art_text(
        &conn_name,
        Rect::new(ui_position.x + 24, ui_position.y + 161, 590, 35),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));
    dialog.push(UiItem::art_text(&tr("Description:"), Rect::new(ui_position.x + 35, ui_position.y + 211, 205, 192), UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_SILVER, 1, -1));
    let desc = ctx.diablo_ui.selgame.selgame_description.clone();
    dialog.push(UiItem::art_text_dynamic(
        desc,
        Rect::new(ui_position.x + 35, ui_position.y + 256, DESCRIPTION_WIDTH, 192),
        UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK,
        1,
        16,
    ));
    dialog.push(UiItem::art_text(
        &tr("Select Action"),
        Rect::new(ui_position.x + 300, ui_position.y + 211, 295, 33),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));

    let mut items: Vec<UiListItemRef> = Vec::new();
    // PACKET_ENCRYPTION is defined in the Windows build (libsodium is linked)
    items.push(UiListItem::new(&tr("Create Game"), 0, UiFlags::COLOR_UI_GOLD));
    items.push(UiListItem::new(&tr("Create Public Game"), 1, UiFlags::COLOR_UI_GOLD));
    items.push(UiListItem::new(&tr("Join Game"), 2, UiFlags::COLOR_UI_GOLD));

    if provider == SELCONN_ZT {
        items.push(UiListItem::new("", -1, UiFlags::ELEMENT_DISABLED));
        items.push(UiListItem::new(&tr("Public Games"), -1, UiFlags::ELEMENT_DISABLED | UiFlags::COLOR_WHITEGOLD));
        if ctx.diablo_ui.selgame.gamelist.is_empty() {
            let first = ctx.diablo_ui.selgame.first_public_game_info_request_send;
            if first == 0 || ctx.platform.ticks().wrapping_sub(first) < 2000 {
                items.push(UiListItem::new(&tr("Loading..."), -1, UiFlags::ELEMENT_DISABLED | UiFlags::COLOR_UI_SILVER));
            } else {
                items.push(UiListItem::new(&tr("None"), -1, UiFlags::ELEMENT_DISABLED | UiFlags::COLOR_UI_SILVER));
            }
        } else {
            for (i, g) in ctx.diablo_ui.selgame.gamelist.iter().enumerate() {
                items.push(UiListItem::new(&g.name, i as i32 + 3, UiFlags::COLOR_UI_GOLD));
            }
        }
    }

    dialog.push(UiItem::list(&items, 6, ui_position.x + 305, ui_position.y + 255, 285, 26, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_24, 1));

    dialog.push(UiItem::art_text_button(
        &tr("OK"),
        ui_focus_navigation_select,
        Rect::new(ui_position.x + 299, ui_position.y + 427, 140, 35),
        UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
    ));
    dialog.push(UiItem::art_text_button(
        &tr("CANCEL"),
        ui_focus_navigation_esc,
        Rect::new(ui_position.x + 449, ui_position.y + 427, 140, 35),
        UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
    ));

    if !search.is_empty() {
        for (i, item) in items.iter().enumerate() {
            let game_index = item.borrow().m_value - 3;
            if game_index < 0 {
                continue;
            }
            if search == ctx.diablo_ui.selgame.gamelist[game_index as usize].name {
                ctx.diablo_ui.selgame.highlighted_item = i;
            }
        }
    }
    if ctx.diablo_ui.selgame.highlighted_item >= items.len() {
        ctx.diablo_ui.selgame.highlighted_item = items.len() - 1;
    }

    ctx.diablo_ui.selgame.vec_sel_game_dlg_items = items;
    ctx.diablo_ui.selgame.vec_sel_game_dialog = dialog.clone();
    let highlighted = ctx.diablo_ui.selgame.highlighted_item;
    ui_init_list(ctx, Some(selgame_game_selection_focus), Some(select_fn), Some(selgame_game_selection_esc), &dialog, true, None, None, highlighted);
}

/// The `selectFn` lambda of `UiInitGameSelectionList`.
fn select_fn(ctx: &mut Ctx, index: i32) {
    let item_value = ctx.diablo_ui.selgame.vec_sel_game_dlg_items[index as usize].borrow().m_value;
    selgame_game_selection_select(ctx, item_value);
}

/// Original: `devilution::selgame_GameSelection_Init` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_GameSelection_Init() sha=f1ffb0bdda81
pub fn selgame_game_selection_init(ctx: &mut Ctx) {
    ui_init_game_selection_list(ctx, "");
}

/// Original: `devilution::selgame_GameSelection_Focus` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_GameSelection_Focus(int value) sha=2fac1317baf0
pub fn selgame_game_selection_focus(ctx: &mut Ctx, value: i32) {
    let index = value as usize;
    ctx.diablo_ui.selgame.highlighted_item = index;
    let m_value = ctx.diablo_ui.selgame.vec_sel_game_dlg_items[index].borrow().m_value;
    let desc = ctx.diablo_ui.selgame.selgame_description.clone();
    match m_value {
        0 => set_text(&desc, &tr("Create a new game with a difficulty setting of your choice."), 512),
        1 => set_text(&desc, &tr("Create a new public game that anyone can join with a difficulty setting of your choice."), 512),
        2 => {
            if ctx.diablo_ui.selgame.provider == SELCONN_ZT {
                set_text(&desc, &tr("Enter Game ID to join a game already in progress."), 512);
            } else {
                set_text(&desc, &tr("Enter an IP or a hostname to join a game already in progress."), 512);
            }
        }
        _ => {
            let game_info = ctx.diablo_ui.selgame.gamelist[(m_value - 3) as usize].clone();
            let mut info_string = tr("Join the public game already in progress.");
            info_string.push_str("\n\n");
            if is_game_compatible(ctx, &game_info.gameData) {
                let difficulty = match game_info.gameData.nDifficulty {
                    DIFF_NORMAL => tr("Normal"),
                    DIFF_NIGHTMARE => tr("Nightmare"),
                    DIFF_HELL => tr("Hell"),
                    _ => String::new(),
                };
                info_string.push_str(&tr("Difficulty: {:s}").replacen("{:s}", &difficulty, 1));
                info_string.push('\n');
                match game_info.gameData.nTickRate {
                    20 => info_string.push_str(&tr("Speed: Normal")),
                    30 => info_string.push_str(&tr("Speed: Fast")),
                    40 => info_string.push_str(&tr("Speed: Faster")),
                    50 => info_string.push_str(&tr("Speed: Fastest")),
                    n => info_string.push_str(&format!("Speed: {n}")),
                }
                info_string.push('\n');
                info_string.push_str(&tr("Players: "));
                for player_name in game_info.players.iter() {
                    info_string.push_str(player_name);
                    info_string.push(' ');
                }
            } else {
                info_string.push_str(&get_error_message_incompatibility(ctx, &game_info.gameData));
            }
            set_text(&desc, &info_string, 512);
        }
    }
    let current = desc.borrow().clone();
    let wrapped = word_wrap_string(ctx, &current, DESCRIPTION_WIDTH as u32, GameFontTables::GameFont12, 1);
    set_text(&desc, &wrapped, 512);
}

/// Original: `UpdateHeroLevel` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::UpdateHeroLevel(_uiheroinfo *pInfo) sha=29f16d1cc60c
fn update_hero_level(ctx: &mut Ctx, info: &crate::pfile::UiHeroInfo) -> bool {
    if info.saveNumber == ctx.menu.g_save_number {
        ctx.diablo_ui.selgame.hero_level = info.level as i32;
    }
    true
}

/// Original: `devilution::selgame_GameSelection_Select` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_GameSelection_Select(int value) sha=1d25a6d94e43
pub fn selgame_game_selection_select(ctx: &mut Ctx, value: i32) {
    ctx.diablo_ui.selgame.selgame_entering_game = true;
    ctx.diablo_ui.selgame.selgame_selected_game = value;

    let info_fn = ctx.diablo_ui.selhero.gfn_hero_info.expect("gfnHeroInfo");
    info_fn(ctx, &mut update_hero_level);

    selgame_free_vectors(ctx);

    if value > 2 {
        let name = ctx.diablo_ui.selgame.gamelist[(value - 3) as usize].name.clone();
        set_text(&ctx.diablo_ui.selgame.selgame_ip, &name, 129);
        selgame_password_select(ctx, value);
        return;
    }

    let mut dialog: Vec<UiItemRef> = Vec::new();
    ui_add_background(ctx, &mut dialog);
    ui_add_logo(ctx, &mut dialog);

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let title = ctx.diablo_ui.selgame.title.clone();
    dialog.push(UiItem::art_text_dynamic(
        title.clone(),
        Rect::new(ui_position.x + 24, ui_position.y + 161, 590, 35),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));
    dialog.push(UiItem::art_text_dynamic(
        ctx.diablo_ui.selgame.selgame_label.clone(),
        Rect::new(ui_position.x + 34, ui_position.y + 211, 205, 33),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));
    dialog.push(UiItem::art_text_dynamic(
        ctx.diablo_ui.selgame.selgame_description.clone(),
        Rect::new(ui_position.x + 35, ui_position.y + 256, DESCRIPTION_WIDTH, 192),
        UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK,
        1,
        16,
    ));

    match value {
        0 | 1 => {
            *title.borrow_mut() = tr("Create Game");
            dialog.push(UiItem::art_text(
                &tr("Select Difficulty"),
                Rect::new(ui_position.x + 299, ui_position.y + 211, 295, 35),
                UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
                3,
                -1,
            ));
            let items = vec![
                UiListItem::new(&tr("Normal"), DIFF_NORMAL as i32, UiFlags::NONE),
                UiListItem::new(&tr("Nightmare"), DIFF_NIGHTMARE as i32, UiFlags::NONE),
                UiListItem::new(&tr("Hell"), DIFF_HELL as i32, UiFlags::NONE),
            ];
            dialog.push(UiItem::list(
                &items,
                items.len(),
                ui_position.x + 300,
                ui_position.y + 282,
                295,
                26,
                UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD,
                1,
            ));
            dialog.push(UiItem::art_text_button(
                &tr("OK"),
                ui_focus_navigation_select,
                Rect::new(ui_position.x + 299, ui_position.y + 427, 140, 35),
                UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
            ));
            dialog.push(UiItem::art_text_button(
                &tr("CANCEL"),
                ui_focus_navigation_esc,
                Rect::new(ui_position.x + 449, ui_position.y + 427, 140, 35),
                UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
            ));
            ctx.diablo_ui.selgame.vec_sel_game_dlg_items = items;
            ctx.diablo_ui.selgame.vec_sel_game_dialog = dialog.clone();
            ui_init_list(ctx, Some(selgame_diff_focus), Some(selgame_diff_select), Some(selgame_diff_esc), &dialog, true, None, None, 0);
        }
        2 => {
            let provider = ctx.diablo_ui.selgame.provider;
            let conn_name = if provider == SELCONN_ZT { CONNECTION_NAMES[provider as usize].to_string() } else { tr(CONNECTION_NAMES[provider as usize]) };
            ctx.diablo_ui.selgame.selgame_title = tr("Join {:s} Games").replacen("{:s}", &conn_name, 1);
            *title.borrow_mut() = ctx.diablo_ui.selgame.selgame_title.clone();

            let input_hint = if provider == SELCONN_ZT { tr("Enter Game ID") } else { tr("Enter address") };
            dialog.push(UiItem::art_text(
                &input_hint,
                Rect::new(ui_position.x + 305, ui_position.y + 211, 285, 33),
                UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
                3,
                -1,
            ));
            dialog.push(UiItem::edit(
                &input_hint,
                ctx.diablo_ui.selgame.selgame_ip.clone(),
                128,
                false,
                Rect::new(ui_position.x + 305, ui_position.y + 314, 285, 33),
                UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD,
            ));
            dialog.push(UiItem::art_text_button(
                &tr("OK"),
                ui_focus_navigation_select,
                Rect::new(ui_position.x + 299, ui_position.y + 427, 140, 35),
                UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
            ));
            dialog.push(UiItem::art_text_button(
                &tr("CANCEL"),
                ui_focus_navigation_esc,
                Rect::new(ui_position.x + 449, ui_position.y + 427, 140, 35),
                UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
            ));
            ctx.diablo_ui.selgame.highlighted_item = 0;
            ctx.diablo_ui.selgame.vec_sel_game_dialog = dialog.clone();
            ui_init_list(ctx, None, Some(selgame_password_init), Some(selgame_game_selection_init), &dialog, false, None, None, 0);
        }
        _ => {}
    }
}

/// Original: `devilution::selgame_GameSelection_Esc` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_GameSelection_Esc() sha=42c11a67f0fe
pub fn selgame_game_selection_esc(ctx: &mut Ctx) {
    ui_init_list_clear(ctx);
    ctx.diablo_ui.selgame.selgame_entering_game = false;
    ctx.diablo_ui.selgame.selgame_end_menu = true;
}

/// Original: `devilution::selgame_Diff_Focus` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Diff_Focus(int value) sha=c91710c467a9
pub fn selgame_diff_focus(ctx: &mut Ctx, value: i32) {
    let label = ctx.diablo_ui.selgame.selgame_label.clone();
    let desc = ctx.diablo_ui.selgame.selgame_description.clone();
    match ctx.diablo_ui.selgame.vec_sel_game_dlg_items[value as usize].borrow().m_value as _difficulty {
        DIFF_NORMAL => {
            set_text(&label, &tr("Normal"), 32);
            set_text(&desc, &tr("Normal Difficulty\nThis is where a starting character should begin the quest to defeat Diablo."), 512);
        }
        DIFF_NIGHTMARE => {
            set_text(&label, &tr("Nightmare"), 32);
            set_text(
                &desc,
                &tr("Nightmare Difficulty\nThe denizens of the Labyrinth have been bolstered and will prove to be a greater challenge. This is recommended for experienced characters only."),
                512,
            );
        }
        DIFF_HELL => {
            set_text(&label, &tr("Hell"), 32);
            set_text(
                &desc,
                &tr("Hell Difficulty\nThe most powerful of the underworld's creatures lurk at the gateway into Hell. Only the most experienced characters should venture in this realm."),
                512,
            );
        }
        _ => {}
    }
    let current = desc.borrow().clone();
    let wrapped = word_wrap_string(ctx, &current, DESCRIPTION_WIDTH as u32, GameFontTables::GameFont12, 1);
    set_text(&desc, &wrapped, 512);
}

/// Original: `IsDifficultyAllowed` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::IsDifficultyAllowed(int value) sha=3048bc40cff7
fn is_difficulty_allowed(ctx: &mut Ctx, value: i32) -> bool {
    let hero_level = ctx.diablo_ui.selgame.hero_level;
    if value == 0 || (value == 1 && hero_level >= 20) || (value == 2 && hero_level >= 30) {
        return true;
    }
    selgame_free(ctx);
    let title = ctx.diablo_ui.selgame.title.borrow().clone();
    if value == 1 {
        crate::diablo_ui::selok::ui_sel_ok_dialog(ctx, Some(&title), &tr("Your character must reach level 20 before you can enter a multiplayer game of Nightmare difficulty."), false);
    }
    if value == 2 {
        crate::diablo_ui::selok::ui_sel_ok_dialog(ctx, Some(&title), &tr("Your character must reach level 30 before you can enter a multiplayer game of Hell difficulty."), false);
    }
    selgame_init(ctx);
    false
}

/// Original: `devilution::selgame_Diff_Select` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Diff_Select(int value) sha=30c08057979f
pub fn selgame_diff_select(ctx: &mut Ctx, value: i32) {
    let v = ctx.diablo_ui.selgame.vec_sel_game_dlg_items[value as usize].borrow().m_value;
    if ctx.diablo_ui.selhero.selhero_is_multi_player && !is_difficulty_allowed(ctx, v) {
        selgame_game_selection_select(ctx, 0);
        return;
    }
    ctx.diablo_ui.selgame.n_difficulty = v as _difficulty;

    if !ctx.diablo_ui.selhero.selhero_is_multi_player {
        // We're in the selhero loop instead of the selgame one.
        ctx.diablo_ui.selhero.selhero_end_menu = true;
        selgame_free_vectors(ctx);
        ui_init_list_clear(ctx);
        return;
    }
    selgame_game_speed_selection(ctx);
}

/// Original: `devilution::selgame_Diff_Esc` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Diff_Esc() sha=8271461d2041
pub fn selgame_diff_esc(ctx: &mut Ctx) {
    if !ctx.diablo_ui.selhero.selhero_is_multi_player {
        selgame_free(ctx);
        crate::diablo_ui::selhero::selhero_init(ctx);
        crate::diablo_ui::selhero::selhero_list_init(ctx);
        return;
    }
    if ctx.diablo_ui.selgame.provider == SELCONN_LOOPBACK {
        selgame_game_selection_esc(ctx);
        return;
    }
    ctx.diablo_ui.selgame.highlighted_item = 0;
    selgame_game_selection_init(ctx);
}

/// Original: `devilution::selgame_GameSpeedSelection` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_GameSpeedSelection() sha=7086604fd780
pub fn selgame_game_speed_selection(ctx: &mut Ctx) {
    let info_fn = ctx.diablo_ui.selhero.gfn_hero_info.expect("gfnHeroInfo");
    info_fn(ctx, &mut update_hero_level);
    selgame_free_vectors(ctx);

    let mut dialog: Vec<UiItemRef> = Vec::new();
    ui_add_background(ctx, &mut dialog);
    ui_add_logo(ctx, &mut dialog);
    let ui_position = crate::utils::display::get_ui_rectangle(ctx);

    dialog.push(UiItem::art_text(
        &tr("Create Game"),
        Rect::new(ui_position.x + 24, ui_position.y + 161, 590, 35),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));
    dialog.push(UiItem::art_text_dynamic(
        ctx.diablo_ui.selgame.selgame_label.clone(),
        Rect::new(ui_position.x + 34, ui_position.y + 211, 205, 33),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));
    dialog.push(UiItem::art_text_dynamic(
        ctx.diablo_ui.selgame.selgame_description.clone(),
        Rect::new(ui_position.x + 35, ui_position.y + 256, DESCRIPTION_WIDTH, 192),
        UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK,
        1,
        16,
    ));
    dialog.push(UiItem::art_text(
        &tr("Select Game Speed"),
        Rect::new(ui_position.x + 299, ui_position.y + 211, 295, 35),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));
    let items = vec![
        UiListItem::new(&tr("Normal"), 20, UiFlags::NONE),
        UiListItem::new(&tr("Fast"), 30, UiFlags::NONE),
        UiListItem::new(&tr("Faster"), 40, UiFlags::NONE),
        UiListItem::new(&tr("Fastest"), 50, UiFlags::NONE),
    ];
    dialog.push(UiItem::list(&items, items.len(), ui_position.x + 300, ui_position.y + 279, 295, 26, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD, 1));
    dialog.push(UiItem::art_text_button(
        &tr("OK"),
        ui_focus_navigation_select,
        Rect::new(ui_position.x + 299, ui_position.y + 427, 140, 35),
        UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
    ));
    dialog.push(UiItem::art_text_button(
        &tr("CANCEL"),
        ui_focus_navigation_esc,
        Rect::new(ui_position.x + 449, ui_position.y + 427, 140, 35),
        UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
    ));
    ctx.diablo_ui.selgame.vec_sel_game_dlg_items = items;
    ctx.diablo_ui.selgame.vec_sel_game_dialog = dialog.clone();
    ui_init_list(ctx, Some(selgame_speed_focus), Some(selgame_speed_select), Some(selgame_speed_esc), &dialog, true, None, None, 0);
}

/// Original: `devilution::selgame_Speed_Focus` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Speed_Focus(int value) sha=6e5adbebc5ae
pub fn selgame_speed_focus(ctx: &mut Ctx, value: i32) {
    let label = ctx.diablo_ui.selgame.selgame_label.clone();
    let desc = ctx.diablo_ui.selgame.selgame_description.clone();
    match ctx.diablo_ui.selgame.vec_sel_game_dlg_items[value as usize].borrow().m_value {
        20 => {
            set_text(&label, &tr("Normal"), 32);
            set_text(&desc, &tr("Normal Speed\nThis is where a starting character should begin the quest to defeat Diablo."), 512);
        }
        30 => {
            set_text(&label, &tr("Fast"), 32);
            set_text(
                &desc,
                &tr("Fast Speed\nThe denizens of the Labyrinth have been hastened and will prove to be a greater challenge. This is recommended for experienced characters only."),
                512,
            );
        }
        40 => {
            set_text(&label, &tr("Faster"), 32);
            set_text(
                &desc,
                &tr("Faster Speed\nMost monsters of the dungeon will seek you out quicker than ever before. Only an experienced champion should try their luck at this speed."),
                512,
            );
        }
        50 => {
            set_text(&label, &tr("Fastest"), 32);
            set_text(&desc, &tr("Fastest Speed\nThe minions of the underworld will rush to attack without hesitation. Only a true speed demon should enter at this pace."), 512);
        }
        _ => {}
    }
    let current = desc.borrow().clone();
    let wrapped = word_wrap_string(ctx, &current, DESCRIPTION_WIDTH as u32, GameFontTables::GameFont12, 1);
    set_text(&desc, &wrapped, 512);
}

/// Original: `devilution::selgame_Speed_Esc` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Speed_Esc() sha=abf7b852e9ef
pub fn selgame_speed_esc(ctx: &mut Ctx) {
    selgame_game_selection_select(ctx, 0);
}

/// Original: `devilution::selgame_Speed_Select` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Speed_Select(int value) sha=f2f325c9ad00
pub fn selgame_speed_select(ctx: &mut Ctx, value: i32) {
    ctx.diablo_ui.selgame.n_tick_rate = ctx.diablo_ui.selgame.vec_sel_game_dlg_items[value as usize].borrow().m_value;
    if ctx.diablo_ui.selgame.provider == SELCONN_LOOPBACK || ctx.diablo_ui.selgame.selgame_selected_game == 1 {
        selgame_password_select(ctx, 0);
        return;
    }
    selgame_password_init(ctx, 0);
}

/// Original: `devilution::selgame_Password_Init` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Password_Init(int) sha=d35995d9d7fb
pub fn selgame_password_init(ctx: &mut Ctx, _value: i32) {
    ctx.diablo_ui.selgame.selgame_password.borrow_mut().clear();
    selgame_free_vectors(ctx);

    let mut dialog: Vec<UiItemRef> = Vec::new();
    ui_add_background(ctx, &mut dialog);
    ui_add_logo(ctx, &mut dialog);
    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let provider = ctx.diablo_ui.selgame.provider;
    let conn_name = if provider == SELCONN_ZT { CONNECTION_NAMES[provider as usize].to_string() } else { tr(CONNECTION_NAMES[provider as usize]) };

    dialog.push(UiItem::art_text(
        &conn_name,
        Rect::new(ui_position.x + 24, ui_position.y + 161, 590, 35),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));
    dialog.push(UiItem::art_text(&tr("Description:"), Rect::new(ui_position.x + 35, ui_position.y + 211, 205, 192), UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_SILVER, 1, -1));
    dialog.push(UiItem::art_text_dynamic(
        ctx.diablo_ui.selgame.selgame_description.clone(),
        Rect::new(ui_position.x + 35, ui_position.y + 256, DESCRIPTION_WIDTH, 192),
        UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK,
        1,
        16,
    ));
    dialog.push(UiItem::art_text(
        &tr("Enter Password"),
        Rect::new(ui_position.x + 305, ui_position.y + 211, 285, 33),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));
    let allow_empty = ctx.diablo_ui.selgame.selgame_selected_game == 2;
    dialog.push(UiItem::edit(
        &tr("Enter Password"),
        ctx.diablo_ui.selgame.selgame_password.clone(),
        15,
        allow_empty,
        Rect::new(ui_position.x + 305, ui_position.y + 314, 285, 33),
        UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD,
    ));
    dialog.push(UiItem::art_text_button(
        &tr("OK"),
        ui_focus_navigation_select,
        Rect::new(ui_position.x + 299, ui_position.y + 427, 140, 35),
        UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
    ));
    dialog.push(UiItem::art_text_button(
        &tr("CANCEL"),
        ui_focus_navigation_esc,
        Rect::new(ui_position.x + 449, ui_position.y + 427, 140, 35),
        UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
    ));
    ctx.diablo_ui.selgame.vec_sel_game_dialog = dialog.clone();
    ui_init_list(ctx, None, Some(selgame_password_select), Some(selgame_password_esc), &dialog, false, None, None, 0);
}

/// Original: `IsGameCompatibleWithErrorMessage` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::IsGameCompatibleWithErrorMessage(const GameData &data) sha=56c21d545463
fn is_game_compatible_with_error_message(ctx: &mut Ctx, data: &GameData) -> bool {
    if is_game_compatible(ctx, data) {
        return is_difficulty_allowed(ctx, data.nDifficulty as i32);
    }
    selgame_free(ctx);
    let error_message = get_error_message_incompatibility(ctx, data);
    let title = ctx.diablo_ui.selgame.title.borrow().clone();
    crate::diablo_ui::selok::ui_sel_ok_dialog(ctx, Some(&title), &error_message, false);
    selgame_init(ctx);
    false
}

/// Original: `devilution::selgame_Password_Select` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Password_Select(int) sha=a1055c91a5ae
pub fn selgame_password_select(ctx: &mut Ctx, _value: i32) {
    let selected = ctx.diablo_ui.selgame.selgame_selected_game;
    let password = ctx.diablo_ui.selgame.selgame_password.borrow().clone();
    let mut game_password: Option<String> = None;
    if selected == 0 {
        game_password = Some(password.clone());
    }
    if selected == 2 && !password.is_empty() {
        game_password = Some(password.clone());
    }
    crate::storm::storm_net::clear_error(ctx);

    if selected > 1 {
        let mut allow_join = true;
        if selected > 2 {
            let data = ctx.diablo_ui.selgame.gamelist[(selected - 3) as usize].gameData;
            allow_join = is_game_compatible(ctx, &data);
        }
        if ctx.diablo_ui.selgame.provider == SELCONN_ZT {
            let lower = ctx.diablo_ui.selgame.selgame_ip.borrow().to_ascii_lowercase();
            *ctx.diablo_ui.selgame.selgame_ip.borrow_mut() = lower.clone();
            ctx.options.network.sz_previous_zt_game = lower;
        } else {
            ctx.options.network.sz_previous_host = ctx.diablo_ui.selgame.selgame_ip.borrow().clone();
        }
        let ip = ctx.diablo_ui.selgame.selgame_ip.borrow().clone();
        let mut player_id = ctx.diablo_ui.selgame.gdw_player_id;
        if allow_join && crate::storm::storm_net::snet_join_game(ctx, &ip, game_password.as_deref(), &mut player_id) {
            ctx.diablo_ui.selgame.gdw_player_id = player_id;
            let data = ctx.multi.sgGameInitInfo;
            if !is_game_compatible_with_error_message(ctx, &data) {
                crate::multi::init_game_info(ctx);
                selgame_game_selection_select(ctx, 1);
                return;
            }
            ui_init_list_clear(ctx);
            ctx.diablo_ui.selgame.selgame_end_menu = true;
        } else {
            crate::multi::init_game_info(ctx);
            selgame_free(ctx);
            let mut error = if !allow_join {
                let data = ctx.diablo_ui.selgame.gamelist[(selected - 3) as usize].gameData;
                get_error_message_incompatibility(ctx, &data)
            } else {
                crate::storm::storm_net::get_error(ctx)
            };
            if error.is_empty() {
                error = "Unknown network error".to_string();
            }
            crate::diablo_ui::selok::ui_sel_ok_dialog(ctx, Some(&tr("Multi Player Game")), &error, false);
            selgame_init(ctx);
            if selected == 2 {
                selgame_password_init(ctx, selected);
            } else {
                ui_init_game_selection_list(ctx, "");
            }
        }
        return;
    }

    {
        let g = &mut ctx.multi.sgGameInitInfo;
        g.nDifficulty = ctx.diablo_ui.selgame.n_difficulty;
        g.nTickRate = ctx.diablo_ui.selgame.n_tick_rate as u8;
        g.bRunInTown = ctx.options.gameplay.run_in_town.get() as u8;
        g.bTheoQuest = ctx.options.gameplay.theo_quest.get() as u8;
        g.bCowQuest = ctx.options.gameplay.cow_quest.get() as u8;
    }
    let data = ctx.multi.sgGameInitInfo;
    let mut player_id = ctx.diablo_ui.selgame.gdw_player_id;
    if crate::storm::storm_net::snet_create_game(ctx, None, game_password.as_deref(), &data, &mut player_id) {
        ctx.diablo_ui.selgame.gdw_player_id = player_id;
        ui_init_list_clear(ctx);
        ctx.diablo_ui.selgame.selgame_end_menu = true;
    } else {
        selgame_free(ctx);
        let mut error = crate::storm::storm_net::get_error(ctx);
        if error.is_empty() {
            error = "Unknown network error".to_string();
        }
        crate::diablo_ui::selok::ui_sel_ok_dialog(ctx, Some(&tr("Multi Player Game")), &error, false);
        selgame_init(ctx);
        selgame_password_init(ctx, 0);
    }
}

/// Original: `devilution::selgame_Password_Esc` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::selgame_Password_Esc() sha=8d76a3b29900
pub fn selgame_password_esc(ctx: &mut Ctx) {
    if ctx.diablo_ui.selgame.selgame_selected_game == 2 {
        selgame_game_selection_select(ctx, 2);
    } else {
        selgame_game_speed_selection(ctx);
    }
}

/// Original: `RefreshGameList` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::RefreshGameList() sha=eb90e9908696
fn refresh_game_list(ctx: &mut Ctx) {
    if ctx.diablo_ui.selgame.selgame_entering_game {
        return;
    }
    let current_time = ctx.platform.ticks();
    let last_request = ctx.diablo_ui.selgame.last_request;
    if (last_request == 0 || current_time.wrapping_sub(last_request) > 30000) && crate::storm::storm_net::dvlnet_send_info_request(ctx) {
        ctx.diablo_ui.selgame.last_request = current_time;
        ctx.diablo_ui.selgame.last_update = current_time.wrapping_sub(3000);
        if ctx.diablo_ui.selgame.first_public_game_info_request_send == 0 {
            ctx.diablo_ui.selgame.first_public_game_info_request_send = current_time;
        }
    }
    let last_update = ctx.diablo_ui.selgame.last_update;
    if last_update == 0 || current_time.wrapping_sub(last_update) > 5000 {
        let highlighted = ctx.diablo_ui.selgame.highlighted_item;
        let game_index = ctx.diablo_ui.selgame.vec_sel_game_dlg_items[highlighted].borrow().m_value - 3;
        let game_search = if game_index >= 0 { ctx.diablo_ui.selgame.gamelist[game_index as usize].name.clone() } else { String::new() };
        let gamelist = crate::storm::storm_net::dvlnet_get_gamelist(ctx);
        ctx.diablo_ui.selgame.gamelist = gamelist;
        ui_init_game_selection_list(ctx, &game_search);
        ctx.diablo_ui.selgame.last_update = current_time;
    }
}

/// Original: `devilution::UiSelectGame` (DiabloUI/multi/selgame.cpp).
// @port DiabloUI/multi/selgame.cpp|devilution::UiSelectGame(GameData *gameData, int *playerId) sha=7fbc2f943bdc
pub fn ui_select_game(ctx: &mut Ctx, player_id: &mut i32) -> bool {
    ctx.diablo_ui.selgame.first_public_game_info_request_send = 0;
    ctx.diablo_ui.selgame.gdw_player_id = *player_id;
    selgame_init(ctx);
    ctx.diablo_ui.selgame.highlighted_item = 0;
    selgame_game_selection_init(ctx);

    ctx.diablo_ui.selgame.selgame_end_menu = false;

    crate::storm::storm_net::dvlnet_clear_password(ctx);
    crate::storm::storm_net::dvlnet_clear_gamelist(ctx);

    while !ctx.diablo_ui.selgame.selgame_end_menu {
        ui_clear_screen(ctx);
        ui_poll_and_render(ctx, None);
        if ctx.diablo_ui.selgame.provider == SELCONN_ZT {
            refresh_game_list(ctx);
        }
    }
    selgame_free(ctx);
    *player_id = ctx.diablo_ui.selgame.gdw_player_id;
    ctx.diablo_ui.selgame.selgame_entering_game
}
