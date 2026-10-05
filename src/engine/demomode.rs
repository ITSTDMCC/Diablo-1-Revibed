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

crate::pending_fn!(pub fn fetch_message(ctx: &mut Ctx) -> Option<(Option<crate::platform::events::Event>, u16)>, "engine/demomode.cpp|devilution::demo::FetchMessage(SDL_Event *event, uint16_t *modState)");
crate::pending_fn!(pub fn record_message(ctx: &mut Ctx, event: Option<&crate::platform::events::Event>, mod_state: u16), "engine/demomode.cpp|devilution::demo::RecordMessage(const SDL_Event &event, uint16_t modState)");
crate::pending_fn!(pub fn get_run_game_loop(ctx: &mut Ctx, draw_game: &mut bool, process_input: &mut bool) -> bool, "engine/demomode.cpp|devilution::demo::GetRunGameLoop(bool &drawGame, bool &processInput)");
crate::pending_fn!(pub fn record_game_loop_result(ctx: &mut Ctx, run_game_loop: bool), "engine/demomode.cpp|devilution::demo::RecordGameLoopResult(bool runGameLoop)");

/// `demo::NotifyGameLoopStart` (engine/demomode.cpp): nothing to do unless recording or playing
/// back, which is still pending.
// @port engine/demomode.cpp|devilution::demo::NotifyGameLoopStart() sha=369a9b27c392
pub fn notify_game_loop_start(ctx: &mut Ctx) {
    if is_recording(ctx) || is_running(ctx) {
        crate::unported!("engine/demomode.cpp|devilution::demo::NotifyGameLoopStart()");
    }
}

/// `demo::NotifyGameLoopEnd` (engine/demomode.cpp): as `notify_game_loop_start`.
// @port engine/demomode.cpp|devilution::demo::NotifyGameLoopEnd() sha=d44a4f2193d2
pub fn notify_game_loop_end(ctx: &mut Ctx) {
    if is_recording(ctx) || is_running(ctx) {
        crate::unported!("engine/demomode.cpp|devilution::demo::NotifyGameLoopEnd()");
    }
}
