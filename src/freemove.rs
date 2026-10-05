//! Free movement (cargo feature `free-movement`, not part of the original): the player moves in
//! any direction at a constant speed, like a modern action RPG, instead of stepping tile to tile
//! in eight directions.
//!
//! The game logic stays tile-based: the player occupies the tile its position rounds to, and
//! everything that reads `position.tile` (monsters, missiles, triggers, lighting, saving) keeps
//! working. On top of that the player has a continuous position, and the sprite and the camera
//! are offset from the tile by the difference. While moving the player stays in `PM_STAND` with
//! the walk animation playing, so an attack, spell or hit interrupts it as it interrupts standing.
//!
//! Input: a click or held mouse button on the ground moves toward the exact point under the
//! cursor (around obstacles by the game's own pathfinding, cutting corners where the way is
//! clear); clicking a monster, item or object walks up to it and then does the original action;
//! the left stick moves in its direction. Single player only: in multiplayer the original
//! movement is used, so games stay compatible.

use crate::ctx::Ctx;
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::enums::*;
use crate::levels::gendung::DungeonType;

/// Whether this build has free movement.
pub const ENABLED: bool = cfg!(feature = "free-movement");

/// Tiles per game tick (the original covers one tile in about 8 ticks, diagonally more).
const SPEED: f32 = 0.15;
/// How close to a waypoint counts as reached.
const ARRIVE: f32 = 0.05;
/// How far into a tile next to a wall the player may stand (half a tile is the edge).
const EDGE: f32 = 0.45;

#[derive(Default)]
pub struct FreeMoveState {
    /// Continuous position in tiles (integers are tile centres).
    pos: (f32, f32),
    /// The tile `pos` belongs to, to notice when the game moved the player itself.
    tile: Point,
    /// Points still to walk through; the last one is the destination.
    waypoints: Vec<(f32, f32)>,
    /// Stop as soon as the player's tile is next to this one (walking up to a target).
    stop_next_to: Option<Point>,
    /// Movement direction from the stick, in tiles (length up to 1).
    stick: Option<(f32, f32)>,
    /// Whether the walk animation is running, and in which direction.
    walk_anim: Option<Direction>,
    /// Sprite offset from the tile centre in screen pixels, last tick and this tick.
    prev_off: (f32, f32),
    cur_off: (f32, f32),
    /// The exact ground point under the cursor (in tiles), set by `CheckCursMove`.
    pub cursor_point: Option<(f32, f32)>,
}

/// Free movement applies to this player now.
pub fn active_for(ctx: &Ctx, pnum: usize) -> bool {
    ENABLED && !ctx.init.gb_is_multiplayer && ctx.players.MyPlayer == Some(pnum)
}

fn world_to_screen(d: (f32, f32)) -> (f32, f32) {
    ((d.0 - d.1) * 32.0, (d.0 + d.1) * 16.0)
}

fn screen_to_world(s: (f32, f32)) -> (f32, f32) {
    ((s.0 / 32.0 + s.1 / 16.0) / 2.0, (s.1 / 16.0 - s.0 / 32.0) / 2.0)
}

fn rounded(p: (f32, f32)) -> Point {
    Point::new(p.0.round() as i32, p.1.round() as i32)
}

fn tile_f(t: Point) -> (f32, f32) {
    (t.x as f32, t.y as f32)
}

/// The sprite's facing for a movement in tiles: the nearest of the eight screen directions.
fn facing(d: (f32, f32)) -> Direction {
    let s = world_to_screen(d);
    // angle on the ground plane seen from above (screen y doubled undoes the isometric
    // squash), 0 = screen right, counter-clockwise
    let angle = (-2.0 * s.1).atan2(s.0).to_degrees();
    let sector = (((angle + 360.0 + 22.5) % 360.0) / 45.0) as usize;
    [
        Direction::East,
        Direction::NorthEast,
        Direction::North,
        Direction::NorthWest,
        Direction::West,
        Direction::SouthWest,
        Direction::South,
        Direction::SouthEast,
    ][sector.min(7)]
}

/// The tile can be stood on (`PosOkPlayer`), or is the player's own.
fn tile_ok(ctx: &Ctx, pnum: usize, t: Point) -> bool {
    t == ctx.players.Players[pnum].position.tile || crate::player::pos_ok_player(ctx, pnum, t)
}

