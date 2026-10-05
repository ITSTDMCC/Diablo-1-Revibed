//! `Source/DiabloUI/hero/selhero.cpp`: hero selection and creation.

use std::cell::RefCell;
use std::rc::Rc;

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{
    load_background_art, ui_add_logo, ui_clear_screen, ui_focus_navigation_esc, ui_focus_navigation_select, ui_focus_navigation_yes_no, ui_get_hero_dialog_sprite,
    ui_init_list, ui_init_list_clear, ui_poll_and_render, ui_render_item, ui_render_items, ui_valid_player_name, NUM_HERO_CLASSES,
};
use crate::diablo_ui::scrollbar::{load_scroll_bar, unload_scroll_bar};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiKind, UiListItem, UiListItemRef};
use crate::engine::render::text_render::UiFlags;
use crate::engine::surface::Rect;
use crate::enums::*;
use crate::menu::SelheroSelections;
use crate::pfile::{UiDefaultStats, UiHeroInfo, MAX_CHARACTERS};
use crate::utils::language::tr;

/// `gfnHeroInfo`
pub type HeroInfoFn = fn(&mut Ctx, &mut dyn FnMut(&mut Ctx, &UiHeroInfo) -> bool) -> bool;
/// `gfnHeroCreate`
pub type HeroCreateFn = fn(&mut Ctx, &mut UiHeroInfo) -> bool;
/// `gfnHeroStats`
pub type HeroStatsFn = fn(u32, &mut UiDefaultStats);
/// `fnremove`
pub type HeroRemoveFn = fn(&mut Ctx, &UiHeroInfo) -> bool;

/// Globals of selhero.cpp.
pub struct SelHeroState {
    pub selhero_end_menu: bool,
    pub selhero_is_multi_player: bool,
    pub gfn_hero_info: Option<HeroInfoFn>,
    pub gfn_hero_create: Option<HeroCreateFn>,
    pub gfn_hero_stats: Option<HeroStatsFn>,
    selhero_save_count: usize,
    selhero_heros: Vec<UiHeroInfo>,
    selhero_hero_info: UiHeroInfo,
    text_stats: [Rc<RefCell<String>>; 6],
    title: Rc<RefCell<String>>,
    selhero_result: SelheroSelections,
    selhero_navigate_yes_no: bool,
    selhero_is_savegame: bool,
    pub vec_sel_hero_dialog: Vec<UiItemRef>,
    pub vec_sel_hero_dlg_items: Vec<UiListItemRef>,
    vec_sel_dlg_items: Vec<UiItemRef>,
    /// `SELHERO_DIALOG_HERO_IMG`
    selhero_dialog_hero_img: Option<UiItemRef>,
    /// `SELLIST_DIALOG_DELETE_BUTTON`
    sellist_dialog_delete_button: Option<UiItemRef>,
    /// The text buffer of the name `UiEdit` (the original edits `selhero_heroInfo.name` in place).
    name_edit: Rc<RefCell<String>>,
}

impl Default for SelHeroState {
    fn default() -> Self {
        SelHeroState {
            selhero_end_menu: false,
            selhero_is_multi_player: false,
            gfn_hero_info: None,
            gfn_hero_create: None,
            gfn_hero_stats: None,
            selhero_save_count: 0,
            selhero_heros: vec![UiHeroInfo::default(); MAX_CHARACTERS],
            selhero_hero_info: UiHeroInfo::default(),
            text_stats: Default::default(),
            title: Default::default(),
            selhero_result: SelheroSelections::NewDungeon,
            selhero_navigate_yes_no: false,
            selhero_is_savegame: false,
            vec_sel_hero_dialog: Vec::new(),
            vec_sel_hero_dlg_items: Vec::new(),
            vec_sel_dlg_items: Vec::new(),
            selhero_dialog_hero_img: None,
            sellist_dialog_delete_button: None,
            name_edit: Default::default(),
        }
    }
}

fn set_title(ctx: &mut Ctx, t: &str) {
    *ctx.diablo_ui.selhero.title.borrow_mut() = t.to_string();
}

/// Original: `SelheroUiFocusNavigationYesNo` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroUiFocusNavigationYesNo() sha=d98a78e480bf
fn selhero_ui_focus_navigation_yes_no(ctx: &mut Ctx) {
    if ctx.diablo_ui.selhero.selhero_is_savegame {
        ui_focus_navigation_yes_no(ctx);
    }
}

/// Original: `SelheroFree` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroFree() sha=3b6a920d6ed3
fn selhero_free(ctx: &mut Ctx) {
    ctx.diablo_ui.art_background = None;
    ctx.diablo_ui.selhero.vec_sel_hero_dialog.clear();
    ctx.diablo_ui.selhero.vec_sel_dlg_items.clear();
    ctx.diablo_ui.selhero.vec_sel_hero_dlg_items.clear();
    unload_scroll_bar(ctx);
}

