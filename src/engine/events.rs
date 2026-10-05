//! `Source/engine/events.cpp` and `Source/controls/input.h`: fetching input events.

use crate::ctx::Ctx;
use crate::platform::events::Event;

/// Original: `devilution::PollEvent` (controls/input.h).
// @port controls/input.h|devilution::PollEvent(SDL_Event *event) sha=082b8e74ec53
pub fn poll_event(ctx: &mut Ctx) -> Option<Event> {
    let event = ctx.platform.poll_event()?;
    crate::controls::controller::unlock_controller_state(ctx, &event);
    crate::controls::controller::process_controller_motion(ctx, &event);
    Some(event)
}
