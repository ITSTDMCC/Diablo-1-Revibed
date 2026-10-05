//! `Source/qol/monhealthbar.cpp`: the monster health bar.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::geometry::{Displacement, Point};
use crate::engine::render::text_render::{draw_string, render_clx_sprite, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::enums::*;

/// Globals of qol/monhealthbar.cpp.
#[derive(Default)]
pub struct MonHealthBarState {
    health_box: Option<ClxSpriteList>,
    resistance: Option<ClxSpriteList>,
    health: Option<ClxSpriteList>,
    health_blue: Option<ClxSpriteList>,
    player_exp_tags: Option<ClxSpriteList>,
}

/// Original: `devilution::InitMonsterHealthBar` (qol/monhealthbar.cpp). The original leaves
/// all but three entries of the blue TRN uninitialised; they are identity here (known
/// difference: the health sprite only uses those three colours).
// @port qol/monhealthbar.cpp|devilution::InitMonsterHealthBar() sha=e4d95fa1d620
pub fn init_monster_health_bar(ctx: &mut Ctx) {
    if !ctx.options.gameplay.enemy_health_bar.get() {
        return;
    }
    ctx.monhealthbar.health_box = Some(crate::engine::load_sprites::load_clx(ctx, "data\\healthbox.clx"));
    ctx.monhealthbar.health = Some(crate::engine::load_sprites::load_clx(ctx, "data\\health.clx"));
    ctx.monhealthbar.resistance = Some(crate::engine::load_sprites::load_clx(ctx, "data\\resistance.clx"));
    ctx.monhealthbar.player_exp_tags = Some(crate::engine::load_sprites::load_clx(ctx, "data\\monstertags.clx"));
    let mut health_blue_trn = [0u8; 256];
    for (i, v) in health_blue_trn.iter_mut().enumerate() {
        *v = i as u8;
    }
    health_blue_trn[234] = 185;
    health_blue_trn[235] = 186;
    health_blue_trn[236] = 187;
    let mut blue = ctx.monhealthbar.health.as_ref().unwrap().deep_clone();
    crate::engine::render::clx_render::clx_apply_trans_list(&mut blue, &health_blue_trn);
    ctx.monhealthbar.health_blue = Some(blue);
}

/// Original: `devilution::FreeMonsterHealthBar` (qol/monhealthbar.cpp).
// @port qol/monhealthbar.cpp|devilution::FreeMonsterHealthBar() sha=576d2b808389
pub fn free_monster_health_bar(ctx: &mut Ctx) {
    let s = &mut ctx.monhealthbar;
    s.health_blue = None;
    s.player_exp_tags = None;
    s.resistance = None;
    s.health = None;
    s.health_box = None;
}

fn p(pt: Point) -> (i32, i32) {
    (pt.x, pt.y)
}

/// Original: `devilution::DrawMonsterHealthBar` (qol/monhealthbar.cpp).
// @port qol/monhealthbar.cpp|devilution::DrawMonsterHealthBar(const Surface &out) sha=7f371c31803e
pub fn draw_monster_health_bar(ctx: &mut Ctx, out: &Surface) {
    if !ctx.options.gameplay.enemy_health_bar.get() {
        return;
    }
    if ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town {
        return;
    }
    if ctx.cursor.pcursmonst == -1 {
        return;
    }
    let mi = ctx.cursor.pcursmonst as usize;
    let s = &ctx.monhealthbar;
    let box_sprite = s.health_box.as_ref().unwrap().get(0);
    let width = box_sprite.width() as i32;
    let bar_width = s.health.as_ref().unwrap().get(0).width() as i32;
    let height = box_sprite.height() as i32;
    let mut position = Point::new((ctx.dx.gn_screen_width - width) / 2, 18);
    if crate::control::can_panels_cover_view(ctx) {
        if crate::control::is_right_panel_open(ctx) {
            position.x -= crate::control::SIDE_PANEL_SIZE.0 / 2;
        }
        if crate::control::is_left_panel_open(ctx) {
            position.x += crate::control::SIDE_PANEL_SIZE.0 / 2;
        }
    }
    const BORDER: i32 = 3;
    let monster = &ctx.monster.Monsters[mi];
    let mut multiplier = 0;
    let mut curr_life = monster.hitPoints;
    // lifestealing monsters can reach HP exceeding their max
    if monster.hitPoints > monster.maxHitPoints {
        multiplier = monster.hitPoints / monster.maxHitPoints;
        curr_life = monster.hitPoints - monster.maxHitPoints * multiplier;
        if curr_life == 0 && multiplier > 0 {
            multiplier -= 1;
            curr_life = monster.maxHitPoints;
        }
    }
    render_clx_sprite(out, &box_sprite, p(position));
    crate::engine::draw_half_transparent_rect_to(ctx, out, position.x + BORDER, position.y + BORDER, width - (BORDER * 2), height - (BORDER * 2));
    let bar_progress = (bar_width * curr_life) / monster.maxHitPoints;
    if bar_progress != 0 {
        let bar = if multiplier > 0 { s.health_blue.as_ref().unwrap() } else { s.health.as_ref().unwrap() }.get(0);
        render_clx_sprite(&out.subregion(position.x + BORDER + 1, position.y + BORDER + 1, bar_progress, height - (BORDER * 2) - 2), &bar, (0, 0));
    }
    if ctx.options.gameplay.show_monster_type.get() {
        let border_color = match crate::monster::monster_data(ctx, mi).monsterClass {
            MonsterClass::Undead => 248,
            MonsterClass::Demon => 232,
            MonsterClass::Animal => 150,
        };
        let border_width = width - (BORDER * 2);
        crate::engine::unsafe_draw_horizontal_line(out, Point::new(position.x + BORDER, position.y + BORDER), border_width, border_color);
        crate::engine::unsafe_draw_horizontal_line(out, Point::new(position.x + BORDER, position.y + height - BORDER - 1), border_width, border_color);
        let border_height = height - (BORDER * 2) - 2;
        crate::engine::unsafe_draw_vertical_line(out, Point::new(position.x + BORDER, position.y + BORDER + 1), border_height, border_color);
        crate::engine::unsafe_draw_vertical_line(out, Point::new(position.x + width - BORDER - 1, position.y + BORDER + 1), border_height, border_color);
    }
    let name = crate::monster::monster_name(ctx, mi);
    let mut style = UiFlags::ALIGN_CENTER | UiFlags::VERTICAL_CENTER;
    draw_string(ctx, out, &name, Rect::new(position.x - 1, position.y + 1, width, height), style | UiFlags::COLOR_BLACK, 1, -1);
    let monster = &ctx.monster.Monsters[mi];
    if monster.is_unique() {
        style = style | UiFlags::COLOR_WHITEGOLD;
    } else if monster.leader != crate::monster::Monster::NoLeader {
        style = style | UiFlags::COLOR_BLUE;
    } else {
        style = style | UiFlags::COLOR_WHITE;
    }
    draw_string(ctx, out, &name, Rect::new(position.x, position.y, width, height), style, 1, -1);
    if multiplier > 0 {
        draw_string(ctx, out, &format!("x{multiplier}"), Rect::new(position.x, position.y, width - 2, height), UiFlags::COLOR_WHITE | UiFlags::ALIGN_RIGHT | UiFlags::VERTICAL_CENTER, 1, -1);
    }
    let monster = &ctx.monster.Monsters[mi];
    let mtype = crate::monster::monster_type_id(ctx, mi);
    let s = &ctx.monhealthbar;
    if monster.is_unique() || ctx.monster.MonsterKillCounts[mtype as usize] >= 15 {
        let immunes = [IMMUNE_MAGIC, IMMUNE_FIRE, IMMUNE_LIGHTNING];
        let resists = [RESIST_MAGIC, RESIST_FIRE, RESIST_LIGHTNING];
        let res = s.resistance.as_ref().unwrap();
        let mut res_offset = 5;
        for i in 0..3 {
            if (monster.resistance & immunes[i] as u16) != 0 {
                render_clx_sprite(out, &res.get(i * 2 + 1), p(position + Displacement::new(res_offset, height - 6)));
                res_offset += res.get(0).width() as i32 + 2;
            } else if (monster.resistance & resists[i] as u16) != 0 {
                render_clx_sprite(out, &res.get(i * 2), p(position + Displacement::new(res_offset, height - 6)));
                res_offset += res.get(0).width() as i32 + 2;
            }
        }
    }
    if ctx.players.Players.len() > 1 {
        let tags = s.player_exp_tags.as_ref().unwrap();
        let mut tag_offset = 5;
        for i in 0..ctx.players.Players.len() {
            if ((1u32 << i) & monster.whoHit as u8 as u32) != 0 {
                render_clx_sprite(out, &tags.get(i + 1), p(position + Displacement::new(tag_offset, height - 31)));
            } else if ctx.players.Players[i].plractive {
                render_clx_sprite(out, &tags.get(0), p(position + Displacement::new(tag_offset, height - 31)));
            }
            tag_offset += tags.get(0).width() as i32;
        }
    }
}
