//! `Source/cursor.cpp`: the mouse cursor (software and hardware), and what it points at.

use crate::ctx::Ctx;
use crate::engine::geometry::{Displacement, Point};
use crate::enums::*;
use crate::utils::language::tr;

/// `cursor_id`
pub const CURSOR_NONE: i32 = 0;
pub const CURSOR_HAND: i32 = 1;
pub const CURSOR_IDENTIFY: i32 = 2;
pub const CURSOR_REPAIR: i32 = 3;
pub const CURSOR_RECHARGE: i32 = 4;
pub const CURSOR_DISARM: i32 = 5;
pub const CURSOR_OIL: i32 = 6;
pub const CURSOR_TELEKINESIS: i32 = 7;
pub const CURSOR_RESURRECT: i32 = 8;
pub const CURSOR_TELEPORT: i32 = 9;
pub const CURSOR_HEALOTHER: i32 = 10;
pub const CURSOR_HOURGLASS: i32 = 11;
pub const CURSOR_FIRSTITEM: i32 = 12;

pub struct CursorState {
    /// `pcurs`
    pub pcurs: i32,
    /// `pcursitem`
    pub pcursitem: i8,
    /// `pcursmonst`
    pub pcursmonst: i32,
    /// `pcursplr`
    pub pcursplr: i8,
    /// `ObjectUnderCursor` (index into `Objects`)
    pub ObjectUnderCursor: Option<usize>,
    /// `pcurstemp`
    pub pcurstemp: i32,
    /// `pcursinvitem`
    pub pcursinvitem: i8,
    /// `pcursstashitem`
    pub pcursstashitem: u16,
    /// `cursPosition`
    pub cursPosition: crate::engine::geometry::Point,
    /// `pCursCels`, `pCursCels2`
    pub p_curs_cels: Option<crate::engine::clx_sprite::ClxSpriteList>,
    pub p_curs_cels2: Option<crate::engine::clx_sprite::ClxSpriteList>,
}

impl Default for CursorState {
    fn default() -> Self {
        CursorState {
            pcurs: 0,
            pcursitem: 0,
            pcursmonst: -1,
            pcursplr: 0,
            ObjectUnderCursor: None,
            pcurstemp: 0,
            pcursinvitem: 0,
            pcursstashitem: 0,
            cursPosition: Default::default(),
            p_curs_cels: None,
            p_curs_cels2: None,
        }
    }
}

/// Original: `devilution::ResetCursor` (cursor.cpp).
// @port cursor.cpp|devilution::ResetCursor() sha=0028c9b9aa5b
pub fn reset_cursor(ctx: &mut Ctx) {
    let p = ctx.cursor.pcurs;
    new_cursor(ctx, p);
}

/// Original: `devilution::NewCursor(int cursId)` (cursor.cpp).
// @port cursor.cpp|devilution::NewCursor(int cursId) sha=c06610c5a462
pub fn new_cursor(ctx: &mut Ctx, curs_id: i32) {
    if ctx.cursor.pcurs >= CURSOR_FIRSTITEM && curs_id > CURSOR_HAND && curs_id < CURSOR_HOURGLASS && !crate::controls::plrctrls::try_drop_item(ctx) {
        return;
    }
    if curs_id < CURSOR_HOURGLASS && crate::player::my_player_exists(ctx) {
        crate::player::clear_my_hold_item(ctx);
    }
    ctx.cursor.pcurs = curs_id;
    if crate::hwcursor::is_hardware_cursor_enabled(ctx) && ctx.controls.control_device == crate::controls::ControlTypes::KeyboardAndMouse {
        let has_art_cursor = ctx.diablo_ui.art_cursor.is_some();
        if !has_art_cursor && curs_id == CURSOR_NONE {
            return;
        }
        let new_cursor =
            if has_art_cursor { crate::hwcursor::CursorInfo::user_interface_cursor() } else { crate::hwcursor::CursorInfo::game_cursor(curs_id) };
        if new_cursor != *crate::hwcursor::get_current_cursor_info(ctx) {
            crate::hwcursor::set_hardware_cursor(ctx, new_cursor);
        }
    }
}


