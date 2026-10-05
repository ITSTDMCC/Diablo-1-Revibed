//! `Source/qol/itemlabels.cpp`: item name labels on the ground.

use crate::ctx::Ctx;
use crate::engine::geometry::Point;
use crate::engine::render::text_render::{draw_string, get_line_width, GameFontTables};
use crate::engine::surface::{Rect, Surface};
use crate::enums::*;
use crate::utils::language::tr;

/// `ItemLabel`
#[derive(Clone, Debug)]
struct ItemLabel {
    id: i32,
    width: i32,
    pos: Point,
    text: String,
}

/// Globals of qol/itemlabels.cpp.
pub struct ItemLabelsState {
    label_queue: Vec<ItemLabel>,
    highlight_key_pressed: bool,
    is_label_highlighted: bool,
    label_center_offsets: [Option<i32>; crate::items::ITEMTYPES],
}

impl Default for ItemLabelsState {
    fn default() -> Self {
        ItemLabelsState { label_queue: Vec::new(), highlight_key_pressed: false, is_label_highlighted: false, label_center_offsets: [None; crate::items::ITEMTYPES] }
    }
}

/// minimal horizontal space between labels
const BORDER_X: i32 = 4;
/// minimal vertical space between labels
const BORDER_Y: i32 = 2;
/// horizontal margins between text and edges of the label
const MARGIN_X: i32 = 2;

/// `PAL8_BLUE` (engine/palette.h)
const PAL8_BLUE: u8 = 128;

/// Original: `TextMarginTop` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::TextMarginTop() sha=1cc6c09182f8
fn text_margin_top(ctx: &Ctx) -> i32 {
    if crate::utils::language::is_small_font_tall(ctx) {
        1
    } else {
        -1
    }
}

/// Original: `TextMarginBottom` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::TextMarginBottom() sha=3c9b78cb900d
fn text_margin_bottom(ctx: &Ctx) -> i32 {
    if crate::utils::language::is_small_font_tall(ctx) {
        1
    } else {
        3
    }
}

/// Original: `LabelHeight` (qol/itemlabels.cpp): the total height of the label box.
// @port qol/itemlabels.cpp|devilution::LabelHeight() sha=81d6f13077fe
fn label_height(ctx: &Ctx) -> i32 {
    (if crate::utils::language::is_small_font_tall(ctx) { 16 } else { 11 }) + text_margin_bottom(ctx) + text_margin_top(ctx)
}

/// Original: `devilution::ToggleItemLabelHighlight` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::ToggleItemLabelHighlight() sha=782875ad3dfd
pub fn toggle_item_label_highlight(ctx: &mut Ctx) {
    let v = !ctx.options.gameplay.show_item_labels.get();
    if let Some(cb) = ctx.options.gameplay.show_item_labels.set_value(v) {
        crate::options::run_option_callback(ctx, cb);
    }
}

/// Original: `devilution::HighlightKeyPressed` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::HighlightKeyPressed(bool pressed) sha=a7e6c407f4fc
pub fn highlight_key_pressed(ctx: &mut Ctx, pressed: bool) {
    ctx.itemlabels.highlight_key_pressed = pressed;
}

/// Original: `devilution::IsItemLabelHighlighted` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::IsItemLabelHighlighted() sha=4b97f09e57b1
pub fn is_item_label_highlighted(ctx: &Ctx) -> bool {
    ctx.itemlabels.is_label_highlighted
}

/// Original: `devilution::ResetItemlabelHighlighted` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::ResetItemlabelHighlighted() sha=0079c02929b7
pub fn reset_itemlabel_highlighted(ctx: &mut Ctx) {
    ctx.itemlabels.is_label_highlighted = false;
}

/// Original: `devilution::IsHighlightingLabelsEnabled` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::IsHighlightingLabelsEnabled() sha=082731cb44c6
pub fn is_highlighting_labels_enabled(ctx: &Ctx) -> bool {
    crate::stores::stextflag_is_none(ctx) && ctx.itemlabels.highlight_key_pressed != ctx.options.gameplay.show_item_labels.get()
}

/// Original: `devilution::AddItemToLabelQueue` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::AddItemToLabelQueue(int id, Point position) sha=948f623e594e
pub fn add_item_to_label_queue(ctx: &mut Ctx, id: i32, mut position: Point) {
    if !is_highlighting_labels_enabled(ctx) {
        return;
    }
    let item = ctx.items.Items[id as usize].clone();
    let text_on_ground = if item._itype == ItemType::Gold {
        tr("{:s} gold").replacen("{:s}", &crate::utils::format_int::format_integer(item._ivalue), 1)
    } else {
        item.get_name(ctx)
    };
    let mut name_width = get_line_width(ctx, &text_on_ground, GameFontTables::GameFont12, 1, None);
    name_width += MARGIN_X * 2;
    let index = crate::tables::items_tables::ItemCAnimTbl[item._iCurs as usize] as usize;
    if ctx.itemlabels.label_center_offsets[index].is_none() {
        let sprite = item.AnimInfo.sprites.as_ref().expect("item sprites").get(item.AnimInfo.currentFrame as usize);
        let item_bounds = crate::engine::render::clx_render::clx_measure_solid_horizontal_bounds(&sprite);
        ctx.itemlabels.label_center_offsets[index] = Some((item_bounds.0 + item_bounds.1) / 2);
    }
    position.x += ctx.itemlabels.label_center_offsets[index].unwrap();
    position.y -= crate::engine::render::dun_render::TILE_HEIGHT;
    if ctx.options.graphics.zoom.get() {
        position = Point::new(position.x * 2, position.y * 2);
    }
    position.x -= name_width / 2;
    position.y -= label_height(ctx);
    ctx.itemlabels.label_queue.push(ItemLabel { id, width: name_width, pos: position, text: text_on_ground });
}

