//! `Source/controls/controller.cpp`, `controls/axis_direction.cpp`,
//! `controls/controller_motion.cpp`, `controls/menu_controls.cpp`: the device-independent
//! controller layer and menu navigation.
//!
//! Gamepads and joysticks (`controls/devices/*`) have no backend in the port yet: the device
//! lists are empty, so `GameController::Get` / `Joystick::Get` never find a device and no
//! controller events exist (known difference: keyboard and mouse only).

use crate::controls::controller_buttons::{ControllerButton, ControllerButtonCombo};
use crate::controls::ControlTypes;
use crate::ctx::Ctx;
use crate::platform::events::{keys::*, Event, BUTTON_X1, KMOD_ALT, KMOD_SHIFT};
use crate::platform::log;

/// `ControllerButtonEvent`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControllerButtonEvent {
    pub button: ControllerButton,
    pub up: bool,
}

/// `AxisDirectionX`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AxisDirectionX {
    #[default]
    None,
    Left,
    Right,
}

/// `AxisDirectionY`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AxisDirectionY {
    #[default]
    None,
    Up,
    Down,
}

/// `AxisDirection`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct AxisDirection {
    pub x: AxisDirectionX,
    pub y: AxisDirectionY,
}

/// `AxisDirectionRepeater`
#[derive(Clone, Copy, Debug)]
pub struct AxisDirectionRepeater {
    last_left: i32,
    last_right: i32,
    last_up: i32,
    last_down: i32,
    min_interval_ms: i32,
}

impl Default for AxisDirectionRepeater {
    fn default() -> Self {
        AxisDirectionRepeater { last_left: 0, last_right: 0, last_up: 0, last_down: 0, min_interval_ms: 200 }
    }
}

impl AxisDirectionRepeater {
    /// `AxisDirectionRepeater(int min_interval_ms)`
    pub fn new(min_interval_ms: i32) -> AxisDirectionRepeater {
        AxisDirectionRepeater { min_interval_ms, ..Default::default() }
    }

    /// Original: `AxisDirectionRepeater::Get` (controls/axis_direction.cpp).
    // @port controls/axis_direction.cpp|devilution::AxisDirectionRepeater::Get(AxisDirection axisDirection) sha=98ba142a106c
    pub fn get(&mut self, now: u32, mut d: AxisDirection) -> AxisDirection {
        let now = now as i32;
        match d.x {
            AxisDirectionX::Left => {
                self.last_right = 0;
                if now.wrapping_sub(self.last_left) < self.min_interval_ms {
                    d.x = AxisDirectionX::None;
                } else {
                    self.last_left = now;
                }
            }
            AxisDirectionX::Right => {
                self.last_left = 0;
                if now.wrapping_sub(self.last_right) < self.min_interval_ms {
                    d.x = AxisDirectionX::None;
                } else {
                    self.last_right = now;
                }
            }
            AxisDirectionX::None => {
                self.last_left = 0;
                self.last_right = 0;
            }
        }
        match d.y {
            AxisDirectionY::Up => {
                self.last_down = 0;
                if now.wrapping_sub(self.last_up) < self.min_interval_ms {
                    d.y = AxisDirectionY::None;
                } else {
                    self.last_up = now;
                }
            }
            AxisDirectionY::Down => {
                self.last_up = 0;
                if now.wrapping_sub(self.last_down) < self.min_interval_ms {
                    d.y = AxisDirectionY::None;
                } else {
                    self.last_down = now;
                }
            }
            AxisDirectionY::None => {
                self.last_up = 0;
                self.last_down = 0;
            }
        }
        d
    }
}

/// Stick state (`leftStickX` ... in controls/controller_motion.cpp).
#[derive(Default, Clone, Copy, Debug)]
pub struct StickState {
    pub left_stick_x: f32,
    pub left_stick_y: f32,
    pub right_stick_x: f32,
    pub right_stick_y: f32,
    pub right_stick_last_move: f32,
    pub simulating_mouse_with_padmapper: bool,
}

