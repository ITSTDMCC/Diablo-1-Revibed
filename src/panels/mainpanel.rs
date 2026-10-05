//! `Source/panels/mainpanel.cpp`: pre-rendered (translated) main panel buttons.

use crate::control::{get_main_panel, is_chat_available, PAN_BTN_POS};
use crate::ctx::Ctx;
use crate::engine::load_sprites::{load_clx, load_optional_clx};
use crate::engine::render::text_render::{draw_string, get_line_width, render_clx_sprite, GameFontTables, UiFlags};
use crate::engine::surface::{OwnedSurface, Rect, Surface};
use crate::utils::language::tr;
use crate::utils::surface_to_clx::surface_to_clx;

/// Original: `DrawButtonText` (panels/mainpanel.cpp).
// @port panels/mainpanel.cpp|devilution::DrawButtonText(const Surface &out, string_view text, Rectangle placement, UiFlags style, int spacing = 1) sha=7f1f758eed8e
fn draw_button_text(ctx: &mut Ctx, out: &Surface, text: &str, placement: Rect, style: UiFlags, spacing: i32) {
    let shadow = Rect::new(placement.x, placement.y + 1, placement.w, placement.h);
    draw_string(ctx, out, text, shadow, UiFlags::ALIGN_CENTER | UiFlags::KERNING_FIT_SPACING | UiFlags::COLOR_BLACK, spacing, -1);
    draw_string(ctx, out, text, placement, UiFlags::ALIGN_CENTER | UiFlags::KERNING_FIT_SPACING | style, spacing, -1);
}

/// The text width and spacing shared by `DrawButtonOnPanel` and `RenderMainButton`.
fn fitted_width(ctx: &mut Ctx, text: &str, button_width: i32) -> (i32, i32) {
    let mut spacing = 2;
    let mut width = get_line_width(ctx, text, GameFontTables::GameFont12, spacing, None).min(button_width);
    if width > 38 {
        spacing = 1;
        width = get_line_width(ctx, text, GameFontTables::GameFont12, spacing, None).min(button_width);
    }
    (width, spacing)
}

/// Original: `DrawButtonOnPanel` (panels/mainpanel.cpp).
// @port panels/mainpanel.cpp|devilution::DrawButtonOnPanel(Point position, string_view text, int frame) sha=7a93f2f24fb3
fn draw_button_on_panel(ctx: &mut Ctx, position: (i32, i32), text: &str, frame: usize) {
    let btm = Surface::of(ctx.control.p_btm_buff.as_mut().expect("pBtmBuff"));
    let panel_button = ctx.panels.panel_button.clone().expect("PanelButton");
    let grime = ctx.panels.panel_button_grime.clone().expect("PanelButtonGrime");
    render_clx_sprite(&btm, &panel_button.get(frame), position);
    let button_width = panel_button.get(0).width() as i32;
    let (width, spacing) = fitted_width(ctx, text, button_width);
    render_clx_sprite(&btm.subregion(position.0 + (button_width - width) / 2, position.1 + 7, width, btm.h() - 7), &grime.get(frame), (0, 0));
    draw_button_text(ctx, &btm, text, Rect::new(position.0, position.1, button_width, 0), UiFlags::COLOR_BUTTONFACE, spacing);
}

/// Original: `RenderMainButton` (panels/mainpanel.cpp).
// @port panels/mainpanel.cpp|devilution::RenderMainButton(const Surface &out, int buttonId, string_view text, int frame) sha=4bcfacd5d711
fn render_main_button(ctx: &mut Ctx, out: &Surface, button_id: usize, text: &str, frame: usize) {
    let panel_position = (PAN_BTN_POS[button_id].x + 4, PAN_BTN_POS[button_id].y + 17);
    draw_button_on_panel(ctx, panel_position, text, frame);
    if is_chat_available(ctx) {
        let h = get_main_panel(ctx).h;
        draw_button_on_panel(ctx, (panel_position.0, panel_position.1 + h + 16), text, frame);
    }
    let position = (0, 19 * button_id as i32);
    let button_width = ctx.panels.panel_button.as_ref().expect("PanelButton").get(0).width() as i32;
    let (width, spacing) = fitted_width(ctx, text, button_width);
    let down_grime = ctx.panels.panel_button_down_grime.clone().expect("PanelButtonDownGrime");
    render_clx_sprite(&out.subregion(position.0 + (button_width - width) / 2, position.1 + 9, width, out.h() - position.1 - 9), &down_grime.get(frame), (0, 0));
    draw_button_text(ctx, out, text, Rect::new(position.0, position.1 + 2, out.w(), 0), UiFlags::COLOR_BUTTONPUSHED, spacing);
}