fn set_hero_img_sprite(ctx: &mut Ctx, index: usize) {
    let sprite = ui_get_hero_dialog_sprite(ctx, index);
    if let Some(img) = &ctx.diablo_ui.selhero.selhero_dialog_hero_img {
        if let UiKind::ImageClx { sprite: s } = &mut img.borrow_mut().kind {
            *s = sprite;
        }
    }
}

/// Original: `SelheroSetStats` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroSetStats() sha=4fe823306592
fn selhero_set_stats(ctx: &mut Ctx) {
    let info = ctx.diablo_ui.selhero.selhero_hero_info;
    set_hero_img_sprite(ctx, info.heroclass as usize);
    let values = [info.level as u32, info.strength as u32, info.magic as u32, info.dexterity as u32, info.vitality as u32, info.saveNumber];
    for (i, v) in values.iter().enumerate() {
        // CopyUtf8(textStats[i], StrCat(v), sizeof(textStats[i])) with char[4]
        *ctx.diablo_ui.selhero.text_stats[i].borrow_mut() = crate::utils::utf8::copy_utf8(&v.to_string(), 4);
    }
}

/// Original: `RenderDifficultyIndicators` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::RenderDifficultyIndicators() sha=1f1a6b5c1fd4
fn render_difficulty_indicators(ctx: &mut Ctx) {
    if !ctx.diablo_ui.selhero.selhero_is_savegame {
        return;
    }
    let Some(indicator) = ctx.diablo_ui.difficulty_indicator.clone() else { return };
    let sprite = indicator.get(0);
    let (width, height) = (sprite.width() as i32, sprite.height() as i32);
    let img_rect = ctx.diablo_ui.selhero.selhero_dialog_hero_img.as_ref().unwrap().borrow().m_rect;
    let mut rect = Rect::new(img_rect.x + 1, img_rect.y + img_rect.h - height - 1, width, height);
    let herorank = ctx.diablo_ui.selhero.selhero_hero_info.herorank as i32;
    for i in 0..=DIFF_LAST as i32 {
        if i >= herorank {
            break;
        }
        let item = UiItem::image_clx(sprite.clone(), rect, UiFlags::NONE);
        ui_render_item(ctx, &item);
        rect.x += width;
    }
}

/// Original: `SelHeroGetHeroInfo` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelHeroGetHeroInfo(_uiheroinfo *pInfo) sha=5949cd11a968
fn sel_hero_get_hero_info(ctx: &mut Ctx, info: &UiHeroInfo) -> bool {
    let s = &mut ctx.diablo_ui.selhero;
    s.selhero_heros[s.selhero_save_count] = *info;
    s.selhero_save_count += 1;
    true
}

/// Original: `SelheroListFocus` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroListFocus(int value) sha=7ade3cdbce51
fn selhero_list_focus(ctx: &mut Ctx, value: i32) {
    let index = value as usize;
    let base_flags = UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30;
    let save_count = ctx.diablo_ui.selhero.selhero_save_count;
    if save_count != 0 && index < save_count {
        ctx.diablo_ui.selhero.selhero_hero_info = ctx.diablo_ui.selhero.selhero_heros[index];
        selhero_set_stats(ctx);
        if let Some(b) = &ctx.diablo_ui.selhero.sellist_dialog_delete_button {
            b.borrow_mut().set_flags(base_flags | UiFlags::COLOR_UI_GOLD);
        }
        ctx.diablo_ui.selhero.selhero_is_savegame = true;
        return;
    }
    set_hero_img_sprite(ctx, NUM_HERO_CLASSES);
    for t in ctx.diablo_ui.selhero.text_stats.iter() {
        *t.borrow_mut() = "--".to_string();
    }
    if let Some(b) = &ctx.diablo_ui.selhero.sellist_dialog_delete_button {
        b.borrow_mut().set_flags(base_flags | UiFlags::COLOR_UI_SILVER | UiFlags::ELEMENT_DISABLED);
    }
    ctx.diablo_ui.selhero.selhero_is_savegame = false;
}

/// Original: `SelheroListDeleteYesNo` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroListDeleteYesNo() sha=ba3819b5b474
fn selhero_list_delete_yes_no(ctx: &mut Ctx) -> bool {
    ctx.diablo_ui.selhero.selhero_navigate_yes_no = ctx.diablo_ui.selhero.selhero_is_savegame;
    ctx.diablo_ui.selhero.selhero_navigate_yes_no
}

