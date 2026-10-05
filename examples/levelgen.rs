//! Developer probe: load a single-player save (as a timedemo replay does), take the stairs down
//! to the next level the way `ShowProgress(WM_DIABNEXTLVL)` does, and print the objects and
//! monsters the level generator placed (in save order), for comparison with a save the original
//! game wrote right after reaching that level.
//!
//!   cargo run --release --example levelgen -- <save_dir> [--spawn]
//!
//! Run from the folder that holds DIABDAT.MPQ. Writes the level-1 save into the save's
//! temp entries (use a copy of the save).

use diablo1_rs::ctx::Ctx;
use diablo1_rs::enums::*;
use diablo1_rs::platform::Platform;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let save_dir = format!("{}/", args[1]);
    let spawn = args.iter().any(|a| a == "--spawn");
    std::panic::set_hook(Box::new(|info| {
        eprintln!("FATAL: {info}");
        std::process::exit(101);
    }));
    let mut ctx = Ctx::new(Platform::headless_from_env());
    let ctx = &mut ctx;
    ctx.paths.set_pref_path(&save_dir);
    ctx.paths.set_config_path(&save_dir);
    diablo1_rs::init::load_core_archives(ctx);
    diablo1_rs::init::load_game_archives(ctx);
    diablo1_rs::diablo::init_keymap_actions(ctx);
    diablo1_rs::options::load_options(ctx);
    ctx.players.Players.truncate(1);
    while ctx.players.Players.is_empty() {
        ctx.players.Players.push(Default::default());
    }
    ctx.players.MyPlayerId = 0;
    ctx.players.MyPlayer = Some(0);
    ctx.init.gb_is_spawn = spawn;
    ctx.init.gb_is_hellfire = false;
    ctx.sound.gb_music_on = false;
    ctx.sound.gb_sound_on = false;
    ctx.diablo.headless_mode = true;
    ctx.menu.g_save_number = 0;
    diablo1_rs::pfile::pfile_ui_set_hero_infos(ctx, &mut |_, _| true);
    ctx.diablo.gb_load_game = true;
    ctx.loadsave.giNumberOfLevels = 17;
    assert!(diablo1_rs::multi::net_init(ctx, true), "NetInit");
    diablo1_rs::loadsave::load_game(ctx, true);
    let me = 0;
    println!("loaded: level {} player at {:?}", ctx.gendung.currlevel, ctx.players.Players[me].position.tile);

    // StartNewLvl(player, WM_DIABNEXTLVL, currlevel + 1) + ShowProgress(WM_DIABNEXTLVL)
    let next = ctx.gendung.currlevel + 1;
    ctx.players.Players[me].plrlevel = next;
    diablo1_rs::interfac::save_level_left(ctx);
    diablo1_rs::diablo::free_game_mem(ctx);
    ctx.gendung.setlevel = false;
    ctx.gendung.currlevel = next;
    ctx.gendung.leveltype = diablo1_rs::levels::gendung::get_level_type(next as i32);
    diablo1_rs::diablo::load_game_level(ctx, false, ENTRY_MAIN);

    println!("level {} objects {} monsters {}", ctx.gendung.currlevel, ctx.objects.ActiveObjectCount, ctx.monster.ActiveMonsterCount);
    for i in 0..ctx.objects.ActiveObjectCount as usize {
        let o = &ctx.objects.Objects[ctx.objects.ActiveObjects[i] as usize];
        println!("object {i}: type {} at ({}, {})", o._otype as i32, o.position.x, o.position.y);
    }
    for i in 0..ctx.monster.ActiveMonsterCount {
        let m = &ctx.monster.Monsters[ctx.monster.ActiveMonsters[i] as usize];
        println!("monster {i}: slot {} levelType {} toHit {} at {:?}", ctx.monster.ActiveMonsters[i], m.levelType, m.toHit, m.position.tile);
    }
}
