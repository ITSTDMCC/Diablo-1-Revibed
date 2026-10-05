//! `Source/controls/devices/game_controller.cpp` and `controls/devices/joystick.cpp`: the
//! connected game controllers and joysticks.
//!
//! The joystick layer is built as for Windows: none of the `JOY_*` button, hat or axis mappings
//! are defined, so joystick buttons and hats turn into `ControllerButton::Ignore` and no joystick
//! axis moves a stick. Game controllers (every pad SDL or gilrs has a mapping for) carry the input.

use crate::controls::controller::ControllerButtonEvent;
use crate::controls::controller_buttons::ControllerButton;
use crate::controls::game_controls::GamepadLayout;
use crate::ctx::Ctx;
use crate::platform::events::{pad, Event};
use crate::platform::log;
use crate::platform::GameControllerType;

/// `GameController`
#[derive(Clone, Debug)]
pub struct GameController {
    instance_id: i32,
    trigger_left_state: ControllerButton,
    trigger_right_state: ControllerButton,
    trigger_left_is_down: bool,
    trigger_right_is_down: bool,
}

/// Globals of controls/devices/*.cpp.
#[derive(Default)]
pub struct DevicesState {
    /// `GameController::controllers_`
    pub controllers: Vec<GameController>,
    /// `Joystick::joysticks_`
    pub joysticks: Vec<Joystick>,
}

impl GameController {
    /// Original: `GameController::UnlockTriggerState` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::UnlockTriggerState() sha=5dd135289188
    pub fn unlock_trigger_state(&mut self) {
        self.trigger_left_state = ControllerButton::None;
        self.trigger_right_state = ControllerButton::None;
    }

    /// Original: `GameController::ToControllerButton` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::ToControllerButton(const SDL_Event &event) sha=2af295e8bb5f
    pub fn to_controller_button(&mut self, event: &Event) -> ControllerButton {
        match *event {
            Event::ControllerAxisMotion { axis, value, .. } => match axis {
                pad::AXIS_TRIGGERLEFT => {
                    if value < 8192 && self.trigger_left_is_down {
                        // 25% pressed
                        self.trigger_left_is_down = false;
                        self.trigger_left_state = ControllerButton::AxisTriggerLeft;
                    }
                    if value > 16384 && !self.trigger_left_is_down {
                        // 50% pressed
                        self.trigger_left_is_down = true;
                        self.trigger_left_state = ControllerButton::AxisTriggerLeft;
                    }
                    self.trigger_left_state
                }
                pad::AXIS_TRIGGERRIGHT => {
                    if value < 8192 && self.trigger_right_is_down {
                        self.trigger_right_is_down = false;
                        self.trigger_right_state = ControllerButton::AxisTriggerRight;
                    }
                    if value > 16384 && !self.trigger_right_is_down {
                        self.trigger_right_is_down = true;
                        self.trigger_right_state = ControllerButton::AxisTriggerRight;
                    }
                    self.trigger_right_state
                }
                _ => ControllerButton::None,
            },
            Event::ControllerButtonDown { button, .. } | Event::ControllerButtonUp { button, .. } => match button {
                pad::BUTTON_A => ControllerButton::ButtonA,
                pad::BUTTON_B => ControllerButton::ButtonB,
                pad::BUTTON_X => ControllerButton::ButtonX,
                pad::BUTTON_Y => ControllerButton::ButtonY,
                pad::BUTTON_LEFTSTICK => ControllerButton::ButtonLeftStick,
                pad::BUTTON_RIGHTSTICK => ControllerButton::ButtonRightStick,
                pad::BUTTON_LEFTSHOULDER => ControllerButton::ButtonLeftShoulder,
                pad::BUTTON_RIGHTSHOULDER => ControllerButton::ButtonRightShoulder,
                pad::BUTTON_START => ControllerButton::ButtonStart,
                pad::BUTTON_BACK => ControllerButton::ButtonBack,
                pad::BUTTON_DPAD_UP => ControllerButton::ButtonDpadUp,
                pad::BUTTON_DPAD_DOWN => ControllerButton::ButtonDpadDown,
                pad::BUTTON_DPAD_LEFT => ControllerButton::ButtonDpadLeft,
                pad::BUTTON_DPAD_RIGHT => ControllerButton::ButtonDpadRight,
                _ => ControllerButton::None,
            },
            _ => ControllerButton::None,
        }
    }

