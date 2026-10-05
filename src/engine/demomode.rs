//! `Source/engine/demomode.cpp`: demo recording and playback.

use crate::ctx::Ctx;

pub struct DemoState {
    /// `DemoNumber`
    pub demo_number: i32,
    /// `Timedemo`
    pub timedemo: bool,
    /// `RecordNumber`
    pub record_number: i32,
}

impl Default for DemoState {
    fn default() -> Self {
        DemoState { demo_number: -1, timedemo: false, record_number: -1 }
    }
}

/// Original: `demo::IsRunning` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::IsRunning() sha=c1bcd2c9a00b
pub fn is_running(ctx: &Ctx) -> bool {
    ctx.demo.demo_number != -1
}

/// Original: `demo::IsRecording` (engine/demomode.cpp).
// @port engine/demomode.cpp|devilution::demo::IsRecording() sha=a3b359a4f217
pub fn is_recording(ctx: &Ctx) -> bool {
    ctx.demo.record_number != -1
}

crate::pending_fn!(pub fn override_options(ctx: &mut Ctx), "engine/demomode.cpp|devilution::demo::OverrideOptions()");
crate::pending_fn!(pub fn init_play_back(ctx: &mut Ctx, demo_number: i32, timedemo: bool), "engine/demomode.cpp|devilution::demo::InitPlayBack(int demoNumber, bool timedemo)");
crate::pending_fn!(pub fn init_recording(ctx: &mut Ctx, record_number: i32, create_demo_reference: bool), "engine/demomode.cpp|devilution::demo::InitRecording(int recordNumber, bool createDemoReference)");