/// `InvItemWidth1`
#[rustfmt::skip]
static InvItemWidth1: [u16; 179] = [33, 32, 32, 32, 32, 32, 32, 32, 32, 32, 23, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56];
/// `InvItemWidth2`
#[rustfmt::skip]
static InvItemWidth2: [u16; 61] = [28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 56, 56, 28, 28, 28, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56];
/// `InvItemHeight1`
#[rustfmt::skip]
static InvItemHeight1: [u16; 179] = [29, 32, 32, 32, 32, 32, 32, 32, 32, 32, 35, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 56, 56, 56, 56, 56, 56, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 56, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84];
/// `InvItemHeight2`
#[rustfmt::skip]
static InvItemHeight2: [u16; 61] = [28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 28, 56, 56, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84, 84];

/// `InvItems1Size`
const InvItems1Size: i32 = InvItemWidth1.len() as i32;

/// Original: `devilution::GetInvItemSize` (cursor.cpp).
// @port cursor.cpp|devilution::GetInvItemSize(int cursId) sha=2b9e41fcd993
pub fn get_inv_item_size(curs_id: i32) -> crate::engine::geometry::Size {
    let i = curs_id - 1;
    if i >= InvItems1Size {
        let j = (i - InvItems1Size) as usize;
        return crate::engine::geometry::Size::new(InvItemWidth2[j] as i32, InvItemHeight2[j] as i32);
    }
    crate::engine::geometry::Size::new(InvItemWidth1[i as usize] as i32, InvItemHeight1[i as usize] as i32)
}

/// Original: `devilution::GetNumInvItems` (cursor.cpp).
// @port cursor.cpp|devilution::GetNumInvItems() sha=d99f2d240ccc
pub fn get_num_inv_items() -> usize {
    InvItemWidth1.len() + InvItemWidth2.len()
}

/// Original: `devilution::NewCursor(const Item &item)` (cursor.cpp).
// @port cursor.cpp|devilution::NewCursor(const Item &item) sha=e024d6d38be2
pub fn new_cursor_item(ctx: &mut Ctx, item_empty: bool, item_curs: u8) {
    if item_empty {
        new_cursor(ctx, CURSOR_HAND);
    } else {
        new_cursor(ctx, item_curs as i32 + CURSOR_FIRSTITEM);
    }
}

/// Original: `devilution::InitCursor` (cursor.cpp).
// @port cursor.cpp|devilution::InitCursor() sha=b4945cd945ef
pub fn init_cursor(ctx: &mut Ctx) {
    assert!(ctx.cursor.p_curs_cels.is_none());
    ctx.cursor.p_curs_cels = Some(crate::engine::load_sprites::load_cel_widths(ctx, "data\\inv\\objcurs", &InvItemWidth1));
    if ctx.init.gb_is_hellfire {
        ctx.cursor.p_curs_cels2 = Some(crate::engine::load_sprites::load_cel_widths(ctx, "data\\inv\\objcurs2", &InvItemWidth2));
    }
    crate::engine::render::scrollrt::clear_cursor(ctx);
}

/// Original: `devilution::FreeCursor` (cursor.cpp).
// @port cursor.cpp|devilution::FreeCursor() sha=c032ca1cdf39
pub fn free_cursor(ctx: &mut Ctx) {
    ctx.cursor.p_curs_cels = None;
    ctx.cursor.p_curs_cels2 = None;
    crate::engine::render::scrollrt::clear_cursor(ctx);
}

/// Original: `devilution::GetInvItemSprite` (cursor.cpp).
// @port cursor.cpp|devilution::GetInvItemSprite(int cursId) sha=b5122d1c0bfb
pub fn get_inv_item_sprite(ctx: &Ctx, curs_id: i32) -> crate::engine::clx_sprite::ClxSprite {
    if curs_id <= InvItems1Size {
        return ctx.cursor.p_curs_cels.as_ref().expect("pCursCels").get((curs_id - 1) as usize);
    }
    ctx.cursor.p_curs_cels2.as_ref().expect("pCursCels2").get((curs_id - InvItems1Size - 1) as usize)
}

