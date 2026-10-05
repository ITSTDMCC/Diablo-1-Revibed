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

/// Walking speed: the original's walk takes 8 ticks per step, and a step covers one tile along
/// a tile axis (a screen diagonal) or a tile diagonal (screen up/down/left/right, 1.41 tiles).
/// Scaling by the direction's L1 length gives exactly those speeds in the eight directions and
/// in between, so the legs keep pace with the ground as in the original.
fn speed(dir: (f32, f32)) -> f32 {
    let len = (dir.0 * dir.0 + dir.1 * dir.1).sqrt();
    if len < 1e-6 {
        return 0.0;
    }
    (dir.0.abs() + dir.1.abs()) / len / 8.0
}
/// Below this a tick's movement counts as standing (pressing into a wall).
const MIN_MOVE: f32 = 0.02;
/// Ticks without movement before the walk animation stops (no stand/walk flicker).
const STOP_GRACE: u8 = 2;
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
    /// The destination of the walk in progress (`go_to`'s target and `endspace`).
    goal: Option<(Point, bool)>,
    /// The heading, smoothed over a few ticks, that decides the facing.
    heading: (f32, f32),
    /// Ticks the player has not moved while the walk animation runs.
    idle_ticks: u8,
    /// Ticks since the facing last changed (a new facing holds for a moment).
    face_age: u8,
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

const FACINGS: [Direction; 8] = [
    Direction::East,
    Direction::NorthEast,
    Direction::North,
    Direction::NorthWest,
    Direction::West,
    Direction::SouthWest,
    Direction::South,
    Direction::SouthEast,
];

/// A movement's angle on the ground plane seen from above (screen y doubled undoes the
/// isometric squash), 0 = screen right, counter-clockwise, in degrees.
fn ground_angle(d: (f32, f32)) -> f32 {
    let s = world_to_screen(d);
    (-2.0 * s.1).atan2(s.0).to_degrees()
}

/// The sprite's facing for a movement: the nearest of the eight directions, but the current
/// facing is kept until the movement is clearly closer to another one, so a path that zigzags
/// between two directions does not flip the sprite back and forth.
fn facing(d: (f32, f32), current: Option<Direction>) -> Direction {
    let angle = ground_angle(d);
    if let Some(cur) = current {
        if let Some(i) = FACINGS.iter().position(|&f| f == cur) {
            let centre = i as f32 * 45.0;
            let diff = ((angle - centre + 540.0) % 360.0 - 180.0).abs();
            if diff <= 22.5 + 6.0 {
                return cur;
            }
        }
    }
    let sector = (((angle + 360.0 + 22.5) % 360.0) / 45.0) as usize;
    FACINGS[sector.min(7)]
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
    s.goal = None;
    s.prev_off = (0.0, 0.0);
    s.cur_off = (0.0, 0.0);
}

/// Tiles searched at most for one path (the original's `FindPath` gives up after 25 steps).
const MAX_SEARCH: usize = 12_000;

/// A* over the tiles the player may enter, eight directions (diagonals may not cut a solid
/// corner). Returns the tiles to walk through, ending at `to`, or when `to` cannot be reached at
/// the reachable tile closest to it (an empty path when that is where the player stands).
fn find_route(ctx: &Ctx, pnum: usize, from: Point, to: Point) -> Vec<Point> {
    use crate::levels::gendung::{MAXDUNX, MAXDUNY};
    use std::cmp::Reverse;
    use std::collections::BinaryHeap;
    let idx = |p: Point| p.x as usize * MAXDUNY + p.y as usize;
    let h = |p: Point| {
        let (dx, dy) = ((p.x - to.x).abs(), (p.y - to.y).abs());
        10 * (dx + dy) - 6 * dx.min(dy)
    };
    let mut g = vec![i32::MAX; MAXDUNX * MAXDUNY];
    let mut parent = vec![u32::MAX; MAXDUNX * MAXDUNY];
    let mut open = BinaryHeap::new();
    g[idx(from)] = 0;
    open.push(Reverse((h(from), 0, from.x, from.y)));
    let mut best = from;
    let mut expanded = 0;
    while let Some(Reverse((_, cost, x, y))) = open.pop() {
        let cur = Point::new(x, y);
        if cost > g[idx(cur)] {
            continue;
        }
        if (h(cur), g[idx(cur)]) < (h(best), g[idx(best)]) {
            best = cur;
        }
        if cur == to {
            break;
        }
        expanded += 1;
        if expanded > MAX_SEARCH {
            break;
        }
        for dir in Direction::ALL8 {
            let next = cur + dir;
            if next.x < 0 || next.y < 0 || next.x >= MAXDUNX as i32 || next.y >= MAXDUNY as i32 {
                continue;
            }
            if !step_ok(ctx, pnum, cur, next) {
                continue;
            }
            let diagonal = next.x != cur.x && next.y != cur.y;
            let ng = cost + if diagonal { 14 } else { 10 };
            if ng < g[idx(next)] {
                g[idx(next)] = ng;
                parent[idx(next)] = idx(cur) as u32;
                open.push(Reverse((ng + h(next), ng, next.x, next.y)));
            }
        }
    }
    let mut route = Vec::new();
    let mut t = best;
    while t != from {
        route.push(t);
        let pi = parent[idx(t)] as usize;
        t = Point::new((pi / MAXDUNY) as i32, (pi % MAXDUNY) as i32);
    }
    route.reverse();
    route
}

