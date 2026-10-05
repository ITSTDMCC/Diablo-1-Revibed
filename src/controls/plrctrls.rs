//! `Source/controls/plrctrls` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;


// The bodies of the gamepad actions registered by `devilution::InitPadmapActions` (diablo.cpp).

use super::game_controls::GameActionType;
use crate::diablo::MouseActionType;

fn pad_action_down(ctx: &mut Ctx, action: GameActionType) {
    ctx.controls.controller_action_held = action;
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
}

/// `PrimaryAction` (pressed).
pub fn primary_action_pressed(ctx: &mut Ctx) {
    pad_action_down(ctx, GameActionType::PrimaryAction);
    perform_primary_action(ctx);
}

/// `SecondaryAction` (pressed).
pub fn secondary_action_pressed(ctx: &mut Ctx) {
    pad_action_down(ctx, GameActionType::SecondaryAction);
    perform_secondary_action(ctx);
}

/// `SpellAction` (pressed).
pub fn spell_action_pressed(ctx: &mut Ctx) {
    pad_action_down(ctx, GameActionType::CastSpell);
    perform_spell_action(ctx);
}

/// `PrimaryAction` / `SecondaryAction` / `SpellAction` (released).
pub fn controller_action_released(ctx: &mut Ctx) {
    pad_action_down(ctx, GameActionType::None);
}

/// `CancelAction` (pressed).
pub fn cancel_action_pressed(ctx: &mut Ctx) {
    if ctx.doom.DoomFlag {
        crate::doom::doom_close(ctx);
        return;
    }

    let mut action = GameActionType::None;
    if ctx.control.spselflag {
        action = GameActionType::ToggleQuickSpellMenu;
    } else if ctx.inv.invflag {
        action = GameActionType::ToggleInventory;
    } else if ctx.control.sbookflag {
        action = GameActionType::ToggleSpellBook;
    } else if ctx.quests.QuestLogIsOpen {
        action = GameActionType::ToggleQuestLog;
    } else if ctx.control.chrflag {
        action = GameActionType::ToggleCharacterInfo;
    }
    process_game_action(ctx, action);
}

/// `CancelAction` (enabled).
pub fn cancel_action_enabled(ctx: &Ctx) -> bool {
    ctx.doom.DoomFlag || ctx.control.spselflag || ctx.inv.invflag || ctx.control.sbookflag || ctx.quests.QuestLogIsOpen || ctx.control.chrflag
}

fn pad_stand_ground(ctx: &Ctx) -> bool {
    let stand_ground_combo = ctx.options.padmapper.button_combo_for_action("StandGround");
    ctx.controls.stand_toggle || super::controller::is_controller_button_combo_pressed(ctx, stand_ground_combo)
}

/// `leftMouseDown`
pub fn pad_left_mouse_down(ctx: &mut Ctx) {
    let stand_ground = pad_stand_ground(ctx);
    ctx.diablo.sgb_mouse_down = crate::enums::CLICK_LEFT;
    crate::diablo_game::left_mouse_down(ctx, if stand_ground { crate::platform::events::KMOD_SHIFT } else { 0 });
}

/// `leftMouseUp`
pub fn pad_left_mouse_up(ctx: &mut Ctx) {
    let stand_ground = pad_stand_ground(ctx);
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
    ctx.diablo.sgb_mouse_down = crate::enums::CLICK_NONE;
    crate::diablo_game::left_mouse_up(ctx, if stand_ground { crate::platform::events::KMOD_SHIFT } else { 0 });
}

/// `rightMouseDown`
pub fn pad_right_mouse_down(ctx: &mut Ctx) {
    let stand_ground = pad_stand_ground(ctx);
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
    ctx.diablo.sgb_mouse_down = crate::enums::CLICK_RIGHT;
    crate::diablo_game::right_mouse_down(ctx, stand_ground);
}

/// `rightMouseUp`
pub fn pad_right_mouse_up(ctx: &mut Ctx) {
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
    ctx.diablo.sgb_mouse_down = crate::enums::CLICK_NONE;
}

/// `toggleGameMenu`
pub fn pad_toggle_game_menu(ctx: &mut Ctx) {
    let in_menu = crate::gmenu::gmenu_is_active(ctx);
    crate::diablo_game::press_esc_key(ctx);
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
    ctx.controls.pad_hotspell_menu_active = false;
    ctx.controls.pad_menu_navigator_active = false;
    if !in_menu {
        crate::gamemenu::gamemenu_on(ctx);
    }
}

use super::controller::ControllerButtonEvent;
use super::ControlTypes;
use crate::platform::events::Event;

/// Original: `IsStickMovementSignificant` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::IsStickMovementSignificant() sha=7bb340d393bf
fn is_stick_movement_significant(ctx: &Ctx) -> bool {
    let s = &ctx.controls.sticks;
    s.left_stick_x >= 0.5 || s.left_stick_x <= -0.5 || s.left_stick_y >= 0.5 || s.left_stick_y <= -0.5 || s.right_stick_x != 0.0 || s.right_stick_y != 0.0
}

/// Original: `GetInputTypeFromEvent` (controls/plrctrls.cpp). The port has no touch input, so
/// mouse events always come from the mouse.
// @port controls/plrctrls.cpp|devilution::GetInputTypeFromEvent(const SDL_Event &event) sha=84b859424d57
fn get_input_type_from_event(ctx: &Ctx, event: &Event) -> ControlTypes {
    use crate::platform::events::pad::{AXIS_TRIGGERLEFT, AXIS_TRIGGERRIGHT};
    match *event {
        Event::KeyDown { .. } | Event::KeyUp { .. } => ControlTypes::KeyboardAndMouse,
        Event::MouseButtonDown { .. } | Event::MouseButtonUp { .. } | Event::MouseMotion { .. } | Event::MouseWheel { .. } => {
            ControlTypes::KeyboardAndMouse
        }
        Event::ControllerAxisMotion { axis, .. } => {
            if axis == AXIS_TRIGGERLEFT || axis == AXIS_TRIGGERRIGHT || is_stick_movement_significant(ctx) {
                ControlTypes::Gamepad
            } else {
                ControlTypes::None
            }
        }
        // SDL_CONTROLLERBUTTONDOWN ... SDL_CONTROLLERDEVICEREMAPPED, SDL_JOYDEVICEADDED/REMOVED
        Event::ControllerButtonDown { .. }
        | Event::ControllerButtonUp { .. }
        | Event::ControllerDeviceAdded { .. }
        | Event::ControllerDeviceRemoved { .. }
        | Event::JoyDeviceAdded { .. }
        | Event::JoyDeviceRemoved { .. } => ControlTypes::Gamepad,
        Event::JoyAxisMotion { .. } => {
            if is_stick_movement_significant(ctx) {
                ControlTypes::Gamepad
            } else {
                ControlTypes::None
            }
        }
        // SDL_JOYBALLMOTION ... SDL_JOYBUTTONUP
        Event::JoyHatMotion { .. } | Event::JoyButtonDown { .. } | Event::JoyButtonUp { .. } => ControlTypes::Gamepad,
        _ => ControlTypes::None,
    }
}

/// Original: `ContinueSimulatedMouseEvent` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::ContinueSimulatedMouseEvent(const SDL_Event &event, const ControllerButtonEvent &gamepadEvent) sha=fefe20df3022
fn continue_simulated_mouse_event(ctx: &mut Ctx, event: &Event, gamepad_event: ControllerButtonEvent) -> bool {
    if crate::automap::automap_active(ctx) {
        return false;
    }
    if matches!(event, Event::JoyAxisMotion { .. } | Event::JoyHatMotion { .. } | Event::JoyButtonDown { .. } | Event::JoyButtonUp { .. })
        && !super::devices::GameController::all(ctx).is_empty()
    {
        return true;
    }
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
    let input_type = get_input_type_from_event(ctx, event);
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
        if ctx.controls.control_device == ControlTypes::Gamepad {
            let new_gamepad_layout = super::devices::GameController::get_layout(ctx, event);
            if new_gamepad_layout != ctx.controls.gamepad_type {
                log_gamepad_change(ctx, new_gamepad_layout);
                ctx.controls.gamepad_type = new_gamepad_layout;
            }
        }
    }
    if new_control_mode != ctx.controls.control_mode {
        ctx.controls.control_mode = new_control_mode;
        crate::control::calculate_panel_areas(ctx);
    }
}

/// Original: `GamepadTypeToString` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::GamepadTypeToString(GamepadLayout gamepadLayout) sha=67adfd879f17
fn gamepad_type_to_string(gamepad_layout: super::game_controls::GamepadLayout) -> &'static str {
    use super::game_controls::GamepadLayout as L;
    match gamepad_layout {
        L::Nintendo => "Nintendo",
        L::PlayStation => "PlayStation",
        L::Xbox => "Xbox",
        L::Generic => "Unknown",
    }
}

/// Original: `LogGamepadChange` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::LogGamepadChange(GamepadLayout newGamepad) sha=4c1c790f8d7a
fn log_gamepad_change(ctx: &Ctx, new_gamepad: super::game_controls::GamepadLayout) {
    let before = ctx.controls.gamepad_type;
    let change = if before == new_gamepad {
        gamepad_type_to_string(before).to_string()
    } else {
        format!("{} -> {}", gamepad_type_to_string(before), gamepad_type_to_string(new_gamepad))
    };
    crate::platform::log::verbose!("Control: gamepad {}", change);
}

pub use crate::control::focus_on_char_info;






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
    /// Original: `RightStickAccumulator::RightStickAccumulator` (controls/plrctrls.cpp).
    // @port controls/plrctrls.cpp|devilution::RightStickAccumulator::RightStickAccumulator() sha=58b4d5877a5a
    fn new(ticks: u32) -> RightStickAccumulator {
        RightStickAccumulator { last_tc: ticks, hires_dx: 0.0, hires_dy: 0.0 }
    }

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
    /// `repeater` (function-local static of `AttrIncBtnSnap`)
    attr_repeater: AxisDirectionRepeater,
    /// `repeater` (function-local static of `CheckInventoryMove`)
    inventory_repeater: AxisDirectionRepeater,
    /// `repeater` (function-local static of `StashMove`)
    stash_repeater: AxisDirectionRepeater,
    /// `repeater` (function-local static of `HotSpellMove`)
    hot_spell_repeater: AxisDirectionRepeater,
    /// `repeater` (function-local static of `SpellBookMove`)
    spell_book_repeater: AxisDirectionRepeater,
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
            attr_repeater: AxisDirectionRepeater::default(),
            inventory_repeater: AxisDirectionRepeater::new(150),
            stash_repeater: AxisDirectionRepeater::new(150),
            hot_spell_repeater: AxisDirectionRepeater::default(),
            spell_book_repeater: AxisDirectionRepeater::default(),
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
    let mut acc = ctx.controls.plrctrls.right_stick_acc.unwrap_or_else(|| RightStickAccumulator::new(ticks));
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