/// Original: `devilution::UnlockControllerState` (controls/controller.cpp): no devices.
// @port controls/controller.cpp|devilution::UnlockControllerState(const SDL_Event &event) sha=06ac39e525dd
pub fn unlock_controller_state(_ctx: &mut Ctx, _event: &Event) {}

/// Original: `devilution::ToControllerButtonEvents` (controls/controller.cpp). Keyboard
/// controller mapping is not compiled in (HAS_KBCTRL == 0) and no device matches the event,
/// so the result is a single NONE event (up for key/button releases).
// @port controls/controller.cpp|devilution::ToControllerButtonEvents(const SDL_Event &event) sha=0c39e558c674
pub fn to_controller_button_events(_ctx: &Ctx, event: &Event) -> Vec<ControllerButtonEvent> {
    let up = matches!(event, Event::KeyUp { .. });
    vec![ControllerButtonEvent { button: ControllerButton::None, up }]
}

/// Original: `devilution::IsControllerButtonPressed` (controls/controller.cpp).
// @port controls/controller.cpp|devilution::IsControllerButtonPressed(ControllerButton button) sha=2f118fd71f38
pub fn is_controller_button_pressed(_ctx: &Ctx, _button: ControllerButton) -> bool {
    // GameController::IsPressedOnAnyController / Joystick::IsPressedOnAnyJoystick: no devices.
    false
}

/// Original: `devilution::IsControllerButtonComboPressed` (controls/controller.cpp).
// @port controls/controller.cpp|devilution::IsControllerButtonComboPressed(ControllerButtonCombo combo) sha=5b4d5a9d02d1
pub fn is_controller_button_combo_pressed(ctx: &Ctx, combo: ControllerButtonCombo) -> bool {
    is_controller_button_pressed(ctx, combo.button) && (combo.modifier == ControllerButton::None || is_controller_button_pressed(ctx, combo.modifier))
}

/// Original: `devilution::HandleControllerAddedOrRemovedEvent` (controls/controller.cpp).
// @port controls/controller.cpp|devilution::HandleControllerAddedOrRemovedEvent(const SDL_Event &event) sha=17e0e7579e31
pub fn handle_controller_added_or_removed_event(_ctx: &mut Ctx, _event: &Event) -> bool {
    false
}

/// Original: `devilution::IsControllerMotion` (controls/controller_motion.cpp).
// @port controls/controller_motion.cpp|devilution::IsControllerMotion(const SDL_Event &event) sha=a82525bbf19f
pub fn is_controller_motion(_event: &Event) -> bool {
    false
}

/// Original: `devilution::ProcessControllerMotion` (controls/controller_motion.cpp).
// @port controls/controller_motion.cpp|devilution::ProcessControllerMotion(const SDL_Event &event) sha=6091b360dcfd
pub fn process_controller_motion(_ctx: &mut Ctx, _event: &Event) {
    // No controller or joystick matches the event, so there is no axis motion to process.
}

/// Original: `IsMovementOverriddenByPadmapper` (controls/controller_motion.cpp).
// @port controls/controller_motion.cpp|devilution::IsMovementOverriddenByPadmapper(ControllerButton button) sha=10385dabda3f
fn is_movement_overridden_by_padmapper(ctx: &Ctx, button: ControllerButton) -> bool {
    let action = crate::options::padmapper_action_name_triggered_by_button_event(ctx, button, true);
    ctx.options.padmapper.button_combo_for_action(&action).modifier != ControllerButton::None
}

/// Original: `TriggersQuickSpellAction` (controls/controller_motion.cpp).
// @port controls/controller_motion.cpp|devilution::TriggersQuickSpellAction(ControllerButton button) sha=c47198fa245f
fn triggers_quick_spell_action(ctx: &Ctx, button: ControllerButton) -> bool {
    crate::options::padmapper_action_name_triggered_by_button_event(ctx, button, true).starts_with("QuickSpell")
}

