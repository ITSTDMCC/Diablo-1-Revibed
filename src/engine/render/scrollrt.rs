//! `Source/engine/render/scrollrt.cpp`: rendering the dungeon view, its sprites and the
//! interface layers on top, and blitting the changed parts of the back buffer.

use crate::ctx::Ctx;
use crate::engine::backbuffer_state::{get_drawn_cursor, is_redraw_component, is_redraw_everything, is_redraw_viewport, redraw_complete, redraw_component_complete, PanelDrawComponent};
use crate::engine::clx_sprite::ClxSprite;
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::engine::render::clx_render::{clx_draw, clx_draw_blended_trn, clx_draw_outline_skip_color_zero, clx_draw_trn};
use crate::engine::render::dun_render::{render_tile, world_draw_black_tile, LevelCelBlock, TILE_HEIGHT, TILE_WIDTH};
use crate::engine::surface::{Rect, Surface};
use crate::enums::*;
use crate::levels::gendung::{in_dungeon_bounds, tile_has_any, DungeonType, MAXDUNX, MAXDUNY};
use crate::lighting::LightsMax;

/// Globals of scrollrt.cpp.
pub struct ScrollrtState {
    /// `LightTableIndex`
    pub LightTableIndex: i32,
    /// `AutoMapShowItems`
    pub AutoMapShowItems: bool,
    /// `MissilesAtRenderingTile` (tile, missile index), in iteration order
    pub(crate) missiles_at_rendering_tile: Vec<(Point, usize)>,
    /// `dRendered`
    d_rendered: Box<[[bool; MAXDUNY]; MAXDUNX]>,
    /// `PrevCursorRect`
    prev_cursor_rect: Rect,
    /// `tileOffset`, `tileShift`, `tileColums`, `tileRows`
    tile_offset: Displacement,
    tile_shift: Displacement,
    tile_colums: i32,
    tile_rows: i32,
    /// `DrawFPS`'s statics
    frames_since_last_update: i32,
    fps_formatted: String,
}

impl Default for ScrollrtState {
    fn default() -> Self {
        ScrollrtState {
            LightTableIndex: 0,
            AutoMapShowItems: false,
            missiles_at_rendering_tile: Vec::new(),
            d_rendered: Box::new([[false; MAXDUNY]; MAXDUNX]),
            prev_cursor_rect: Rect::default(),
            tile_offset: Displacement::default(),
            tile_shift: Displacement::default(),
            tile_colums: 0,
            tile_rows: 0,
            frames_since_last_update: 0,
            fps_formatted: String::new(),
        }
    }
}

fn p(pt: Point) -> (i32, i32) {
    (pt.x, pt.y)
}

fn light_table(ctx: &Ctx) -> &[u8; 256] {
    &ctx.lighting.LightTables[ctx.scrollrt.LightTableIndex as usize]
}

/// `ClxDrawLight` (engine/render/clx_render.hpp)
// @port engine/render/clx_render.hpp|devilution::ClxDrawLight(const Surface &out, Point position, ClxSprite clx) sha=07dd8f3dacd7
pub fn clx_draw_light(ctx: &Ctx, out: &Surface, position: Point, clx: &ClxSprite) {
    if ctx.scrollrt.LightTableIndex != 0 {
        clx_draw_trn(out, p(position), clx, light_table(ctx));
    } else {
        clx_draw(out, p(position), clx);
    }
}

/// `ClxDrawLightBlended` (engine/render/clx_render.hpp)
// @port engine/render/clx_render.hpp|devilution::ClxDrawLightBlended(const Surface &out, Point position, ClxSprite clx) sha=6e43982d3d3d
pub fn clx_draw_light_blended(ctx: &Ctx, out: &Surface, position: Point, clx: &ClxSprite) {
    clx_draw_blended_trn(out, p(position), clx, light_table(ctx), &ctx.dx.pal.palette_transparency_lookup);
}

/// Original: `CouldMissileCollide` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::CouldMissileCollide(Point tile, bool checkPlayerAndMonster) sha=d341617dd974
fn could_missile_collide(ctx: &Ctx, tile: Point, check_player_and_monster: bool) -> bool {
    if !in_dungeon_bounds(tile) {
        return true;
    }
    if check_player_and_monster {
        if ctx.gendung.dMonster[tile.x as usize][tile.y as usize] > 0 {
            return true;
        }
        if ctx.gendung.dPlayer[tile.x as usize][tile.y as usize] > 0 {
            return true;
        }
    }
    crate::missiles::is_missile_blocked_by_tile(ctx, tile)
}

/// Original: `UpdateMissilePositionForRendering` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::UpdateMissilePositionForRendering(Missile &m, int progress) sha=f3c2aa31fd92
fn update_missile_position_for_rendering(m: &mut crate::missiles::Missile, progress: i32) {
    let vx = (m.position.velocity.delta_x as i64 * progress as i64) / crate::engine::animationinfo::AnimationInfo::BASE_VALUE_FRACTION as i64;
    let vy = (m.position.velocity.delta_y as i64 * progress as i64) / crate::engine::animationinfo::AnimationInfo::BASE_VALUE_FRACTION as i64;
    let travelled = m.position.traveled + Displacement::new(vx as i32, vy as i32);
    let pixels_travelled = Displacement::new(travelled.delta_x >> 16, travelled.delta_y >> 16);
    let tile_offset = pixels_travelled.screen_to_missile();
    // calculcate the future missile position
    m.position.tileForRendering = m.position.start + tile_offset;
    m.position.offsetForRendering = pixels_travelled + tile_offset.world_to_screen();
}

/// Original: `UpdateMissileRendererData` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::UpdateMissileRendererData(Missile &m) sha=4fb1fbfaafe4
fn update_missile_renderer_data(ctx: &mut Ctx, mi: usize) {
    let progress_tick = ctx.nthread.ProgressToNextGameTick as i32;
    let mut m = std::mem::take(&mut ctx.missiles.Missiles[mi]);
    m.position.tileForRendering = m.position.tile;
    m.position.offsetForRendering = m.position.offset;
    let missile_movement = crate::missiles::get_missile_data(m._mitype).movementDistribution;
    // don't calculate missile position if they don't move
    if missile_movement == MissileMovementDistribution::Disabled || m.position.velocity == Displacement::default() {
        ctx.missiles.Missiles[mi] = m;
        return;
    }
    let mut progress = progress_tick;
    update_missile_position_for_rendering(&mut m, progress);
    // If we are still at the current tile, this tile was already checked and is a valid tile
    if m.position.tileForRendering == m.position.tile
        // If no collision can happen at the new tile we can advance
        || !could_missile_collide(ctx, m.position.tileForRendering, missile_movement == MissileMovementDistribution::Blockable)
    {
        ctx.missiles.Missiles[mi] = m;
        return;
    }
    // The new tile could be invalid, so don't advance to it.
    // We search the last offset that is in the old (valid) tile.
    while m.position.tile != m.position.tileForRendering {
        progress -= 1;
        if progress <= 0 {
            m.position.tileForRendering = m.position.tile;
            m.position.offsetForRendering = m.position.offset;
            break;
        }
        update_missile_position_for_rendering(&mut m, progress);
    }
    ctx.missiles.Missiles[mi] = m;
}

/// Original: `UpdateMissilesRendererData` (engine/render/scrollrt.cpp). The original keeps an
/// `std::unordered_multimap`; libstdc++ inserts a new element in front of the elements with an
/// equal key, so missiles on one tile are visited newest first, which the lookup reproduces.
// @port engine/render/scrollrt.cpp|devilution::UpdateMissilesRendererData() sha=9a93f30568e6
pub(crate) fn update_missiles_renderer_data(ctx: &mut Ctx) {
    ctx.scrollrt.missiles_at_rendering_tile.clear();
    for mi in 0..ctx.missiles.Missiles.len() {
        update_missile_renderer_data(ctx, mi);
        let t = ctx.missiles.Missiles[mi].position.tileForRendering;
        ctx.scrollrt.missiles_at_rendering_tile.push((t, mi));
    }
}

