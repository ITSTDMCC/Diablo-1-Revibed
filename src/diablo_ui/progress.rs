//! `Source/DiabloUI/progress.cpp`: the progress dialog shown while a joining player waits for
//! the game state (`msg_wait_resync`).

use crate::ctx::Ctx;
use crate::diablo_ui::button::{DIALOG_BUTTON_HEIGHT, DIALOG_BUTTON_WIDTH};
use crate::diablo_ui::diabloui::{diablo_ui_surface, draw_mouse, get_center_offset, ui_fade_in, ui_handle_events, ui_item_mouse_events, ui_render_items};
use crate::diablo_ui::ui_item::{UiItem, UiItemRef};
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::load_sprites::load_pcx;
use crate::engine::render::text_render::{render_clx_sprite, UiFlags};
use crate::engine::surface::Rect;
use crate::platform::events::Event;
use crate::utils::language::tr;

/// Globals of progress.cpp.
#[derive(Default)]
pub struct ProgressState {
    art_popup_sm: Option<ClxSpriteList>,
    art_prog_bg: Option<ClxSpriteList>,
    prog_fil: Option<ClxSpriteList>,
    vec_progress: Vec<UiItemRef>,
    end_menu: bool,
}

/// Original: `DialogActionCancel` (DiabloUI/progress.cpp).
// @port DiabloUI/progress.cpp|devilution::DialogActionCancel() sha=d2e0e89cbe20
fn dialog_action_cancel(ctx: &mut Ctx) {
    ctx.diablo_ui.progress.end_menu = true;
}

/// Original: `ProgressLoadBackground` (DiabloUI/progress.cpp).
// @port DiabloUI/progress.cpp|devilution::ProgressLoadBackground() sha=d5796cb5118f
fn progress_load_background(ctx: &mut Ctx) {
    crate::diablo_ui::diabloui::ui_load_black_background(ctx);
    ctx.diablo_ui.progress.art_popup_sm = load_pcx(ctx, "ui_art\\spopup", None, None, true);
    ctx.diablo_ui.progress.art_prog_bg = load_pcx(ctx, "ui_art\\prog_bg", None, None, true);
}

/// Original: `ProgressLoadForeground` (DiabloUI/progress.cpp).
// @port DiabloUI/progress.cpp|devilution::ProgressLoadForeground() sha=be66703496cd
fn progress_load_foreground(ctx: &mut Ctx) {
    crate::diablo_ui::button::load_dialog_button_graphics(ctx);
    ctx.diablo_ui.progress.prog_fil = load_pcx(ctx, "ui_art\\prog_fil", None, None, true);
    let ui_position = crate::utils::display::get_ui_rectangle(ctx);
    let rect3 = Rect::new(ui_position.x + 265, ui_position.y + 267, DIALOG_BUTTON_WIDTH, DIALOG_BUTTON_HEIGHT);
    ctx.diablo_ui.progress.vec_progress.push(UiItem::button(&tr("Cancel"), dialog_action_cancel, rect3, UiFlags::NONE));
}

/// Original: `ProgressFreeBackground` (DiabloUI/progress.cpp).
// @port DiabloUI/progress.cpp|devilution::ProgressFreeBackground() sha=eba0ff715ecb
fn progress_free_background(ctx: &mut Ctx) {
    ctx.diablo_ui.art_background = None;
    ctx.diablo_ui.progress.art_popup_sm = None;
    ctx.diablo_ui.progress.art_prog_bg = None;
}

/// Original: `ProgressFreeForeground` (DiabloUI/progress.cpp).
// @port DiabloUI/progress.cpp|devilution::ProgressFreeForeground() sha=81e306ab8f14
fn progress_free_foreground(ctx: &mut Ctx) {
    ctx.diablo_ui.progress.vec_progress.clear();
    ctx.diablo_ui.progress.prog_fil = None;
    crate::diablo_ui::button::free_dialog_button_graphics(ctx);
}

/// Original: `GetPosition` (DiabloUI/progress.cpp).
// @port DiabloUI/progress.cpp|devilution::GetPosition() sha=5bbc283b36d1
fn get_position(ctx: &Ctx) -> (i32, i32) {
    (get_center_offset(ctx, 280, 0), get_center_offset(ctx, 144, ctx.dx.gn_screen_height))
}

