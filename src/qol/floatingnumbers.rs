//! `Source/qol/floatingnumbers.cpp`: floating damage numbers.

use std::collections::VecDeque;

use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::engine::render::dun_render::{TILE_HEIGHT, TILE_WIDTH};
use crate::engine::render::text_render::{draw_string, get_line_width, GameFontTables, UiFlags};
use crate::engine::surface::{Rect, Surface};
use crate::enums::*;
use crate::options::FloatingNumbers;

/// `FloatingNumber`
#[derive(Clone, Debug)]
struct FloatingNumber {
    start_pos: Point,
    start_offset: Displacement,
    end_offset: Displacement,
    text: String,
    time: u32,
    last_merge: u32,
    style: UiFlags,
    type_: DamageType,
    value: i32,
    index: i32,
    reverse_direction: bool,
}

/// Globals of qol/floatingnumbers.cpp.
#[derive(Default)]
pub struct FloatingNumbersState {
    /// `FloatingQueue`
    floating_queue: VecDeque<FloatingNumber>,
}

fn mode(ctx: &Ctx) -> i32 {
    ctx.options.gameplay.enable_floating_numbers.get_raw()
}

/// Original: `ClearExpiredNumbers` (qol/floatingnumbers.cpp).
// @port qol/floatingnumbers.cpp|devilution::ClearExpiredNumbers() sha=e494b47baa79
fn clear_expired_numbers(ctx: &mut Ctx) {
    let now = ctx.platform.ticks();
    while let Some(num) = ctx.floatingnumbers.floating_queue.front() {
        if num.time > now {
            break;
        }
        ctx.floatingnumbers.floating_queue.pop_front();
    }
}

/// Original: `GetGameFontSizeByDamage` (qol/floatingnumbers.cpp).
// @port qol/floatingnumbers.cpp|devilution::GetGameFontSizeByDamage(int value) sha=b852cd536c7e
fn get_game_font_size_by_damage(mut value: i32) -> GameFontTables {
    value >>= 6;
    if value >= 300 {
        return GameFontTables::GameFont30;
    }
    if value >= 100 {
        return GameFontTables::GameFont24;
    }
    GameFontTables::GameFont12
}

/// Original: `GetFontSizeByDamage` (qol/floatingnumbers.cpp).
// @port qol/floatingnumbers.cpp|devilution::GetFontSizeByDamage(int value) sha=f115a424bc8f
fn get_font_size_by_damage(mut value: i32) -> UiFlags {
    value >>= 6;
    if value >= 300 {
        return UiFlags::FONT_SIZE_30;
    }
    if value >= 100 {
        return UiFlags::FONT_SIZE_24;
    }
    UiFlags::FONT_SIZE_12
}

/// Original: `UpdateFloatingData` (qol/floatingnumbers.cpp).
// @port qol/floatingnumbers.cpp|devilution::UpdateFloatingData(FloatingNumber &num) sha=e607347220da
fn update_floating_data(num: &mut FloatingNumber) {
    if num.value > 0 && num.value < 64 {
        num.text = format!("{:.2}", num.value as f64 / 64.0);
    } else {
        num.text = (num.value >> 6).to_string();
    }
    num.style = num.style & !(UiFlags::FONT_SIZE_12 | UiFlags::FONT_SIZE_24 | UiFlags::FONT_SIZE_30);
    num.style = num.style | get_font_size_by_damage(num.value);
    num.style = num.style
        | match num.type_ {
            DamageType::Physical => UiFlags::COLOR_GOLD,
            DamageType::Fire => UiFlags::COLOR_UI_SILVER, // UiSilver appears dark red ingame
            DamageType::Lightning => UiFlags::COLOR_BLUE,
            DamageType::Magic => UiFlags::COLOR_ORANGE,
            DamageType::Acid => UiFlags::COLOR_YELLOW,
        };
}

/// Original: `AddFloatingNumber(Point pos, Displacement offset, DamageType type, int value, int index, bool damageToPlayer)` (qol/floatingnumbers.cpp).
// @port qol/floatingnumbers.cpp|devilution::AddFloatingNumber(Point pos, Displacement offset, DamageType type, int value, int index, bool damageToPlayer) sha=099d9b5ca45c
fn add_floating_number_at(ctx: &mut Ctx, pos: Point, offset: Displacement, type_: DamageType, value: i32, index: i32, damage_to_player: bool) {
    // 45 deg angles to avoid jitter caused by px alignment
    let good_angles = [Displacement::new(0, -140), Displacement::new(100, -100), Displacement::new(-100, -100)];
    let mut end_offset = Displacement::default();
    if mode(ctx) == FloatingNumbers::Random as i32 {
        end_offset = good_angles[(ctx.crt_rand.rand() % 3) as usize];
    } else if mode(ctx) == FloatingNumbers::Vertical as i32 {
        end_offset = good_angles[0];
    }
    if damage_to_player {
        end_offset = Displacement::new(-end_offset.delta_x, -end_offset.delta_y);
    }
    let now = ctx.platform.ticks();
    for num in ctx.floatingnumbers.floating_queue.iter_mut() {
        if num.reverse_direction == damage_to_player && num.type_ == type_ && num.index == index && (now as i32).wrapping_sub(num.last_merge as i32) <= 100 {
            num.value += value;
            num.last_merge = now;
            update_floating_data(num);
            return;
        }
    }
    let mut num = FloatingNumber {
        start_pos: pos,
        start_offset: offset,
        end_offset,
        text: String::new(),
        time: now.wrapping_add(2500),
        last_merge: now,
        style: UiFlags::OUTLINED,
        type_,
        value,
        index,
        reverse_direction: damage_to_player,
    };
    update_floating_data(&mut num);
    ctx.floatingnumbers.floating_queue.push_back(num);
}