/// Original: `devilution::DrawItem` (cursor.cpp): an inventory item sprite, red if unusable.
// @port cursor.cpp|devilution::DrawItem(const Item &item, const Surface &out, Point position, ClxSprite clx) sha=39778a322680
pub fn draw_item(ctx: &Ctx, item: &crate::items::Item, out: &crate::engine::surface::Surface, position: Point, clx: &crate::engine::clx_sprite::ClxSprite) {
    let usable = if !crate::player::is_inspecting_player(ctx) {
        item._iStatFlag
    } else {
        ctx.players.Players[ctx.players.InspectPlayer.expect("InspectPlayer")].can_use_item(item)
    };
    if usable {
        crate::engine::render::clx_render::clx_draw(out, (position.x, position.y), clx);
    } else {
        crate::engine::render::clx_render::clx_draw_trn(out, (position.x, position.y), clx, crate::engine::trn::get_infravision_trn(ctx));
    }
}

/// Original: `devilution::DrawSoftwareCursor` (cursor.cpp).
// @port cursor.cpp|devilution::DrawSoftwareCursor(const Surface &out, Point position, int cursId) sha=4d87930c5703
pub fn draw_software_cursor(ctx: &mut Ctx, out: &crate::engine::surface::Surface, position: Point, curs_id: i32) {
    let sprite = get_inv_item_sprite(ctx, curs_id);
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let held_item = &ctx.players.Players[me].HoldItem;
    if !held_item.is_empty() {
        crate::engine::render::clx_render::clx_draw_outline(out, crate::items::get_outline_color(held_item, true), (position.x, position.y), &sprite);
        draw_item(ctx, held_item, out, position, &sprite);
    } else {
        crate::engine::render::clx_render::clx_draw(out, (position.x, position.y), &sprite);
    }
}

/// Original: `devilution::InitLevelCursor` (cursor.cpp).
// @port cursor.cpp|devilution::InitLevelCursor() sha=2b585402d295
pub fn init_level_cursor(ctx: &mut Ctx) {
    new_cursor(ctx, CURSOR_HAND);
    ctx.cursor.cursPosition = ctx.gendung.ViewPosition;
    ctx.cursor.pcurstemp = -1;
    ctx.cursor.pcursmonst = -1;
    ctx.cursor.ObjectUnderCursor = None;
    ctx.cursor.pcursitem = -1;
    ctx.cursor.pcursstashitem = crate::qol::stash::StashStruct::EmptyCell;
    ctx.cursor.pcursplr = -1;
    crate::engine::render::scrollrt::clear_cursor(ctx);
}

/// Original: `CheckTown` (cursor.cpp).
// @port cursor.cpp|devilution::CheckTown() sha=ca5139b1e897
pub fn check_town(ctx: &mut Ctx) {
    for mi in 0..ctx.missiles.Missiles.len() {
        let missile = &ctx.missiles.Missiles[mi];
        if missile._mitype == MissileID::TownPortal && crate::levels::trigs::entrance_boundary_contains(missile.position.tile, ctx.cursor.cursPosition) {
            let (tile, source) = (missile.position.tile, missile._misource as usize);
            ctx.trigs.trigflag = true;
            ctx.control.info_string = tr("Town Portal");
            let name = ctx.players.Players[source]._pName.as_str().to_string();
            crate::control::add_panel_string(ctx, &tr("from {:s}").replacen("{:s}", &name, 1));
            ctx.cursor.cursPosition = tile;
        }
    }
}

/// Original: `CheckRportal` (cursor.cpp).
// @port cursor.cpp|devilution::CheckRportal() sha=ae9960bd3b03
pub fn check_rportal(ctx: &mut Ctx) {
    for mi in 0..ctx.missiles.Missiles.len() {
        let missile = &ctx.missiles.Missiles[mi];
        if missile._mitype == MissileID::RedPortal && crate::levels::trigs::entrance_boundary_contains(missile.position.tile, ctx.cursor.cursPosition) {
            let tile = missile.position.tile;
            ctx.trigs.trigflag = true;
            ctx.control.info_string = tr("Portal to");
            let s = if !ctx.gendung.setlevel { tr("The Unholy Altar") } else { tr("level 15") };
            crate::control::add_panel_string(ctx, &s);
            ctx.cursor.cursPosition = tile;
        }
    }
}