/// Original: `SelheroListSelect` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroListSelect(int value) sha=d73fb6aea912
fn selhero_list_select(ctx: &mut Ctx, value: i32) {
    let ui_position = crate::utils::display::get_ui_rectangle(ctx);

    if value as usize == ctx.diablo_ui.selhero.selhero_save_count {
        let mut items: Vec<UiItemRef> = Vec::new();

        let rect1 = Rect::new(ui_position.x + 242, ui_position.y + 211, 365, 33);
        items.push(UiItem::art_text(&tr("Choose Class"), rect1, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER, 3, -1));

        let mut dlg_items: Vec<UiListItemRef> = Vec::new();
        let mut item_h = 33;
        dlg_items.push(UiListItem::new(&tr("Warrior"), HeroClass::Warrior as i32, UiFlags::NONE));
        dlg_items.push(UiListItem::new(&tr("Rogue"), HeroClass::Rogue as i32, UiFlags::NONE));
        dlg_items.push(UiListItem::new(&tr("Sorcerer"), HeroClass::Sorcerer as i32, UiFlags::NONE));
        if ctx.init.gb_is_hellfire {
            dlg_items.push(UiListItem::new(&tr("Monk"), HeroClass::Monk as i32, UiFlags::NONE));
        }
        if ctx.diablo.gb_bard || ctx.options.gameplay.test_bard.get() {
            dlg_items.push(UiListItem::new(&tr("Bard"), HeroClass::Bard as i32, UiFlags::NONE));
        }
        if ctx.diablo.gb_barbarian || ctx.options.gameplay.test_barbarian.get() {
            dlg_items.push(UiListItem::new(&tr("Barbarian"), HeroClass::Barbarian as i32, UiFlags::NONE));
        }
        if dlg_items.len() > 4 {
            item_h = 26;
        }
        // `246 + (176 - size * itemH) / 2` with size_t arithmetic
        let item_y = 246usize.wrapping_add((176usize.wrapping_sub(dlg_items.len() * item_h as usize)) / 2) as i32;
        items.push(UiItem::list(
            &dlg_items,
            dlg_items.len(),
            ui_position.x + 264,
            ui_position.y + item_y,
            320,
            item_h,
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD,
            1,
        ));

        let rect2 = Rect::new(ui_position.x + 279, ui_position.y + 429, 140, 35);
        items.push(UiItem::art_text_button(&tr("OK"), ui_focus_navigation_select, rect2, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD));

        let rect3 = Rect::new(ui_position.x + 429, ui_position.y + 429, 144, 35);
        items.push(UiItem::art_text_button(&tr("Cancel"), ui_focus_navigation_esc, rect3, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD));

        ctx.diablo_ui.selhero.vec_sel_dlg_items = items.clone();
        ctx.diablo_ui.selhero.vec_sel_hero_dlg_items = dlg_items;
        ui_init_list(ctx, Some(selhero_class_selector_focus), Some(selhero_class_selector_select), Some(selhero_class_selector_esc), &items, true, None, None, 0);
        ctx.diablo_ui.selhero.selhero_hero_info.name = [0; 16];
        ctx.diablo_ui.selhero.selhero_hero_info.saveNumber = crate::pfile::pfile_ui_get_first_unused_save_num(ctx);
        selhero_set_stats(ctx);
        let t = if ctx.diablo_ui.selhero.selhero_is_multi_player { tr("New Multi Player Hero") } else { tr("New Single Player Hero") };
        set_title(ctx, &t);
        ctx.diablo_ui.selhero.selhero_is_savegame = false;
        return;
    }

    if ctx.diablo_ui.selhero.selhero_hero_info.hassaved {
        let mut items: Vec<UiItemRef> = Vec::new();

        let rect1 = Rect::new(ui_position.x + 242, ui_position.y + 211, 365, 33);
        items.push(UiItem::art_text(&tr("Save File Exists"), rect1, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER, 3, -1));

        let dlg_items = vec![UiListItem::new(&tr("Load Game"), 0, UiFlags::NONE), UiListItem::new(&tr("New Game"), 1, UiFlags::NONE)];
        items.push(UiItem::list(
            &dlg_items,
            dlg_items.len(),
            ui_position.x + 265,
            ui_position.y + 285,
            320,
            33,
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD,
            1,
        ));

        let rect2 = Rect::new(ui_position.x + 279, ui_position.y + 427, 140, 35);
        items.push(UiItem::art_text_button(
            &tr("OK"),
            ui_focus_navigation_select,
            rect2,
            UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
        ));

        let rect3 = Rect::new(ui_position.x + 429, ui_position.y + 427, 144, 35);
        items.push(UiItem::art_text_button(
            &tr("Cancel"),
            ui_focus_navigation_esc,
            rect3,
            UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD,
        ));

        ctx.diablo_ui.selhero.vec_sel_dlg_items = items.clone();
        ctx.diablo_ui.selhero.vec_sel_hero_dlg_items = dlg_items;
        ui_init_list(ctx, Some(selhero_load_focus), Some(selhero_load_select), Some(selhero_list_init), &items, true, None, None, 0);
        set_title(ctx, &tr("Single Player Characters"));
        return;
    }

    selhero_load_select(ctx, 1);
}

