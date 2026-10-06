//! Input events in SDL2 terms (keycodes, modifiers, buttons), so the ported input code can keep
//! the original's key handling and the key names stored in diablo.ini.

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Quit,
    KeyDown { key: i32, mods: u16 },
    KeyUp { key: i32, mods: u16 },
    /// Mouse position in game (back buffer) coordinates.
    MouseMotion { x: i32, y: i32 },
    /// `clicks`: 1 for a single click, 2 for a double click (filled in by the platform).
    MouseButtonDown { button: u8, x: i32, y: i32, clicks: u8 },
    MouseButtonUp { button: u8, x: i32, y: i32, clicks: u8 },
    MouseWheel { x: i32, y: i32 },
    TextInput(String),
    FocusGained,
    FocusLost,
    /// `SDL_WINDOWEVENT_HIDDEN` / `MINIMIZED` (the window became occluded).
    WindowHidden,
    /// `SDL_WINDOWEVENT_SHOWN` / `EXPOSED` / `RESTORED`.
    WindowShown,
    /// `SDL_WINDOWEVENT_SIZE_CHANGED`
    WindowSizeChanged,
    /// `SDL_WINDOWEVENT_LEAVE`: the mouse left the window.
    WindowLeave,
    /// Front-end to platform only: a game controller was connected (`instance_id` is unique per
    /// connection, as SDL's). The platform turns it into `ControllerDeviceAdded`.
    PadConnected { instance_id: i32, vendor_id: u16 },
    /// Front-end to platform only: becomes `ControllerDeviceRemoved`.
    PadDisconnected { instance_id: i32 },
    /// `SDL_CONTROLLERDEVICEADDED`: `which` is the device index.
    ControllerDeviceAdded { which: i32 },
    /// `SDL_CONTROLLERDEVICEREMOVED`: `which` is the instance id.
    ControllerDeviceRemoved { which: i32 },
    /// `SDL_CONTROLLERBUTTONDOWN` (`button` is an `SDL_GameControllerButton`)
    ControllerButtonDown { which: i32, button: u8 },
    /// `SDL_CONTROLLERBUTTONUP`
    ControllerButtonUp { which: i32, button: u8 },
    /// `SDL_CONTROLLERAXISMOTION` (`axis` is an `SDL_GameControllerAxis`; Y axes point down)
    ControllerAxisMotion { which: i32, axis: u8, value: i16 },
    /// `SDL_JOYDEVICEADDED` / `SDL_JOYDEVICEREMOVED` and the raw joystick events. The Bevy
    /// front-end delivers every device as a game controller, so it produces none of these; they
    /// exist so the joystick paths of the original can be ported as they are.
    JoyDeviceAdded { which: i32 },
    JoyDeviceRemoved { which: i32 },
    JoyButtonDown { which: i32, button: u8 },
    JoyButtonUp { which: i32, button: u8 },
    JoyAxisMotion { which: i32, axis: u8, value: i16 },
    JoyHatMotion { which: i32, hat: u8, value: u8 },
    /// A game event (`CustomEventToSdlEvent(interface_mode)`, interfac.cpp).
    Custom(crate::enums::interface_mode),
    /// Test hook (input script `warp <level>`): enter that dungeon level as if taking the stairs down.
    TestWarp(i32),
    /// Test hook (input script `setwarp <setlevel> <dungeon type>`): enter that quest level.
    TestSetWarp(i32, i32),
    /// Test hook (input script `store <TalkID>`): open that store page as if talking to its owner.
    TestStore(i32),
    /// Test hook (input script `killdiablo`): Diablo dies as if the player had killed him.
    TestKillDiablo,
}

pub const BUTTON_LEFT: u8 = 1;
pub const BUTTON_MIDDLE: u8 = 2;
pub const BUTTON_RIGHT: u8 = 3;
pub const BUTTON_X1: u8 = 4;
pub const BUTTON_X2: u8 = 5;

pub const KMOD_LSHIFT: u16 = 0x0001;
pub const KMOD_RSHIFT: u16 = 0x0002;
pub const KMOD_LCTRL: u16 = 0x0040;
pub const KMOD_RCTRL: u16 = 0x0080;
pub const KMOD_LALT: u16 = 0x0100;
pub const KMOD_RALT: u16 = 0x0200;
pub const KMOD_SHIFT: u16 = KMOD_LSHIFT | KMOD_RSHIFT;
pub const KMOD_CTRL: u16 = KMOD_LCTRL | KMOD_RCTRL;
pub const KMOD_ALT: u16 = KMOD_LALT | KMOD_RALT;

