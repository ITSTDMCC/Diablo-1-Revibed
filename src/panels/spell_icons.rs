//! `Source/panels/spell_icons.cpp`: large and small spell icons.

use crate::ctx::Ctx;
use crate::engine::geometry::{Point, Rectangle, Size};
use crate::engine::load_sprites::load_cel;
use crate::engine::render::clx_render::clx_draw_trn;
use crate::engine::surface::Surface;
use crate::enums::*;

/// `SPLICONLENGTH`
pub const SPLICONLENGTH: i32 = 56;

/// Palette ranges (engine/palette.h).
pub const PAL8_YELLOW: u8 = 144;
pub const PAL16_BEIGE: u8 = 160;
pub const PAL16_BLUE: u8 = 176;
pub const PAL16_YELLOW: u8 = 192;
pub const PAL16_ORANGE: u8 = 208;
pub const PAL16_RED: u8 = 224;
pub const PAL16_GRAY: u8 = 240;

/// `SpellITbl`: maps from SpellID to spelicon.cel frame number.
const SPELL_I_TBL: [u8; 52] = [
    26, 0, 1, 2, 3, 4, 5, 6, 7, 8, 27, 12, 11, 17, 15, 13, 17, 18, 10, 19, 14, 20, 22, 23, 24, 21, 25, 28, 36, 37, 38, 41, 40, 39, 9, 35, 29, 50, 50, 49, 45, 46, 42, 44, 47, 48, 43,
    34, 34, 34, 34, 34,
];

/// Original: `devilution::LoadLargeSpellIcons` (panels/spell_icons.cpp), MPQ build.
// @port panels/spell_icons.cpp|devilution::LoadLargeSpellIcons() sha=4153a2d461eb
pub fn load_large_spell_icons(ctx: &mut Ctx) {
    let path = if !ctx.init.gb_is_hellfire { "ctrlpan\\spelicon" } else { "data\\spelicon" };
    ctx.panels.large_spell_icons = Some(load_cel(ctx, path, SPLICONLENGTH as u16));
    set_spell_trans(ctx, SpellType::Skill);
}

/// Original: `devilution::FreeLargeSpellIcons` (panels/spell_icons.cpp).
// @port panels/spell_icons.cpp|devilution::FreeLargeSpellIcons() sha=507400215cd1
pub fn free_large_spell_icons(ctx: &mut Ctx) {
    ctx.panels.large_spell_icons = None;
}

/// Original: `devilution::LoadSmallSpellIcons` (panels/spell_icons.cpp), MPQ build.
// @port panels/spell_icons.cpp|devilution::LoadSmallSpellIcons() sha=3d46815e1033
pub fn load_small_spell_icons(ctx: &mut Ctx) {
    ctx.panels.small_spell_icons = Some(load_cel(ctx, "data\\spelli2", 37));
}

/// Original: `devilution::FreeSmallSpellIcons` (panels/spell_icons.cpp).
// @port panels/spell_icons.cpp|devilution::FreeSmallSpellIcons() sha=22acd869e925
pub fn free_small_spell_icons(ctx: &mut Ctx) {
    ctx.panels.small_spell_icons = None;
}

/// Original: `devilution::DrawLargeSpellIcon` (panels/spell_icons.cpp).
// @port panels/spell_icons.cpp|devilution::DrawLargeSpellIcon(const Surface &out, Point position, SpellID spell) sha=a2088c42835a
pub fn draw_large_spell_icon(ctx: &Ctx, out: &Surface, position: Point, spell: SpellID) {
    let sprite = ctx.panels.large_spell_icons.as_ref().expect("LargeSpellIcons").get(SPELL_I_TBL[spell as i8 as usize] as usize);
    clx_draw_trn(out, (position.x, position.y), &sprite, &ctx.panels.spl_trans_tbl);
}

/// Original: `devilution::DrawSmallSpellIcon` (panels/spell_icons.cpp).
// @port panels/spell_icons.cpp|devilution::DrawSmallSpellIcon(const Surface &out, Point position, SpellID spell) sha=59fa5a935c30
pub fn draw_small_spell_icon(ctx: &Ctx, out: &Surface, position: Point, spell: SpellID) {
    let sprite = ctx.panels.small_spell_icons.as_ref().expect("SmallSpellIcons").get(SPELL_I_TBL[spell as i8 as usize] as usize);
    clx_draw_trn(out, (position.x, position.y), &sprite, &ctx.panels.spl_trans_tbl);
}

