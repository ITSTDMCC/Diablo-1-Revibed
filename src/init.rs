//! `Source/init.cpp`: archives and the main window.

use crate::ctx::Ctx;
use crate::engine::assets::{asset_size, find_asset, open_asset_ref, Archives, AssetRef};
use crate::mpq::MpqArchive;
use crate::platform::log;
use crate::utils::language::tr;

#[derive(Default)]
pub struct InitState {
    /// `gbActive`: the game is the active window.
    pub gb_active: bool,
    pub gb_is_spawn: bool,
    pub gb_is_hellfire: bool,
    pub gb_vanilla: bool,
    pub force_hellfire: bool,
    /// `gbIsHellfireSaveGame` (pfile.cpp)
    pub gb_is_hellfire_save_game: bool,
    /// `gbIsMultiplayer` (multi.cpp)
    pub gb_is_multiplayer: bool,
    pub archives: Archives,
}

const EXTRA_FONTS_VERSION: &str = "1\n";

/// Original: `devilution::LoadMPQ` (init.cpp).
// @port init.cpp|devilution::LoadMPQ(const std::vector<std::string> &paths, string_view mpqName) sha=6fe5ebe9f779
fn load_mpq(paths: &[String], mpq_name: &str) -> Option<MpqArchive> {
    let mut had_error = false;
    for path in paths {
        let abs = format!("{path}{mpq_name}");
        match MpqArchive::open(&abs) {
            Ok(Some(a)) => {
                log::verbose!("  Found: {} in {}", mpq_name, path);
                return Some(a);
            }
            Ok(None) => {}
            Err(e) => {
                had_error = true;
                log::error!("Error {}: {}", e, abs);
            }
        }
    }
    if !had_error {
        log::verbose!("Missing: {}", mpq_name);
    }
    None
}

/// Original: `devilution::GetMPQSearchPaths` (init.cpp). The GOG install lookup
/// (`fsg_get_gog_game_path`, registry) is not ported: it only adds search paths when a GOG copy
/// of Diablo is installed; use `--data-dir` instead (known difference).
// @port init.cpp|devilution::GetMPQSearchPaths() sha=645427a4725e
pub fn get_mpq_search_paths(ctx: &mut Ctx) -> Vec<String> {
    let mut paths = vec![ctx.paths.base_path().to_string(), ctx.paths.pref_path().to_string()];
    if paths[0] == paths[1] {
        paths.pop();
    }
    paths.push(ctx.paths.config_path().to_string());
    if paths[0] == paths[1] || (paths.len() == 3 && (paths[0] == paths[2] || paths[1] == paths[2])) {
        paths.pop();
    }
    paths.push(String::new()); // PWD
    log::verbose!(
        "Paths:\n    base: {}\n    pref: {}\n  config: {}\n  assets: {}",
        ctx.paths.base_path().to_string(),
        ctx.paths.pref_path().to_string(),
        ctx.paths.config_path().to_string(),
        ctx.paths.assets_path().to_string()
    );
    paths
}

/// Original: `devilution::CheckExtraFontsVersion` (init.cpp).
// @port init.cpp|devilution::CheckExtraFontsVersion(AssetRef &&ref) sha=acb6a60b9750
fn check_extra_fonts_version(ctx: &mut Ctx, r: AssetRef) -> bool {
    let size = asset_size(ctx, &r);
    let mut handle = open_asset_ref(ctx, r);
    if !handle.ok() {
        return true;
    }
    let mut contents = vec![0u8; size];
    if !handle.read(&mut contents) {
        return true;
    }
    contents != EXTRA_FONTS_VERSION.as_bytes()
}

/// Original: `devilution::AreExtraFontsOutOfDate()` (init.h): `font_mpq && AreExtraFontsOutOfDate(*font_mpq)`.
// @port init.h|devilution::AreExtraFontsOutOfDate() sha=8a90eb765465
pub fn are_extra_fonts_out_of_date(ctx: &mut Ctx) -> bool {
    ctx.init.archives.font_mpq.is_some() && are_extra_fonts_out_of_date_archive(ctx)
}

/// Original: `devilution::AreExtraFontsOutOfDate(MpqArchive &)` (init.cpp), for the loaded
/// fonts.mpq.
// @port init.cpp|devilution::AreExtraFontsOutOfDate(MpqArchive &archive) sha=39a45c471f4e
fn are_extra_fonts_out_of_date_archive(ctx: &mut Ctx) -> bool {
    let filename = "fonts\\VERSION";
    let hash = MpqArchive::calculate_file_hash(filename);
    let Some(file_number) = ctx.init.archives.font_mpq.as_ref().and_then(|a| a.get_file_number(hash)) else {
        return true;
    };
    let r = AssetRef::Mpq { archive: crate::engine::assets::ArchiveId::Font, file_number, filename: filename.to_string() };
    check_extra_fonts_version(ctx, r)
}