/// Moving between two neighbouring tiles; diagonal steps may not cut a solid corner.
fn step_ok(ctx: &Ctx, pnum: usize, from: Point, to: Point) -> bool {
    if from == to {
        return true;
    }
    if !tile_ok(ctx, pnum, to) {
        return false;
    }
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    if dx != 0 && dy != 0 {
        use crate::engine::path::is_tile_solid;
        if is_tile_solid(ctx, Point::new(from.x + dx, from.y)) || is_tile_solid(ctx, Point::new(from.x, from.y + dy)) {
            return false;
        }
    }
    true
}

/// A straight walk from `a` to `b` crosses only tiles that can be stood on.
fn line_clear(ctx: &Ctx, pnum: usize, a: (f32, f32), b: (f32, f32)) -> bool {
    let (dx, dy) = (b.0 - a.0, b.1 - a.1);
    let n = ((dx.abs().max(dy.abs())) / 0.2).ceil().max(1.0) as i32;
    let mut prev = rounded(a);
    for i in 1..=n {
        let t = i as f32 / n as f32;
        let p = rounded((a.0 + dx * t, a.1 + dy * t));
        if p != prev {
            if !step_ok(ctx, pnum, prev, p) {
                return false;
            }
            prev = p;
        }
    }
    true
}

fn reset_to_tile(ctx: &mut Ctx, pnum: usize) {
    let t = ctx.players.Players[pnum].position.tile;
    let s = &mut ctx.freemove;
    s.pos = tile_f(t);
    s.tile = t;
    s.waypoints.clear();
    s.stop_next_to = None;
    s.prev_off = (0.0, 0.0);
    s.cur_off = (0.0, 0.0);
}

/// Instead of `MakePlrPath`: walk to `target` (onto it, or up to it when `endspace` is false,
/// as the original's path would end next to it). A ground click walks to the exact point under
/// the cursor.
pub fn go_to(ctx: &mut Ctx, pnum: usize, target: Point, endspace: bool) {
    if ctx.freemove.tile != ctx.players.Players[pnum].position.tile {
        reset_to_tile(ctx, pnum);
    }
    let from = ctx.players.Players[pnum].position.tile;
    let mut waypoints = Vec::new();
    let mut walkpath = [WALK_NONE as i8; crate::engine::path::MaxPathLength];
    let steps = crate::engine::path::find_path(ctx, &|ctx, pos| crate::player::pos_ok_player(ctx, pnum, pos), from, target, &mut walkpath);
    let mut t = from;
    for &code in walkpath.iter().take(steps.max(0) as usize) {
        let d = match code as i32 {
            WALK_N => Direction::North,
            WALK_NE => Direction::NorthEast,
            WALK_E => Direction::East,
            WALK_SE => Direction::SouthEast,
            WALK_S => Direction::South,
            WALK_SW => Direction::SouthWest,
            WALK_W => Direction::West,
            WALK_NW => Direction::NorthWest,
            _ => break,
        };
        t = t + d;
        waypoints.push(tile_f(t));
    }
    let s = &mut ctx.freemove;
    if endspace {
        // the exact point under the cursor when this is a click on the ground there
        let exact = s.cursor_point.filter(|&p| rounded(p) == target);
        match waypoints.last_mut() {
            Some(last) if rounded(*last) == target => {
                if let Some(p) = exact {
                    *last = p;
                }
            }
            Some(_) => {}
            None if steps <= 0 && (target.x - from.x).abs() <= 1 && (target.y - from.y).abs() <= 1 => {
                waypoints.push(exact.unwrap_or(tile_f(target)));
            }
            None => {}
        }
        s.stop_next_to = None;
    } else {
        if waypoints.is_empty() {
            waypoints.push(tile_f(target));
        }
        s.stop_next_to = Some(target);
    }
    s.waypoints = waypoints;
    s.stick = None;
}

/// Stick input (gamepad builds): `dir` is the stick in screen terms (x right, y up), or `None`.
pub fn stick(ctx: &mut Ctx, dir: Option<(f32, f32)>) {
    match dir {
        Some((x, y)) if x * x + y * y > 0.01 => {
            let w = screen_to_world((x * 32.0, -y * 16.0 * 2.0));
            let len = (w.0 * w.0 + w.1 * w.1).sqrt();
            let mag = (x * x + y * y).sqrt().min(1.0);
            ctx.freemove.stick = Some((w.0 / len * mag, w.1 / len * mag));
            ctx.freemove.waypoints.clear();
            ctx.freemove.stop_next_to = None;
        }
        _ => ctx.freemove.stick = None,
    }
}

