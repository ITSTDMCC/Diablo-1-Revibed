//! Bevy front-end: shows the game's presented frames in a window and forwards input to the game
//! thread as SDL-style events. Replaces DevilutionX's SDL window/renderer/texture code
//! (`utils/display.cpp`, `engine/dx.cpp` RenderPresent): the 8-bit frame is expanded through its
//! palette into an RGBA texture and drawn scaled to the window with the aspect ratio kept
//! (letterbox/pillarbox), nearest-neighbour, like SDL_RenderSetLogicalSize.

use std::sync::mpsc::{Receiver, Sender};
use std::sync::Mutex;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyCode, KeyboardInput};
use bevy::input::mouse::{MouseButton, MouseButtonInput, MouseWheel};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::window::{
    CursorGrabMode, CursorLeft, CursorMoved, CursorOptions, ExitCondition, MonitorSelection, VideoModeSelection, WindowCloseRequested, WindowFocused, WindowMode, WindowOccluded, WindowResized,
    WindowResolution,
};

use super::events::{self, keys::*, Event};
use super::{FrameSlot, FrontCommand, Fullscreen, WindowInfoSlot};
use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage};

#[derive(Resource)]
struct Bridge {
    frames: FrameSlot,
    events: Sender<Event>,
    exit: Mutex<Receiver<()>>,
    commands: Mutex<Receiver<FrontCommand>>,
    window_info: WindowInfoSlot,
    image: Handle<Image>,
    integer_scale: bool,
    size: (u32, u32),
    last_frame: u64,
    mods: u16,
    rgba: Vec<u8>,
}

#[derive(Component)]
struct Screen;

/// Runs the window on the current (main) thread until the game thread asks to exit. The
/// window starts hidden and is shown when the game creates it (`SpawnWindow`).
pub fn run(frames: FrameSlot, events: Sender<Event>, exit: Receiver<()>, commands: Receiver<FrontCommand>, window_info: WindowInfoSlot) {
    let size = (640u32, 480u32);
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "DevilutionX".to_string(),
                    resolution: WindowResolution::new(size.0, size.1),
                    visible: false,
                    ..default()
                }),
                exit_condition: ExitCondition::DontExit,
                // closing goes through the game's own quit path (SDL_QUIT)
                close_when_requested: false,
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    );
    app.insert_resource(Bridge {
        frames,
        events,
        exit: Mutex::new(exit),
        commands: Mutex::new(commands),
        window_info,
        image: Handle::default(),
        integer_scale: false,
        size,
        last_frame: 0,
        mods: 0,
        rgba: Vec::new(),
    });
    app.add_systems(Startup, setup);
    app.add_systems(Update, (apply_commands, forward_input, show_frame, check_exit).chain());
    #[cfg(feature = "gamepad")]
    {
        app.init_resource::<super::bevy_gamepad::PadIds>();
        app.add_systems(Update, forward_gamepads.after(forward_input));
    }
    app.run();
}

fn new_image(w: u32, h: u32) -> Image {
    let mut image = Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        vec![0; (w * h * 4) as usize],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>, mut bridge: ResMut<Bridge>) {
    commands.spawn(Camera2d);
    let handle = images.add(new_image(bridge.size.0, bridge.size.1));
    bridge.image = handle.clone();
    commands.spawn((Sprite::from_image(handle), Screen));
}

fn mode_of(f: Fullscreen) -> WindowMode {
    match f {
        Fullscreen::Windowed => WindowMode::Windowed,
        Fullscreen::Desktop => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        Fullscreen::Exclusive => WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current),
    }
}