/// Original: `devilution::IsMouseOverGameArea` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::IsMouseOverGameArea() sha=8b78fd095f40
fn is_mouse_over_game_area(ctx: &Ctx) -> bool {
    let mouse = Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1);
    if crate::control::is_right_panel_open(ctx) && crate::control::get_right_panel(ctx).contains(mouse) {
        return false;
    }
    if crate::control::is_left_panel_open(ctx) && crate::control::get_left_panel(ctx).contains(mouse) {
        return false;
    }
    if crate::control::get_main_panel(ctx).contains(mouse) {
        return false;
    }
    true
}

/// Original: `devilution::FillRect` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::FillRect(const Surface &out, int x, int y, int width, int height, Uint8 col) sha=056ca3375eb0
fn fill_rect(out: &Surface, x: i32, y: i32, width: i32, height: i32, col: u8) {
    for j in 0..height {
        crate::engine::draw_horizontal_line(out, Point::new(x, y + j), width, col);
    }
}

/// Original: `devilution::DrawItemNameLabels` (qol/itemlabels.cpp).
// @port qol/itemlabels.cpp|devilution::DrawItemNameLabels(const Surface &out) sha=e6ec38fe3a38
pub fn draw_item_name_labels(ctx: &mut Ctx, out: &Surface) {
    let clipped_out = out.subregion_y(0, ctx.dx.gn_viewport_height);
    ctx.itemlabels.is_label_highlighted = false;
    if ctx.itemlabels.label_queue.is_empty() {
        return;
    }
    let mut used_x: Vec<i32> = Vec::new();
    let label_height = label_height(ctx);
    let label_margin_top = text_margin_top(ctx);
    let mut queue = std::mem::take(&mut ctx.itemlabels.label_queue);
    for i in 0..queue.len() {
        used_x.clear();
        loop {
            let mut can_show = true;
            for j in 0..i {
                let (a_pos, a_width) = (queue[i].pos, queue[i].width);
                let b = &queue[j];
                if (b.pos.y - a_pos.y).abs() < label_height + BORDER_Y {
                    let width_a = a_width + BORDER_X + MARGIN_X * 2;
                    let width_b = b.width + BORDER_X + MARGIN_X * 2;
                    let mut newpos = b.pos.x;
                    if b.pos.x >= a_pos.x && b.pos.x - a_pos.x < width_a {
                        newpos -= width_a;
                        if used_x.contains(&newpos) {
                            newpos = b.pos.x + width_b;
                        }
                    } else if b.pos.x < a_pos.x && a_pos.x - b.pos.x < width_b {
                        newpos += width_b;
                        if used_x.contains(&newpos) {
                            newpos = b.pos.x - width_a;
                        }
                    } else {
                        continue;
                    }
                    can_show = false;
                    queue[i].pos.x = newpos;
                    if !used_x.contains(&newpos) {
                        used_x.push(newpos);
                    }
                }
            }
            if can_show {
                break;
            }
        }
    }
    let (mx, my) = ctx.diablo.mouse_position;
    for label in queue.iter() {
        let item_position = ctx.items.Items[label.id as usize].position;
        if mx >= label.pos.x && mx < label.pos.x + label.width && my >= label.pos.y && my < label.pos.y + label_height {
            if !crate::gmenu::gmenu_is_active(ctx)
                && ctx.diablo.pause_mode == 0
                && !ctx.players.MyPlayerIsDead
                && crate::stores::stextflag_is_none(ctx)
                && is_mouse_over_game_area(ctx)
                && ctx.diablo.last_mouse_button_action == crate::diablo::MouseActionType::None
            {
                ctx.itemlabels.is_label_highlighted = true;
                ctx.cursor.cursPosition = item_position;
                ctx.cursor.pcursitem = label.id as i8;
            }
        }
        if ctx.cursor.pcursitem as i32 == label.id && crate::stores::stextflag_is_none(ctx) {
            fill_rect(&clipped_out, label.pos.x, label.pos.y, label.width, label_height, PAL8_BLUE + 6);
        } else {
            crate::engine::draw_half_transparent_rect_to(ctx, &clipped_out, label.pos.x, label.pos.y, label.width, label_height);
        }
        let color = ctx.items.Items[label.id as usize].get_text_color();
        draw_string(ctx, &clipped_out, &label.text, Rect::new(label.pos.x + MARGIN_X, label.pos.y + label_margin_top, label.width, label_height), color, 1, -1);
    }
    queue.clear();
    ctx.itemlabels.label_queue = queue;
}