/// The player is walking somewhere (the original's `CheckNewPath` waits until it arrives).
pub fn is_moving(ctx: &Ctx, pnum: usize) -> bool {
    active_for(ctx, pnum) && !ctx.freemove.waypoints.is_empty()
}

/// Moves into the tile `to` (the game's occupancy, light and view follow).
fn enter_tile(ctx: &mut Ctx, pnum: usize, to: Point) {
    let id = (pnum + 1) as i8;
    let from = ctx.players.Players[pnum].position.tile;
    if ctx.gendung.dPlayer[from.x as usize][from.y as usize] == id {
        ctx.gendung.dPlayer[from.x as usize][from.y as usize] = 0;
    }
    {
        let p = &mut ctx.players.Players[pnum];
        p.position.old = from;
        p.position.tile = to;
        p.position.future = to;
    }
    ctx.gendung.dPlayer[to.x as usize][to.y as usize] = id;
    ctx.gendung.ViewPosition = to;
    if ctx.gendung.leveltype != DungeonType::Town {
        let lid = ctx.players.Players[pnum].lightId;
        crate::lighting::change_light_xy(ctx, lid, to);
        crate::lighting::change_vision_xy(ctx, pnum as i32, to);
    }
    ctx.freemove.tile = to;
    // keep last tick's sprite offset in the new tile's terms, so drawing between ticks is smooth
    let shift = world_to_screen(((to.x - from.x) as f32, (to.y - from.y) as f32));
    ctx.freemove.prev_off = (ctx.freemove.prev_off.0 - shift.0, ctx.freemove.prev_off.1 - shift.1);
    crate::qol::autopickup::auto_pickup(ctx, pnum);
}

/// Tries to move to `new`; slides along walls. Returns whether the player moved.
fn try_move(ctx: &mut Ctx, pnum: usize, new: (f32, f32)) -> bool {
    let pos = ctx.freemove.pos;
    let tile = ctx.freemove.tile;
    // the full step, then sliding along one axis
    let candidates = [new, (new.0, pos.1), (pos.0, new.1)];
    for cand in candidates {
        if (cand.0 - pos.0).abs() < 1e-4 && (cand.1 - pos.1).abs() < 1e-4 {
            continue;
        }
        let t = rounded(cand);
        if t == tile {
            ctx.freemove.pos = cand;
            return true;
        }
        if (t.x - tile.x).abs() <= 1 && (t.y - tile.y).abs() <= 1 && step_ok(ctx, pnum, tile, t) {
            enter_tile(ctx, pnum, t);
            ctx.freemove.pos = cand;
            return true;
        }
    }
    // blocked: go up to the edge of the current tile
    let clamped = (new.0.clamp(tile.x as f32 - EDGE, tile.x as f32 + EDGE), new.1.clamp(tile.y as f32 - EDGE, tile.y as f32 + EDGE));
    if (clamped.0 - pos.0).abs() > 1e-3 || (clamped.1 - pos.1).abs() > 1e-3 {
        ctx.freemove.pos = clamped;
        return true;
    }
    false
}

