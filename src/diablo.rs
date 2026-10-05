//! `Source/diablo.cpp`: start-up, command line, main loop.

use crate::ctx::Ctx;
use crate::options::{self, StartUpGameMode, StartUpIntro, StartUpSplash};
use crate::utils::console::{print_in_console, print_newline_in_console};
use crate::utils::language::tr;

/// `PROJECT_NAME` / `PROJECT_VERSION` (CMakeLists.txt, VERSION) of the build being ported.
pub const PROJECT_NAME: &str = "DevilutionX";
pub const PROJECT_VERSION: &str = "1.5.3";

pub use crate::diablo_game::{
    can_player_take_action, character_sheet_key_pressed, close_panels, diablo_color_cyc_logic, diablo_pause_game, disable_input_event_handler,
    display_spells_key_pressed, game_event_handler, game_loop, help_key_pressed, inventory_key_pressed, is_diablo_alive, is_game_running,
    load_game_level, press_esc_key, print_screen, quest_log_key_pressed, spell_book_key_pressed, start_game, try_icon_curs,
};

pub struct QuickMessage {
    pub key: &'static str,
    pub message: &'static str,
}

/// `QuickMessages`
pub const QUICK_MESSAGES: [QuickMessage; options::QUICK_MESSAGE_OPTIONS] = [
    QuickMessage { key: "QuickMessage1", message: "I need help! Come Here!" },
    QuickMessage { key: "QuickMessage2", message: "Follow me." },
    QuickMessage { key: "QuickMessage3", message: "Here's something for you." },
    QuickMessage { key: "QuickMessage4", message: "Now you DIE!" },
];

/// Globals of diablo.cpp (and a few file-scope ones the start-up path needs).
pub struct DiabloState {
    pub gb_run_game: bool,
    pub gb_run_game_result: bool,
    pub return_to_main_menu: bool,
    pub gb_process_players: bool,
    pub gb_load_game: bool,
    pub cineflag: bool,
    pub pause_mode: i32,
    pub gb_bard: bool,
    pub gb_barbarian: bool,
    pub headless_mode: bool,
    /// `gnTickDelay`: milliseconds per game logic tick.
    pub gn_tick_delay: u16,
    pub gsz_product_name: String,
    pub gsz_version_number: String,
    pub gb_game_loop_startup: bool,
    pub force_spawn: bool,
    pub force_diablo: bool,
    pub sgn_timeout_curs: i32,
    pub gb_show_intro: bool,
    pub was_archives_init: bool,
    pub was_window_init: bool,
    pub was_ui_init: bool,
    /// `forceLocale` (utils/language.cpp)
    pub force_locale: String,
    /// `frameflag`, `lastFpsUpdateInMs` (engine/render/scrollrt.cpp)
    pub frameflag: bool,
    pub last_fps_update_in_ms: u32,
    pub mouse_position: (i32, i32),
    /// `LastMouseButtonAction`
    pub last_mouse_button_action: MouseActionType,
    /// `GameWasAlreadyPaused`
    pub game_was_already_paused: bool,
    /// `MinimizePaused`
    pub minimize_paused: bool,
    /// `glSeedTbl`
    pub glSeedTbl: [u32; crate::player::NUMLEVELS],
    /// `sgbMouseDown`
    pub sgb_mouse_down: crate::enums::clicktype,
    /// `gGameLogicStep`
    pub g_game_logic_step: crate::enums::GameLogicStep,
}

/// `MouseActionType`
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MouseActionType {
    #[default]
    None,
    Walk,
    Spell,
    SpellMonsterTarget,
    SpellPlayerTarget,
    Attack,
    AttackMonsterTarget,
    AttackPlayerTarget,
    OperateObject,
}

impl Default for DiabloState {
    fn default() -> Self {
        DiabloState {
            gb_run_game: false,
            gb_run_game_result: false,
            return_to_main_menu: false,
            gb_process_players: false,
            gb_load_game: false,
            cineflag: false,
            pause_mode: 0,
            gb_bard: false,
            gb_barbarian: false,
            headless_mode: false,
            gn_tick_delay: 50,
            gsz_product_name: "DevilutionX vUnknown".to_string(),
            gsz_version_number: "internal version unknown".to_string(),
            gb_game_loop_startup: false,
            force_spawn: false,
            force_diablo: false,
            sgn_timeout_curs: 0,
            gb_show_intro: true,
            was_archives_init: false,
            was_window_init: false,
            was_ui_init: false,
            force_locale: String::new(),
            frameflag: false,
            last_fps_update_in_ms: 0,
            mouse_position: (0, 0),
            last_mouse_button_action: MouseActionType::None,
            game_was_already_paused: false,
            minimize_paused: false,
            glSeedTbl: [0; crate::player::NUMLEVELS],
            sgb_mouse_down: crate::enums::CLICK_NONE,
            g_game_logic_step: crate::enums::GameLogicStep::None,
        }
    }
}

/// Original: `devilution::PrintWithRightPadding` (diablo.cpp).
// @port diablo.cpp|devilution::PrintWithRightPadding(string_view str, size_t width) sha=a510c06455c9
fn print_with_right_padding(s: &str, width: usize) {
    print_in_console(s);
    let len = s.chars().count();
    if len >= width {
        return;
    }
    print_in_console(&" ".repeat(width - len));
}

