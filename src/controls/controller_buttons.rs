//! `Source/controls/controller_buttons.h`

/// `ControllerButton`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum ControllerButton {
    #[default]
    None,
    Ignore,
    AxisTriggerLeft,
    AxisTriggerRight,
    ButtonA,
    ButtonB,
    ButtonX,
    ButtonY,
    ButtonLeftStick,
    ButtonRightStick,
    ButtonLeftShoulder,
    ButtonRightShoulder,
    ButtonStart,
    ButtonBack,
    ButtonDpadUp,
    ButtonDpadDown,
    ButtonDpadLeft,
    ButtonDpadRight,
}

impl ControllerButton {
    pub const COUNT: usize = 18;

    pub fn from_index(i: usize) -> ControllerButton {
        use ControllerButton::*;
        [
            None, Ignore, AxisTriggerLeft, AxisTriggerRight, ButtonA, ButtonB, ButtonX, ButtonY, ButtonLeftStick, ButtonRightStick,
            ButtonLeftShoulder, ButtonRightShoulder, ButtonStart, ButtonBack, ButtonDpadUp, ButtonDpadDown, ButtonDpadLeft, ButtonDpadRight,
        ][i]
    }
}

/// `ControllerButtonCombo`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ControllerButtonCombo {
    pub modifier: ControllerButton,
    pub button: ControllerButton,
}

impl ControllerButtonCombo {
    pub const fn single(button: ControllerButton) -> Self {
        ControllerButtonCombo { modifier: ControllerButton::None, button }
    }
    pub const fn new(modifier: ControllerButton, button: ControllerButton) -> Self {
        ControllerButtonCombo { modifier, button }
    }
}

/// `IsDPadButton`
pub fn is_dpad_button(b: ControllerButton) -> bool {
    matches!(b, ControllerButton::ButtonDpadUp | ControllerButton::ButtonDpadDown | ControllerButton::ButtonDpadLeft | ControllerButton::ButtonDpadRight)
}
