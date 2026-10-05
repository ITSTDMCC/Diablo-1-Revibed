//! `Source/qol/xpbar.cpp`: the experience bar.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::geometry::{Displacement, Point};
use crate::engine::render::text_render::UiFlags;
use crate::engine::surface::Surface;
use crate::player::MaxCharacterLevel;
use crate::tables::playerdat::ExpLvlsTbl;
use crate::utils::format_int::format_integer;
use crate::utils::language::tr;

const BAR_WIDTH: i32 = 307;

type ColorGradient = [u8; 12];
const GOLD_GRADIENT: ColorGradient = [0xCF, 0xCE, 0xCD, 0xCC, 0xCB, 0xCA, 0xC9, 0xC8, 0xC7, 0xC6, 0xC5, 0xC4];
const SILVER_GRADIENT: ColorGradient = [0xFE, 0xFD, 0xFC, 0xFB, 0xFA, 0xF9, 0xF8, 0xF7, 0xF6, 0xF5, 0xF4, 0xF3];

const BACK_WIDTH: i32 = 313;
const BACK_HEIGHT: i32 = 9;

/// Globals of qol/xpbar.cpp.
#[derive(Default)]
pub struct XpBarState {
    /// `xpbarArt`
    xpbar_art: Option<ClxSpriteList>,
}

/// Original: `DrawBar` (qol/xpbar.cpp).
// @port qol/xpbar.cpp|devilution::DrawBar(const Surface &out, Point screenPosition, int width, const ColorGradient &gradient) sha=e542aa23a0c5
fn draw_bar(out: &Surface, screen_position: Point, width: i32, gradient: &ColorGradient) {
    crate::engine::unsafe_draw_horizontal_line(out, screen_position + Displacement::new(0, 1), width, gradient[gradient.len() * 3 / 4 - 1]);
    crate::engine::unsafe_draw_horizontal_line(out, screen_position + Displacement::new(0, 2), width, gradient[gradient.len() - 1]);
    crate::engine::unsafe_draw_horizontal_line(out, screen_position + Displacement::new(0, 3), width, gradient[gradient.len() / 2 - 1]);
}

/// Original: `DrawEndCap` (qol/xpbar.cpp).
// @port qol/xpbar.cpp|devilution::DrawEndCap(const Surface &out, Point point, int idx, const ColorGradient &gradient) sha=8b7e05396c9c
fn draw_end_cap(out: &Surface, point: Point, idx: usize, gradient: &ColorGradient) {
    out.set_pixel(point.x, point.y + 1, gradient[idx * 3 / 4]);
    out.set_pixel(point.x, point.y + 2, gradient[idx]);
    out.set_pixel(point.x, point.y + 3, gradient[idx / 2]);
}

/// Original: `devilution::InitXPBar` (qol/xpbar.cpp).
// @port qol/xpbar.cpp|devilution::InitXPBar() sha=2f815b36ec18
pub fn init_xp_bar(ctx: &mut Ctx) {
    if ctx.options.gameplay.experience_bar.get() {
        ctx.xpbar.xpbar_art = Some(crate::engine::load_sprites::load_clx(ctx, "data\\xpbar.clx"));
    }
}

/// Original: `devilution::FreeXPBar` (qol/xpbar.cpp).
// @port qol/xpbar.cpp|devilution::FreeXPBar() sha=8c47c6e6f6f8
pub fn free_xp_bar(ctx: &mut Ctx) {
    ctx.xpbar.xpbar_art = None;
}