/// Called once per game tick for the player, before its mode runs (`ProcessPlayers`).
pub fn tick(ctx: &mut Ctx, pnum: usize) {
    if !active_for(ctx, pnum) {
        return;
    }
    if ctx.freemove.tile != ctx.players.Players[pnum].position.tile {
        // the game placed the player itself (level change, teleport, load)
        reset_to_tile(ctx, pnum);
        ctx.freemove.walk_anim = None;
    }
    ctx.freemove.prev_off = ctx.freemove.cur_off;
    if std::env::var_os("DIABLO_FREEMOVE_TRACE").is_some() {
        let p = &ctx.players.Players[pnum];
        eprintln!("FREEMODE mode={} da={} dp={} tile=({},{})", p._pmode, p.destAction, p.destParam1, p.position.tile.x, p.position.tile.y);
    }
    let standing = ctx.players.Players[pnum]._pmode == PM_STAND && !ctx.players.Players[pnum]._pInvincible;
    if !standing {
        ctx.freemove.waypoints.clear();
        ctx.freemove.stop_next_to = None;
        ctx.freemove.walk_anim = None;
        return;
    }

    // walking up to a monster or player: follow it
    let da = ctx.players.Players[pnum].destAction;
    let dp = ctx.players.Players[pnum].destParam1;
    if ctx.freemove.stop_next_to.is_some() {
        let target = match da {
            ACTION_ATTACKMON if (dp as usize) < ctx.monster.Monsters.len() => Some(ctx.monster.Monsters[dp as usize].position.future),
            ACTION_ATTACKPLR if (dp as usize) < ctx.players.Players.len() => Some(ctx.players.Players[dp as usize].position.future),
            _ => None,
        };
        if let Some(t) = target {
            if Some(t) != ctx.freemove.stop_next_to {
                go_to(ctx, pnum, t, false);
            }
        }
        if let Some(t) = ctx.freemove.stop_next_to {
            let me = ctx.freemove.tile;
            if (me.x - t.x).abs() <= 1 && (me.y - t.y).abs() <= 1 {
                ctx.freemove.waypoints.clear();
                ctx.freemove.stop_next_to = None;
            }
        }
    }

    // the direction to move this tick
    let pos = ctx.freemove.pos;
    let mut step: Option<(f32, f32)> = None;
    if let Some(v) = ctx.freemove.stick {
        step = Some((v.0 * SPEED, v.1 * SPEED));
    } else {
        // skip waypoints that can be reached in a straight line
        while ctx.freemove.waypoints.len() > 1 && line_clear(ctx, pnum, pos, ctx.freemove.waypoints[1]) {
            ctx.freemove.waypoints.remove(0);
        }
        if let Some(&target) = ctx.freemove.waypoints.first() {
            let (dx, dy) = (target.0 - pos.0, target.1 - pos.1);
            let dist = (dx * dx + dy * dy).sqrt();
            if dist <= ARRIVE.max(SPEED) {
                step = Some((dx, dy));
                ctx.freemove.waypoints.remove(0);
            } else {
                step = Some((dx / dist * SPEED, dy / dist * SPEED));
            }
        }
    }
    if ctx.gendung.leveltype == DungeonType::Town && ctx.multi.sgGameInitInfo.bRunInTown != 0 {
        step = step.map(|s| (s.0 * 1.5, s.1 * 1.5));
    }

    let moved = match step {
        Some(s) if s.0.abs() > 1e-4 || s.1.abs() > 1e-4 => {
            let ok = try_move(ctx, pnum, (pos.0 + s.0, pos.1 + s.1));
            if !ok {
                // walled in: give up the destination
                ctx.freemove.waypoints.clear();
            }
            if ok { Some(s) } else { None }
        }
        _ => None,
    };

    match moved {
        Some(s) => {
            let dir = facing(s);
            let p = &ctx.players.Players[pnum];
            let walking_anim = p.AnimInfo.numberOfFrames == p._pWFrames;
            if ctx.freemove.walk_anim != Some(dir) || !walking_anim {
                let frame = if walking_anim { p.AnimInfo.currentFrame } else { 0 };
                crate::player::new_plr_anim(ctx, pnum, player_graphic::Walk, dir, AnimationDistributionFlags::None, 0, 0);
                ctx.players.Players[pnum].AnimInfo.currentFrame = frame;
                ctx.freemove.walk_anim = Some(dir);
            }
            ctx.players.Players[pnum]._pdir = dir;
            if ctx.options.audio.walking_sound.get() && (ctx.gendung.leveltype != DungeonType::Town || ctx.multi.sgGameInitInfo.bRunInTown == 0) {
                let cf = ctx.players.Players[pnum].AnimInfo.currentFrame;
                if (cf == 0 || cf == 4) && ctx.players.Players[pnum].AnimInfo.tickCounterOfCurrentFrame == 0 {
                    let t = ctx.players.Players[pnum].position.tile;
                    crate::effects::play_sfx_loc(ctx, crate::effects_data::PS_WALK1, t, true);
                }
            }
        }
        None => {
            if ctx.freemove.walk_anim.take().is_some() {
                let d = ctx.players.Players[pnum]._pdir;
                crate::player::new_plr_anim(ctx, pnum, player_graphic::Stand, d, AnimationDistributionFlags::None, 0, 0);
            }
            ctx.freemove.waypoints.clear();
        }
    }

    if std::env::var_os("DIABLO_FREEMOVE_TRACE").is_some() {
        let s = &ctx.freemove;
        let mut near = None;
        for i in 0..ctx.monster.ActiveMonsterCount {
            let m = ctx.monster.ActiveMonsters[i] as usize;
            let mp = ctx.monster.Monsters[m].position.tile;
            let d = (mp.x - s.tile.x).abs().max((mp.y - s.tile.y).abs());
            if ctx.monster.Monsters[m].hitPoints > 0 && d < 12 && near.is_none_or(|(_, dd, _)| d < dd) {
                near = Some((m, d, mp));
            }
        }
        eprintln!("FREEMOVE near={:?} hp={}", near, ctx.players.Players[pnum]._pHitPoints >> 6);
        eprintln!("FREEMOVE pos=({:.2},{:.2}) tile=({},{}) wp={:?} cursor={:?} curs=({},{}) da={} mode={}", s.pos.0, s.pos.1, s.tile.x, s.tile.y, s.waypoints.first(), s.cursor_point, ctx.cursor.cursPosition.x, ctx.cursor.cursPosition.y, ctx.players.Players[pnum].destAction, ctx.players.Players[pnum]._pmode);
    }
    let off = (ctx.freemove.pos.0 - ctx.freemove.tile.x as f32, ctx.freemove.pos.1 - ctx.freemove.tile.y as f32);
    ctx.freemove.cur_off = world_to_screen(off);
    if ctx.gendung.leveltype != DungeonType::Town {
        let lid = ctx.players.Players[pnum].lightId;
        if lid != crate::lighting::NO_LIGHT {
            let o = Displacement::new(ctx.freemove.cur_off.0.round() as i32, ctx.freemove.cur_off.1.round() as i32);
            crate::lighting::change_light_offset(ctx, lid, o.screen_to_light());
        }
    }
}