    /// Original: `GameController::ToSdlGameControllerButton` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::ToSdlGameControllerButton(ControllerButton button) sha=cc6ff108a037
    pub fn to_sdl_game_controller_button(button: ControllerButton) -> u8 {
        if matches!(button, ControllerButton::AxisTriggerLeft | ControllerButton::AxisTriggerRight) {
            log::error!("UNIMPLEMENTED: GameController::ToSdlGameControllerButton");
        }
        match button {
            ControllerButton::ButtonA => pad::BUTTON_A,
            ControllerButton::ButtonB => pad::BUTTON_B,
            ControllerButton::ButtonX => pad::BUTTON_X,
            ControllerButton::ButtonY => pad::BUTTON_Y,
            ControllerButton::ButtonBack => pad::BUTTON_BACK,
            ControllerButton::ButtonStart => pad::BUTTON_START,
            ControllerButton::ButtonLeftStick => pad::BUTTON_LEFTSTICK,
            ControllerButton::ButtonRightStick => pad::BUTTON_RIGHTSTICK,
            ControllerButton::ButtonLeftShoulder => pad::BUTTON_LEFTSHOULDER,
            ControllerButton::ButtonRightShoulder => pad::BUTTON_RIGHTSHOULDER,
            ControllerButton::ButtonDpadUp => pad::BUTTON_DPAD_UP,
            ControllerButton::ButtonDpadDown => pad::BUTTON_DPAD_DOWN,
            ControllerButton::ButtonDpadLeft => pad::BUTTON_DPAD_LEFT,
            ControllerButton::ButtonDpadRight => pad::BUTTON_DPAD_RIGHT,
            _ => pad::BUTTON_INVALID,
        }
    }

    /// Original: `GameController::IsPressed` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::IsPressed(ControllerButton button) sha=f0bc3d3f41af
    pub fn is_pressed(&self, ctx: &Ctx, button: ControllerButton) -> bool {
        if button == ControllerButton::AxisTriggerLeft {
            return self.trigger_left_is_down;
        }
        if button == ControllerButton::AxisTriggerRight {
            return self.trigger_right_is_down;
        }
        let gc_button = Self::to_sdl_game_controller_button(button);
        // SDL_GameControllerHasButton: every mapped pad reports the standard buttons.
        gc_button != pad::BUTTON_INVALID && ctx.platform.game_controller_get_button(self.instance_id, gc_button)
    }

    /// Original: `GameController::ProcessAxisMotion` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::ProcessAxisMotion(const SDL_Event &event) sha=e46638849923
    pub fn process_axis_motion(ctx: &mut Ctx, event: &Event) -> bool {
        let Event::ControllerAxisMotion { axis, value, .. } = *event else { return false };
        let s = &mut ctx.controls.sticks;
        match axis {
            pad::AXIS_LEFTX => {
                s.left_stick_x_unscaled = value as f32;
                s.left_stick_needs_scaling = true;
            }
            pad::AXIS_LEFTY => {
                s.left_stick_y_unscaled = -(value as i32) as f32;
                s.left_stick_needs_scaling = true;
            }
            pad::AXIS_RIGHTX => {
                s.right_stick_x_unscaled = value as f32;
                s.right_stick_needs_scaling = true;
            }
            pad::AXIS_RIGHTY => {
                s.right_stick_y_unscaled = -(value as i32) as f32;
                s.right_stick_needs_scaling = true;
            }
            _ => return false,
        }
        true
    }

    /// Original: `GameController::Add` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::Add(int joystickIndex) sha=47c8fc5bddf4
    pub fn add(ctx: &mut Ctx, joystick_index: i32) {
        log::info!("Opening game controller for joystick at index {}", joystick_index);
        let Some(instance_id) = ctx.platform.game_controller_open(joystick_index) else {
            log::info!("Invalid game controller device index {}", joystick_index);
            return;
        };
        ctx.controls.devices.controllers.push(GameController {
            instance_id,
            trigger_left_state: ControllerButton::None,
            trigger_right_state: ControllerButton::None,
            trigger_left_is_down: false,
            trigger_right_is_down: false,
        });
    }

