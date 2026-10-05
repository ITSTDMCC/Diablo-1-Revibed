//! Replays one of DevilutionX's own timedemo fixtures and compares the resulting save with the
//! reference save DevilutionX produced - the port of `test/timedemo_test.cpp`.
//!
//!   cargo run --release --example timedemo -- <work_dir> <fixture_name>
//!
//! `<work_dir>` must contain `test/fixtures/timedemo/<fixture_name>/` (demo_0.dmo, the hero save
//! and its reference) and `test/fixtures/memory_map/` - copy them from the DevilutionX source;
//! the replay writes `demo_0_actual_*.sv` next to the fixture. Run it from the folder that holds
//! DIABDAT.MPQ (or spawn.mpq): the archives are found through the working directory.

use diablo1_rs::ctx::Ctx;
use diablo1_rs::platform::Platform;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let work_dir = &args[1];
    let fixture = &args[2];
    std::panic::set_hook(Box::new(|info| {
        eprintln!("FATAL: {info}");
        std::process::exit(101);
    }));

    let mut ctx = Ctx::new(Platform::headless_from_env());
    let ctx = &mut ctx;

    // RunTimedemo
    let unit_test_folder_complete_path = format!("{work_dir}/test/fixtures/timedemo/{fixture}/");
    ctx.paths.set_base_path(&format!("{work_dir}/"));
    ctx.paths.set_pref_path(&unit_test_folder_complete_path);
    ctx.paths.set_config_path(&unit_test_folder_complete_path);
    diablo1_rs::init::load_core_archives(ctx);
    diablo1_rs::init::load_game_archives(ctx);

    // The tests need spawn.mpq or diabdat.mpq
    assert!(diablo1_rs::init::have_spawn(ctx) || diablo1_rs::init::have_diabdat(ctx), "spawn.mpq or DIABDAT.MPQ not found");

    diablo1_rs::diablo::init_keymap_actions(ctx);
    diablo1_rs::options::load_options(ctx);

    let demo_number = 0;

    ctx.players.Players.truncate(1);
    while ctx.players.Players.is_empty() {
        ctx.players.Players.push(Default::default());
    }
    ctx.players.MyPlayerId = demo_number as usize;
    ctx.players.MyPlayer = Some(ctx.players.MyPlayerId);
    ctx.players.Players[0] = Default::default();

    // Currently only spawn.mpq is present when building on github actions
    ctx.init.gb_is_spawn = true;
    ctx.init.gb_is_hellfire = false;
    ctx.sound.gb_music_on = false;
    ctx.sound.gb_sound_on = false;
    ctx.diablo.headless_mode = true;
    diablo1_rs::engine::demomode::init_play_back(ctx, demo_number, true);

    diablo1_rs::pfile::pfile_ui_set_hero_infos(ctx, &mut |_, _| true);
    ctx.diablo.gb_load_game = true;

    diablo1_rs::engine::demomode::override_options(ctx);

    let force_resolution = ctx.dx.force_resolution;
    diablo1_rs::utils::display::adjust_to_screen_geometry(ctx, force_resolution);

    let started = std::time::Instant::now();
    diablo1_rs::diablo_game::start_game(ctx, false, true);
    let elapsed = started.elapsed();

    let (status, message) = diablo1_rs::pfile::pfile_compare_hero_demo(ctx, demo_number, true);
    println!("replayed in {elapsed:?}; gbRunGame = {}", ctx.diablo.gb_run_game);
    match status {
        diablo1_rs::pfile::HERO_COMPARE_SAME => println!("RESULT: Same (the port reproduces DevilutionX's outcome)"),
        diablo1_rs::pfile::HERO_COMPARE_REFERENCE_NOT_FOUND => println!("RESULT: reference not found"),
        _ => println!("RESULT: Difference\n{message}"),
    }
    std::process::exit(if status == diablo1_rs::pfile::HERO_COMPARE_SAME { 0 } else { 1 });
}