/// Original: `SelheroListEsc` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroListEsc() sha=8323a8cbabf6
fn selhero_list_esc(ctx: &mut Ctx) {
    ui_init_list_clear(ctx);
    ctx.diablo_ui.selhero.selhero_end_menu = true;
    ctx.diablo_ui.selhero.selhero_result = SelheroSelections::Previous;
}

/// Original: `SelheroClassSelectorFocus` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroClassSelectorFocus(int value) sha=900242c94ee1
fn selhero_class_selector_focus(ctx: &mut Ctx, value: i32) {
    let hero_class = HeroClass::from_raw(ctx.diablo_ui.selhero.vec_sel_hero_dlg_items[value as usize].borrow().m_value as u8);
    let mut defaults = UiDefaultStats::default();
    (ctx.diablo_ui.selhero.gfn_hero_stats.unwrap())(hero_class as u32, &mut defaults);
    let info = &mut ctx.diablo_ui.selhero.selhero_hero_info;
    info.level = 1;
    info.heroclass = hero_class;
    info.strength = defaults.strength;
    info.magic = defaults.magic;
    info.dexterity = defaults.dexterity;
    info.vitality = defaults.vitality;
    selhero_set_stats(ctx);
}

/// Original: `ShouldPrefillHeroName` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::ShouldPrefillHeroName() sha=4e9ae4905b7c
fn should_prefill_hero_name(ctx: &Ctx) -> bool {
    ctx.controls.control_mode != crate::controls::ControlTypes::KeyboardAndMouse
}

/// Original: `RemoveSelHeroBackground` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::RemoveSelHeroBackground() sha=f7d88563c592
fn remove_sel_hero_background(ctx: &mut Ctx) {
    ctx.diablo_ui.selhero.vec_sel_hero_dialog.remove(0);
    ctx.diablo_ui.art_background = None;
}

/// Original: `AddSelHeroBackground` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::AddSelHeroBackground() sha=682f2456a79a
fn add_sel_hero_background(ctx: &mut Ctx) {
    load_background_art(ctx, "ui_art\\selhero", 1);
    let y = crate::utils::display::get_ui_rectangle(ctx).y;
    let bg = ctx.diablo_ui.art_background.clone().expect("ArtBackground");
    ctx.diablo_ui.selhero.vec_sel_hero_dialog.insert(0, UiItem::image_clx(bg.get(0), Rect::new(0, y, 0, 0), UiFlags::ALIGN_CENTER));
}

/// Original: `SelheroClassSelectorSelect` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroClassSelectorSelect(int value) sha=8970d211c439
fn selhero_class_selector_select(ctx: &mut Ctx, value: i32) {
    let h_class = HeroClass::from_raw(ctx.diablo_ui.selhero.vec_sel_hero_dlg_items[value as usize].borrow().m_value as u8);
    if ctx.init.gb_is_spawn && (h_class == HeroClass::Rogue || h_class == HeroClass::Sorcerer || (h_class == HeroClass::Bard && !ctx.diablo.gb_bard)) {
        remove_sel_hero_background(ctx);
        crate::diablo_ui::selok::ui_sel_ok_dialog(
            ctx,
            None,
            &tr("The Rogue and Sorcerer are only available in the full retail version of Diablo. Visit https://www.gog.com/game/diablo to purchase."),
            false,
        );
        add_sel_hero_background(ctx);
        let n = ctx.diablo_ui.selhero.selhero_save_count as i32;
        selhero_list_select(ctx, n);
        return;
    }

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);

    let t = if ctx.diablo_ui.selhero.selhero_is_multi_player { tr("New Multi Player Hero") } else { tr("New Single Player Hero") };
    set_title(ctx, &t);
    ctx.diablo_ui.selhero.selhero_hero_info.name = [0; 16];
    if should_prefill_hero_name(ctx) {
        let name = selhero_generate_name(ctx, ctx.diablo_ui.selhero.selhero_hero_info.heroclass);
        ctx.diablo_ui.selhero.selhero_hero_info.set_name(name);
    }
    let mut items: Vec<UiItemRef> = Vec::new();
    let rect1 = Rect::new(ui_position.x + 242, ui_position.y + 211, 365, 33);
    items.push(UiItem::art_text(&tr("Enter Name"), rect1, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER, 3, -1));

    // The edit field writes into selhero_heroInfo.name; it is copied back in SelheroNameSelect.
    let name_buffer = Rc::new(RefCell::new(ctx.diablo_ui.selhero.selhero_hero_info.name_str().to_string()));
    ctx.diablo_ui.selhero.name_edit = name_buffer.clone();
    let rect2 = Rect::new(ui_position.x + 265, ui_position.y + 317, 320, 33);
    items.push(UiItem::edit(&tr("Enter Name"), name_buffer, 15, false, rect2, UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD));

    let rect3 = Rect::new(ui_position.x + 279, ui_position.y + 429, 140, 35);
    items.push(UiItem::art_text_button(&tr("OK"), ui_focus_navigation_select, rect3, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD));

    let rect4 = Rect::new(ui_position.x + 429, ui_position.y + 429, 144, 35);
    items.push(UiItem::art_text_button(&tr("Cancel"), ui_focus_navigation_esc, rect4, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD));

    ctx.diablo_ui.selhero.vec_sel_dlg_items = items.clone();
    ui_init_list(ctx, None, Some(selhero_name_select), Some(selhero_name_esc), &items, false, None, None, 0);
}

