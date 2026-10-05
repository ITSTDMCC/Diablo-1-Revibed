//! The port of DevilutionX's `test/timedemo_test.cpp`: replays the WarriorLevel1to2 demo that
//! DevilutionX recorded (a warrior playing dungeon level 1 down to level 2) and requires the
//! resulting hero, game and level saves to be byte-identical to the reference DevilutionX wrote.
//!
//! Needs the game data (`DIABLO_DATA_DIR` = a folder with DIABDAT.MPQ or spawn.mpq) and the
//! DevilutionX source (`DEVILUTIONX_SOURCE`, default `../Decomp/source_1.5.3`); skipped without
//! them. The fixtures are copied to `target/timedemo/` because the replay rewrites the save.

mod common;

use diablo1_rs::ctx::Ctx;
use diablo1_rs::platform::Platform;

fn copy_dir(from: &std::path::Path, to: &std::path::Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let target = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_dir(&e.path(), &target);
        } else {
            std::fs::copy(e.path(), target).unwrap();
        }
    }
}

fn run_timedemo(fixture: &str) {
    let Some(data_dir) = std::env::var_os("DIABLO_DATA_DIR") else {
        eprintln!("skipped: set DIABLO_DATA_DIR to the folder with DIABDAT.MPQ");
        return;
    };
    let source = common::drlg::source_dir().join("test").join("fixtures");
    if !source.join("timedemo").join(fixture).is_dir() {
        eprintln!("skipped: {} not found", source.display());
        return;
    }
    let work = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target").join("timedemo");
    let _ = std::fs::remove_dir_all(&work);
    copy_dir(&source.join("timedemo").join(fixture), &work.join("test/fixtures/timedemo").join(fixture));
    copy_dir(&source.join("memory_map"), &work.join("test/fixtures/memory_map"));
    // The archives are found through the working directory, as the original's PWD search path.
    std::env::set_current_dir(&data_dir).unwrap();

    let mut ctx = Ctx::new(Platform::headless_from_env());
    let ctx = &mut ctx;
    let unit_test_folder_complete_path = format!("{}/test/fixtures/timedemo/{fixture}/", work.display());
    ctx.paths.set_base_path(&format!("{}/", work.display()));
    ctx.paths.set_pref_path(&unit_test_folder_complete_path);
    ctx.paths.set_config_path(&unit_test_folder_complete_path);
    diablo1_rs::init::load_core_archives(ctx);
    diablo1_rs::init::load_game_archives(ctx);
    assert!(diablo1_rs::init::have_spawn(ctx) || diablo1_rs::init::have_diabdat(ctx), "The tests need spawn.mpq or diabdat.mpq");

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

    diablo1_rs::diablo_game::start_game(ctx, false, true);

    let (status, message) = diablo1_rs::pfile::pfile_compare_hero_demo(ctx, demo_number, true);
    assert_eq!(status, diablo1_rs::pfile::HERO_COMPARE_SAME, "{message}");
    assert!(!ctx.diablo.gb_run_game);
}

#[test]
fn timedemo_warrior_level1to2() {
    run_timedemo("WarriorLevel1to2");
}