/// Original: `devilution::PrintHelpOption` (diablo.cpp).
// @port diablo.cpp|devilution::PrintHelpOption(string_view flags, string_view description) sha=10de81f7b735
fn print_help_option(flags: &str, description: &str) {
    print_in_console("    ");
    print_with_right_padding(flags, 20);
    print_in_console(" ");
    print_with_right_padding(description, 30);
    print_newline_in_console();
}

/// Original: `devilution::PrintHelpAndExit` (diablo.cpp).
// @port diablo.cpp|devilution::PrintHelpAndExit() sha=54b63a5a770c
fn print_help_and_exit(ctx: &mut Ctx) -> ! {
    print_in_console("Options:");
    print_newline_in_console();
    print_help_option("-h, --help", &tr("Print this message and exit"));
    print_help_option("--version", &tr("Print the version and exit"));
    print_help_option("--data-dir", &tr("Specify the folder of diabdat.mpq"));
    print_help_option("--save-dir", &tr("Specify the folder of save files"));
    print_help_option("--config-dir", &tr("Specify the location of diablo.ini"));
    print_help_option("--lang", &tr("Specify the language code (e.g. en or pt_BR)"));
    print_help_option("-n", &tr("Skip startup videos"));
    print_help_option("-f", &tr("Display frames per second"));
    print_help_option("--verbose", &tr("Enable verbose logging"));
    print_help_option("--record <#>", &tr("Record a demo file"));
    print_help_option("--demo <#>", &tr("Play a demo file"));
    print_help_option("--timedemo", &tr("Disable all frame limiting during demo playback"));
    print_newline_in_console();
    print_in_console(&tr("Game selection:"));
    print_newline_in_console();
    print_help_option("--spawn", &tr("Force Shareware mode"));
    print_help_option("--diablo", &tr("Force Diablo mode"));
    print_help_option("--hellfire", &tr("Force Hellfire mode"));
    print_in_console(&tr("Hellfire options:"));
    print_newline_in_console();
    print_newline_in_console();
    print_in_console(&tr("Report bugs at https://github.com/diasurgical/devilutionX/"));
    print_newline_in_console();
    diablo_quit(ctx, 0)
}

/// Original: `devilution::PrintFlagsRequiresArgument` (diablo.cpp).
// @port diablo.cpp|devilution::PrintFlagsRequiresArgument(string_view flag) sha=02df83ced28b
fn print_flags_requires_argument(flag: &str) {
    print_in_console(flag);
    print_in_console(" requires an argument");
    print_newline_in_console();
}

/// `SDL_atoi`
fn atoi(s: &str) -> i32 {
    let s = s.trim_start();
    let (neg, rest) = match s.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, s.strip_prefix('+').unwrap_or(s)),
    };
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    let v: i64 = digits.parse().unwrap_or(0);
    (if neg { -v } else { v }) as i32
}

/// Original: `devilution::DiabloParseFlags` (diablo.cpp).
// @port diablo.cpp|devilution::DiabloParseFlags(int argc, char **argv) sha=9315f1d7ef6f
pub fn diablo_parse_flags(ctx: &mut Ctx, argv: &[String]) {
    let mut timedemo = false;
    let mut demo_number = -1;
    let mut record_number = -1;
    let mut create_demo_reference = false;
    let argc = argv.len();
    let mut i = 1;
    while i < argc {
        let arg = argv[i].as_str();
        match arg {
            "-h" | "--help" => print_help_and_exit(ctx),
            "--version" => {
                print_in_console(PROJECT_NAME);
                print_in_console(" v");
                print_in_console(PROJECT_VERSION);
                print_newline_in_console();
                diablo_quit(ctx, 0);
            }
            "--data-dir" | "--save-dir" | "--config-dir" | "--lang" | "--demo" | "--record" => {
                if i + 1 == argc {
                    print_flags_requires_argument(arg);
                    diablo_quit(ctx, 64);
                }
                i += 1;
                let v = argv[i].clone();
                match arg {
                    "--data-dir" => ctx.paths.set_base_path(&v),
                    "--save-dir" => ctx.paths.set_pref_path(&v),
                    "--config-dir" => ctx.paths.set_config_path(&v),
                    "--lang" => ctx.diablo.force_locale = v,
                    "--demo" => {
                        demo_number = atoi(&v);
                        ctx.diablo.gb_show_intro = false;
                    }
                    _ => record_number = atoi(&v),
                }
            }
            "--timedemo" => timedemo = true,
            "--create-reference" => create_demo_reference = true,
            "-n" => ctx.diablo.gb_show_intro = false,
            "-f" => enable_frame_count(ctx),
            "--spawn" => ctx.diablo.force_spawn = true,
            "--diablo" => ctx.diablo.force_diablo = true,
            "--hellfire" => ctx.init.force_hellfire = true,
            "--vanilla" => ctx.init.gb_vanilla = true,
            "--verbose" => {
                // SDL_LogSetAllPriority(SDL_LOG_PRIORITY_VERBOSE)
                // SAFETY: the game thread is the only one reading the variable afterwards.
                unsafe { std::env::set_var("DIABLO_VERBOSE", "1") };
            }
            _ => {
                print_in_console("unrecognized option '");
                print_in_console(arg);
                print_in_console("'");
                print_newline_in_console();
                print_help_and_exit(ctx);
            }
        }
        i += 1;
    }
    if demo_number != -1 {
        crate::engine::demomode::init_play_back(ctx, demo_number, timedemo);
    }
    if record_number != -1 {
        crate::engine::demomode::init_recording(ctx, record_number, create_demo_reference);
    }
}

