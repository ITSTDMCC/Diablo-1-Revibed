//! First-person view (part of the `free-movement` build, not in the original): `X` switches the
//! dungeon view to a first-person view from the hero's eyes, drawn with a column raycaster.
//!
//! The level's own art is reused: every level piece is drawn once, as the isometric renderer
//! draws it, into an offscreen picture, and a point of the 3D world (on the floor or on a wall
//! face, at some height) takes the colour the isometric picture shows at that point's
//! projection. Monsters, NPCs, items, objects and missiles are upright billboards using the
//! sprite of the direction they face relative to the viewer. The game's light tables shade
//! everything by the tile's light level, as in the isometric view.
//!
//! Controls: W/S walk forward/back, A/D turn (with Shift: step sideways). The mouse aims: what is
//! under the cursor in the first-person view is the target of clicks, so attacking, picking up,
//! talking and casting work as usual. The panels, inventory and menus are unchanged.
//! The game logic is untouched; only the picture and the movement input change.

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSprite;
use crate::engine::geometry::{Direction, Displacement, Point};
use crate::engine::render::clx_render::{clx_draw, clx_draw_trn};
use crate::engine::render::dun_render::{render_tile, LevelCelBlock};
use crate::engine::surface::{OwnedSurface, Surface};
use crate::enums::*;
use crate::levels::gendung::{in_dungeon_bounds, DungeonType, MAXDUNX, MAXDUNY};
use crate::lighting::LightsMax;
use std::rc::Rc;

/// Screen pixels of the isometric art per tile, horizontally (64 px is a tile's diagonal) and
/// vertically.
const PX_PER_TILE: f32 = 45.25;
/// Eye height in tiles (the hero sprite is about 2 tiles tall).
const EYE: f32 = 1.0;
/// Horizontal field of view.
const FOV_DEG: f32 = 100.0;
/// How far the view reaches, in tiles.
const MAX_DIST: f32 = 40.0;
/// Where the horizon sits in the part of the view the control panel leaves visible (0 = top).
const HORIZON: f32 = 0.45;
/// The hero's own sprite at the bottom of the view (weapon, shield, spells): its size relative
/// to the visible height, and which part of the sprite sits on the bottom edge (pixels above the
/// sprite's ground point; the hero is about 75 pixels tall).
const HERO_VIEW_SCALE: f32 = 1.0 / 80.0;
const HERO_VIEW_CUT: f32 = 26.0;
/// The hero stands a little right of the middle (looking over the left shoulder), so the middle
/// of the view stays clear.
const HERO_VIEW_X: f32 = 0.58;
/// How far around the mouse a click still finds a monster, item or object (pixels).
const PICK_SLACK: i32 = 10;
/// Turning speed, degrees per second.
const TURN_DEG_PER_S: f32 = 150.0;
/// How far behind the hero the eye is, at most (tiles).
const CAMERA_BACK: f32 = 0.8;
/// The toggle key.
const KEY_TOGGLE: i32 = b'x' as i32;

struct PieceTex {
    h: i32,
    /// A wall stands on the tile's x - 0.5 edge (left half of the picture) / y - 0.5 edge.
    left_wall: bool,
    right_wall: bool,
    /// How high the face of that wall goes (tiles): above it the picture shows the wall's top,
    /// which the eye (below the top) cannot see.
    left_top: f32,
    right_top: f32,
    /// A free-standing thing (pillar, lamp post) rather than a wall: the columns its art takes
    /// and the picture row it stands on.
    prop: Option<(i32, i32, i32)>,
    /// A door's picture: the picture columns showing the door, spread over the door's edge.
    door_span: Option<(f32, f32)>,
    px: Vec<u8>,
    opaque: Vec<bool>,
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Monster,
    Item,
    Object,
    Other,
}

struct Billboard {
    pos: (f32, f32),
    depth: f32,
    sprite: Option<ClxSprite>,
    /// A level piece standing upright instead of the sprite (pillar, lamp post).
    prop: Option<Rc<PieceTex>>,
    trn: Option<Rc<[u8; 256]>>,
    /// Light table index, or `None` to draw the colours as they are.
    light: Option<u8>,
    tile: Point,
    kind: Kind,
}

#[derive(Default)]
pub struct FirstPersonState {
    on: bool,
    /// View direction in tile space, radians.
    yaw: f32,
    keys: [bool; 5], // W A S D Shift
    drove_stick: bool,
    last_ms: Option<u32>,
    level_key: (u8, i32, usize),
    pieces: Vec<Option<Rc<PieceTex>>>,
    /// The level's most common wall pictures (left, right edge), for wall faces the original never
    /// drew (the sides of walls facing away from the isometric camera).
    fallback: Option<(Rc<PieceTex>, bool)>,
    /// Door pictures with the door drawn in, by piece and door frame.
    doors: Vec<((usize, u32), Rc<PieceTex>)>,
    /// The usual height of the level's walls (tiles): no wall face is drawn higher (above it the
    /// pictures show the raised tops of the rock behind the wall).
    wall_height: f32,
    // the last drawn frame, for aiming
    cam: (f32, f32),
    w: i32,
    h: i32,
    horizon: f32,
    zbuf: Vec<f32>,
    pick: Vec<u16>,
    picked: Vec<(Point, Kind)>,
}

/// The first-person view is on.
pub fn active(ctx: &Ctx) -> bool {
    ctx.firstperson.on && ctx.players.MyPlayer.is_some_and(|me| crate::freemove::active_for(ctx, me))
}

fn key_index(vkey: i32) -> Option<usize> {
    use crate::platform::events::keys::*;
    match vkey {
        k if k == b'w' as i32 || k == b'W' as i32 => Some(0),
        k if k == b'a' as i32 || k == b'A' as i32 => Some(1),
        k if k == b's' as i32 || k == b'S' as i32 => Some(2),
        k if k == b'd' as i32 || k == b'D' as i32 => Some(3),
        k if k == SDLK_LSHIFT || k == SDLK_RSHIFT => Some(4),
        _ => None,
    }
}

fn dir_angle(d: Direction) -> f32 {
    let v = Displacement::from_direction(d);
    (v.delta_y as f32).atan2(v.delta_x as f32)
}

/// `PressKey`: returns true when the key was used here.
pub fn press_key(ctx: &mut Ctx, vkey: i32) -> bool {
    if !crate::freemove::ENABLED || ctx.control.talkflag {
        return false;
    }
    let me = match ctx.players.MyPlayer {
        Some(me) if crate::freemove::active_for(ctx, me) => me,
        _ => return false,
    };
    if vkey == KEY_TOGGLE || vkey == b'X' as i32 {
        let s = &mut ctx.firstperson;
        s.on = !s.on;
        s.keys = [false; 5];
        if s.on {
            s.yaw = dir_angle(ctx.players.Players[me]._pdir);
            if let Some(y) = test_door_view(ctx, me) {
                ctx.firstperson.yaw = y;
            } else if let Some(y) = test_yaw(ctx, me) {
                ctx.firstperson.yaw = y;
            }
            ctx.firstperson.last_ms = None;
        }
        crate::control::calculate_panel_areas(ctx);
        crate::engine::backbuffer_state::redraw_everything(ctx);
        return true;
    }
    if let Some(i) = key_index(vkey) {
        ctx.firstperson.keys[i] = true;
        return ctx.firstperson.on && i < 4;
    }
    false
}

/// Test hook `DIABLO_FP_YAW`: a view direction in degrees, or `monster` / `door` to face the
/// nearest monster (or NPC in town) / door in plain sight.
fn test_yaw(ctx: &mut Ctx, me: usize) -> Option<f32> {
    let v = std::env::var("DIABLO_FP_YAW").ok()?;
    if v != "monster" && v != "door" {
        return v.parse::<f32>().ok().map(f32::to_radians);
    }
    let at = ctx.players.Players[me].position.tile;
    let mut targets: Vec<Point> = if v == "door" {
        ctx.objects.Objects.iter().filter(|o| o.is_door()).map(|o| o.position).collect()
    } else {
        (0..ctx.monster.ActiveMonsterCount).map(|i| ctx.monster.Monsters[ctx.monster.ActiveMonsters[i] as usize].position.tile).collect()
    };
    if ctx.gendung.leveltype == DungeonType::Town {
        targets.extend(ctx.towners.towners.iter().map(|t| t.position));
    }
    targets.sort_by_key(|t| (t.x - at.x).pow(2) + (t.y - at.y).pow(2));
    let mut pieces = std::mem::take(&mut ctx.firstperson.pieces);
    let mut found = None;
    for t in targets {
        let d = (((t.x - at.x).pow(2) + (t.y - at.y).pow(2)) as f32).sqrt();
        if d < 1.5 {
            continue;
        }
        let dir = ((t.x - at.x) as f32 / d, (t.y - at.y) as f32 / d);
        if first_wall(ctx, &mut pieces, (at.x as f32, at.y as f32), dir, d) >= d - if v == "door" { 1.0 } else { 0.0 } {
            eprintln!("FP facing {:?} from {:?}", t, at);
            found = Some(dir.1.atan2(dir.0));
            break;
        }
    }
    ctx.firstperson.pieces = pieces;
    found
}

