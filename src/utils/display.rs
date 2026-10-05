//! `Source/utils/display.cpp`: window, game resolution and renderer set-up.
//!
//! SDL window/renderer calls become platform front-end commands. "renderer != nullptr" (the
//! upscaling renderer) is `ctx.dx.has_renderer`.

use crate::ctx::Ctx;
use crate::engine::surface::Rect;
use crate::platform::Fullscreen;
use crate::platform::win32;

/// Original: `devilution::GetScreenWidth` (utils/display.cpp).
// @port utils/display.cpp|devilution::GetScreenWidth() sha=f70adf252ff7
pub fn get_screen_width(ctx: &Ctx) -> i32 {
    ctx.dx.gn_screen_width
}

/// Original: `devilution::GetScreenHeight` (utils/display.cpp).
// @port utils/display.cpp|devilution::GetScreenHeight() sha=7690cb8d082d
pub fn get_screen_height(ctx: &Ctx) -> i32 {
    ctx.dx.gn_screen_height
}

/// Original: `devilution::GetViewportHeight` (utils/display.cpp).
// @port utils/display.cpp|devilution::GetViewportHeight() sha=8ef20d93e414
pub fn get_viewport_height(ctx: &Ctx) -> i32 {
    ctx.dx.gn_viewport_height
}

/// Original: `devilution::GetUIRectangle` (utils/display.cpp).
// @port utils/display.cpp|devilution::GetUIRectangle() sha=ba10a55b9cd1
pub fn get_ui_rectangle(ctx: &Ctx) -> Rect {
    ctx.dx.ui_rectangle
}

/// Original: `CalculatePreferredWindowSize` (utils/display.cpp).
// @port utils/display.cpp|devilution::CalculatePreferredWindowSize(int &width, int &height) sha=52440fded129
fn calculate_preferred_window_size(ctx: &Ctx, width: &mut i32, height: &mut i32) {
    let mode = win32::desktop_display_mode();
    let (mut mw, mut mh) = (mode.w, mode.h);
    if mw < mh {
        std::mem::swap(&mut mw, &mut mh);
    }
    if ctx.options.graphics.integer_scaling.get() {
        let factor = (mw / *width).min(mh / *height);
        *width = mw / factor;
        *height = mh / factor;
        return;
    }
    let w_factor = mw as f32 / *width as f32;
    let h_factor = mh as f32 / *height as f32;
    if w_factor > h_factor {
        *width = mw * *height / mh;
    } else {
        *height = mh * *width / mw;
    }
}

/// Original: `GetNearestDisplayMode` (utils/display.cpp).
// @port utils/display.cpp|devilution::GetNearestDisplayMode(Size preferredSize) sha=7a20571faff7
fn get_nearest_display_mode(preferred: (i32, i32)) -> win32::DisplayMode {
    let mut nearest = win32::desktop_display_mode();
    for m in win32::display_modes() {
        let diff_height = (nearest.h - preferred.1).abs() - (m.h - preferred.1).abs();
        let diff_width = (nearest.w - preferred.0).abs() - (m.w - preferred.0).abs();
        if diff_height < 0 {
            continue;
        }
        if diff_height == 0 && diff_width < 0 {
            continue;
        }
        nearest = m;
    }
    nearest
}

/// Original: `CalculateUIRectangle` (utils/display.cpp).
// @port utils/display.cpp|devilution::CalculateUIRectangle() sha=2a0cabc93030
fn calculate_ui_rectangle(ctx: &mut Ctx) {
    let (w, h) = (640, 480);
    ctx.dx.ui_rectangle = Rect::new((ctx.dx.gn_screen_width - w) / 2, (ctx.dx.gn_screen_height - h) / 2, w, h);
}

/// Original: `GetPreferredWindowSize` (utils/display.cpp).
// @port utils/display.cpp|devilution::GetPreferredWindowSize() sha=2e1f1f5608f1
fn get_preferred_window_size(ctx: &mut Ctx) -> (i32, i32) {
    let (mut w, mut h) = if ctx.dx.force_resolution.0 != 0 { ctx.dx.force_resolution } else { ctx.options.graphics.resolution.get() };
    if ctx.options.graphics.upscale.get() && ctx.options.graphics.fit_to_screen.get() {
        if follows_window(ctx) && ctx.options.graphics.fullscreen.get() && !ctx.platform.headless {
            // free-movement build: full screen at the screen's own resolution
            let m = win32::desktop_display_mode();
            (w, h) = (m.w.max(m.h), m.w.min(m.h));
        } else {
            calculate_preferred_window_size(ctx, &mut w, &mut h);
        }
    }
    adjust_to_screen_geometry(ctx, (w, h));
    (w, h)
}

