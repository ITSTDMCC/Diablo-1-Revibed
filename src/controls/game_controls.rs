//! `Source/controls/game_controls.cpp`

#[allow(unused_imports)]
use crate::ctx::Ctx;
use super::controller_buttons::ControllerButton;

/// `GameActionType`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum GameActionType {
    #[default]
    None,
    UseHealthPotion,
    UseManaPotion,
    PrimaryAction,
    SecondaryAction,
    CastSpell,
    ToggleInventory,
    ToggleCharacterInfo,
    ToggleQuickSpellMenu,
    ToggleSpellBook,
    ToggleQuestLog,
    SendKey,
}

crate::pending_fn!(pub fn to_string(ctx: &Ctx, button: ControllerButton) -> String, "controls/controller_buttons.cpp|devilution::ToString(ControllerButton button)");

/// `GamepadLayout`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GamepadLayout {
    #[default]
    Generic,
    Nintendo,
    PlayStation,
    Xbox,
}

/// Original: `devilution::TranslateTo` (controls/game_controls.cpp), with `GamepadType`.
// @port controls/game_controls.cpp|devilution::TranslateTo(GamepadLayout layout, ControllerButton button) sha=9a97d638d084
pub fn translate_to(ctx: &Ctx, button: ControllerButton) -> ControllerButton {
    if ctx.controls.gamepad_type != GamepadLayout::Nintendo {
        return button;
    }
    match button {
        ControllerButton::ButtonA => ControllerButton::ButtonB,
        ControllerButton::ButtonB => ControllerButton::ButtonA,
        ControllerButton::ButtonX => ControllerButton::ButtonY,
        ControllerButton::ButtonY => ControllerButton::ButtonX,
        b => b,
    }
}

/// Original: `devilution::SkipsMovie` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::SkipsMovie(ControllerButtonEvent ctrlEvent) sha=66842f51137e
pub fn skips_movie(e: super::controller::ControllerButtonEvent) -> bool {
    matches!(e.button, ControllerButton::ButtonA | ControllerButton::ButtonB | ControllerButton::ButtonStart | ControllerButton::ButtonBack)
}

/// Original: `devilution::IsSimulatedMouseClickBinding` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::IsSimulatedMouseClickBinding(ControllerButtonEvent ctrlEvent) sha=f3483229192e
pub fn is_simulated_mouse_click_binding(ctx: &Ctx, e: super::controller::ControllerButtonEvent) -> bool {
    if e.button == ControllerButton::None {
        return false;
    }
    if !e.up && e.button == ctx.controls.suppressed_button {
        return false;
    }
    let action = crate::options::padmapper_action_name_triggered_by_button_event(ctx, e.button, e.up);
    matches!(action.as_str(), "LeftMouseClick1" | "LeftMouseClick2" | "RightMouseClick1" | "RightMouseClick2")
}

/// Original: `devilution::GetMoveDirection` (controls/game_controls.cpp).
// @port controls/game_controls.cpp|devilution::GetMoveDirection() sha=29758e684ec2
pub fn get_move_direction(ctx: &Ctx) -> super::controller::AxisDirection {
    super::controller::get_left_stick_or_dpad_direction(ctx, true)
}