/// Test hook `DIABLO_FP_DOOR=front|back`: moves the hero two tiles in front of (or behind) the
/// first door on the level, facing it.
fn test_door_view(ctx: &mut Ctx, me: usize) -> Option<f32> {
    let side = std::env::var("DIABLO_FP_DOOR").ok()?;
    let sign = if side == "back" { -1 } else { 1 };
    let mut pieces = std::mem::take(&mut ctx.firstperson.pieces);
    let mut found = None;
    for o in ctx.objects.Objects.iter().filter(|o| o.is_door()) {
        let d = o.position;
        let piece = ctx.gendung.dPiece[d.x as usize][d.y as usize] as usize;
        let Some(_) = piece_tex(ctx, &mut pieces, piece) else { continue };
        // approach across the gap in the wall
        let across_x = blocks(ctx, d + Displacement::new(0, 1)) && blocks(ctx, d + Displacement::new(0, -1));
        let step = if across_x { Point::new(sign, 0) } else { Point::new(0, sign) };
        let at = Point::new(d.x + 2 * step.x, d.y + 2 * step.y);
        if in_dungeon_bounds(at) && !blocks(ctx, at) && ctx.gendung.dMonster[at.x as usize][at.y as usize] == 0 {
            found = Some((at, d));
            break;
        }
    }
    ctx.firstperson.pieces = pieces;
    let (at, d) = found?;
    let old = ctx.players.Players[me].position.tile;
    ctx.gendung.dPlayer[old.x as usize][old.y as usize] = 0;
    ctx.players.Players[me].position.tile = at;
    ctx.gendung.dPlayer[at.x as usize][at.y as usize] = me as i8 + 1;
    let dir = ctx.players.Players[me]._pdir;
    crate::player::fix_player_location(ctx, me, dir);
    eprintln!("FP door at {:?}, hero at {:?}, state {} frame {} type {}", d, at, ctx.objects.Objects.iter().find(|o| o.position == d).map_or(-1, |o| o._oVar4), ctx.objects.Objects.iter().find(|o| o.position == d).map_or(0, |o| o._oAnimFrame), ctx.objects.Objects.iter().find(|o| o.position == d).map_or(0, |o| o._otype as i32));
    Some(((d.y - at.y) as f32).atan2((d.x - at.x) as f32))
}

/// `ReleaseKey`.
pub fn release_key(ctx: &mut Ctx, vkey: i32) -> bool {
    if let Some(i) = key_index(vkey) {
        ctx.firstperson.keys[i] = false;
        return ctx.firstperson.on && i < 4;
    }
    false
}

/// The walking direction from W/A/S/D in tile space, if any is held (called each game tick by
/// the free movement before it moves the player).
pub fn movement(ctx: &mut Ctx) -> Option<(f32, f32)> {
    if !active(ctx) {
        ctx.firstperson.drove_stick = false;
        return None;
    }
    if let (Some(d), Some(me)) = (view_direction(ctx), ctx.players.MyPlayer) {
        let p = &ctx.players.Players[me];
        if p._pmode == PM_STAND && p._pdir != d && crate::freemove::is_idle(ctx) {
            crate::player::new_plr_anim(ctx, me, player_graphic::Stand, d, AnimationDistributionFlags::None, 0, 0);
        }
    }
    let k = ctx.firstperson.keys;
    let (f, r) = (ctx.firstperson.yaw.cos(), ctx.firstperson.yaw.sin());
    let fwd = (f, r);
    let right = (-r, f);
    let mut v = (0.0f32, 0.0f32);
    let mut add = |d: (f32, f32), s: f32| {
        v.0 += d.0 * s;
        v.1 += d.1 * s;
    };
    if k[0] {
        add(fwd, 1.0);
    }
    if k[2] {
        add(fwd, -1.0);
    }
    if k[4] {
        if k[1] {
            add(right, -1.0);
        }
        if k[3] {
            add(right, 1.0);
        }
    }
    let len = (v.0 * v.0 + v.1 * v.1).sqrt();
    if len < 1e-3 {
        let was = ctx.firstperson.drove_stick;
        ctx.firstperson.drove_stick = false;
        // keys released: stop the movement they started
        return if was { Some((0.0, 0.0)) } else { None };
    }
    ctx.firstperson.drove_stick = true;
    Some((v.0 / len, v.1 / len))
}

fn update_turning(ctx: &mut Ctx) {
    let now = ctx.platform.ticks();
    let s = &mut ctx.firstperson;
    let dt = s.last_ms.map(|l| now.wrapping_sub(l) as f32 / 1000.0).unwrap_or(0.0).min(0.1);
    s.last_ms = Some(now);
    if s.keys[4] {
        return; // Shift: A/D step sideways
    }
    let turn = TURN_DEG_PER_S.to_radians() * dt;
    if s.keys[1] {
        s.yaw -= turn;
    }
    if s.keys[3] {
        s.yaw += turn;
    }
    // controller: the right stick turns
    s.yaw += ctx.controls.sticks.right_stick_x * turn;
}

/// The view direction as one of the eight directions, while the first-person view is on (the
/// hero faces it, so the controller's automatic targeting picks what is in view).
pub fn view_direction(ctx: &Ctx) -> Option<Direction> {
    if !active(ctx) {
        return None;
    }
    let v = (ctx.firstperson.yaw.cos(), ctx.firstperson.yaw.sin());
    DIRS.iter().copied().max_by(|&a, &b| {
        let (da, db) = (Displacement::from_direction(a), Displacement::from_direction(b));
        let la = (da.delta_x as f32 * v.0 + da.delta_y as f32 * v.1) / da.magnitude();
        let lb = (db.delta_x as f32 * v.0 + db.delta_y as f32 * v.1) / db.magnitude();
        la.partial_cmp(&lb).unwrap_or(std::cmp::Ordering::Equal)
    })
}

/// W/A/S/D walking is in progress (the controller's stick leaves it alone).
pub fn keys_held(ctx: &Ctx) -> bool {
    ctx.firstperson.drove_stick
}

/// The controller's left stick in the first-person view: up walks forward, sideways steps
/// sideways. `x`/`y` are the stick (x right, y up); returns the walking direction in tiles.
pub fn stick_direction(ctx: &Ctx, x: f32, y: f32) -> Option<(f32, f32)> {
    if !active(ctx) {
        return None;
    }
    let f = (ctx.firstperson.yaw.cos(), ctx.firstperson.yaw.sin());
    let r = (-f.1, f.0);
    Some((f.0 * y + r.0 * x, f.1 * y + r.1 * x))
}

fn piece_tex(ctx: &Ctx, cache: &mut Vec<Option<Rc<PieceTex>>>, piece: usize) -> Option<Rc<PieceTex>> {
    if piece >= cache.len() {
        cache.resize(piece + 1, None);
    }
    if let Some(t) = &cache[piece] {
        return Some(t.clone());
    }
    let t = Rc::new(render_piece(ctx, piece, None)?);
    cache[piece] = Some(t.clone());
    Some(t)
}