use crate::engine::geometry::{Displacement, Rectangle, Size};
use crate::enums::*;
use crate::inv::{InvRect, InventorySlotSizeInPixels, INV_SLOT_HALF_SIZE_PX};

/// `INV_ROW_SLOT_SIZE` (inv.h)
const INV_ROW_SLOT_SIZE: i32 = 10;

fn my_player(ctx: &Ctx) -> usize {
    ctx.players.MyPlayer.expect("MyPlayer")
}

fn mouse(ctx: &Ctx) -> Point {
    Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1)
}

fn set_cursor(ctx: &mut Ctx, p: Point) {
    super::set_cursor_pos(ctx, (p.x, p.y));
}

/// Original: `GetRotaryDistance` (controls/plrctrls.cpp): number of angles to turn to face the coordinate.
// @port controls/plrctrls.cpp|devilution::GetRotaryDistance(Point destination) sha=31df9ecd6818
fn get_rotary_distance(ctx: &Ctx, destination: Point) -> i32 {
    let my_player = &ctx.players.Players[my_player(ctx)];
    if my_player.position.future == destination {
        return -1;
    }
    let d1 = my_player._pdir as i32;
    let d2 = crate::engine::get_direction(my_player.position.future, destination) as i32;
    let d = (d1 - d2).abs();
    if d > 4 {
        return 4 - (d % 4);
    }
    d
}

/// Original: `GetMinDistance` (controls/plrctrls.cpp): the best case walking steps to coordinates.
// @port controls/plrctrls.cpp|devilution::GetMinDistance(Point position) sha=76d32cae68ee
fn get_min_distance(ctx: &Ctx, position: Point) -> i32 {
    ctx.players.Players[my_player(ctx)].position.future.walking_distance(position)
}

/// Original: `GetDistance` (controls/plrctrls.cpp): walking steps to coordinate, or 0 if not reachable.
// @port controls/plrctrls.cpp|devilution::GetDistance(Point destination, int maxDistance) sha=dca534706a9b
fn get_distance(ctx: &Ctx, destination: Point, max_distance: i32) -> i32 {
    if get_min_distance(ctx, destination) > max_distance {
        return 0;
    }
    let mut walkpath = [0i8; crate::engine::path::MaxPathLength];
    let me = my_player(ctx);
    let start = ctx.players.Players[me].position.future;
    let steps = crate::engine::path::find_path(ctx, &|ctx: &Ctx, position: Point| crate::player::pos_ok_player(ctx, me, position), start, destination, &mut walkpath);
    if steps > max_distance {
        return 0;
    }
    steps
}

/// Original: `GetDistanceRanged` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::GetDistanceRanged(Point destination) sha=5f48ac59d90a
fn get_distance_ranged(ctx: &Ctx, destination: Point) -> i32 {
    ctx.players.Players[my_player(ctx)].position.future.exact_distance(destination)
}

/// Original: `FindItemOrObject` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::FindItemOrObject() sha=cdb136dffe7e
fn find_item_or_object(ctx: &mut Ctx) {
    let future_position = ctx.players.Players[my_player(ctx)].position.future;
    let mut rotations = 5;
    let search_area: Vec<Point> = crate::engine::geometry::points_in_rectangle_col_major(Rectangle::from_center(future_position, 1)).collect();
    for &target_position in search_area.iter() {
        // As the player can not stand on the edge of the map this is safe from OOB
        let item_id = ctx.items.dItem[target_position.x as usize][target_position.y as usize] - 1;
        if item_id < 0 {
            // there shouldn't be any items that occupy multiple ground tiles, but just in case only considering positive indexes here
            continue;
        }
        let item = &ctx.items.Items[item_id as usize];
        if item.is_empty() || item._iSelFlag == 0 {
            continue;
        }
        let new_rotations = get_rotary_distance(ctx, target_position);
        if rotations < new_rotations {
            continue;
        }
        if target_position != future_position && get_distance(ctx, target_position, 1) == 0 {
            // Don't check the tile we're leaving if the player is walking
            continue;
        }
        rotations = new_rotations;
        ctx.cursor.pcursitem = item_id;
        ctx.cursor.cursPosition = target_position;
    }
    if ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town || ctx.cursor.pcursitem != -1 {
        return; // Don't look for objects in town
    }
    for &target_position in search_area.iter() {
        let Some(oi) = crate::objects::find_object_at_position(ctx, target_position, true) else {
            continue;
        };
        let object = &ctx.objects.Objects[oi];
        if object._oSelFlag == 0 {
            // No object or non-interactive object
            continue;
        }
        if target_position == future_position && object._oDoorFlag {
            continue; // Ignore doorway so we don't get stuck behind barrels
        }
        let new_rotations = get_rotary_distance(ctx, target_position);
        if rotations < new_rotations {
            continue;
        }
        if target_position != future_position && get_distance(ctx, target_position, 1) == 0 {
            // Don't check the tile we're leaving if the player is walking
            continue;
        }
        if ctx.objects.Objects[oi].is_disabled_opt(ctx.options.gameplay.disable_crippling_shrines.get()) {
            continue;
        }
        rotations = new_rotations;
        ctx.cursor.ObjectUnderCursor = Some(oi);
        ctx.cursor.cursPosition = target_position;
    }
}

/// Original: `CheckTownersNearby` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::CheckTownersNearby() sha=4490207b7b0a
fn check_towners_nearby(ctx: &mut Ctx) {
    for i in 0..16 {
        let distance = get_distance(ctx, ctx.towners.towners[i].position, 2);
        if distance == 0 {
            continue;
        }
        ctx.cursor.pcursmonst = i as i32;
    }
}

/// Original: `HasRangedSpell` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::HasRangedSpell() sha=0f3ad3795b00
fn has_ranged_spell(ctx: &Ctx) -> bool {
    let spl = ctx.players.Players[my_player(ctx)]._pRSpell;
    spl != SpellID::Invalid
        && spl != SpellID::TownPortal
        && spl != SpellID::Teleport
        && crate::items::get_spell_data(spl).is_targeted()
        && !crate::items::get_spell_data(spl).is_allowed_in_town()
}

/// Original: `CanTargetMonster` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::CanTargetMonster(const Monster &monster) sha=2f3ac931d263
fn can_target_monster(ctx: &Ctx, m: usize) -> bool {
    let monster = &ctx.monster.Monsters[m];
    if (monster.flags & MFLAG_HIDDEN as u32) != 0 {
        return false;
    }
    if monster.is_player_minion() {
        return false;
    }
    if monster.hitPoints >> 6 <= 0 {
        // dead
        return false;
    }
    if !crate::levels::gendung::is_tile_lit(ctx, monster.position.tile) {
        // not visible
        return false;
    }
    let (mx, my) = (monster.position.tile.x as usize, monster.position.tile.y as usize);
    if ctx.gendung.dMonster[mx][my] == 0 {
        return false;
    }
    true
}

/// Original: `FindRangedTarget` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::FindRangedTarget() sha=3356015a44be
fn find_ranged_target(ctx: &mut Ctx) {
    let mut rotations = 0;
    let mut distance = 0;
    let mut can_talk = false;
    for i in 0..ctx.monster.ActiveMonsterCount {
        let mi = ctx.monster.ActiveMonsters[i] as usize;
        if !can_target_monster(ctx, mi) {
            continue;
        }
        let new_can_talk = crate::monster::can_talk_to_monst(ctx, mi);
        if ctx.cursor.pcursmonst != -1 && !can_talk && new_can_talk {
            continue;
        }
        let future = ctx.monster.Monsters[mi].position.future;
        let new_ddistance = get_distance_ranged(ctx, future);
        let new_rotations = get_rotary_distance(ctx, future);
        if ctx.cursor.pcursmonst != -1 && can_talk == new_can_talk {
            if distance < new_ddistance {
                continue;
            }
            if distance == new_ddistance && rotations < new_rotations {
                continue;
            }
        }
        distance = new_ddistance;
        rotations = new_rotations;
        can_talk = new_can_talk;
        ctx.cursor.pcursmonst = mi as i32;
    }
}

/// Original: `FindMeleeTarget` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::FindMeleeTarget() sha=175c4b255eb0
fn find_melee_target(ctx: &mut Ctx) {
    use crate::levels::gendung::{MAXDUNX, MAXDUNY};
    let mut visited = vec![[false; MAXDUNY]; MAXDUNX];
    let mut max_steps = 25; // Max steps for FindPath is 25
    let mut rotations = 0;
    let mut can_talk = false;
    let mut queue: std::collections::VecDeque<(i32, i32, i32)> = std::collections::VecDeque::new();
    let me = my_player(ctx);
    {
        let start = ctx.players.Players[me].position.future;
        visited[start.x as usize][start.y as usize] = true;
        queue.push_back((start.x, start.y, 0));
    }
    while let Some((nx, ny, steps)) = queue.pop_front() {
        for path_dir in crate::engine::path::PATH_DIRS {
            let dx = nx + path_dir.delta_x;
            let dy = ny + path_dir.delta_y;
            if visited[dx as usize][dy as usize] {
                continue; // already visisted
            }
            if steps > max_steps {
                visited[dx as usize][dy as usize] = true;
                continue;
            }
            if !crate::player::pos_ok_player(ctx, me, Point::new(dx, dy)) {
                visited[dx as usize][dy as usize] = true;
                let dm = ctx.gendung.dMonster[dx as usize][dy as usize];
                if dm != 0 {
                    let mi = (dm as i32).unsigned_abs() as usize - 1;
                    if can_target_monster(ctx, mi) {
                        let new_can_talk = crate::monster::can_talk_to_monst(ctx, mi);
                        if ctx.cursor.pcursmonst != -1 && !can_talk && new_can_talk {
                            continue;
                        }
                        let new_rotations = get_rotary_distance(ctx, Point::new(dx, dy));
                        if ctx.cursor.pcursmonst != -1 && can_talk == new_can_talk && rotations < new_rotations {
                            continue;
                        }
                        rotations = new_rotations;
                        can_talk = new_can_talk;
                        ctx.cursor.pcursmonst = mi as i32;
                        if !can_talk {
                            max_steps = steps; // Monsters found, cap search to current steps
                        }
                    }
                }
                continue;
            }
            if crate::engine::path::path_solid_pieces(ctx, Point::new(nx, ny), Point::new(dx, dy)) {
                queue.push_back((dx, dy, steps + 1));
                visited[dx as usize][dy as usize] = true;
            }
        }
    }
}

