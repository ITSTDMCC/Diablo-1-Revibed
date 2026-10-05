//! `Source/menu.cpp`: the main-menu loop and hero selection hand-off.

use crate::ctx::Ctx;
use crate::diablo_ui::diabloui::MainmenuSelections;
use crate::engine::sound::*;

/// `_selhero_selections`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelheroSelections {
    NewDungeon,
    Continue,
    Connect,
    Previous,
}

/// Globals of menu.cpp.
#[derive(Default)]
pub struct MenuState {
    /// `gSaveNumber`
    pub g_save_number: u32,
}

/// Original: `NextTrack` (menu.cpp).
// @port menu.cpp|devilution::NextTrack() sha=3cbe51a4e4c6
fn next_track(ctx: &Ctx) -> u8 {
    if ctx.init.gb_is_spawn {
        return TMUSIC_INTRO;
    }
    let hellfire = ctx.init.gb_is_hellfire;
    match ctx.sound.sgn_music_track {
        TMUSIC_INTRO => TMUSIC_CATACOMBS,
        TMUSIC_CATACOMBS => TMUSIC_CAVES,
        TMUSIC_CAVES => TMUSIC_HELL,
        TMUSIC_HELL => {
            if hellfire {
                TMUSIC_NEST
            } else {
                TMUSIC_INTRO
            }
        }
        TMUSIC_NEST => {
            if hellfire {
                TMUSIC_CRYPT
            } else {
                TMUSIC_INTRO
            }
        }
        _ => TMUSIC_INTRO,
    }
}

/// Original: `RefreshMusic` (menu.cpp).
// @port menu.cpp|devilution::RefreshMusic() sha=4953033acfd4
fn refresh_music(ctx: &mut Ctx) {
    let t = next_track(ctx);
    music_start(ctx, t);
}

/// Original: `InitMenu` (menu.cpp).
// @port menu.cpp|devilution::InitMenu(_selhero_selections type) sha=c71f2e72dea1
fn init_menu(ctx: &mut Ctx, type_: SelheroSelections) -> bool {
    if type_ == SelheroSelections::Previous {
        return true;
    }
    let success = crate::diablo::start_game(ctx, type_ != SelheroSelections::Continue, type_ != SelheroSelections::Connect);
    if success {
        refresh_music(ctx);
    }
    success
}

/// Original: `InitSinglePlayerMenu` (menu.cpp).
// @port menu.cpp|devilution::InitSinglePlayerMenu() sha=ad6abe7e0dd5
fn init_single_player_menu(ctx: &mut Ctx) -> bool {
    ctx.init.gb_is_multiplayer = false;
    init_menu(ctx, SelheroSelections::NewDungeon)
}

/// Original: `InitMultiPlayerMenu` (menu.cpp).
// @port menu.cpp|devilution::InitMultiPlayerMenu() sha=164680229bcc
fn init_multi_player_menu(ctx: &mut Ctx) -> bool {
    ctx.init.gb_is_multiplayer = true;
    init_menu(ctx, SelheroSelections::Connect)
}

/// Original: `PlayIntro` (menu.cpp).
// @port menu.cpp|devilution::PlayIntro() sha=9fac94c02107
fn play_intro(ctx: &mut Ctx) {
    music_stop(ctx);
    if ctx.init.gb_is_hellfire {
        crate::movie::play_movie(ctx, "gendata\\Hellfire.smk", true);
    } else {
        crate::movie::play_movie(ctx, "gendata\\diablo1.smk", true);
    }
    refresh_music(ctx);
}

/// Original: `devilution::mainmenu_wait_for_button_sound` (menu.cpp).
// @port menu.cpp|devilution::mainmenu_wait_for_button_sound() sha=1049b4cefeac
pub fn mainmenu_wait_for_button_sound(ctx: &mut Ctx) {
    if let Some(s) = ctx.dx.pal_surface.as_mut() {
        s.pixels.fill(0);
    }
    crate::diablo_ui::diabloui::ui_fade_in(ctx);
    ctx.platform.delay(350); // delay to let button pressed sound finish playing
}

/// Original: `devilution::mainmenu_loop` (menu.cpp).
// @port menu.cpp|devilution::mainmenu_loop() sha=437ec23951cd
pub fn mainmenu_loop(ctx: &mut Ctx) {
    refresh_music(ctx);
    let mut done = false;
    loop {
        let mut menu = MainmenuSelections::None;
        if crate::engine::demomode::is_running(ctx) {
            menu = MainmenuSelections::SinglePlayer;
        } else {
            let name = ctx.diablo.gsz_product_name.clone();
            if !crate::diablo_ui::mainmenu::ui_main_menu_dialog(ctx, &name, &mut menu, 30) {
                crate::appfat::app_fatal(ctx, &crate::utils::language::tr("Unable to display mainmenu"));
            }
        }
        match menu {
            MainmenuSelections::None => {}
            MainmenuSelections::SinglePlayer => {
                if !init_single_player_menu(ctx) {
                    done = true;
                }
            }
            MainmenuSelections::Multiplayer => {
                if !init_multi_player_menu(ctx) {
                    done = true;
                }
            }
            MainmenuSelections::AttractMode => {
                if ctx.init.gb_is_spawn && ctx.init.archives.diabdat_mpq.is_none() {
                    done = false;
                } else if ctx.init.gb_active {
                    play_intro(ctx);
                }
            }
            MainmenuSelections::ShowCredits => crate::diablo_ui::credits::ui_credits_dialog(ctx),
            MainmenuSelections::ShowSupport => crate::diablo_ui::credits::ui_support_dialog(ctx),
            MainmenuSelections::ExitDiablo => {
                mainmenu_wait_for_button_sound(ctx);
                done = true;
            }
            MainmenuSelections::Settings => crate::diablo_ui::settingsmenu::ui_settings_menu(ctx),
        }
        if done {
            break;
        }
    }
    music_stop(ctx);
}