/// Original: `devilution::EnableFrameCount` (engine/render/scrollrt.cpp).
// @port engine/render/scrollrt.cpp|devilution::EnableFrameCount() sha=06f27cd5752a
pub fn enable_frame_count(ctx: &mut Ctx) {
    ctx.diablo.frameflag = true;
    ctx.diablo.last_fps_update_in_ms = ctx.platform.ticks();
}

/// Original: `devilution::SetApplicationVersions` (diablo.cpp).
// @port diablo.cpp|devilution::SetApplicationVersions() sha=f655960c55f1
fn set_application_versions(ctx: &mut Ctx) {
    ctx.diablo.gsz_product_name = format!("{PROJECT_NAME} v{PROJECT_VERSION}");
    ctx.diablo.gsz_version_number = format!("version {PROJECT_VERSION}");
}

/// Original: `devilution::ReadOnlyTest` (restrict.cpp).
// @port restrict.cpp|devilution::ReadOnlyTest() sha=3d48e6749c57
fn read_only_test(ctx: &mut Ctx) {
    let pref = ctx.paths.pref_path().to_string();
    let path = format!("{pref}Diablo1ReadOnlyTest.foo");
    if std::fs::File::create(&path).is_err() {
        crate::appfat::dir_error_dlg(ctx, &pref);
    }
    let _ = std::fs::remove_file(&path);
}

/// Original: `devilution::ApplicationInit` (diablo.cpp).
// @port diablo.cpp|devilution::ApplicationInit() sha=5912863d8ff0
fn application_init(ctx: &mut Ctx) {
    if ctx.options.graphics.show_fps.get() {
        enable_frame_count(ctx);
    }
    crate::init::init_create_window(ctx);
    ctx.diablo.was_window_init = true;
    crate::utils::language::language_initialize(ctx);
    set_application_versions(ctx);
    read_only_test(ctx);
}

/// Original: `devilution::DiabloInit` (diablo.cpp).
// @port diablo.cpp|devilution::DiabloInit() sha=81d6ca2309f8
fn diablo_init(ctx: &mut Ctx) {
    if ctx.diablo.force_spawn || ctx.options.start_up.shareware.get() {
        ctx.init.gb_is_spawn = true;
    }
    if ctx.diablo.force_diablo || ctx.options.start_up.game_mode() == StartUpGameMode::Diablo {
        ctx.init.gb_is_hellfire = false;
    }
    if ctx.init.force_hellfire {
        ctx.init.gb_is_hellfire = true;
    }
    ctx.init.gb_is_hellfire_save_game = ctx.init.gb_is_hellfire;

    for (i, qm) in QUICK_MESSAGES.iter().enumerate() {
        if ctx.options.chat.sz_hot_key_msgs[i].is_empty() {
            ctx.options.chat.sz_hot_key_msgs[i].push(tr(qm.message));
        }
    }

    crate::controls::touch::initialize_virtual_gamepad(ctx);

    crate::diablo_ui::diabloui::ui_initialize(ctx);
    ctx.diablo.was_ui_init = true;

    if ctx.init.gb_is_hellfire && !ctx.init.force_hellfire && ctx.options.start_up.game_mode() == StartUpGameMode::Ask {
        crate::diablo_ui::selstart::ui_sel_start_up_game_option(ctx);
        if !ctx.init.gb_is_hellfire {
            // Reinitialize the UI Elements cause we changed the game
            crate::diablo_ui::diabloui::unload_ui_gfx(ctx);
            crate::diablo_ui::diabloui::ui_initialize(ctx);
            if crate::hwcursor::is_hardware_cursor(ctx) {
                crate::hwcursor::set_hardware_cursor(ctx, crate::hwcursor::CursorInfo::unknown_cursor());
            }
        }
    }

    diablo_init_screen(ctx);
    crate::engine::sound::snd_init(ctx);
    crate::effects::ui_sound_init(ctx);
    // Item graphics are loaded early, they already get touched during hero selection.
    crate::items::init_item_gfx(ctx);
    // Always available.
    crate::engine::render::text_render::load_small_selection_spinner(ctx);
    crate::diablo_ui::diabloui::check_archives_up_to_date(ctx);
}

/// Original: `devilution::DiabloInitScreen` (diablo.cpp).
// @port diablo.cpp|devilution::DiabloInitScreen() sha=866fefe0d9c8
fn diablo_init_screen(ctx: &mut Ctx) {
    let (w, h) = (ctx.dx.gn_screen_width, ctx.dx.gn_screen_height);
    ctx.diablo.mouse_position = (w / 2, h / 2);
    if ctx.controls.control_mode == crate::controls::ControlTypes::KeyboardAndMouse {
        crate::controls::set_cursor_pos(ctx, ctx.diablo.mouse_position);
    }
    crate::error::clr_diablo_msg(ctx);
}