/// Original: `devilution::AddFloatingNumber(DamageType damageType, const Monster &monster, int damage)` (qol/floatingnumbers.cpp).
// @port qol/floatingnumbers.cpp|devilution::AddFloatingNumber(DamageType damageType, const Monster &monster, int damage) sha=ef291b3e64da
pub fn add_floating_number_monster(ctx: &mut Ctx, damage_type: DamageType, m: usize, damage: i32) {
    if mode(ctx) == FloatingNumbers::Off as i32 {
        return;
    }
    let mut offset = Displacement::default();
    let monster = &ctx.monster.Monsters[m];
    if crate::monster::is_walking(ctx, m) {
        offset = crate::engine::render::scrollrt::get_offset_for_walking(ctx, &monster.animInfo, monster.direction, false);
        if monster.mode == MonsterMode::MoveSideways {
            if monster.direction == Direction::West {
                offset = offset - Displacement::new(64, 0);
            } else {
                offset = offset + Displacement::new(64, 0);
            }
        }
    }
    if monster.animInfo.sprites.is_some() {
        let sprite = monster.animInfo.current_sprite(ctx.nthread.ProgressToNextGameTick);
        offset.delta_y -= sprite.height() as i32 / 2;
    }
    let tile = monster.position.tile;
    add_floating_number_at(ctx, tile, offset, damage_type, damage, m as i32, false);
}

/// Original: `devilution::AddFloatingNumber(DamageType damageType, const Player &player, int damage)` (qol/floatingnumbers.cpp).
// @port qol/floatingnumbers.cpp|devilution::AddFloatingNumber(DamageType damageType, const Player &player, int damage) sha=5427f916690f
pub fn add_floating_number_player(ctx: &mut Ctx, damage_type: DamageType, pnum: usize, damage: i32) {
    if mode(ctx) == FloatingNumbers::Off as i32 {
        return;
    }
    let mut offset = Displacement::default();
    let player = &ctx.players.Players[pnum];
    if player.is_walking() {
        offset = crate::engine::render::scrollrt::get_offset_for_walking(ctx, &player.AnimInfo, player._pdir, false);
        if player._pmode == PM_WALK_SIDEWAYS {
            if player._pdir == Direction::West {
                offset = offset - Displacement::new(64, 0);
            } else {
                offset = offset + Displacement::new(64, 0);
            }
        }
    }
    let tile = player.position.tile;
    add_floating_number_at(ctx, tile, offset, damage_type, damage, pnum as i32, true);
}

/// Original: `devilution::DrawFloatingNumbers` (qol/floatingnumbers.cpp).
// @port qol/floatingnumbers.cpp|devilution::DrawFloatingNumbers(const Surface &out, Point viewPosition, Displacement offset) sha=75f85021bf5e
pub fn draw_floating_numbers(ctx: &mut Ctx, out: &Surface, view_position: Point, offset: Displacement) {
    if mode(ctx) == FloatingNumbers::Off as i32 {
        return;
    }
    let zoom = ctx.options.graphics.zoom.get();
    let queue: Vec<FloatingNumber> = ctx.floatingnumbers.floating_queue.iter().cloned().collect();
    for floating_num in queue.iter() {
        let mut world_offset = view_position - floating_num.start_pos;
        world_offset = world_offset.world_to_screen() + offset + Displacement::new(TILE_WIDTH / 2, -TILE_HEIGHT / 2) + floating_num.start_offset;
        if zoom {
            world_offset = Displacement::new(world_offset.delta_x * 2, world_offset.delta_y * 2);
        }
        let mut screen_position = Point::new(world_offset.delta_x, world_offset.delta_y);
        let line_width = get_line_width(ctx, &floating_num.text, get_game_font_size_by_damage(floating_num.value), 1, None);
        screen_position.x -= line_width / 2;
        let time_left = floating_num.time.wrapping_sub(ctx.platform.ticks());
        let mul = 1.0f32 - (time_left as f32 / 2500.0f32);
        // `Displacement * float` rounds each component toward zero (static_cast<int>).
        screen_position.x += (floating_num.end_offset.delta_x as f32 * mul) as i32;
        screen_position.y += (floating_num.end_offset.delta_y as f32 * mul) as i32;
        draw_string(ctx, out, &floating_num.text, Rect::new(screen_position.x, screen_position.y, line_width, 0), floating_num.style, 1, -1);
    }
    clear_expired_numbers(ctx);
}

/// Original: `devilution::ClearFloatingNumbers` (qol/floatingnumbers.cpp).
// @port qol/floatingnumbers.cpp|devilution::ClearFloatingNumbers() sha=d81791ce950a
pub fn clear_floating_numbers(ctx: &mut Ctx) {
    let t = ctx.platform.time() as u32;
    ctx.crt_rand.srand(t);
    ctx.floatingnumbers.floating_queue.clear();
}
