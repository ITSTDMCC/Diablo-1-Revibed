//! `Source/controls/modifier_hints.cpp`: gamepad circle menu hints.

use crate::control::get_main_panel;
use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::geometry::{Displacement, Point};
use crate::engine::load_sprites::load_clx;
use crate::engine::render::text_render::render_clx_sprite;
use crate::engine::surface::Surface;
use crate::enums::*;

/// Globals of controls/modifier_hints.cpp.
#[derive(Default)]
pub struct ModifierHintsState {
    hint_box: Option<ClxSpriteList>,
    hint_box_background: Option<ClxSpriteList>,
    hint_icons: Option<ClxSpriteList>,
}

/// Vertical distance between text lines.
const LINE_HEIGHT: i32 = 25;
/// Horizontal margin of the hints circle from panel edge.
const CIRCLE_MARGIN_X: i32 = 16;
/// Distance between the panel top and the circle top.
const CIRCLE_TOP: i32 = 101;
/// Spell icon side size.
const ICON_SIZE: i32 = 37;
const HINT_BOX_SIZE: i32 = 39;
const HINT_BOX_MARGIN: i32 = 5;

/// `HintIcon`
const ICON_CHAR: u8 = 0;
const ICON_INV: u8 = 1;
const ICON_QUESTS: u8 = 2;
const ICON_SPELLS: u8 = 3;
const ICON_MAP: u8 = 4;
const ICON_MENU: u8 = 5;
const ICON_NULL: u8 = 6;

/// `CircleMenuHint`
struct CircleMenuHint {
    top: u8,
    right: u8,
    bottom: u8,
    left: u8,
}

fn hint_box_positions(origin: Point) -> [Point; 4] {
    [
        origin + Displacement::new(0, LINE_HEIGHT - HINT_BOX_SIZE),
        origin + Displacement::new(HINT_BOX_SIZE + HINT_BOX_MARGIN, LINE_HEIGHT - HINT_BOX_SIZE * 2 - HINT_BOX_MARGIN),
        origin + Displacement::new(HINT_BOX_SIZE + HINT_BOX_MARGIN, LINE_HEIGHT + HINT_BOX_MARGIN),
        origin + Displacement::new(HINT_BOX_SIZE * 2 + HINT_BOX_MARGIN * 2, LINE_HEIGHT - HINT_BOX_SIZE),
    ]
}

/// Original: `DrawCircleMenuHint` (controls/modifier_hints.cpp).
// @port controls/modifier_hints.cpp|devilution::DrawCircleMenuHint(const Surface &out, const CircleMenuHint &hint, const Point &origin) sha=beb3cbfa0d66
fn draw_circle_menu_hint(ctx: &Ctx, out: &Surface, hint: &CircleMenuHint, origin: Point) {
    let background_displacement = Displacement::new((HINT_BOX_SIZE - ICON_SIZE) / 2 + 1, (HINT_BOX_SIZE - ICON_SIZE) / 2 - 1);
    let boxes = hint_box_positions(origin);
    let icon_indices = [hint.left, hint.top, hint.bottom, hint.right];
    let s = &ctx.modifier_hints;
    for slot in 0..4 {
        if icon_indices[slot] == ICON_NULL {
            continue;
        }
        let icon_position = boxes[slot] + background_displacement;
        render_clx_sprite(out, &s.hint_box_background.as_ref().unwrap().get(0), (icon_position.x, icon_position.y));
        render_clx_sprite(&out.subregion(icon_position.x, icon_position.y, 37, 38), &s.hint_icons.as_ref().unwrap().get(icon_indices[slot] as usize), (0, 0));
        render_clx_sprite(out, &s.hint_box.as_ref().unwrap().get(0), (boxes[slot].x, boxes[slot].y));
    }
}