/// Original: `devilution::DiabloSplash` (diablo.cpp).
// @port diablo.cpp|devilution::DiabloSplash() sha=bc58b157038d
fn diablo_splash(ctx: &mut Ctx) {
    if !ctx.diablo.gb_show_intro {
        return;
    }
    if ctx.options.start_up.splash() == StartUpSplash::LogoAndTitleDialog {
        crate::movie::play_movie(ctx, "gendata\\logo.smk", true);
    }
    let hellfire = ctx.init.gb_is_hellfire;
    let intro = if hellfire { &ctx.options.start_up.hellfire_intro } else { &ctx.options.start_up.diablo_intro };
    let intro_value = options::intro_value(intro);
    if intro_value != StartUpIntro::Off {
        if hellfire {
            crate::movie::play_movie(ctx, "gendata\\Hellfire.smk", true);
        } else {
            crate::movie::play_movie(ctx, "gendata\\diablo1.smk", true);
        }
        if intro_value == StartUpIntro::Once {
            let intro = if hellfire { &mut ctx.options.start_up.hellfire_intro } else { &mut ctx.options.start_up.diablo_intro };
            if let Some(cb) = intro.set_raw(StartUpIntro::Off as i32) {
                options::run_option_callback(ctx, cb);
            }
            options::save_options(ctx);
        }
    }
    if matches!(ctx.options.start_up.splash(), StartUpSplash::TitleDialog | StartUpSplash::LogoAndTitleDialog) {
        crate::diablo_ui::title::ui_title_dialog(ctx);
    }
}

/// Original: `devilution::DiabloDeinit` (diablo.cpp).
// @port diablo.cpp|devilution::DiabloDeinit() sha=6bd443ea5301
fn diablo_deinit(ctx: &mut Ctx) {
    crate::items::free_item_gfx(ctx);
    if crate::engine::sound::gb_snd_inited(ctx) {
        crate::effects::effects_cleanup_sfx(ctx);
    }
    crate::engine::sound::snd_deinit(ctx);
    if ctx.diablo.was_ui_init {
        crate::diablo_ui::diabloui::ui_destroy(ctx);
    }
    if ctx.diablo.was_archives_init {
        crate::init::init_cleanup(ctx);
    }
    if ctx.diablo.was_window_init {
        crate::engine::dx::dx_cleanup(ctx);
    }
    crate::engine::render::text_render::unload_fonts(ctx);
    // SDL_Quit: the front-end closes the window when the game thread ends.
}

/// Original: `devilution::diablo_quit` (diablo.cpp).
// @port diablo.cpp|devilution::diablo_quit(int exitStatus) sha=c1047061dc8b
pub fn diablo_quit(ctx: &mut Ctx, exit_status: i32) -> ! {
    free_game_mem(ctx);
    crate::engine::sound::music_stop(ctx);
    diablo_deinit(ctx);
    ctx.platform.request_exit();
    std::process::exit(exit_status)
}

/// Original: `devilution::FreeGameMem` (diablo.cpp).
// @port diablo.cpp|devilution::FreeGameMem() sha=7b43c5519583
pub fn free_game_mem(ctx: &mut Ctx) {
    ctx.gendung.pDungeonCels = None;
    ctx.gendung.pMegaTiles = None;
    ctx.gendung.pSpecialCels = None;
    crate::monster::free_monsters(ctx);
    crate::missiles::free_missile_gfx(ctx);
    crate::objects::free_object_gfx(ctx);
    crate::towners::free_towner_gfx(ctx);
    crate::qol::stash::free_stash_gfx(ctx);
    crate::controls::touch::deactivate_virtual_gamepad(ctx);
    crate::controls::touch::free_virtual_gamepad_gfx(ctx);
}

/// Original: `devilution::DiabloMain` (diablo.cpp).
// @port diablo.cpp|devilution::DiabloMain(int argc, char **argv) sha=e6a157e7e592
pub fn diablo_main(ctx: &mut Ctx, argv: &[String]) -> i32 {
    diablo_parse_flags(ctx, argv);
    init_keymap_actions(ctx);
    init_padmap_actions(ctx);

    // Need to ensure devilutionx.mpq (and fonts.mpq if available) are loaded before attempting to read translation settings
    crate::init::load_core_archives(ctx);
    ctx.diablo.was_archives_init = true;

    // Read settings including translation next.
    ctx.options.language.code.have_extra_fonts = crate::engine::render::text_render::have_extra_fonts(ctx);
    options::load_options(ctx);
    // Then look for a voice pack file based on the selected translation
    crate::init::load_language_archive(ctx);

    application_init(ctx);
    options::save_options(ctx);

    // Finally load game data
    crate::init::load_game_archives(ctx);

    diablo_init(ctx);
    options::save_options(ctx);

    diablo_splash(ctx);
    crate::menu::mainmenu_loop(ctx);
    diablo_deinit(ctx);
    0
}