/// Original: `devilution::DrawLargeSpellIconBorder` (panels/spell_icons.cpp).
// @port panels/spell_icons.cpp|devilution::DrawLargeSpellIconBorder(const Surface &out, Point position, uint8_t color) sha=3b3ecc8285f8
pub fn draw_large_spell_icon_border(ctx: &Ctx, out: &Surface, position: Point, color: u8) {
    let first = ctx.panels.large_spell_icons.as_ref().expect("LargeSpellIcons").get(0);
    let (width, height) = (first.width() as i32, first.height() as i32);
    crate::engine::unsafe_draw_border_2px(out, Rectangle::new(Point::new(position.x, position.y - height + 1), Size::new(width, height)), color);
}

/// Original: `devilution::DrawSmallSpellIconBorder` (panels/spell_icons.cpp).
// @port panels/spell_icons.cpp|devilution::DrawSmallSpellIconBorder(const Surface &out, Point position) sha=ae9f3913cf9c
pub fn draw_small_spell_icon_border(ctx: &Ctx, out: &Surface, position: Point) {
    let first = ctx.panels.small_spell_icons.as_ref().expect("SmallSpellIcons").get(0);
    let (width, height) = (first.width() as i32, first.height() as i32);
    let color = ctx.panels.spl_trans_tbl[(PAL8_YELLOW + 2) as usize];
    crate::engine::unsafe_draw_border_2px(out, Rectangle::new(Point::new(position.x, position.y - height + 1), Size::new(width, height)), color);
}

/// Original: `devilution::SetSpellTrans` (panels/spell_icons.cpp).
// @port panels/spell_icons.cpp|devilution::SetSpellTrans(SpellType t) sha=88515d57decb
pub fn set_spell_trans(ctx: &mut Ctx, t: SpellType) {
    let tbl = &mut ctx.panels.spl_trans_tbl;
    if t == SpellType::Skill {
        for i in 0..128 {
            tbl[i] = i as u8;
        }
    }
    for i in 128..256 {
        tbl[i] = i as u8;
    }
    tbl[255] = 0;

    let (y, b, ye, o, g, be) = (PAL8_YELLOW as usize, PAL16_BLUE as usize, PAL16_YELLOW as usize, PAL16_ORANGE as usize, PAL16_GRAY as usize, PAL16_BEIGE as usize);
    match t {
        SpellType::Spell => {
            tbl[y] = PAL16_BLUE + 1;
            tbl[y + 1] = PAL16_BLUE + 3;
            tbl[y + 2] = PAL16_BLUE + 5;
            for i in b..b + 16 {
                tbl[be - b + i] = i as u8;
                tbl[ye - b + i] = i as u8;
                tbl[o - b + i] = i as u8;
            }
        }
        SpellType::Scroll => {
            tbl[y] = PAL16_BEIGE + 1;
            tbl[y + 1] = PAL16_BEIGE + 3;
            tbl[y + 2] = PAL16_BEIGE + 5;
            for i in be..be + 16 {
                tbl[ye - be + i] = i as u8;
                tbl[o - be + i] = i as u8;
            }
        }
        SpellType::Charges => {
            tbl[y] = PAL16_ORANGE + 1;
            tbl[y + 1] = PAL16_ORANGE + 3;
            tbl[y + 2] = PAL16_ORANGE + 5;
            for i in o..o + 16 {
                tbl[be + i - o] = i as u8;
                tbl[ye + i - o] = i as u8;
            }
        }
        SpellType::Invalid => {
            tbl[y] = PAL16_GRAY + 1;
            tbl[y + 1] = PAL16_GRAY + 3;
            tbl[y + 2] = PAL16_GRAY + 5;
            for i in g..g + 15 {
                tbl[be + i - g] = i as u8;
                tbl[ye + i - g] = i as u8;
                tbl[o + i - g] = i as u8;
            }
            tbl[be + 15] = 0;
            tbl[ye + 15] = 0;
            tbl[o + 15] = 0;
        }
        SpellType::Skill => {}
    }
}