/// Applies the game's window requests (SDL window calls).
fn apply_commands(
    mut commands: Commands,
    mut bridge: ResMut<Bridge>,
    mut windows: Query<(Entity, &mut Window, &mut CursorOptions)>,
    mut images: ResMut<Assets<Image>>,
) {
    let Ok((window_entity, mut window, mut cursor)) = windows.single_mut() else { return };
    let cmds: Vec<FrontCommand> = bridge.commands.lock().unwrap().try_iter().collect();
    for c in cmds {
        match c {
            FrontCommand::CreateWindow { title, width, height, fullscreen, resizable } => {
                window.title = title;
                window.resolution = WindowResolution::new(width as u32, height as u32);
                window.mode = mode_of(fullscreen);
                window.resizable = resizable;
                window.visible = true;
            }
            FrontCommand::SetFullscreen(f) => window.mode = mode_of(f),
            FrontCommand::SetWindowSize(w, h) => window.resolution.set(w as f32, h as f32),
            FrontCommand::SetResizable(r) => window.resizable = r,
            FrontCommand::SetGrab(g) => cursor.grab_mode = if g { CursorGrabMode::Confined } else { CursorGrabMode::None },
            FrontCommand::ShowCursor(v) => cursor.visible = v,
            FrontCommand::SetPresentation { integer_scale, linear_filter } => {
                bridge.integer_scale = integer_scale;
                if let Some(mut img) = images.get_mut(&bridge.image) {
                    img.sampler = if linear_filter { ImageSampler::linear() } else { ImageSampler::nearest() };
                }
            }
            FrontCommand::Hide => window.visible = false,
            FrontCommand::WarpMouse(x, y) => {
                let (scale, ox, oy) = letterbox(&window, bridge.size, bridge.integer_scale);
                window.set_cursor_position(Some(Vec2::new(x as f32 * scale + ox, y as f32 * scale + oy)));
            }
            FrontCommand::SetCursorImage { rgba, width, height, hotspot } => {
                let mut img = Image::new(
                    Extent3d { width, height, depth_or_array_layers: 1 },
                    TextureDimension::D2,
                    rgba,
                    TextureFormat::Rgba8UnormSrgb,
                    RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
                );
                img.sampler = ImageSampler::nearest();
                let handle = images.add(img);
                commands.entity(window_entity).insert(CursorIcon::Custom(CustomCursor::Image(CustomCursorImage {
                    handle,
                    hotspot,
                    ..default()
                })));
            }
        }
    }
}

/// Window rect (logical pixels, origin top-left) the game image occupies:
/// SDL_RenderSetLogicalSize (aspect kept, letterboxed), optionally integer-scaled.
fn letterbox(window: &Window, size: (u32, u32), integer: bool) -> (f32, f32, f32) {
    let mut scale = (window.width() / size.0 as f32).min(window.height() / size.1 as f32);
    if integer && scale >= 1.0 {
        scale = scale.floor();
    }
    let ox = (window.width() - size.0 as f32 * scale) / 2.0;
    let oy = (window.height() - size.1 as f32 * scale) / 2.0;
    (scale, ox, oy)
}

fn show_frame(
    mut bridge: ResMut<Bridge>,
    mut images: ResMut<Assets<Image>>,
    mut sprites: Query<&mut Sprite, With<Screen>>,
    windows: Query<&Window>,
) {
    let frame = bridge.frames.lock().unwrap().take();
    if let Some(frame) = frame {
        if frame.number != bridge.last_frame {
            bridge.last_frame = frame.number;
            let (w, h) = (frame.width as u32, frame.height as u32);
            let mut rgba = std::mem::take(&mut bridge.rgba);
            frame.to_rgba(&mut rgba);
            if (w, h) != bridge.size {
                bridge.size = (w, h);
                let handle = bridge.image.clone();
                images.insert(&handle, new_image(w, h)).ok();
            }
            if let Some(mut img) = images.get_mut(&bridge.image) {
                if let Some(data) = img.data.as_mut() {
                    data.copy_from_slice(&rgba);
                }
            }
            bridge.rgba = rgba;
        }
    }
    if let (Ok(window), Ok(mut sprite)) = (windows.single(), sprites.single_mut()) {
        let (scale, _, _) = letterbox(window, bridge.size, bridge.integer_scale);
        if let Ok(mut info) = bridge.window_info.lock() {
            info.render_scale = scale;
            info.size = (window.width() as i32, window.height() as i32);
        }
        sprite.custom_size = Some(Vec2::new(bridge.size.0 as f32 * scale, bridge.size.1 as f32 * scale));
    }
}

