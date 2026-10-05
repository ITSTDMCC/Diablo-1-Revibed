//! `Source/engine/*`
pub mod assets;
pub mod backbuffer_state;
pub mod demomode;
pub mod dx;
pub mod events;
pub mod load_file;
pub mod palette;
pub mod random;
pub mod render;
pub mod surface;
pub mod sound;
pub mod clx_sprite;
pub mod load_sprites;

use crate::ctx::Ctx;

/// Original: `devilution::GetAnimationFrame` (engine.h). `fps` defaults to 60 in the original
/// (it is really milliseconds per frame).
// @port engine.h|devilution::GetAnimationFrame(int frames, int fps = 60) sha=b8f57ee56fbc
pub fn get_animation_frame(ctx: &Ctx, frames: i32, fps: i32) -> i32 {
    let frame = (ctx.platform.ticks() / fps as u32) as i32 % frames;
    if frame > frames { 0 } else { frame }
}