fn sync_name_from_edit(ctx: &mut Ctx) {
    let s = ctx.diablo_ui.selhero.name_edit.borrow().clone();
    ctx.diablo_ui.selhero.selhero_hero_info.set_name(&s);
}

/// Original: `SelheroClassSelectorEsc` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroClassSelectorEsc() sha=9e1582be6be8
fn selhero_class_selector_esc(ctx: &mut Ctx) {
    ctx.diablo_ui.selhero.vec_sel_dlg_items.clear();
    ctx.diablo_ui.selhero.vec_sel_hero_dlg_items.clear();
    if ctx.diablo_ui.selhero.selhero_save_count != 0 {
        selhero_list_init(ctx);
        return;
    }
    selhero_list_esc(ctx);
}

/// Original: `SelheroNameSelect` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroNameSelect(int) sha=300240e224a6
fn selhero_name_select(ctx: &mut Ctx, _value: i32) {
    sync_name_from_edit(ctx);
    let name = ctx.diablo_ui.selhero.selhero_hero_info.name_str().to_string();
    if ctx.diablo_ui.selhero.selhero_is_multi_player && !ui_valid_player_name(&name) {
        remove_sel_hero_background(ctx);
        let title = ctx.diablo_ui.selhero.title.borrow().clone();
        crate::diablo_ui::selok::ui_sel_ok_dialog(ctx, Some(&title), &tr("Invalid name. A name cannot contain spaces, reserved characters, or reserved words.\n"), false);
        add_sel_hero_background(ctx);
    } else {
        let mut info = ctx.diablo_ui.selhero.selhero_hero_info;
        let created = (ctx.diablo_ui.selhero.gfn_hero_create.unwrap())(ctx, &mut info);
        ctx.diablo_ui.selhero.selhero_hero_info = info;
        if created {
            selhero_load_select(ctx, 1);
            return;
        }
        crate::diablo_ui::dialogs::ui_error_ok_dialog(ctx, "", &tr("Unable to create character."), true);
    }
    ctx.diablo_ui.selhero.selhero_hero_info.name = [0; 16];
    selhero_class_selector_select(ctx, 0);
}

/// Original: `SelheroNameEsc` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroNameEsc() sha=1dfc7ea3aa46
fn selhero_name_esc(ctx: &mut Ctx) {
    let n = ctx.diablo_ui.selhero.selhero_save_count as i32;
    selhero_list_select(ctx, n);
}

/// Original: `SelheroLoadFocus` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroLoadFocus(int value) sha=fe7e4c446202
fn selhero_load_focus(_ctx: &mut Ctx, _value: i32) {}

/// Original: `SelheroLoadSelect` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroLoadSelect(int value) sha=dd2179b5d9f2
fn selhero_load_select(ctx: &mut Ctx, value: i32) {
    ui_init_list_clear(ctx);
    ctx.diablo_ui.selhero.selhero_end_menu = true;
    if ctx.diablo_ui.selhero.vec_sel_hero_dlg_items[value as usize].borrow().m_value == 0 {
        ctx.diablo_ui.selhero.selhero_result = SelheroSelections::Continue;
        return;
    }
    if !ctx.diablo_ui.selhero.selhero_is_multi_player {
        // The original's "dangerous hack": selhero's loop renders selgame's difficulty items.
        ctx.diablo_ui.selhero.selhero_end_menu = false;
        ctx.diablo_ui.selhero.selhero_is_savegame = false;
        selhero_free(ctx);
        load_background_art(ctx, "ui_art\\selgame", 1);
        crate::diablo_ui::selgame::selgame_game_selection_select(ctx, 0);
    }
    ctx.diablo_ui.selhero.selhero_result = SelheroSelections::NewDungeon;
}