    /// Original: `GameController::Remove` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::Remove(SDL_JoystickID instanceId) sha=f40506aa0910
    pub fn remove(ctx: &mut Ctx, instance_id: i32) {
        log::info!("Removing game controller with instance id {}", instance_id);
        let controllers = &mut ctx.controls.devices.controllers;
        if let Some(i) = controllers.iter().position(|c| c.instance_id == instance_id) {
            controllers.remove(i);
            return;
        }
        log::info!("Game controller not found with instance id: {}", instance_id);
    }

    /// Original: `GameController::Get(SDL_JoystickID)` (controls/devices/game_controller.cpp):
    /// the index into the controller list.
    // @port controls/devices/game_controller.cpp|devilution::GameController::Get(SDL_JoystickID instanceId) sha=64451469c6fc
    pub fn get(ctx: &Ctx, instance_id: i32) -> Option<usize> {
        ctx.controls.devices.controllers.iter().position(|c| c.instance_id == instance_id)
    }

    /// Original: `GameController::Get(const SDL_Event &)` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::Get(const SDL_Event &event) sha=a98f11f3a4ac
    pub fn get_for_event(ctx: &Ctx, event: &Event) -> Option<usize> {
        match *event {
            Event::ControllerAxisMotion { which, .. } | Event::ControllerButtonDown { which, .. } | Event::ControllerButtonUp { which, .. } => {
                Self::get(ctx, which)
            }
            _ => None,
        }
    }

    /// Original: `GameController::All` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::All() sha=7137a079fd26
    pub fn all(ctx: &Ctx) -> &[GameController] {
        &ctx.controls.devices.controllers
    }

    /// Original: `GameController::IsPressedOnAnyController` (controls/devices/game_controller.cpp).
    // @port controls/devices/game_controller.cpp|devilution::GameController::IsPressedOnAnyController(ControllerButton button, SDL_JoystickID *which) sha=e1f9b4b46d9f
    pub fn is_pressed_on_any_controller(ctx: &Ctx, button: ControllerButton) -> Option<i32> {
        ctx.controls.devices.controllers.iter().find(|c| c.is_pressed(ctx, button)).map(|c| c.instance_id)
    }

    /// Original: `GameController::getLayout` (controls/devices/game_controller.cpp). Like the
    /// original, the event's `which` is read as a device index whatever the event type.
    // @port controls/devices/game_controller.cpp|devilution::GameController::getLayout(const SDL_Event &event) sha=1cad27f626e8
    pub fn get_layout(ctx: &Ctx, event: &Event) -> GamepadLayout {
        let index = match *event {
            Event::ControllerDeviceAdded { which }
            | Event::ControllerDeviceRemoved { which }
            | Event::ControllerButtonDown { which, .. }
            | Event::ControllerButtonUp { which, .. }
            | Event::ControllerAxisMotion { which, .. }
            | Event::JoyDeviceAdded { which }
            | Event::JoyDeviceRemoved { which }
            | Event::JoyButtonDown { which, .. }
            | Event::JoyButtonUp { which, .. }
            | Event::JoyAxisMotion { which, .. }
            | Event::JoyHatMotion { which, .. } => which,
            _ => 0,
        };
        match ctx.platform.game_controller_type_for_index(index) {
            GameControllerType::NintendoSwitchPro => GamepadLayout::Nintendo,
            GameControllerType::PlayStation => GamepadLayout::PlayStation,
            GameControllerType::Xbox => GamepadLayout::Xbox,
            GameControllerType::Unknown => GamepadLayout::Generic,
        }
    }
}

/// `Joystick::HatState`
#[derive(Clone, Copy, Debug, Default)]
struct HatState {
    pressed: bool,
    did_state_change: bool,
}