/// Original: `IsPressedForMovement` (controls/controller_motion.cpp).
// @port controls/controller_motion.cpp|devilution::IsPressedForMovement(ControllerButton button) sha=b700ee06ef70
fn is_pressed_for_movement(ctx: &Ctx, button: ControllerButton) -> bool {
    !ctx.controls.pad_menu_navigator_active
        && is_controller_button_pressed(ctx, button)
        && !is_movement_overridden_by_padmapper(ctx, button)
        && !(ctx.control.spselflag && triggers_quick_spell_action(ctx, button))
}

/// Original: `devilution::GetLeftStickOrDpadDirection` (controls/controller_motion.cpp).
// @port controls/controller_motion.cpp|devilution::GetLeftStickOrDpadDirection(bool usePadmapper) sha=738364db4640
pub fn get_left_stick_or_dpad_direction(ctx: &Ctx, use_padmapper: bool) -> AxisDirection {
    let s = ctx.controls.sticks;
    let mut up = s.left_stick_y >= 0.5;
    let mut down = s.left_stick_y <= -0.5;
    let mut left = s.left_stick_x <= -0.5;
    let mut right = s.left_stick_x >= 0.5;
    if use_padmapper {
        up |= ctx.options.padmapper.is_active("MoveUp");
        down |= ctx.options.padmapper.is_active("MoveDown");
        left |= ctx.options.padmapper.is_active("MoveLeft");
        right |= ctx.options.padmapper.is_active("MoveRight");
    } else if !s.simulating_mouse_with_padmapper {
        up |= is_pressed_for_movement(ctx, ControllerButton::ButtonDpadUp);
        down |= is_pressed_for_movement(ctx, ControllerButton::ButtonDpadDown);
        left |= is_pressed_for_movement(ctx, ControllerButton::ButtonDpadLeft);
        right |= is_pressed_for_movement(ctx, ControllerButton::ButtonDpadRight);
    }
    // ControlMode == VirtualGamepad: no touch devices on the targets.
    let mut result = AxisDirection::default();
    if up {
        result.y = AxisDirectionY::Up;
    } else if down {
        result.y = AxisDirectionY::Down;
    }
    if left {
        result.x = AxisDirectionX::Left;
    } else if right {
        result.x = AxisDirectionX::Right;
    }
    result
}

/// `MenuAction`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuAction {
    None,
    Select,
    Back,
    Delete,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
}

/// Original: `devilution::GetMenuHeldUpDownAction` (controls/menu_controls.cpp).
// @port controls/menu_controls.cpp|devilution::GetMenuHeldUpDownAction() sha=9cf1e3c8d30f
pub fn get_menu_held_up_down_action(ctx: &mut Ctx) -> MenuAction {
    let d = get_left_stick_or_dpad_direction(ctx, false);
    let now = ctx.platform.ticks();
    let dir = ctx.controls.menu_held_repeater.get(now, d);
    match dir.y {
        AxisDirectionY::Up => MenuAction::Up,
        AxisDirectionY::Down => MenuAction::Down,
        AxisDirectionY::None => MenuAction::None,
    }
}