/// Instead of `MakePlrPath`: walk to `target` (onto it, or up to it when `endspace` is false,
/// as the original's path would end next to it). A ground click walks to the exact point under
/// the cursor; a target that cannot be reached is walked toward as far as possible.
pub fn go_to(ctx: &mut Ctx, pnum: usize, target: Point, endspace: bool) {
    if ctx.freemove.tile != ctx.players.Players[pnum].position.tile {
        reset_to_tile(ctx, pnum);
    }
    let from = ctx.players.Players[pnum].position.tile;
    let exact = if endspace { ctx.freemove.cursor_point.filter(|&p| rounded(p) == target) } else { None };
    // Same destination as the walk in progress: keep its route (re-planning every tick makes
    // the hero waver between equally short routes), only the exact end point follows the cursor.
    if ctx.freemove.goal == Some((target, endspace)) && !ctx.freemove.waypoints.is_empty() {
        if let (Some(p), Some(last)) = (exact, ctx.freemove.waypoints.last_mut()) {
            if rounded(*last) == target {
                *last = p;
            }
        }
        ctx.freemove.stick = None;
        return;
    }
    let pos = ctx.freemove.pos;
    let end = exact.unwrap_or(tile_f(target));
    let mut waypoints: Vec<(f32, f32)>;
    if endspace && line_clear(ctx, pnum, pos, end) {
        // nothing in the way: straight there
        waypoints = vec![end];
    } else {
        let route = find_route(ctx, pnum, from, target);
        waypoints = route.iter().map(|&t| tile_f(t)).collect();
        let reached = route.last().copied().unwrap_or(from) == target;
        if endspace && reached {
            match waypoints.last_mut() {
                Some(last) => *last = end,
                None => waypoints.push(end), // a spot in the tile the player stands on
            }
        }
    }
    let s = &mut ctx.freemove;
    s.stop_next_to = if endspace { None } else { Some(target) };
    s.goal = Some((target, endspace));
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

/// The walk animation for `dir` is what the player shows (another action or an equipment change
/// may have replaced it).
fn is_walk_anim(ctx: &Ctx, pnum: usize, dir: Option<Direction>) -> bool {
    let Some(dir) = dir else { return false };
    let p = &ctx.players.Players[pnum];
    match (&p.AnimInfo.sprites, p.AnimationData[player_graphic::Walk as usize].sprites_for_direction(dir)) {
        (Some(cur), Some(walk)) => cur.get(0) == walk.get(0),
        // no graphics (headless runs): the frame count is all there is
        _ => p.AnimInfo.numberOfFrames == p._pWFrames,
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
        eprintln!("FREEMODE t={} mode={} da={} tile=({},{}) wps={} frame={}/{} goal={:?} face={:?}", ctx.platform.ticks(), p._pmode, p.destAction, p.position.tile.x, p.position.tile.y, ctx.freemove.waypoints.len(), p.AnimInfo.currentFrame, p.AnimInfo.numberOfFrames, ctx.freemove.goal, ctx.freemove.walk_anim);
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
        let sp = speed(v);
        step = Some((v.0 * sp, v.1 * sp));
    } else {
        // skip waypoints that can be reached in a straight line
        while ctx.freemove.waypoints.len() > 1 && line_clear(ctx, pnum, pos, ctx.freemove.waypoints[1]) {
            ctx.freemove.waypoints.remove(0);
        }
        if let Some(&target) = ctx.freemove.waypoints.first() {
            let (dx, dy) = (target.0 - pos.0, target.1 - pos.1);
            let dist = (dx * dx + dy * dy).sqrt();
            let sp = speed((dx, dy));
            if dist <= ARRIVE.max(sp) {
                step = Some((dx, dy));
                ctx.freemove.waypoints.remove(0);
            } else {
                step = Some((dx / dist * sp, dy / dist * sp));
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
            let p2 = ctx.freemove.pos;
            let d = (p2.0 - pos.0, p2.1 - pos.1);
            if ok && (d.0 * d.0 + d.1 * d.1).sqrt() >= MIN_MOVE { Some(d) } else { None }
        }
        _ => None,
    };

    match moved {
        Some(s) => {
            ctx.freemove.idle_ticks = 0;
            // the facing follows the heading smoothed over a few ticks, so single ticks of
            // sideways movement (corners, sliding) do not turn the sprite
            // Face where the walk is going (the destination), not this tick's step: routes
            // around trees and walls step between tile centres in alternating directions, and
            // following those steps turns the sprite back and forth.
            let p2 = ctx.freemove.pos;
            let toward = ctx.freemove.waypoints.last().map(|w| (w.0 - p2.0, w.1 - p2.1)).filter(|d| d.0 * d.0 + d.1 * d.1 > 0.75 * 0.75);
            let aim = toward.or(ctx.freemove.stick).unwrap_or(s);
            let len = (aim.0 * aim.0 + aim.1 * aim.1).sqrt().max(1e-6);
            let h = ctx.freemove.heading;
            let fresh = ctx.freemove.walk_anim.is_none() || h == (0.0, 0.0);
            let k = if fresh { 1.0 } else { 0.5 };
            let nh = (h.0 * (1.0 - k) + aim.0 / len * k, h.1 * (1.0 - k) + aim.1 / len * k);
            ctx.freemove.heading = nh;
            let mut dir = facing(nh, ctx.freemove.walk_anim);
            // a facing just taken holds for a few ticks unless the turn is large
            if let Some(cur) = ctx.freemove.walk_anim {
                if dir != cur && ctx.freemove.face_age < 4 && facing(nh, None) == dir {
                    let i = FACINGS.iter().position(|&f| f == cur).unwrap_or(0) as i32;
                    let j = FACINGS.iter().position(|&f| f == dir).unwrap_or(0) as i32;
                    if (i - j).rem_euclid(8) == 1 || (j - i).rem_euclid(8) == 1 {
                        dir = cur;
                    }
                }
            }
            if Some(dir) != ctx.freemove.walk_anim {
                ctx.freemove.face_age = 0;
            } else {
                ctx.freemove.face_age = ctx.freemove.face_age.saturating_add(1);
            }
            let walking_anim = is_walk_anim(ctx, pnum, ctx.freemove.walk_anim);
            if ctx.freemove.walk_anim != Some(dir) || !walking_anim {
                let p = &ctx.players.Players[pnum];
                let frame = if walking_anim { p.AnimInfo.currentFrame } else { 0 };
                if std::env::var_os("DIABLO_FREEMOVE_TRACE").is_some() {
                    eprintln!("FREEANIM walk dir={:?} was={:?} walking_anim={} frame={}", dir, ctx.freemove.walk_anim, walking_anim, frame);
                }
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
            let walking = step.is_some() && !ctx.freemove.waypoints.is_empty() || ctx.freemove.stick.is_some();
            ctx.freemove.idle_ticks = ctx.freemove.idle_ticks.saturating_add(1);
            if !walking || ctx.freemove.idle_ticks > STOP_GRACE {
                if !walking {
                    ctx.freemove.goal = None;
                    ctx.freemove.waypoints.clear();
                }
                if ctx.freemove.walk_anim.take().is_some() {
                    let d = ctx.players.Players[pnum]._pdir;
                    crate::player::new_plr_anim(ctx, pnum, player_graphic::Stand, d, AnimationDistributionFlags::None, 0, 0);
                }
                ctx.freemove.heading = (0.0, 0.0);
            }
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
        eprintln!("FREEMOVE pos=({:.2},{:.2}) tile=({},{}) wp={:?} cursor={:?} curs=({},{}) da={} mode={} face={:?}", s.pos.0, s.pos.1, s.tile.x, s.tile.y, s.waypoints.first(), s.cursor_point, ctx.cursor.cursPosition.x, ctx.cursor.cursPosition.y, ctx.players.Players[pnum].destAction, ctx.players.Players[pnum]._pmode, s.walk_anim);
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