/// Original: `DrawSpellsCircleMenuHint` (controls/modifier_hints.cpp).
// @port controls/modifier_hints.cpp|devilution::DrawSpellsCircleMenuHint(const Surface &out, const Point &origin) sha=fd4b595ee96a
fn draw_spells_circle_menu_hint(ctx: &mut Ctx, out: &Surface, origin: Point) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let spell_icon_displacement = Displacement::new((HINT_BOX_SIZE - ICON_SIZE) / 2 + 1, HINT_BOX_SIZE - (HINT_BOX_SIZE - ICON_SIZE) / 2 - 1);
    let boxes = hint_box_positions(origin);
    let p = &ctx.players.Players[me];
    let spells = p._pAblSpells | p._pMemSpells | p._pScrlSpells | p._pISpells;
    for slot in 0..4 {
        let p = &ctx.players.Players[me];
        let mut spl_id = p._pSplHotKey[slot];
        let spl_type;
        if crate::spells::is_valid_spell(ctx, spl_id) && (spells & crate::spells::get_spell_bitmask(spl_id)) != 0 {
            spl_type = if ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town && !crate::items::get_spell_data(spl_id).is_allowed_in_town() {
                SpellType::Invalid
            } else {
                p._pSplTHotKey[slot]
            };
        } else {
            spl_type = SpellType::Invalid;
            spl_id = SpellID::Null;
        }
        crate::panels::spell_icons::set_spell_trans(ctx, spl_type);
        crate::panels::spell_icons::draw_small_spell_icon(ctx, out, boxes[slot] + spell_icon_displacement, spl_id);
        render_clx_sprite(out, &ctx.modifier_hints.hint_box.as_ref().unwrap().get(0), (boxes[slot].x, boxes[slot].y));
    }
}

/// Original: `DrawGamepadMenuNavigator` (controls/modifier_hints.cpp).
// @port controls/modifier_hints.cpp|devilution::DrawGamepadMenuNavigator(const Surface &out) sha=bbdd2a2d8a69
fn draw_gamepad_menu_navigator(ctx: &Ctx, out: &Surface) {
    if !ctx.controls.pad_menu_navigator_active || ctx.controls.sticks.simulating_mouse_with_padmapper {
        return;
    }
    const D_PAD: CircleMenuHint = CircleMenuHint { top: ICON_MENU, right: ICON_INV, bottom: ICON_MAP, left: ICON_CHAR };
    const BUTTONS: CircleMenuHint = CircleMenuHint { top: ICON_NULL, right: ICON_NULL, bottom: ICON_SPELLS, left: ICON_QUESTS };
    let main_panel = get_main_panel(ctx);
    draw_circle_menu_hint(ctx, out, &D_PAD, Point::new(main_panel.x + CIRCLE_MARGIN_X, main_panel.y - CIRCLE_TOP));
    draw_circle_menu_hint(ctx, out, &BUTTONS, Point::new(main_panel.x + main_panel.w - HINT_BOX_SIZE * 3 - CIRCLE_MARGIN_X - HINT_BOX_MARGIN * 2, main_panel.y - CIRCLE_TOP));
}

/// Original: `DrawGamepadHotspellMenu` (controls/modifier_hints.cpp).
// @port controls/modifier_hints.cpp|devilution::DrawGamepadHotspellMenu(const Surface &out) sha=599391a49629
fn draw_gamepad_hotspell_menu(ctx: &mut Ctx, out: &Surface) {
    if !ctx.controls.pad_hotspell_menu_active || ctx.controls.sticks.simulating_mouse_with_padmapper {
        return;
    }
    let main_panel = get_main_panel(ctx);
    draw_spells_circle_menu_hint(ctx, out, Point::new(main_panel.x + main_panel.w - HINT_BOX_SIZE * 3 - CIRCLE_MARGIN_X - HINT_BOX_MARGIN * 2, main_panel.y - CIRCLE_TOP));
}

/// Original: `devilution::InitModifierHints` (controls/modifier_hints.cpp).
// @port controls/modifier_hints.cpp|devilution::InitModifierHints() sha=ca153982a7b3
pub fn init_modifier_hints(ctx: &mut Ctx) {
    ctx.modifier_hints.hint_box = Some(load_clx(ctx, "data\\hintbox.clx"));
    ctx.modifier_hints.hint_box_background = Some(load_clx(ctx, "data\\hintboxbackground.clx"));
    ctx.modifier_hints.hint_icons = Some(load_clx(ctx, "data\\hinticons.clx"));
}

/// Original: `devilution::FreeModifierHints` (controls/modifier_hints.cpp).
// @port controls/modifier_hints.cpp|devilution::FreeModifierHints() sha=19435338b7c5
pub fn free_modifier_hints(ctx: &mut Ctx) {
    ctx.modifier_hints.hint_icons = None;
    ctx.modifier_hints.hint_box_background = None;
    ctx.modifier_hints.hint_box = None;
}

/// Original: `devilution::DrawControllerModifierHints` (controls/modifier_hints.cpp).
// @port controls/modifier_hints.cpp|devilution::DrawControllerModifierHints(const Surface &out) sha=17f156b0e536
pub fn draw_controller_modifier_hints(ctx: &mut Ctx, out: &Surface) {
    draw_gamepad_menu_navigator(ctx, out);
    draw_gamepad_hotspell_menu(ctx, out);
}
