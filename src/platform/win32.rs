//! The few Win32 calls the original makes through SDL or directly (no crate: plain FFI).

#![allow(non_snake_case, clippy::upper_case_acronyms)]

#[cfg(windows)]
mod ffi {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        pub fn GetUserDefaultLocaleName(lpLocaleName: *mut u16, cchLocaleName: i32) -> i32;
        pub fn GetUserPreferredUILanguages(dwFlags: u32, pulNumLanguages: *mut u32, pwszLanguagesBuffer: *mut u16, pcchLanguagesBuffer: *mut u32) -> i32;
        pub fn lstrlenW(lpString: *const u16) -> i32;
        pub fn WideCharToMultiByte(
            CodePage: u32,
            dwFlags: u32,
            lpWideCharStr: *const u16,
            cchWideChar: i32,
            lpMultiByteStr: *mut u8,
            cbMultiByte: i32,
            lpDefaultChar: *const u8,
            lpUsedDefaultChar: *mut i32,
        ) -> i32;
    }
    #[link(name = "user32")]
    unsafe extern "system" {
        pub fn EnumDisplaySettingsW(lpszDeviceName: *const u16, iModeNum: u32, lpDevMode: *mut u8) -> i32;
        pub fn MessageBoxW(hWnd: *mut core::ffi::c_void, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
    }
}

const LOCALE_NAME_MAX_LENGTH: usize = 85;
const MUI_LANGUAGE_NAME: u32 = 0x8;
const CP_UTF8: u32 = 65001;
const ENUM_CURRENT_SETTINGS: u32 = 0xFFFF_FFFF;
const DEVMODEW_SIZE: usize = 220;

/// `IetfToPosix` (platform/locale.cpp)
fn ietf_to_posix(lang: &str) -> String {
    if lang.starts_with("zh-Hans") {
        return "zh_CN".to_string();
    }
    if lang.starts_with("zh-Hant") {
        return "zh_TW".to_string();
    }
    lang.replace('-', "_")
}

/// `GetLocales` (platform/locale.cpp), Windows branch. As in the original, the offset into the
/// preferred-language list advances by the length of the *first* entry each time.
#[cfg(windows)]
pub fn get_locales() -> Vec<String> {
    let mut locales = Vec::new();
    let wide_char_to_utf8 = |p: *const u16| -> String {
        let mut buf = [0u8; 16];
        // SAFETY: p points into a NUL-terminated buffer of at least 10 wide chars or terminates earlier.
        unsafe { ffi::WideCharToMultiByte(CP_UTF8, 0, p, 10, buf.as_mut_ptr(), buf.len() as i32, std::ptr::null(), std::ptr::null_mut()) };
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        ietf_to_posix(&String::from_utf8_lossy(&buf[..end]))
    };
    let mut buffer = [0u16; LOCALE_NAME_MAX_LENGTH];
    // SAFETY: buffer is LOCALE_NAME_MAX_LENGTH wide chars.
    if unsafe { ffi::GetUserDefaultLocaleName(buffer.as_mut_ptr(), (buffer.len() * 2) as i32) } != 0 {
        locales.push(wide_char_to_utf8(buffer.as_ptr()));
    }
    let mut count = 0u32;
    let mut size = (buffer.len() * 2) as u32;
    // SAFETY: size describes the buffer (the original passes the byte size, as here).
    unsafe { ffi::GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, buffer.as_mut_ptr(), &mut size) };
    let mut offset = 0usize;
    for _ in 0..count {
        if offset >= buffer.len() {
            break;
        }
        locales.push(wide_char_to_utf8(buffer[offset..].as_ptr()));
        // SAFETY: buffer is NUL-terminated.
        offset += unsafe { ffi::lstrlenW(buffer.as_ptr()) } as usize + 1;
    }
    locales
}

#[cfg(not(windows))]
pub fn get_locales() -> Vec<String> {
    Vec::new()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DisplayMode {
    pub w: i32,
    pub h: i32,
    pub refresh_rate: i32,
}

#[cfg(windows)]
fn enum_display(mode: u32) -> Option<DisplayMode> {
    let mut dm = [0u8; DEVMODEW_SIZE];
    dm[68..70].copy_from_slice(&(DEVMODEW_SIZE as u16).to_le_bytes());
    // SAFETY: dm is a DEVMODEW-sized buffer with dmSize set.
    if unsafe { ffi::EnumDisplaySettingsW(std::ptr::null(), mode, dm.as_mut_ptr()) } == 0 {
        return None;
    }
    let u32_at = |o: usize| u32::from_le_bytes(dm[o..o + 4].try_into().unwrap());
    Some(DisplayMode { w: u32_at(172) as i32, h: u32_at(176) as i32, refresh_rate: u32_at(184) as i32 })
}

/// `SDL_GetDesktopDisplayMode(0)`
#[cfg(windows)]
pub fn desktop_display_mode() -> DisplayMode {
    enum_display(ENUM_CURRENT_SETTINGS).unwrap_or(DisplayMode { w: 640, h: 480, refresh_rate: 60 })
}

/// `SDL_GetNumDisplayModes(0)` / `SDL_GetDisplayMode(0, i)`: distinct modes, largest first
/// (SDL sorts by width, height, bpp, refresh rate, descending).
#[cfg(windows)]
pub fn display_modes() -> Vec<DisplayMode> {
    let mut modes = Vec::new();
    let mut i = 0;
    while let Some(m) = enum_display(i) {
        if !modes.contains(&m) {
            modes.push(m);
        }
        i += 1;
    }
    modes.sort_by(|a, b| (b.w, b.h, b.refresh_rate).cmp(&(a.w, a.h, a.refresh_rate)));
    modes
}

#[cfg(not(windows))]
pub fn desktop_display_mode() -> DisplayMode {
    DisplayMode { w: 640, h: 480, refresh_rate: 60 }
}

#[cfg(not(windows))]
pub fn display_modes() -> Vec<DisplayMode> {
    vec![desktop_display_mode()]
}

/// `SDL_ShowSimpleMessageBox(SDL_MESSAGEBOX_ERROR, ...)`
pub fn show_error_message_box(caption: &str, text: &str) {
    #[cfg(windows)]
    {
        let w = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
        let (t, c) = (w(text), w(caption));
        const MB_ICONERROR: u32 = 0x10;
        // SAFETY: both strings are NUL-terminated UTF-16.
        unsafe { ffi::MessageBoxW(std::ptr::null_mut(), t.as_ptr(), c.as_ptr(), MB_ICONERROR) };
    }
    #[cfg(not(windows))]
    eprintln!("{caption}: {text}");
}