/// Original: `SelheroGenerateName` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::SelheroGenerateName(HeroClass heroClass) sha=b34f39a73df6
fn selhero_generate_name(ctx: &mut Ctx, hero_class: HeroClass) -> &'static str {
    const NAMES: [[&str; 10]; 6] = [
        ["Aidan", "Qarak", "Born", "Cathan", "Halbu", "Lenalas", "Maximus", "Vane", "Myrdgar", "Rothat"],
        ["Moreina", "Akara", "Kashya", "Flavie", "Divo", "Oriana", "Iantha", "Shikha", "Basanti", "Elexa"],
        ["Jazreth", "Drognan", "Armin", "Fauztin", "Jere", "Kazzulk", "Ranslor", "Sarnakyle", "Valthek", "Horazon"],
        ["Akyev", "Dvorak", "Kekegi", "Kharazim", "Mikulov", "Shenlong", "Vedenin", "Vhalit", "Vylnas", "Zhota"],
        ["Moreina", "Akara", "Kashya", "Flavie", "Divo", "Oriana", "Iantha", "Shikha", "Basanti", "Elexa"],
        ["Alaric", "Barloc", "Egtheow", "Guthlaf", "Heorogar", "Hrothgar", "Oslaf", "Qual-Kehk", "Ragnar", "Ulf"],
    ];
    let i_rand = ctx.crt_rand.rand() % 10;
    NAMES[hero_class as usize % 6][i_rand as usize]
}

/// Original: `devilution::selhero_Init` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::selhero_Init() sha=d22d056f008d
pub fn selhero_init(ctx: &mut Ctx) {
    add_sel_hero_background(ctx);
    let mut dialog = std::mem::take(&mut ctx.diablo_ui.selhero.vec_sel_hero_dialog);
    ui_add_logo(ctx, &mut dialog);
    load_scroll_bar(ctx);

    ctx.diablo_ui.selhero.selhero_save_count = 0;
    let info_fn = ctx.diablo_ui.selhero.gfn_hero_info.unwrap();
    info_fn(ctx, &mut sel_hero_get_hero_info);
    let n = ctx.diablo_ui.selhero.selhero_save_count;
    ctx.diablo_ui.selhero.selhero_heros[..n].reverse();

    let ui_position = crate::utils::display::get_ui_rectangle(ctx);

    ctx.diablo_ui.selhero.vec_sel_dlg_items.clear();
    let title = ctx.diablo_ui.selhero.title.clone();
    dialog.push(UiItem::art_text_dynamic(
        title,
        Rect::new(ui_position.x + 24, ui_position.y + 161, 590, 35),
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER,
        3,
        -1,
    ));

    let rect = Rect::new(ui_position.x + 30, ui_position.y + 211, 180, 76);
    let hero_img = UiItem::image_clx(ui_get_hero_dialog_sprite(ctx, 0), rect, UiFlags::NONE);
    ctx.diablo_ui.selhero.selhero_dialog_hero_img = Some(hero_img.clone());
    dialog.push(hero_img);

    let label_flags = UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK | UiFlags::ALIGN_RIGHT;
    let value_flags = UiFlags::FONT_SIZE_12 | UiFlags::COLOR_UI_SILVER_DARK | UiFlags::ALIGN_CENTER;
    let label_x = ui_position.x + 39;
    let value_x = ui_position.x + 159;
    let label_width = 110;
    let value_width = 40;
    let stat_height = 21;

    let stats = ctx.diablo_ui.selhero.text_stats.clone();
    dialog.push(UiItem::art_text(&tr("Level:"), Rect::new(label_x, ui_position.y + 323, label_width, stat_height), label_flags, 1, -1));
    dialog.push(UiItem::art_text_dynamic(stats[0].clone(), Rect::new(value_x, ui_position.y + 323, value_width, stat_height), value_flags, 1, -1));

    let stat_labels = [tr("Strength:"), tr("Magic:"), tr("Dexterity:"), tr("Vitality:")];
    let mut stat_y = ui_position.y + 358;
    for (i, label) in stat_labels.iter().enumerate() {
        dialog.push(UiItem::art_text(label, Rect::new(label_x, stat_y, label_width, stat_height), label_flags, 1, -1));
        dialog.push(UiItem::art_text_dynamic(stats[i + 1].clone(), Rect::new(value_x, stat_y, value_width, stat_height), value_flags, 1, -1));
        stat_y += stat_height;
    }
    ctx.diablo_ui.selhero.vec_sel_hero_dialog = dialog;
}