/// Original: `UndrawCursor` (engine/render/scrollrt.cpp): restores what was behind the cursor.
// @port engine/render/scrollrt.cpp|devilution::UndrawCursor(const Surface &out) sha=4a9304dcaac9
fn undraw_cursor(ctx: &mut Ctx, out: &Surface) {
    let cursor = get_drawn_cursor(ctx);
    let rect = cursor.rect;
    for i in 0..rect.h {
        let dst = out.at(rect.x, rect.y + i);
        // SAFETY: the rectangle was clipped to `out` when the cursor was drawn.
        unsafe { std::ptr::copy_nonoverlapping(cursor.behind_buffer.as_ptr().add((i * rect.w) as usize), dst, rect.w as usize) };
    }
    ctx.scrollrt.prev_cursor_rect = rect;
}

/// Original: `ShouldShowCursor` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::ShouldShowCursor() sha=0bae49b7358f
fn should_show_cursor(ctx: &Ctx) -> bool {
    if ctx.controls.control_mode == crate::controls::ControlTypes::KeyboardAndMouse {
        return true;
    }
    if ctx.cursor.pcurs == crate::cursor::CURSOR_TELEPORT {
        return true;
    }
    if ctx.inv.invflag {
        return true;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.control.chrflag && ctx.players.Players[me]._pStatPts > 0 {
        return true;
    }
    false
}

/// Original: `DrawCursor` (engine/render/scrollrt.cpp): saves what is behind the cursor and
/// draws it.
// @port engine/render/scrollrt.cpp|devilution::DrawCursor(const Surface &out) sha=fcea68df858e
fn draw_cursor(ctx: &mut Ctx, out: &Surface) {
    if crate::hwcursor::is_hardware_cursor(ctx) {
        let show = should_show_cursor(ctx);
        crate::hwcursor::set_hardware_cursor_visible(ctx, show);
        let c = get_drawn_cursor(ctx);
        c.rect.w = 0;
        c.rect.h = 0;
        return;
    }
    let pcurs = ctx.cursor.pcurs;
    if pcurs <= crate::cursor::CURSOR_NONE || !should_show_cursor(ctx) {
        let c = get_drawn_cursor(ctx);
        c.rect.w = 0;
        c.rect.h = 0;
        return;
    }
    let curs_size = crate::cursor::get_inv_item_size(pcurs);
    if curs_size.width == 0 || curs_size.height == 0 {
        let c = get_drawn_cursor(ctx);
        c.rect.w = 0;
        c.rect.h = 0;
        return;
    }
    let clip = |pos: &mut i32, length: &mut i32, pos_end: i32| {
        if *pos + *length <= 0 || *pos >= pos_end {
            *pos = 0;
            *length = 0;
        } else if *pos < 0 {
            *length += *pos;
            *pos = 0;
        } else if *pos + *length > pos_end {
            *length = pos_end - *pos;
        }
    };
    // Copy the buffer before the item cursor and its 1px outline are drawn to a temporary buffer.
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let holding = !ctx.players.Players[me].HoldItem.is_empty();
    let outline_width = if holding { 1 } else { 0 };
    let offset = if holding { Displacement::new(curs_size.width / 2, curs_size.height / 2) } else { Displacement::new(0, 0) };
    let mouse = Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1);
    let curs_position = mouse - offset;
    let mut rect = Rect::new(curs_position.x - outline_width, curs_position.y - outline_width, curs_size.width + 2 * outline_width, curs_size.height + 2 * outline_width);
    clip(&mut rect.x, &mut rect.w, out.w());
    clip(&mut rect.y, &mut rect.h, out.h());
    get_drawn_cursor(ctx).rect = rect;
    if rect.w == 0 || rect.h == 0 {
        return;
    }
    {
        let cursor = get_drawn_cursor(ctx);
        for i in 0..rect.h {
            let src = out.at(rect.x, rect.y + i);
            // SAFETY: `rect` is clipped to `out`; the behind buffer holds 8192 bytes, enough for the
            // largest cursor (as in the original).
            unsafe { std::ptr::copy_nonoverlapping(src, cursor.behind_buffer.as_mut_ptr().add((i * rect.w) as usize), rect.w as usize) };
        }
    }
    crate::cursor::draw_software_cursor(ctx, out, curs_position + Displacement::new(0, curs_size.height - 1), pcurs);
}

/// Original: `DrawMissilePrivate` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawMissilePrivate(const Surface &out, const Missile &missile, Point targetBufferPosition, bool pre) sha=8193696efe2b
fn draw_missile_private(ctx: &Ctx, out: &Surface, mi: usize, target_buffer_position: Point, pre: bool) {
    let missile = &ctx.missiles.Missiles[mi];
    if missile._miPreFlag != pre || !missile._miDrawFlag {
        return;
    }
    let pos = target_buffer_position + missile.position.offsetForRendering - Displacement::new(missile._miAnimWidth2 as i32, 0);
    let sprite = missile._miAnimData.as_ref().expect("missile anim").get((missile._miAnimFrame - 1) as usize);
    if missile._miUniqTrans != 0 {
        let trn = ctx.monster.Monsters[missile._misource as usize].uniqueMonsterTRN.as_ref().expect("unique TRN");
        clx_draw_trn(out, p(pos), &sprite, trn);
    } else if missile._miLightFlag {
        clx_draw_light(ctx, out, pos, &sprite);
    } else {
        clx_draw(out, p(pos), &sprite);
    }
}

/// Original: `DrawMissile` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawMissile(const Surface &out, Point tilePosition, Point targetBufferPosition, bool pre) sha=5d78f9703ae4
fn draw_missile(ctx: &Ctx, out: &Surface, tile_position: Point, target_buffer_position: Point, pre: bool) {
    for &(t, mi) in ctx.scrollrt.missiles_at_rendering_tile.iter().rev() {
        if t == tile_position {
            draw_missile_private(ctx, out, mi, target_buffer_position, pre);
        }
    }
}

/// Original: `DrawMonster` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawMonster(const Surface &out, Point tilePosition, Point targetBufferPosition, const Monster &monster) sha=5015372b01c8
fn draw_monster(ctx: &Ctx, out: &Surface, tile_position: Point, target_buffer_position: Point, m: usize) {
    let monster = &ctx.monster.Monsters[m];
    if monster.animInfo.sprites.is_none() {
        crate::platform::log::info!("Draw Monster \"{}\": NULL Cel Buffer", crate::monster::monster_name(ctx, m));
        return;
    }
    let sprite = monster.animInfo.current_sprite(ctx.nthread.ProgressToNextGameTick);
    if !crate::levels::gendung::is_tile_lit(ctx, tile_position) {
        clx_draw_trn(out, p(target_buffer_position), &sprite, crate::engine::trn::get_infravision_trn(ctx));
        return;
    }
    let mut trn: Option<&[u8; 256]> = None;
    if monster.is_unique() {
        trn = monster.uniqueMonsterTRN.as_deref();
    }
    if monster.mode == MonsterMode::Petrified {
        trn = Some(crate::engine::trn::get_stone_trn(ctx));
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.players.Players[me]._pInfraFlag && ctx.scrollrt.LightTableIndex > 8 {
        trn = Some(crate::engine::trn::get_infravision_trn(ctx));
    }
    match trn {
        Some(t) => clx_draw_trn(out, p(target_buffer_position), &sprite, t),
        None => clx_draw_light(ctx, out, target_buffer_position, &sprite),
    }
}

/// Original: `DrawPlayerIconHelper` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawPlayerIconHelper(const Surface &out, MissileGraphicID missileGraphicId, Point position, bool lighting, bool infraVision) sha=3d567921ddf5
fn draw_player_icon_helper(ctx: &Ctx, out: &Surface, missile_graphic_id: MissileGraphicID, mut position: Point, lighting: bool, infra_vision: bool) {
    let data = crate::missiles::get_missile_sprite_data(ctx, missile_graphic_id);
    position.x -= data.animWidth2 as i32;
    let sprite = data.sprites.as_ref().expect("missile sprites").list().get(0);
    if !lighting {
        clx_draw(out, p(position), &sprite);
        return;
    }
    if infra_vision {
        clx_draw_trn(out, p(position), &sprite, crate::engine::trn::get_infravision_trn(ctx));
        return;
    }
    clx_draw_light(ctx, out, position, &sprite);
}

/// Original: `DrawPlayerIcons` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawPlayerIcons(const Surface &out, const Player &player, Point position, bool infraVision) sha=e6de88cec8c4
fn draw_player_icons(ctx: &Ctx, out: &Surface, pnum: usize, position: Point, infra_vision: bool) {
    let player = &ctx.players.Players[pnum];
    let not_me = ctx.players.MyPlayer != Some(pnum);
    if player.pManaShield {
        draw_player_icon_helper(ctx, out, MissileGraphicID::ManaShield, position, not_me, infra_vision);
    }
    if player.wReflections > 0 {
        draw_player_icon_helper(ctx, out, MissileGraphicID::Reflect, position + Displacement::new(0, 16), not_me, infra_vision);
    }
}

/// Original: `DrawPlayer` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawPlayer(const Surface &out, const Player &player, Point tilePosition, Point targetBufferPosition) sha=a143b94d108f
fn draw_player(ctx: &mut Ctx, out: &Surface, pnum: usize, tile_position: Point, target_buffer_position: Point) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let my_infra = ctx.players.Players[me]._pInfraFlag;
    let my_arena = ctx.players.Players[me].is_on_arena_level();
    let lit = crate::levels::gendung::is_tile_lit(ctx, tile_position);
    if !lit && !my_infra && !my_arena && ctx.gendung.leveltype != DungeonType::Town {
        return;
    }
    let player = &ctx.players.Players[pnum];
    let sprite = match &player.previewCelSprite {
        Some(s) => s.clone(),
        None => player.AnimInfo.current_sprite(ctx.nthread.ProgressToNextGameTick),
    };
    let sprite_buffer_position = target_buffer_position - Displacement::new(crate::engine::calculate_width2(sprite.width() as i32), 0);
    if (ctx.cursor.pcursplr as u8 as usize) < ctx.players.Players.len() && ctx.cursor.pcursplr as u8 as usize == pnum {
        clx_draw_outline_skip_color_zero(out, 165, p(sprite_buffer_position), &sprite);
    }
    if pnum == me && !matches!(ctx.gendung.leveltype, DungeonType::Nest | DungeonType::Crypt) {
        clx_draw(out, p(sprite_buffer_position), &sprite);
        draw_player_icons(ctx, out, pnum, target_buffer_position, false);
        return;
    }
    if !lit || ((my_infra || my_arena) && ctx.scrollrt.LightTableIndex > 8) {
        clx_draw_trn(out, p(sprite_buffer_position), &sprite, crate::engine::trn::get_infravision_trn(ctx));
        draw_player_icons(ctx, out, pnum, target_buffer_position, true);
        return;
    }
    let l = ctx.scrollrt.LightTableIndex;
    if ctx.scrollrt.LightTableIndex < 5 {
        ctx.scrollrt.LightTableIndex = 0;
    } else {
        ctx.scrollrt.LightTableIndex -= 5;
    }
    clx_draw_light(ctx, out, sprite_buffer_position, &sprite);
    draw_player_icons(ctx, out, pnum, target_buffer_position, false);
    ctx.scrollrt.LightTableIndex = l;
}