/// Original: `devilution::GetMenuActions` (controls/menu_controls.cpp).
// @port controls/menu_controls.cpp|devilution::GetMenuActions(const SDL_Event &event) sha=851439d3f628
pub fn get_menu_actions(ctx: &mut Ctx, event: &Event) -> Vec<MenuAction> {
    let mut menu_actions = Vec::new();
    for ctrl_event in to_controller_button_events(ctx, event) {
        if ctrl_event.button == ControllerButton::Ignore {
            continue;
        }
        let is_gamepad_motion = is_controller_motion(event);
        crate::controls::plrctrls::detect_input_method(ctx, event, ctrl_event);
        if is_gamepad_motion {
            menu_actions.push(get_menu_held_up_down_action(ctx));
            continue;
        }
        if !ctrl_event.up {
            use ControllerButton as B;
            match crate::controls::game_controls::translate_to(ctx, ctrl_event.button) {
                B::ButtonA | B::ButtonStart => menu_actions.push(MenuAction::Select),
                B::ButtonBack | B::ButtonB => menu_actions.push(MenuAction::Back),
                B::ButtonX => menu_actions.push(MenuAction::Delete),
                B::ButtonDpadUp | B::ButtonDpadDown => menu_actions.push(get_menu_held_up_down_action(ctx)),
                B::ButtonDpadLeft => menu_actions.push(MenuAction::Left),
                B::ButtonDpadRight => menu_actions.push(MenuAction::Right),
                B::ButtonLeftShoulder => menu_actions.push(MenuAction::PageUp),
                B::ButtonRightShoulder => menu_actions.push(MenuAction::PageDown),
                _ => {}
            }
        }
    }
    if !menu_actions.is_empty() {
        return menu_actions;
    }
    if let Event::MouseButtonDown { button, .. } = event {
        if *button == BUTTON_X1 {
            return vec![MenuAction::Back];
        }
    }
    if let Event::KeyDown { key, .. } = event {
        let sym = crate::controls::remap_keyboard_key(*key);
        let mods = ctx.platform.mod_state();
        return match sym {
            SDLK_UP => vec![MenuAction::Up],
            SDLK_DOWN => vec![MenuAction::Down],
            SDLK_TAB => {
                if mods & KMOD_SHIFT != 0 {
                    vec![MenuAction::Up]
                } else {
                    vec![MenuAction::Down]
                }
            }
            SDLK_PAGEUP => vec![MenuAction::PageUp],
            SDLK_PAGEDOWN => vec![MenuAction::PageDown],
            SDLK_RETURN if mods & KMOD_ALT == 0 => vec![MenuAction::Select],
            SDLK_KP_ENTER => vec![MenuAction::Select],
            SDLK_SPACE if !ctx.diablo_ui.text_input_active => vec![MenuAction::Select],
            SDLK_DELETE => vec![MenuAction::Delete],
            SDLK_LEFT => vec![MenuAction::Left],
            SDLK_RIGHT => vec![MenuAction::Right],
            SDLK_ESCAPE => vec![MenuAction::Back],
            _ => vec![],
        };
    }
    vec![]
}

/// `ControlTypeToString` (controls/plrctrls.cpp)
// @port controls/plrctrls.cpp|devilution::ControlTypeToString(ControlTypes controlType) sha=05c5e8114be5
pub fn control_type_to_string(t: ControlTypes) -> &'static str {
    match t {
        ControlTypes::None => "None",
        ControlTypes::KeyboardAndMouse => "KeyboardAndMouse",
        ControlTypes::Gamepad => "Gamepad",
        ControlTypes::VirtualGamepad => "VirtualGamepad",
    }
}

/// `LogControlDeviceAndModeChange` (controls/plrctrls.cpp)
// @port controls/plrctrls.cpp|devilution::LogControlDeviceAndModeChange(ControlTypes newControlDevice, ControlTypes newControlMode) sha=00b648894377
pub fn log_control_device_and_mode_change(ctx: &Ctx, new_device: ControlTypes, new_mode: ControlTypes) {
    if new_device == ctx.controls.control_device && new_mode == ctx.controls.control_mode {
        return;
    }
    let change = |before: ControlTypes, after: ControlTypes| {
        if before == after {
            control_type_to_string(before).to_string()
        } else {
            format!("{} -> {}", control_type_to_string(before), control_type_to_string(after))
        }
    };
    log::verbose!(
        "Control: device {}, mode {}",
        change(ctx.controls.control_device, new_device),
        change(ctx.controls.control_mode, new_mode)
    );
}