/// Original: `devilution::DrawXPBar` (qol/xpbar.cpp).
// @port qol/xpbar.cpp|devilution::DrawXPBar(const Surface &out) sha=246cf9a2099c
pub fn draw_xp_bar(ctx: &mut Ctx, out: &Surface) {
    if !ctx.options.gameplay.experience_bar.get() || ctx.control.talkflag {
        return;
    }
    let player = &ctx.players.Players[ctx.players.MyPlayer.expect("MyPlayer")];
    let main_panel = crate::control::get_main_panel(ctx);
    let back = Point::new(main_panel.x + main_panel.w / 2 - 155, main_panel.y + main_panel.h - 11);
    let position = back + Displacement::new(3, 2);
    let art = ctx.xpbar.xpbar_art.as_ref().expect("xpbarArt").get(0);
    crate::engine::render::text_render::render_clx_sprite(out, &art, (back.x, back.y));
    let char_level = player._pLevel;
    if char_level as i32 == MaxCharacterLevel {
        // Draw a nice golden bar for max level characters.
        draw_bar(out, position, BAR_WIDTH, &GOLD_GRADIENT);
        return;
    }
    let prev_xp = ExpLvlsTbl[(char_level - 1) as usize] as u64;
    let exp = player._pExperience as u64;
    if exp < prev_xp {
        return;
    }
    let prev_xp_delta1 = exp - prev_xp;
    let prev_xp_delta = ExpLvlsTbl[char_level as usize] as u64 - prev_xp;
    let full_bar = BAR_WIDTH as u64 * prev_xp_delta1 / prev_xp_delta;
    // Figure out how much to fill the last pixel of the XP bar, to make it gradually appear with gained XP
    let one_px = prev_xp_delta / BAR_WIDTH as u64 + 1;
    let last_full_px = full_bar * prev_xp_delta / BAR_WIDTH as u64;
    let fade = (prev_xp_delta1 - last_full_px) * (SILVER_GRADIENT.len() as u64 - 1) / one_px;
    // Draw beginning of bar full brightness
    draw_bar(out, position, full_bar as i32, &SILVER_GRADIENT);
    // End pixels appear gradually
    draw_end_cap(out, position + Displacement::new(full_bar as i32, 0), fade as usize, &SILVER_GRADIENT);
}

/// Original: `devilution::CheckXPBarInfo` (qol/xpbar.cpp).
// @port qol/xpbar.cpp|devilution::CheckXPBarInfo() sha=d42b5c6fb2c4
pub fn check_xp_bar_info(ctx: &mut Ctx) -> bool {
    if !ctx.options.gameplay.experience_bar.get() {
        return false;
    }
    let main_panel = crate::control::get_main_panel(ctx);
    let back_x = main_panel.x + main_panel.w / 2 - 155;
    let back_y = main_panel.y + main_panel.h - 11;
    let (mx, my) = ctx.diablo.mouse_position;
    if mx < back_x || mx >= back_x + BACK_WIDTH || my < back_y || my >= back_y + BACK_HEIGHT {
        return false;
    }
    let player = &ctx.players.Players[ctx.players.MyPlayer.expect("MyPlayer")];
    let char_level = player._pLevel;
    let exp = player._pExperience;
    crate::control::add_panel_string(ctx, &tr("Level {:d}").replacen("{:d}", &char_level.to_string(), 1));
    if char_level as i32 == MaxCharacterLevel {
        // Show a maximum level indicator for max level players.
        ctx.control.info_color = UiFlags::COLOR_WHITEGOLD;
        let s = tr("Experience: {:s}").replacen("{:s}", &format_integer(ExpLvlsTbl[(char_level - 1) as usize] as i32), 1);
        crate::control::add_panel_string(ctx, &s);
        crate::control::add_panel_string(ctx, &tr("Maximum Level"));
        return true;
    }
    ctx.control.info_color = UiFlags::COLOR_WHITE;
    let next = ExpLvlsTbl[char_level as usize];
    crate::control::add_panel_string(ctx, &tr("Experience: {:s}").replacen("{:s}", &format_u64(exp as u64), 1));
    crate::control::add_panel_string(ctx, &tr("Next Level: {:s}").replacen("{:s}", &format_u64(next as u64), 1));
    let s = tr("{:s} to Level {:d}").replacen("{:s}", &format_u64((next - exp) as u64), 1).replacen("{:d}", &(char_level as i32 + 1).to_string(), 1);
    crate::control::add_panel_string(ctx, &s);
    true
}

/// `FormatInteger` for values that may exceed `int` (the original's overload takes `int`;
/// experience values stay below 2^31).
fn format_u64(v: u64) -> String {
    format_integer(v as i32)
}
