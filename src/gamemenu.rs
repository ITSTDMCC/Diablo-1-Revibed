//! `Source/gamemenu.cpp`: the in-game menu functions.

use crate::ctx::Ctx;
use crate::engine::sound::{VOLUME_MAX, VOLUME_MIN};
use crate::enums::*;
use crate::gmenu::{gmenu_set_items, gmenu_slider_get, gmenu_slider_set, gmenu_slider_steps, menu_items, MenuId, TMenuItem, GMENU_ENABLED, GMENU_SLIDER};
use crate::utils::language::tr;

/// `VOLUME_STEPS` (engine/sound_defs.hpp)
const VOLUME_STEPS: i32 = 64;

/// Globals of gamemenu.cpp: the menus, which the menu handlers modify in place.
pub struct GamemenuState {
    /// `sgSingleMenu`: the game menu items of the single player menu.
    pub sg_single_menu: Vec<TMenuItem>,
    /// `sgMultiMenu`: the game menu items of the multi player menu.
    pub sg_multi_menu: Vec<TMenuItem>,
    /// `sgOptionsMenu`
    pub sg_options_menu: Vec<TMenuItem>,
}

impl Default for GamemenuState {
    fn default() -> Self {
        let e = GMENU_ENABLED;
        let es = GMENU_ENABLED | GMENU_SLIDER;
        GamemenuState {
            sg_single_menu: vec![
                TMenuItem::new(e, Some("Save Game"), Some(gamemenu_save_game)),
                TMenuItem::new(e, Some("Options"), Some(gamemenu_options)),
                TMenuItem::new(e, Some("New Game"), Some(gamemenu_new_game)),
                TMenuItem::new(e, Some("Load Game"), Some(gamemenu_load_game)),
                TMenuItem::new(e, Some("Quit Game"), Some(gamemenu_quit_game)),
                TMenuItem::new(e, None, None),
            ],
            sg_multi_menu: vec![
                TMenuItem::new(e, Some("Options"), Some(gamemenu_options)),
                TMenuItem::new(e, Some("New Game"), Some(gamemenu_new_game)),
                TMenuItem::new(e, Some("Restart In Town"), Some(gamemenu_restart_town)),
                TMenuItem::new(e, Some("Quit Game"), Some(gamemenu_quit_game)),
                TMenuItem::new(e, None, None),
            ],
            sg_options_menu: vec![
                TMenuItem::new(es, None, Some(gamemenu_music_volume)),
                TMenuItem::new(es, None, Some(gamemenu_sound_volume)),
                TMenuItem::new(es, Some("Gamma"), Some(gamemenu_gamma)),
                TMenuItem::new(es, Some("Speed"), Some(gamemenu_speed)),
                TMenuItem::new(e, Some("Previous Menu"), Some(gamemenu_previous)),
                TMenuItem::new(e, None, None),
            ],
        }
    }
}

/// Specifies the menu names for music enabled and disabled.
const MUSIC_TOGGLE_NAMES: [&str; 2] = ["Music", "Music Disabled"];
/// Specifies the menu names for sound enabled and disabled.
const SOUND_TOGGLE_NAMES: [&str; 2] = ["Sound", "Sound Disabled"];

fn options_item(ctx: &mut Ctx, i: usize) -> &mut TMenuItem {
    &mut menu_items(ctx, MenuId::Options)[i]
}

/// Original: `GamemenuUpdateSingle` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuUpdateSingle() sha=0ab2c6871d07
fn gamemenu_update_single(ctx: &mut Ctx) {
    let valid = crate::pfile::gb_valid_save_file(ctx);
    ctx.gamemenu.sg_single_menu[3].set_enabled(valid);
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let enable = ctx.players.Players[me]._pmode != PM_DEATH && !ctx.players.MyPlayerIsDead;
    ctx.gamemenu.sg_single_menu[0].set_enabled(enable);
}

/// Original: `GamemenuUpdateMulti` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuUpdateMulti() sha=8221ce05a32e
fn gamemenu_update_multi(ctx: &mut Ctx) {
    let dead = ctx.players.MyPlayerIsDead;
    ctx.gamemenu.sg_multi_menu[2].set_enabled(dead);
}

/// Original: `GamemenuPrevious` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuPrevious(bool) sha=9c16cebe425a
fn gamemenu_previous(ctx: &mut Ctx, _activate: bool) {
    gamemenu_on(ctx);
}