/// Original: `CheckMonstersNearby` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::CheckMonstersNearby() sha=19a0a8ab38a2
fn check_monsters_nearby(ctx: &mut Ctx) {
    if ctx.players.Players[my_player(ctx)].uses_ranged_weapon() || has_ranged_spell(ctx) {
        find_ranged_target(ctx);
        return;
    }
    find_melee_target(ctx);
}

/// Original: `CheckPlayerNearby` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::CheckPlayerNearby() sha=0eb474a67f06
fn check_player_nearby(ctx: &mut Ctx) {
    let mut rotations = 0;
    let mut distance = 0;
    if ctx.cursor.pcursmonst != -1 {
        return;
    }
    let me = my_player(ctx);
    let spl = ctx.players.Players[me]._pRSpell;
    if ctx.players.Players[me].friendlyMode && spl != SpellID::Resurrect && spl != SpellID::HealOther {
        return;
    }
    for i in 0..ctx.players.Players.len() {
        if i == me {
            continue;
        }
        let future = ctx.players.Players[i].position.future;
        let hp = ctx.players.Players[i]._pHitPoints;
        if ctx.gendung.dPlayer[future.x as usize][future.y as usize] == 0 || !crate::levels::gendung::is_tile_lit(ctx, future) || (hp == 0 && spl != SpellID::Resurrect) {
            continue;
        }
        let new_ddistance;
        if ctx.players.Players[me].uses_ranged_weapon() || has_ranged_spell(ctx) || spl == SpellID::HealOther {
            new_ddistance = get_distance_ranged(ctx, future);
        } else {
            new_ddistance = get_distance(ctx, future, distance);
            if new_ddistance == 0 {
                continue;
            }
        }
        if ctx.cursor.pcursplr != -1 && distance < new_ddistance {
            continue;
        }
        let new_rotations = get_rotary_distance(ctx, future);
        if ctx.cursor.pcursplr != -1 && distance == new_ddistance && rotations < new_rotations {
            continue;
        }
        distance = new_ddistance;
        rotations = new_rotations;
        ctx.cursor.pcursplr = i as i8;
    }
}

/// Original: `FindActor` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::FindActor() sha=5bf9532e797e
fn find_actor(ctx: &mut Ctx) {
    if ctx.gendung.leveltype != crate::levels::gendung::DungeonType::Town {
        check_monsters_nearby(ctx);
    } else {
        check_towners_nearby(ctx);
    }
    if ctx.init.gb_is_multiplayer {
        check_player_nearby(ctx);
    }
}

/// Original: `FindTrigger` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::FindTrigger() sha=cc527586bb61
fn find_trigger(ctx: &mut Ctx) {
    let mut rotations = 0;
    let mut distance = 0;
    if ctx.cursor.pcursitem != -1 || ctx.cursor.ObjectUnderCursor.is_some() {
        return; // Prefer showing items/objects over triggers (use of cursm* conflicts)
    }
    for mi in 0..ctx.missiles.Missiles.len() {
        let missile = &ctx.missiles.Missiles[mi];
        if missile._mitype == MissileID::TownPortal || missile._mitype == MissileID::RedPortal {
            let tile = missile.position.tile;
            let new_distance = get_distance(ctx, tile, 2);
            if new_distance == 0 {
                continue;
            }
            if ctx.controls.plrctrls.pcursmissile.is_some() && distance < new_distance {
                continue;
            }
            let new_rotations = get_rotary_distance(ctx, tile);
            if ctx.controls.plrctrls.pcursmissile.is_some() && distance == new_distance && rotations < new_rotations {
                continue;
            }
            ctx.cursor.cursPosition = tile;
            ctx.controls.plrctrls.pcursmissile = Some(mi);
            distance = new_distance;
            rotations = new_rotations;
        }
    }
    if ctx.controls.plrctrls.pcursmissile.is_none() {
        for i in 0..ctx.trigs.numtrigs as usize {
            let tx = ctx.trigs.trigs[i].position.x;
            let mut ty = ctx.trigs.trigs[i].position.y;
            if ctx.trigs.trigs[i]._tlvl == 13 {
                ty -= 1;
            }
            let new_distance = get_distance(ctx, Point::new(tx, ty), 2);
            if new_distance == 0 {
                continue;
            }
            ctx.cursor.cursPosition = Point::new(tx, ty);
            ctx.controls.plrctrls.pcurstrig = i as i32;
        }
        if ctx.controls.plrctrls.pcurstrig == -1 {
            let quests = ctx.quests.Quests;
            for quest in quests.iter() {
                if quest._qidx == Q_BETRAYER || ctx.gendung.currlevel != quest._qlevel || quest._qslvl == 0 {
                    continue;
                }
                let new_distance = get_distance(ctx, quest.position, 2);
                if new_distance == 0 {
                    continue;
                }
                ctx.cursor.cursPosition = quest.position;
                ctx.controls.plrctrls.pcursquest = quest._qidx;
            }
        }
    }
    let c = &ctx.cursor;
    if c.pcursmonst != -1 || c.pcursplr != -1 || c.cursPosition.x == -1 || c.cursPosition.y == -1 {
        return; // Prefer monster/player info text
    }
    crate::levels::trigs::check_trig_force(ctx);
    crate::cursor::check_town(ctx);
    crate::cursor::check_rportal(ctx);
}

/// Original: `Interact` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::Interact() sha=f67f17d2aa4a
fn interact(ctx: &mut Ctx) {
    use crate::diablo::MouseActionType;
    let town = ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town;
    if town && ctx.cursor.pcursmonst != -1 {
        let m = ctx.cursor.pcursmonst;
        let pos = ctx.towners.towners[m as usize].position;
        crate::msg::net_send_cmd_loc_param1(ctx, true, CMD_TALKXY, pos, m as u16);
        return;
    }
    let me = my_player(ctx);
    let my_id = ctx.players.MyPlayerId;
    if !town && is_standing_ground(ctx) {
        let mut pdir = ctx.players.Players[me]._pdir;
        let move_dir = super::game_controls::get_move_direction(ctx);
        let motion = move_dir.x != AxisDirectionX::None || move_dir.y != AxisDirectionY::None;
        if motion {
            pdir = FACE_DIR[move_dir.x as usize][move_dir.y as usize];
        }
        let mut position = ctx.players.Players[me].position.tile + pdir;
        if ctx.cursor.pcursmonst != -1 && !motion {
            position = ctx.monster.Monsters[ctx.cursor.pcursmonst as usize].position.tile;
        }
        let cmd = if ctx.players.Players[me].uses_ranged_weapon() { CMD_RATTACKXY } else { CMD_SATTACKXY };
        crate::msg::net_send_cmd_loc(ctx, my_id, true, cmd, position);
        ctx.diablo.last_mouse_button_action = MouseActionType::Attack;
        return;
    }
    if ctx.cursor.pcursmonst != -1 {
        let m = ctx.cursor.pcursmonst as usize;
        if !ctx.players.Players[me].uses_ranged_weapon() || crate::monster::can_talk_to_monst(ctx, m) {
            crate::msg::net_send_cmd_param1(ctx, true, CMD_ATTACKID, m as u16);
        } else {
            crate::msg::net_send_cmd_param1(ctx, true, CMD_RATTACKID, m as u16);
        }
        ctx.diablo.last_mouse_button_action = MouseActionType::AttackMonsterTarget;
        return;
    }
    if !town && ctx.cursor.pcursplr != -1 && !ctx.players.Players[me].friendlyMode {
        let cmd = if ctx.players.Players[me].uses_ranged_weapon() { CMD_RATTACKPID } else { CMD_ATTACKPID };
        let p = ctx.cursor.pcursplr as u16;
        crate::msg::net_send_cmd_param1(ctx, true, cmd, p);
        ctx.diablo.last_mouse_button_action = MouseActionType::AttackPlayerTarget;
        return;
    }
    if ctx.cursor.ObjectUnderCursor.is_some() {
        let curs = ctx.cursor.cursPosition;
        crate::msg::net_send_cmd_loc(ctx, my_id, true, CMD_OPOBJXY, curs);
        ctx.diablo.last_mouse_button_action = MouseActionType::OperateObject;
    }
}

/// Original: `AttrIncBtnSnap` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::AttrIncBtnSnap(AxisDirection dir) sha=2cc87222618c
fn attr_inc_btn_snap(ctx: &mut Ctx, dir: AxisDirection) {
    let now = ctx.platform.ticks();
    let dir = ctx.controls.plrctrls.attr_repeater.get(now, dir);
    if dir.y == AxisDirectionY::None {
        return;
    }
    if ctx.control.chrbtnactive && ctx.players.Players[my_player(ctx)]._pStatPts <= 0 {
        return;
    }
    // first, find our cursor location
    let mut slot = 0;
    for i in 0..4 {
        let mut button = crate::control::CHR_BTNS_RECT[i];
        button.position = crate::control::get_panel_position(ctx, UiPanels::Character, button.position);
        if button.contains(mouse(ctx)) {
            slot = i;
            break;
        }
    }
    if dir.y == AxisDirectionY::Up {
        if slot > 0 {
            slot -= 1;
        }
    } else if dir.y == AxisDirectionY::Down && slot < 3 {
        slot += 1;
    }
    // move cursor to our new location
    let mut button = crate::control::CHR_BTNS_RECT[slot];
    button.position = crate::control::get_panel_position(ctx, UiPanels::Character, button.position);
    set_cursor(ctx, button.center());
}

/// Original: `InvGetEquipSlotCoord` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::InvGetEquipSlotCoord(const inv_body_loc invSlot) sha=915068c05d19
fn inv_get_equip_slot_coord(ctx: &Ctx, inv_slot: inv_body_loc) -> Point {
    let mut result = crate::control::get_panel_position(ctx, UiPanels::Inventory, Point::new(0, 0));
    let xy = match inv_slot {
        INVLOC_HEAD => Some(SLOTXY_HEAD),
        INVLOC_RING_LEFT => Some(SLOTXY_RING_LEFT),
        INVLOC_RING_RIGHT => Some(SLOTXY_RING_RIGHT),
        INVLOC_AMULET => Some(SLOTXY_AMULET),
        INVLOC_HAND_LEFT => Some(SLOTXY_HAND_LEFT),
        INVLOC_HAND_RIGHT => Some(SLOTXY_HAND_RIGHT),
        INVLOC_CHEST => Some(SLOTXY_CHEST),
        _ => None,
    };
    if let Some(xy) = xy {
        let c = InvRect[xy as usize].center();
        result.x += c.x;
        result.y += c.y;
    }
    result
}