/// The player's sprite offset from its tile in screen pixels, between the last two ticks.
pub fn render_offset(ctx: &Ctx, pnum: usize) -> Option<Displacement> {
    if !active_for(ctx, pnum) || ctx.freemove.tile != ctx.players.Players[pnum].position.tile {
        return None;
    }
    let s = &ctx.freemove;
    let f = ctx.nthread.ProgressToNextGameTick as f32 / crate::engine::animationinfo::AnimationInfo::BASE_VALUE_FRACTION as f32;
    let x = s.prev_off.0 + (s.cur_off.0 - s.prev_off.0) * f;
    let y = s.prev_off.1 + (s.cur_off.1 - s.prev_off.1) * f;
    if x == 0.0 && y == 0.0 && s.prev_off == (0.0, 0.0) {
        return None;
    }
    Some(Displacement::new(x.round() as i32, y.round() as i32))
}

/// Where to draw the player: the tile whose drawing pass the sprite belongs to and the offset
/// from that tile. Like the original's walking player, the sprite is drawn with the
/// southernmost tile it stands between, so walls in front of it still cover it.
pub fn draw_position(ctx: &Ctx, pnum: usize) -> Option<(Point, Displacement)> {
    let off = render_offset(ctx, pnum)?;
    let tile = ctx.players.Players[pnum].position.tile;
    let w = screen_to_world((off.delta_x as f32, off.delta_y as f32));
    let d = Point::new(tile.x + (w.0 - 1e-3).ceil() as i32, tile.y + (w.1 - 1e-3).ceil() as i32);
    let s = world_to_screen(((d.x - tile.x) as f32, (d.y - tile.y) as f32));
    Some((d, Displacement::new(off.delta_x - s.0.round() as i32, off.delta_y - s.1.round() as i32)))
}

/// `CheckCursMove` found tile `base` with the cursor at (`px`, `py`) pixels inside the cell
/// whose left corner is that tile's centre: remember the exact ground point.
pub fn set_cursor_point(ctx: &mut Ctx, base: Point, px: i32, py: i32) {
    if !ENABLED {
        return;
    }
    let d = screen_to_world((px as f32, (py - crate::engine::render::dun_render::TILE_HEIGHT / 2) as f32));
    ctx.freemove.cursor_point = Some((base.x as f32 + d.0, base.y as f32 + d.1));
}
