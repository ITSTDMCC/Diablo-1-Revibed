//! Platform layer: what DevilutionX gets from SDL (window, 8-bit presentation, events, ticks,
//! delays), provided to the game thread by a front-end.
//!
//! The game logic keeps the original's imperative control flow (blocking dialog loops,
//! `RunGameLoop`) on its own thread. Front-ends:
//! - `bevy_front`: a Bevy app on the main thread shows presented frames and forwards input.
//! - headless: no window; used by tests and scripted runs (`DIABLO_HEADLESS=1`).
//!
//! Test hooks (environment variables, test-only, documented in the README):
//! - `DIABLO_HEADLESS=1`            run without a window
//! - `DIABLO_FIXED_STEP=1`          virtual clock: time advances only through `delay`, so runs replay exactly
//! - `DIABLO_SCREENSHOT_FRAMES=a,b` write `frame_<n>.png` for those presented frames
//! - `DIABLO_SCREENSHOT_DIR=dir`    where screenshots go (default `screenshots`)
//! - `DIABLO_INPUT_SCRIPT=file`     scripted input: lines `<frame> key <sdl keycode>`, `<frame> click <x> <y>`,
//!                                  `<frame> move <x> <y>`, `<frame> text <string>`, `<frame> quit`,
//!                                  `<frame> warp <level>` (enter a dungeon level directly)
//! - `DIABLO_MAX_FRAMES=n`          send Quit after n presented frames
//! - `DIABLO_NO_SAVE=1`, `DIABLO_NO_AUDIO=1` are read by the game code that owns saving/audio.

pub mod audio;
pub mod bevy_front;
pub mod events;
pub mod png;
pub mod win32;

/// `platform/locale.cpp`
pub mod locale {
    pub use super::win32::get_locales;
}

/// Display information for the resolution option list.
pub mod display {
    use crate::options::ResolutionInputs;

    /// Display modes and desktop mode for `OptionEntryResolution::CheckResolutionsAreInitialized`
    /// (SDL_GetNumDisplayModes / SDL_GetDisplayMode / SDL_GetDesktopDisplayMode).
    pub fn resolution_inputs() -> ResolutionInputs {
        let d = super::win32::desktop_display_mode();
        ResolutionInputs {
            modes: super::win32::display_modes().iter().map(|m| (m.w, m.h)).collect(),
            desktop: (d.w, d.h),
            scale_factor: 1.0,
            upscale: true,
            fit_to_screen: true,
        }
    }
}

/// `SDL_GetNumAudioDevices(false)`
pub fn num_audio_devices() -> usize {
    audio::output_device_names().len()
}

/// `SDL_GetAudioDeviceName(index - 1, false)`; index 0 is the default device ("").
pub fn audio_device_name(index: usize) -> String {
    if index == 0 {
        return String::new();
    }
    audio::output_device_names().get(index - 1).cloned().unwrap_or_default()
}

/// Window state requests from the game thread to the front-end (SDL window calls).
#[derive(Clone, Debug, PartialEq)]
pub enum FrontCommand {
    /// `SDL_CreateWindow` + show
    CreateWindow { title: String, width: i32, height: i32, fullscreen: Fullscreen, resizable: bool },
    SetFullscreen(Fullscreen),
    SetWindowSize(i32, i32),
    SetResizable(bool),
    SetGrab(bool),
    ShowCursor(bool),
    /// `SDL_RenderSetLogicalSize` / integer scaling / filtering of the presented image
    SetPresentation { integer_scale: bool, linear_filter: bool },
    Hide,
    /// `SDL_CreateColorCursor` + `SDL_SetCursor`: RGBA pixels, size, hot spot.
    SetCursorImage { rgba: Vec<u8>, width: u32, height: u32, hotspot: (u16, u16) },
    /// `SDL_WarpMouseInWindow`, in game (back buffer) coordinates.
    WarpMouse(i32, i32),
}

/// Window state the front-end reports back (`SDL_RenderGetScale`, `SDL_GetWindowSize`).
#[derive(Clone, Copy, Debug)]
pub struct WindowInfo {
    pub render_scale: f32,
    pub size: (i32, i32),
}

