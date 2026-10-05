//! `Source/engine/dx.cpp`: the 8-bit back buffer and presenting it.
//!
//! SDL's renderer/texture path (`RendererTextureSurface`, `SDL_UpdateTexture`,
//! `SDL_RenderPresent`) is replaced by the platform front-end: `blit` converts the 8-bit
//! surface through the current palette into the RGB output surface, and `render_present` hands
//! that to the front-end. With vsync the original blocks in SDL_RenderPresent until the next
//! refresh; the port waits for the next refresh interval the same way.

use crate::ctx::Ctx;
use crate::engine::surface::{OwnedSurface, Rect, Surface};

pub type Color = [u8; 3];

#[derive(Default)]
pub struct DxState {
    /// `refreshDelay` (microseconds per refresh)
    pub refresh_delay: i32,
    /// `Palette`: the SDL palette the back buffer is drawn with.
    pub palette: Option<Box<[Color; 256]>>,
    pub pal_surface_palette_version: u32,
    /// `RendererTextureSurface`: RGB output, `gnScreenWidth` x `gnScreenHeight`.
    pub output: Vec<u8>,
    /// `PalSurface` (owned here: `PinnedPalSurface`).
    pub pal_surface: Option<OwnedSurface>,
    /// `RenderDirectlyToOutputSurface`
    pub render_directly_to_output_surface: bool,
    /// `frameDeadline` (static in LimitFrameRate)
    frame_deadline: u32,
    /// `gnScreenWidth`, `gnScreenHeight`, `gnViewportHeight` (utils/display.cpp)
    pub gn_screen_width: i32,
    pub gn_screen_height: i32,
    pub gn_viewport_height: i32,
    /// `forceResolution`
    pub force_resolution: (i32, i32),
    /// `UIRectangle`
    pub ui_rectangle: Rect,
    /// renderer != nullptr (upscaling renderer created)
    pub has_renderer: bool,
    /// `system_palette`, `logical_palette`, `orig_palette`, lookup tables (engine/palette.cpp)
    pub pal: crate::engine::palette::PaletteState,
    /// `States` (engine/backbuffer_state.cpp)
    pub backbuffer: crate::engine::backbuffer_state::BackbufferStates,
}

/// Original: `LimitFrameRate` (engine/dx.cpp).
// @port engine/dx.cpp|devilution::LimitFrameRate() sha=f15bba4d5054
fn limit_frame_rate(ctx: &mut Ctx) {
    if !ctx.options.graphics.limit_fps.get() {
        return;
    }
    let refresh_delay = ctx.dx.refresh_delay.max(1) as u32;
    let tc = ctx.platform.ticks().wrapping_mul(1000);
    let mut v = 0;
    if ctx.dx.frame_deadline > tc {
        v = tc % refresh_delay;
        ctx.platform.delay(v / 1000 + 1); // ceil
    }
    ctx.dx.frame_deadline = tc.wrapping_add(v).wrapping_add(refresh_delay);
}

/// Original: `devilution::dx_init` (engine/dx.cpp).
// @port engine/dx.cpp|devilution::dx_init() sha=c14e44513bfb
pub fn dx_init(ctx: &mut Ctx) {
    // SDL_RaiseWindow / SDL_ShowWindow: the front-end shows the window when it is created.
    crate::engine::palette::palette_init(ctx);
    create_back_buffer(ctx);
    ctx.dx.pal_surface_palette_version = 1;
}

/// Original: `devilution::GlobalBackBuffer` (engine/dx.cpp).
// @port engine/dx.cpp|devilution::GlobalBackBuffer() sha=dcd027c150ad
pub fn global_back_buffer(ctx: &mut Ctx) -> Surface {
    let (w, h) = (ctx.dx.gn_screen_width, ctx.dx.gn_screen_height);
    match ctx.dx.pal_surface.as_mut() {
        Some(s) => Surface::of(s).subregion(0, 0, w, h),
        None => Surface::default(),
    }
}

/// Original: `devilution::dx_cleanup` (engine/dx.cpp).
// @port engine/dx.cpp|devilution::dx_cleanup() sha=5b318ed6f40e
pub fn dx_cleanup(ctx: &mut Ctx) {
    ctx.platform.hide_window();
    ctx.dx.pal_surface = None;
    ctx.dx.palette = None;
    ctx.dx.output = Vec::new();
    ctx.dx.has_renderer = false;
}