const fn sc(n: i32) -> i32 {
    n | (1 << 30)
}

/// SDL2 keycodes used by the game.
#[allow(non_upper_case_globals)]
pub mod keys {
    use super::sc;
    pub const SDLK_UNKNOWN: i32 = 0;
    pub const SDLK_BACKSPACE: i32 = 8;
    pub const SDLK_TAB: i32 = 9;
    pub const SDLK_RETURN: i32 = 13;
    pub const SDLK_ESCAPE: i32 = 27;
    pub const SDLK_SPACE: i32 = 32;
    pub const SDLK_DELETE: i32 = 127;
    pub const SDLK_CAPSLOCK: i32 = sc(57);
    pub const SDLK_F1: i32 = sc(58);
    pub const SDLK_F11: i32 = sc(68);
    pub const SDLK_F12: i32 = sc(69);
    pub const SDLK_PRINTSCREEN: i32 = sc(70);
    pub const SDLK_SCROLLLOCK: i32 = sc(71);
    pub const SDLK_PAUSE: i32 = sc(72);
    pub const SDLK_INSERT: i32 = sc(73);
    pub const SDLK_HOME: i32 = sc(74);
    pub const SDLK_PAGEUP: i32 = sc(75);
    pub const SDLK_END: i32 = sc(77);
    pub const SDLK_PAGEDOWN: i32 = sc(78);
    pub const SDLK_RIGHT: i32 = sc(79);
    pub const SDLK_LEFT: i32 = sc(80);
    pub const SDLK_DOWN: i32 = sc(81);
    pub const SDLK_UP: i32 = sc(82);
    pub const SDLK_NUMLOCKCLEAR: i32 = sc(83);
    pub const SDLK_KP_DIVIDE: i32 = sc(84);
    pub const SDLK_KP_MULTIPLY: i32 = sc(85);
    pub const SDLK_KP_MINUS: i32 = sc(86);
    pub const SDLK_PLUS: i32 = b'+' as i32;
    pub const SDLK_EQUALS: i32 = b'=' as i32;
    pub const SDLK_MINUS: i32 = b'-' as i32;
    pub const SDLK_UNDERSCORE: i32 = b'_' as i32;
    pub const SDLK_KP_EQUALS: i32 = sc(103);
    pub const SDLK_KP_PLUS: i32 = sc(87);
    pub const SDLK_KP_ENTER: i32 = sc(88);
    pub const SDLK_KP_1: i32 = sc(89);
    pub const SDLK_KP_0: i32 = sc(98);
    pub const SDLK_KP_PERIOD: i32 = sc(99);
    pub const SDLK_LCTRL: i32 = sc(224);
    pub const SDLK_LSHIFT: i32 = sc(225);
    pub const SDLK_LALT: i32 = sc(226);
    pub const SDLK_LGUI: i32 = sc(227);
    pub const SDLK_RCTRL: i32 = sc(228);
    pub const SDLK_RSHIFT: i32 = sc(229);
    pub const SDLK_RALT: i32 = sc(230);
    pub const SDLK_RGUI: i32 = sc(231);
}

/// `SDL_GameControllerButton`
pub mod pad {
    pub const BUTTON_A: u8 = 0;
    pub const BUTTON_B: u8 = 1;
    pub const BUTTON_X: u8 = 2;
    pub const BUTTON_Y: u8 = 3;
    pub const BUTTON_BACK: u8 = 4;
    pub const BUTTON_GUIDE: u8 = 5;
    pub const BUTTON_START: u8 = 6;
    pub const BUTTON_LEFTSTICK: u8 = 7;
    pub const BUTTON_RIGHTSTICK: u8 = 8;
    pub const BUTTON_LEFTSHOULDER: u8 = 9;
    pub const BUTTON_RIGHTSHOULDER: u8 = 10;
    pub const BUTTON_DPAD_UP: u8 = 11;
    pub const BUTTON_DPAD_DOWN: u8 = 12;
    pub const BUTTON_DPAD_LEFT: u8 = 13;
    pub const BUTTON_DPAD_RIGHT: u8 = 14;
    /// `SDL_CONTROLLER_BUTTON_INVALID`
    pub const BUTTON_INVALID: u8 = 0xFF;

    /// `SDL_GameControllerAxis`
    pub const AXIS_LEFTX: u8 = 0;
    pub const AXIS_LEFTY: u8 = 1;
    pub const AXIS_RIGHTX: u8 = 2;
    pub const AXIS_RIGHTY: u8 = 3;
    pub const AXIS_TRIGGERLEFT: u8 = 4;
    pub const AXIS_TRIGGERRIGHT: u8 = 5;
}