/// Original: `InvGetEquipSlotCoordFromInvSlot` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::InvGetEquipSlotCoordFromInvSlot(const inv_xy_slot slot) sha=27085b5ff6b1
fn inv_get_equip_slot_coord_from_inv_slot(ctx: &Ctx, slot: i32) -> Point {
    let loc = match slot {
        s if s == SLOTXY_HEAD as i32 => INVLOC_HEAD,
        s if s == SLOTXY_RING_LEFT as i32 => INVLOC_RING_LEFT,
        s if s == SLOTXY_RING_RIGHT as i32 => INVLOC_RING_RIGHT,
        s if s == SLOTXY_AMULET as i32 => INVLOC_AMULET,
        s if s == SLOTXY_HAND_LEFT as i32 => INVLOC_HAND_LEFT,
        s if s == SLOTXY_HAND_RIGHT as i32 => INVLOC_HAND_RIGHT,
        s if s == SLOTXY_CHEST as i32 => INVLOC_CHEST,
        _ => return Point::default(),
    };
    inv_get_equip_slot_coord(ctx, loc)
}

/// Original: `GetSlotCoord` (controls/plrctrls.cpp): coordinates for a given slot.
// @port controls/plrctrls.cpp|devilution::GetSlotCoord(int slot) sha=2c833db7378b
fn get_slot_coord(ctx: &Ctx, slot: i32) -> Point {
    if slot >= SLOTXY_BELT_FIRST as i32 && slot <= SLOTXY_BELT_LAST as i32 {
        return crate::control::get_panel_position(ctx, UiPanels::Main, InvRect[slot as usize].center());
    }
    crate::control::get_panel_position(ctx, UiPanels::Inventory, InvRect[slot as usize].center())
}

/// Original: `GetItemIdOnSlot` (controls/plrctrls.cpp): the item id of the current slot.
// @port controls/plrctrls.cpp|devilution::GetItemIdOnSlot(int slot) sha=64e15a9aaab9
fn get_item_id_on_slot(ctx: &Ctx, slot: i32) -> i32 {
    if slot >= SLOTXY_INV_FIRST as i32 && slot <= SLOTXY_INV_LAST as i32 {
        return (ctx.players.Players[my_player(ctx)].InvGrid[(slot - SLOTXY_INV_FIRST as i32) as usize] as i32).abs();
    }
    0
}

/// Original: `GetItemSizeOnSlot` (controls/plrctrls.cpp): item size (grid size) on the slot; 1x1 if none exists.
// @port controls/plrctrls.cpp|devilution::GetItemSizeOnSlot(int slot) sha=0cfe4d2e081e
fn get_item_size_on_slot(ctx: &Ctx, slot: i32) -> Size {
    if slot >= SLOTXY_INV_FIRST as i32 && slot <= SLOTXY_INV_LAST as i32 {
        let ii = get_item_id_on_slot(ctx, slot) as i8;
        if ii != 0 {
            let item = &ctx.players.Players[my_player(ctx)].InvList[(ii - 1) as usize];
            if !item.is_empty() {
                return crate::inv::get_inventory_size(item);
            }
        }
    }
    Size::new(1, 1)
}

/// Original: `FindFirstSlotOnItem` (controls/plrctrls.cpp): the first slot occupied by an item in the inventory.
// @port controls/plrctrls.cpp|devilution::FindFirstSlotOnItem(int8_t itemInvId) sha=faaa3ce18806
fn find_first_slot_on_item(ctx: &Ctx, item_inv_id: i8) -> i32 {
    if item_inv_id == 0 {
        return -1;
    }
    for s in SLOTXY_INV_FIRST as i32..SLOTXY_INV_LAST as i32 {
        if get_item_id_on_slot(ctx, s) == item_inv_id as i32 {
            return s;
        }
    }
    -1
}

const INVALID_STASH_POINT: Point = Point::new(-1, -1);

/// Original: `FindFirstStashSlotOnItem` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::FindFirstStashSlotOnItem(StashStruct::StashCell itemInvId) sha=4c953b13bc0c
fn find_first_stash_slot_on_item(ctx: &mut Ctx, item_inv_id: u16) -> Point {
    if item_inv_id == crate::qol::stash::StashStruct::EmptyCell {
        return INVALID_STASH_POINT;
    }
    for point in crate::engine::geometry::points_in_rectangle(Rectangle::new(Point::new(0, 0), Size::new(10, 10))) {
        if ctx.stash.Stash.get_item_id_at_position(point) == item_inv_id {
            return point;
        }
    }
    INVALID_STASH_POINT
}

/// Original: `ResetInvCursorPosition` (controls/plrctrls.cpp): reset cursor position based on the current slot.
// @port controls/plrctrls.cpp|devilution::ResetInvCursorPosition() sha=c86061186b32
fn reset_inv_cursor_position(ctx: &mut Ctx) {
    let slot = ctx.controls.plrctrls.slot;
    let mut mouse_pos;
    if slot >= SLOTXY_INV_FIRST as i32 && slot <= SLOTXY_INV_LAST as i32 {
        let item_inv_id = get_item_id_on_slot(ctx, slot) as i8;
        if item_inv_id != 0 {
            mouse_pos = get_slot_coord(ctx, find_first_slot_on_item(ctx, item_inv_id));
            let item_size = get_item_size_on_slot(ctx, slot);
            mouse_pos.x += ((item_size.width - 1) * InventorySlotSizeInPixels.width) / 2;
            mouse_pos.y += ((item_size.height - 1) * InventorySlotSizeInPixels.height) / 2;
        } else {
            mouse_pos = get_slot_coord(ctx, slot);
        }
    } else if slot >= SLOTXY_BELT_FIRST as i32 && slot <= SLOTXY_BELT_LAST as i32 {
        mouse_pos = get_slot_coord(ctx, slot);
    } else {
        mouse_pos = inv_get_equip_slot_coord_from_inv_slot(ctx, slot);
    }
    set_cursor(ctx, mouse_pos);
}

/// Original: `FindClosestInventorySlot` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::FindClosestInventorySlot(Point mousePos) sha=152faa4b2d30
fn find_closest_inventory_slot(ctx: &Ctx, mouse_pos: Point) -> i32 {
    let mut shortest_distance = i32::MAX;
    let mut best_slot = 0;
    for i in 0..NUM_XY_SLOTS as i32 {
        let distance = mouse_pos.manhattan_distance(get_slot_coord(ctx, i));
        if distance < shortest_distance {
            shortest_distance = distance;
            best_slot = i;
        }
    }
    best_slot
}

/// Original: `FindClosestStashSlot` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::FindClosestStashSlot(Point mousePos) sha=89d4b6db912c
fn find_closest_stash_slot(ctx: &Ctx, mouse_pos: Point) -> Point {
    let mut shortest_distance = i32::MAX;
    let mut best_slot = Point::default();
    for point in crate::engine::geometry::points_in_rectangle(Rectangle::new(Point::new(0, 0), Size::new(10, 10))) {
        let distance = mouse_pos.manhattan_distance(crate::qol::stash::get_stash_slot_coord(ctx, point));
        if distance < shortest_distance {
            shortest_distance = distance;
            best_slot = point;
        }
    }
    best_slot
}

/// Original: `InventoryMoveToBody` (controls/plrctrls.cpp): where on the body to move when on the first row.
// @port controls/plrctrls.cpp|devilution::InventoryMoveToBody(int slot) sha=e36e9a612a27
fn inventory_move_to_body(ctx: &mut Ctx, slot: i32) -> i32 {
    ctx.controls.plrctrls.previous_inventory_column = slot - SLOTXY_INV_ROW1_FIRST as i32;
    if slot <= SLOTXY_INV_ROW1_FIRST as i32 + 2 {
        // first 3 general slots
        return SLOTXY_RING_LEFT as i32;
    }
    if slot <= SLOTXY_INV_ROW1_FIRST as i32 + 6 {
        // middle 4 general slots
        return SLOTXY_CHEST as i32;
    }
    // last 3 general slots
    SLOTXY_RING_RIGHT as i32
}

const ROW_FIRSTS: [i32; 5] = [SLOTXY_INV_ROW1_FIRST as i32, SLOTXY_INV_ROW2_FIRST as i32, SLOTXY_INV_ROW3_FIRST as i32, SLOTXY_INV_ROW4_FIRST as i32, SLOTXY_BELT_FIRST as i32];
const ROW_LASTS: [i32; 5] = [SLOTXY_INV_ROW1_LAST as i32, SLOTXY_INV_ROW2_LAST as i32, SLOTXY_INV_ROW3_LAST as i32, SLOTXY_INV_ROW4_LAST as i32, SLOTXY_BELT_LAST as i32];

