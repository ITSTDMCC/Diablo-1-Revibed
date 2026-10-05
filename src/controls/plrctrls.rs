//! `Source/controls/plrctrls` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn is_movement_handler_active(ctx: &crate::ctx::Ctx) -> bool, "controls/plrctrls.cpp|devilution::IsMovementHandlerActive()");

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

crate::pending_fn!(pub fn focus_on_char_info(ctx: &mut Ctx), "controls/plrctrls.cpp|devilution::FocusOnCharInfo()");

crate::pending_fn!(pub fn try_drop_item(ctx: &mut crate::ctx::Ctx) -> bool, "controls/plrctrls.cpp|devilution::TryDropItem()");