/// Original: `devilution::LoadMainPanel` (panels/mainpanel.cpp).
// @port panels/mainpanel.cpp|devilution::LoadMainPanel() sha=45016d671225
pub fn load_main_panel(ctx: &mut Ctx) {
    const NUM_BUTTON_SPRITES: u32 = 6;
    let mut owned = {
        let background = load_clx(ctx, "data\\panel8bucp.clx");
        let first = background.get(0);
        let mut owned = OwnedSurface::new(first.width() as i32, first.height() as i32 * NUM_BUTTON_SPRITES as i32);
        let out = owned.view();
        let mut y = 0;
        for sprite in background.iter() {
            render_clx_sprite(&out, &sprite, (0, y));
            y += sprite.height() as i32;
        }
        owned
    };
    let out = owned.view();

    ctx.panels.panel_button = load_optional_clx(ctx, "data\\panel8buc.clx");
    ctx.panels.panel_button_grime = load_optional_clx(ctx, "data\\dirtybuc.clx");
    ctx.panels.panel_button_down_grime = load_optional_clx(ctx, "data\\dirtybucp.clx");

    render_main_button(ctx, &out, 0, &tr("char"), 0);
    render_main_button(ctx, &out, 1, &tr("quests"), 1);
    render_main_button(ctx, &out, 2, &tr("map"), 1);
    render_main_button(ctx, &out, 3, &tr("menu"), 0);
    render_main_button(ctx, &out, 4, &tr("inv"), 1);
    render_main_button(ctx, &out, 5, &tr("spells"), 0);
    ctx.panels.panel_button_down = Some(surface_to_clx(&out, NUM_BUTTON_SPRITES, None));
    drop(owned);

    if is_chat_available(ctx) {
        let talk_button = load_clx(ctx, "data\\talkbutton.clx");
        let talk_button_width = talk_button.get(0).width() as i32;
        let panel_button = ctx.panels.panel_button.clone().expect("PanelButton");
        let grime = ctx.panels.panel_button_grime.clone().expect("PanelButtonGrime");
        let main_h = get_main_panel(ctx).h;

        const NUM_OTHER_PLAYERS: i32 = 3;
        // Render the unpressed voice buttons to pBtmBuff.
        let text = tr("voice");
        let text_width = get_line_width(ctx, &text, GameFontTables::GameFont12, 1, None);
        let btm = Surface::of(ctx.control.p_btm_buff.as_mut().expect("pBtmBuff"));
        for i in 0..NUM_OTHER_PLAYERS {
            let position = (176, main_h + 101 + 18 * i);
            render_clx_sprite(&btm, &talk_button.get(0), position);
            let width = text_width.min(panel_button.get(0).width() as i32);
            render_clx_sprite(&btm.subregion(position.0 + (talk_button_width - width) / 2, position.1 + 6, width, 9), &grime.get(1), (0, 0));
            draw_button_text(ctx, &btm, &text, Rect::new(position.0, position.1, talk_button_width, 0), UiFlags::COLOR_BUTTONFACE, 1);
        }

        let talk_button_height = talk_button.get(0).height() as i32;
        const NUM_TALK_BUTTON_SPRITES: u32 = 3;
        let mut talk_owned = OwnedSurface::new(talk_button_width, talk_button_height * NUM_TALK_BUTTON_SPRITES as i32);
        let talk_surface = talk_owned.view();

        // Prerender translated versions of the other button states for voice buttons
        render_clx_sprite(&talk_surface, &talk_button.get(0), (0, 0)); // background for unpressed mute button
        render_clx_sprite(&talk_surface, &talk_button.get(1), (0, talk_button_height)); // background for pressed mute button
        render_clx_sprite(&talk_surface, &talk_button.get(1), (0, talk_button_height * 2)); // background for pressed voice button
        drop(talk_button);

        let mute = tr("mute");
        let mute_width = get_line_width(ctx, &mute, GameFontTables::GameFont12, 2, None);
        render_clx_sprite(&talk_surface.subregion((talk_button_width - mute_width) / 2, 6, mute_width, 9), &grime.get(1), (0, 0));
        draw_button_text(ctx, &talk_surface, &mute, Rect::new(0, 0, talk_button_width, 0), UiFlags::COLOR_BUTTONFACE, 1);
        render_clx_sprite(&talk_surface.subregion((talk_button_width - mute_width) / 2, 23, mute_width, 9), &grime.get(1), (0, 0));
        draw_button_text(ctx, &talk_surface, &mute, Rect::new(0, 17, talk_button_width, 0), UiFlags::COLOR_BUTTONPUSHED, 1);
        let voice = tr("voice");
        let voice_width = get_line_width(ctx, &voice, GameFontTables::GameFont12, 2, None);
        render_clx_sprite(&talk_surface.subregion((talk_button_width - voice_width) / 2, 39, voice_width, 9), &grime.get(1), (0, 0));
        draw_button_text(ctx, &talk_surface, &voice, Rect::new(0, 33, talk_button_width, 0), UiFlags::COLOR_BUTTONPUSHED, 1);
        ctx.panels.talk_button = Some(surface_to_clx(&talk_surface, NUM_TALK_BUTTON_SPRITES, None));
        drop(talk_owned);
    }

    ctx.panels.panel_button_down_grime = None;
    ctx.panels.panel_button_grime = None;
    ctx.panels.panel_button = None;
}

/// Original: `devilution::FreeMainPanel` (panels/mainpanel.cpp).
// @port panels/mainpanel.cpp|devilution::FreeMainPanel() sha=148387f471ce
pub fn free_main_panel(ctx: &mut Ctx) {
    ctx.panels.talk_button = None;
    ctx.panels.panel_button_down = None;
}