/// Original: `DrawDeadPlayer` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawDeadPlayer(const Surface &out, Point tilePosition, Point targetBufferPosition) sha=d2e9c8aa92c8
fn draw_dead_player(ctx: &mut Ctx, out: &Surface, tile_position: Point, target_buffer_position: Point) {
    let (x, y) = (tile_position.x as usize, tile_position.y as usize);
    ctx.gendung.dFlags[x][y] &= !DungeonFlag::DeadPlayer;
    for pnum in 0..ctx.players.Players.len() {
        let pl = &ctx.players.Players[pnum];
        if pl.plractive && pl._pHitPoints == 0 && crate::player::is_on_active_level(ctx, pnum) && pl.position.tile == tile_position {
            ctx.gendung.dFlags[x][y] |= DungeonFlag::DeadPlayer;
            draw_player(ctx, out, pnum, tile_position, target_buffer_position);
        }
    }
}

/// Original: `DrawObject` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawObject(const Surface &out, Point tilePosition, Point targetBufferPosition, bool pre) sha=9eee4c3494e5
fn draw_object(ctx: &Ctx, out: &Surface, tile_position: Point, target_buffer_position: Point, pre: bool) {
    if ctx.scrollrt.LightTableIndex >= LightsMax as i32 {
        return;
    }
    let Some(oi) = crate::objects::find_object_at_position(ctx, tile_position, true) else { return };
    let object = &ctx.objects.Objects[oi];
    if object._oPreFlag != pre {
        return;
    }
    let sprite = object._oAnimData.as_ref().expect("object anim").get((object._oAnimFrame - 1) as usize);
    let mut screen_position = target_buffer_position - Displacement::new(crate::engine::calculate_width2(sprite.width() as i32), 0);
    if object.position != tile_position {
        // drawing a large or offset object, calculate the correct position for the center of the sprite
        let world_offset = object.position - tile_position;
        screen_position = screen_position - world_offset.world_to_screen();
    }
    if ctx.cursor.ObjectUnderCursor == Some(oi) {
        clx_draw_outline_skip_color_zero(out, 194, p(screen_position), &sprite);
    }
    if object.applyLighting {
        clx_draw_light(ctx, out, screen_position, &sprite);
    } else {
        clx_draw(out, p(screen_position), &sprite);
    }
}

/// Original: `DrawCell` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawCell(const Surface &out, Point tilePosition, Point targetBufferPosition) sha=df4fd05992be
fn draw_cell(ctx: &Ctx, out: &Surface, tile_position: Point, mut target_buffer_position: Point) {
    let (x, y) = (tile_position.x as usize, tile_position.y as usize);
    let level_piece_id = ctx.gendung.dPiece[x][y];
    let p_map = &ctx.gendung.DPieceMicros[level_piece_id as usize];
    let tbl = ctx.scrollrt.LightTableIndex as usize;
    let transparency = tile_has_any(ctx, level_piece_id as i32, TileProperties::Transparent) && ctx.gendung.TransList[ctx.gendung.dTransVal[x][y] as u8 as usize];
    let foliage = !tile_has_any(ctx, level_piece_id as i32, TileProperties::Solid);
    let has = |prop: TileProperties| tile_has_any(ctx, level_piece_id as i32, prop);

    let get_first_tile_mask_left = |tile: TileType| -> MaskType {
        if transparency {
            return match tile {
                TileType::LeftTrapezoid | TileType::TransparentSquare => {
                    if has(TileProperties::TransparentLeft) {
                        MaskType::Left
                    } else {
                        MaskType::Solid
                    }
                }
                TileType::LeftTriangle => MaskType::Solid,
                _ => MaskType::Transparent,
            };
        }
        if foliage {
            return MaskType::LeftFoliage;
        }
        MaskType::Solid
    };
    let get_first_tile_mask_right = |tile: TileType| -> MaskType {
        if transparency {
            return match tile {
                TileType::RightTrapezoid | TileType::TransparentSquare => {
                    if has(TileProperties::TransparentRight) {
                        MaskType::Right
                    } else {
                        MaskType::Solid
                    }
                }
                TileType::RightTriangle => MaskType::Solid,
                _ => MaskType::Transparent,
            };
        }
        if foliage {
            return MaskType::RightFoliage;
        }
        MaskType::Solid
    };

    // The first micro tile may be rendered with a foliage mask.
    // Only `TransparentSquare` tiles are rendered when `foliage` is true.
    {
        let level_cel_block = LevelCelBlock(p_map.mt[0]);
        if level_cel_block.has_value() {
            let tile_type = level_cel_block.type_();
            let mask_type = get_first_tile_mask_left(tile_type);
            if mask_type != MaskType::LeftFoliage || tile_type == TileType::TransparentSquare {
                render_tile(ctx, out, target_buffer_position, level_cel_block, mask_type, tbl);
            }
        }
    }
    {
        let level_cel_block = LevelCelBlock(p_map.mt[1]);
        if level_cel_block.has_value() {
            let tile_type = level_cel_block.type_();
            let mask_type = get_first_tile_mask_right(tile_type);
            if (transparency || !foliage || tile_type == TileType::TransparentSquare) && (mask_type != MaskType::RightFoliage || tile_type == TileType::TransparentSquare) {
                render_tile(ctx, out, target_buffer_position + Displacement::new(TILE_WIDTH / 2, 0), level_cel_block, mask_type, tbl);
            }
        }
    }
    target_buffer_position.y -= TILE_HEIGHT;

    let n = ctx.gendung.MicroTileLen as usize;
    let mask = if transparency { MaskType::Transparent } else { MaskType::Solid };
    let mut i = 2;
    while i < n {
        let left = LevelCelBlock(p_map.mt[i]);
        if left.has_value() {
            render_tile(ctx, out, target_buffer_position, left, mask, tbl);
        }
        let right = LevelCelBlock(p_map.mt[i + 1]);
        if right.has_value() {
            render_tile(ctx, out, target_buffer_position + Displacement::new(TILE_WIDTH / 2, 0), right, mask, tbl);
        }
        target_buffer_position.y -= TILE_HEIGHT;
        i += 2;
    }
}