/// Original: `devilution::selhero_List_Init` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::selhero_List_Init() sha=540b4cdf4d0c
pub fn selhero_list_init(ctx: &mut Ctx) {
    let ui_position = crate::utils::display::get_ui_rectangle(ctx);

    let mut selected_item = 0usize;
    let mut items: Vec<UiItemRef> = Vec::new();

    let rect1 = Rect::new(ui_position.x + 242, ui_position.y + 211, 365, 33);
    items.push(UiItem::art_text(&tr("Select Hero"), rect1, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER, 3, -1));

    let mut dlg_items: Vec<UiListItemRef> = Vec::new();
    let save_count = ctx.diablo_ui.selhero.selhero_save_count;
    for i in 0..save_count {
        let hero = ctx.diablo_ui.selhero.selhero_heros[i];
        dlg_items.push(UiListItem::new(hero.name_str(), i as i32, UiFlags::NONE));
        if hero.saveNumber == ctx.diablo_ui.selhero.selhero_hero_info.saveNumber {
            selected_item = i;
        }
    }
    dlg_items.push(UiListItem::new(&tr("New Hero"), save_count as i32, UiFlags::NONE));

    items.push(UiItem::list(&dlg_items, 6, ui_position.x + 265, ui_position.y + 256, 320, 26, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_GOLD, 1));

    let rect2 = Rect::new(ui_position.x + 585, ui_position.y + 244, 25, 178);
    {
        let sb = &ctx.diablo_ui.scrollbar;
        let bg = sb.art_scroll_bar_background.as_ref().expect("ArtScrollBarBackground").get(0);
        let thumb = sb.art_scroll_bar_thumb.as_ref().expect("ArtScrollBarThumb").get(0);
        let arrow = sb.art_scroll_bar_arrow.clone().expect("ArtScrollBarArrow");
        items.push(UiItem::scrollbar(bg, thumb, arrow, rect2, UiFlags::NONE));
    }

    let rect3 = Rect::new(ui_position.x + 239, ui_position.y + 429, 120, 35);
    items.push(UiItem::art_text_button(&tr("OK"), ui_focus_navigation_select, rect3, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD));

    let rect4 = Rect::new(ui_position.x + 364, ui_position.y + 429, 120, 35);
    let delete_button = UiItem::art_text_button(
        &tr("Delete"),
        selhero_ui_focus_navigation_yes_no,
        rect4,
        UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_SILVER | UiFlags::ELEMENT_DISABLED,
    );
    ctx.diablo_ui.selhero.sellist_dialog_delete_button = Some(delete_button.clone());
    items.push(delete_button);

    let rect5 = Rect::new(ui_position.x + 489, ui_position.y + 429, 144, 35);
    items.push(UiItem::art_text_button(&tr("Cancel"), ui_focus_navigation_esc, rect5, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_30 | UiFlags::COLOR_UI_GOLD));

    ctx.diablo_ui.selhero.vec_sel_dlg_items = items.clone();
    ctx.diablo_ui.selhero.vec_sel_hero_dlg_items = dlg_items;
    ui_init_list(
        ctx,
        Some(selhero_list_focus),
        Some(selhero_list_select),
        Some(selhero_list_esc),
        &items,
        false,
        None,
        Some(selhero_list_delete_yes_no),
        selected_item,
    );
    if ctx.diablo_ui.selhero.selhero_is_multi_player {
        set_title(ctx, &tr("Multi Player Characters"));
    } else {
        set_title(ctx, &tr("Single Player Characters"));
    }
}