/// Original: `devilution::init_cleanup` (init.cpp).
// @port init.cpp|devilution::init_cleanup() sha=5a8c9df9e635
pub fn init_cleanup(ctx: &mut Ctx) {
    if ctx.init.gb_is_multiplayer && ctx.diablo.gb_run_game {
        crate::pfile::pfile_write_hero(ctx, false);
        crate::pfile::sfile_write_stash(ctx);
    }
    ctx.init.archives = Archives::default();
    crate::multi::net_close(ctx);
}

/// Original: `devilution::LoadCoreArchives` (init.cpp).
// @port init.cpp|devilution::LoadCoreArchives() sha=7feb8d207a9a
pub fn load_core_archives(ctx: &mut Ctx) {
    let paths = get_mpq_search_paths(ctx);
    // Load devilutionx.mpq first to get the font file for error messages
    ctx.init.archives.devilutionx_mpq = load_mpq(&paths, "devilutionx.mpq");
    ctx.init.archives.font_mpq = load_mpq(&paths, "fonts.mpq"); // Extra fonts
}

/// Original: `devilution::LoadLanguageArchive` (init.cpp).
// @port init.cpp|devilution::LoadLanguageArchive() sha=278631069c4d
pub fn load_language_archive(ctx: &mut Ctx) {
    ctx.init.archives.lang_mpq = None;
    let code = crate::utils::language::get_language_code(ctx);
    if code != "en" {
        let name = format!("{code}.mpq");
        let paths = get_mpq_search_paths(ctx);
        ctx.init.archives.lang_mpq = load_mpq(&paths, &name);
    }
}

/// Original: `devilution::LoadGameArchives` (init.cpp).
// @port init.cpp|devilution::LoadGameArchives() sha=690a456e7972
pub fn load_game_archives(ctx: &mut Ctx) {
    let paths = get_mpq_search_paths(ctx);
    let a = &mut ctx.init.archives;
    a.diabdat_mpq = load_mpq(&paths, "DIABDAT.MPQ");
    if a.diabdat_mpq.is_none() {
        // DIABDAT.MPQ is uppercase on the original CD and the GOG version.
        a.diabdat_mpq = load_mpq(&paths, "diabdat.mpq");
    }
    if a.diabdat_mpq.is_none() {
        a.spawn_mpq = load_mpq(&paths, "spawn.mpq");
        if a.spawn_mpq.is_some() {
            ctx.init.gb_is_spawn = true;
        }
    }
    if !ctx.diablo.headless_mode {
        let r = find_asset(ctx, "ui_art\\title.pcx");
        if !r.ok() {
            log::error!("File not found");
            crate::appfat::insert_cd_dlg(ctx, &tr("diabdat.mpq or spawn.mpq"));
        }
    }
    let a = &mut ctx.init.archives;
    a.hellfire_mpq = load_mpq(&paths, "hellfire.mpq");
    if a.hellfire_mpq.is_some() {
        ctx.init.gb_is_hellfire = true;
    }
    if ctx.init.force_hellfire && ctx.init.archives.hellfire_mpq.is_none() {
        crate::appfat::insert_cd_dlg(ctx, "hellfire.mpq");
    }
    let a = &mut ctx.init.archives;
    a.hfmonk_mpq = load_mpq(&paths, "hfmonk.mpq");
    a.hfbard_mpq = load_mpq(&paths, "hfbard.mpq");
    if a.hfbard_mpq.is_some() {
        ctx.diablo.gb_bard = true;
    }
    a.hfbarb_mpq = load_mpq(&paths, "hfbarb.mpq");
    if a.hfbarb_mpq.is_some() {
        ctx.diablo.gb_barbarian = true;
    }
    a.hfmusic_mpq = load_mpq(&paths, "hfmusic.mpq");
    a.hfvoice_mpq = load_mpq(&paths, "hfvoice.mpq");
    if ctx.init.gb_is_hellfire && (a.hfmonk_mpq.is_none() || a.hfmusic_mpq.is_none() || a.hfvoice_mpq.is_none()) {
        crate::diablo_ui::dialogs::ui_error_ok_dialog(
            ctx,
            &tr("Some Hellfire MPQs are missing"),
            &tr("Not all Hellfire MPQs were found.\nPlease copy all the hf*.mpq files."),
            true,
        );
        crate::diablo::diablo_quit(ctx, 1);
    }
}

/// Original: `devilution::init_create_window` (init.cpp).
// @port init.cpp|devilution::init_create_window() sha=f22c6cbbf437
pub fn init_create_window(ctx: &mut Ctx) {
    if !crate::utils::display::spawn_window(ctx, crate::diablo::PROJECT_NAME) {
        crate::appfat::app_fatal(ctx, &tr("Unable to create main window"));
    }
    crate::engine::dx::dx_init(ctx);
    ctx.init.gb_active = true;
    // SDL_DisableScreenSaver: winit keeps the screen awake while the window has focus.
}

crate::pending_fn!(pub fn main_wnd_proc(ctx: &mut crate::ctx::Ctx, event: &crate::platform::events::Event), "init.cpp|devilution::MainWndProc(const SDL_Event &event)");