/// Original: `GamemenuNewGame` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuNewGame(bool) sha=0b18eb0395ce
fn gamemenu_new_game(ctx: &mut Ctx, _activate: bool) {
    for player in ctx.players.Players.iter_mut() {
        player._pmode = PM_QUIT;
        player._pInvincible = true;
    }
    ctx.players.MyPlayerIsDead = false;
    if !ctx.diablo.headless_mode {
        crate::engine::backbuffer_state::redraw_everything(ctx);
        crate::engine::render::scrollrt::scrollrt_draw_game_screen(ctx);
    }
    ctx.items.CornerStone.activated = false;
    ctx.diablo.gb_run_game = false;
    gamemenu_off(ctx);
}

/// Original: `GamemenuRestartTown` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuRestartTown(bool) sha=f21210cbd0db
fn gamemenu_restart_town(ctx: &mut Ctx, _activate: bool) {
    crate::msg::net_send_cmd(ctx, true, CMD_RETOWN);
}

/// Original: `GamemenuSoundMusicToggle` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuSoundMusicToggle(const char *const *names, TMenuItem *menuItem, int volume) sha=ca8df1f19f9c
fn gamemenu_sound_music_toggle(ctx: &mut Ctx, names: &[&str; 2], index: usize, volume: i32) {
    let inited = ctx.sound.gb_snd_inited;
    let menu_item = options_item(ctx, index);
    if inited {
        menu_item.add_flags(GMENU_ENABLED | GMENU_SLIDER);
        menu_item.psz_str = Some(names[0].to_string());
        gmenu_slider_steps(menu_item, VOLUME_STEPS);
        gmenu_slider_set(menu_item, VOLUME_MIN, VOLUME_MAX, volume);
        return;
    }
    menu_item.remove_flags(GMENU_ENABLED | GMENU_SLIDER);
    menu_item.psz_str = Some(names[1].to_string());
}

/// Original: `GamemenuSliderMusicSound` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuSliderMusicSound(TMenuItem *menuItem) sha=0252c12824a6
fn gamemenu_slider_music_sound(menu_item: &TMenuItem) -> i32 {
    gmenu_slider_get(menu_item, VOLUME_MIN, VOLUME_MAX)
}

/// Original: `GamemenuGetMusic` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuGetMusic() sha=5eb2df46b376
fn gamemenu_get_music(ctx: &mut Ctx) {
    let v = crate::engine::sound::sound_get_or_set_music_volume(ctx, 1);
    gamemenu_sound_music_toggle(ctx, &MUSIC_TOGGLE_NAMES, 0, v);
}

/// Original: `GamemenuGetSound` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuGetSound() sha=6378caea11c5
fn gamemenu_get_sound(ctx: &mut Ctx) {
    let v = crate::engine::sound::sound_get_or_set_sound_volume(ctx, 1);
    gamemenu_sound_music_toggle(ctx, &SOUND_TOGGLE_NAMES, 1, v);
}

/// Original: `GamemenuGetGamma` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuGetGamma() sha=7b21c91c7aa1
fn gamemenu_get_gamma(ctx: &mut Ctx) {
    gmenu_slider_steps(options_item(ctx, 2), 15);
    let gamma = crate::engine::palette::update_gamma(ctx, 0);
    gmenu_slider_set(options_item(ctx, 2), 30, 100, gamma);
}

/// Original: `GamemenuGetSpeed` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuGetSpeed() sha=6bf2469846b2
fn gamemenu_get_speed(ctx: &mut Ctx) {
    let tick_rate = ctx.multi.sgGameInitInfo.nTickRate as i32;
    if ctx.init.gb_is_multiplayer {
        let item = options_item(ctx, 3);
        item.remove_flags(GMENU_ENABLED | GMENU_SLIDER);
        if tick_rate >= 50 {
            item.psz_str = Some(tr("Speed: Fastest"));
        } else if tick_rate >= 40 {
            item.psz_str = Some(tr("Speed: Faster"));
        } else if tick_rate >= 30 {
            item.psz_str = Some(tr("Speed: Fast"));
        } else if tick_rate == 20 {
            item.psz_str = Some(tr("Speed: Normal"));
        }
        return;
    }
    let item = options_item(ctx, 3);
    item.add_flags(GMENU_ENABLED | GMENU_SLIDER);
    item.psz_str = Some(tr("Speed"));
    gmenu_slider_steps(item, 46);
    gmenu_slider_set(item, 20, 50, tick_rate);
}