/// Original: `DrawFloor(const Surface &out, Point tilePosition, Point targetBufferPosition)` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawFloor(const Surface &out, Point tilePosition, Point targetBufferPosition) sha=6d1d73f7f405
fn draw_floor_tile(ctx: &mut Ctx, out: &Surface, tile_position: Point, target_buffer_position: Point) {
    let (x, y) = (tile_position.x as usize, tile_position.y as usize);
    ctx.scrollrt.LightTableIndex = ctx.gendung.dLight[x][y] as i32;
    let tbl = ctx.scrollrt.LightTableIndex as usize;
    let level_piece_id = ctx.gendung.dPiece[x][y] as usize;
    let micros = &ctx.gendung.DPieceMicros[level_piece_id];
    let left = LevelCelBlock(micros.mt[0]);
    if left.has_value() {
        render_tile(ctx, out, target_buffer_position, left, MaskType::Solid, tbl);
    }
    let right = LevelCelBlock(micros.mt[1]);
    if right.has_value() {
        render_tile(ctx, out, target_buffer_position + Displacement::new(TILE_WIDTH / 2, 0), right, MaskType::Solid, tbl);
    }
}

/// Original: `DrawItem` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawItem(const Surface &out, Point tilePosition, Point targetBufferPosition, bool pre) sha=fd104168dc80
fn draw_item(ctx: &mut Ctx, out: &Surface, tile_position: Point, target_buffer_position: Point, pre: bool) {
    let b_item = ctx.items.dItem[tile_position.x as usize][tile_position.y as usize];
    if b_item <= 0 {
        return;
    }
    let ii = (b_item - 1) as usize;
    let item = &ctx.items.Items[ii];
    if item._iPostDraw == pre {
        return;
    }
    let sprite = item.AnimInfo.current_sprite(ctx.nthread.ProgressToNextGameTick);
    let px = target_buffer_position.x - crate::engine::calculate_width2(sprite.width() as i32);
    let position = Point::new(px, target_buffer_position.y);
    if crate::stores::stextflag_is_none(ctx) && (ii as i8 == ctx.cursor.pcursitem || ctx.scrollrt.AutoMapShowItems) {
        clx_draw_outline_skip_color_zero(out, crate::items::get_outline_color(item, false), p(position), &sprite);
    }
    clx_draw_light(ctx, out, position, &sprite);
    let item = &ctx.items.Items[ii];
    if item.AnimInfo.is_last_frame() || item._iCurs as item_cursor_graphic == ICURS_MAGIC_ROCK {
        crate::qol::itemlabels::add_item_to_label_queue(ctx, ii as i32, position);
    }
}

