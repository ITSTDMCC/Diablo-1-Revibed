//! `Source/DiabloUI/credits.cpp`: the credits and support screens.

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::{diablo_ui_surface, load_background_art, ui_fade_in, ui_handle_events};
use crate::diablo_ui::text_lines::{CREDIT_LINES, SUPPORT_LINES};
use crate::engine::render::text_render::{draw_string_at, render_clx_sprite, word_wrap_string, GameFontTables, UiFlags};
use crate::platform::events::Event;
use crate::utils::language::tr;

/// `VIEWPORT`
const VIEWPORT: (i32, i32, i32, i32) = (0, 114, 640, 251);
const LINE_H: i32 = 22;

/// `MAX_VISIBLE_LINES`: the number of whole lines (VIEWPORT.h / LINE_H) rounded up, plus one
/// extra line for when a line is leaving the screen while another one is entering.
const MAX_VISIBLE_LINES: usize = ((VIEWPORT.3 - 1) / LINE_H + 2) as usize;

/// `CreditsRenderer::LineContent`
struct LineContent {
    offset: u16,
    text: String,
}

/// `CreditsRenderer`
struct CreditsRenderer {
    lines_to_render: Vec<LineContent>,
    finished: bool,
    ticks_begin: u32,
    prev_offset_y: i32,
}

impl CreditsRenderer {
    /// Original: `CreditsRenderer::CreditsRenderer` (DiabloUI/credits.cpp).
    // @port DiabloUI/credits.cpp|devilution::CreditsRenderer::CreditsRenderer(char const *const *text, std::size_t textLines) sha=d47b30d6b119
    fn new(ctx: &mut Ctx, text: &[&str]) -> CreditsRenderer {
        let mut lines_to_render = Vec::new();
        for line in text {
            let org_text = tr(line);

            let mut offset: u16 = 0;
            let mut index_first_not_tab = 0;
            let bytes = org_text.as_bytes();
            while index_first_not_tab < bytes.len() && bytes[index_first_not_tab] == b'\t' {
                offset += 40;
                index_first_not_tab += 1;
            }

            let paragraphs = word_wrap_string(ctx, &org_text[index_first_not_tab..], 580 - offset as u32, GameFontTables::FontSizeDialog, 1);

            for part in paragraphs.split('\n') {
                lines_to_render.push(LineContent { offset, text: part.to_string() });
            }
        }

        CreditsRenderer { lines_to_render, finished: false, ticks_begin: ctx.platform.ticks(), prev_offset_y: 0 }
    }

    /// Original: `CreditsRenderer::Render` (DiabloUI/credits.cpp).
    // @port DiabloUI/credits.cpp|devilution::CreditsRenderer::Render() sha=92f07ade1686
    fn render(&mut self, ctx: &mut Ctx) {
        let offset_y = -VIEWPORT.3 + (ctx.platform.ticks().wrapping_sub(self.ticks_begin) / 40) as i32;
        if offset_y == self.prev_offset_y {
            return;
        }
        self.prev_offset_y = offset_y;

        let surface = diablo_ui_surface(ctx);
        for y in 0..surface.h() {
            surface.row(y)[..surface.w() as usize].fill(0);
        }
        let ui_position = crate::utils::display::get_ui_rectangle(ctx);
        if let Some(w) = &ctx.diablo_ui.art_background_widescreen {
            render_clx_sprite(&surface, &w.get(0), (ui_position.x - 320, ui_position.y));
        }
        let bg = ctx.diablo_ui.art_background.as_ref().expect("ArtBackground").get(0);
        render_clx_sprite(&surface, &bg, (ui_position.x, ui_position.y));

        let lines_begin = (offset_y / LINE_H).max(0) as usize;
        let lines_end = (lines_begin + MAX_VISIBLE_LINES).min(self.lines_to_render.len());

        if lines_begin >= lines_end {
            if lines_end == self.lines_to_render.len() {
                self.finished = true;
            }
            return;
        }

        // ScaleOutputRect: the port draws the menus at the output resolution, so it is the identity.
        let viewport = (VIEWPORT.0 + ui_position.x, VIEWPORT.1 + ui_position.y, VIEWPORT.2, VIEWPORT.3);
        let out = surface.subregion(viewport.0, viewport.1, viewport.2, viewport.3);

        let mut dest_y = ui_position.y + VIEWPORT.1 - (offset_y - lines_begin as i32 * LINE_H);
        for line in &self.lines_to_render[lines_begin..lines_end] {
            let dest_x = ui_position.x + VIEWPORT.0 + 31;
            let pos = (dest_x + line.offset as i32 - viewport.0, dest_y - viewport.1);
            draw_string_at(ctx, &out, &line.text, pos, UiFlags::FONT_SIZE_DIALOG | UiFlags::COLOR_DIALOG_WHITE, -1, -1);
            dest_y += LINE_H;
        }
    }
}

/// Original: `TextDialog` (DiabloUI/credits.cpp).
// @port DiabloUI/credits.cpp|devilution::TextDialog(char const *const *text, std::size_t textLines) sha=927c9a3a4db1
fn text_dialog(ctx: &mut Ctx, text: &[&str]) -> bool {
    use crate::controls::controller::MenuAction;
    let mut credits_renderer = CreditsRenderer::new(ctx, text);
    let mut end_menu = false;

    if crate::hwcursor::is_hardware_cursor(ctx) {
        crate::hwcursor::set_hardware_cursor_visible(ctx, false);
    }

    loop {
        credits_renderer.render(ctx);
        ui_fade_in(ctx);
        while let Some(event) = crate::engine::events::poll_event(ctx) {
            match event {
                Event::KeyDown { .. } | Event::MouseButtonUp { .. } => end_menu = true,
                _ => {
                    for menu_action in crate::controls::controller::get_menu_actions(ctx, &event) {
                        if !matches!(menu_action, MenuAction::Back | MenuAction::Select) {
                            continue;
                        }
                        end_menu = true;
                        break;
                    }
                }
            }
            ui_handle_events(ctx, &event);
        }
        if end_menu || credits_renderer.finished {
            break;
        }
    }

    // ~CreditsRenderer
    ctx.diablo_ui.art_background_widescreen = None;
    ctx.diablo_ui.art_background = None;
    true
}

/// Original: `devilution::UiCreditsDialog` (DiabloUI/credits.cpp).
// @port DiabloUI/credits.cpp|devilution::UiCreditsDialog() sha=e9dc5b34603b
pub fn ui_credits_dialog(ctx: &mut Ctx) -> bool {
    ctx.diablo_ui.art_background_widescreen = crate::engine::load_sprites::load_optional_clx(ctx, "ui_art\\creditsw.clx");
    load_background_art(ctx, "ui_art\\credits", 1);

    text_dialog(ctx, &CREDIT_LINES)
}

/// Original: `devilution::UiSupportDialog` (DiabloUI/credits.cpp).
// @port DiabloUI/credits.cpp|devilution::UiSupportDialog() sha=a8dc8a614c2f
pub fn ui_support_dialog(ctx: &mut Ctx) -> bool {
    if ctx.init.gb_is_hellfire {
        ctx.diablo_ui.art_background_widescreen = crate::engine::load_sprites::load_optional_clx(ctx, "ui_art\\supportw.clx");
        load_background_art(ctx, "ui_art\\support", 1);
    } else {
        ctx.diablo_ui.art_background_widescreen = crate::engine::load_sprites::load_optional_clx(ctx, "ui_art\\creditsw.clx");
        load_background_art(ctx, "ui_art\\credits", 1);
    }

    text_dialog(ctx, &SUPPORT_LINES)
}