fn check_exit(bridge: Res<Bridge>, mut exit: MessageWriter<AppExit>) {
    if bridge.exit.lock().unwrap().try_recv().is_ok() {
        exit.write(AppExit::Success);
    }
}

fn mod_bit(key: i32) -> u16 {
    match key {
        SDLK_LSHIFT => events::KMOD_LSHIFT,
        SDLK_RSHIFT => events::KMOD_RSHIFT,
        SDLK_LCTRL => events::KMOD_LCTRL,
        SDLK_RCTRL => events::KMOD_RCTRL,
        SDLK_LALT => events::KMOD_LALT,
        SDLK_RALT => events::KMOD_RALT,
        _ => 0,
    }
}

#[allow(clippy::too_many_arguments)]
fn forward_input(
    mut bridge: ResMut<Bridge>,
    windows: Query<&Window>,
    mut keyboard: MessageReader<KeyboardInput>,
    mut buttons: MessageReader<MouseButtonInput>,
    mut moved: MessageReader<CursorMoved>,
    mut wheel: MessageReader<MouseWheel>,
    mut focus: MessageReader<WindowFocused>,
    mut close: MessageReader<WindowCloseRequested>,
    mut left: MessageReader<CursorLeft>,
    mut occluded: MessageReader<WindowOccluded>,
    mut resized: MessageReader<WindowResized>,
) {
    let Ok(window) = windows.single() else { return };
    let (scale, ox, oy) = letterbox(window, bridge.size, bridge.integer_scale);
    let to_game = |p: Vec2| (((p.x - ox) / scale) as i32, ((p.y - oy) / scale) as i32);
    let mut out = Vec::new();
    for e in moved.read() {
        let (x, y) = to_game(e.position);
        out.push(Event::MouseMotion { x, y });
    }
    let cursor = window.cursor_position().map(to_game).unwrap_or((0, 0));
    for e in keyboard.read() {
        let key = sdl_keycode(&e.logical_key, e.key_code);
        let bit = mod_bit(key);
        match e.state {
            ButtonState::Pressed => {
                bridge.mods |= bit;
                out.push(Event::KeyDown { key, mods: bridge.mods });
                if let Some(t) = &e.text {
                    if t.chars().all(|c| !c.is_control()) {
                        out.push(Event::TextInput(t.to_string()));
                    }
                }
            }
            ButtonState::Released => {
                bridge.mods &= !bit;
                out.push(Event::KeyUp { key, mods: bridge.mods });
            }
        }
    }
    for e in buttons.read() {
        let button = match e.button {
            MouseButton::Left => events::BUTTON_LEFT,
            MouseButton::Middle => events::BUTTON_MIDDLE,
            MouseButton::Right => events::BUTTON_RIGHT,
            MouseButton::Back => events::BUTTON_X1,
            MouseButton::Forward => events::BUTTON_X2,
            MouseButton::Other(_) => continue,
        };
        let (x, y) = cursor;
        out.push(match e.state {
            ButtonState::Pressed => Event::MouseButtonDown { button, x, y, clicks: 1 },
            ButtonState::Released => Event::MouseButtonUp { button, x, y, clicks: 1 },
        });
    }
    for e in wheel.read() {
        out.push(Event::MouseWheel { x: e.x.signum() as i32, y: e.y.signum() as i32 });
    }
    for e in focus.read() {
        out.push(if e.focused { Event::FocusGained } else { Event::FocusLost });
    }
    for _ in close.read() {
        out.push(Event::Quit);
    }
    for _ in left.read() {
        out.push(Event::WindowLeave);
    }
    for e in occluded.read() {
        out.push(if e.occluded { Event::WindowHidden } else { Event::WindowShown });
    }
    for _ in resized.read() {
        out.push(Event::WindowSizeChanged);
    }
    for e in out {
        let _ = bridge.events.send(e);
    }
}

#[cfg(feature = "gamepad")]
fn forward_gamepads(bridge: Res<Bridge>, mut ids: ResMut<super::bevy_gamepad::PadIds>, mut raw: MessageReader<bevy::input::gamepad::RawGamepadEvent>) {
    for e in super::bevy_gamepad::translate(&mut ids, raw.read()) {
        let _ = bridge.events.send(e);
    }
}