/// Original: `devilution::InitKeymapActions` (diablo.cpp).
// @port diablo.cpp|devilution::InitKeymapActions() sha=7e29f9d13920
pub fn init_keymap_actions(ctx: &mut Ctx) {
    use crate::platform::events::keys::*;
    use std::rc::Rc;
    let can_take = || -> Option<options::EnableFn> { Some(Rc::new(can_player_take_action)) };
    let km = &mut ctx.options.keymapper;
    for i in 0..8u32 {
        km.add_action(
            "BeltItem{}",
            "Belt item {}",
            "Use Belt item.",
            b'1' as u32 + i,
            Some(Rc::new(move |ctx: &mut Ctx| crate::inv::use_belt_item_slot(ctx, i as usize))),
            None,
            can_take(),
            i + 1,
        );
    }
    for i in 0..crate::spells::NUM_HOTKEYS as u32 {
        km.add_action(
            "QuickSpell{}",
            "Quick spell {}",
            "Hotkey for skill or spell.",
            if i < 4 { SDLK_F1 as u32 + 4 + i } else { SDLK_UNKNOWN as u32 },
            Some(Rc::new(move |ctx: &mut Ctx| crate::control::quick_spell_hotkey(ctx, i as usize))),
            None,
            can_take(),
            i + 1,
        );
    }
    let simple: &[(&str, &'static str, &'static str, u32, fn(&mut Ctx), Option<fn(&mut Ctx)>, u8)] = &[
        ("UseHealthPotion", "Use health potion", "Use health potions from belt.", SDLK_UNKNOWN as u32, |c| crate::inv::use_belt_item(c, crate::inv::BLT_HEALING), None, 1),
        ("UseManaPotion", "Use mana potion", "Use mana potions from belt.", SDLK_UNKNOWN as u32, |c| crate::inv::use_belt_item(c, crate::inv::BLT_MANA), None, 1),
        ("DisplaySpells", "Speedbook", "Open Speedbook.", b'S' as u32, crate::diablo::display_spells_key_pressed, None, 1),
        ("QuickSave", "Quick save", "Saves the game.", SDLK_F1 as u32 + 1, |c| crate::gamemenu::gamemenu_save_game(c, false), None, 2),
        ("QuickLoad", "Quick load", "Loads the game.", SDLK_F1 as u32 + 2, |c| crate::gamemenu::gamemenu_load_game(c, false), None, 3),
        ("QuitGame", "Quit game", "Closes the game.", SDLK_UNKNOWN as u32, |c| crate::gamemenu::gamemenu_quit_game(c, false), None, 0),
        ("StopHero", "Stop hero", "Stops walking and cancel pending actions.", SDLK_UNKNOWN as u32, crate::player::stop_my_player, None, 1),
        ("Item Highlighting", "Item highlighting", "Show/hide items on ground.", SDLK_LALT as u32, |c| crate::qol::itemlabels::highlight_key_pressed(c, true), Some(|c| crate::qol::itemlabels::highlight_key_pressed(c, false)), 0),
        ("Toggle Automap", "Toggle automap", "Toggles if automap is displayed.", SDLK_TAB as u32, crate::automap::do_auto_map, None, 4),
        ("Inventory", "Inventory", "Open Inventory screen.", b'I' as u32, crate::diablo::inventory_key_pressed, None, 1),
        ("Character", "Character", "Open Character screen.", b'C' as u32, crate::diablo::character_sheet_key_pressed, None, 1),
        ("QuestLog", "Quest log", "Open Quest log.", b'Q' as u32, crate::diablo::quest_log_key_pressed, None, 1),
        ("SpellBook", "Spellbook", "Open Spellbook.", b'B' as u32, crate::diablo::spell_book_key_pressed, None, 1),
    ];
    let enable_for = |kind: u8| -> Option<options::EnableFn> {
        match kind {
            1 => Some(Rc::new(can_player_take_action)),
            2 => Some(Rc::new(|c: &Ctx| !c.init.gb_is_multiplayer && can_player_take_action(c))),
            3 => Some(Rc::new(|c: &Ctx| {
                !c.init.gb_is_multiplayer && crate::pfile::gb_valid_save_file(c) && crate::stores::stextflag_is_none(c) && is_game_running(c)
            })),
            4 => Some(Rc::new(is_game_running)),
            _ => None,
        }
    };
    for &(key, name, desc, default_key, pressed, released, enable) in simple.iter().take(8) {
        km.add_action(key, name, desc, default_key, Some(Rc::new(pressed)), released.map(|r| Rc::new(r) as options::ActionFn), enable_for(enable), 0);
    }
    km.add_action(
        "Toggle Item Highlighting",
        "Toggle item highlighting",
        "Permanent show/hide items on ground.",
        SDLK_RCTRL as u32,
        None,
        Some(Rc::new(crate::qol::itemlabels::toggle_item_label_highlight)),
        None,
        0,
    );
    for &(key, name, desc, default_key, pressed, released, enable) in simple.iter().skip(8) {
        km.add_action(key, name, desc, default_key, Some(Rc::new(pressed)), released.map(|r| Rc::new(r) as options::ActionFn), enable_for(enable), 0);
    }
    for i in 0..4u32 {
        km.add_action(
            "QuickMessage{}",
            "Quick Message {}",
            "Use Quick Message in chat.",
            SDLK_F1 as u32 + 8 + i,
            Some(Rc::new(move |ctx: &mut Ctx| crate::control::diablo_hotkey_msg(ctx, i))),
            None,
            None,
            i + 1,
        );
    }
    km.add_action("Hide Info Screens", "Hide Info Screens", "Hide all info screens.", SDLK_SPACE as u32, Some(Rc::new(hide_info_screens)), None, Some(Rc::new(is_game_running)), 0);
    km.add_action("Zoom", "Zoom", "Zoom Game Screen.", b'Z' as u32, Some(Rc::new(toggle_zoom)), None, can_take(), 0);
    km.add_action("Pause Game", "Pause Game", "Pauses the game.", b'P' as u32, Some(Rc::new(diablo_pause_game)), None, None, 0);
    km.add_action("DecreaseGamma", "Decrease Gamma", "Reduce screen brightness.", b'G' as u32, Some(Rc::new(crate::engine::palette::decrease_gamma)), None, can_take(), 0);
    km.add_action("IncreaseGamma", "Increase Gamma", "Increase screen brightness.", b'F' as u32, Some(Rc::new(crate::engine::palette::increase_gamma)), None, can_take(), 0);
    km.add_action("Help", "Help", "Open Help Screen.", SDLK_F1 as u32, Some(Rc::new(crate::diablo::help_key_pressed)), None, can_take(), 0);
    km.add_action("Screenshot", "Screenshot", "Takes a screenshot.", SDLK_PRINTSCREEN as u32, None, Some(Rc::new(crate::capture::capture_screen)), None, 0);
    km.add_action("GameInfo", "Game info", "Displays game infos.", b'V' as u32, Some(Rc::new(game_info)), None, can_take(), 0);
    km.add_action("ChatLog", "Chat Log", "Displays chat log.", b'L' as u32, Some(Rc::new(crate::qol::chatlog::toggle_chat_log)), None, None, 0);
    km.commit_actions();
}

