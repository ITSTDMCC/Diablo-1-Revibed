//! diablo1_rs: runs the ported game on its own thread (DevilutionX's imperative control flow)
//! and the Bevy window on the main thread.

use std::sync::{mpsc, Arc, Mutex};

use diablo1_rs::ctx::Ctx;
use diablo1_rs::platform::{self, Platform, TestHooks};

fn run_game(platform: Platform, args: Vec<String>) {
    let mut ctx = Ctx::new(platform);
    let code = diablo1_rs::diablo::diablo_main(&mut ctx, &args);
    ctx.platform.request_exit();
    std::process::exit(code);
}

// @port main.cpp|main(int argc, char **argv) sha=6675298b394f
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let headless = std::env::var_os("DIABLO_HEADLESS").is_some_and(|v| v != "0");
    // A panic on the game thread (an unported function, a bug) ends the process with its message.
    std::panic::set_hook(Box::new(|info| {
        eprintln!("FATAL: {info}");
        std::process::exit(101);
    }));
    if headless {
        run_game(Platform::headless_from_env(), args);
        return;
    }
    let (events_tx, events_rx) = mpsc::channel();
    let (exit_tx, exit_rx) = mpsc::channel();
    let (commands_tx, commands_rx) = mpsc::channel();
    let frames: platform::FrameSlot = Arc::new(Mutex::new(None));
    let game_frames = frames.clone();
    let window_info: platform::WindowInfoSlot = Arc::new(Mutex::new(platform::WindowInfo { render_scale: 1.0, size: (0, 0) }));
    let game_window_info = window_info.clone();
    std::thread::Builder::new()
        .name("game".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let mut p = Platform::new(Some(events_rx), game_frames, TestHooks::from_env(), false, Some(exit_tx), Some(commands_tx));
            p.window_info = game_window_info;
            run_game(p, args);
        })
        .expect("spawn game thread");
    platform::bevy_front::run(frames, events_tx, exit_rx, commands_rx, window_info);
}