/// Original: `ProgressRenderBackground` (DiabloUI/progress.cpp).
// @port DiabloUI/progress.cpp|devilution::ProgressRenderBackground() sha=2c25f2e75db6
fn progress_render_background(ctx: &mut Ctx) {
    let out = diablo_ui_surface(ctx);
    for y in 0..out.h() {
        out.row(y).fill(0);
    }
    let position = get_position(ctx);
    let p = &ctx.diablo_ui.progress;
    if let Some(popup) = &p.art_popup_sm {
        render_clx_sprite(&out.subregion(position.0, position.1, 280, 140), &popup.get(0), (0, 0));
    }
    if let Some(bg) = &p.art_prog_bg {
        render_clx_sprite(&out.subregion(get_center_offset(ctx, 227, 0), 0, 227, out.h()), &bg.get(0), (0, position.1 + 52));
    }
}

/// Original: `ProgressRenderForeground` (DiabloUI/progress.cpp).
// @port DiabloUI/progress.cpp|devilution::ProgressRenderForeground(int progress) sha=d80c18b80964
fn progress_render_foreground(ctx: &mut Ctx, progress: i32) {
    let out = diablo_ui_surface(ctx);
    let position = get_position(ctx);
    if progress != 0 {
        let x = get_center_offset(ctx, 227, 0);
        let w = 227 * progress / 100;
        if let Some(fil) = &ctx.diablo_ui.progress.prog_fil {
            render_clx_sprite(&out.subregion(x, 0, w, out.h()), &fil.get(0), (0, position.1 + 52));
        }
    }
    // Not rendering an actual button, only the top 2 rows of its graphics.
    let button = crate::diablo_ui::button::button_sprite(ctx, false);
    render_clx_sprite(&out.subregion(get_center_offset(ctx, 110, 0), position.1 + 99, DIALOG_BUTTON_WIDTH, 2), &button, (0, 0));
}

/// Original: `devilution::UiProgressDialog` (DiabloUI/progress.cpp). Returns whether the
/// progress reached 100 before the player cancelled.
// @port DiabloUI/progress.cpp|devilution::UiProgressDialog(int (*fnfunc)()) sha=9a7c991ce84c
pub fn ui_progress_dialog(ctx: &mut Ctx, fnfunc: fn(&mut Ctx) -> i32) -> bool {
    crate::engine::palette::set_fade_level(ctx, 256, true);

    // Blit the background once and then free it.
    progress_load_background(ctx);
    progress_render_background(ctx);
    // RenderDirectlyToOutputSurface is never set in the port.
    progress_free_background(ctx);

    progress_load_foreground(ctx);

    ctx.diablo_ui.progress.end_menu = false;
    let mut progress = 0;
    while !ctx.diablo_ui.progress.end_menu && progress < 100 {
        progress = fnfunc(ctx);
        progress_render_foreground(ctx, progress);
        let items = ctx.diablo_ui.progress.vec_progress.clone();
        ui_render_items(ctx, &items);
        draw_mouse(ctx);
        ui_fade_in(ctx);

        while let Some(event) = crate::engine::events::poll_event(ctx) {
            match event {
                Event::MouseButtonDown { .. } | Event::MouseButtonUp { .. } => {
                    ui_item_mouse_events(ctx, &event, &items);
                }
                _ => {
                    // The original's `case SDLK_ESCAPE/SDLK_RETURN/SDLK_SPACE/SDLK_KP_ENTER` compare
                    // key codes with the event type, so they never match; menu actions end it.
                    for menu_action in crate::controls::controller::get_menu_actions(ctx, &event) {
                        use crate::controls::controller::MenuAction;
                        if !matches!(menu_action, MenuAction::Back | MenuAction::Select) {
                            continue;
                        }
                        ctx.diablo_ui.progress.end_menu = true;
                        break;
                    }
                }
            }
            ui_handle_events(ctx, &event);
        }
    }
    progress_free_foreground(ctx);

    progress == 100
}
