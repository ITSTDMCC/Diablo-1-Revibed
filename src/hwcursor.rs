//! `Source/hwcursor.cpp` / `hwcursor.hpp`: the operating-system ("hardware") mouse cursor
//! showing the game's cursor art. The front-end turns the RGBA image into a window cursor
//! (SDL_CreateColorCursor).

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSprite;
use crate::engine::surface::OwnedSurface;

/// `CursorType`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CursorType {
    #[default]
    Unknown,
    UserInterface,
    Game,
}

/// `CursorInfo`
#[derive(Clone, Copy, Debug, Default)]
pub struct CursorInfo {
    type_: CursorType,
    id: i32,
    enabled: bool,
    needs_reinitialization: bool,
}

impl PartialEq for CursorInfo {
    /// `operator==`: type and id.
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_ && self.id == other.id
    }
}

impl CursorInfo {
    /// Original: `CursorInfo::UserInterfaceCursor` (hwcursor.hpp).
    pub fn user_interface_cursor() -> CursorInfo {
        CursorInfo { type_: CursorType::UserInterface, ..Default::default() }
    }
    /// Original: `CursorInfo::GameCursor` (hwcursor.hpp).
    pub fn game_cursor(game_sprite_id: i32) -> CursorInfo {
        CursorInfo { type_: CursorType::Game, id: game_sprite_id, ..Default::default() }
    }
    /// Original: `CursorInfo::UnknownCursor` (hwcursor.hpp).
    pub fn unknown_cursor() -> CursorInfo {
        CursorInfo { type_: CursorType::Unknown, ..Default::default() }
    }
    pub fn cursor_type(&self) -> CursorType {
        self.type_
    }
    pub fn id(&self) -> i32 {
        self.id
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn set_enabled(&mut self, value: bool) {
        self.enabled = value;
    }
    pub fn needs_reinitialization(&self) -> bool {
        self.needs_reinitialization
    }
    pub fn set_needs_reinitialization(&mut self, value: bool) {
        self.needs_reinitialization = value;
    }
}

#[derive(Default)]
pub struct HwCursorState {
    /// `CurrentCursorInfo`
    current_cursor_info: CursorInfo,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Center is used by SetHardwareCursorFromSprite (pending)
enum HotpointPosition {
    TopLeft,
    Center,
}

/// Original: `ScaledSize` (hwcursor.cpp).
// @port hwcursor.cpp|devilution::ScaledSize(Size size) sha=5eb2a49199f1
fn scaled_size(ctx: &Ctx, size: (i32, i32)) -> (i32, i32) {
    if ctx.dx.has_renderer {
        let s = ctx.platform.render_scale();
        return ((size.0 as f32 * s) as i32, (size.1 as f32 * s) as i32);
    }
    size
}

/// Original: `IsCursorSizeAllowed` (hwcursor.cpp).
// @port hwcursor.cpp|devilution::IsCursorSizeAllowed(Size size) sha=1ed850143c76
fn is_cursor_size_allowed(ctx: &Ctx, size: (i32, i32)) -> bool {
    let max = ctx.options.graphics.hardware_cursor_max_size.get();
    if max <= 0 {
        return true;
    }
    let size = scaled_size(ctx, size);
    size.0 <= max && size.1 <= max
}

/// Original: `GetHotpointPosition` (hwcursor.cpp).
// @port hwcursor.cpp|devilution::GetHotpointPosition(const SDL_Surface &surface, HotpointPosition position) sha=b6e3ef3ccd74
fn get_hotpoint_position(w: i32, h: i32, position: HotpointPosition) -> (i32, i32) {
    match position {
        HotpointPosition::TopLeft => (0, 0),
        HotpointPosition::Center => (w / 2, h / 2),
    }
}

/// Original: `ShouldUseBilinearScaling` (hwcursor.cpp).
// @port hwcursor.cpp|devilution::ShouldUseBilinearScaling() sha=8f004b4b6b4d
fn should_use_bilinear_scaling(ctx: &Ctx) -> bool {
    ctx.options.graphics.scale_quality.get_raw() != crate::options::ScalingQuality::NearestPixel as i32
}

/// RGBA scaling for the cursor image: nearest neighbour (SDL_BlitScaled) or bilinear
/// (`BilinearScale32`, utils/sdl_bilinear_scale.cpp).
fn scale_rgba(src: &[u8], w: i32, h: i32, dw: i32, dh: i32, bilinear: bool) -> Vec<u8> {
    let mut out = vec![0u8; (dw * dh * 4) as usize];
    for y in 0..dh {
        for x in 0..dw {
            let o = ((y * dw + x) * 4) as usize;
            if !bilinear {
                let sx = (x * w / dw).min(w - 1);
                let sy = (y * h / dh).min(h - 1);
                let s = ((sy * w + sx) * 4) as usize;
                out[o..o + 4].copy_from_slice(&src[s..s + 4]);
                continue;
            }
            let fx = ((x as f32 + 0.5) * w as f32 / dw as f32 - 0.5).max(0.0);
            let fy = ((y as f32 + 0.5) * h as f32 / dh as f32 - 0.5).max(0.0);
            let (x0, y0) = (fx as i32, fy as i32);
            let (x1, y1) = ((x0 + 1).min(w - 1), (y0 + 1).min(h - 1));
            let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
            for c in 0..4 {
                let p = |xx: i32, yy: i32| src[((yy * w + xx) * 4) as usize + c] as f32;
                let top = p(x0, y0) * (1.0 - tx) + p(x1, y0) * tx;
                let bottom = p(x0, y1) * (1.0 - tx) + p(x1, y1) * tx;
                out[o + c] = (top * (1.0 - ty) + bottom * ty).round() as u8;
            }
        }
    }
    out
}

/// Original: `SetHardwareCursorFromSurface` (hwcursor.cpp): the 8-bit surface through the
/// current palette, colour key 0 (or the given index) transparent.
// @port hwcursor.cpp|devilution::SetHardwareCursorFromSurface(SDL_Surface *surface, HotpointPosition hotpointPosition) sha=b1df6b80e04a
fn set_hardware_cursor_from_surface(ctx: &mut Ctx, surface: &OwnedSurface, color_key: u8, hotpoint_position: HotpointPosition) -> bool {
    let pal = **ctx.dx.palette.as_ref().expect("Palette");
    let mut rgba = Vec::with_capacity((surface.w * surface.h * 4) as usize);
    for y in 0..surface.h {
        for x in 0..surface.w {
            let p = surface.pixels[(y * surface.pitch + x) as usize];
            let c = pal[p as usize];
            rgba.extend_from_slice(&[c[0], c[1], c[2], if p == color_key { 0 } else { 255 }]);
        }
    }
    let size = (surface.w, surface.h);
    let scaled = scaled_size(ctx, size);
    let (data, w, h) = if size == scaled {
        (rgba, size.0, size.1)
    } else {
        let bilinear = should_use_bilinear_scaling(ctx);
        (scale_rgba(&rgba, size.0, size.1, scaled.0, scaled.1, bilinear), scaled.0, scaled.1)
    };
    let hot = get_hotpoint_position(w, h, hotpoint_position);
    ctx.platform.set_cursor_image(data, w as u32, h as u32, (hot.0 as u16, hot.1 as u16))
}

/// Original: `SetHardwareCursorFromClxSprite` (hwcursor.cpp).
// @port hwcursor.cpp|devilution::SetHardwareCursorFromClxSprite(ClxSprite sprite, HotpointPosition hotpointPosition) sha=761ef5417016
fn set_hardware_cursor_from_clx_sprite(ctx: &mut Ctx, sprite: &ClxSprite, hotpoint_position: HotpointPosition) -> bool {
    let mut surface = OwnedSurface::new(sprite.width() as i32, sprite.height() as i32);
    let view = surface.view();
    crate::engine::render::text_render::render_clx_sprite(&view, sprite, (0, 0));
    set_hardware_cursor_from_surface(ctx, &surface, 0, hotpoint_position)
}

/// Original: `devilution::GetCurrentCursorInfo` (hwcursor.cpp).
// @port hwcursor.cpp|devilution::GetCurrentCursorInfo() sha=92d1374145fc
pub fn get_current_cursor_info(ctx: &mut Ctx) -> &mut CursorInfo {
    &mut ctx.hwcursor.current_cursor_info
}

/// Original: `devilution::IsHardwareCursorEnabled` (hwcursor.hpp).
// @port hwcursor.hpp|devilution::IsHardwareCursorEnabled() sha=668c047b0c60
pub fn is_hardware_cursor_enabled(ctx: &Ctx) -> bool {
    ctx.options.graphics.hardware_cursor.get() && crate::options::hardware_cursor_supported()
}

/// Original: `devilution::IsHardwareCursor` (hwcursor.hpp).
// @port hwcursor.hpp|devilution::IsHardwareCursor() sha=16e8b299bfb5
pub fn is_hardware_cursor(ctx: &Ctx) -> bool {
    ctx.hwcursor.current_cursor_info.enabled()
}

/// Original: `devilution::SetHardwareCursor` (hwcursor.cpp).
// @port hwcursor.cpp|devilution::SetHardwareCursor(CursorInfo cursorInfo) sha=2f47de8d20f7
pub fn set_hardware_cursor(ctx: &mut Ctx, cursor_info: CursorInfo) {
    ctx.hwcursor.current_cursor_info = cursor_info;
    ctx.hwcursor.current_cursor_info.set_needs_reinitialization(false);
    let enabled = match cursor_info.cursor_type() {
        CursorType::Game => crate::cursor::set_hardware_cursor_from_sprite(ctx, cursor_info.id()),
        CursorType::UserInterface => {
            // ArtCursor is null while loading the game on the progress screen.
            match ctx.diablo_ui.art_cursor.clone() {
                Some(art) => {
                    let s = art.get(0);
                    is_cursor_size_allowed(ctx, (s.width() as i32, s.height() as i32))
                        && set_hardware_cursor_from_clx_sprite(ctx, &s, HotpointPosition::TopLeft)
                }
                None => false,
            }
        }
        CursorType::Unknown => false,
    };
    ctx.hwcursor.current_cursor_info.set_enabled(enabled);
    if !ctx.hwcursor.current_cursor_info.enabled() {
        set_hardware_cursor_visible(ctx, false);
    }
}

/// Original: `devilution::DoReinitializeHardwareCursor` (hwcursor.hpp).
// @port hwcursor.hpp|devilution::DoReinitializeHardwareCursor() sha=91ddd064d2e6
pub fn do_reinitialize_hardware_cursor(ctx: &mut Ctx) {
    let info = ctx.hwcursor.current_cursor_info;
    set_hardware_cursor(ctx, info);
}

/// Original: `devilution::IsHardwareCursorVisible` (hwcursor.hpp).
// @port hwcursor.hpp|devilution::IsHardwareCursorVisible() sha=d82a2b1d1451
pub fn is_hardware_cursor_visible(ctx: &Ctx) -> bool {
    ctx.platform.is_cursor_visible()
}

/// Original: `devilution::SetHardwareCursorVisible` (hwcursor.hpp).
// @port hwcursor.hpp|devilution::SetHardwareCursorVisible(bool visible) sha=fd1ec9879f3c
pub fn set_hardware_cursor_visible(ctx: &mut Ctx, visible: bool) {
    if is_hardware_cursor_visible(ctx) == visible {
        return;
    }
    if visible && ctx.hwcursor.current_cursor_info.needs_reinitialization() {
        do_reinitialize_hardware_cursor(ctx);
    }
    ctx.platform.show_cursor(visible);
}

/// Original: `devilution::ReinitializeHardwareCursor` (hwcursor.hpp).
// @port hwcursor.hpp|devilution::ReinitializeHardwareCursor() sha=5bee9d406bb9
pub fn reinitialize_hardware_cursor(ctx: &mut Ctx) {
    if is_hardware_cursor_visible(ctx) {
        do_reinitialize_hardware_cursor(ctx);
    } else {
        ctx.hwcursor.current_cursor_info.set_needs_reinitialization(true);
    }
}
