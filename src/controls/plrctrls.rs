//! `Source/controls/plrctrls` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;


crate::pending_fn!(pub fn primary_action_pressed(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() PrimaryAction lambda");

crate::pending_fn!(pub fn secondary_action_pressed(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() SecondaryAction lambda");

crate::pending_fn!(pub fn spell_action_pressed(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() SpellAction lambda");

crate::pending_fn!(pub fn cancel_action_pressed(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() CancelAction lambda");

crate::pending_fn!(pub fn cancel_action_enabled(ctx: &crate::ctx::Ctx) -> bool, "diablo.cpp|devilution::InitPadmapActions() CancelAction enable lambda");

crate::pending_fn!(pub fn controller_action_released(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() action release lambda");

crate::pending_fn!(pub fn pad_left_mouse_down(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() leftMouseDown");

crate::pending_fn!(pub fn pad_left_mouse_up(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() leftMouseUp");

crate::pending_fn!(pub fn pad_right_mouse_down(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() rightMouseDown");

crate::pending_fn!(pub fn pad_right_mouse_up(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() rightMouseUp");

crate::pending_fn!(pub fn pad_toggle_game_menu(ctx: &mut crate::ctx::Ctx), "diablo.cpp|devilution::InitPadmapActions() toggleGameMenu");

crate::pending_fn!(pub fn process_game_action(ctx: &mut crate::ctx::Ctx, action: super::game_controls::GameActionType), "controls/plrctrls.cpp|devilution::ProcessGameAction(const GameAction &action)");

use super::controller::ControllerButtonEvent;
use super::ControlTypes;
use crate::platform::events::Event;

/// Original: `GetInputTypeFromEvent` (controls/plrctrls.cpp). The port has no touch or
/// controller events.
// @port controls/plrctrls.cpp|devilution::GetInputTypeFromEvent(const SDL_Event &event) sha=84b859424d57
fn get_input_type_from_event(event: &Event) -> ControlTypes {
    match event {
        Event::KeyDown { .. } | Event::KeyUp { .. } => ControlTypes::KeyboardAndMouse,
        Event::MouseButtonDown { .. } | Event::MouseButtonUp { .. } | Event::MouseMotion { .. } | Event::MouseWheel { .. } => {
            ControlTypes::KeyboardAndMouse
        }
        _ => ControlTypes::None,
    }
}

/// Original: `ContinueSimulatedMouseEvent` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::ContinueSimulatedMouseEvent(const SDL_Event &event, const ControllerButtonEvent &gamepadEvent) sha=fefe20df3022
fn continue_simulated_mouse_event(ctx: &mut Ctx, _event: &Event, gamepad_event: ControllerButtonEvent) -> bool {
    if crate::automap::automap_active(ctx) {
        return false;
    }
    // Joystick events with game controllers present: no devices in the port.
    let s = &mut ctx.controls.sticks;
    if s.right_stick_x != 0.0 || s.right_stick_y != 0.0 || s.right_stick_last_move != 0.0 {
        s.right_stick_last_move = s.right_stick_x + s.right_stick_y;
        return true;
    }
    ctx.controls.sticks.simulating_mouse_with_padmapper || super::game_controls::is_simulated_mouse_click_binding(ctx, gamepad_event)
}

/// Original: `devilution::DetectInputMethod` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::DetectInputMethod(const SDL_Event &event, const ControllerButtonEvent &gamepadEvent) sha=74ab3c039662
pub fn detect_input_method(ctx: &mut Ctx, event: &Event, gamepad_event: ControllerButtonEvent) {
    let input_type = get_input_type_from_event(event);
    if input_type == ControlTypes::None {
        return;
    }
    let new_control_device = input_type;
    let mut new_control_mode = input_type;
    if continue_simulated_mouse_event(ctx, event, gamepad_event) {
        new_control_mode = ctx.controls.control_mode;
    }
    super::controller::log_control_device_and_mode_change(ctx, new_control_device, new_control_mode);
    if new_control_device != ctx.controls.control_device {
        ctx.controls.control_device = new_control_device;
        if ctx.controls.control_device != ControlTypes::KeyboardAndMouse {
            if crate::hwcursor::is_hardware_cursor(ctx) {
                crate::hwcursor::set_hardware_cursor(ctx, crate::hwcursor::CursorInfo::unknown_cursor());
            }
        } else {
            crate::cursor::reset_cursor(ctx);
        }
        // Gamepad layout detection: no controllers in the port.
    }
    if new_control_mode != ctx.controls.control_mode {
        ctx.controls.control_mode = new_control_mode;
        crate::control::calculate_panel_areas(ctx);
    }
}

pub use crate::control::focus_on_char_info;

crate::pending_fn!(pub fn try_drop_item(ctx: &mut crate::ctx::Ctx) -> bool, "controls/plrctrls.cpp|devilution::TryDropItem()");





/// `ControllerActionHeld != GameActionType_NONE`
pub fn controller_action_held(ctx: &crate::ctx::Ctx) -> bool {
    ctx.controls.controller_action_held != super::game_controls::GameActionType::None
}


use super::controller::{get_left_stick_or_dpad_direction, AxisDirection, AxisDirectionRepeater, AxisDirectionX, AxisDirectionY};
use crate::engine::geometry::{Direction, Point};

/// `RightStickAccumulator` (controls/plrctrls.cpp)
#[derive(Clone, Copy, Debug, Default)]
pub struct RightStickAccumulator {
    last_tc: u32,
    hires_dx: f32,
    hires_dy: f32,
}

impl RightStickAccumulator {
    /// Original: `RightStickAccumulator::Pool` (controls/plrctrls.cpp).
    // @port controls/plrctrls.cpp|devilution::RightStickAccumulator::Pool(int *x, int *y, int slowdown) sha=d21cef95e7d5
    fn pool(&mut self, ticks: u32, right_stick_x: f32, right_stick_y: f32, x: &mut i32, y: &mut i32, slowdown: i32) {
        let tc = ticks;
        let dtc = tc.wrapping_sub(self.last_tc) as i32;
        self.hires_dx += right_stick_x * dtc as f32;
        self.hires_dy += right_stick_y * dtc as f32;
        let dx = (self.hires_dx / slowdown as f32) as i32;
        let dy = (self.hires_dy / slowdown as f32) as i32;
        *x += dx;
        *y -= dy;
        self.last_tc = tc;
        // keep track of remainder for sub-pixel motion
        self.hires_dx -= (dx * slowdown) as f32;
        self.hires_dy -= (dy * slowdown) as f32;
    }

    /// Original: `RightStickAccumulator::Clear` (controls/plrctrls.cpp).
    // @port controls/plrctrls.cpp|devilution::RightStickAccumulator::Clear() sha=5fcd81feded3
    fn clear(&mut self, ticks: u32) {
        self.last_tc = ticks;
    }
}

/// Globals of controls/plrctrls.cpp.
pub struct PlrCtrlsState {
    /// `Slot`
    pub slot: i32,
    /// `ActiveStashSlot`
    pub active_stash_slot: Point,
    /// `PreviousInventoryColumn`
    pub previous_inventory_column: i32,
    /// `BeltReturnsToStash`
    pub belt_returns_to_stash: bool,
    /// `PointAndClickState`
    pub point_and_click_state: bool,
    /// `pcurstrig`
    pub pcurstrig: i32,
    /// `pcursmissile` (index into `Missiles`)
    pub pcursmissile: Option<usize>,
    /// `pcursquest`
    pub pcursquest: crate::enums::quest_id,
    /// `acc` (function-local static of `HandleRightStickMotion`)
    right_stick_acc: Option<RightStickAccumulator>,
    /// `lastMouseSetTick` (function-local static of `HandleRightStickMotion`)
    last_mouse_set_tick: i32,
    /// `repeater` (function-local static of `QuestLogMove`)
    quest_log_repeater: AxisDirectionRepeater,
    /// `repeater` (function-local static of `StoreMove`)
    store_repeater: AxisDirectionRepeater,
}

impl Default for PlrCtrlsState {
    fn default() -> Self {
        PlrCtrlsState {
            slot: crate::enums::SLOTXY_INV_FIRST as i32,
            active_stash_slot: Point::new(-1, -1),
            previous_inventory_column: -1,
            belt_returns_to_stash: false,
            point_and_click_state: false,
            pcurstrig: -1,
            pcursmissile: None,
            pcursquest: crate::enums::Q_INVALID,
            right_stick_acc: None,
            last_mouse_set_tick: 0,
            quest_log_repeater: AxisDirectionRepeater::default(),
            store_repeater: AxisDirectionRepeater::default(),
        }
    }
}

/// `FaceDir`
const FACE_DIR: [[Direction; 3]; 3] = [
    // NONE             UP                DOWN
    [Direction::South, Direction::North, Direction::South],        // NONE
    [Direction::West, Direction::NorthWest, Direction::SouthWest], // LEFT
    [Direction::East, Direction::NorthEast, Direction::SouthEast], // RIGHT
];

/// Original: `devilution::InGameMenu` (controls/plrctrls.cpp): native game menu, controlled by simulating a keyboard.
// @port controls/plrctrls.cpp|devilution::InGameMenu() sha=3298eab4bcb7
pub fn in_game_menu(ctx: &Ctx) -> bool {
    !crate::stores::stextflag_is_none(ctx)
        || ctx.help.HelpFlag
        || ctx.chatlog.ChatLogFlag
        || ctx.control.talkflag
        || crate::minitext::qtextflag(ctx)
        || crate::gmenu::gmenu_is_active(ctx)
        || ctx.diablo.pause_mode == 2
        || ctx.players.MyPlayer.map(|me| ctx.players.Players[me]._pInvincible && ctx.players.Players[me]._pHitPoints == 0).unwrap_or(false)
}

/// Original: `IsStandingGround` (controls/plrctrls.cpp), no virtual gamepad.
// @port controls/plrctrls.cpp|devilution::IsStandingGround() sha=8475de437fbe
fn is_standing_ground(ctx: &Ctx) -> bool {
    if ctx.controls.control_mode == ControlTypes::Gamepad {
        let stand_ground_combo = ctx.options.padmapper.button_combo_for_action("StandGround");
        return ctx.controls.stand_toggle || super::controller::is_controller_button_combo_pressed(ctx, stand_ground_combo);
    }
    false
}

/// Original: `IsPathBlocked` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::IsPathBlocked(Point position, Direction dir) sha=40d525433cab
fn is_path_blocked(ctx: &Ctx, position: Point, dir: Direction) -> bool {
    if !matches!(dir, Direction::North | Direction::East | Direction::South | Direction::West) {
        return false; // Steps along a major axis don't need to check corners
    }
    let left_step = position + crate::engine::geometry::left(dir);
    let right_step = position + crate::engine::geometry::right(dir);
    if crate::engine::path::is_tile_not_solid(ctx, left_step) && crate::engine::path::is_tile_not_solid(ctx, right_step) {
        return false;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    !crate::player::pos_ok_player(ctx, me, left_step) && !crate::player::pos_ok_player(ctx, me, right_step)
}

/// Original: `WalkInDir` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::WalkInDir(size_t playerId, AxisDirection dir) sha=1b3fe6568dd5
fn walk_in_dir(ctx: &mut Ctx, player_id: usize, dir: AxisDirection) {
    use crate::enums::*;
    if dir.x == AxisDirectionX::None && dir.y == AxisDirectionY::None {
        let p = &ctx.players.Players[player_id];
        if ctx.controls.control_mode != ControlTypes::KeyboardAndMouse && p.walkpath[0] as i32 != WALK_NONE && p.destAction == ACTION_NONE {
            let future = p.position.future;
            crate::msg::net_send_cmd_loc(ctx, player_id, true, CMD_WALKXY, future); // Stop walking
        }
        return;
    }
    let pdir = FACE_DIR[dir.x as usize][dir.y as usize];
    let future = ctx.players.Players[player_id].position.future;
    let delta = future + pdir;
    {
        let p = &mut ctx.players.Players[player_id];
        if !p.is_walking() && p.can_change_action() {
            p._pdir = pdir;
        }
    }
    if is_standing_ground(ctx) {
        if ctx.players.Players[player_id]._pmode == PM_STAND {
            crate::player::start_stand(ctx, player_id, pdir);
        }
        return;
    }
    if crate::player::pos_ok_player(ctx, player_id, delta) && is_path_blocked(ctx, future, pdir) {
        if ctx.players.Players[player_id]._pmode == PM_STAND {
            crate::player::start_stand(ctx, player_id, pdir);
        }
        return; // Don't start backtrack around obstacles
    }
    crate::msg::net_send_cmd_loc(ctx, player_id, true, CMD_WALKXY, delta);
}

/// Original: `QuestLogMove` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::QuestLogMove(AxisDirection moveDir) sha=25d783aafe01
fn quest_log_move(ctx: &mut Ctx, move_dir: AxisDirection) {
    let now = ctx.platform.ticks();
    let move_dir = ctx.controls.plrctrls.quest_log_repeater.get(now, move_dir);
    if move_dir.y == AxisDirectionY::Up {
        crate::quests::questlog_up(ctx);
    } else if move_dir.y == AxisDirectionY::Down {
        crate::quests::questlog_down(ctx);
    }
}

/// Original: `StoreMove` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::StoreMove(AxisDirection moveDir) sha=70d34b235ef4
fn store_move(ctx: &mut Ctx, move_dir: AxisDirection) {
    let now = ctx.platform.ticks();
    let move_dir = ctx.controls.plrctrls.store_repeater.get(now, move_dir);
    if move_dir.y == AxisDirectionY::Up {
        crate::stores::store_up(ctx);
    } else if move_dir.y == AxisDirectionY::Down {
        crate::stores::store_down(ctx);
    }
}

crate::pending_fn!(fn stash_move(ctx: &mut Ctx, dir: AxisDirection), "controls/plrctrls.cpp|devilution::StashMove(AxisDirection dir)");
crate::pending_fn!(fn check_inventory_move(ctx: &mut Ctx, dir: AxisDirection), "controls/plrctrls.cpp|devilution::CheckInventoryMove(AxisDirection dir)");
crate::pending_fn!(fn attr_inc_btn_snap(ctx: &mut Ctx, dir: AxisDirection), "controls/plrctrls.cpp|devilution::AttrIncBtnSnap(AxisDirection dir)");
crate::pending_fn!(fn hot_spell_move(ctx: &mut Ctx, dir: AxisDirection), "controls/plrctrls.cpp|devilution::HotSpellMove(AxisDirection dir)");
crate::pending_fn!(fn spell_book_move(ctx: &mut Ctx, dir: AxisDirection), "controls/plrctrls.cpp|devilution::SpellBookMove(AxisDirection dir)");
crate::pending_fn!(fn reset_inv_cursor_position(ctx: &mut Ctx), "controls/plrctrls.cpp|devilution::ResetInvCursorPosition()");
crate::pending_fn!(fn find_actor(ctx: &mut Ctx), "controls/plrctrls.cpp|devilution::FindActor()");
crate::pending_fn!(fn find_item_or_object(ctx: &mut Ctx), "controls/plrctrls.cpp|devilution::FindItemOrObject()");
crate::pending_fn!(fn find_trigger(ctx: &mut Ctx), "controls/plrctrls.cpp|devilution::FindTrigger()");

/// `HandleLeftStickOrDPadFn`
type HandleLeftStickOrDPadFn = fn(&mut Ctx, AxisDirection);

/// Original: `GetLeftStickOrDPadGameUIHandler` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::GetLeftStickOrDPadGameUIHandler() sha=6dd7d37adf37
fn get_left_stick_or_dpad_game_ui_handler(ctx: &Ctx) -> Option<HandleLeftStickOrDPadFn> {
    if ctx.stash.IsStashOpen {
        return Some(stash_move);
    }
    if ctx.inv.invflag {
        return Some(check_inventory_move);
    }
    if ctx.control.chrflag && ctx.players.Players[ctx.players.MyPlayer.expect("MyPlayer")]._pStatPts > 0 {
        return Some(attr_inc_btn_snap);
    }
    if ctx.control.spselflag {
        return Some(hot_spell_move);
    }
    if ctx.control.sbookflag {
        return Some(spell_book_move);
    }
    if ctx.quests.QuestLogIsOpen {
        return Some(quest_log_move);
    }
    if !crate::stores::stextflag_is_none(ctx) {
        return Some(store_move);
    }
    None
}

/// Original: `ProcessLeftStickOrDPadGameUI` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::ProcessLeftStickOrDPadGameUI() sha=b8e46f84d788
fn process_left_stick_or_dpad_game_ui(ctx: &mut Ctx) {
    if let Some(handler) = get_left_stick_or_dpad_game_ui_handler(ctx) {
        let dir = get_left_stick_or_dpad_direction(ctx, false);
        handler(ctx, dir);
    }
}

/// Original: `Movement` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::Movement(size_t playerId) sha=a203d6e41e1a
fn movement(ctx: &mut Ctx, player_id: usize) {
    if ctx.controls.pad_menu_navigator_active || ctx.controls.pad_hotspell_menu_active || in_game_menu(ctx) {
        return;
    }
    if get_left_stick_or_dpad_game_ui_handler(ctx).is_none() {
        let dir = super::game_controls::get_move_direction(ctx);
        walk_in_dir(ctx, player_id, dir);
    }
}

/// Original: `HandleRightStickMotion` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::HandleRightStickMotion() sha=1c71c3916dd4
fn handle_right_stick_motion(ctx: &mut Ctx) {
    let ticks = ctx.platform.ticks();
    let mut acc = ctx.controls.plrctrls.right_stick_acc.unwrap_or(RightStickAccumulator { last_tc: ticks, hires_dx: 0.0, hires_dy: 0.0 });
    let (rx, ry) = (ctx.controls.sticks.right_stick_x, ctx.controls.sticks.right_stick_y);
    // deadzone is handled in ScaleJoystickAxes() already
    if rx == 0.0 && ry == 0.0 {
        acc.clear(ticks);
        ctx.controls.plrctrls.right_stick_acc = Some(acc);
        return;
    }
    if crate::automap::automap_active(ctx) {
        // move map
        let mut dx = 0;
        let mut dy = 0;
        acc.pool(ticks, rx, ry, &mut dx, &mut dy, 32);
        ctx.controls.plrctrls.right_stick_acc = Some(acc);
        ctx.automap.AutomapOffset.delta_x += dy + dx;
        ctx.automap.AutomapOffset.delta_y += dy - dx;
        return;
    }
    // move cursor
    invalidate_inventory_slot(ctx);
    let (mut x, mut y) = ctx.diablo.mouse_position;
    acc.pool(ticks, rx, ry, &mut x, &mut y, 2);
    ctx.controls.plrctrls.right_stick_acc = Some(acc);
    x = x.max(0).min(ctx.dx.gn_screen_width - 1);
    y = y.max(0).min(ctx.dx.gn_screen_height - 1);
    // We avoid calling `SetCursorPos` within the same SDL tick because
    // that can cause all stick motion events to arrive before all
    // cursor position events.
    let now = ticks as i32;
    if now - ctx.controls.plrctrls.last_mouse_set_tick > 0 {
        crate::cursor::reset_cursor(ctx);
        super::set_cursor_pos(ctx, (x, y));
        let device = ctx.controls.control_device;
        super::controller::log_control_device_and_mode_change(ctx, device, ControlTypes::KeyboardAndMouse);
        ctx.controls.control_mode = ControlTypes::KeyboardAndMouse;
        ctx.controls.plrctrls.last_mouse_set_tick = now;
    }
}

/// Original: `devilution::InvalidateInventorySlot` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::InvalidateInventorySlot() sha=b1168645dd72
pub fn invalidate_inventory_slot(ctx: &mut Ctx) {
    ctx.controls.plrctrls.slot = -1;
    ctx.controls.plrctrls.active_stash_slot = Point::new(-1, -1);
}

/// Original: `devilution::FocusOnInventory` (controls/plrctrls.cpp): moves the mouse to the first inventory slot.
// @port controls/plrctrls.cpp|devilution::FocusOnInventory() sha=f8f61698465e
pub fn focus_on_inventory(ctx: &mut Ctx) {
    ctx.controls.plrctrls.slot = crate::enums::SLOTXY_INV_FIRST as i32;
    reset_inv_cursor_position(ctx);
}

/// Original: `devilution::SetPointAndClick` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::SetPointAndClick(bool value) sha=0fdb3321885b
pub fn set_point_and_click(ctx: &mut Ctx, value: bool) {
    ctx.controls.plrctrls.point_and_click_state = value;
}

/// Original: `devilution::IsPointAndClick` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::IsPointAndClick() sha=d370aa422726
pub fn is_point_and_click(ctx: &Ctx) -> bool {
    ctx.controls.plrctrls.point_and_click_state
}

/// Original: `devilution::IsMovementHandlerActive` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::IsMovementHandlerActive() sha=6e7e409cec5c
pub fn is_movement_handler_active(ctx: &Ctx) -> bool {
    get_left_stick_or_dpad_game_ui_handler(ctx).is_some()
}

/// Original: `devilution::plrctrls_after_check_curs_move` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::plrctrls_after_check_curs_move() sha=1debf341223e
pub fn plrctrls_after_check_curs_move(ctx: &mut Ctx) {
    use crate::diablo::MouseActionType;
    // check for monsters first, then items, then towners.
    if ctx.controls.control_mode == ControlTypes::KeyboardAndMouse || is_point_and_click(ctx) {
        return;
    }
    // While holding the button down we should retain target (but potentially lose it if it dies, goes out of view, etc)
    let lmba = ctx.diablo.last_mouse_button_action;
    if controller_action_held(ctx) && !matches!(lmba, MouseActionType::None | MouseActionType::Attack | MouseActionType::Spell) {
        crate::track::invalidate_targets(ctx);
        let c = &ctx.cursor;
        if c.pcursmonst == -1
            && c.ObjectUnderCursor.is_none()
            && c.pcursitem == -1
            && c.pcursinvitem == -1
            && c.pcursstashitem == crate::qol::stash::StashStruct::EmptyCell
            && c.pcursplr == -1
        {
            find_trigger(ctx);
        }
        return;
    }
    // Clear focuse set by cursor
    ctx.cursor.pcursplr = -1;
    ctx.cursor.pcursmonst = -1;
    ctx.cursor.pcursitem = -1;
    ctx.cursor.ObjectUnderCursor = None;
    ctx.controls.plrctrls.pcursmissile = None;
    ctx.controls.plrctrls.pcurstrig = -1;
    ctx.controls.plrctrls.pcursquest = crate::enums::Q_INVALID;
    ctx.cursor.cursPosition = Point::new(-1, -1);
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.players.Players[me]._pInvincible {
        return;
    }
    if ctx.doom.DoomFlag {
        return;
    }
    if !ctx.inv.invflag {
        ctx.control.info_string.clear();
        find_actor(ctx);
        find_item_or_object(ctx);
        find_trigger(ctx);
    }
}

/// Original: `devilution::plrctrls_every_frame` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::plrctrls_every_frame() sha=4d2544b11a7e
pub fn plrctrls_every_frame(ctx: &mut Ctx) {
    process_left_stick_or_dpad_game_ui(ctx);
    handle_right_stick_motion(ctx);
}

/// Original: `devilution::plrctrls_after_game_logic` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::plrctrls_after_game_logic() sha=d05bed2f2f31
pub fn plrctrls_after_game_logic(ctx: &mut Ctx) {
    let id = ctx.players.MyPlayerId;
    movement(ctx, id);
}
crate::pending_fn!(pub fn update_spell_target(ctx: &mut Ctx, spell: crate::enums::SpellID), "controls/plrctrls.cpp|devilution::UpdateSpellTarget(SpellID spell)");