pub type WindowInfoSlot = Arc<Mutex<WindowInfo>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fullscreen {
    Windowed,
    /// SDL_WINDOW_FULLSCREEN_DESKTOP
    Desktop,
    /// SDL_WINDOW_FULLSCREEN (changes the display mode)
    Exclusive,
}

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub use events::Event;

/// One presented frame: the RGB output surface (3 bytes per pixel).
#[derive(Clone)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
    pub rgb: Vec<u8>,
    pub number: u64,
}

impl Frame {
    pub fn to_rgba(&self, out: &mut Vec<u8>) {
        out.clear();
        out.reserve(self.width * self.height * 4);
        for c in self.rgb.chunks_exact(3) {
            out.extend_from_slice(&[c[0], c[1], c[2], 255]);
        }
    }
}

/// Latest presented frame, shared with the front-end. Only the newest frame matters.
pub type FrameSlot = Arc<Mutex<Option<Frame>>>;

struct ScriptedEvent {
    frame: u64,
    events: Vec<Event>,
}

pub struct TestHooks {
    pub fixed_step: bool,
    screenshot_frames: Vec<u64>,
    screenshot_dir: PathBuf,
    script: VecDeque<ScriptedEvent>,
    max_frames: Option<u64>,
    /// `DIABLO_TIME`: what `time(nullptr)` returns at start (seconds), for reproducible runs.
    pub fixed_time: Option<i64>,
}

impl TestHooks {
    pub fn from_env() -> TestHooks {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        let screenshot_frames = var("DIABLO_SCREENSHOT_FRAMES")
            .map(|v| v.split(',').filter_map(|s| s.trim().parse().ok()).collect())
            .unwrap_or_default();
        let script = var("DIABLO_INPUT_SCRIPT").map(|p| parse_script(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("DIABLO_INPUT_SCRIPT {p}: {e}")))).unwrap_or_default();
        TestHooks {
            fixed_step: var("DIABLO_FIXED_STEP").is_some_and(|v| v != "0"),
            screenshot_frames,
            screenshot_dir: var("DIABLO_SCREENSHOT_DIR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("screenshots")),
            script,
            max_frames: var("DIABLO_MAX_FRAMES").and_then(|v| v.parse().ok()),
            fixed_time: var("DIABLO_TIME").and_then(|v| v.parse().ok()),
        }
    }

    pub fn none() -> TestHooks {
        TestHooks { fixed_step: false, screenshot_frames: vec![], screenshot_dir: PathBuf::new(), script: VecDeque::new(), max_frames: None, fixed_time: None }
    }
}

fn parse_script(text: &str) -> VecDeque<ScriptedEvent> {
    let mut out: Vec<ScriptedEvent> = Vec::new();
    for (n, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut it = line.splitn(3, ' ');
        let frame: u64 = it.next().unwrap().parse().unwrap_or_else(|_| panic!("input script line {}: bad frame", n + 1));
        let cmd = it.next().unwrap_or("");
        let rest = it.next().unwrap_or("");
        let nums = || rest.split_whitespace().map(|s| s.parse::<i32>().unwrap()).collect::<Vec<_>>();
        let evs = match cmd {
            "key" => {
                let k = nums()[0];
                vec![Event::KeyDown { key: k, mods: 0 }, Event::KeyUp { key: k, mods: 0 }]
            }
            "click" => {
                let v = nums();
                vec![
                    Event::MouseMotion { x: v[0], y: v[1] },
                    Event::MouseButtonDown { button: events::BUTTON_LEFT, x: v[0], y: v[1], clicks: 1 },
                    Event::MouseButtonUp { button: events::BUTTON_LEFT, x: v[0], y: v[1], clicks: 1 },
                ]
            }
            "move" => {
                let v = nums();
                vec![Event::MouseMotion { x: v[0], y: v[1] }]
            }
            "text" => vec![Event::TextInput(rest.to_string())],
            "warp" => vec![Event::TestWarp(nums()[0])],
            "store" => vec![Event::TestStore(nums()[0])],
            "setwarp" => {
                let v = nums();
                vec![Event::TestSetWarp(v[0], v[1])]
            }
            "quit" => vec![Event::Quit],
            _ => panic!("input script line {}: unknown command {cmd}", n + 1),
        };
        out.push(ScriptedEvent { frame, events: evs });
    }
    out.sort_by_key(|e| e.frame);
    out.into()
}