/// SDL2 keycode for a key: characters use their lower-case ASCII value (layout-aware, like
/// SDL keycodes); other keys map from the physical key.
pub fn sdl_keycode(logical: &Key, code: KeyCode) -> i32 {
    if let Key::Character(s) = logical {
        let mut chars = s.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            if c.is_ascii() && !c.is_ascii_control() {
                return c.to_ascii_lowercase() as i32;
            }
        }
    }
    match code {
        KeyCode::Backspace => SDLK_BACKSPACE,
        KeyCode::Tab => SDLK_TAB,
        KeyCode::Enter => SDLK_RETURN,
        KeyCode::Escape => SDLK_ESCAPE,
        KeyCode::Space => SDLK_SPACE,
        KeyCode::Delete => SDLK_DELETE,
        KeyCode::CapsLock => SDLK_CAPSLOCK,
        KeyCode::F1 => SDLK_F1,
        KeyCode::F2 => SDLK_F1 + 1,
        KeyCode::F3 => SDLK_F1 + 2,
        KeyCode::F4 => SDLK_F1 + 3,
        KeyCode::F5 => SDLK_F1 + 4,
        KeyCode::F6 => SDLK_F1 + 5,
        KeyCode::F7 => SDLK_F1 + 6,
        KeyCode::F8 => SDLK_F1 + 7,
        KeyCode::F9 => SDLK_F1 + 8,
        KeyCode::F10 => SDLK_F1 + 9,
        KeyCode::F11 => SDLK_F1 + 10,
        KeyCode::F12 => SDLK_F12,
        KeyCode::PrintScreen => SDLK_PRINTSCREEN,
        KeyCode::ScrollLock => SDLK_SCROLLLOCK,
        KeyCode::Pause => SDLK_PAUSE,
        KeyCode::Insert => SDLK_INSERT,
        KeyCode::Home => SDLK_HOME,
        KeyCode::PageUp => SDLK_PAGEUP,
        KeyCode::End => SDLK_END,
        KeyCode::PageDown => SDLK_PAGEDOWN,
        KeyCode::ArrowRight => SDLK_RIGHT,
        KeyCode::ArrowLeft => SDLK_LEFT,
        KeyCode::ArrowDown => SDLK_DOWN,
        KeyCode::ArrowUp => SDLK_UP,
        KeyCode::NumLock => SDLK_NUMLOCKCLEAR,
        KeyCode::NumpadDivide => SDLK_KP_DIVIDE,
        KeyCode::NumpadMultiply => SDLK_KP_MULTIPLY,
        KeyCode::NumpadSubtract => SDLK_KP_MINUS,
        KeyCode::NumpadAdd => SDLK_KP_PLUS,
        KeyCode::NumpadEnter => SDLK_KP_ENTER,
        KeyCode::Numpad0 => SDLK_KP_0,
        KeyCode::Numpad1 => SDLK_KP_1,
        KeyCode::Numpad2 => SDLK_KP_1 + 1,
        KeyCode::Numpad3 => SDLK_KP_1 + 2,
        KeyCode::Numpad4 => SDLK_KP_1 + 3,
        KeyCode::Numpad5 => SDLK_KP_1 + 4,
        KeyCode::Numpad6 => SDLK_KP_1 + 5,
        KeyCode::Numpad7 => SDLK_KP_1 + 6,
        KeyCode::Numpad8 => SDLK_KP_1 + 7,
        KeyCode::Numpad9 => SDLK_KP_1 + 8,
        KeyCode::NumpadDecimal => SDLK_KP_PERIOD,
        KeyCode::ControlLeft => SDLK_LCTRL,
        KeyCode::ShiftLeft => SDLK_LSHIFT,
        KeyCode::AltLeft => SDLK_LALT,
        KeyCode::SuperLeft => SDLK_LGUI,
        KeyCode::ControlRight => SDLK_RCTRL,
        KeyCode::ShiftRight => SDLK_RSHIFT,
        KeyCode::AltRight => SDLK_RALT,
        KeyCode::SuperRight => SDLK_RGUI,
        _ => SDLK_UNKNOWN,
    }
}
