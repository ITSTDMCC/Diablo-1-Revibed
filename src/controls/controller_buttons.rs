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
/// Original: `devilution::ToPlayStationIcon` (controls/controller_buttons.cpp).
// @port controls/controller_buttons.cpp|devilution::ToPlayStationIcon(ControllerButton button) sha=924805779a50
pub fn to_play_station_icon(button: ControllerButton) -> &'static str {
    match button {
        ControllerButton::ButtonA => "\u{E002}",
        ControllerButton::ButtonB => "\u{E003}",
        ControllerButton::ButtonX => "\u{E001}",
        ControllerButton::ButtonY => "\u{E000}",
        ControllerButton::ButtonStart => "\u{E004}",
        ControllerButton::ButtonBack => "\u{E005}",
        ControllerButton::AxisTriggerLeft => "\u{E006}",
        ControllerButton::AxisTriggerRight => "\u{E007}",
        ControllerButton::ButtonLeftShoulder => "\u{E008}",
        ControllerButton::ButtonRightShoulder => "\u{E009}",
        ControllerButton::ButtonLeftStick => "\u{E017}",
        ControllerButton::ButtonRightStick => "\u{E021}",
        ControllerButton::ButtonDpadUp => "\u{E00A}",
        ControllerButton::ButtonDpadDown => "\u{E00C}",
        ControllerButton::ButtonDpadLeft => "\u{E00D}",
        ControllerButton::ButtonDpadRight => "\u{E00B}",
        _ => to_generic_button_text(button),
    }
}

/// Original: `devilution::ToNintendoIcon` (controls/controller_buttons.cpp).
// @port controls/controller_buttons.cpp|devilution::ToNintendoIcon(ControllerButton button) sha=4cdf77784e4a
pub fn to_nintendo_icon(button: ControllerButton) -> &'static str {
    match button {
        ControllerButton::ButtonA => "\u{E025}",
        ControllerButton::ButtonB => "\u{E026}",
        ControllerButton::ButtonX => "\u{E024}",
        ControllerButton::ButtonY => "\u{E023}",
        ControllerButton::ButtonStart => "\u{E027}",
        ControllerButton::ButtonBack => "\u{E028}",
        ControllerButton::AxisTriggerLeft => "\u{E029}",
        ControllerButton::AxisTriggerRight => "\u{E02A}",
        ControllerButton::ButtonLeftShoulder => "\u{E02B}",
        ControllerButton::ButtonRightShoulder => "\u{E02C}",
        ControllerButton::ButtonLeftStick => "\u{E03A}",
        ControllerButton::ButtonRightStick => "\u{E044}",
        ControllerButton::ButtonDpadUp => "\u{E02D}",
        ControllerButton::ButtonDpadDown => "\u{E02F}",
        ControllerButton::ButtonDpadLeft => "\u{E030}",
        ControllerButton::ButtonDpadRight => "\u{E02E}",
        _ => to_generic_button_text(button),
    }
}

/// Original: `devilution::ToXboxIcon` (controls/controller_buttons.cpp).
// @port controls/controller_buttons.cpp|devilution::ToXboxIcon(ControllerButton button) sha=1728d4f32a29
pub fn to_xbox_icon(button: ControllerButton) -> &'static str {
    match button {
        ControllerButton::ButtonA => "\u{E04B}",
        ControllerButton::ButtonB => "\u{E04C}",
        ControllerButton::ButtonX => "\u{E04A}",
        ControllerButton::ButtonY => "\u{E049}",
        ControllerButton::ButtonStart => "\u{E04D}",
        ControllerButton::ButtonBack => "\u{E04E}",
        ControllerButton::AxisTriggerLeft => "\u{E04F}",
        ControllerButton::AxisTriggerRight => "\u{E050}",
        ControllerButton::ButtonLeftShoulder => "\u{E051}",
        ControllerButton::ButtonRightShoulder => "\u{E052}",
        ControllerButton::ButtonLeftStick => "\u{E05F}",
        ControllerButton::ButtonRightStick => "\u{E069}",
        ControllerButton::ButtonDpadUp => "\u{E053}",
        ControllerButton::ButtonDpadDown => "\u{E055}",
        ControllerButton::ButtonDpadLeft => "\u{E056}",
        ControllerButton::ButtonDpadRight => "\u{E054}",
        _ => to_generic_button_text(button),
    }
}

/// Original: `devilution::ToGenericButtonText` (controls/controller_buttons.cpp).
// @port controls/controller_buttons.cpp|devilution::ToGenericButtonText(ControllerButton button) sha=ff50f8612910
pub fn to_generic_button_text(button: ControllerButton) -> &'static str {
    match button {
        ControllerButton::ButtonA => "A",
        ControllerButton::ButtonB => "B",
        ControllerButton::ButtonX => "X",
        ControllerButton::ButtonY => "Y",
        ControllerButton::ButtonStart => "Start",
        ControllerButton::ButtonBack => "Select",
        ControllerButton::AxisTriggerLeft => "LT",
        ControllerButton::AxisTriggerRight => "RT",
        ControllerButton::ButtonLeftShoulder => "LB",
        ControllerButton::ButtonRightShoulder => "RB",
        ControllerButton::ButtonLeftStick => "LS",
        ControllerButton::ButtonRightStick => "RS",
        ControllerButton::ButtonDpadUp => "Up",
        ControllerButton::ButtonDpadDown => "Down",
        ControllerButton::ButtonDpadLeft => "Left",
        ControllerButton::ButtonDpadRight => "Right",
        ControllerButton::None => "None",
        ControllerButton::Ignore => "Ignored",
        _ => "Unknown",
    }
}

