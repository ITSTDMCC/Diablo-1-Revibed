//! `Source/DiabloUI/button.cpp`: dialog buttons.

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::diablo_ui_surface;
use crate::diablo_ui::ui_item::{UiItem, UiItemRef, UiKind};
use crate::engine::clx_sprite::ClxSprite;
use crate::engine::render::text_render::{draw_string, render_clx_sprite, UiFlags};
use crate::engine::surface::Rect;
use crate::platform::events::{Event, BUTTON_LEFT};

pub const DIALOG_BUTTON_WIDTH: i32 = 110;
pub const DIALOG_BUTTON_HEIGHT: i32 = 28;

/// Original: `devilution::LoadDialogButtonGraphics` (DiabloUI/button.cpp).
// @port DiabloUI/button.cpp|devilution::LoadDialogButtonGraphics() sha=4f71d8d78db0
pub fn load_dialog_button_graphics(ctx: &mut Ctx) {
    ctx.diablo_ui.button_sprites = crate::engine::load_sprites::load_optional_clx(ctx, "ui_art\\dvl_but_sml.clx");
    if ctx.diablo_ui.button_sprites.is_none() {
        ctx.diablo_ui.button_sprites = crate::engine::load_sprites::load_pcx_sprite_list(ctx, "ui_art\\but_sml", 15, None, None, true);
    }
}

/// Original: `devilution::FreeDialogButtonGraphics` (DiabloUI/button.cpp).
// @port DiabloUI/button.cpp|devilution::FreeDialogButtonGraphics() sha=7a1de559c39d
pub fn free_dialog_button_graphics(ctx: &mut Ctx) {
    ctx.diablo_ui.button_sprites = None;
}

/// Original: `devilution::ButtonSprite` (DiabloUI/button.cpp).
// @port DiabloUI/button.cpp|devilution::ButtonSprite(bool pressed) sha=0cd9823f34b4
pub fn button_sprite(ctx: &Ctx, pressed: bool) -> ClxSprite {
    ctx.diablo_ui.button_sprites.as_ref().expect("ButtonSprites").get(if pressed { 1 } else { 0 })
}

/// Original: `devilution::RenderButton` (DiabloUI/button.cpp).
// @port DiabloUI/button.cpp|devilution::RenderButton(const UiButton &button) sha=34901fc4eed9
pub fn render_button(ctx: &mut Ctx, button: &UiItem) {
    let UiKind::Button { text, pressed, .. } = &button.kind else { return };
    let r = button.m_rect;
    let out = diablo_ui_surface(ctx).subregion(r.x, r.y, r.w, r.h);
    let sprite = button_sprite(ctx, *pressed);
    render_clx_sprite(&out, &sprite, (0, 0));
    let mut text_rect = Rect::new(0, 0, r.w, r.h);
    if !*pressed {
        text_rect.y -= 1;
    }
    draw_string(ctx, &out, text, text_rect, UiFlags::ALIGN_CENTER | UiFlags::FONT_SIZE_DIALOG | UiFlags::COLOR_DIALOG_WHITE, 1, -1);
}

/// Original: `devilution::HandleMouseEventButton` (DiabloUI/button.cpp).
// @port DiabloUI/button.cpp|devilution::HandleMouseEventButton(const SDL_Event &event, UiButton *button) sha=9261a619ee7e
pub fn handle_mouse_event_button(ctx: &mut Ctx, event: &Event, button: &UiItemRef) -> bool {
    match event {
        Event::MouseButtonUp { button: BUTTON_LEFT, .. } => {
            let action = match &button.borrow().kind {
                UiKind::Button { pressed: true, action, .. } => Some(*action),
                _ => None,
            };
            match action {
                Some(a) => {
                    a(ctx);
                    true
                }
                None => false,
            }
        }
        Event::MouseButtonDown { button: BUTTON_LEFT, .. } => {
            if let UiKind::Button { pressed, .. } = &mut button.borrow_mut().kind {
                *pressed = true;
            }
            true
        }
        _ => false,
    }
}

/// Original: `devilution::HandleGlobalMouseUpButton` (DiabloUI/button.cpp).
// @port DiabloUI/button.cpp|devilution::HandleGlobalMouseUpButton(UiButton *button) sha=af021462baea
pub fn handle_global_mouse_up_button(button: &UiItemRef) {
    if let UiKind::Button { pressed, .. } = &mut button.borrow_mut().kind {
        *pressed = false;
    }
}