/// `Joystick`
#[derive(Clone, Debug, Default)]
pub struct Joystick {
    instance_id: i32,
    hat_state: [HatState; 4],
    lock_hat_state: bool,
}

impl Joystick {
    /// Original: `Joystick::ToControllerButtonEvents` (controls/devices/joystick.cpp). No
    /// `JOY_BUTTON_*` mapping is defined, so every button is `ControllerButton::Ignore`.
    // @port controls/devices/joystick.cpp|devilution::Joystick::ToControllerButtonEvents(const SDL_Event &event) sha=9e0c3cebba99
    pub fn to_controller_button_events(ctx: &mut Ctx, event: &Event) -> Vec<ControllerButtonEvent> {
        match *event {
            Event::JoyButtonDown { .. } | Event::JoyButtonUp { .. } => {
                let up = matches!(event, Event::JoyButtonUp { .. });
                vec![ControllerButtonEvent { button: ControllerButton::Ignore, up }]
            }
            Event::JoyHatMotion { .. } => {
                let Some(i) = Self::get_for_event(ctx, event) else {
                    return vec![ControllerButtonEvent { button: ControllerButton::Ignore, up: false }];
                };
                let joystick = &mut ctx.controls.devices.joysticks[i];
                joystick.update_hat_state(event);
                joystick.get_hat_events()
            }
            // ProcessAxisMotion() requires a ControllerButtonEvent parameter
            // so provide one here using ControllerButton_NONE
            Event::JoyAxisMotion { .. } => vec![ControllerButtonEvent { button: ControllerButton::None, up: false }],
            _ => vec![],
        }
    }

    /// Original: `Joystick::GetHatEvents` (controls/devices/joystick.cpp).
    // @port controls/devices/joystick.cpp|devilution::Joystick::GetHatEvents() sha=c305674674d9
    fn get_hat_events(&self) -> Vec<ControllerButtonEvent> {
        let mut hat_events = Vec::new();
        let buttons = [ControllerButton::ButtonDpadUp, ControllerButton::ButtonDpadDown, ControllerButton::ButtonDpadLeft, ControllerButton::ButtonDpadRight];
        for (state, button) in self.hat_state.iter().zip(buttons) {
            if state.did_state_change {
                hat_events.push(ControllerButtonEvent { button, up: !state.pressed });
            }
        }
        if hat_events.is_empty() {
            hat_events.push(ControllerButtonEvent { button: ControllerButton::Ignore, up: false });
        }
        hat_events
    }

    /// Original: `Joystick::UpdateHatState` (controls/devices/joystick.cpp). No `JOY_HAT_*`
    /// mapping is defined, so only the lock is taken.
    // @port controls/devices/joystick.cpp|devilution::Joystick::UpdateHatState(const SDL_JoyHatEvent &event) sha=000e26fbd761
    fn update_hat_state(&mut self, _event: &Event) {
        if self.lock_hat_state {
            return;
        }
        self.lock_hat_state = true;
    }

    /// Original: `Joystick::UnlockHatState` (controls/devices/joystick.cpp).
    // @port controls/devices/joystick.cpp|devilution::Joystick::UnlockHatState() sha=1a2c25c6fa3d
    pub fn unlock_hat_state(&mut self) {
        self.lock_hat_state = false;
        for hat_state in self.hat_state.iter_mut() {
            hat_state.did_state_change = false;
        }
    }

    /// Original: `Joystick::ToSdlJoyButton` (controls/devices/joystick.cpp): no `JOY_BUTTON_*`
    /// mapping is defined.
    // @port controls/devices/joystick.cpp|devilution::Joystick::ToSdlJoyButton(ControllerButton button) sha=82437c10eabc
    fn to_sdl_joy_button(_button: ControllerButton) -> i32 {
        -1
    }

    /// Original: `Joystick::IsHatButtonPressed` (controls/devices/joystick.cpp): no `JOY_HAT_*`
    /// mapping is defined.
    // @port controls/devices/joystick.cpp|devilution::Joystick::IsHatButtonPressed(ControllerButton button) sha=5d6ef54fc027
    fn is_hat_button_pressed(&self, _button: ControllerButton) -> bool {
        false
    }