/// Original: `devilution::CreateBackBuffer` (engine/dx.cpp). The port never renders directly
/// to the output surface (SDL2 builds don't either).
// @port engine/dx.cpp|devilution::CreateBackBuffer() sha=b23b1f21b6d8
pub fn create_back_buffer(ctx: &mut Ctx) {
    ctx.dx.pal_surface = Some(OwnedSurface::new(ctx.dx.gn_screen_width, ctx.dx.gn_screen_height));
    ctx.dx.render_directly_to_output_surface = false;
    // In SDL2, `PalSurface` points to the global `palette` (shared, nothing to copy).
}

/// Original: `devilution::InitPalette` (engine/dx.cpp): SDL_AllocPalette initialises all
/// colours to white.
// @port engine/dx.cpp|devilution::InitPalette() sha=2f66293c7fec
pub fn init_palette(ctx: &mut Ctx) {
    ctx.dx.palette = Some(Box::new([[255; 3]; 256]));
}

/// Original: `devilution::BltFast` (engine/dx.cpp): the whole back buffer to the output.
// @port engine/dx.cpp|devilution::BltFast(SDL_Rect *srcRect, SDL_Rect *dstRect) sha=4563b66b5ed1
pub fn blt_fast(ctx: &mut Ctx, src_rect: Option<Rect>, dst_rect: Option<Rect>) {
    if ctx.dx.render_directly_to_output_surface {
        return;
    }
    blit(ctx, src_rect, dst_rect);
}

/// Original: `devilution::Blit` (engine/dx.cpp): `SDL_BlitSurface` from the 8-bit back buffer
/// to the RGB output surface (palette conversion), clipped to both.
// @port engine/dx.cpp|devilution::Blit(SDL_Surface *src, SDL_Rect *srcRect, SDL_Rect *dstRect) sha=346bfcf279c5
pub fn blit(ctx: &mut Ctx, src_rect: Option<Rect>, dst_rect: Option<Rect>) {
    if ctx.diablo.headless_mode {
        return;
    }
    let (w, h) = (ctx.dx.gn_screen_width, ctx.dx.gn_screen_height);
    if ctx.dx.output.len() != (w * h * 3) as usize {
        ctx.dx.output = vec![0; (w * h * 3) as usize];
    }
    let Some(src) = ctx.dx.pal_surface.as_ref() else { return };
    let Some(pal) = ctx.dx.palette.as_ref() else { return };
    let s = src_rect.unwrap_or(Rect::new(0, 0, src.w, src.h));
    let d = dst_rect.unwrap_or(Rect::new(0, 0, s.w, s.h));
    for y in 0..s.h.min(h - d.y).min(src.h - s.y) {
        for x in 0..s.w.min(w - d.x).min(src.w - s.x) {
            let c = pal[src.pixels[((s.y + y) * src.pitch + s.x + x) as usize] as usize];
            let o = (((d.y + y) * w + d.x + x) * 3) as usize;
            ctx.dx.output[o..o + 3].copy_from_slice(&c);
        }
    }
}

/// Original: `devilution::RenderPresent` (engine/dx.cpp).
// @port engine/dx.cpp|devilution::RenderPresent() sha=500bdac1f7c6
pub fn render_present(ctx: &mut Ctx) {
    crate::nthread::nthread_pump(ctx);
    if ctx.diablo.headless_mode {
        return;
    }
    if !ctx.init.gb_active {
        limit_frame_rate(ctx);
        return;
    }
    let (w, h) = (ctx.dx.gn_screen_width as usize, ctx.dx.gn_screen_height as usize);
    if ctx.dx.output.len() != w * h * 3 {
        ctx.dx.output = vec![0; w * h * 3];
    }
    if ctx.dx.has_renderer {
        // Clear + RenderCopy + RenderPresent
        let out = std::mem::take(&mut ctx.dx.output);
        ctx.platform.present(w, h, &out);
        ctx.dx.output = out;
        if ctx.controls.control_mode == crate::controls::ControlTypes::VirtualGamepad {
            crate::controls::touch::render_virtual_gamepad(ctx);
        }
        if ctx.options.graphics.v_sync.get() {
            ctx.platform.wait_for_vsync(ctx.dx.refresh_delay);
        } else {
            limit_frame_rate(ctx);
        }
    } else {
        let out = std::mem::take(&mut ctx.dx.output);
        ctx.platform.present(w, h, &out);
        ctx.dx.output = out;
        limit_frame_rate(ctx);
    }
}

/// Original: `devilution::PaletteGetEntries` (engine/dx.cpp).
// @port engine/dx.cpp|devilution::PaletteGetEntries(int dwNumEntries, SDL_Color *lpEntries) sha=1f41300791c1
pub fn palette_get_entries(ctx: &Ctx, n: usize) -> Vec<Color> {
    ctx.dx.pal.system_palette[..n].to_vec()
}