/// The game's resolution follows the window's size (free-movement build, with Upscale and Fit
/// to Screen on; not in the original, which keeps the resolution option and scales the picture).
fn follows_window(ctx: &Ctx) -> bool {
    crate::freemove::ENABLED && ctx.options.graphics.upscale.get() && ctx.options.graphics.fit_to_screen.get()
}

/// Called before drawing a frame: when the window's size changed, the game's resolution changes
/// with it, so a full-screen or resized window shows more of the world instead of a stretched
/// picture (free-movement build).
pub fn follow_window_size(ctx: &mut Ctx) {
    if !follows_window(ctx) || !ctx.platform.window_created || !ctx.dx.has_renderer {
        return;
    }
    let (w, h) = ctx.platform.current_window_size();
    // the game's screens need at least 640 x 480
    let (w, h) = (w.max(640), h.max(480));
    if (w, h) == (ctx.dx.gn_screen_width, ctx.dx.gn_screen_height) || ctx.platform.current_window_size().0 <= 0 {
        return;
    }
    adjust_to_screen_geometry(ctx, (w, h));
    ctx.dx.output = vec![0; (w * h * 3) as usize];
    crate::engine::dx::create_back_buffer(ctx);
    crate::engine::render::scrollrt::calc_viewport_geometry(ctx);
    crate::controls::touch::initialize_virtual_gamepad(ctx);
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

/// Original: `devilution::AdjustToScreenGeometry` (utils/display.cpp).
// @port utils/display.cpp|devilution::AdjustToScreenGeometry(Size windowSize) sha=a60e83ff462b
pub fn adjust_to_screen_geometry(ctx: &mut Ctx, window_size: (i32, i32)) {
    ctx.dx.gn_screen_width = window_size.0;
    ctx.dx.gn_screen_height = window_size.1;
    calculate_ui_rectangle(ctx);
    crate::control::calculate_panel_areas(ctx);
}

/// Original: `devilution::GetDpiScalingFactor` (utils/display.cpp). The front-end draws in
/// logical pixels, so render output and window sizes match.
// @port utils/display.cpp|devilution::GetDpiScalingFactor() sha=71472f2157ba
pub fn get_dpi_scaling_factor(ctx: &Ctx) -> f32 {
    let _ = ctx;
    1.0
}

/// Original: `devilution::IsFullScreen` (utils/display.cpp).
// @port utils/display.cpp|devilution::IsFullScreen() sha=ca567f81d0f8
pub fn is_full_screen(ctx: &Ctx) -> bool {
    ctx.platform.fullscreen != Fullscreen::Windowed
}

/// Original: `devilution::SpawnWindow` (utils/display.cpp). SDL hints, joystick/controller
/// initialisation and custom event registration have no counterpart in the front-end.
// @port utils/display.cpp|devilution::SpawnWindow(const char *lpWindowName) sha=dd1ccbb88898
pub fn spawn_window(ctx: &mut Ctx, window_name: &str) -> bool {
    let window_size = get_preferred_window_size(ctx);
    let upscale = ctx.options.graphics.upscale.get();
    let fullscreen = ctx.options.graphics.fullscreen.get();
    let mode = if upscale {
        if fullscreen { Fullscreen::Desktop } else { Fullscreen::Windowed }
    } else if fullscreen {
        Fullscreen::Exclusive
    } else {
        Fullscreen::Windowed
    };
    let created = ctx.platform.create_window(window_name, window_size.0, window_size.1, mode, upscale);
    if created {
        let grab = ctx.options.gameplay.grab_input.get();
        ctx.platform.set_window_grab(grab);
    }
    let mut refresh_rate = 60;
    let m = win32::desktop_display_mode();
    if m.refresh_rate != 0 {
        refresh_rate = m.refresh_rate;
    }
    ctx.dx.refresh_delay = 1_000_000 / refresh_rate;
    reinitialize_renderer(ctx);
    created
}

/// Original: `devilution::ReinitializeTexture` (utils/display.cpp): the texture filter follows
/// the Scaling Quality option (nearest vs linear; anisotropic is linear for a 2D quad).
// @port utils/display.cpp|devilution::ReinitializeTexture() sha=b9b51a2cd957
pub fn reinitialize_texture(ctx: &mut Ctx) {
    if !ctx.dx.has_renderer {
        return;
    }
    let linear = ctx.options.graphics.scale_quality.get_raw() != crate::options::ScalingQuality::NearestPixel as i32;
    let integer = ctx.options.graphics.integer_scaling.get();
    ctx.platform.set_presentation(integer, linear);
}

/// Original: `devilution::ReinitializeIntegerScale` (utils/display.cpp).
// @port utils/display.cpp|devilution::ReinitializeIntegerScale() sha=f503dcea9bec
pub fn reinitialize_integer_scale(ctx: &mut Ctx) {
    if ctx.options.graphics.fit_to_screen.get() {
        resize_window(ctx);
        return;
    }
    if ctx.dx.has_renderer {
        reinitialize_texture(ctx);
    }
}

/// Original: `devilution::ReinitializeRenderer` (utils/display.cpp).
// @port utils/display.cpp|devilution::ReinitializeRenderer() sha=70daac9b557f
pub fn reinitialize_renderer(ctx: &mut Ctx) {
    if !ctx.platform.window_created {
        return;
    }
    ctx.dx.has_renderer = false;
    if ctx.options.graphics.upscale.get() {
        ctx.dx.has_renderer = true;
        reinitialize_texture(ctx);
        let (w, h) = (ctx.dx.gn_screen_width, ctx.dx.gn_screen_height);
        ctx.dx.output = vec![0; (w * h * 3) as usize];
    } else {
        let window_size = ctx.platform.window_size;
        adjust_to_screen_geometry(ctx, window_size);
    }
}

/// Original: `devilution::SetFullscreenMode` (utils/display.cpp).
// @port utils/display.cpp|devilution::SetFullscreenMode() sha=990327fe63e9
pub fn set_fullscreen_mode(ctx: &mut Ctx) {
    let fullscreen = ctx.options.graphics.fullscreen.get();
    let upscale = ctx.options.graphics.upscale.get();
    if fullscreen && !upscale {
        let window_size = get_preferred_window_size(ctx);
        let _ = get_nearest_display_mode(window_size);
    }
    let mode = if fullscreen {
        if upscale { Fullscreen::Desktop } else { Fullscreen::Exclusive }
    } else {
        Fullscreen::Windowed
    };
    ctx.platform.set_window_fullscreen(mode);
    if !fullscreen {
        let window_size = get_preferred_window_size(ctx);
        ctx.platform.set_window_size(window_size.0, window_size.1);
    }
    if !upscale {
        reinitialize_renderer(ctx);
        crate::engine::dx::create_back_buffer(ctx);
    }
    crate::controls::touch::initialize_virtual_gamepad(ctx);
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

/// Original: `devilution::ResizeWindow` (utils/display.cpp).
// @port utils/display.cpp|devilution::ResizeWindow() sha=600ca9894cc7
pub fn resize_window(ctx: &mut Ctx) {
    if !ctx.platform.window_created {
        return;
    }
    let window_size = get_preferred_window_size(ctx);
    let fullscreen = ctx.options.graphics.fullscreen.get();
    let upscale = ctx.options.graphics.upscale.get();
    let true_fullscreen = fullscreen && !upscale;
    if true_fullscreen {
        let _ = get_nearest_display_mode(window_size);
    }
    let upscale_changed = upscale != ctx.dx.has_renderer;
    if upscale_changed && fullscreen {
        ctx.platform.set_window_fullscreen(if upscale { Fullscreen::Desktop } else { Fullscreen::Exclusive });
    }
    if !true_fullscreen {
        ctx.platform.set_window_size(window_size.0, window_size.1);
    }
    reinitialize_renderer(ctx);
    let resizable = ctx.dx.has_renderer;
    ctx.platform.set_window_resizable(resizable);
    crate::controls::touch::initialize_virtual_gamepad(ctx);
    crate::engine::dx::create_back_buffer(ctx);
    crate::engine::backbuffer_state::redraw_everything(ctx);
}

/// Original: `devilution::OutputRequiresScaling` (utils/display.cpp): SDL2 scales in the renderer.
// @port utils/display.cpp|devilution::OutputRequiresScaling() sha=75665522d11b
pub fn output_requires_scaling() -> bool {
    false
}