/// Original: `GamemenuSliderGamma` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuSliderGamma() sha=f3cd09e27f02
fn gamemenu_slider_gamma(ctx: &mut Ctx) -> i32 {
    gmenu_slider_get(options_item(ctx, 2), 30, 100)
}

/// Original: `GamemenuOptions` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuOptions(bool) sha=3d559b9882d3
fn gamemenu_options(ctx: &mut Ctx, _activate: bool) {
    gamemenu_get_music(ctx);
    gamemenu_get_sound(ctx);
    gamemenu_get_gamma(ctx);
    gamemenu_get_speed(ctx);
    gmenu_set_items(ctx, Some(MenuId::Options), None);
}

/// Original: `GamemenuMusicVolume` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuMusicVolume(bool bActivate) sha=d3c025ca8ec6
fn gamemenu_music_volume(ctx: &mut Ctx, activate: bool) {
    use crate::engine::sound::*;
    let lt = ctx.gendung.leveltype;
    if activate {
        if ctx.sound.gb_music_on {
            ctx.sound.gb_music_on = false;
            music_stop(ctx);
            sound_get_or_set_music_volume(ctx, VOLUME_MIN);
        } else {
            ctx.sound.gb_music_on = true;
            sound_get_or_set_music_volume(ctx, VOLUME_MAX);
            music_start(ctx, get_level_music(lt));
        }
    } else {
        let volume = gamemenu_slider_music_sound(options_item(ctx, 0));
        sound_get_or_set_music_volume(ctx, volume);
        if volume == VOLUME_MIN {
            if ctx.sound.gb_music_on {
                ctx.sound.gb_music_on = false;
                music_stop(ctx);
            }
        } else if !ctx.sound.gb_music_on {
            ctx.sound.gb_music_on = true;
            music_start(ctx, get_level_music(lt));
        }
    }
    gamemenu_get_music(ctx);
}

/// Original: `GamemenuSoundVolume` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuSoundVolume(bool bActivate) sha=254f44bfc409
fn gamemenu_sound_volume(ctx: &mut Ctx, activate: bool) {
    use crate::engine::sound::sound_get_or_set_sound_volume;
    if activate {
        if ctx.sound.gb_sound_on {
            ctx.sound.gb_sound_on = false;
            crate::effects::sound_stop(ctx);
            sound_get_or_set_sound_volume(ctx, VOLUME_MIN);
        } else {
            ctx.sound.gb_sound_on = true;
            sound_get_or_set_sound_volume(ctx, VOLUME_MAX);
        }
    } else {
        let volume = gamemenu_slider_music_sound(options_item(ctx, 1));
        sound_get_or_set_sound_volume(ctx, volume);
        if volume == VOLUME_MIN {
            if ctx.sound.gb_sound_on {
                ctx.sound.gb_sound_on = false;
                crate::effects::sound_stop(ctx);
            }
        } else if !ctx.sound.gb_sound_on {
            ctx.sound.gb_sound_on = true;
        }
    }
    crate::effects::play_sfx(ctx, crate::effects::IS_TITLEMOV);
    gamemenu_get_sound(ctx);
}

/// Original: `GamemenuGamma` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuGamma(bool bActivate) sha=f864f1b72de4
fn gamemenu_gamma(ctx: &mut Ctx, activate: bool) {
    let gamma = if activate {
        if crate::engine::palette::update_gamma(ctx, 0) == 30 {
            100
        } else {
            30
        }
    } else {
        gamemenu_slider_gamma(ctx)
    };
    crate::engine::palette::update_gamma(ctx, gamma);
    gamemenu_get_gamma(ctx);
}

/// Original: `GamemenuSpeed` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::GamemenuSpeed(bool bActivate) sha=009717494d8b
fn gamemenu_speed(ctx: &mut Ctx, activate: bool) {
    if activate {
        let info = &mut ctx.multi.sgGameInitInfo;
        info.nTickRate = if info.nTickRate != 20 { 20 } else { 50 };
        let rate = info.nTickRate as i32;
        gmenu_slider_set(options_item(ctx, 3), 20, 50, rate);
    } else {
        let rate = gmenu_slider_get(options_item(ctx, 3), 20, 50);
        ctx.multi.sgGameInitInfo.nTickRate = rate as _;
    }
    let rate = ctx.multi.sgGameInitInfo.nTickRate as i32;
    if let Some(cb) = ctx.options.gameplay.tick_rate.set_value(rate) {
        crate::options::run_option_callback(ctx, cb);
    }
    ctx.diablo.gn_tick_delay = (1000 / rate) as u16;
}