/// Original: `InventoryMove` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::InventoryMove(AxisDirection dir) sha=6b16f5365a1d
fn inventory_move(ctx: &mut Ctx, dir: AxisDirection) {
    let mut mouse_pos = mouse(ctx);
    let mut slot = ctx.controls.plrctrls.slot;
    // normalize slots
    if slot < 0 {
        slot = find_closest_inventory_slot(ctx, mouse_pos);
    } else if slot > SLOTXY_BELT_LAST as i32 {
        slot = SLOTXY_BELT_LAST as i32;
    }
    let initial_slot = slot;
    let held_item = ctx.players.Players[my_player(ctx)].HoldItem.clone();
    let is_holding_item = !held_item.is_empty();
    let mut item_size = if is_holding_item { crate::inv::get_inventory_size(&held_item) } else { Size::new(1, 1) };
    let inv_first = SLOTXY_INV_FIRST as i32;
    let inv_last = SLOTXY_INV_LAST as i32;
    let belt_last = SLOTXY_BELT_LAST as i32;
    let row1_first = SLOTXY_INV_ROW1_FIRST as i32;
    let row1_last = SLOTXY_INV_ROW1_LAST as i32;
    let row2_first = SLOTXY_INV_ROW2_FIRST as i32;
    let row4_last = SLOTXY_INV_ROW4_LAST as i32;

    // when item is on cursor (pcurs > 1), this is the real cursor XY
    if dir.x == AxisDirectionX::Left {
        if is_holding_item {
            if slot >= inv_first && slot <= belt_last {
                if !ROW_FIRSTS.contains(&slot) {
                    slot -= 1;
                }
            } else if held_item._itype == ItemType::Ring {
                slot = SLOTXY_RING_LEFT as i32;
            } else if held_item.is_weapon() || held_item.is_shield() {
                slot = SLOTXY_HAND_LEFT as i32;
            }
        } else if slot == SLOTXY_HAND_RIGHT as i32 {
            slot = SLOTXY_CHEST as i32;
        } else if slot == SLOTXY_CHEST as i32 {
            slot = SLOTXY_HAND_LEFT as i32;
        } else if slot == SLOTXY_AMULET as i32 {
            slot = SLOTXY_HEAD as i32;
        } else if slot == SLOTXY_RING_RIGHT as i32 {
            slot = SLOTXY_RING_LEFT as i32;
        } else if slot >= inv_first && slot <= belt_last {
            let item_id = get_item_id_on_slot(ctx, slot) as i8;
            if item_id != 0 {
                let mut i = 1;
                while i < INV_ROW_SLOT_SIZE && !ROW_FIRSTS.contains(&(slot - i + 1)) {
                    if item_id as i32 != get_item_id_on_slot(ctx, slot - i) {
                        slot -= i;
                        break;
                    }
                    i += 1;
                }
            } else if !ROW_FIRSTS.contains(&slot) {
                slot -= 1;
            }
        }
    } else if dir.x == AxisDirectionX::Right {
        if is_holding_item {
            if slot >= inv_first && slot <= belt_last {
                if !ROW_LASTS.contains(&(slot + item_size.width - 1)) {
                    slot += 1;
                }
            } else if held_item._itype == ItemType::Ring {
                slot = SLOTXY_RING_RIGHT as i32;
            } else if held_item.is_weapon() || held_item.is_shield() {
                slot = SLOTXY_HAND_RIGHT as i32;
            }
        } else if slot == SLOTXY_RING_LEFT as i32 {
            slot = SLOTXY_RING_RIGHT as i32;
        } else if slot == SLOTXY_HAND_LEFT as i32 {
            slot = SLOTXY_CHEST as i32;
        } else if slot == SLOTXY_CHEST as i32 {
            slot = SLOTXY_HAND_RIGHT as i32;
        } else if slot == SLOTXY_HEAD as i32 {
            slot = SLOTXY_AMULET as i32;
        } else if slot >= inv_first && slot <= belt_last {
            let item_id = get_item_id_on_slot(ctx, slot) as i8;
            if item_id != 0 {
                let mut i = 1;
                while i < INV_ROW_SLOT_SIZE && !ROW_LASTS.contains(&(slot + i - 1)) {
                    if item_id as i32 != get_item_id_on_slot(ctx, slot + i) {
                        slot += i;
                        break;
                    }
                    i += 1;
                }
            } else if !ROW_LASTS.contains(&slot) {
                slot += 1;
            }
        }
    }
    if dir.y == AxisDirectionY::Up {
        if is_holding_item {
            if slot >= row2_first {
                // general inventory
                slot -= INV_ROW_SLOT_SIZE;
            } else if slot >= inv_first {
                if held_item._itype == ItemType::Ring {
                    if slot >= row1_first && slot <= row1_first + (INV_ROW_SLOT_SIZE / 2) - 1 {
                        slot = SLOTXY_RING_LEFT as i32;
                    } else {
                        slot = SLOTXY_RING_RIGHT as i32;
                    }
                } else if held_item.is_weapon() {
                    slot = SLOTXY_HAND_LEFT as i32;
                } else if held_item.is_shield() {
                    slot = SLOTXY_HAND_RIGHT as i32;
                } else if held_item.is_helm() {
                    slot = SLOTXY_HEAD as i32;
                } else if held_item.is_armor() {
                    slot = SLOTXY_CHEST as i32;
                } else if held_item._itype == ItemType::Amulet {
                    slot = SLOTXY_AMULET as i32;
                }
            }
        } else if slot >= row1_first && slot <= row1_last {
            slot = inventory_move_to_body(ctx, slot);
        } else if slot == SLOTXY_CHEST as i32 || slot == SLOTXY_HAND_LEFT as i32 {
            slot = SLOTXY_HEAD as i32;
        } else if slot == SLOTXY_RING_LEFT as i32 {
            slot = SLOTXY_HAND_LEFT as i32;
        } else if slot == SLOTXY_RING_RIGHT as i32 {
            slot = SLOTXY_HAND_RIGHT as i32;
        } else if slot == SLOTXY_HAND_RIGHT as i32 {
            slot = SLOTXY_AMULET as i32;
        } else if slot >= row2_first {
            let item_id = get_item_id_on_slot(ctx, slot) as i8;
            if item_id != 0 {
                for i in 1..5 {
                    if slot - i * INV_ROW_SLOT_SIZE < row1_first {
                        slot = inventory_move_to_body(ctx, slot - (i - 1) * INV_ROW_SLOT_SIZE);
                        break;
                    }
                    if item_id as i32 != get_item_id_on_slot(ctx, slot - i * INV_ROW_SLOT_SIZE) {
                        slot -= i * INV_ROW_SLOT_SIZE;
                        break;
                    }
                }
            } else {
                slot -= INV_ROW_SLOT_SIZE;
            }
        }
    } else if dir.y == AxisDirectionY::Down {
        let prev_col = ctx.controls.plrctrls.previous_inventory_column;
        if is_holding_item {
            if slot == SLOTXY_HEAD as i32 || slot == SLOTXY_CHEST as i32 {
                slot = row1_first + 4;
            } else if slot == SLOTXY_RING_LEFT as i32 || slot == SLOTXY_HAND_LEFT as i32 {
                slot = row1_first + if item_size.width > 1 { 0 } else { 1 };
            } else if slot == SLOTXY_RING_RIGHT as i32 || slot == SLOTXY_HAND_RIGHT as i32 || slot == SLOTXY_AMULET as i32 {
                slot = row1_last - 1;
            } else if slot <= (row4_last - (item_size.height * INV_ROW_SLOT_SIZE)) {
                slot += INV_ROW_SLOT_SIZE;
            } else if slot <= inv_last && held_item._itype == ItemType::Misc && item_size == Size::new(1, 1) {
                // forcing only 1x1 misc items
                if slot + INV_ROW_SLOT_SIZE <= belt_last {
                    slot += INV_ROW_SLOT_SIZE;
                }
            }
        } else if slot == SLOTXY_HEAD as i32 {
            slot = SLOTXY_CHEST as i32;
        } else if slot == SLOTXY_CHEST as i32 {
            if (3..=6).contains(&prev_col) {
                slot = row1_first + prev_col;
            } else {
                slot = row1_first + (INV_ROW_SLOT_SIZE / 2);
            }
        } else if slot == SLOTXY_HAND_LEFT as i32 {
            slot = SLOTXY_RING_LEFT as i32;
        } else if slot == SLOTXY_RING_LEFT as i32 {
            if (0..=2).contains(&prev_col) {
                slot = row1_first + prev_col;
            } else {
                slot = row1_first + 1;
            }
        } else if slot == SLOTXY_RING_RIGHT as i32 {
            if (7..=9).contains(&prev_col) {
                slot = row1_first + prev_col;
            } else {
                slot = row1_last - 1;
            }
        } else if slot == SLOTXY_AMULET as i32 {
            slot = SLOTXY_HAND_RIGHT as i32;
        } else if slot == SLOTXY_HAND_RIGHT as i32 {
            slot = SLOTXY_RING_RIGHT as i32;
        } else if slot <= inv_last {
            let item_id = get_item_id_on_slot(ctx, slot) as i8;
            if item_id != 0 {
                let mut i = 1;
                while i < 5 && slot + i * INV_ROW_SLOT_SIZE <= belt_last {
                    if item_id as i32 != get_item_id_on_slot(ctx, slot + i * INV_ROW_SLOT_SIZE) {
                        slot += i * INV_ROW_SLOT_SIZE;
                        break;
                    }
                    i += 1;
                }
            } else if slot + INV_ROW_SLOT_SIZE <= belt_last {
                slot += INV_ROW_SLOT_SIZE;
            }
        }
    }
    ctx.controls.plrctrls.slot = slot;

    // no movement was made
    if slot == initial_slot {
        return;
    }
    if slot < inv_first {
        mouse_pos = inv_get_equip_slot_coord_from_inv_slot(ctx, slot);
    } else {
        mouse_pos = get_slot_coord(ctx, slot);
    }
    // If we're in the inventory we may need to move the cursor to an area that doesn't line up with the center of a cell
    if slot >= inv_first && slot <= inv_last {
        if !is_holding_item {
            // If we're not holding an item
            let item_inv_id = get_item_id_on_slot(ctx, slot) as i8;
            if item_inv_id != 0 {
                // but the cursor moved over an item
                let mut item_slot = find_first_slot_on_item(ctx, item_inv_id);
                if item_slot < 0 {
                    item_slot = slot;
                }
                // then we need to offset the cursor so it shows over the center of the item
                mouse_pos = get_slot_coord(ctx, item_slot);
                item_size = get_item_size_on_slot(ctx, item_slot);
            }
        }
        // At this point itemSize is either the size of the cell/item the hand cursor is over, or the size of the item we're currently holding.
        // mousePos is the center of the top left cell of the item under the hand cursor, or the top left cell of the region that could fit the item we're holding.
        // either way we need to offset the mouse position to account for items (we're holding or hovering over) with a dimension larger than a single cell.
        mouse_pos.x += ((item_size.width - 1) * InventorySlotSizeInPixels.width) / 2;
        mouse_pos.y += ((item_size.height - 1) * InventorySlotSizeInPixels.height) / 2;
    }
    if mouse_pos == mouse(ctx) {
        return; // Avoid wobeling when scalled
    }
    set_cursor(ctx, mouse_pos);
}

/// Original: `CheckInventoryMove` (controls/plrctrls.cpp): move the cursor around in the inventory.
// @port controls/plrctrls.cpp|devilution::CheckInventoryMove(AxisDirection dir) sha=96d83bc2b3d3
fn check_inventory_move(ctx: &mut Ctx, dir: AxisDirection) {
    let now = ctx.platform.ticks();
    let dir = ctx.controls.plrctrls.inventory_repeater.get(now, dir);
    if dir.x == AxisDirectionX::None && dir.y == AxisDirectionY::None {
        return;
    }
    inventory_move(ctx, dir);
}

/// Original: `BlurInventory` (controls/plrctrls.cpp): try to clean the inventory related cursor
/// states; true if it is safe to close the inventory.
// @port controls/plrctrls.cpp|devilution::BlurInventory() sha=a619496d9907
fn blur_inventory(ctx: &mut Ctx) -> bool {
    let me = my_player(ctx);
    if !ctx.players.Players[me].HoldItem.is_empty() && !try_drop_item(ctx) {
        crate::player::player_say(ctx, me, HeroSpeech::WhereWouldIPutThis);
        return false;
    }
    crate::inv::close_inventory(ctx);
    if ctx.cursor.pcurs > crate::cursor::CURSOR_HAND {
        crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HAND);
    }
    if ctx.control.chrflag {
        crate::control::focus_on_char_info(ctx);
    }
    true
}