/// Monster selection test of `CheckCursMove`: a live monster at `(x, y)` (relative tile `d`)
/// whose selection type has `bit`, optionally only the one remembered in `pcurstemp`.
fn try_select_monster(ctx: &mut Ctx, mx: i32, my: i32, d: (i32, i32), bit: i8, only_temp: bool) {
    let (x, y) = ((mx + d.0) as usize, (my + d.1) as usize);
    let dm = ctx.gendung.dMonster[x][y];
    if dm == 0 || !crate::levels::gendung::is_tile_lit(ctx, Point::new(x as i32, y as i32)) {
        return;
    }
    let monster_id = ((dm as i32).abs() - 1) as u16 as usize;
    if only_temp && monster_id as i32 != ctx.cursor.pcurstemp {
        return;
    }
    let m = &ctx.monster.Monsters[monster_id];
    if m.hitPoints >> 6 > 0 && (crate::monster::monster_data(ctx, monster_id).selectionType & bit) != 0 {
        ctx.cursor.cursPosition = Point::new(mx, my) + Displacement::new(d.0, d.1);
        ctx.cursor.pcursmonst = monster_id as i32;
    }
}

/// Original: `devilution::CheckCursMove` (cursor.cpp).
// @port cursor.cpp|devilution::CheckCursMove() sha=f851dcd4779b
pub fn check_curs_move(ctx: &mut Ctx) {
    use crate::levels::gendung::{MAXDUNX, MAXDUNY};
    if crate::qol::itemlabels::is_item_label_highlighted(ctx) {
        return;
    }
    let mouse = Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1);
    let mut sx = mouse.x;
    let mut sy = mouse.y;
    if crate::control::can_panels_cover_view(ctx) {
        if crate::control::is_left_panel_open(ctx) {
            sx -= crate::utils::display::get_screen_width(ctx) / 4;
        } else if crate::control::is_right_panel_open(ctx) {
            sx += crate::utils::display::get_screen_width(ctx) / 4;
        }
    }
    let main_panel = crate::control::get_main_panel(ctx);
    if main_panel.contains(mouse) && crate::track::track_isscrolling(ctx) {
        sy = main_panel.y - 1;
    }
    let zoom = ctx.options.graphics.zoom.get();
    if zoom {
        sx /= 2;
        sy /= 2;
    }
    // Adjust by player offset and tile grid alignment
    let (xo, yo) = crate::engine::render::scrollrt::calc_tile_offset(ctx);
    sx += xo;
    sy += yo;

    let me = ctx.players.MyPlayer.expect("MyPlayer");
    {
        let my_player = &ctx.players.Players[me];
        if my_player.is_walking() {
            let offset = crate::engine::render::scrollrt::get_offset_for_walking(ctx, &my_player.AnimInfo, my_player._pdir, true);
            sx -= offset.delta_x;
            sy -= offset.delta_y;
            // Predict the next frame when walking to avoid input jitter
            let progress = ctx.nthread.ProgressToNextGameTick;
            let offset2 = my_player.position.calculate_walking_offset_shifted8(my_player._pdir, &my_player.AnimInfo, progress);
            let velocity = my_player.position.get_walking_velocity_shifted8(my_player._pdir, &my_player.AnimInfo);
            let (o2x, o2y) = (offset2.delta_x as i16 as i32, offset2.delta_y as i16 as i32);
            let (vx, vy) = (velocity.delta_x as i16 as i32, velocity.delta_y as i16 as i32);
            let mut fx = o2x / 256;
            let mut fy = o2y / 256;
            fx -= (o2x + vx) / 256;
            fy -= (o2y + vy) / 256;
            sx -= fx;
            sy -= fy;
        }
    }

    // Convert to tile grid
    let mut mx = ctx.gendung.ViewPosition.x;
    let mut my = ctx.gendung.ViewPosition.y;
    let (columns, rows) = crate::engine::render::scrollrt::tiles_in_view(ctx);
    let lrow = rows - crate::engine::render::scrollrt::rows_covered_by_panel(ctx);
    // Center player tile on screen
    crate::engine::render::scrollrt::shift_grid(&mut mx, &mut my, -columns / 2, -lrow / 2);
    // Align grid
    if (columns % 2) == 0 && (lrow % 2) == 0 {
        sy += crate::engine::render::dun_render::TILE_HEIGHT / 2;
    } else if (columns % 2) != 0 && (lrow % 2) != 0 {
        sx -= crate::engine::render::dun_render::TILE_WIDTH / 2;
    } else if (columns % 2) != 0 && (lrow % 2) == 0 {
        my += 1;
    }
    if zoom {
        sy -= crate::engine::render::dun_render::TILE_HEIGHT / 4;
    }
    let tx = sx / crate::engine::render::dun_render::TILE_WIDTH;
    let ty = sy / crate::engine::render::dun_render::TILE_HEIGHT;
    crate::engine::render::scrollrt::shift_grid(&mut mx, &mut my, tx, ty);
    // Shift position to match diamond grid aligment
    let px = sx % crate::engine::render::dun_render::TILE_WIDTH;
    let py = sy % crate::engine::render::dun_render::TILE_HEIGHT;
    let flipy = py < (px / 2);
    if flipy {
        my -= 1;
    }
    let flipx = py >= crate::engine::render::dun_render::TILE_HEIGHT - (px / 2);
    if flipx {
        mx += 1;
    }
    mx = mx.clamp(0, MAXDUNX as i32 - 1);
    my = my.clamp(0, MAXDUNY as i32 - 1);
    let current_tile = Point::new(mx, my);

    // While holding the button down we should retain target (but potentially lose it if it dies, goes out of view, etc)
    let lmba = ctx.diablo.last_mouse_button_action;
    use crate::diablo::MouseActionType;
    if (ctx.diablo.sgb_mouse_down != crate::enums::CLICK_NONE || crate::controls::plrctrls::controller_action_held(ctx))
        && !matches!(lmba, MouseActionType::None | MouseActionType::Attack | MouseActionType::Spell)
    {
        crate::track::invalidate_targets(ctx);
        let c = &ctx.cursor;
        if c.pcursmonst == -1 && c.ObjectUnderCursor.is_none() && c.pcursitem == -1 && c.pcursinvitem == -1 && c.pcursstashitem == crate::qol::stash::StashStruct::EmptyCell && c.pcursplr == -1 {
            ctx.cursor.cursPosition = Point::new(mx, my);
            crate::levels::trigs::check_trig_force(ctx);
            check_town(ctx);
            check_rportal(ctx);
        }
        return;
    }

    let flipflag = (flipy && flipx) || ((flipy || flipx) && px < crate::engine::render::dun_render::TILE_WIDTH / 2);

    ctx.cursor.pcurstemp = ctx.cursor.pcursmonst;
    ctx.cursor.pcursmonst = -1;
    ctx.cursor.ObjectUnderCursor = None;
    ctx.cursor.pcursitem = -1;
    if ctx.cursor.pcursinvitem != -1 {
        crate::engine::backbuffer_state::redraw_component(ctx, crate::engine::backbuffer_state::PanelDrawComponent::Belt);
    }
    ctx.cursor.pcursinvitem = -1;
    ctx.cursor.pcursstashitem = crate::qol::stash::StashStruct::EmptyCell;
    ctx.cursor.pcursplr = -1;
    ctx.items.ShowUniqueItemInfoBox = false;
    ctx.control.panelflag = false;
    ctx.trigs.trigflag = false;

    if ctx.players.Players[me]._pInvincible {
        return;
    }
    if !ctx.players.Players[me].HoldItem.is_empty() || ctx.control.spselflag {
        ctx.cursor.cursPosition = Point::new(mx, my);
        return;
    }
    if main_panel.contains(mouse) {
        crate::control::check_panel_info(ctx);
        return;
    }
    if ctx.doom.DoomFlag {
        return;
    }
    if ctx.inv.invflag && crate::control::get_right_panel(ctx).contains(mouse) {
        ctx.cursor.pcursinvitem = crate::inv::check_inv_h_light(ctx);
        return;
    }
    if ctx.stash.IsStashOpen && crate::control::get_left_panel(ctx).contains(mouse) {
        ctx.cursor.pcursstashitem = crate::qol::stash::check_stash_h_light(ctx, mouse);
    }
    if ctx.control.sbookflag && crate::control::get_right_panel(ctx).contains(mouse) {
        return;
    }
    if crate::control::is_left_panel_open(ctx) && crate::control::get_left_panel(ctx).contains(mouse) {
        return;
    }

    let (mxu, myu) = (MAXDUNX as i32, MAXDUNY as i32);
    if ctx.gendung.leveltype != crate::levels::gendung::DungeonType::Town {
        if ctx.cursor.pcurstemp != -1 {
            if !flipflag && mx + 2 < mxu && my + 1 < myu {
                try_select_monster(ctx, mx, my, (2, 1), 4, true);
            }
            if flipflag && mx + 1 < mxu && my + 2 < myu {
                try_select_monster(ctx, mx, my, (1, 2), 4, true);
            }
            if mx + 2 < mxu && my + 2 < myu {
                try_select_monster(ctx, mx, my, (2, 2), 4, true);
            }
            if mx + 1 < mxu && !flipflag {
                try_select_monster(ctx, mx, my, (1, 0), 2, true);
            }
            if my + 1 < myu && flipflag {
                try_select_monster(ctx, mx, my, (0, 1), 2, true);
            }
            try_select_monster(ctx, mx, my, (0, 0), 1, true);
            if mx + 1 < mxu && my + 1 < myu {
                try_select_monster(ctx, mx, my, (1, 1), 2, true);
            }
            let pm = ctx.cursor.pcursmonst;
            if pm != -1 && (ctx.monster.Monsters[pm as usize].flags & MFLAG_HIDDEN as u32) != 0 {
                ctx.cursor.pcursmonst = -1;
                ctx.cursor.cursPosition = Point::new(mx, my);
            }
            let pm = ctx.cursor.pcursmonst;
            if pm != -1 && ctx.monster.Monsters[pm as usize].is_player_minion() {
                ctx.cursor.pcursmonst = -1;
            }
            if ctx.cursor.pcursmonst != -1 {
                return;
            }
        }
        if !flipflag && mx + 2 < mxu && my + 1 < myu {
            try_select_monster(ctx, mx, my, (2, 1), 4, false);
        }
        if flipflag && mx + 1 < mxu && my + 2 < myu {
            try_select_monster(ctx, mx, my, (1, 2), 4, false);
        }
        if mx + 2 < mxu && my + 2 < myu {
            try_select_monster(ctx, mx, my, (2, 2), 4, false);
        }
        if !flipflag && mx + 1 < mxu {
            try_select_monster(ctx, mx, my, (1, 0), 2, false);
        }
        if flipflag && my + 1 < myu {
            try_select_monster(ctx, mx, my, (0, 1), 2, false);
        }
        try_select_monster(ctx, mx, my, (0, 0), 1, false);
        if mx + 1 < mxu && my + 1 < myu {
            try_select_monster(ctx, mx, my, (1, 1), 2, false);
        }
        let pm = ctx.cursor.pcursmonst;
        if pm != -1 && (ctx.monster.Monsters[pm as usize].flags & MFLAG_HIDDEN as u32) != 0 {
            ctx.cursor.pcursmonst = -1;
            ctx.cursor.cursPosition = Point::new(mx, my);
        }
        let pm = ctx.cursor.pcursmonst;
        if pm != -1 && (ctx.monster.Monsters[pm as usize].is_player_minion() || matches!(ctx.cursor.pcurs, CURSOR_HEALOTHER | CURSOR_RESURRECT)) {
            ctx.cursor.pcursmonst = -1;
        }
    } else {
        let dm = |ctx: &Ctx, x: i32, y: i32| ctx.gendung.dMonster[x as usize][y as usize];
        if !flipflag && mx + 1 < mxu && dm(ctx, mx + 1, my) > 0 {
            ctx.cursor.pcursmonst = dm(ctx, mx + 1, my) as i32 - 1;
            ctx.cursor.cursPosition = Point::new(mx + 1, my);
        }
        if flipflag && my + 1 < myu && dm(ctx, mx, my + 1) > 0 {
            ctx.cursor.pcursmonst = dm(ctx, mx, my + 1) as i32 - 1;
            ctx.cursor.cursPosition = Point::new(mx, my + 1);
        }
        if dm(ctx, mx, my) > 0 {
            ctx.cursor.pcursmonst = dm(ctx, mx, my) as i32 - 1;
            ctx.cursor.cursPosition = Point::new(mx, my);
        }
        if mx + 1 < mxu && my + 1 < myu && dm(ctx, mx + 1, my + 1) > 0 {
            ctx.cursor.pcursmonst = dm(ctx, mx + 1, my + 1) as i32 - 1;
            ctx.cursor.cursPosition = Point::new(mx + 1, my + 1);
        }
    }

    if ctx.cursor.pcursmonst == -1 {
        let dp = |ctx: &Ctx, x: i32, y: i32| ctx.gendung.dPlayer[x as usize][y as usize];
        let try_player = |ctx: &mut Ctx, x: i32, y: i32| {
            let player_id = ((dp(ctx, x, y) as i32).abs() - 1) as u8 as usize;
            let p = &ctx.players.Players[player_id];
            if Some(player_id) != ctx.players.MyPlayer && p._pHitPoints != 0 {
                ctx.cursor.cursPosition = Point::new(x, y);
                ctx.cursor.pcursplr = player_id as i8;
            }
        };
        if !flipflag && mx + 1 < mxu && dp(ctx, mx + 1, my) != 0 {
            try_player(ctx, mx + 1, my);
        }
        if flipflag && my + 1 < myu && dp(ctx, mx, my + 1) != 0 {
            try_player(ctx, mx, my + 1);
        }
        if dp(ctx, mx, my) != 0 {
            let player_id = ((dp(ctx, mx, my) as i32).abs() - 1) as u8 as usize;
            if player_id != ctx.players.MyPlayerId {
                ctx.cursor.cursPosition = Point::new(mx, my);
                ctx.cursor.pcursplr = player_id as i8;
            }
        }
        if crate::levels::gendung::tile_contains_dead_player(ctx, Point::new(mx, my)) {
            for pnum in 0..ctx.players.Players.len() {
                if ctx.players.Players[pnum].position.tile == Point::new(mx, my) && Some(pnum) != ctx.players.MyPlayer {
                    ctx.cursor.cursPosition = Point::new(mx, my);
                    ctx.cursor.pcursplr = pnum as i8;
                }
            }
        }
        if ctx.cursor.pcurs == CURSOR_RESURRECT {
            for xx in -1..2 {
                for yy in -1..2 {
                    if crate::levels::gendung::tile_contains_dead_player(ctx, Point::new(mx + xx, my + yy)) {
                        for pnum in 0..ctx.players.Players.len() {
                            let t = ctx.players.Players[pnum].position.tile;
                            if t.x == mx + xx && t.y == my + yy && Some(pnum) != ctx.players.MyPlayer {
                                ctx.cursor.cursPosition = Point::new(mx + xx, my + yy);
                                ctx.cursor.pcursplr = pnum as i8;
                            }
                        }
                    }
                }
            }
        }
        if mx + 1 < mxu && my + 1 < myu && dp(ctx, mx + 1, my + 1) != 0 {
            try_player(ctx, mx + 1, my + 1);
        }
    }
    if ctx.cursor.pcursmonst == -1 && ctx.cursor.pcursplr == -1 {
        // No monsters or players under the cursor, try find an object starting with the tile below the current tile (tall
        //  objects like doors)
        let sel = |ctx: &Ctx, o: Option<usize>| o.map(|oi| ctx.objects.Objects[oi]._oSelFlag);
        let mut test_position = current_tile + crate::engine::geometry::Direction::South;
        let mut object = crate::objects::find_object_at_position(ctx, test_position, true);
        if object.is_none() || sel(ctx, object).unwrap() < 2 {
            // Either no object or can't interact from the test position, try the current tile
            test_position = current_tile;
            object = crate::objects::find_object_at_position(ctx, test_position, true);
            if object.is_none() || !matches!(sel(ctx, object).unwrap(), 1 | 3) {
                // Still no object (that could be activated from this position), try the tile to the bottom left or right
                //  (whichever is closest to the cursor as determined when we set flipflag earlier)
                let d = if flipflag { crate::engine::geometry::Direction::SouthWest } else { crate::engine::geometry::Direction::SouthEast };
                test_position = current_tile + d;
                object = crate::objects::find_object_at_position(ctx, test_position, true);
                if object.is_some() && sel(ctx, object).unwrap() < 2 {
                    // Found an object but it's not in range, clear the pointer
                    object = None;
                }
            }
        }
        if object.is_some() {
            // found object that can be activated with the given cursor position
            ctx.cursor.cursPosition = test_position;
            ctx.cursor.ObjectUnderCursor = object;
        }
    }
    if ctx.cursor.pcursplr == -1 && ctx.cursor.ObjectUnderCursor.is_none() && ctx.cursor.pcursmonst == -1 {
        let di = |ctx: &Ctx, x: i32, y: i32| ctx.items.dItem[x as usize][y as usize];
        let try_item = |ctx: &mut Ctx, x: i32, y: i32, ok: &dyn Fn(u8) -> bool| {
            let item_id = (di(ctx, x, y) - 1) as u8;
            if ok(ctx.items.Items[item_id as usize]._iSelFlag) {
                ctx.cursor.cursPosition = Point::new(x, y);
                ctx.cursor.pcursitem = item_id as i8;
            }
        };
        if !flipflag && mx + 1 < mxu && di(ctx, mx + 1, my) > 0 {
            try_item(ctx, mx + 1, my, &|s| s >= 2);
        }
        if flipflag && my + 1 < myu && di(ctx, mx, my + 1) > 0 {
            try_item(ctx, mx, my + 1, &|s| s >= 2);
        }
        if di(ctx, mx, my) > 0 {
            try_item(ctx, mx, my, &|s| s == 1 || s == 3);
        }
        if mx + 1 < mxu && my + 1 < myu && di(ctx, mx + 1, my + 1) > 0 {
            try_item(ctx, mx + 1, my + 1, &|s| s >= 2);
        }
        if ctx.cursor.pcursitem == -1 {
            ctx.cursor.cursPosition = Point::new(mx, my);
            crate::levels::trigs::check_trig_force(ctx);
            check_town(ctx);
            check_rportal(ctx);
        }
    }
    if ctx.cursor.pcurs == CURSOR_IDENTIFY {
        ctx.cursor.ObjectUnderCursor = None;
        ctx.cursor.pcursmonst = -1;
        ctx.cursor.pcursitem = -1;
        ctx.cursor.cursPosition = Point::new(mx, my);
    }
    let pm = ctx.cursor.pcursmonst;
    if pm != -1 && ctx.gendung.leveltype != crate::levels::gendung::DungeonType::Town && ctx.monster.Monsters[pm as usize].is_player_minion() {
        ctx.cursor.pcursmonst = -1;
    }
}

crate::pending_fn!(pub fn get_half_size_item_sprite(ctx: &Ctx, curs_id: i32) -> crate::engine::clx_sprite::ClxSprite, "cursor.cpp|devilution::GetHalfSizeItemSprite(int cursId)");
crate::pending_fn!(pub fn get_half_size_item_sprite_red(ctx: &Ctx, curs_id: i32) -> crate::engine::clx_sprite::ClxSprite, "cursor.cpp|devilution::GetHalfSizeItemSpriteRed(int cursId)");
crate::pending_fn!(pub fn create_half_size_item_sprites(ctx: &mut Ctx), "cursor.cpp|devilution::CreateHalfSizeItemSprites()");
crate::pending_fn!(pub fn free_half_size_item_sprites(ctx: &mut Ctx), "cursor.cpp|devilution::FreeHalfSizeItemSprites()");