/// The SDL-like services the game thread uses.
pub struct Platform {
    start: Instant,
    virtual_ms: u64,
    events_rx: Option<Receiver<Event>>,
    pending: VecDeque<Event>,
    frame_slot: FrameSlot,
    frames_presented: u64,
    hooks: TestHooks,
    /// Set by the front-end when the window is shown; headless runs have no window.
    pub headless: bool,
    exit_tx: Option<Sender<()>>,
    commands_tx: Option<Sender<FrontCommand>>,
    /// Window size in logical pixels as last created/resized (`SDL_GetWindowSize`).
    pub window_size: (i32, i32),
    pub window_created: bool,
    pub fullscreen: Fullscreen,
    mod_state: u16,
    keys_down: std::collections::HashSet<i32>,
    mouse_position: (i32, i32),
    mouse_buttons: u32,
    /// time, button, position, click count of the last button press
    last_click: (u32, u8, (i32, i32), u8),
    /// `SDL_ShowCursor(SDL_QUERY)`
    cursor_visible: bool,
    /// `SDL_GetKeyboardFocus() == ghMainWnd` (headless runs count as focused)
    keyboard_focus: bool,
    pub window_info: WindowInfoSlot,
    /// `SDL_IsTextInputActive`
    text_input_active: bool,
    /// `SDL_SetTextInputRect`
    pub text_input_rect: (i32, i32, i32, i32),
}

impl Platform {
    pub fn new(
        events_rx: Option<Receiver<Event>>,
        frame_slot: FrameSlot,
        hooks: TestHooks,
        headless: bool,
        exit_tx: Option<Sender<()>>,
        commands_tx: Option<Sender<FrontCommand>>,
    ) -> Platform {
        Platform {
            commands_tx,
            window_size: (0, 0),
            window_created: false,
            fullscreen: Fullscreen::Windowed,
            mod_state: 0,
            keys_down: Default::default(),
            mouse_position: (0, 0),
            mouse_buttons: 0,
            last_click: (0, 0, (0, 0), 0),
            cursor_visible: true,
            keyboard_focus: true,
            text_input_active: false,
            text_input_rect: (0, 0, 0, 0),
            window_info: Arc::new(Mutex::new(WindowInfo { render_scale: 1.0, size: (0, 0) })),
            start: Instant::now(),
            virtual_ms: 0,
            events_rx,
            pending: VecDeque::new(),
            frame_slot,
            frames_presented: 0,
            hooks,
            headless,
            exit_tx,
        }
    }

    /// `SDL_StartTextInput`: typed characters arrive as text events.
    pub fn start_text_input(&mut self) {
        self.text_input_active = true;
    }

    /// `SDL_StopTextInput`
    pub fn stop_text_input(&mut self) {
        self.text_input_active = false;
    }

    /// `SDL_IsTextInputActive`
    pub fn is_text_input_active(&self) -> bool {
        self.text_input_active
    }

    pub fn headless_from_env() -> Platform {
        Platform::new(None, Arc::new(Mutex::new(None)), TestHooks::from_env(), true, None, None)
    }