/// Original: `StashMove` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::StashMove(AxisDirection dir) sha=f13cf5125071
fn stash_move(ctx: &mut Ctx, dir: AxisDirection) {
    let now = ctx.platform.ticks();
    let mut dir = ctx.controls.plrctrls.stash_repeater.get(now, dir);
    if dir.x == AxisDirectionX::None && dir.y == AxisDirectionY::None {
        return;
    }
    let m = mouse(ctx);
    if ctx.controls.plrctrls.slot < 0 && ctx.controls.plrctrls.active_stash_slot == INVALID_STASH_POINT {
        let inv_slot = find_closest_inventory_slot(ctx, m);
        let inv_slot_coord = get_slot_coord(ctx, inv_slot);
        let inv_distance = m.manhattan_distance(inv_slot_coord);
        let stash_slot = find_closest_stash_slot(ctx, m);
        let stash_slot_coord = crate::qol::stash::get_stash_slot_coord(ctx, stash_slot);
        let stash_distance = m.manhattan_distance(stash_slot_coord);
        if inv_distance < stash_distance {
            ctx.controls.plrctrls.belt_returns_to_stash = false;
            inventory_move(ctx, dir);
            return;
        }
        ctx.controls.plrctrls.active_stash_slot = stash_slot;
    }
    let me = my_player(ctx);
    let hold_item = ctx.players.Players[me].HoldItem.clone();
    let item_size = if hold_item.is_empty() { Size::new(1, 1) } else { crate::inv::get_inventory_size(&hold_item) };
    let slot = ctx.controls.plrctrls.slot;
    // Jump from belt to stash
    if ctx.controls.plrctrls.belt_returns_to_stash && slot >= SLOTXY_BELT_FIRST as i32 && slot <= SLOTXY_BELT_LAST as i32 && dir.y == AxisDirectionY::Up {
        let belt_slot = slot - SLOTXY_BELT_FIRST as i32;
        invalidate_inventory_slot(ctx);
        ctx.controls.plrctrls.active_stash_slot = Point::new(2 + belt_slot, 10 - item_size.height);
        dir.y = AxisDirectionY::None;
    }
    // Jump from general inventory to stash
    let slot = ctx.controls.plrctrls.slot;
    if slot >= SLOTXY_INV_FIRST as i32 && slot <= SLOTXY_INV_LAST as i32 {
        let mut first_slot = slot;
        if hold_item.is_empty() {
            let item_id = get_item_id_on_slot(ctx, slot) as i8;
            if item_id != 0 {
                first_slot = find_first_slot_on_item(ctx, item_id);
            }
        }
        if ROW_FIRSTS[..4].contains(&first_slot) && dir.x == AxisDirectionX::Left {
            let slot_coord = get_slot_coord(ctx, slot);
            invalidate_inventory_slot(ctx);
            ctx.controls.plrctrls.active_stash_slot = find_closest_stash_slot(ctx, slot_coord) - Displacement::new(item_size.width - 1, 0);
            dir.x = AxisDirectionX::None;
        }
    }
    let slot = ctx.controls.plrctrls.slot;
    let is_head_slot = SLOTXY_HEAD as i32 == slot;
    let is_left_hand_slot = SLOTXY_HAND_LEFT as i32 == slot;
    let is_left_ring_slot = slot == SLOTXY_RING_LEFT as i32;
    if (is_head_slot || is_left_hand_slot || is_left_ring_slot) && dir.x == AxisDirectionX::Left {
        let slot_coord = get_slot_coord(ctx, slot);
        invalidate_inventory_slot(ctx);
        ctx.controls.plrctrls.active_stash_slot = find_closest_stash_slot(ctx, slot_coord) - Displacement::new(item_size.width - 1, 0);
        dir.x = AxisDirectionX::None;
    }
    if ctx.controls.plrctrls.slot >= 0 {
        inventory_move(ctx, dir);
        return;
    }
    if dir.x == AxisDirectionX::Left {
        if ctx.controls.plrctrls.active_stash_slot.x > 0 {
            ctx.controls.plrctrls.active_stash_slot.x -= 1;
        }
    } else if dir.x == AxisDirectionX::Right {
        if ctx.controls.plrctrls.active_stash_slot.x < 10 - item_size.width {
            ctx.controls.plrctrls.active_stash_slot.x += 1;
        } else {
            let stash_slot_coord = crate::qol::stash::get_stash_slot_coord(ctx, ctx.controls.plrctrls.active_stash_slot);
            let right_panel_coord = Point::new(crate::control::get_right_panel(ctx).x, stash_slot_coord.y);
            ctx.controls.plrctrls.slot = find_closest_inventory_slot(ctx, right_panel_coord);
            ctx.controls.plrctrls.active_stash_slot = INVALID_STASH_POINT;
            ctx.controls.plrctrls.belt_returns_to_stash = false;
        }
    }
    if dir.y == AxisDirectionY::Up {
        if ctx.controls.plrctrls.active_stash_slot.y > 0 {
            ctx.controls.plrctrls.active_stash_slot.y -= 1;
        }
    } else if dir.y == AxisDirectionY::Down {
        if ctx.controls.plrctrls.active_stash_slot.y < 10 - item_size.height {
            ctx.controls.plrctrls.active_stash_slot.y += 1;
        } else if (hold_item.is_empty() || crate::inv::can_be_placed_on_belt(ctx, &hold_item)) && ctx.controls.plrctrls.active_stash_slot.x > 1 {
            let belt_slot = ctx.controls.plrctrls.active_stash_slot.x - 2;
            ctx.controls.plrctrls.slot = SLOTXY_BELT_FIRST as i32 + belt_slot;
            ctx.controls.plrctrls.active_stash_slot = INVALID_STASH_POINT;
            ctx.controls.plrctrls.belt_returns_to_stash = true;
        }
    }
    if ctx.controls.plrctrls.slot >= 0 {
        reset_inv_cursor_position(ctx);
        return;
    }
    if ctx.controls.plrctrls.active_stash_slot != INVALID_STASH_POINT {
        let mut mouse_pos = crate::qol::stash::get_stash_slot_coord(ctx, ctx.controls.plrctrls.active_stash_slot);
        // Stash coordinates are all the top left of the cell, so we need to shift the mouse to the center of the held item
        // or the center of the cell if we have a hand cursor (itemSize will be 1x1 here so we can use the same calculation)
        mouse_pos = mouse_pos + Displacement::new(item_size.width * INV_SLOT_HALF_SIZE_PX, item_size.height * INV_SLOT_HALF_SIZE_PX);
        set_cursor(ctx, mouse_pos);
        return;
    }
    focus_on_inventory(ctx);
}

/// Original: `HotSpellMove` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::HotSpellMove(AxisDirection dir) sha=354c54ca7615
fn hot_spell_move(ctx: &mut Ctx, dir: AxisDirection) {
    use crate::panels::spell_icons::SPLICONLENGTH;
    let now = ctx.platform.ticks();
    let dir = ctx.controls.plrctrls.hot_spell_repeater.get(now, dir);
    if dir.x == AxisDirectionX::None && dir.y == AxisDirectionY::None {
        return;
    }
    let spell_list_items = crate::panels::spell_list::get_spell_list_items(ctx);
    let m = mouse(ctx);
    let mut position = m;
    let mut shortest_distance = i32::MAX;
    for item in spell_list_items.iter() {
        let center = item.location + Displacement::new(SPLICONLENGTH / 2, -SPLICONLENGTH / 2);
        let distance = m.manhattan_distance(center);
        if distance < shortest_distance {
            position = center;
            shortest_distance = distance;
        }
    }
    let search = |position: &mut Point, dir: AxisDirection, search_forward: bool| {
        if dir.x == AxisDirectionX::None && dir.y == AxisDirectionY::None {
            return;
        }
        for i in 0..spell_list_items.len() {
            let index = if search_forward { spell_list_items.len() - i - 1 } else { i };
            let item = &spell_list_items[index];
            if item.is_selected {
                continue;
            }
            let center = item.location + Displacement::new(SPLICONLENGTH / 2, -SPLICONLENGTH / 2);
            if dir.x == AxisDirectionX::Left && center.x >= m.x {
                continue;
            }
            if dir.x == AxisDirectionX::Right && center.x <= m.x {
                continue;
            }
            if dir.x == AxisDirectionX::None && center.x != position.x {
                continue;
            }
            if dir.y == AxisDirectionY::Up && center.y >= m.y {
                continue;
            }
            if dir.y == AxisDirectionY::Down && center.y <= m.y {
                continue;
            }
            if dir.y == AxisDirectionY::None && center.y != position.y {
                continue;
            }
            *position = center;
            break;
        }
    };
    search(&mut position, AxisDirection { x: AxisDirectionX::None, y: dir.y }, dir.y == AxisDirectionY::Down);
    search(&mut position, AxisDirection { x: dir.x, y: AxisDirectionY::None }, dir.x == AxisDirectionX::Right);
    if position != m {
        set_cursor(ctx, position);
    }
}

/// Original: `SpellBookMove` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::SpellBookMove(AxisDirection dir) sha=2d9d786736b5
fn spell_book_move(ctx: &mut Ctx, dir: AxisDirection) {
    let now = ctx.platform.ticks();
    let dir = ctx.controls.plrctrls.spell_book_repeater.get(now, dir);
    if dir.x == AxisDirectionX::Left {
        if ctx.control.sbooktab > 0 {
            ctx.control.sbooktab -= 1;
        }
    } else if dir.x == AxisDirectionX::Right {
        let hf = ctx.init.gb_is_hellfire;
        if (hf && ctx.control.sbooktab < 4) || (!hf && ctx.control.sbooktab < 3) {
            ctx.control.sbooktab += 1;
        }
    }
}