fn hide_info_screens(ctx: &mut Ctx) {
    crate::diablo::close_panels(ctx);
    crate::help::set_help_flag(ctx, false);
    crate::qol::chatlog::clear_chat_log_flag(ctx);
    ctx.control.spselflag = false;
    if crate::minitext::qtextflag(ctx) && crate::levels::gendung::leveltype_is_town(ctx) {
        crate::minitext::set_qtextflag(ctx, false);
        crate::effects::stream_stop(ctx);
    }
    crate::automap::set_automap_active(ctx, false);
    crate::error::cancel_current_diablo_msg(ctx);
    crate::gamemenu::gamemenu_off(ctx);
    crate::doom::doom_close(ctx);
}

fn toggle_zoom(ctx: &mut Ctx) {
    let v = !ctx.options.graphics.zoom.get();
    if let Some(cb) = ctx.options.graphics.zoom.set_value(v) {
        options::run_option_callback(ctx, cb);
    }
    crate::engine::render::scrollrt::calc_viewport_geometry(ctx);
}

fn game_info(ctx: &mut Ctx) {
    let msg = tr("{:s} {:s}").replacen("{:s}", PROJECT_NAME, 1).replacen("{:s}", PROJECT_VERSION, 1);
    crate::plrmsg::event_plr_msg_style(ctx, &msg, crate::engine::render::text_render::UiFlags::COLOR_WHITE);
}