/// Original: `UiSelHeroDialog` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::UiSelHeroDialog(bool (*fninfo)(bool (*fninfofunc)(_uiheroinfo *)), bool (*fncreate)(_uiheroinfo *), void (*fnstats)(unsigned int, _uidefaultstats *), bool (*fnremove)(_uiheroinfo *), _selhero_selections *dlgresult, uint32_t *saveNumber) sha=81a1f72cfc94
fn ui_sel_hero_dialog(
    ctx: &mut Ctx,
    fninfo: HeroInfoFn,
    fncreate: HeroCreateFn,
    fnstats: HeroStatsFn,
    fnremove: HeroRemoveFn,
    dlgresult: &mut SelheroSelections,
    save_number: &mut u32,
) {
    loop {
        ctx.diablo_ui.selhero.gfn_hero_info = Some(fninfo);
        ctx.diablo_ui.selhero.gfn_hero_create = Some(fncreate);
        ctx.diablo_ui.selhero.gfn_hero_stats = Some(fnstats);
        ctx.diablo_ui.selhero.selhero_result = *dlgresult;
        ctx.diablo_ui.selhero.selhero_navigate_yes_no = false;

        selhero_init(ctx);

        let save_count = ctx.diablo_ui.selhero.selhero_save_count;
        if save_count != 0 {
            ctx.diablo_ui.selhero.selhero_hero_info = UiHeroInfo::default();
            for i in 0..save_count {
                if ctx.diablo_ui.selhero.selhero_heros[i].saveNumber == *save_number {
                    ctx.diablo_ui.selhero.selhero_hero_info = ctx.diablo_ui.selhero.selhero_heros[i];
                    break;
                }
            }
            selhero_list_init(ctx);
        } else {
            selhero_list_select(ctx, save_count as i32);
        }

        ctx.diablo_ui.selhero.selhero_end_menu = false;
        while !ctx.diablo_ui.selhero.selhero_end_menu && !ctx.diablo_ui.selhero.selhero_navigate_yes_no {
            ui_clear_screen(ctx);
            let dialog = ctx.diablo_ui.selhero.vec_sel_hero_dialog.clone();
            ui_render_items(ctx, &dialog);
            render_difficulty_indicators(ctx);
            ui_poll_and_render(ctx, None);
        }
        selhero_free(ctx);

        if ctx.diablo_ui.selhero.selhero_navigate_yes_no {
            let dialog_title = if ctx.diablo_ui.selhero.selhero_is_multi_player { tr("Delete Multi Player Hero") } else { tr("Delete Single Player Hero") };
            let dialog_title = crate::utils::utf8::copy_utf8(&dialog_title, 128);
            let dialog_text = tr("Are you sure you want to delete the character \"{:s}\"?").replacen("{:s}", ctx.diablo_ui.selhero.selhero_hero_info.name_str(), 1);
            if crate::diablo_ui::selyesno::ui_sel_hero_yes_no_dialog(ctx, &dialog_title, &dialog_text) {
                let info = ctx.diablo_ui.selhero.selhero_hero_info;
                fnremove(ctx, &info);
            }
        }
        if !ctx.diablo_ui.selhero.selhero_navigate_yes_no {
            break;
        }
    }

    *dlgresult = ctx.diablo_ui.selhero.selhero_result;
    *save_number = ctx.diablo_ui.selhero.selhero_hero_info.saveNumber;
}

/// Original: `devilution::UiSelHeroSingDialog` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::UiSelHeroSingDialog(bool (*fninfo)(bool (*fninfofunc)(_uiheroinfo *)), bool (*fncreate)(_uiheroinfo *), bool (*fnremove)(_uiheroinfo *), void (*fnstats)(unsigned int, _uidefaultstats *), _selhero_selections *dlgresult, uint32_t *saveNumber, _difficulty *difficulty) sha=6a13eadaa78f
#[allow(clippy::too_many_arguments)]
pub fn ui_sel_hero_sing_dialog(
    ctx: &mut Ctx,
    fninfo: HeroInfoFn,
    fncreate: HeroCreateFn,
    fnremove: HeroRemoveFn,
    fnstats: HeroStatsFn,
    dlgresult: &mut SelheroSelections,
    save_number: &mut u32,
    difficulty: &mut _difficulty,
) {
    ctx.diablo_ui.selhero.selhero_is_multi_player = false;
    ui_sel_hero_dialog(ctx, fninfo, fncreate, fnstats, fnremove, dlgresult, save_number);
    *difficulty = ctx.diablo_ui.selgame.n_difficulty;
}

/// Original: `devilution::UiSelHeroMultDialog` (DiabloUI/hero/selhero.cpp).
// @port DiabloUI/hero/selhero.cpp|devilution::UiSelHeroMultDialog(bool (*fninfo)(bool (*fninfofunc)(_uiheroinfo *)), bool (*fncreate)(_uiheroinfo *), bool (*fnremove)(_uiheroinfo *), void (*fnstats)(unsigned int, _uidefaultstats *), _selhero_selections *dlgresult, uint32_t *saveNumber) sha=f095fefcb35d
pub fn ui_sel_hero_mult_dialog(
    ctx: &mut Ctx,
    fninfo: HeroInfoFn,
    fncreate: HeroCreateFn,
    fnremove: HeroRemoveFn,
    fnstats: HeroStatsFn,
    dlgresult: &mut SelheroSelections,
    save_number: &mut u32,
) {
    ctx.diablo_ui.selhero.selhero_is_multi_player = true;
    ui_sel_hero_dialog(ctx, fninfo, fncreate, fnstats, fnremove, dlgresult, save_number);
}