    /// `SDL_GetTicks`: milliseconds since start (virtual under `DIABLO_FIXED_STEP`).
    /// `time(nullptr)`: wall-clock seconds, or `DIABLO_TIME` plus elapsed time when set.
    pub fn time(&self) -> i64 {
        match self.hooks.fixed_time {
            Some(t) => t + self.ticks() as i64 / 1000,
            None => std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0),
        }
    }

    pub fn ticks(&self) -> u32 {
        if self.hooks.fixed_step { self.virtual_ms as u32 } else { self.start.elapsed().as_millis() as u32 }
    }

    /// `SDL_Delay`
    pub fn delay(&mut self, ms: u32) {
        if self.hooks.fixed_step {
            self.virtual_ms += ms as u64;
        } else {
            std::thread::sleep(Duration::from_millis(ms as u64));
        }
    }

    /// `SDL_PollEvent`. Also keeps SDL's keyboard/mouse state (`SDL_GetModState`,
    /// `SDL_GetKeyboardState`, `SDL_GetMouseState`) and SDL's double-click counting
    /// (500 ms, 32 px, as SDL2's defaults on Windows).
    /// `SDL_PushEvent`: queues an event after the ones already received.
    pub fn push_event(&mut self, e: Event) {
        if let Some(rx) = &self.events_rx {
            while let Ok(e) = rx.try_recv() {
                self.pending.push_back(e);
            }
        }
        self.pending.push_back(e);
    }

    pub fn poll_event(&mut self) -> Option<Event> {
        if let Some(rx) = &self.events_rx {
            while let Ok(e) = rx.try_recv() {
                self.pending.push_back(e);
            }
        }
        let mut e = self.pending.pop_front()?;
        let now = self.ticks();
        match &mut e {
            Event::KeyDown { key, mods } => {
                self.mod_state = *mods;
                self.keys_down.insert(*key);
            }
            Event::KeyUp { key, mods } => {
                self.mod_state = *mods;
                self.keys_down.remove(key);
            }
            Event::MouseMotion { x, y } => self.mouse_position = (*x, *y),
            Event::MouseButtonDown { button, x, y, clicks } => {
                let (last_t, last_b, last_pos, last_clicks) = self.last_click;
                let near = (last_pos.0 - *x).abs() <= 32 && (last_pos.1 - *y).abs() <= 32;
                *clicks = if last_b == *button && near && now.wrapping_sub(last_t) <= 500 { last_clicks.saturating_add(1) } else { 1 };
                self.last_click = (now, *button, (*x, *y), *clicks);
                self.mouse_buttons |= 1 << *button;
                self.mouse_position = (*x, *y);
            }
            Event::MouseButtonUp { button, x, y, clicks } => {
                *clicks = if self.last_click.1 == *button { self.last_click.3 } else { 1 };
                self.mouse_buttons &= !(1 << *button);
                self.mouse_position = (*x, *y);
            }
            Event::FocusGained => self.keyboard_focus = true,
            Event::FocusLost => self.keyboard_focus = false,
            _ => {}
        }
        Some(e)
    }

    /// Whether the game window has keyboard focus.
    pub fn has_keyboard_focus(&self) -> bool {
        self.keyboard_focus
    }

    /// `SDL_GetModState`
    pub fn mod_state(&self) -> u16 {
        self.mod_state
    }

    /// `SDL_GetKeyboardState()[...]` for a key code.
    pub fn is_key_down(&self, key: i32) -> bool {
        self.keys_down.contains(&key)
    }

    /// `SDL_GetMouseState`: position and button mask (bit = 1 << button).
    pub fn mouse_state(&self) -> ((i32, i32), u32) {
        (self.mouse_position, self.mouse_buttons)
    }

    /// `SDL_WarpMouseInWindow`: moves the OS cursor (the front-end maps game coordinates to the
    /// window, `LogicalToOutput`), which reports a motion event back as SDL does. Without a window
    /// the motion event is generated here.
    pub fn warp_mouse(&mut self, x: i32, y: i32) {
        self.mouse_position = (x, y);
        match &self.commands_tx {
            Some(tx) if self.window_created => {
                let _ = tx.send(FrontCommand::WarpMouse(x, y));
            }
            _ => self.pending.push_back(Event::MouseMotion { x, y }),
        }
    }

    pub fn frames_presented(&self) -> u64 {
        self.frames_presented
    }

    /// `RenderPresent`: hands the frame to the front-end, runs screenshot/script hooks.
    pub fn present(&mut self, width: usize, height: usize, rgb: &[u8]) {
        self.frames_presented += 1;
        let n = self.frames_presented;
        if self.hooks.screenshot_frames.contains(&n) || !self.headless {
            let frame = Frame { width, height, rgb: rgb.to_vec(), number: n };
            if self.hooks.screenshot_frames.contains(&n) {
                let _ = std::fs::create_dir_all(&self.hooks.screenshot_dir);
                let path = self.hooks.screenshot_dir.join(format!("frame_{n}.png"));
                let mut rgba = Vec::new();
                frame.to_rgba(&mut rgba);
                png::write_rgba(&path, width as u32, height as u32, &rgba).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                log::info!("screenshot {}", path.display());
            }
            if !self.headless {
                *self.frame_slot.lock().unwrap() = Some(frame);
            }
        }
        while self.hooks.script.front().is_some_and(|e| e.frame <= n) {
            let e = self.hooks.script.pop_front().unwrap();
            self.pending.extend(e.events);
        }
        if self.hooks.max_frames.is_some_and(|m| n >= m) {
            self.pending.push_back(Event::Quit);
        }
    }

    fn send(&self, c: FrontCommand) {
        if let Some(tx) = &self.commands_tx {
            let _ = tx.send(c);
        }
    }

    /// `SDL_CreateWindow`
    pub fn create_window(&mut self, title: &str, width: i32, height: i32, fullscreen: Fullscreen, resizable: bool) -> bool {
        self.window_size = (width, height);
        self.window_created = true;
        self.fullscreen = fullscreen;
        self.send(FrontCommand::CreateWindow { title: title.to_string(), width, height, fullscreen, resizable });
        true
    }

    /// `SDL_SetWindowFullscreen`
    pub fn set_window_fullscreen(&mut self, fullscreen: Fullscreen) {
        self.fullscreen = fullscreen;
        self.send(FrontCommand::SetFullscreen(fullscreen));
    }

    /// `SDL_SetWindowSize`
    pub fn set_window_size(&mut self, width: i32, height: i32) {
        self.window_size = (width, height);
        self.send(FrontCommand::SetWindowSize(width, height));
    }

    /// `SDL_SetWindowResizable`
    pub fn set_window_resizable(&mut self, resizable: bool) {
        self.send(FrontCommand::SetResizable(resizable));
    }

    /// `SDL_SetWindowGrab`
    pub fn set_window_grab(&mut self, grab: bool) {
        self.send(FrontCommand::SetGrab(grab));
    }

    /// `SDL_ShowCursor`
    pub fn show_cursor(&mut self, show: bool) {
        self.cursor_visible = show;
        self.send(FrontCommand::ShowCursor(show));
    }

    /// `SDL_ShowCursor(SDL_QUERY) == SDL_ENABLE`
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// `SDL_CreateColorCursor` + `SDL_SetCursor`
    pub fn set_cursor_image(&mut self, rgba: Vec<u8>, width: u32, height: u32, hotspot: (u16, u16)) -> bool {
        self.send(FrontCommand::SetCursorImage { rgba, width, height, hotspot });
        true
    }

    /// `SDL_RenderGetScale`: the scale from the game image to the window.
    pub fn render_scale(&self) -> f32 {
        self.window_info.lock().unwrap().render_scale
    }

    /// Renderer settings: integer scaling and SDL_HINT_RENDER_SCALE_QUALITY.
    pub fn set_presentation(&mut self, integer_scale: bool, linear_filter: bool) {
        self.send(FrontCommand::SetPresentation { integer_scale, linear_filter });
    }

    /// `SDL_HideWindow`
    pub fn hide_window(&mut self) {
        self.send(FrontCommand::Hide);
    }

    /// Blocking until the next display refresh, as SDL_RenderPresent does with vsync on.
    /// `refresh_delay_us` is the refresh period in microseconds.
    pub fn wait_for_vsync(&mut self, refresh_delay_us: i32) {
        let period = (refresh_delay_us.max(1000) / 1000) as u64;
        if self.hooks.fixed_step {
            self.virtual_ms += period;
            return;
        }
        let now = self.start.elapsed().as_millis() as u64;
        let next = (now / period + 1) * period;
        std::thread::sleep(Duration::from_millis(next - now));
    }

    /// Tells the front-end the game has finished (the window closes).
    pub fn request_exit(&self) {
        if let Some(tx) = &self.exit_tx {
            let _ = tx.send(());
        }
    }
}

/// Minimal logging facade (the original logs through SDL_Log).
pub mod log {
    macro_rules! info {
        ($($t:tt)*) => { eprintln!("INFO: {}", format!($($t)*)) };
    }
    macro_rules! error {
        ($($t:tt)*) => { eprintln!("ERROR: {}", format!($($t)*)) };
    }
    macro_rules! verbose {
        ($($t:tt)*) => { if std::env::var_os("DIABLO_VERBOSE").is_some() { eprintln!("VERBOSE: {}", format!($($t)*)) } };
    }
    pub(crate) use {error, info, verbose};
}