/// Original: `devilution::gamemenu_quit_game` (gamemenu.cpp), NOEXIT not defined.
// @port gamemenu.cpp|devilution::gamemenu_quit_game(bool bActivate) sha=123e2d3df316
pub fn gamemenu_quit_game(ctx: &mut Ctx, activate: bool) {
    gamemenu_new_game(ctx, activate);
    ctx.diablo.gb_run_game_result = false;
}

/// Original: `devilution::gamemenu_load_game` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::gamemenu_load_game(bool) sha=31d578505588
pub fn gamemenu_load_game(ctx: &mut Ctx, _activate: bool) {
    let save_proc = crate::engine::events::set_event_handler(ctx, Some(crate::diablo::disable_input_event_handler));
    gamemenu_off(ctx);
    crate::qol::floatingnumbers::clear_floating_numbers(ctx);
    crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_NONE);
    crate::error::init_diablo_msg_id(ctx, EMSG_LOADING as usize, 3500);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    crate::engine::render::scrollrt::draw_and_blit(ctx);
    crate::loadsave::load_game(ctx, false);
    crate::error::clr_diablo_msg(ctx);
    ctx.items.CornerStone.activated = false;
    crate::engine::palette::palette_fade_out(ctx, 8);
    ctx.players.MyPlayerIsDead = false;
    crate::engine::backbuffer_state::redraw_everything(ctx);
    crate::engine::render::scrollrt::draw_and_blit(ctx);
    crate::quests::load_p_water_palette(ctx);
    crate::engine::palette::palette_fade_in(ctx, 8);
    crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HAND);
    crate::interfac::interface_msg_pump(ctx);
    crate::engine::events::set_event_handler(ctx, save_proc);
}

/// Original: `devilution::gamemenu_save_game` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::gamemenu_save_game(bool) sha=de798347419a
pub fn gamemenu_save_game(ctx: &mut Ctx, _activate: bool) {
    if ctx.cursor.pcurs != crate::cursor::CURSOR_HAND {
        return;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.players.Players[me]._pmode == PM_DEATH || ctx.players.MyPlayerIsDead {
        gamemenu_off(ctx);
        return;
    }
    let save_proc = crate::engine::events::set_event_handler(ctx, Some(crate::diablo::disable_input_event_handler));
    crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_NONE);
    gamemenu_off(ctx);
    crate::error::init_diablo_msg_id(ctx, EMSG_SAVING as usize, 3500);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    crate::engine::render::scrollrt::draw_and_blit(ctx);
    crate::loadsave::save_game(ctx);
    crate::error::clr_diablo_msg(ctx);
    crate::error::init_diablo_msg_id(ctx, EMSG_GAME_SAVED as usize, 1000);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    crate::cursor::new_cursor(ctx, crate::cursor::CURSOR_HAND);
    if ctx.items.CornerStone.activated {
        crate::items::cornerstone_save(ctx);
        crate::options::save_options(ctx);
    }
    crate::interfac::interface_msg_pump(ctx);
    crate::engine::events::set_event_handler(ctx, save_proc);
}

/// Original: `devilution::gamemenu_on` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::gamemenu_on() sha=85f64fd2e9b2
pub fn gamemenu_on(ctx: &mut Ctx) {
    if !ctx.init.gb_is_multiplayer {
        gmenu_set_items(ctx, Some(MenuId::Single), Some(gamemenu_update_single));
    } else {
        gmenu_set_items(ctx, Some(MenuId::Multi), Some(gamemenu_update_multi));
    }
    crate::diablo::press_esc_key(ctx);
}

/// Original: `devilution::gamemenu_off` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::gamemenu_off() sha=3d003cf3a5f8
pub fn gamemenu_off(ctx: &mut Ctx) {
    gmenu_set_items(ctx, None, None);
}

/// Original: `devilution::gamemenu_handle_previous` (gamemenu.cpp).
// @port gamemenu.cpp|devilution::gamemenu_handle_previous() sha=21f8a16be665
pub fn gamemenu_handle_previous(ctx: &mut Ctx) {
    if crate::gmenu::gmenu_is_active(ctx) {
        gamemenu_off(ctx);
    } else {
        gamemenu_on(ctx);
    }
}