/// Original: `devilution::ProcessGameAction` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::ProcessGameAction(const GameAction &action) sha=e6249bff6465
pub fn process_game_action(ctx: &mut Ctx, action: super::game_controls::GameActionType) {
    use super::game_controls::GameActionType as A;
    use crate::cursor::{new_cursor, CURSOR_DISARM, CURSOR_HAND};
    match action {
        A::None | A::SendKey => {}
        A::UseHealthPotion => use_belt_item(ctx, crate::inv::BLT_HEALING),
        A::UseManaPotion => use_belt_item(ctx, crate::inv::BLT_MANA),
        A::PrimaryAction => perform_primary_action(ctx),
        A::SecondaryAction => perform_secondary_action(ctx),
        A::CastSpell => perform_spell_action(ctx),
        A::ToggleQuickSpellMenu => {
            if !ctx.inv.invflag || blur_inventory(ctx) {
                if !ctx.control.spselflag {
                    crate::panels::spell_list::do_speed_book(ctx);
                } else {
                    ctx.control.spselflag = false;
                }
                crate::control::close_char_panel(ctx);
                ctx.quests.QuestLogIsOpen = false;
                ctx.control.sbookflag = false;
                crate::qol::stash::close_gold_withdraw(ctx);
                crate::inv::close_stash(ctx);
            }
        }
        A::ToggleCharacterInfo => {
            crate::control::toggle_char_panel(ctx);
            if ctx.control.chrflag {
                ctx.control.spselflag = false;
                if ctx.cursor.pcurs == CURSOR_DISARM {
                    new_cursor(ctx, CURSOR_HAND);
                }
                crate::control::focus_on_char_info(ctx);
            }
        }
        A::ToggleQuestLog => {
            if !ctx.quests.QuestLogIsOpen {
                crate::quests::start_questlog(ctx);
                crate::control::close_char_panel(ctx);
                crate::qol::stash::close_gold_withdraw(ctx);
                crate::inv::close_stash(ctx);
                ctx.control.spselflag = false;
            } else {
                ctx.quests.QuestLogIsOpen = false;
            }
        }
        A::ToggleInventory => {
            if ctx.inv.invflag {
                blur_inventory(ctx);
            } else {
                ctx.control.sbookflag = false;
                ctx.control.spselflag = false;
                ctx.inv.invflag = true;
                if ctx.cursor.pcurs == CURSOR_DISARM {
                    new_cursor(ctx, CURSOR_HAND);
                }
                focus_on_inventory(ctx);
            }
        }
        A::ToggleSpellBook => {
            if blur_inventory(ctx) {
                crate::inv::close_inventory(ctx);
                ctx.control.spselflag = false;
                ctx.control.sbookflag = !ctx.control.sbookflag;
            }
        }
    }
}

/// Original: `devilution::UseBeltItem` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::UseBeltItem(int type) sha=811b6becca7c
pub fn use_belt_item(ctx: &mut Ctx, type_: i32) {
    let me = my_player(ctx);
    for i in 0..crate::player::MaxBeltItems {
        let item = &ctx.players.Players[me].SpdList[i];
        if item.is_empty() {
            continue;
        }
        let is_rejuvenation = matches!(item._iMiscId, IMISC_REJUV | IMISC_FULLREJUV) || (item._iMiscId == IMISC_ARENAPOT && ctx.players.Players[me].is_on_arena_level());
        let is_healing = is_rejuvenation || matches!(item._iMiscId, IMISC_HEAL | IMISC_FULLHEAL) || item.is_scroll_of(SpellID::Healing);
        let is_mana = is_rejuvenation || matches!(item._iMiscId, IMISC_MANA | IMISC_FULLMANA);
        if (type_ == crate::inv::BLT_HEALING && is_healing) || (type_ == crate::inv::BLT_MANA && is_mana) {
            crate::inv::use_inv_item(ctx, INVITEM_BELT_FIRST as i32 + i as i32);
            break;
        }
    }
}

/// Original: `devilution::PerformPrimaryAction` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::PerformPrimaryAction() sha=ac60cfb377cb
pub fn perform_primary_action(ctx: &mut Ctx) {
    use crate::cursor::{new_cursor, CURSOR_FIRSTITEM, CURSOR_HAND};
    let me = my_player(ctx);
    if ctx.inv.invflag {
        // inventory is open
        let m = mouse(ctx);
        if ctx.cursor.pcurs > CURSOR_HAND && ctx.cursor.pcurs < CURSOR_FIRSTITEM {
            crate::diablo::try_icon_curs(ctx);
            new_cursor(ctx, CURSOR_HAND);
        } else if crate::control::get_right_panel(ctx).contains(m) || crate::control::get_main_panel(ctx).contains(m) {
            let slot = ctx.controls.plrctrls.slot;
            let inventory_slot = if slot >= 0 { slot } else { find_closest_inventory_slot(ctx, m) };
            // If the cursor is over an inventory slot we may need to adjust it due to pasting items of different sizes over each other
            let mut jump_slot = inventory_slot;
            let inv_first = SLOTXY_INV_FIRST as i32;
            let inv_last = SLOTXY_INV_LAST as i32;
            if inventory_slot >= inv_first && inventory_slot <= inv_last {
                let hold = ctx.players.Players[me].HoldItem.clone();
                let cursor_size_in_cells = if hold.is_empty() { Size::new(1, 1) } else { crate::inv::get_inventory_size(&hold) };
                // Find any item occupying a slot that is currently under the cursor
                let mut item_under_cursor = 0;
                'outer: for x in 0..cursor_size_in_cells.width {
                    for y in 0..cursor_size_in_cells.height {
                        let slot_under_cursor = inventory_slot + x + y * INV_ROW_SLOT_SIZE;
                        if slot_under_cursor > inv_last {
                            continue;
                        }
                        let item_id = get_item_id_on_slot(ctx, slot_under_cursor);
                        if item_id != 0 {
                            item_under_cursor = item_id;
                            break 'outer;
                        }
                    }
                }
                // Capture the first slot of the first item (if any) under the cursor
                if item_under_cursor as i8 > 0 {
                    jump_slot = find_first_slot_on_item(ctx, item_under_cursor as i8);
                }
            }
            crate::inv::check_inv_item(ctx, false, false);
            if inventory_slot >= inv_first && inventory_slot <= inv_last {
                let mut mouse_pos = get_slot_coord(ctx, jump_slot);
                ctx.controls.plrctrls.slot = jump_slot;
                let hold = ctx.players.Players[me].HoldItem.clone();
                let new_cursor_size_in_cells = if hold.is_empty() { get_item_size_on_slot(ctx, jump_slot) } else { crate::inv::get_inventory_size(&hold) };
                mouse_pos.x += ((new_cursor_size_in_cells.width - 1) * InventorySlotSizeInPixels.width) / 2;
                mouse_pos.y += ((new_cursor_size_in_cells.height - 1) * InventorySlotSizeInPixels.height) / 2;
                set_cursor(ctx, mouse_pos);
            }
        } else if ctx.stash.IsStashOpen && crate::control::get_left_panel(ctx).contains(m) {
            let active = ctx.controls.plrctrls.active_stash_slot;
            let stash_slot = if active != INVALID_STASH_POINT { active } else { find_closest_stash_slot(ctx, m) };
            let hold = ctx.players.Players[me].HoldItem.clone();
            let mut cursor_size_in_cells = if hold.is_empty() { Size::new(1, 1) } else { crate::inv::get_inventory_size(&hold) };
            // Find any item occupying a slot that is currently under the cursor
            let mut item_under_cursor = crate::qol::stash::StashStruct::EmptyCell;
            if stash_slot != INVALID_STASH_POINT {
                for slot_under_cursor in crate::engine::geometry::points_in_rectangle(Rectangle::new(stash_slot, cursor_size_in_cells)) {
                    if slot_under_cursor.x >= 10 || slot_under_cursor.y >= 10 {
                        continue;
                    }
                    let item_id = ctx.stash.Stash.get_item_id_at_position(slot_under_cursor);
                    if item_id != crate::qol::stash::StashStruct::EmptyCell {
                        item_under_cursor = item_id;
                        break;
                    }
                }
            }
            let jump_slot = if item_under_cursor == crate::qol::stash::StashStruct::EmptyCell { stash_slot } else { find_first_stash_slot_on_item(ctx, item_under_cursor) };
            crate::qol::stash::check_stash_item(ctx, m, false, false);
            let mut mouse_pos = crate::qol::stash::get_stash_slot_coord(ctx, jump_slot);
            ctx.controls.plrctrls.active_stash_slot = jump_slot;
            if ctx.players.Players[me].HoldItem.is_empty() {
                // For inventory cut/paste we can combine the cases where we swap or simply paste items. Because stash movement is always cell based (there's no fast
                // movement over large items) it looks better if we offset the hand cursor to the bottom right cell of the item we just placed.
                ctx.controls.plrctrls.active_stash_slot = jump_slot + Displacement::new(cursor_size_in_cells.width - 1, cursor_size_in_cells.height - 1);
                // Then we displace the mouse position to the bottom right corner of the item, then shift it back half a cell to center it.
                mouse_pos = mouse_pos
                    + Displacement::new(
                        cursor_size_in_cells.width * InventorySlotSizeInPixels.width - InventorySlotSizeInPixels.width / 2,
                        cursor_size_in_cells.height * InventorySlotSizeInPixels.height - InventorySlotSizeInPixels.height / 2,
                    );
            } else {
                // If we've picked up an item then use the same logic as the inventory so that the cursor is offset to the center of where the old item location was
                // (in this case jumpSlot was the top left cell of where it used to be in the grid, and we need to update the cursor size since we're now holding the item)
                cursor_size_in_cells = crate::inv::get_inventory_size(&ctx.players.Players[me].HoldItem);
                mouse_pos.x += (cursor_size_in_cells.width * InventorySlotSizeInPixels.width) / 2;
                mouse_pos.y += (cursor_size_in_cells.height * InventorySlotSizeInPixels.height) / 2;
            }
            set_cursor(ctx, mouse_pos);
        }
        return;
    }
    if ctx.control.spselflag {
        crate::panels::spell_list::set_spell(ctx);
        return;
    }
    if ctx.control.chrflag && !ctx.control.chrbtnactive && ctx.players.Players[me]._pStatPts > 0 {
        crate::control::check_chr_btns(ctx);
        if ctx.control.chrbtnactive {
            crate::control::release_chr_btns(ctx, false);
        }
        return;
    }
    interact(ctx);
}

/// Original: `SpellHasActorTarget` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::SpellHasActorTarget() sha=d987f9d9e233
fn spell_has_actor_target(ctx: &mut Ctx) -> bool {
    let spl = ctx.players.Players[my_player(ctx)]._pRSpell;
    if spl == SpellID::TownPortal || spl == SpellID::Teleport {
        return false;
    }
    if crate::spells::is_wall_spell(spl) && ctx.cursor.pcursmonst != -1 {
        ctx.cursor.cursPosition = ctx.monster.Monsters[ctx.cursor.pcursmonst as usize].position.tile;
    }
    ctx.cursor.pcursplr != -1 || ctx.cursor.pcursmonst != -1
}

