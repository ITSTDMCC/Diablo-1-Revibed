//! `Source/capture` (pending functions are declared with `pending_fn!`).

#[allow(unused_imports)]
use crate::ctx::Ctx;

crate::pending_fn!(pub fn capture_screen(ctx: &mut Ctx), "capture.cpp|devilution::CaptureScreen()");