/// Original: `DrawMonsterHelper` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawMonsterHelper(const Surface &out, Point tilePosition, Point targetBufferPosition) sha=b1311353cfc9
fn draw_monster_helper(ctx: &Ctx, out: &Surface, tile_position: Point, target_buffer_position: Point) {
    let mut mi = ctx.gendung.dMonster[tile_position.x as usize][tile_position.y as usize] as i32;
    let is_negative_monster = mi < 0;
    mi = mi.abs() - 1;
    if ctx.gendung.leveltype == DungeonType::Town {
        if is_negative_monster {
            return;
        }
        let towner = &ctx.towners.towners[mi as usize];
        let px = target_buffer_position.x - crate::engine::calculate_width2(towner._tAnimWidth as i32);
        let position = Point::new(px, target_buffer_position.y);
        let sprite = towner.current_sprite();
        if mi == ctx.cursor.pcursmonst {
            clx_draw_outline_skip_color_zero(out, 166, p(position), &sprite);
        }
        clx_draw(out, p(position), &sprite);
        return;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if !crate::levels::gendung::is_tile_lit(ctx, tile_position) && !ctx.players.Players[me]._pInfraFlag {
        return;
    }
    if mi as usize >= crate::monster::MaxMonsters {
        crate::platform::log::info!("Draw Monster: tried to draw illegal monster {}", mi);
        return;
    }
    let monster = &ctx.monster.Monsters[mi as usize];
    if (monster.flags & MFLAG_HIDDEN as u32) != 0 {
        return;
    }
    let sprite = monster.animInfo.current_sprite(ctx.nthread.ProgressToNextGameTick);
    let mut offset = Displacement::default();
    if crate::monster::is_walking(ctx, mi as usize) {
        let is_side_walking_to_left = monster.mode == MonsterMode::MoveSideways && monster.direction == Direction::West;
        if is_negative_monster && !is_side_walking_to_left {
            return;
        }
        if !is_negative_monster && is_side_walking_to_left {
            return;
        }
        offset = get_offset_for_walking(ctx, &monster.animInfo, monster.direction, false);
        if is_side_walking_to_left {
            offset = offset - Displacement::new(64, 0);
        }
    } else if is_negative_monster {
        return;
    }
    let monster_render_position = target_buffer_position + offset - Displacement::new(crate::engine::calculate_width2(sprite.width() as i32), 0);
    if mi == ctx.cursor.pcursmonst {
        clx_draw_outline_skip_color_zero(out, 233, p(monster_render_position), &sprite);
    }
    draw_monster(ctx, out, tile_position, monster_render_position, mi as usize);
}

/// Original: `DrawPlayerHelper` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawPlayerHelper(const Surface &out, const Player &player, Point tilePosition, Point targetBufferPosition) sha=ea62b0715414
fn draw_player_helper(ctx: &mut Ctx, out: &Surface, pnum: usize, tile_position: Point, target_buffer_position: Point) {
    let player = &ctx.players.Players[pnum];
    let mut offset = Displacement::default();
    if player.is_walking() {
        offset = get_offset_for_walking(ctx, &player.AnimInfo, player._pdir, false);
    }
    draw_player(ctx, out, pnum, tile_position, target_buffer_position + offset);
}

/// Original: `DrawDungeon` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawDungeon(const Surface &out, Point tilePosition, Point targetBufferPosition) sha=4ef98f53aa1f
fn draw_dungeon(ctx: &mut Ctx, out: &Surface, tile_position: Point, target_buffer_position: Point) {
    debug_assert!(in_dungeon_bounds(tile_position));
    let (x, y) = (tile_position.x as usize, tile_position.y as usize);
    if ctx.scrollrt.d_rendered[x][y] {
        return;
    }
    ctx.scrollrt.d_rendered[x][y] = true;
    ctx.scrollrt.LightTableIndex = ctx.gendung.dLight[x][y] as i32;
    draw_cell(ctx, out, tile_position, target_buffer_position);
    let b_dead = ctx.gendung.dCorpse[x][y];
    let b_map = ctx.gendung.dTransVal[x][y];
    if ctx.missiles.MissilePreFlag {
        draw_missile(ctx, out, tile_position, target_buffer_position, true);
    }
    if ctx.scrollrt.LightTableIndex < LightsMax as i32 && b_dead != 0 {
        let corpse = &ctx.dead.Corpses[((b_dead & 0x1F) - 1) as usize];
        let position = Point::new(target_buffer_position.x - crate::engine::calculate_width2(corpse.width as i32), target_buffer_position.y);
        let sprite = corpse.sprites_for_direction(Direction::from_u8(((b_dead >> 5) & 7) as u8)).get(corpse.frame as usize);
        if corpse.translationPaletteIndex != 0 {
            let trn = ctx.monster.Monsters[(corpse.translationPaletteIndex - 1) as usize].uniqueMonsterTRN.as_ref().expect("unique TRN");
            clx_draw_trn(out, p(position), &sprite, trn);
        } else {
            clx_draw_light(ctx, out, position, &sprite);
        }
    }
    draw_object(ctx, out, tile_position, target_buffer_position, true);
    draw_item(ctx, out, tile_position, target_buffer_position, true);
    if crate::levels::gendung::tile_contains_dead_player(ctx, tile_position) {
        draw_dead_player(ctx, out, tile_position, target_buffer_position);
    }
    let player_id = ctx.gendung.dPlayer[x][y];
    // free movement: the player is drawn in the pass of the tile it stands in front of
    let free = ctx.players.MyPlayer.and_then(|me| crate::freemove::draw_position(ctx, me).map(|(t, o)| (me, t, o)));
    let skip = free.is_some_and(|(me, _, _)| player_id as i32 - 1 == me as i32);
    if ((player_id as i32 - 1) as usize) < ctx.players.Players.len() && !skip {
        draw_player_helper(ctx, out, (player_id - 1) as usize, tile_position, target_buffer_position);
    }
    if let Some((me, t, o)) = free {
        if t == tile_position {
            // visibility is the player's own tile's
            let own = ctx.players.Players[me].position.tile;
            draw_player(ctx, out, me, own, target_buffer_position + o);
        }
    }
    if ctx.gendung.dMonster[x][y] != 0 {
        draw_monster_helper(ctx, out, tile_position, target_buffer_position);
    }
    draw_missile(ctx, out, tile_position, target_buffer_position, false);
    draw_object(ctx, out, tile_position, target_buffer_position, false);
    draw_item(ctx, out, tile_position, target_buffer_position, false);

    if ctx.gendung.leveltype != DungeonType::Town {
        let b_arch = ctx.gendung.dSpecial[x][y];
        if b_arch != 0 {
            let transparency = ctx.gendung.TransList[b_map as u8 as usize];
            let sprite = ctx.gendung.pSpecialCels.as_ref().expect("pSpecialCels").get((b_arch - 1) as usize);
            if transparency {
                clx_draw_light_blended(ctx, out, target_buffer_position, &sprite);
            } else {
                clx_draw_light(ctx, out, target_buffer_position, &sprite);
            }
        }
    } else {
        // Tree leaves should always cover player when entering or leaving the tile,
        // So delay the rendering until after the next row is being drawn.
        // This could probably have been better solved by sprites in screen space.
        if tile_position.x > 0 && tile_position.y > 0 && target_buffer_position.y > TILE_HEIGHT {
            let b_arch = ctx.gendung.dSpecial[x - 1][y - 1];
            if b_arch != 0 {
                let sprite = ctx.gendung.pSpecialCels.as_ref().expect("pSpecialCels").get((b_arch - 1) as usize);
                clx_draw(out, p(target_buffer_position + Displacement::new(0, -TILE_HEIGHT)), &sprite);
            }
        }
    }
}

/// Original: `DrawFloor(const Surface &out, Point tilePosition, Point targetBufferPosition, int rows, int columns)` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawFloor(const Surface &out, Point tilePosition, Point targetBufferPosition, int rows, int columns) sha=26cde23dce4a
fn draw_floor(ctx: &mut Ctx, out: &Surface, mut tile_position: Point, mut target_buffer_position: Point, rows: i32, mut columns: i32) {
    for i in 0..rows {
        for _ in 0..columns {
            if in_dungeon_bounds(tile_position) {
                let piece = ctx.gendung.dPiece[tile_position.x as usize][tile_position.y as usize];
                if !tile_has_any(ctx, piece as i32, TileProperties::Solid) {
                    draw_floor_tile(ctx, out, tile_position, target_buffer_position);
                }
            } else {
                world_draw_black_tile(out, target_buffer_position.x, target_buffer_position.y);
            }
            tile_position = tile_position + Direction::East;
            target_buffer_position.x += TILE_WIDTH;
        }
        // Return to start of row
        tile_position = tile_position + Displacement::from_direction(Direction::West) * columns;
        target_buffer_position.x -= columns * TILE_WIDTH;
        // Jump to next row
        target_buffer_position.y += TILE_HEIGHT / 2;
        if (i & 1) != 0 {
            tile_position.x += 1;
            columns -= 1;
            target_buffer_position.x += TILE_WIDTH / 2;
        } else {
            tile_position.y += 1;
            columns += 1;
            target_buffer_position.x -= TILE_WIDTH / 2;
        }
    }
}

/// Original: `IsWall` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::IsWall(Point position) sha=6536e70f32dc
fn is_wall(ctx: &Ctx, position: Point) -> bool {
    let (x, y) = (position.x as usize, position.y as usize);
    tile_has_any(ctx, ctx.gendung.dPiece[x][y] as i32, TileProperties::Solid) || ctx.gendung.dSpecial[x][y] != 0
}

/// Original: `DrawTileContent` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawTileContent(const Surface &out, Point tilePosition, Point targetBufferPosition, int rows, int columns) sha=f832252d8620
fn draw_tile_content(ctx: &mut Ctx, out: &Surface, mut tile_position: Point, mut target_buffer_position: Point, mut rows: i32, mut columns: i32) {
    // Keep evaluating until MicroTiles can't affect screen
    rows += ctx.gendung.MicroTileLen as i32;
    for row in ctx.scrollrt.d_rendered.iter_mut() {
        row.fill(false);
    }
    let screen_width = ctx.dx.gn_screen_width;
    for i in 0..rows {
        for _ in 0..columns {
            if in_dungeon_bounds(tile_position) {
                if tile_position.x + 1 < MAXDUNX as i32 && tile_position.y - 1 >= 0 && target_buffer_position.x + TILE_WIDTH <= screen_width {
                    // Render objects behind walls first to prevent sprites, that are moving
                    // between tiles, from poking through the walls as they exceed the tile bounds.
                    if is_wall(ctx, tile_position)
                        && (is_wall(ctx, tile_position + Displacement::new(1, 0)) || (tile_position.x > 0 && is_wall(ctx, tile_position + Displacement::new(-1, 0))))
                        && crate::engine::path::is_tile_not_solid(ctx, tile_position + Displacement::new(1, -1))
                        && crate::engine::path::is_tile_not_solid(ctx, tile_position + Displacement::new(0, -1))
                    {
                        // Has walkable area behind it
                        draw_dungeon(ctx, out, tile_position + Direction::East, Point::new(target_buffer_position.x + TILE_WIDTH, target_buffer_position.y));
                    }
                }
                draw_dungeon(ctx, out, tile_position, target_buffer_position);
            }
            tile_position = tile_position + Direction::East;
            target_buffer_position.x += TILE_WIDTH;
        }
        // Return to start of row
        tile_position = tile_position + Displacement::from_direction(Direction::West) * columns;
        target_buffer_position.x -= columns * TILE_WIDTH;
        // Jump to next row
        target_buffer_position.y += TILE_HEIGHT / 2;
        if (i & 1) != 0 {
            tile_position.x += 1;
            columns -= 1;
            target_buffer_position.x += TILE_WIDTH / 2;
        } else {
            tile_position.y += 1;
            columns += 1;
            target_buffer_position.x -= TILE_WIDTH / 2;
        }
    }
}

/// Original: `Zoom` (engine/render/scrollrt.cpp): scales up the top left part of the buffer 2x.
// @port engine/render/scrollrt.cpp|devilution::Zoom(const Surface &out) sha=9884904db922
fn zoom(ctx: &Ctx, out: &Surface) {
    let mut viewport_width = out.w();
    let mut viewport_offset_x = 0;
    if crate::control::can_panels_cover_view(ctx) {
        if crate::control::is_left_panel_open(ctx) {
            viewport_width -= crate::control::SIDE_PANEL_SIZE.0;
            viewport_offset_x = crate::control::SIDE_PANEL_SIZE.0;
        } else if crate::control::is_right_panel_open(ctx) {
            viewport_width -= crate::control::SIDE_PANEL_SIZE.0;
        }
    }
    // We round to even for the source width and height.
    // If the width / height was odd, we copy just one extra pixel / row later on.
    let src_width = (viewport_width + 1) / 2;
    let doubleable_width = viewport_width / 2;
    let src_height = (out.h() + 1) / 2;
    let doubleable_height = out.h() / 2;
    let pitch = out.pitch() as isize;
    let mut src = out.at(src_width - 1, src_height - 1);
    let mut dst = out.at(viewport_offset_x + viewport_width - 1, out.h() - 1);
    let odd_viewport_width = (viewport_width % 2) == 1;
    // SAFETY: as in the original, everything stays inside `out` (reads only from the top-left
    // quarter that is not yet overwritten).
    unsafe {
        for _ in 0..doubleable_height {
            // Double the pixels in the line.
            for _ in 0..doubleable_width {
                *dst = *src;
                dst = dst.wrapping_offset(-1);
                *dst = *src;
                dst = dst.wrapping_offset(-1);
                src = src.wrapping_offset(-1);
            }
            // Copy a single extra pixel if the output width is odd.
            if odd_viewport_width {
                *dst = *src;
                dst = dst.wrapping_offset(-1);
                src = src.wrapping_offset(-1);
            }
            // Skip the rest of the source line.
            src = src.wrapping_offset(-(pitch - src_width as isize));
            // Double the line.
            std::ptr::copy(dst.wrapping_offset(1), dst.wrapping_offset(-pitch + 1), viewport_width as usize);
            // Skip the rest of the destination line.
            dst = dst.wrapping_offset(-(2 * pitch - viewport_width as isize));
        }
        if (out.h() % 2) == 1 {
            std::ptr::copy(dst.wrapping_offset(1), dst.wrapping_offset(-pitch + 1), viewport_width as usize);
        }
    }
}

/// Original: `CalcFirstTilePosition` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::CalcFirstTilePosition(Point &position, Displacement &offset) sha=a3de13e4e723
fn calc_first_tile_position(ctx: &Ctx, position: &mut Point, offset: &mut Displacement) {
    // Adjust by player offset and tile grid alignment
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let my_player = &ctx.players.Players[me];
    *offset = ctx.scrollrt.tile_offset;
    if my_player.is_walking() {
        *offset = *offset + get_offset_for_walking(ctx, &my_player.AnimInfo, my_player._pdir, true);
    }
    let free = crate::freemove::render_offset(ctx, me);
    if let Some(o) = free {
        // free movement: the camera follows the player between tiles
        *offset = *offset - o;
    }
    *position = *position + ctx.scrollrt.tile_shift;
    let zoom = ctx.options.graphics.zoom.get();
    // Skip rendering parts covered by the panels
    if crate::control::can_panels_cover_view(ctx) && (crate::control::is_left_panel_open(ctx) || crate::control::is_right_panel_open(ctx)) {
        let multiplier = if zoom { 1 } else { 2 };
        *position = *position + Displacement::from_direction(Direction::East) * multiplier;
        offset.delta_x += -TILE_WIDTH * multiplier / 2 / 2;
        if crate::control::is_left_panel_open(ctx) && !zoom {
            offset.delta_x += crate::control::SIDE_PANEL_SIZE.0;
            // SidePanelSize.width accounted for in Zoom()
        }
    }
    // Draw areas moving in and out of the screen
    if free.is_some() {
        offset.delta_y -= TILE_HEIGHT;
        *position = *position + Direction::North;
        offset.delta_x -= TILE_WIDTH;
        *position = *position + Direction::West;
    }
    if my_player.is_walking() {
        match my_player._pdir {
            Direction::North | Direction::NorthEast => {
                offset.delta_y -= TILE_HEIGHT;
                *position = *position + Direction::North;
            }
            Direction::SouthWest | Direction::West => {
                offset.delta_x -= TILE_WIDTH;
                *position = *position + Direction::West;
            }
            Direction::NorthWest => {
                offset.delta_x -= TILE_WIDTH / 2;
                offset.delta_y -= TILE_HEIGHT / 2;
                *position = *position + Direction::NorthWest;
            }
            _ => {}
        }
    }
}

/// Original: `DrawGame` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawGame(const Surface &fullOut, Point position, Displacement offset) sha=dc4ed1fb117d
fn draw_game(ctx: &mut Ctx, full_out: &Surface, position: Point, offset: Displacement) {
    let zoom = ctx.options.graphics.zoom.get();
    let vh = ctx.dx.gn_viewport_height;
    // Limit rendering to the view area
    let out = if !zoom { full_out.subregion_y(0, vh) } else { full_out.subregion_y(0, (vh + 1) / 2) };
    let mut columns = ctx.scrollrt.tile_colums;
    let mut rows = ctx.scrollrt.tile_rows;
    // Skip rendering parts covered by the panels
    if crate::control::can_panels_cover_view(ctx) && (crate::control::is_left_panel_open(ctx) || crate::control::is_right_panel_open(ctx)) {
        columns -= if zoom { 2 } else { 4 };
    }
    update_missiles_renderer_data(ctx);
    // Draw areas moving in and out of the screen
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if crate::freemove::render_offset(ctx, me).is_some() {
        rows += 3;
        columns += 2;
    }
    if ctx.players.Players[me].is_walking() {
        match ctx.players.Players[me]._pdir {
            Direction::NoDirection => {}
            Direction::North | Direction::South => rows += 2,
            Direction::NorthEast => {
                columns += 1;
                rows += 2;
            }
            Direction::East | Direction::West => columns += 1,
            Direction::SouthEast | Direction::SouthWest | Direction::NorthWest => {
                columns += 1;
                rows += 1;
            }
        }
    }
    let start = Point::new(0, 0) + offset;
    draw_floor(ctx, &out, position, start, rows, columns);
    draw_tile_content(ctx, &out, position, start, rows, columns);
    if zoom {
        zoom_view(ctx, &full_out.subregion_y(0, vh));
    }
}

fn zoom_view(ctx: &Ctx, out: &Surface) {
    zoom(ctx, out);
}

/// Original: `DrawView` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawView(const Surface &out, Point startPosition) sha=f2935d3b6f0b
fn draw_view(ctx: &mut Ctx, out: &Surface, mut start_position: Point) {
    let mut offset = Displacement::default();
    let first_person = crate::firstperson::draw(ctx, &out.subregion_y(0, ctx.dx.gn_viewport_height));
    if !first_person {
        calc_first_tile_position(ctx, &mut start_position, &mut offset);
        draw_game(ctx, out, start_position, offset);
    }
    if crate::automap::automap_active(ctx) {
        let vh = ctx.dx.gn_viewport_height;
        crate::automap::draw_automap(ctx, &out.subregion_y(0, vh));
    }
    crate::qol::itemlabels::draw_item_name_labels(ctx, out);
    crate::qol::monhealthbar::draw_monster_health_bar(ctx, out);
    if !first_person {
        crate::qol::floatingnumbers::draw_floating_numbers(ctx, out, start_position, offset);
    }

    if !crate::stores::stextflag_is_none(ctx) && !crate::minitext::qtextflag(ctx) {
        crate::stores::draw_s_text(ctx, out);
    }
    if ctx.inv.invflag {
        crate::inv::draw_inv(ctx, out);
    } else if ctx.control.sbookflag {
        crate::panels::spell_book::draw_spell_book(ctx, out);
    }
    crate::control::draw_dur_icon(ctx, out);
    if ctx.control.chrflag {
        crate::panels::charpanel::draw_chr(ctx, out);
    } else if ctx.quests.QuestLogIsOpen {
        crate::quests::draw_quest_log(ctx, out);
    } else if ctx.stash.IsStashOpen {
        crate::qol::stash::draw_stash(ctx, out);
    }
    crate::control::draw_level_up_icon(ctx, out);
    if ctx.items.ShowUniqueItemInfoBox {
        crate::items::draw_unique_info(ctx, out);
    }
    if crate::minitext::qtextflag(ctx) {
        crate::minitext::draw_q_text(ctx, out);
    }
    if ctx.control.spselflag {
        crate::panels::spell_list::draw_spell_list(ctx, out);
    }
    if ctx.control.drop_gold_flag {
        let v = ctx.control.drop_gold_value;
        crate::control::draw_gold_split(ctx, out, v);
    }
    crate::qol::stash::draw_gold_withdraw(ctx, out);
    if ctx.help.HelpFlag {
        crate::help::draw_help(ctx, out);
    }
    if ctx.chatlog.ChatLogFlag {
        crate::qol::chatlog::draw_chat_log(ctx, out);
    }
    if crate::error::is_diablo_msg_available(ctx) {
        crate::error::draw_diablo_msg(ctx, out);
    }
    if ctx.players.MyPlayerIsDead {
        crate::control::red_back(ctx, out);
    } else if ctx.diablo.pause_mode != 0 {
        crate::gmenu::gmenu_draw_pause(ctx, out);
    }
    crate::controls::modifier_hints::draw_controller_modifier_hints(ctx, out);
    crate::plrmsg::draw_plr_msg(ctx, out);
    crate::gmenu::gmenu_draw(ctx, out);
    crate::doom::doom_draw(ctx, out);
    crate::control::draw_info_box(ctx, out);
    crate::control::control_update_life_mana(ctx); // Update life/mana totals before rendering any portion of the flask.
    crate::control::draw_life_flask_upper(ctx, out);
    crate::control::draw_mana_flask_upper(ctx, out);
}

/// Original: `DrawFPS` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawFPS(const Surface &out) sha=17942c1ff9a8
fn draw_fps(ctx: &mut Ctx, out: &Surface) {
    if !ctx.diablo.frameflag || !ctx.init.gb_active {
        return;
    }
    ctx.scrollrt.frames_since_last_update += 1;
    let runtime_in_ms = ctx.platform.ticks();
    let ms_since_last_update = runtime_in_ms.wrapping_sub(ctx.diablo.last_fps_update_in_ms);
    if ms_since_last_update >= 1000 {
        ctx.diablo.last_fps_update_in_ms = runtime_in_ms;
        const FPS_POW10: i32 = 10;
        let fps = (1000 * FPS_POW10 * ctx.scrollrt.frames_since_last_update) / ms_since_last_update as i32;
        ctx.scrollrt.frames_since_last_update = 0;
        ctx.scrollrt.fps_formatted =
            if fps >= 100 * FPS_POW10 { format!("{} FPS", fps / FPS_POW10) } else { format!("{}.{} FPS", fps / FPS_POW10, fps % FPS_POW10) };
    }
    let s = ctx.scrollrt.fps_formatted.clone();
    crate::engine::render::text_render::draw_string(ctx, out, &s, Rect::new(8, 68, 0, 0), crate::engine::render::text_render::UiFlags::COLOR_RED, 1, -1);
}

/// Original: `DoBlitScreen` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DoBlitScreen(int x, int y, int w, int h) sha=c9e263ad0e44
fn do_blit_screen(ctx: &mut Ctx, x: i32, y: i32, w: i32, h: i32) {
    let r = Rect::new(x, y, w, h);
    crate::engine::dx::blt_fast(ctx, Some(r), Some(r));
}

/// Original: `DrawMain` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawMain(const Surface &out, int dwHgt, bool drawDesc, bool drawHp, bool drawMana, bool drawSbar, bool drawBtn) sha=47a5ca932f3a
#[allow(clippy::too_many_arguments)]
fn draw_main(ctx: &mut Ctx, dw_hgt: i32, draw_desc: bool, draw_hp: bool, draw_mana: bool, draw_sbar: bool, draw_btn: bool) {
    if !ctx.init.gb_active || ctx.dx.render_directly_to_output_surface {
        return;
    }
    let sw = ctx.dx.gn_screen_width;
    let sh = ctx.dx.gn_screen_height;
    debug_assert!(dw_hgt >= 0 && dw_hgt <= sh);
    if dw_hgt > 0 {
        do_blit_screen(ctx, 0, 0, sw, dw_hgt);
    }
    if dw_hgt < sh {
        let mp = crate::control::get_main_panel(ctx);
        if draw_sbar {
            do_blit_screen(ctx, mp.x + 204, mp.y + 5, 232, 28);
        }
        if draw_desc {
            if ctx.control.talkflag {
                // When chat input is displayed, the belt is hidden and the chat moves up.
                do_blit_screen(ctx, mp.x + 171, mp.y + 6, 298, 116);
            } else {
                let (tl, size) = (crate::control::INFO_BOX_TOP_LEFT, crate::control::INFO_BOX_SIZE);
                do_blit_screen(ctx, mp.x + tl.delta_x, mp.y + tl.delta_y, size.width, size.height);
            }
        }
        if draw_mana {
            do_blit_screen(ctx, mp.x + 460, mp.y, 88, 72);
            do_blit_screen(ctx, mp.x + 564, mp.y + 64, 56, 56);
        }
        if draw_hp {
            do_blit_screen(ctx, mp.x + 96, mp.y, 88, 72);
        }
        if draw_btn {
            do_blit_screen(ctx, mp.x + 8, mp.y + 7, 74, 114);
            do_blit_screen(ctx, mp.x + 559, mp.y + 7, 74, 48);
            if ctx.init.gb_is_multiplayer {
                do_blit_screen(ctx, mp.x + 86, mp.y + 91, 34, 32);
                do_blit_screen(ctx, mp.x + 526, mp.y + 91, 34, 32);
            }
        }
        let prev = ctx.scrollrt.prev_cursor_rect;
        if prev.w != 0 && prev.h != 0 {
            do_blit_screen(ctx, prev.x, prev.y, prev.w, prev.h);
        }
        let cursor_rect = get_drawn_cursor(ctx).rect;
        if cursor_rect.w != 0 && cursor_rect.h != 0 {
            do_blit_screen(ctx, cursor_rect.x, cursor_rect.y, cursor_rect.w, cursor_rect.h);
        }
    }
}

/// Original: `devilution::GetOffsetForWalking` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::GetOffsetForWalking(const AnimationInfo &animationInfo, const Direction dir, bool cameraMode) sha=5a6c2143c7c0
pub fn get_offset_for_walking(ctx: &Ctx, animation_info: &crate::engine::animationinfo::AnimationInfo, dir: Direction, camera_mode: bool) -> Displacement {
    //                                    South,     SouthWest,   West,      NorthWest, North,     NorthEast,  East,       SouthEast,
    const START_OFFSET: [(i32, i32); 8] = [(0, -32), (32, -16), (64, 0), (0, 0), (0, 0), (0, 0), (-64, 0), (-32, -16)];
    const MOVING_OFFSET: [(i32, i32); 8] = [(0, 32), (-32, 16), (-64, 0), (-32, -16), (0, -32), (32, -16), (64, 0), (32, 16)];
    let animation_progress = animation_info.get_animation_progress(ctx.nthread.ProgressToNextGameTick) as i32;
    let m = MOVING_OFFSET[dir as usize];
    let base = crate::engine::animationinfo::AnimationInfo::BASE_VALUE_FRACTION;
    let mut offset = Displacement::new(m.0 * animation_progress / base, m.1 * animation_progress / base);
    if camera_mode {
        offset = Displacement::new(-offset.delta_x, -offset.delta_y);
    } else {
        let s = START_OFFSET[dir as usize];
        offset = offset + Displacement::new(s.0, s.1);
    }
    offset
}

/// Original: `devilution::ClearCursor` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::ClearCursor() sha=40d3851f94fa
pub fn clear_cursor(ctx: &mut Ctx) {
    ctx.scrollrt.prev_cursor_rect = Rect::default();
}

/// Original: `devilution::ShiftGrid` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::ShiftGrid(int *x, int *y, int horizontal, int vertical) sha=8169f5dcdeb8
pub fn shift_grid(x: &mut i32, y: &mut i32, horizontal: i32, vertical: i32) {
    *x += vertical + horizontal;
    *y += vertical - horizontal;
}

/// Original: `devilution::RowsCoveredByPanel` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::RowsCoveredByPanel() sha=c5a53537879c
pub fn rows_covered_by_panel(ctx: &Ctx) -> i32 {
    let mp = crate::control::get_main_panel(ctx);
    if crate::utils::display::get_screen_width(ctx) <= mp.w {
        return 0;
    }
    let mut rows = mp.h / TILE_HEIGHT;
    if ctx.options.graphics.zoom.get() {
        rows /= 2;
    }
    rows
}

/// Original: `devilution::CalcTileOffset` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::CalcTileOffset(int *offsetX, int *offsetY) sha=a7cddbc6e038
pub fn calc_tile_offset(ctx: &Ctx) -> (i32, i32) {
    let screen_width = crate::utils::display::get_screen_width(ctx) as u16 as i32;
    let viewport_height = crate::utils::display::get_viewport_height(ctx) as u16 as i32;
    let (mut x, mut y) = if !ctx.options.graphics.zoom.get() {
        (screen_width % TILE_WIDTH, viewport_height % TILE_HEIGHT)
    } else {
        ((screen_width / 2) % TILE_WIDTH, (viewport_height / 2) % TILE_HEIGHT)
    };
    if x != 0 {
        x = (TILE_WIDTH - x) / 2;
    }
    if y != 0 {
        y = (TILE_HEIGHT - y) / 2;
    }
    (x, y)
}

/// Original: `devilution::TilesInView` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::TilesInView(int *rcolumns, int *rrows) sha=0b6889af0434
pub fn tiles_in_view(ctx: &Ctx) -> (i32, i32) {
    let screen_width = crate::utils::display::get_screen_width(ctx) as u16 as i32;
    let viewport_height = crate::utils::display::get_viewport_height(ctx) as u16 as i32;
    let mut columns = screen_width / TILE_WIDTH;
    if (screen_width % TILE_WIDTH) != 0 {
        columns += 1;
    }
    let mut rows = viewport_height / TILE_HEIGHT;
    if (viewport_height % TILE_HEIGHT) != 0 {
        rows += 1;
    }
    if ctx.options.graphics.zoom.get() {
        // Half the number of tiles, rounded up
        if (columns & 1) != 0 {
            columns += 1;
        }
        columns /= 2;
        if (rows & 1) != 0 {
            rows += 1;
        }
        rows /= 2;
    }
    (columns, rows)
}

/// Original: `devilution::CalcViewportGeometry` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::CalcViewportGeometry() sha=ba0b7590c962
pub fn calc_viewport_geometry(ctx: &mut Ctx) {
    let zoom = ctx.options.graphics.zoom.get();
    let zoom_factor = if zoom { 2 } else { 1 };
    let screen_width = crate::utils::display::get_screen_width(ctx) / zoom_factor;
    let screen_height = crate::utils::display::get_screen_height(ctx) / zoom_factor;
    let panel_height = crate::control::get_main_panel(ctx).h / zoom_factor;
    let pixels_to_panel = screen_height - panel_height;
    let mut player_position = Point::new(screen_width / 2, pixels_to_panel / 2);
    if zoom {
        player_position.y += TILE_HEIGHT / 4;
    }
    let tiles_to_top = (player_position.y + TILE_HEIGHT - 1) / TILE_HEIGHT;
    let tiles_to_left = (player_position.x + TILE_WIDTH - 1) / TILE_WIDTH;
    // Location of the center of the tile from which to start rendering, relative to the viewport origin
    let mut start_position = player_position - Displacement::new(tiles_to_left * TILE_WIDTH, tiles_to_top * TILE_HEIGHT);
    // Position of the tile from which to start rendering in tile space,
    // relative to the tile the player character occupies
    let mut tile_shift = Displacement::new(0, 0);
    tile_shift = tile_shift + Displacement::from_direction(Direction::North) * tiles_to_top;
    tile_shift = tile_shift + Displacement::from_direction(Direction::West) * tiles_to_left;
    // The rendering loop expects to start on a row with fewer columns
    if tiles_to_left * TILE_WIDTH >= player_position.x {
        start_position = start_position + Displacement::new(TILE_WIDTH / 2, -TILE_HEIGHT / 2);
        tile_shift = tile_shift + Displacement::from_direction(Direction::NorthEast);
    } else if tiles_to_top * TILE_HEIGHT < player_position.y {
        // There is one row above the current row that needs to be rendered,
        // but we skip to the row above it because it has too many columns
        start_position = start_position + Displacement::new(0, -TILE_HEIGHT);
        tile_shift = tile_shift + Displacement::from_direction(Direction::North);
    }
    ctx.scrollrt.tile_shift = tile_shift;
    // Location of the bottom-left corner of the bounding box around the
    // tile from which to start rendering, relative to the viewport origin
    ctx.scrollrt.tile_offset = Displacement::new(start_position.x - TILE_WIDTH / 2, start_position.y + TILE_HEIGHT / 2 - 1);
    // Compute the number of rows to be rendered as well as
    // the number of columns to be rendered in the first row
    let viewport_height = crate::utils::display::get_viewport_height(ctx) / zoom_factor;
    let render_start = start_position - Displacement::new(TILE_WIDTH / 2, TILE_HEIGHT / 2);
    ctx.scrollrt.tile_rows = (viewport_height - render_start.y + TILE_HEIGHT / 2 - 1) / (TILE_HEIGHT / 2);
    ctx.scrollrt.tile_colums = (screen_width - render_start.x + TILE_WIDTH - 1) / TILE_WIDTH;
}

/// Original: `devilution::ClearScreenBuffer` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::ClearScreenBuffer() sha=a4409be5c4f2
pub fn clear_screen_buffer(ctx: &mut Ctx) {
    if ctx.diablo.headless_mode {
        return;
    }
    let s = ctx.dx.pal_surface.as_mut().expect("PalSurface");
    s.pixels.fill(0);
}

/// Original: `devilution::scrollrt_draw_game_screen` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::scrollrt_draw_game_screen() sha=f3aadc9b278c
pub fn scrollrt_draw_game_screen(ctx: &mut Ctx) {
    if ctx.diablo.headless_mode {
        return;
    }
    let mut hgt = 0;
    if is_redraw_everything(ctx) {
        redraw_complete(ctx);
        hgt = ctx.dx.gn_screen_height;
    }
    let out = crate::engine::dx::global_back_buffer(ctx);
    undraw_cursor(ctx, &out);
    draw_main(ctx, hgt, false, false, false, false, false);
    draw_cursor(ctx, &out);
    crate::engine::dx::render_present(ctx);
}

/// Original: `devilution::DrawAndBlit` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::DrawAndBlit() sha=fcaa87269b0a
pub fn draw_and_blit(ctx: &mut Ctx) {
    if !ctx.diablo.gb_run_game || ctx.diablo.headless_mode {
        return;
    }
    crate::firstperson::sync(ctx);
    let mut hgt = 0;
    let mut draw_health = is_redraw_component(ctx, PanelDrawComponent::Health);
    let mut draw_mana = is_redraw_component(ctx, PanelDrawComponent::Mana);
    let mut draw_control_buttons = is_redraw_component(ctx, PanelDrawComponent::ControlButtons);
    let mut draw_belt = is_redraw_component(ctx, PanelDrawComponent::Belt);
    let draw_chat_input = ctx.control.talkflag;
    let mut draw_info_box = false;
    let mut draw_ctrl_pan = false;
    let main_panel = crate::control::get_main_panel(ctx);
    if ctx.dx.gn_screen_width > main_panel.w || is_redraw_everything(ctx) {
        draw_health = true;
        draw_mana = true;
        draw_control_buttons = true;
        draw_belt = true;
        draw_info_box = false;
        draw_ctrl_pan = true;
        hgt = ctx.dx.gn_screen_height;
    } else if is_redraw_viewport(ctx) {
        draw_info_box = true;
        draw_ctrl_pan = false;
        hgt = ctx.dx.gn_viewport_height;
    }
    let out = crate::engine::dx::global_back_buffer(ctx);
    undraw_cursor(ctx, &out);
    crate::nthread::nthread_update_progress_to_next_game_tick(ctx);
    let view = ctx.gendung.ViewPosition;
    draw_view(ctx, &out, view);
    if draw_ctrl_pan {
        crate::control::draw_ctrl_pan(ctx, &out);
    }
    if draw_health {
        crate::control::draw_life_flask_lower(ctx, &out);
    }
    if draw_mana {
        crate::control::draw_mana_flask_lower(ctx, &out);
        crate::panels::spell_list::draw_spell(ctx, &out);
    }
    if draw_control_buttons {
        crate::control::draw_ctrl_btns(ctx, &out);
    }
    if draw_belt {
        crate::inv::draw_inv_belt(ctx, &out);
    }
    if draw_chat_input {
        crate::control::draw_talk_pan(ctx, &out);
    }
    crate::qol::xpbar::draw_xp_bar(ctx, &out);
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.options.gameplay.show_health_values.get() {
        let (hp, max) = (ctx.players.Players[me]._pHitPoints >> 6, ctx.players.Players[me]._pMaxHP >> 6);
        crate::control::draw_flask_values(ctx, &out, Point::new(main_panel.x + 134, main_panel.y + 28), hp, max);
    }
    if ctx.options.gameplay.show_mana_values.get() {
        let (mana, max) = (ctx.players.Players[me]._pMana >> 6, ctx.players.Players[me]._pMaxMana >> 6);
        crate::control::draw_flask_values(ctx, &out, Point::new(main_panel.x + main_panel.w - 138, main_panel.y + 28), mana, max);
    }
    draw_cursor(ctx, &out);
    draw_fps(ctx, &out);
    draw_main(ctx, hgt, draw_info_box, draw_health, draw_mana, draw_belt, draw_control_buttons);
    redraw_complete(ctx);
    for component in [PanelDrawComponent::Health, PanelDrawComponent::Mana, PanelDrawComponent::ControlButtons, PanelDrawComponent::Belt] {
        if is_redraw_component(ctx, component) {
            redraw_component_complete(ctx, component);
        }
    }
    crate::engine::dx::render_present(ctx);
}
