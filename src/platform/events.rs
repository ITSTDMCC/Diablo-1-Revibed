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
    /// A game event (`CustomEventToSdlEvent(interface_mode)`, interfac.cpp).
    Custom(crate::enums::interface_mode),
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
