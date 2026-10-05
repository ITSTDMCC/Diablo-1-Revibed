//! Game controllers through Bevy's gilrs backend (cargo feature `gamepad`): raw gilrs input is
//! forwarded as SDL game controller events, so the ported `controls/` code sees what SDL would
//! give it. Raw events are used, not Bevy's filtered ones, because the game applies its own
//! dead zone (`ScaleJoystickAxes`).
//!
//! Mapping (gilrs names to SDL): South/East/West/North = A/B/X/Y, LeftTrigger/RightTrigger =
//! shoulders, LeftTrigger2/RightTrigger2 = the trigger axes, Select = Back, Mode = Guide. SDL's
//! stick Y axes point down, gilrs' point up, so they are negated.

use bevy::input::gamepad::{GamepadAxis, GamepadButton, GamepadConnection, RawGamepadEvent};
use bevy::prelude::*;

use super::events::{pad, Event};

/// Instance ids handed out per connection, as SDL does.
#[derive(Resource, Default)]
pub struct PadIds {
    ids: Vec<(Entity, i32)>,
    next: i32,
}

fn sdl_button(b: GamepadButton) -> Option<u8> {
    Some(match b {
        GamepadButton::South => pad::BUTTON_A,
        GamepadButton::East => pad::BUTTON_B,
        GamepadButton::West => pad::BUTTON_X,
        GamepadButton::North => pad::BUTTON_Y,
        GamepadButton::Select => pad::BUTTON_BACK,
        GamepadButton::Mode => pad::BUTTON_GUIDE,
        GamepadButton::Start => pad::BUTTON_START,
        GamepadButton::LeftThumb => pad::BUTTON_LEFTSTICK,
        GamepadButton::RightThumb => pad::BUTTON_RIGHTSTICK,
        GamepadButton::LeftTrigger => pad::BUTTON_LEFTSHOULDER,
        GamepadButton::RightTrigger => pad::BUTTON_RIGHTSHOULDER,
        GamepadButton::DPadUp => pad::BUTTON_DPAD_UP,
        GamepadButton::DPadDown => pad::BUTTON_DPAD_DOWN,
        GamepadButton::DPadLeft => pad::BUTTON_DPAD_LEFT,
        GamepadButton::DPadRight => pad::BUTTON_DPAD_RIGHT,
        _ => return None,
    })
}

/// -1.0..=1.0 to SDL's -32768..=32767 range.
fn axis_value(v: f32) -> i16 {
    (v.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// Translates this frame's raw gamepad events.
pub fn translate<'a>(ids: &mut PadIds, raw: impl Iterator<Item = &'a RawGamepadEvent>) -> Vec<Event> {
    let mut out = Vec::new();
    for e in raw {
        match e {
            RawGamepadEvent::Connection(c) => match &c.connection {
                GamepadConnection::Connected { vendor_id, .. } => {
                    let instance_id = ids.next;
                    ids.next += 1;
                    ids.ids.push((c.gamepad, instance_id));
                    out.push(Event::PadConnected { instance_id, vendor_id: vendor_id.unwrap_or(0) });
                }
                GamepadConnection::Disconnected => {
                    if let Some(i) = ids.ids.iter().position(|(g, _)| *g == c.gamepad) {
                        let (_, instance_id) = ids.ids.remove(i);
                        out.push(Event::PadDisconnected { instance_id });
                    }
                }
            },
            RawGamepadEvent::Button(b) => {
                let Some(&(_, which)) = ids.ids.iter().find(|(g, _)| *g == b.gamepad) else { continue };
                match b.button {
                    GamepadButton::LeftTrigger2 => out.push(Event::ControllerAxisMotion { which, axis: pad::AXIS_TRIGGERLEFT, value: axis_value(b.value) }),
                    GamepadButton::RightTrigger2 => out.push(Event::ControllerAxisMotion { which, axis: pad::AXIS_TRIGGERRIGHT, value: axis_value(b.value) }),
                    button => {
                        let Some(button) = sdl_button(button) else { continue };
                        // The platform keeps the pressed state and drops repeats, as SDL reports changes only.
                        out.push(if b.value >= 0.5 { Event::ControllerButtonDown { which, button } } else { Event::ControllerButtonUp { which, button } });
                    }
                }
            }
            RawGamepadEvent::Axis(a) => {
                let Some(&(_, which)) = ids.ids.iter().find(|(g, _)| *g == a.gamepad) else { continue };
                let (axis, value) = match a.axis {
                    GamepadAxis::LeftStickX => (pad::AXIS_LEFTX, axis_value(a.value)),
                    GamepadAxis::LeftStickY => (pad::AXIS_LEFTY, axis_value(-a.value)),
                    GamepadAxis::RightStickX => (pad::AXIS_RIGHTX, axis_value(a.value)),
                    GamepadAxis::RightStickY => (pad::AXIS_RIGHTY, axis_value(-a.value)),
                    _ => continue,
                };
                out.push(Event::ControllerAxisMotion { which, axis, value });
            }
        }
    }
    out
}