    /// Original: `Joystick::IsPressed` (controls/devices/joystick.cpp).
    // @port controls/devices/joystick.cpp|devilution::Joystick::IsPressed(ControllerButton button) sha=bff2c2efa63f
    pub fn is_pressed(&self, button: ControllerButton) -> bool {
        if self.is_hat_button_pressed(button) {
            return true;
        }
        // SDL_JoystickGetButton is only reached with a joystick button mapping.
        Self::to_sdl_joy_button(button) != -1
    }

    /// Original: `Joystick::ProcessAxisMotion` (controls/devices/joystick.cpp): no `JOY_AXIS_*`
    /// mapping is defined.
    // @port controls/devices/joystick.cpp|devilution::Joystick::ProcessAxisMotion(const SDL_Event &event) sha=30b7f50ad0d5
    pub fn process_axis_motion(_ctx: &mut Ctx, event: &Event) -> bool {
        let Event::JoyAxisMotion { .. } = event else { return false };
        false
    }

    /// Original: `Joystick::Add` (controls/devices/joystick.cpp). The platform has no raw
    /// joysticks to open.
    // @port controls/devices/joystick.cpp|devilution::Joystick::Add(int deviceIndex) sha=64c1648341f3
    pub fn add(_ctx: &mut Ctx, device_index: i32) {
        log::info!("Adding joystick {}: no raw joystick devices", device_index);
    }

    /// Original: `Joystick::Remove` (controls/devices/joystick.cpp).
    // @port controls/devices/joystick.cpp|devilution::Joystick::Remove(SDL_JoystickID instanceId) sha=dbbb13f00cae
    pub fn remove(ctx: &mut Ctx, instance_id: i32) {
        log::info!("Removing joystick (instance id: {})", instance_id);
        let joysticks = &mut ctx.controls.devices.joysticks;
        if let Some(i) = joysticks.iter().position(|j| j.instance_id == instance_id) {
            joysticks.remove(i);
            return;
        }
        log::info!("Joystick not found with instance id: {}", instance_id);
    }

    /// Original: `Joystick::All` (controls/devices/joystick.cpp).
    // @port controls/devices/joystick.cpp|devilution::Joystick::All() sha=679949666a4a
    pub fn all(ctx: &Ctx) -> &[Joystick] {
        &ctx.controls.devices.joysticks
    }

    /// Original: `Joystick::Get(SDL_JoystickID)` (controls/devices/joystick.cpp).
    // @port controls/devices/joystick.cpp|devilution::Joystick::Get(SDL_JoystickID instanceId) sha=f51338f41c2e
    pub fn get(ctx: &Ctx, instance_id: i32) -> Option<usize> {
        ctx.controls.devices.joysticks.iter().position(|j| j.instance_id == instance_id)
    }

    /// Original: `Joystick::Get(const SDL_Event &)` (controls/devices/joystick.cpp).
    // @port controls/devices/joystick.cpp|devilution::Joystick::Get(const SDL_Event &event) sha=64c7ef20d774
    pub fn get_for_event(ctx: &Ctx, event: &Event) -> Option<usize> {
        match *event {
            Event::JoyAxisMotion { which, .. } | Event::JoyHatMotion { which, .. } | Event::JoyButtonDown { which, .. } | Event::JoyButtonUp { which, .. } => {
                Self::get(ctx, which)
            }
            _ => None,
        }
    }

    /// Original: `Joystick::IsPressedOnAnyJoystick` (controls/devices/joystick.cpp).
    // @port controls/devices/joystick.cpp|devilution::Joystick::IsPressedOnAnyJoystick(ControllerButton button) sha=9d1a4bc4b82e
    pub fn is_pressed_on_any_joystick(ctx: &Ctx, button: ControllerButton) -> bool {
        ctx.controls.devices.joysticks.iter().any(|j| j.is_pressed(button))
    }

    /// Original: `Joystick::instance_id` (controls/devices/joystick.h).
    // @port controls/devices/joystick.h|devilution::Joystick::instance_id() sha=c79b68a9d9f4
    pub fn instance_id(&self) -> i32 {
        self.instance_id
    }
}