/// Original: `devilution::InitPadmapActions` (diablo.cpp). The lambda bodies live in
/// `controls::plrctrls`.
// @port diablo.cpp|devilution::InitPadmapActions() sha=b3a4ffc55a91
pub fn init_padmap_actions(ctx: &mut Ctx) {
    use crate::controls::controller_buttons::{ControllerButton as B, ControllerButtonCombo as C};
    use std::rc::Rc;
    let can_take = || -> Option<options::EnableFn> { Some(Rc::new(can_player_take_action)) };
    let pm = &mut ctx.options.padmapper;
    let one = C::single;
    let two = C::new;
    for i in 0..8u32 {
        pm.add_action("BeltItem{}", "Belt item {}", "Use Belt item.", one(B::None), Some(Rc::new(move |ctx: &mut Ctx| crate::inv::use_belt_item_slot(ctx, i as usize))), None, can_take(), i + 1);
    }
    for i in 0..crate::spells::NUM_HOTKEYS as u32 {
        pm.add_action("QuickSpell{}", "Quick spell {}", "Hotkey for skill or spell.", one(B::None), Some(Rc::new(move |ctx: &mut Ctx| crate::control::quick_spell_hotkey(ctx, i as usize))), None, can_take(), i + 1);
    }
    use crate::controls::plrctrls as pc;
    pm.add_action("PrimaryAction", "Primary action", "Attack monsters, talk to towners, lift and place inventory items.", one(B::ButtonB), Some(Rc::new(pc::primary_action_pressed)), Some(Rc::new(pc::controller_action_released)), can_take(), 0);
    pm.add_action("SecondaryAction", "Secondary action", "Open chests, interact with doors, pick up items.", one(B::ButtonY), Some(Rc::new(pc::secondary_action_pressed)), Some(Rc::new(pc::controller_action_released)), can_take(), 0);
    pm.add_action("SpellAction", "Spell action", "Cast the active spell.", one(B::ButtonX), Some(Rc::new(pc::spell_action_pressed)), Some(Rc::new(pc::controller_action_released)), can_take(), 0);
    pm.add_action("CancelAction", "Cancel action", "Close menus.", one(B::ButtonA), Some(Rc::new(pc::cancel_action_pressed)), None, Some(Rc::new(pc::cancel_action_enabled)), 0);
    let noop = || -> Option<options::ActionFn> { Some(Rc::new(|_: &mut Ctx| {})) };
    pm.add_action("MoveUp", "Move up", "Moves the player character up.", one(B::ButtonDpadUp), noop(), None, None, 0);
    pm.add_action("MoveDown", "Move down", "Moves the player character down.", one(B::ButtonDpadDown), noop(), None, None, 0);
    pm.add_action("MoveLeft", "Move left", "Moves the player character left.", one(B::ButtonDpadLeft), noop(), None, None, 0);
    pm.add_action("MoveRight", "Move right", "Moves the player character right.", one(B::ButtonDpadRight), noop(), None, None, 0);
    pm.add_action("StandGround", "Stand ground", "Hold to prevent the player from moving.", one(B::None), noop(), None, None, 0);
    pm.add_action("ToggleStandGround", "Toggle stand ground", "Toggle whether the player moves.", one(B::None), Some(Rc::new(|c: &mut Ctx| c.controls.stand_toggle = !c.controls.stand_toggle)), None, can_take(), 0);
    pm.add_action("UseHealthPotion", "Use health potion", "Use health potions from belt.", one(B::ButtonLeftShoulder), Some(Rc::new(|c: &mut Ctx| crate::inv::use_belt_item(c, crate::inv::BLT_HEALING))), None, can_take(), 0);
    pm.add_action("UseManaPotion", "Use mana potion", "Use mana potions from belt.", one(B::ButtonRightShoulder), Some(Rc::new(|c: &mut Ctx| crate::inv::use_belt_item(c, crate::inv::BLT_MANA))), None, can_take(), 0);
    use crate::controls::game_controls::GameActionType as G;
    let act = |t: G| -> Option<options::ActionFn> { Some(Rc::new(move |c: &mut Ctx| crate::controls::plrctrls::process_game_action(c, t))) };
    pm.add_action("Character", "Character", "Open Character screen.", one(B::AxisTriggerLeft), act(G::ToggleCharacterInfo), None, None, 0);
    pm.add_action("Inventory", "Inventory", "Open Inventory screen.", one(B::AxisTriggerRight), act(G::ToggleInventory), None, can_take(), 0);
    pm.add_action("QuestLog", "Quest log", "Open Quest log.", two(B::ButtonBack, B::AxisTriggerLeft), act(G::ToggleQuestLog), None, can_take(), 0);
    pm.add_action("SpellBook", "Spellbook", "Open Spellbook.", two(B::ButtonBack, B::AxisTriggerRight), act(G::ToggleSpellBook), None, can_take(), 0);
    pm.add_action("DisplaySpells", "Speedbook", "Open Speedbook.", one(B::ButtonA), act(G::ToggleQuickSpellMenu), None, can_take(), 0);
    pm.add_action("Toggle Automap", "Toggle automap", "Toggles if automap is displayed.", one(B::ButtonLeftStick), Some(Rc::new(crate::automap::do_auto_map)), None, None, 0);
    pm.add_action("MouseUp", "Move mouse up", "Simulates upward mouse movement.", two(B::ButtonBack, B::ButtonDpadUp), noop(), None, None, 0);
    pm.add_action("MouseDown", "Move mouse down", "Simulates downward mouse movement.", two(B::ButtonBack, B::ButtonDpadDown), noop(), None, None, 0);
    pm.add_action("MouseLeft", "Move mouse left", "Simulates leftward mouse movement.", two(B::ButtonBack, B::ButtonDpadLeft), noop(), None, None, 0);
    pm.add_action("MouseRight", "Move mouse right", "Simulates rightward mouse movement.", two(B::ButtonBack, B::ButtonDpadRight), noop(), None, None, 0);
    pm.add_action("LeftMouseClick1", "Left mouse click", "Simulates the left mouse button.", one(B::ButtonRightStick), Some(Rc::new(pc::pad_left_mouse_down)), Some(Rc::new(pc::pad_left_mouse_up)), None, 0);
    pm.add_action("LeftMouseClick2", "Left mouse click", "Simulates the left mouse button.", two(B::ButtonBack, B::ButtonLeftShoulder), Some(Rc::new(pc::pad_left_mouse_down)), Some(Rc::new(pc::pad_left_mouse_up)), None, 0);
    pm.add_action("RightMouseClick1", "Right mouse click", "Simulates the right mouse button.", two(B::ButtonBack, B::ButtonRightStick), Some(Rc::new(pc::pad_right_mouse_down)), Some(Rc::new(pc::pad_right_mouse_up)), None, 0);
    pm.add_action("RightMouseClick2", "Right mouse click", "Simulates the right mouse button.", two(B::ButtonBack, B::ButtonRightShoulder), Some(Rc::new(pc::pad_right_mouse_down)), Some(Rc::new(pc::pad_right_mouse_up)), None, 0);
    pm.add_action("PadHotspellMenu", "Gamepad hotspell menu", "Hold to set or use spell hotkeys.", one(B::ButtonBack), Some(Rc::new(|c: &mut Ctx| c.controls.pad_hotspell_menu_active = true)), Some(Rc::new(|c: &mut Ctx| c.controls.pad_hotspell_menu_active = false)), None, 0);
    pm.add_action("PadMenuNavigator", "Gamepad menu navigator", "Hold to access gamepad menu navigation.", one(B::ButtonStart), Some(Rc::new(|c: &mut Ctx| c.controls.pad_menu_navigator_active = true)), Some(Rc::new(|c: &mut Ctx| c.controls.pad_menu_navigator_active = false)), None, 0);
    pm.add_action("ToggleGameMenu1", "Toggle game menu", "Opens the game menu.", two(B::ButtonBack, B::ButtonStart), Some(Rc::new(pc::pad_toggle_game_menu)), None, None, 0);
    pm.add_action("ToggleGameMenu2", "Toggle game menu", "Opens the game menu.", two(B::ButtonStart, B::ButtonBack), Some(Rc::new(pc::pad_toggle_game_menu)), None, None, 0);
    pm.add_action("QuickSave", "Quick save", "Saves the game.", one(B::None), Some(Rc::new(|c: &mut Ctx| crate::gamemenu::gamemenu_save_game(c, false))), None, Some(Rc::new(|c: &Ctx| !c.init.gb_is_multiplayer && can_player_take_action(c))), 0);
    pm.add_action("QuickLoad", "Quick load", "Loads the game.", one(B::None), Some(Rc::new(|c: &mut Ctx| crate::gamemenu::gamemenu_load_game(c, false))), None, Some(Rc::new(|c: &Ctx| !c.init.gb_is_multiplayer && crate::pfile::gb_valid_save_file(c) && crate::stores::stextflag_is_none(c) && is_game_running(c))), 0);
    pm.add_action("Item Highlighting", "Item highlighting", "Show/hide items on ground.", one(B::None), Some(Rc::new(|c: &mut Ctx| crate::qol::itemlabels::highlight_key_pressed(c, true))), Some(Rc::new(|c: &mut Ctx| crate::qol::itemlabels::highlight_key_pressed(c, false))), None, 0);
    pm.add_action("Toggle Item Highlighting", "Toggle item highlighting", "Permanent show/hide items on ground.", one(B::None), None, Some(Rc::new(crate::qol::itemlabels::toggle_item_label_highlight)), None, 0);
    pm.add_action("Hide Info Screens", "Hide Info Screens", "Hide all info screens.", one(B::None), Some(Rc::new(hide_info_screens)), None, Some(Rc::new(is_game_running)), 0);
    pm.add_action("Zoom", "Zoom", "Zoom Game Screen.", one(B::None), Some(Rc::new(toggle_zoom)), None, can_take(), 0);
    pm.add_action("Pause Game", "Pause Game", "Pauses the game.", one(B::None), Some(Rc::new(diablo_pause_game)), None, None, 0);
    pm.add_action("DecreaseGamma", "Decrease Gamma", "Reduce screen brightness.", one(B::None), Some(Rc::new(crate::engine::palette::decrease_gamma)), None, can_take(), 0);
    pm.add_action("IncreaseGamma", "Increase Gamma", "Increase screen brightness.", one(B::None), Some(Rc::new(crate::engine::palette::increase_gamma)), None, can_take(), 0);
    pm.add_action("Help", "Help", "Open Help Screen.", one(B::None), Some(Rc::new(crate::diablo::help_key_pressed)), None, can_take(), 0);
    pm.add_action("Screenshot", "Screenshot", "Takes a screenshot.", one(B::None), None, Some(Rc::new(crate::capture::capture_screen)), None, 0);
    pm.add_action("GameInfo", "Game info", "Displays game infos.", one(B::None), Some(Rc::new(game_info)), None, can_take(), 0);
    pm.add_action("ChatLog", "Chat Log", "Displays chat log.", one(B::None), Some(Rc::new(crate::qol::chatlog::toggle_chat_log)), None, None, 0);
    pm.commit_actions();
}

