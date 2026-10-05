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

// `controller_button_icon` glyphs (controller_buttons.cpp) used by item descriptions.
pub const PLAYSTATION_TRIANGLE: &str = "\u{E000}";
pub const PLAYSTATION_SQUARE: &str = "\u{E001}";
pub const NINTENDO_X: &str = "\u{E023}";
pub const NINTENDO_Y: &str = "\u{E024}";
pub const XBOX_Y: &str = "\u{E049}";
pub const XBOX_X: &str = "\u{E04A}";
