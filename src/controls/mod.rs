//! `Source/controls/*`

#[allow(unused_imports)]
use crate::ctx::Ctx;

pub mod controller;
pub mod controller_buttons;
pub mod devices;
pub mod game_controls;
pub mod plrctrls;
pub mod touch;

/// `ControlTypes` (controls/plrctrls.h)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ControlTypes {
    #[default]
    None,
    KeyboardAndMouse,
    Gamepad,
    VirtualGamepad,
}

/// Globals of the controls/ sources.
#[derive(Default)]
pub struct ControlsState {
    /// `ControlMode` (controls/plrctrls.cpp)
    pub control_mode: ControlTypes,
    /// `SuppressedButton` (controls/game_controls.cpp)
    pub suppressed_button: controller_buttons::ControllerButton,
    /// `StandToggle` (controls/game_controls.cpp)
    pub stand_toggle: bool,
    /// `PadHotspellMenuActive`, `PadMenuNavigatorActive` (controls/game_controls.cpp)
    pub pad_hotspell_menu_active: bool,
    pub pad_menu_navigator_active: bool,
    /// `ControlDevice` (controls/plrctrls.cpp)
    pub control_device: ControlTypes,
    /// `ControllerActionHeld` (controls/plrctrls.cpp)
    pub controller_action_held: game_controls::GameActionType,
    /// `GamepadType` (controls/plrctrls.cpp)
    pub gamepad_type: game_controls::GamepadLayout,
    /// stick values (controls/controller_motion.cpp)
    pub sticks: controller::StickState,
    /// static `repeater` in GetMenuHeldUpDownAction
    pub menu_held_repeater: controller::AxisDirectionRepeater,
    /// Globals of controls/plrctrls.cpp
    pub plrctrls: plrctrls::PlrCtrlsState,
    /// Connected game controllers and joysticks (controls/devices/*.cpp)
    pub devices: devices::DevicesState,
}

/// `remap_keyboard_key` (controls/remap_keyboard.h): no remapping unless REMAP_KEYBOARD_KEYS
/// (not defined for Windows).
// @port controls/remap_keyboard.h|devilution::remap_keyboard_key(SDL_Keycode *sym) sha=267147c5bb5a
pub fn remap_keyboard_key(sym: i32) -> i32 {
    sym
}

/// Original: `devilution::SetCursorPos` (diablo.cpp). `LogicalToOutput` is done by the front-end.
// @port diablo.cpp|devilution::SetCursorPos(Point position) sha=4fbd7538db89
pub fn set_cursor_pos(ctx: &mut Ctx, position: (i32, i32)) {
    if ctx.controls.control_device != ControlTypes::KeyboardAndMouse {
        ctx.diablo.mouse_position = position;
        return;
    }
    if !crate::engine::demomode::is_running(ctx) {
        ctx.platform.warp_mouse(position.0, position.1);
    }
}
pub mod modifier_hints;