/// Original: `devilution::diablo_is_focused` (diablo.cpp).
// @port diablo.cpp|devilution::diablo_is_focused() sha=e9cca6dfba07
pub fn diablo_is_focused(ctx: &Ctx) -> bool {
    ctx.platform.has_keyboard_focus()
}

/// Original: `devilution::diablo_focus_pause` (diablo.cpp).
// @port diablo.cpp|devilution::diablo_focus_pause() sha=040f9f3b90e3
pub fn diablo_focus_pause(ctx: &mut Ctx) {
    if !crate::movie::movie_playing(ctx) && (ctx.init.gb_is_multiplayer || ctx.diablo.minimize_paused) {
        return;
    }
    ctx.diablo.game_was_already_paused = ctx.diablo.pause_mode != 0;
    if !ctx.diablo.game_was_already_paused {
        ctx.diablo.pause_mode = 2;
        crate::effects::sound_stop(ctx);
        ctx.diablo.last_mouse_button_action = MouseActionType::None;
    }
    crate::storm::storm_svid::svid_mute(ctx);
    crate::engine::sound::music_mute(ctx);
    ctx.diablo.minimize_paused = true;
}

/// Original: `devilution::diablo_focus_unpause` (diablo.cpp).
// @port diablo.cpp|devilution::diablo_focus_unpause() sha=07197f0d761e
pub fn diablo_focus_unpause(ctx: &mut Ctx) {
    if !ctx.diablo.game_was_already_paused {
        ctx.diablo.pause_mode = 0;
    }
    crate::storm::storm_svid::svid_unmute(ctx);
    crate::engine::sound::music_unmute(ctx);
    ctx.diablo.minimize_paused = false;
}
