//! `Source/DiabloUI/title.cpp`: the title screen.

use crate::controls::controller::{get_menu_actions, MenuAction};
use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{load_background_art, ui_add_background, ui_fade_in, ui_handle_events, ui_render_items};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef};
use crate::engine::load_sprites::{load_optional_clx, load_pcx_sprite_list};
use crate::engine::render::text_render::UiFlags;
use crate::engine::surface::Rect;
use crate::platform::events::Event;
use crate::utils::language::tr;

/// Original: `TitleLoad` (DiabloUI/title.cpp).
// @port DiabloUI/title.cpp|devilution::TitleLoad() sha=264240035444
fn title_load(ctx: &mut Ctx) {
    if ctx.init.gb_is_hellfire {
        load_background_art(ctx, "ui_art\\hf_logo1", 16);
        ctx.diablo_ui.art_background_widescreen = load_optional_clx(ctx, "ui_art\\hf_titlew.clx");
    } else {
        load_background_art(ctx, "ui_art\\title", 1);
        ctx.diablo_ui.diablo_title_logo = load_pcx_sprite_list(ctx, "ui_art\\logo", 15, Some(250), None, true);
    }
}

/// Original: `TitleFree` (DiabloUI/title.cpp).
// @port DiabloUI/title.cpp|devilution::TitleFree() sha=eec7b2426180
fn title_free(ctx: &mut Ctx, vec_title_screen: &mut Vec<UiItemRef>) {
    ctx.diablo_ui.art_background = None;
    ctx.diablo_ui.art_background_widescreen = None;
    ctx.diablo_ui.diablo_title_logo = None;
    vec_title_screen.clear();
}

/// Original: `devilution::UiTitleDialog` (DiabloUI/title.cpp).
// @port DiabloUI/title.cpp|devilution::UiTitleDialog() sha=b860bf450660
pub fn ui_title_dialog(ctx: &mut Ctx) {
    let mut vec_title_screen: Vec<UiItemRef> = Vec::new();
    title_load(ctx);
    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    if ctx.init.gb_is_hellfire {
        let rect = Rect::new(0, ui_position.y, 0, 0);
        if let Some(w) = &ctx.diablo_ui.art_background_widescreen {
            vec_title_screen.push(UiItem::image_clx(w.get(0), rect, UiFlags::ALIGN_CENTER));
        }
        let bg = ctx.diablo_ui.art_background.clone().expect("ArtBackground");
        vec_title_screen.push(UiItem::image_animated_clx(bg, rect, UiFlags::ALIGN_CENTER));
    } else {
        ui_add_background(ctx, &mut vec_title_screen);
        let logo = ctx.diablo_ui.diablo_title_logo.clone().expect("DiabloTitleLogo");
        vec_title_screen.push(UiItem::image_animated_clx(logo, Rect::new(0, ui_position.y + 182, 0, 0), UiFlags::ALIGN_CENTER));
        let rect = Rect::new(ui_position.x, ui_position.y + 410, 640, 26);
        vec_title_screen.push(UiItem::art_text(
            &tr("Copyright © 1996-2001 Blizzard Entertainment"),
            rect,
            UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_24 | UiFlags::COLOR_UI_SILVER,
            1,
            -1,
        ));
    }

    let mut end_menu = false;
    let time_out = ctx.platform.ticks().wrapping_add(7000);
    while !end_menu && ctx.platform.ticks() < time_out {
        ui_render_items(ctx, &vec_title_screen);
        ui_fade_in(ctx);
        // discord_manager::UpdateMenu: Discord is off in the port.
        while let Some(event) = crate::engine::events::poll_event(ctx) {
            let menu_actions = get_menu_actions(ctx, &event);
            if menu_actions.iter().any(|a| *a != MenuAction::None) {
                end_menu = true;
                break;
            }
            if matches!(event, Event::KeyDown { .. } | Event::MouseButtonUp { .. }) {
                end_menu = true;
            }
            ui_handle_events(ctx, &event);
        }
    }
    title_free(ctx, &mut vec_title_screen);
}