/// Original: `devilution::UpdateSpellTarget` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::UpdateSpellTarget(SpellID spell) sha=f80b2bac08d1
pub fn update_spell_target(ctx: &mut Ctx, spell: SpellID) {
    if spell_has_actor_target(ctx) {
        return;
    }
    ctx.cursor.pcursplr = -1;
    ctx.cursor.pcursmonst = -1;
    let me = my_player(ctx);
    let range = if spell == SpellID::Teleport { 4 } else { 1 };
    let p = &ctx.players.Players[me];
    let d = Displacement::from(p._pdir);
    ctx.cursor.cursPosition = p.position.future + Displacement::new(d.delta_x * range, d.delta_y * range);
}

/// Original: `devilution::TryDropItem` (controls/plrctrls.cpp): try dropping item in all 9 possible places.
// @port controls/plrctrls.cpp|devilution::TryDropItem() sha=6dd90ba0fb22
pub fn try_drop_item(ctx: &mut Ctx) -> bool {
    use crate::cursor::{new_cursor, CURSOR_HAND};
    let me = my_player(ctx);
    if ctx.players.Players[me].HoldItem.is_empty() {
        return false;
    }
    if ctx.gendung.leveltype == crate::levels::gendung::DungeonType::Town {
        let (hold, tile) = (ctx.players.Players[me].HoldItem.clone(), ctx.players.Players[me].position.tile);
        if crate::items::use_item_opens_hive(ctx, &hold, tile) {
            crate::levels::town::open_hive(ctx);
            new_cursor(ctx, CURSOR_HAND);
            return true;
        }
        if crate::items::use_item_opens_grave(ctx, &hold, tile) {
            crate::levels::town::open_grave(ctx);
            new_cursor(ctx, CURSOR_HAND);
            return true;
        }
    }
    let (future, dir) = (ctx.players.Players[me].position.future, ctx.players.Players[me]._pdir);
    let Some(item_tile) = crate::inv::find_adjacent_position_for_item(ctx, future, dir) else {
        crate::player::player_say(ctx, me, HeroSpeech::WhereWouldIPutThis);
        return false;
    };
    let hold = ctx.players.Players[me].HoldItem.clone();
    crate::msg::net_send_cmd_p_item(ctx, true, CMD_PUTITEM, item_tile, &hold);
    ctx.players.Players[me].HoldItem.clear();
    new_cursor(ctx, CURSOR_HAND);
    true
}

/// Original: `devilution::PerformSpellAction` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::PerformSpellAction() sha=b8cc34933afb
pub fn perform_spell_action(ctx: &mut Ctx) {
    use crate::cursor::{new_cursor, CURSOR_HAND};
    use crate::diablo::MouseActionType;
    if in_game_menu(ctx) || ctx.quests.QuestLogIsOpen || ctx.control.sbookflag {
        return;
    }
    let me = my_player(ctx);
    if ctx.inv.invflag {
        if !ctx.players.Players[me].HoldItem.is_empty() {
            try_drop_item(ctx);
        } else if ctx.cursor.pcurs > CURSOR_HAND {
            crate::diablo::try_icon_curs(ctx);
            new_cursor(ctx, CURSOR_HAND);
        } else if ctx.cursor.pcursinvitem != -1 {
            let slot = ctx.controls.plrctrls.slot;
            let item_id = get_item_id_on_slot(ctx, slot);
            crate::inv::check_inv_item(ctx, true, false);
            if item_id != get_item_id_on_slot(ctx, slot) {
                reset_inv_cursor_position(ctx);
            }
        } else if ctx.cursor.pcursstashitem != crate::qol::stash::StashStruct::EmptyCell {
            let m = mouse(ctx);
            crate::qol::stash::check_stash_item(ctx, m, true, false);
        }
        return;
    }
    if !ctx.players.Players[me].HoldItem.is_empty() && !try_drop_item(ctx) {
        return;
    }
    if ctx.cursor.pcurs > CURSOR_HAND {
        new_cursor(ctx, CURSOR_HAND);
    }
    if ctx.control.spselflag {
        crate::panels::spell_list::set_spell(ctx);
        return;
    }
    let spl = ctx.players.Players[me]._pRSpell;
    if (ctx.cursor.pcursplr == -1 && (spl == SpellID::Resurrect || spl == SpellID::HealOther)) || (ctx.cursor.ObjectUnderCursor.is_none() && spl == SpellID::TrapDisarm) {
        crate::player::player_say(ctx, me, HeroSpeech::ICantCastThatHere);
        return;
    }
    update_spell_target(ctx, spl);
    let (s, t) = (ctx.players.Players[me]._pRSpell, ctx.players.Players[me]._pRSplType);
    crate::player::check_plr_spell(ctx, false, s, t);
    ctx.diablo.last_mouse_button_action = if ctx.cursor.pcursplr != -1 {
        MouseActionType::SpellPlayerTarget
    } else if ctx.cursor.pcursmonst != -1 {
        MouseActionType::SpellMonsterTarget
    } else {
        MouseActionType::Spell
    };
}

/// Original: `CtrlUseInvItem` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::CtrlUseInvItem() sha=af0748ab7017
fn ctrl_use_inv_item(ctx: &mut Ctx) {
    if ctx.cursor.pcursinvitem == -1 {
        return;
    }
    let me = my_player(ctx);
    let ci = ctx.cursor.pcursinvitem as i32;
    let item = crate::inv::get_inventory_item(ctx, me, ci).clone();
    if item.is_scroll() {
        if crate::spells::targets_monster(item._iSpell) {
            return;
        }
        if crate::items::get_spell_data(item._iSpell).is_targeted() {
            update_spell_target(ctx, item._iSpell);
        }
    }
    let slot = ctx.controls.plrctrls.slot;
    let item_id = get_item_id_on_slot(ctx, slot);
    if item.is_equipment() {
        crate::inv::check_inv_item(ctx, true, false); // auto-equip if it's an equipment
    } else {
        crate::inv::use_inv_item(ctx, ci);
    }
    if item_id != get_item_id_on_slot(ctx, slot) {
        reset_inv_cursor_position(ctx);
    }
}

/// Original: `CtrlUseStashItem` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::CtrlUseStashItem() sha=3f29640a573c
fn ctrl_use_stash_item(ctx: &mut Ctx) {
    let si = ctx.cursor.pcursstashitem;
    if si == crate::qol::stash::StashStruct::EmptyCell {
        return;
    }
    let item = ctx.stash.Stash.stashList[si as usize].clone();
    if item.is_scroll() {
        if crate::spells::targets_monster(item._iSpell) {
            return;
        }
        if crate::items::get_spell_data(item._iSpell).is_targeted() {
            update_spell_target(ctx, item._iSpell);
        }
    }
    if item.is_equipment() {
        let m = mouse(ctx);
        crate::qol::stash::check_stash_item(ctx, m, true, false); // Auto-equip if it's equipment
    } else {
        crate::qol::stash::use_stash_item(ctx, si);
    }
    // Todo reset cursor position if item is moved
}

/// Original: `devilution::PerformSecondaryAction` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::PerformSecondaryAction() sha=8d22389ee955
pub fn perform_secondary_action(ctx: &mut Ctx) {
    use crate::cursor::{new_cursor, CURSOR_FIRSTITEM, CURSOR_HAND};
    use crate::diablo::MouseActionType;
    let me = my_player(ctx);
    if ctx.inv.invflag {
        if ctx.cursor.pcurs > CURSOR_HAND && ctx.cursor.pcurs < CURSOR_FIRSTITEM {
            crate::diablo::try_icon_curs(ctx);
            new_cursor(ctx, CURSOR_HAND);
        } else if ctx.stash.IsStashOpen {
            if ctx.cursor.pcursstashitem != crate::qol::stash::StashStruct::EmptyCell {
                let si = ctx.cursor.pcursstashitem;
                crate::qol::stash::transfer_item_to_inventory(ctx, me, si);
            } else if ctx.cursor.pcursinvitem != -1 {
                let ii = ctx.cursor.pcursinvitem as i32;
                crate::inv::transfer_item_to_stash(ctx, me, ii);
            }
        } else {
            ctrl_use_inv_item(ctx);
        }
        return;
    }
    if !ctx.players.Players[me].HoldItem.is_empty() && !try_drop_item(ctx) {
        return;
    }
    if ctx.cursor.pcurs > CURSOR_HAND {
        new_cursor(ctx, CURSOR_HAND);
    }
    let curs = ctx.cursor.cursPosition;
    if ctx.cursor.pcursitem != -1 {
        let ii = ctx.cursor.pcursitem as u16;
        crate::msg::net_send_cmd_loc_param1(ctx, true, CMD_GOTOAGETITEM, curs, ii);
    } else if ctx.cursor.ObjectUnderCursor.is_some() {
        let id = ctx.players.MyPlayerId;
        crate::msg::net_send_cmd_loc(ctx, id, true, CMD_OPOBJXY, curs);
        ctx.diablo.last_mouse_button_action = MouseActionType::OperateObject;
    } else if let Some(mi) = ctx.controls.plrctrls.pcursmissile {
        let tile = ctx.missiles.Missiles[mi].position.tile;
        crate::player::make_plr_path(ctx, me, tile, true);
        ctx.players.Players[me].destAction = ACTION_WALK;
    } else if ctx.controls.plrctrls.pcurstrig != -1 {
        let pos = ctx.trigs.trigs[ctx.controls.plrctrls.pcurstrig as usize].position;
        crate::player::make_plr_path(ctx, me, pos, true);
        ctx.players.Players[me].destAction = ACTION_WALK;
    } else if ctx.controls.plrctrls.pcursquest != Q_INVALID {
        let pos = ctx.quests.Quests[ctx.controls.plrctrls.pcursquest as usize].position;
        crate::player::make_plr_path(ctx, me, pos, true);
        ctx.players.Players[me].destAction = ACTION_WALK;
    }
}

/// Original: `devilution::QuickCast` (controls/plrctrls.cpp).
// @port controls/plrctrls.cpp|devilution::QuickCast(size_t slot) sha=a4c2c44da29f
pub fn quick_cast(ctx: &mut Ctx, slot: usize) {
    let prev_mouse_button_action = ctx.diablo.last_mouse_button_action;
    let me = my_player(ctx);
    let spell = ctx.players.Players[me]._pSplHotKey[slot];
    let spell_type = ctx.players.Players[me]._pSplTHotKey[slot];
    if ctx.controls.control_mode != ControlTypes::KeyboardAndMouse {
        update_spell_target(ctx, spell);
    }
    crate::player::check_plr_spell(ctx, false, spell, spell_type);
    ctx.diablo.last_mouse_button_action = prev_mouse_button_action;
}