/// Renders level piece `piece` as the isometric view draws it, with `overlay` (a sprite drawn
/// at the tile, like a door) on top.
fn render_piece(ctx: &Ctx, piece: usize, overlay: Option<&ClxSprite>) -> Option<PieceTex> {
    let micros = ctx.gendung.DPieceMicros.get(piece)?;
    let n = ctx.gendung.MicroTileLen as usize;
    let h = (n as i32 / 2) * 32;
    // drawn twice, on black and on white: the pixels that differ are not part of the piece
    let mut a = OwnedSurface::new(64, h);
    let mut b = OwnedSurface::new(64, h);
    b.pixels.iter_mut().for_each(|p| *p = 255);
    for surf in [&mut a, &mut b] {
        let out = surf.view();
        let mut y = h - 1;
        let mut i = 0;
        while i + 1 < n.max(2) {
            for half in 0..2 {
                let block = LevelCelBlock(micros.mt[i + half]);
                if block.has_value() {
                    render_tile(ctx, &out, Point::new(half as i32 * 32, y), block, MaskType::Solid, 0);
                }
            }
            y -= 32;
            i += 2;
        }
        if let Some(sprite) = overlay {
            // where `DrawObject` puts it: centred on the tile, standing on its bottom row
            let x = -crate::engine::calculate_width2(sprite.width() as i32);
            clx_draw(&out, (x, h - 1), sprite);
        }
    }
    let (av, bv) = (a.view(), b.view());
    let mut px = vec![0u8; 64 * h as usize];
    let mut opaque = vec![false; 64 * h as usize];
    for y in 0..h {
        for x in 0..64 {
            let (ca, cb) = (av.get(x, y), bv.get(x, y));
            let i = (y * 64 + x) as usize;
            px[i] = ca;
            opaque[i] = ca == cb;
        }
    }
    if let Some(dir) = std::env::var_os("DIABLO_FP_DUMP") {
        // debugging: the piece picture as PPM (magenta = transparent)
        let mut data = format!("P6 64 {h} 255
").into_bytes();
        for i in 0..px.len() {
            let c = if opaque[i] { ctx.dx.pal.logical_palette[px[i] as usize] } else { [255, 0, 255] };
            data.extend_from_slice(&c);
        }
        let _ = std::fs::write(std::path::Path::new(&dir).join(format!("piece_{piece}.ppm")), data);
    }
    let mut t = PieceTex { h, left_wall: false, right_wall: false, left_top: 0.0, right_top: 0.0, prop: None, door_span: None, px, opaque };
    (t.left_wall, t.left_top) = wall_face(&t, true);
    (t.right_wall, t.right_top) = wall_face(&t, false);
    if !t.left_wall && !t.right_wall {
        t.prop = prop_shape(&t);
    }
    Some(t)
}

/// The picture of a door's tile with the door itself (an object sprite) drawn in, as the
/// isometric view shows it, for the door's current frame.
fn door_tex(ctx: &Ctx, cache: &mut Vec<((usize, u32), Rc<PieceTex>)>, t: Point) -> Option<Rc<PieceTex>> {
    let piece = ctx.gendung.dPiece[t.x as usize][t.y as usize] as usize;
    let o = ctx.gendung.dObject[t.x as usize][t.y as usize];
    let obj = &ctx.objects.Objects[(o.unsigned_abs() - 1) as usize];
    let key = (piece, obj._oAnimFrame);
    if let Some((_, tex)) = cache.iter().find(|(k, _)| *k == key) {
        return Some(tex.clone());
    }
    let sprite = obj._oAnimData.as_ref().map(|a| a.get((obj._oAnimFrame.max(1) - 1) as usize));
    let mut tex = render_piece(ctx, piece, sprite.as_ref())?;
    if let Some(dir) = std::env::var_os("DIABLO_FP_DUMP") {
        let mut data = format!("P6 64 {} 255
", tex.h).into_bytes();
        for i in 0..tex.px.len() {
            let c = if tex.opaque[i] { ctx.dx.pal.logical_palette[tex.px[i] as usize] } else { [255, 0, 255] };
            data.extend_from_slice(&c);
        }
        let _ = std::fs::write(std::path::Path::new(&dir).join(format!("door_{piece}_{}_{}x{}.ppm", obj._oAnimFrame, sprite.as_ref().map_or(0, |s| s.width()), sprite.as_ref().map_or(0, |s| s.height()))), data);
    }
    // The door stands across the gap in the wall: the walls on either side of it tell which
    // edge it is on (the closed door of some levels is drawn on the other half of the picture
    // than the doorway's arch).
    let across_x = blocks(ctx, t + Displacement::new(0, 1)) && blocks(ctx, t + Displacement::new(0, -1));
    let across_y = blocks(ctx, t + Displacement::new(1, 0)) && blocks(ctx, t + Displacement::new(-1, 0));
    if across_x != across_y {
        // only that edge's half of the picture counts as the door
        (tex.left_wall, tex.right_wall) = (across_x, across_y);
        let top = tex.left_top.max(tex.right_top);
        (tex.left_top, tex.right_top) = (top, top);
        // The door leaf continues the wall plane past the tile's corner in the art (the closed
        // door covers the gap and a little more): the columns of the picture that show the door
        // on its plane at door height are spread over the tile's edge.
        let mut cols = (64, -1);
        for sx in 0..64 {
            let (fx, fy) = plane_point(across_x, sx as f32 + 0.5);
            if (0..8).any(|k| wall_pixel(&tex, fx, fy, 0.3 + k as f32 * 0.12).is_some()) {
                cols = (cols.0.min(sx), cols.1.max(sx));
            }
        }
        // Where the doorway itself is only a dark opening (the door leaf is all sprite, drawn
        // beside it in the art), only the sprite's columns are the door.
        let plain = render_piece(ctx, piece, None)?;
        let pal = &ctx.dx.pal.logical_palette;
        let (mut dark, mut all) = (0, 0);
        for i in 0..8 {
            let along = -0.3 + 0.6 * i as f32 / 7.0;
            let (fx, fy) = if across_x { (-0.5, along) } else { (along, -0.5) };
            for k in 0..6 {
                if let Some(px) = wall_pixel(&plain, fx, fy, 0.3 + k as f32 * 0.15) {
                    all += 1;
                    let c = pal[plain.px[px] as usize];
                    if (c[0] as u32 + c[1] as u32 + c[2] as u32) < 75 {
                        dark += 1;
                    }
                }
            }
        }
        if all > 0 && dark * 3 > all {
            let mut sprite_cols = (64, -1);
            for sx in 0..64 {
                if (0..tex.h).any(|sy| {
                    let i = (sy * 64 + sx) as usize;
                    tex.opaque[i] && (!plain.opaque[i] || plain.px[i] != tex.px[i])
                }) {
                    sprite_cols = (sprite_cols.0.min(sx), sprite_cols.1.max(sx));
                }
            }
            if sprite_cols.1 > sprite_cols.0 {
                cols = sprite_cols;
            }
        }
        if cols.1 > cols.0 {
            tex.door_span = Some((cols.0 as f32, cols.1 as f32 + 1.0));
        }
    }
    let tex = Rc::new(tex);
    cache.push((key, tex.clone()));
    Some(tex)
}

/// Wall art on a tile the hero can walk through is an archway or an open doorway: its opening
/// is painted dark in the isometric art, so only the part above this height (tiles) is drawn.
const ARCH_TOP: f32 = 1.25;

/// The tile blocks the way like a wall (a solid tile, or a closed door).
fn blocks(ctx: &Ctx, t: Point) -> bool {
    if crate::engine::path::is_tile_solid(ctx, t) {
        return true;
    }
    let o = ctx.gendung.dObject[t.x as usize][t.y as usize];
    if o != 0 {
        let obj = &ctx.objects.Objects[(o.unsigned_abs() - 1) as usize];
        if obj.is_door() && obj._oVar4 == 0 {
            return true;
        }
    }
    false
}

/// The wall on the left (x - 0.5) or right (y - 0.5) edge of a piece, if its picture has one:
/// art standing on that edge from the floor up (not only higher up, like the tops of the solid
/// rock between rooms, which the pictures show raised to wall height), and the height of its
/// face below the wall's top.
fn wall_face(tex: &PieceTex, left: bool) -> (bool, f32) {
    const SAMPLES: usize = 16;
    /// The wall's top as the pictures show it, in picture rows.
    const CAP_ROWS: f32 = 16.0;
    // standing on the floor: some art low down (solid walls, arches, the bars of a grate)
    let mut low = 0;
    // and spanning the edge: art somewhere between knee and head height along most of it
    let mut spans = 0;
    let mut tops = Vec::with_capacity(SAMPLES);
    for i in 0..SAMPLES {
        let along = -0.45 + 0.9 * i as f32 / (SAMPLES - 1) as f32;
        let (fx, fy) = if left { (-0.5, along) } else { (along, -0.5) };
        if wall_pixel(tex, fx, fy, 0.25).is_some() {
            low += 1;
        }
        if (0..=20).any(|k| wall_pixel(tex, fx, fy, 0.3 + k as f32 * 0.05).is_some()) {
            spans += 1;
        }
        // the highest art on the edge
        let mut top = 0.0;
        let mut hgt = 0.2;
        while hgt < 5.0 {
            if wall_pixel(tex, fx, fy, hgt).is_some() {
                top = hgt;
            }
            hgt += 1.0 / PX_PER_TILE;
        }
        tops.push(top);
    }
    tops.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let top = (tops[SAMPLES / 2] - CAP_ROWS / PX_PER_TILE).max(0.5);
    (low >= 3 && spans * 4 >= SAMPLES * 3, top)
}

/// A narrow thing standing above the floor (pillar, lamp post): the columns its art takes above
/// the floor diamond and the lowest row of those columns (where it stands). Wide art above the
/// floor (the raised tops of the rock between rooms) is not one.
fn prop_shape(tex: &PieceTex) -> Option<(i32, i32, i32)> {
    let h = tex.h;
    let (mut x0, mut x1, mut n) = (64, -1, 0);
    for y in 0..h - 40 {
        for x in 0..64 {
            if tex.opaque[(y * 64 + x) as usize] {
                x0 = x0.min(x);
                x1 = x1.max(x);
                n += 1;
            }
        }
    }
    if n < 40 || x1 - x0 > 40 {
        return None;
    }
    // it must reach down to the floor diamond (the raised rock tops float above it)
    let reaches_floor = (h - 40..h - 16).any(|y| (x0..=x1).any(|x| tex.opaque[(y * 64 + x) as usize]));
    if !reaches_floor {
        return None;
    }
    let base = (h - 40..h).rev().find(|&y| (x0..=x1).any(|x| tex.opaque[(y * 64 + x) as usize])).unwrap_or(h - 16);
    Some((x0, x1, base.min(h - 16)))
}

/// A door (open or closed) stands on the tile.
fn door_at(ctx: &Ctx, t: Point) -> bool {
    if !in_dungeon_bounds(t) {
        return false;
    }
    let o = ctx.gendung.dObject[t.x as usize][t.y as usize];
    o != 0 && ctx.objects.Objects[(o.unsigned_abs() - 1) as usize].is_door()
}

/// The point of the left (x - 0.5) or right (y - 0.5) wall plane that picture column `sx` shows
/// (the plane continued past the tile where `sx` is in the other half).
fn plane_point(left: bool, sx: f32) -> (f32, f32) {
    if left { (-0.5, -0.5 - (sx - 32.0) / 32.0) } else { ((sx - 32.0) / 32.0 - 0.5, -0.5) }
}

/// The picture's pixel showing the point of a wall face at (`fx`, `fy`) from the tile centre and
/// `height` tiles above the floor, if the picture has one there.
fn wall_pixel(tex: &PieceTex, fx: f32, fy: f32, height: f32) -> Option<usize> {
    let sx = (32.0 + (fx - fy) * 32.0).clamp(0.0, 63.0) as i32;
    let sy = ((tex.h - 16) as f32 + (fx + fy) * 16.0 - height * PX_PER_TILE) as i32;
    if sy < 0 || sy >= tex.h {
        return None;
    }
    let i = (sy * 64 + sx) as usize;
    tex.opaque[i].then_some(i)
}

/// The distance along `dir` (unit length) to the first wall from `from`, up to `max`. Only walls
/// solid at about waist height count (an archway does not).

fn first_wall(ctx: &Ctx, pieces: &mut Vec<Option<Rc<PieceTex>>>, from: (f32, f32), dir: (f32, f32), max: f32) -> f32 {
    let q = (from.0 + 0.5, from.1 + 0.5);
    let mut cell = (q.0.floor() as i32, q.1.floor() as i32);
    let delta = (if dir.0 == 0.0 { f32::MAX } else { (1.0 / dir.0).abs() }, if dir.1 == 0.0 { f32::MAX } else { (1.0 / dir.1).abs() });
    let step = (if dir.0 < 0.0 { -1 } else { 1 }, if dir.1 < 0.0 { -1 } else { 1 });
    let mut side = (
        if dir.0 < 0.0 { (q.0 - cell.0 as f32) * delta.0 } else { (cell.0 as f32 + 1.0 - q.0) * delta.0 },
        if dir.1 < 0.0 { (q.1 - cell.1 as f32) * delta.1 } else { (cell.1 as f32 + 1.0 - q.1) * delta.1 },
    );
    loop {
        let from_t = Point::new(cell.0, cell.1);
        let (dist, xside) = if side.0 < side.1 {
            cell.0 += step.0;
            let d = side.0;
            side.0 += delta.0;
            (d, true)
        } else {
            cell.1 += step.1;
            let d = side.1;
            side.1 += delta.1;
            (d, false)
        };
        let to = Point::new(cell.0, cell.1);
        if dist > max {
            return max;
        }
        if !in_dungeon_bounds(to) {
            return dist;
        }
        let owner = if xside {
            if step.0 > 0 { to } else { from_t }
        } else if step.1 > 0 {
            to
        } else {
            from_t
        };
        let piece = ctx.gendung.dPiece[owner.x as usize][owner.y as usize] as usize;
        if let Some(tex) = piece_tex(ctx, pieces, piece) {
            if ((xside && tex.left_wall) || (!xside && tex.right_wall)) && blocks(ctx, owner) {
                let hp = (from.0 + dir.0 * dist, from.1 + dir.1 * dist);
                let (fx, fy) = if xside { (-0.5, hp.1 - owner.y as f32) } else { (hp.0 - owner.x as f32, -0.5) };
                if wall_pixel(&tex, fx, fy, 0.6).is_some() && wall_pixel(&tex, fx, fy, 1.2).is_some() {
                    return dist;
                }
            }
        }
    }
}

/// How close to a wall the hero may walk in the first-person view (the tile rules alone let it
/// walk right up to the walls, which stand on tile edges).
const WALL_MARGIN: f32 = 0.3;

/// A move from `from` to `to` would bring the hero closer to a wall than `WALL_MARGIN`.
pub fn too_close_to_wall(ctx: &mut Ctx, from: (f32, f32), to: (f32, f32)) -> bool {
    if !active(ctx) {
        return false;
    }
    let d = (to.0 - from.0, to.1 - from.1);
    let len = (d.0 * d.0 + d.1 * d.1).sqrt();
    if len < 1e-5 {
        return false;
    }
    let dir = (d.0 / len, d.1 / len);
    let mut pieces = std::mem::take(&mut ctx.firstperson.pieces);
    let hit = first_wall(ctx, &mut pieces, from, dir, len + WALL_MARGIN);
    ctx.firstperson.pieces = pieces;
    hit < len + WALL_MARGIN
}

/// The level's plain wall: the plainest of its common wall pictures, and whether its wall is on
/// the left (x - 0.5) edge.
fn fallback_walls(ctx: &Ctx, pieces: &mut Vec<Option<Rc<PieceTex>>>) -> Option<(Rc<PieceTex>, bool)> {
    let mut counts: [std::collections::HashMap<usize, usize>; 2] = Default::default();
    for x in 0..MAXDUNX {
        for y in 0..MAXDUNY {
            let t = Point::new(x as i32, y as i32);
            if !blocks(ctx, t) {
                continue;
            }
            let piece = ctx.gendung.dPiece[x][y] as usize;
            if let Some(tex) = piece_tex(ctx, pieces, piece) {
                // only plain walls: solid along the whole edge (no arches or gaps)
                let solid = |left: bool| {
                    (0..16).all(|i| {
                        let along = -0.45 + 0.9 * i as f32 / 15.0;
                        let (fx, fy) = if left { (-0.5, along) } else { (along, -0.5) };
                        [0.2, 0.6, 1.0].iter().all(|&hgt| wall_pixel(&tex, fx, fy, hgt).is_some())
                    })
                };
                if tex.left_wall && solid(true) {
                    *counts[0].entry(piece).or_default() += 1;
                }
                if tex.right_wall && solid(false) {
                    *counts[1].entry(piece).or_default() += 1;
                }
            }
        }
    }
    // the plainest of the common ones: fewest dark pixels on its face (arches and doorways are
    // painted dark), so it does not repeat a feature over and over where it stands in
    let dark_share = |tex: &PieceTex, left: bool| {
        let pal = &ctx.dx.pal.logical_palette;
        let (mut dark, mut all) = (0, 0);
        for i in 0..16 {
            let along = -0.45 + 0.9 * i as f32 / 15.0;
            let (fx, fy) = if left { (-0.5, along) } else { (along, -0.5) };
            let top = if left { tex.left_top } else { tex.right_top };
            let mut hgt = 0.1;
            while hgt < top {
                if let Some(px) = wall_pixel(tex, fx, fy, hgt) {
                    let c = pal[tex.px[px] as usize];
                    all += 1;
                    if (c[0] as u32 + c[1] as u32 + c[2] as u32) < 90 {
                        dark += 1;
                    }
                }
                hgt += 0.05;
            }
        }
        if all == 0 { 1.0 } else { dark as f32 / all as f32 }
    };
    let mut pick = |c: &std::collections::HashMap<usize, usize>, left: bool| {
        let most = c.values().copied().max().unwrap_or(0);
        let mut best: Option<(f32, usize, usize)> = None;
        for (&piece, &n) in c {
            if n * 4 < most {
                continue; // a rare one
            }
            let tex = piece_tex(ctx, pieces, piece)?;
            let d = (dark_share(&tex, left) * 20.0).round();
            if best.is_none_or(|b| (d, std::cmp::Reverse(n), piece) < (b.0, std::cmp::Reverse(b.1), b.2)) {
                best = Some((d, n, piece));
            }
        }
        let (d, _, piece) = best?;
        Some((d, piece_tex(ctx, pieces, piece)?))
    };
    let l = pick(&counts[0], true);
    let r = pick(&counts[1], false);
    let (tex, left) = match (l, r) {
        (Some(l), Some(r)) => if r.0 < l.0 { (r.1, false) } else { (l.1, true) },
        (Some(l), None) => (l.1, true),
        (None, Some(r)) => (r.1, false),
        (None, None) => return None,
    };
    Some((Rc::new(plain_wall(ctx, &tex, left)), left))
}

/// The wall picture with its features (arch outlines, doorways) brushed out: the face is filled
/// by repeating its plainest band of brick (the rows with no dark pixels) above a thin plinth, so
/// a long run of it reads as one plain wall instead of the same feature over and over.
fn plain_wall(ctx: &Ctx, tex: &PieceTex, left: bool) -> PieceTex {
    const PLINTH: f32 = 0.12;
    let pal = &ctx.dx.pal.logical_palette;
    let top = if left { tex.left_top } else { tex.right_top };
    let at = |along: f32, hgt: f32| {
        let (fx, fy) = if left { (-0.5, along) } else { (along, -0.5) };
        wall_pixel(tex, fx, fy, hgt)
    };
    // rows (one picture row apart) with every sample along the edge present and not dark
    let step = 1.0 / PX_PER_TILE;
    let rows: Vec<bool> = (0..((top / step) as usize))
        .map(|k| {
            let hgt = k as f32 * step;
            (0..16).all(|i| {
                at(-0.45 + 0.9 * i as f32 / 15.0, hgt).is_some_and(|px| {
                    let c = pal[tex.px[px] as usize];
                    c[0] as u32 + c[1] as u32 + c[2] as u32 >= 90
                })
            })
        })
        .collect();
    // the longest run above the plinth
    let (mut best, mut start) = ((0, 0), None);
    for (k, &ok) in rows.iter().enumerate().chain(std::iter::once((rows.len(), &false))) {
        let hgt = k as f32 * step;
        match (ok && hgt >= PLINTH, start) {
            (true, None) => start = Some(k),
            (false, Some(s0)) => {
                if k - s0 > best.1 - best.0 {
                    best = (s0, k);
                }
                start = None;
            }
            _ => {}
        }
    }
    let mut out = PieceTex { h: tex.h, left_wall: tex.left_wall, right_wall: tex.right_wall, left_top: tex.left_top, right_top: tex.right_top, prop: None, door_span: None, px: tex.px.clone(), opaque: tex.opaque.clone() };
    if best.1 - best.0 < 8 {
        return out; // no plain band worth repeating
    }
    let (b0, b1) = (best.0 as f32 * step, best.1 as f32 * step);
    // and the stretch along the edge (a third of it) with the least going on in that band
    const WIDTH: f32 = 0.3;
    let lum = |along: f32, hgt: f32| at(along, hgt).map_or(0.0, |px| {
        let c = pal[tex.px[px] as usize];
        (c[0] as f32 + c[1] as f32 + c[2] as f32) / 3.0
    });
    let mut a0 = -0.5;
    let mut least = f32::MAX;
    for k in 0..=7 {
        let start = -0.5 + k as f32 * (1.0 - WIDTH) / 7.0;
        let samples: Vec<f32> = (0..6).flat_map(|i| (0..8).map(move |j| (start + WIDTH * i as f32 / 5.0, b0 + (b1 - b0) * j as f32 / 7.0))).map(|(a, hh)| lum(a, hh)).collect();
        let mean = samples.iter().sum::<f32>() / samples.len() as f32;
        let var = samples.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>();
        if var < least {
            least = var;
            a0 = start;
        }
    }
    let h = tex.h;
    for sy in 0..h {
        for sx in if left { 0..32 } else { 32..64 } {
            // the point of the face this pixel shows
            let (along, hgt) = if left {
                let fy = (32 - sx) as f32 / 32.0 - 0.5;
                (fy, ((h - 16) as f32 + (fy - 0.5) * 16.0 - sy as f32) / PX_PER_TILE)
            } else {
                let fx = (sx - 32) as f32 / 32.0 - 0.5;
                (fx, ((h - 16) as f32 + (fx - 0.5) * 16.0 - sy as f32) / PX_PER_TILE)
            };
            if hgt < PLINTH || hgt > top {
                continue;
            }
            let src = b0 + (hgt - PLINTH).rem_euclid(b1 - b0);
            let src_along = a0 + (along + 0.5).rem_euclid(WIDTH);
            if let Some(px) = at(src_along, src) {
                let i = (sy * 64 + sx) as usize;
                out.px[i] = tex.px[px];
                out.opaque[i] = true;
            }
        }
    }
    out
}

/// The tile is a free-standing pillar or post (drawn upright, not as walls).
fn is_prop(ctx: &Ctx, pieces: &mut Vec<Option<Rc<PieceTex>>>, t: Point) -> bool {
    let piece = ctx.gendung.dPiece[t.x as usize][t.y as usize] as usize;
    piece_tex(ctx, pieces, piece).is_some_and(|tex| tex.prop.is_some())
}

/// The camera position in tiles, between game ticks like the sprite.
fn camera_pos(ctx: &Ctx, me: usize) -> (f32, f32) {
    let tile = ctx.players.Players[me].position.tile;
    let o = crate::freemove::render_offset(ctx, me).unwrap_or_default();
    let (sx, sy) = (o.delta_x as f32, o.delta_y as f32);
    (tile.x as f32 + (sx / 32.0 + sy / 16.0) / 2.0, tile.y as f32 + (sy / 16.0 - sx / 32.0) / 2.0)
}

fn screen_to_world(s: (f32, f32)) -> (f32, f32) {
    ((s.0 / 32.0 + s.1 / 16.0) / 2.0, (s.1 / 16.0 - s.0 / 32.0) / 2.0)
}

fn light_at(ctx: &Ctx, t: Point) -> u8 {
    if !in_dungeon_bounds(t) {
        return LightsMax as u8;
    }
    ctx.gendung.dLight[t.x as usize][t.y as usize]
}

/// Darkening with distance where the level has no lighting of its own (town).
fn fog(ctx: &Ctx, d: f32) -> u8 {
    if ctx.gendung.leveltype != DungeonType::Town {
        return 0;
    }
    ((d - 9.0) * 0.5).clamp(0.0, 12.0) as u8
}

fn shade(ctx: &Ctx, c: u8, light: u8) -> u8 {
    let l = (light as usize).min(LightsMax as usize);
    if l == 0 {
        c
    } else {
        ctx.lighting.LightTables[l][c as usize]
    }
}

/// The sprite of monster `m` as seen from `view` (the direction from the viewer to it).
fn monster_sprite(ctx: &Ctx, m: usize, view: (f32, f32)) -> Option<ClxSprite> {
    let monster = &ctx.monster.Monsters[m];
    let cur = monster.animInfo.sprites.as_ref()?;
    let frame = monster.animInfo.get_frame_to_use_for_rendering(ctx.nthread.ProgressToNextGameTick) as usize;
    let shown = rotated(monster.direction, view);
    let anims = &crate::monster::monster_type(ctx, m).anims;
    let first = cur.get(0);
    for a in anims.iter() {
        for d in 0..8 {
            let dir = DIRS[d];
            if let Some(list) = a.sprites_for_direction(dir) {
                if list.get(0) == first {
                    return Some(a.sprites_for_direction(shown).unwrap_or(list).get(frame));
                }
            }
        }
    }
    Some(cur.get(frame))
}

const DIRS: [Direction; 8] = [
    Direction::South,
    Direction::SouthWest,
    Direction::West,
    Direction::NorthWest,
    Direction::North,
    Direction::NorthEast,
    Direction::East,
    Direction::SouthEast,
];

/// The isometric sprite direction that shows an actor facing `facing` to a viewer looking along
/// `view`: the isometric camera looks North, so the relative turn is applied to that.
fn rotated(facing: Direction, view: (f32, f32)) -> Direction {
    let Some(fi) = DIRS.iter().position(|&d| d == facing) else { return facing };
    let va = view.1.atan2(view.0);
    let mut best = 0;
    let mut best_diff = f32::MAX;
    for (i, &d) in DIRS.iter().enumerate() {
        let diff = ((dir_angle(d) - va).rem_euclid(std::f32::consts::TAU) + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        if diff.abs() < best_diff {
            best_diff = diff.abs();
            best = i;
        }
    }
    // the isometric camera looks North (index 4)
    DIRS[(fi + 8 + 4 - best) % 8]
}

fn collect_billboards(ctx: &mut Ctx, pieces: &[Option<Rc<PieceTex>>], cam: (f32, f32), fwd: (f32, f32), me: usize) -> Vec<Billboard> {
    use crate::engine::render::scrollrt::{get_offset_for_walking, update_missiles_renderer_data};
    update_missiles_renderer_data(ctx);
    let ctx = &*ctx;
    let mut out = Vec::new();
    let r = MAX_DIST.min(24.0) as i32;
    let (cx, cy) = (cam.0.round() as i32, cam.1.round() as i32);
    let infra = ctx.players.Players[me]._pInfraFlag;
    let town = ctx.gendung.leveltype == DungeonType::Town;
    let progress = ctx.nthread.ProgressToNextGameTick;
    let push = |out: &mut Vec<Billboard>, pos: (f32, f32), sprite: Option<ClxSprite>, trn: Option<Rc<[u8; 256]>>, light: Option<u8>, tile: Point, kind: Kind| {
        let Some(sprite) = sprite else { return };
        let d = ((pos.0 - cam.0) * fwd.0 + (pos.1 - cam.1) * fwd.1).max(0.0);
        if d < 0.15 {
            return;
        }
        out.push(Billboard { pos, depth: d, sprite: Some(sprite), prop: None, trn, light, tile, kind });
    };
    let mut props = Vec::new();
    for x in (cx - r).max(0)..=(cx + r).min(MAXDUNX as i32 - 1) {
        for y in (cy - r).max(0)..=(cy + r).min(MAXDUNY as i32 - 1) {
            let tile = Point::new(x, y);
            let tf = (x as f32, y as f32);
            let light = ctx.gendung.dLight[x as usize][y as usize];
            if light >= LightsMax as u8 && !town && !infra {
                continue;
            }
            // pillars and lamp posts of the level
            props.push(tile);
            // items
            let it = ctx.items.dItem[x as usize][y as usize];
            if it > 0 {
                let item = &ctx.items.Items[(it - 1) as usize];
                if item.AnimInfo.sprites.is_some() {
                    push(&mut out, tf, Some(item.AnimInfo.current_sprite(progress)), None, Some(light), tile, Kind::Item);
                }
            }
            // objects (the main tile of each)
            let o = ctx.gendung.dObject[x as usize][y as usize];
            if o > 0 {
                let oi = (o - 1) as usize;
                let obj = &ctx.objects.Objects[oi];
                if obj.position == tile && !(obj.is_door()) {
                    if let Some(anim) = &obj._oAnimData {
                        let sprite = anim.get((obj._oAnimFrame.max(1) - 1) as usize);
                        push(&mut out, tf, Some(sprite), None, obj.applyLighting.then_some(light), tile, Kind::Object);
                    }
                }
            }
            // monsters and NPCs (as `DrawMonsterHelper` finds them)
            let mut mi = ctx.gendung.dMonster[x as usize][y as usize] as i32;
            if mi != 0 {
                let negative = mi < 0;
                mi = mi.abs() - 1;
                if town {
                    if !negative {
                        if let Some(t) = ctx.towners.towners.get(mi as usize) {
                            if t.anim.is_some() || t.owned_anim.is_some() {
                                push(&mut out, tf, Some(t.current_sprite()), None, None, tile, Kind::Monster);
                            }
                        }
                    }
                } else if (mi as usize) < crate::monster::MaxMonsters && (crate::levels::gendung::is_tile_lit(ctx, tile) || infra) {
                    let monster = &ctx.monster.Monsters[mi as usize];
                    if monster.flags & MFLAG_HIDDEN as u32 == 0 && monster.animInfo.sprites.is_some() {
                        let mut offset = Displacement::default();
                        let mut skip = negative;
                        if crate::monster::is_walking(ctx, mi as usize) {
                            let left = monster.mode == MonsterMode::MoveSideways && monster.direction == Direction::West;
                            skip = negative != left;
                            offset = get_offset_for_walking(ctx, &monster.animInfo, monster.direction, false);
                            if left {
                                offset = offset - Displacement::new(64, 0);
                            }
                        }
                        if !skip {
                            let w = screen_to_world((offset.delta_x as f32, offset.delta_y as f32));
                            let pos = (tf.0 + w.0, tf.1 + w.1);
                            let view = (pos.0 - cam.0, pos.1 - cam.1);
                            let lit = crate::levels::gendung::is_tile_lit(ctx, tile);
                            let mut trn = if monster.is_unique() { monster.uniqueMonsterTRN.clone() } else { None };
                            if monster.mode == MonsterMode::Petrified {
                                trn = Some(Rc::new(*crate::engine::trn::get_stone_trn(ctx)));
                            }
                            let mut l = Some(light);
                            if !lit || (infra && light > 8) {
                                trn = Some(Rc::new(*crate::engine::trn::get_infravision_trn(ctx)));
                                l = None;
                            }
                            push(&mut out, pos, monster_sprite(ctx, mi as usize, view), trn, l, tile, Kind::Monster);
                        }
                    }
                }
            }
        }
    }
    // missiles
    for &(t, mi) in ctx.scrollrt.missiles_at_rendering_tile.iter() {
        if (t.x - cx).abs() > r || (t.y - cy).abs() > r || !in_dungeon_bounds(t) {
            continue;
        }
        let m = &ctx.missiles.Missiles[mi];
        if !m._miDrawFlag {
            continue;
        }
        let Some(anim) = &m._miAnimData else { continue };
        let sprite = anim.get((m._miAnimFrame.max(1) - 1) as usize);
        let off = m.position.offsetForRendering;
        let w = screen_to_world((off.delta_x as f32, off.delta_y as f32));
        let light = if m._miLightFlag { Some(ctx.gendung.dLight[t.x as usize][t.y as usize]) } else { None };
        let trn = if m._miUniqTrans != 0 { ctx.monster.Monsters[m._misource as usize].uniqueMonsterTRN.clone() } else { None };
        push(&mut out, (t.x as f32 + w.0, t.y as f32 + w.1), Some(sprite), trn, light, t, Kind::Other);
    }
    {
        for tile in props {
            let piece = ctx.gendung.dPiece[tile.x as usize][tile.y as usize] as usize;
            let Some(tex) = pieces.get(piece).cloned().flatten() else { continue };
            let Some((_, _, base)) = tex.prop else { continue };
            // where the picture's base stands on the floor
            let f = (base - (tex.h - 16)) as f32 / 32.0;
            let pos = (tile.x as f32 + f, tile.y as f32 + f);
            let d = (pos.0 - cam.0) * fwd.0 + (pos.1 - cam.1) * fwd.1;
            if d < 0.15 {
                continue;
            }
            let light = ctx.gendung.dLight[tile.x as usize][tile.y as usize];
            out.push(Billboard { pos, depth: d, sprite: None, prop: Some(tex), trn: None, light: Some(light), tile, kind: Kind::Other });
        }
    }
    out.sort_by(|a, b| b.depth.partial_cmp(&a.depth).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Draws the first-person view into `out` (the game view area) if it is on.
pub fn draw(ctx: &mut Ctx, out: &Surface) -> bool {
    if !active(ctx) {
        return false;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    update_turning(ctx);
    let key = (ctx.gendung.leveltype as u8, ctx.gendung.currlevel as i32, ctx.gendung.pDungeonCels.as_ref().map(|c| c.as_ptr() as usize).unwrap_or(0));
    if ctx.firstperson.level_key != key {
        ctx.firstperson.level_key = key;
        ctx.firstperson.pieces.clear();
        ctx.firstperson.doors.clear();
        let mut pieces = std::mem::take(&mut ctx.firstperson.pieces);
        ctx.firstperson.fallback = fallback_walls(ctx, &mut pieces);
        ctx.firstperson.wall_height = ctx.firstperson.fallback.as_ref().map_or(f32::MAX, |(t, left)| if *left { t.left_top } else { t.right_top }) + 0.05;
        ctx.firstperson.pieces = pieces;
    }
    let yaw = ctx.firstperson.yaw;
    let fwd = (yaw.cos(), yaw.sin());
    let right = (-fwd.1, fwd.0);
    let mut pieces = std::mem::take(&mut ctx.firstperson.pieces);
    // The eye sits a little behind the hero (the tile rules let the hero stand right at a wall),
    // without going through a wall behind.
    let at = camera_pos(ctx, me);
    let back = (-fwd.0, -fwd.1);
    let pull = (first_wall(ctx, &mut pieces, at, back, CAMERA_BACK + 0.15) - 0.15).clamp(0.0, CAMERA_BACK);
    let cam = (at.0 + back.0 * pull, at.1 + back.1 * pull);
    let (w, h) = (out.w(), out.h());
    let (top, bottom) = visible_rows(ctx, h);
    let horizon = top + (bottom - top) * HORIZON;
    let half = (FOV_DEG.to_radians() / 2.0).tan();
    let focal = (w as f32 / 2.0) / half;
    let mut depth = vec![f32::MAX; (w * h) as usize];
    let q = (cam.0 + 0.5, cam.1 + 0.5);
    let mut hits: Vec<(f32, f32, f32, Rc<PieceTex>, u8, f32, f32)> = Vec::new(); // dist, fx, fy, picture, light, lowest and highest height drawn
    for col in 0..w {
        let camx = 2.0 * (col as f32 + 0.5) / w as f32 - 1.0;
        let ray = (fwd.0 + right.0 * camx * half, fwd.1 + right.1 * camx * half);
        // Walk the tile edges the ray crosses; tile (i, j) covers [i - 0.5, i + 0.5). The walls
        // stand on the back edges of a tile (x - 0.5 for the left half of the picture, y - 0.5
        // for the right half), so an edge may hold the wall of the tile on its far side.
        let mut cell = (q.0.floor() as i32, q.1.floor() as i32);
        let delta = (if ray.0 == 0.0 { f32::MAX } else { (1.0 / ray.0).abs() }, if ray.1 == 0.0 { f32::MAX } else { (1.0 / ray.1).abs() });
        let step = (if ray.0 < 0.0 { -1 } else { 1 }, if ray.1 < 0.0 { -1 } else { 1 });
        let mut side = (
            if ray.0 < 0.0 { (q.0 - cell.0 as f32) * delta.0 } else { (cell.0 as f32 + 1.0 - q.0) * delta.0 },
            if ray.1 < 0.0 { (q.1 - cell.1 as f32) * delta.1 } else { (cell.1 as f32 + 1.0 - q.1) * delta.1 },
        );
        hits.clear();
        loop {
            let from = Point::new(cell.0, cell.1);
            let (dist, xside) = if side.0 < side.1 {
                cell.0 += step.0;
                let d = side.0;
                side.0 += delta.0;
                (d, true)
            } else {
                cell.1 += step.1;
                let d = side.1;
                side.1 += delta.1;
                (d, false)
            };
            let to = Point::new(cell.0, cell.1);
            if dist > MAX_DIST || !in_dungeon_bounds(to) || hits.len() >= 24 {
                break;
            }
            // the tile whose back edge this is: the one with the larger coordinate
            let owner = if xside {
                if step.0 > 0 { to } else { from }
            } else if step.1 > 0 {
                to
            } else {
                from
            };
            // Walking into solid rock whose side facing the viewer the original never drew (it faces
            // away from the isometric camera): the level's usual wall stands there instead.
            // A closed door: its own picture, whichever side it is seen from (the door is drawn on
            // one edge of its tile in the art; from the other side the same picture stands there).
            if blocks(ctx, to) && !blocks(ctx, from) && door_at(ctx, to) {
                let mut doors = std::mem::take(&mut ctx.firstperson.doors);
                let door = door_tex(ctx, &mut doors, to);
                ctx.firstperson.doors = doors;
                if let Some(tex) = door {
                    if tex.left_wall || tex.right_wall {
                        let left = if xside { tex.left_wall } else { !tex.right_wall };
                        let hp = (cam.0 + ray.0 * dist, cam.1 + ray.1 * dist);
                        let along = if xside { hp.1 - to.y as f32 } else { hp.0 - to.x as f32 };
                        let (fx, fy) = match tex.door_span {
                            Some((s0, s1)) => {
                                let t = (along + 0.5).clamp(0.0, 1.0);
                                plane_point(left, if left { s1 - t * (s1 - s0) } else { s0 + t * (s1 - s0) })
                            }
                            None => if left { (-0.5, along) } else { (along, -0.5) },
                        };
                        let light = light_at(ctx, from).saturating_add(fog(ctx, dist));
                        let top = (if left { tex.left_top } else { tex.right_top }).min(ctx.firstperson.wall_height);
                        hits.push((dist, fx, fy, tex, light, 0.0, top));
                        // the wall around and above the door
                        if let Some((plain, pl)) = ctx.firstperson.fallback.clone() {
                            let (px, py) = if pl { (-0.5, along) } else { (along, -0.5) };
                            let ptop = (if pl { plain.left_top } else { plain.right_top }).min(ctx.firstperson.wall_height);
                            hits.push((dist, px, py, plain, light, 0.0, ptop));
                        }
                        continue;
                    }
                }
            }
            if blocks(ctx, to) && !blocks(ctx, from) && !is_prop(ctx, &mut pieces, to) {
                let to_piece = ctx.gendung.dPiece[to.x as usize][to.y as usize] as usize;
                let drawn = piece_tex(ctx, &mut pieces, to_piece).is_some_and(|t| if xside { t.left_wall } else { t.right_wall });
                if !drawn {
                    if let Some((tex, left)) = ctx.firstperson.fallback.clone() {
                        let hp = (cam.0 + ray.0 * dist, cam.1 + ray.1 * dist);
                        let along = if xside { hp.1 - to.y as f32 } else { hp.0 - to.x as f32 };
                        let (fx, fy) = if left { (-0.5, along) } else { (along, -0.5) };
                        let light = light_at(ctx, from).saturating_add(fog(ctx, dist));
                        let top = (if left { tex.left_top } else { tex.right_top }).min(ctx.firstperson.wall_height);
                        hits.push((dist, fx, fy, tex, light, 0.0, top));
                        continue;
                    }
                }
            }
            let piece = ctx.gendung.dPiece[owner.x as usize][owner.y as usize] as usize;
            let Some(mut tex) = piece_tex(ctx, &mut pieces, piece) else { continue };
            if (xside && !tex.left_wall) || (!xside && !tex.right_wall) {
                continue;
            }
            // Seen from behind (from the side the isometric camera never sees), the picture of a
            // solid wall shows its front, thickness and all, which does not line up from this side:
            // the level's plain wall stands in for it. Archways look the same from both sides.
            let hp = (cam.0 + ray.0 * dist, cam.1 + ray.1 * dist);
            let along = if xside { hp.1 - owner.y as f32 } else { hp.0 - owner.x as f32 };
            let mut tex_left = xside;
            // (an archway or open door seen from behind: plain brick above the opening)
            if owner == to && !door_at(ctx, owner) || owner == to && !blocks(ctx, owner) {
                if let Some((plain, left)) = ctx.firstperson.fallback.clone() {
                    tex = plain;
                    tex_left = left;
                }
            }
            let (fx, fy) = if tex_left { (-0.5, along) } else { (along, -0.5) };
            let light = light_at(ctx, owner).min(light_at(ctx, from)).saturating_add(fog(ctx, dist));
            let floor_of_art = if blocks(ctx, owner) { 0.0 } else { ARCH_TOP };
            let top = (if tex_left { tex.left_top } else { tex.right_top }).min(ctx.firstperson.wall_height);
            hits.push((dist, fx, fy, tex, light, floor_of_art, top));
        }
        for row in 0..h {
            let dy = row as f32 + 0.5 - horizon;
            let mut colour: Option<(u8, f32)> = None;
            for (dist, fx, fy, tex, light, lowest, highest) in &hits {
                let height = EYE - dy * dist / focal;
                if height < *lowest || height > *highest {
                    continue;
                }
                if let Some(i) = wall_pixel(tex, *fx, *fy, height) {
                    colour = Some((shade(ctx, tex.px[i], *light), *dist));
                    break;
                }
            }
            if colour.is_none() && dy > 0.0 {
                let t = EYE * focal / dy;
                if t < MAX_DIST {
                    let p = (cam.0 + ray.0 * t, cam.1 + ray.1 * t);
                    let tile = Point::new(p.0.round() as i32, p.1.round() as i32);
                    if in_dungeon_bounds(tile) {
                        let piece = ctx.gendung.dPiece[tile.x as usize][tile.y as usize] as usize;
                        if let Some(tex) = piece_tex(ctx, &mut pieces, piece) {
                            let (fx, fy) = (p.0 - tile.x as f32, p.1 - tile.y as f32);
                            let sx = (32.0 + (fx - fy) * 32.0).clamp(0.0, 63.0) as i32;
                            let sy = ((tex.h - 16) as f32 + (fx + fy) * 16.0).clamp(0.0, (tex.h - 1) as f32) as i32;
                            let i = (sy * 64 + sx) as usize;
                            if tex.opaque[i] {
                                colour = Some((shade(ctx, tex.px[i], light_at(ctx, tile).saturating_add(fog(ctx, t))), t));
                            }
                        }
                    }
                }
            }
            let (c, d) = colour.unwrap_or((0, f32::MAX));
            out.put(col, row, c);
            depth[(row * w + col) as usize] = d;
        }
    }
    ctx.firstperson.pieces = pieces;

    // billboards, far to near
    let mut pieces = std::mem::take(&mut ctx.firstperson.pieces);
    // make sure the pictures of nearby pillars and posts are ready
    let (cx, cy) = (cam.0.round() as i32, cam.1.round() as i32);
    for x in (cx - 24).max(0)..=(cx + 24).min(MAXDUNX as i32 - 1) {
        for y in (cy - 24).max(0)..=(cy + 24).min(MAXDUNY as i32 - 1) {
            let piece = ctx.gendung.dPiece[x as usize][y as usize] as usize;
            piece_tex(ctx, &mut pieces, piece);
        }
    }
    let boards = collect_billboards(ctx, &pieces, cam, fwd, me);
    ctx.firstperson.pieces = pieces;
    let depth_at = depth;
    let mut pick = vec![0u16; (w * h) as usize];
    let mut picked = Vec::new();
    for b in &boards {
        let rel = (b.pos.0 - cam.0, b.pos.1 - cam.1);
        let lat = rel.0 * right.0 + rel.1 * right.1;
        let depth = b.depth;
        let scale = focal / (depth * PX_PER_TILE);
        let (sw, sh, below) = match &b.prop {
            Some(t) => (64, t.h, (t.h - 1 - t.prop.map_or(t.h - 16, |p| p.2)) as f32),
            None => match &b.sprite {
                Some(sp) => (sp.width() as i32, sp.height() as i32, 16.0),
                None => continue,
            },
        };
        if sw <= 0 || sh <= 0 {
            continue;
        }
        let centre = w as f32 / 2.0 + focal * lat / depth;
        let bottom = horizon + focal * (EYE + below / PX_PER_TILE) / depth;
        let left = centre - sw as f32 * scale / 2.0;
        let top = bottom - sh as f32 * scale;
        let (x0, x1) = (left.floor().max(0.0) as i32, (left + sw as f32 * scale).ceil().min(w as f32) as i32);
        let (y0, y1) = (top.floor().max(0.0) as i32, bottom.ceil().min(h as f32) as i32);
        if x0 >= x1 || y0 >= y1 {
            continue;
        }
        // the picture's pixels; a sprite is drawn on black and on white to find the transparent ones
        let (px, op): (Vec<u8>, Vec<bool>) = match &b.prop {
            Some(t) => {
                let (x0, x1, base) = t.prop.unwrap_or((0, 63, t.h - 16));
                // within the floor diamond only the columns of the thing itself; nothing above the
                // level's wall height (the tops the eye cannot see)
                let highest = base as f32 - ctx.firstperson.wall_height * PX_PER_TILE;
                let op = (0..sw * sh).map(|i| t.opaque[i as usize] && (i / 64) as f32 >= highest && (i / 64 < t.h - 32 || (x0..=x1).contains(&(i % 64)))).collect();
                (t.px.clone(), op)
            }
            None => {
                let mut a = OwnedSurface::new(sw, sh);
                let mut bsurf = OwnedSurface::new(sw, sh);
                bsurf.pixels.iter_mut().for_each(|p| *p = 255);
                for s in [&mut a, &mut bsurf] {
                    let v = s.view();
                    match &b.trn {
                        Some(t) => clx_draw_trn(&v, (0, sh - 1), b.sprite.as_ref().unwrap(), t),
                        None => clx_draw(&v, (0, sh - 1), b.sprite.as_ref().unwrap()),
                    }
                }
                let (av, bv) = (a.view(), bsurf.view());
                let mut px = vec![0u8; (sw * sh) as usize];
                let mut op = vec![false; (sw * sh) as usize];
                for y in 0..sh {
                    for x in 0..sw {
                        let (ca, cb) = (av.get(x, y), bv.get(x, y));
                        px[(y * sw + x) as usize] = ca;
                        op[(y * sw + x) as usize] = ca == cb;
                    }
                }
                (px, op)
            }
        };
        let id = {
            picked.push((b.tile, b.kind));
            picked.len() as u16
        };
        let light = b.light.map(|l| l.saturating_add(fog(ctx, depth)));
        for col in x0..x1 {
            let u = (((col as f32 + 0.5 - left) / scale) as i32).clamp(0, sw - 1);
            for row in y0..y1 {
                let v = (((row as f32 + 0.5 - top) / scale) as i32).clamp(0, sh - 1);
                let i = (v * sw + u) as usize;
                let c = px[i];
                if !op[i] || depth_at[(row * w + col) as usize] <= depth {
                    continue;
                }
                let c = match light {
                    Some(l) => shade(ctx, c, l),
                    None => c,
                };
                out.put(col, row, c);
                pick[(row * w + col) as usize] = id;
            }
        }
    }
    draw_hero_view(ctx, out, me, fwd, bottom);
    let s = &mut ctx.firstperson;
    s.cam = cam;
    s.w = w;
    s.h = h;
    s.horizon = horizon;
    s.zbuf = depth_at;
    s.pick = pick;
    s.picked = picked;
    true
}

/// The rows of the view the control panel leaves visible (it sits at the top in this view).
fn visible_rows(ctx: &Ctx, h: i32) -> (f32, f32) {
    let panel = crate::control::get_main_panel(ctx);
    if panel.y == 0 {
        (panel.h.min(h / 2) as f32, h as f32)
    } else {
        (0.0, panel.y.clamp(h / 2, h) as f32)
    }
}

/// The hero's own sprite as the viewer sees it from behind (facing away, North in the pictures).
fn hero_sprite(ctx: &Ctx, me: usize, view: (f32, f32)) -> Option<ClxSprite> {
    let p = &ctx.players.Players[me];
    let cur = p.AnimInfo.sprites.as_ref()?;
    let frame = p.AnimInfo.get_frame_to_use_for_rendering(ctx.nthread.ProgressToNextGameTick) as usize;
    let shown = rotated(p._pdir, view);
    let first = cur.get(0);
    for a in p.AnimationData.iter() {
        if a.sprites_for_direction(p._pdir).is_some_and(|l| l.get(0) == first) {
            return Some(a.sprites_for_direction(shown).unwrap_or_else(|| cur.clone()).get(frame));
        }
    }
    Some(cur.get(frame))
}

/// Draws the hero, seen from just behind, cut off at the bottom of the view: the weapon, shield,
/// bow swings and spell casting show as in the isometric view.
fn draw_hero_view(ctx: &Ctx, out: &Surface, me: usize, fwd: (f32, f32), bottom: f32) {
    // only while attacking, shooting, blocking or casting, and see-through, so it never hides
    // a monster
    if !matches!(ctx.players.Players[me]._pmode, PM_ATTACK | PM_RATTACK | PM_BLOCK | PM_SPELL) {
        return;
    }
    let Some(sprite) = hero_sprite(ctx, me, fwd) else { return };
    let (sw, sh) = (sprite.width() as i32, sprite.height() as i32);
    if sw <= 0 || sh <= 0 {
        return;
    }
    let mut a = OwnedSurface::new(sw, sh);
    let mut b = OwnedSurface::new(sw, sh);
    b.pixels.iter_mut().for_each(|p| *p = 255);
    for s in [&mut a, &mut b] {
        clx_draw(&s.view(), (0, sh - 1), &sprite);
    }
    let (av, bv) = (a.view(), b.view());
    let (w, h) = (out.w(), out.h());
    let scale = (bottom - visible_rows(ctx, h).0) * HERO_VIEW_SCALE;
    // sprite row of the ground point: 16 pixels above the bottom (as the isometric view places it)
    let cut_row = (sh - 16) as f32 - HERO_VIEW_CUT;
    let top = bottom - cut_row * scale;
    let left = w as f32 * HERO_VIEW_X - sw as f32 * scale / 2.0;
    let (x0, x1) = (left.max(0.0) as i32, (left + sw as f32 * scale).min(w as f32) as i32);
    let (y0, y1) = (top.max(0.0) as i32, bottom as i32);
    for row in y0..y1 {
        let v = ((row as f32 + 0.5 - top) / scale) as i32;
        if v < 0 || v >= sh {
            continue;
        }
        for col in x0..x1 {
            let u = ((col as f32 + 0.5 - left) / scale) as i32;
            if u < 0 || u >= sw {
                continue;
            }
            let c = av.get(u, v);
            if c == bv.get(u, v) && (row + col) % 2 == 0 {
                out.put(col, row, c);
            }
        }
    }
}

/// `CheckCursMove` in the first-person view: the tile under the mouse (a monster, item or
/// object drawn there, else the floor point the mouse points at) and the exact ground point.
pub fn pick(ctx: &Ctx, mouse: Point) -> Option<(Point, (f32, f32))> {
    if !active(ctx) {
        return None;
    }
    let s = &ctx.firstperson;
    if mouse.x < 0 || mouse.y < 0 || mouse.x >= s.w || mouse.y >= s.h || s.zbuf.len() != (s.w * s.h) as usize {
        return None;
    }
    let mut id = s.pick[(mouse.y * s.w + mouse.x) as usize];
    if id == 0 || s.picked[id as usize - 1].1 == Kind::Other {
        // nothing exactly under the mouse: the nearest monster, item or object close by
        let mut best = i32::MAX;
        for dy in -PICK_SLACK..=PICK_SLACK {
            for dx in -PICK_SLACK..=PICK_SLACK {
                let (x, y) = (mouse.x + dx, mouse.y + dy);
                if x < 0 || y < 0 || x >= s.w || y >= s.h || dx * dx + dy * dy >= best {
                    continue;
                }
                let i = s.pick[(y * s.w + x) as usize];
                if i > 0 && s.picked[i as usize - 1].1 != Kind::Other {
                    best = dx * dx + dy * dy;
                    id = i;
                }
            }
        }
    }
    if id > 0 {
        let (t, kind) = s.picked[id as usize - 1];
        if kind != Kind::Other {
            return Some((t, (t.x as f32, t.y as f32)));
        }
    }
    let yaw = s.yaw;
    let fwd = (yaw.cos(), yaw.sin());
    let right = (-fwd.1, fwd.0);
    let half = (FOV_DEG.to_radians() / 2.0).tan();
    let focal = (s.w as f32 / 2.0) / half;
    let camx = 2.0 * (mouse.x as f32 + 0.5) / s.w as f32 - 1.0;
    let ray = (fwd.0 + right.0 * camx * half, fwd.1 + right.1 * camx * half);
    let dy = mouse.y as f32 + 0.5 - s.horizon;
    let seen = s.zbuf[(mouse.y * s.w + mouse.x) as usize];
    let floor = if dy > 0.0 { EYE * focal / dy } else { f32::MAX };
    // the floor point, or just in front of the wall drawn there
    let t = if seen < floor - 1e-3 { seen - 0.3 } else { floor }.min(MAX_DIST).max(0.0);
    let p = (s.cam.0 + ray.0 * t, s.cam.1 + ray.1 * t);
    let tile = Point::new(p.0.round().clamp(0.0, MAXDUNX as f32 - 1.0) as i32, p.1.round().clamp(0.0, MAXDUNY as f32 - 1.0) as i32);
    Some((tile, p))
}
