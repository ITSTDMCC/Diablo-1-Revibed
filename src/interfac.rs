//! `Source/interfac.cpp`: load screens (the cutscene backdrop and progress bar shown while a
//! level loads).

use crate::ctx::Ctx;
use crate::engine::clx_sprite::ClxSpriteList;
use crate::engine::surface::Rect;
use crate::enums::*;
use crate::levels::gendung::{get_level_type, DungeonType};
use crate::platform::events::Event;

const MAX_PROGRESS: u32 = 534;

/// The color used for the progress bar as an index into the palette.
const BAR_COLOR: [u8; 3] = [138, 43, 254];
/// The screen position of the top left corner of the progress bar.
const BAR_POS: [[i32; 2]; 3] = [[53, 37], [53, 421], [53, 37]];

/// Globals of interfac.cpp.
#[derive(Default)]
pub struct InterfacState {
    /// `sgpBackCel`
    sgp_back_cel: Option<ClxSpriteList>,
    /// `IsProgress`
    is_progress: bool,
    /// `sgdwProgress`
    sgdw_progress: u32,
    /// `progress_id`
    progress_id: usize,
    /// `ArtCutsceneWidescreen`
    art_cutscene_widescreen: Option<ClxSpriteList>,
}

/// Original: `GetCutSceneFromLevelType` (interfac.cpp).
// @port interfac.cpp|devilution::GetCutSceneFromLevelType(dungeon_type type) sha=5ee353482535
fn get_cut_scene_from_level_type(type_: DungeonType) -> Cutscenes {
    match type_ {
        DungeonType::Town => CutTown,
        DungeonType::Cathedral => CutLevel1,
        DungeonType::Catacombs => CutLevel2,
        DungeonType::Caves => CutLevel3,
        DungeonType::Hell => CutLevel4,
        DungeonType::Nest => CutLevel6,
        DungeonType::Crypt => CutLevel5,
        _ => CutLevel1,
    }
}

/// Original: `PickCutscene` (interfac.cpp).
// @port interfac.cpp|devilution::PickCutscene(interface_mode uMsg) sha=ed2b1325f0fb
fn pick_cutscene(ctx: &mut Ctx, u_msg: interface_mode) -> Cutscenes {
    match u_msg {
        WM_DIABLOADGAME | WM_DIABNEWGAME => CutStart,
        WM_DIABRETOWN => CutTown,
        WM_DIABNEXTLVL | WM_DIABPREVLVL | WM_DIABTOWNWARP | WM_DIABTWARPUP => {
            let me = ctx.players.MyPlayer.expect("MyPlayer");
            let lvl = ctx.players.Players[me].plrlevel as i32;
            if lvl == 1 && u_msg == WM_DIABNEXTLVL {
                return CutTown;
            }
            if lvl == 16 && u_msg == WM_DIABNEXTLVL {
                return CutGate;
            }
            get_cut_scene_from_level_type(get_level_type(lvl))
        }
        WM_DIABWARPLVL => CutPortal,
        WM_DIABSETLVL | WM_DIABRTNLVL => {
            let n = ctx.gendung.setlvlnum;
            if n == SL_BONECHAMB {
                return CutLevel2;
            }
            if n == SL_VILEBETRAYER {
                return CutPortalRed;
            }
            if crate::levels::gendung::is_arena_level(n) {
                if u_msg == WM_DIABSETLVL {
                    return get_cut_scene_from_level_type(ctx.gendung.setlvltype);
                }
                return CutTown;
            }
            CutLevel1
        }
        _ => crate::appfat::app_fatal(ctx, "Unknown progress mode"),
    }
}

/// Original: `LoadCutsceneBackground` (interfac.cpp).
// @port interfac.cpp|devilution::LoadCutsceneBackground(interface_mode uMsg) sha=2122060a6ca9
fn load_cutscene_background(ctx: &mut Ctx, u_msg: interface_mode) {
    let (wide, cel_path, pal_path, progress_id) = match pick_cutscene(ctx, u_msg) {
        CutStart => ("gendata\\cutstartw.clx", "gendata\\cutstart", "gendata\\cutstart.pal", 1),
        CutTown => ("gendata\\cutttw.clx", "gendata\\cuttt", "gendata\\cuttt.pal", 1),
        CutLevel1 => ("gendata\\cutl1dw.clx", "gendata\\cutl1d", "gendata\\cutl1d.pal", 0),
        CutLevel2 => ("gendata\\cut2w.clx", "gendata\\cut2", "gendata\\cut2.pal", 2),
        CutLevel3 => ("gendata\\cut3w.clx", "gendata\\cut3", "gendata\\cut3.pal", 1),
        CutLevel4 => ("gendata\\cut4w.clx", "gendata\\cut4", "gendata\\cut4.pal", 1),
        CutLevel5 => ("nlevels\\cutl5w.clx", "nlevels\\cutl5", "nlevels\\cutl5.pal", 1),
        CutLevel6 => ("nlevels\\cutl6w.clx", "nlevels\\cutl6", "nlevels\\cutl6.pal", 1),
        CutPortal => ("gendata\\cutportlw.clx", "gendata\\cutportl", "gendata\\cutportl.pal", 1),
        CutPortalRed => ("gendata\\cutportrw.clx", "gendata\\cutportr", "gendata\\cutportr.pal", 1),
        _ => ("gendata\\cutgatew.clx", "gendata\\cutgate", "gendata\\cutgate.pal", 1),
    };
    ctx.interfac.art_cutscene_widescreen = crate::engine::load_sprites::load_optional_clx(ctx, wide);
    ctx.interfac.progress_id = progress_id;
    assert!(ctx.interfac.sgp_back_cel.is_none());
    ctx.interfac.sgp_back_cel = Some(crate::engine::load_sprites::load_cel(ctx, cel_path, 640));
    crate::engine::palette::load_palette(ctx, pal_path, true);
    ctx.interfac.sgdw_progress = 0;
}

/// Original: `FreeCutsceneBackground` (interfac.cpp).
// @port interfac.cpp|devilution::FreeCutsceneBackground() sha=6c04ddf273b7
fn free_cutscene_background(ctx: &mut Ctx) {
    ctx.interfac.sgp_back_cel = None;
    ctx.interfac.art_cutscene_widescreen = None;
}

/// `SDL_FillRect` on the 8-bit back buffer (`rect` in surface coordinates, clipped).
fn fill_rect(ctx: &mut Ctx, rect: Option<Rect>, color: u8) {
    let Some(s) = ctx.dx.pal_surface.as_mut() else { return };
    let r = rect.unwrap_or(Rect::new(0, 0, s.w, s.h));
    let x0 = r.x.max(0);
    let y0 = r.y.max(0);
    let x1 = (r.x + r.w).min(s.w);
    let y1 = (r.y + r.h).min(s.h);
    for y in y0..y1 {
        let row = (y * s.pitch) as usize;
        s.pixels[row + x0 as usize..row + x1.max(x0) as usize].fill(color);
    }
}

/// Original: `DrawCutsceneBackground` (interfac.cpp).
// @port interfac.cpp|devilution::DrawCutsceneBackground() sha=300932b1f198
fn draw_cutscene_background(ctx: &mut Ctx) {
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    let out = crate::engine::dx::global_back_buffer(ctx);
    fill_rect(ctx, None, 0);
    if let Some(wide) = ctx.interfac.art_cutscene_widescreen.as_ref() {
        let sprite = wide.get(0);
        crate::engine::render::text_render::render_clx_sprite(&out, &sprite, (ui.x - (sprite.width() as i32 - ui.w) / 2, ui.y));
    }
    let cel = ctx.interfac.sgp_back_cel.as_ref().expect("sgpBackCel").get(0);
    crate::engine::render::clx_render::clx_draw(&out, (ui.x, 480 - 1 + ui.y), &cel);
}

/// Original: `DrawCutsceneForeground` (interfac.cpp).
// @port interfac.cpp|devilution::DrawCutsceneForeground() sha=9240c9e04095
fn draw_cutscene_foreground(ctx: &mut Ctx) {
    let ui = crate::utils::display::get_ui_rectangle(ctx);
    let out = crate::engine::dx::global_back_buffer(ctx);
    const PROGRESS_HEIGHT: i32 = 22;
    let pid = ctx.interfac.progress_id;
    let rect = Rect::new(
        out.region.x + BAR_POS[pid][0] + ui.x,
        out.region.y + BAR_POS[pid][1] + ui.y,
        ctx.interfac.sgdw_progress as i32,
        PROGRESS_HEIGHT,
    );
    fill_rect(ctx, Some(rect), BAR_COLOR[pid]);
    // DiabloUiSurface() is PalSurface in SDL2 builds.
    crate::engine::dx::blt_fast(ctx, Some(rect), Some(rect));
    crate::engine::dx::render_present(ctx);
}

/// Original: `devilution::interface_msg_pump` (interfac.cpp).
// @port interfac.cpp|devilution::interface_msg_pump() sha=5eeb37fedeb0
pub fn interface_msg_pump(ctx: &mut Ctx) {
    while let Some((event, mod_state)) = crate::engine::events::fetch_message(ctx) {
        if let Some(e) = event {
            if !matches!(e, Event::Quit) {
                crate::engine::events::handle_message(ctx, &e, mod_state);
            }
        }
    }
}

/// Original: `devilution::IncProgress` (interfac.cpp).
// @port interfac.cpp|devilution::IncProgress() sha=2d12429a2345
pub fn inc_progress(ctx: &mut Ctx) {
    let headless = ctx.diablo.headless_mode;
    if !headless && !crate::engine::demomode::is_running(ctx) {
        interface_msg_pump(ctx);
    }
    if !ctx.interfac.is_progress {
        return;
    }
    ctx.interfac.sgdw_progress += 23;
    if ctx.interfac.sgdw_progress > MAX_PROGRESS {
        ctx.interfac.sgdw_progress = MAX_PROGRESS;
    }
    if !headless && !crate::engine::demomode::is_running(ctx) {
        draw_cutscene_foreground(ctx);
    }
}

/// Original: `devilution::CompleteProgress` (interfac.cpp).
// @port interfac.cpp|devilution::CompleteProgress() sha=c2ac725d78a7
pub fn complete_progress(ctx: &mut Ctx) {
    if ctx.diablo.headless_mode {
        return;
    }
    if !ctx.interfac.is_progress {
        return;
    }
    while ctx.interfac.sgdw_progress < MAX_PROGRESS {
        inc_progress(ctx);
    }
}

/// The level change shared by most `ShowProgress` cases: save the level being left, free it
/// and load the new one.
fn save_level_left(ctx: &mut Ctx) {
    inc_progress(ctx);
    if !ctx.init.gb_is_multiplayer {
        crate::pfile::pfile_save_level(ctx);
    } else {
        crate::msg::delta_save_level(ctx);
    }
    inc_progress(ctx);
}

/// Original: `devilution::ShowProgress` (interfac.cpp).
// @port interfac.cpp|devilution::ShowProgress(interface_mode uMsg) sha=378b091c3a2a
pub fn show_progress(ctx: &mut Ctx, u_msg: interface_mode) {
    ctx.interfac.is_progress = true;
    ctx.multi.gbSomebodyWonGameKludge = false;
    crate::plrmsg::plrmsg_delay(ctx, true);

    let mut previous_handler = crate::engine::events::set_event_handler(ctx, Some(crate::diablo::disable_input_event_handler));

    if !ctx.diablo.headless_mode {
        interface_msg_pump(ctx);
        crate::engine::render::scrollrt::clear_screen_buffer(ctx);
        crate::engine::render::scrollrt::scrollrt_draw_game_screen(ctx);
        if crate::hwcursor::is_hardware_cursor(ctx) {
            crate::hwcursor::set_hardware_cursor_visible(ctx, false);
        }
        crate::engine::palette::black_palette(ctx);
        // Blit the background once and then free it.
        load_cutscene_background(ctx, u_msg);
        draw_cutscene_background(ctx);
        // RenderDirectlyToOutputSurface is never set in this port.
        free_cutscene_background(ctx);
        crate::engine::palette::palette_fade_in(ctx, 8);
        inc_progress(ctx);
        crate::effects::sound_init(ctx);
        inc_progress(ctx);
    }

    let me = ctx.players.MyPlayer.expect("MyPlayer");
    match u_msg {
        WM_DIABLOADGAME => {
            inc_progress(ctx);
            inc_progress(ctx);
            crate::loadsave::load_game(ctx, true);
            inc_progress(ctx);
            inc_progress(ctx);
        }
        WM_DIABNEWGAME => {
            ctx.players.Players[me].pOriginalCathedral = !ctx.init.gb_is_hellfire;
            inc_progress(ctx);
            crate::diablo::free_game_mem(ctx);
            inc_progress(ctx);
            crate::pfile::pfile_remove_temp_files(ctx);
            inc_progress(ctx);
            crate::diablo::load_game_level(ctx, true, ENTRY_MAIN);
            inc_progress(ctx);
        }
        WM_DIABNEXTLVL => {
            save_level_left(ctx);
            crate::diablo::free_game_mem(ctx);
            ctx.gendung.setlevel = false;
            ctx.gendung.currlevel = ctx.players.Players[me].plrlevel;
            ctx.gendung.leveltype = get_level_type(ctx.gendung.currlevel as i32);
            inc_progress(ctx);
            crate::diablo::load_game_level(ctx, false, ENTRY_MAIN);
            inc_progress(ctx);
        }
        WM_DIABPREVLVL => {
            save_level_left(ctx);
            crate::diablo::free_game_mem(ctx);
            ctx.gendung.currlevel = ctx.gendung.currlevel.wrapping_sub(1);
            ctx.gendung.leveltype = get_level_type(ctx.gendung.currlevel as i32);
            assert!(crate::player::is_on_active_level(ctx, me));
            inc_progress(ctx);
            crate::diablo::load_game_level(ctx, false, ENTRY_PREV);
            inc_progress(ctx);
        }
        WM_DIABSETLVL => {
            // Note: ReturnLevel, ReturnLevelType and ReturnLvlPosition is only set to ensure vanilla compatibility
            ctx.quests.ReturnLevel = crate::quests::get_map_return_level(ctx);
            ctx.quests.ReturnLevelType = get_level_type(ctx.quests.ReturnLevel);
            ctx.quests.ReturnLvlPosition = crate::quests::get_map_return_position(ctx);
            save_level_left(ctx);
            ctx.gendung.setlevel = true;
            ctx.gendung.leveltype = ctx.gendung.setlvltype;
            ctx.gendung.currlevel = ctx.gendung.setlvlnum as u8;
            crate::diablo::free_game_mem(ctx);
            inc_progress(ctx);
            crate::diablo::load_game_level(ctx, false, ENTRY_SETLVL);
            inc_progress(ctx);
        }
        WM_DIABRTNLVL => {
            save_level_left(ctx);
            ctx.gendung.setlevel = false;
            crate::diablo::free_game_mem(ctx);
            inc_progress(ctx);
            ctx.gendung.currlevel = crate::quests::get_map_return_level(ctx) as u8;
            ctx.gendung.leveltype = get_level_type(ctx.gendung.currlevel as i32);
            crate::diablo::load_game_level(ctx, false, ENTRY_RTNLVL);
            inc_progress(ctx);
        }
        WM_DIABWARPLVL => {
            save_level_left(ctx);
            crate::diablo::free_game_mem(ctx);
            crate::portal::get_portal_level(ctx);
            inc_progress(ctx);
            crate::diablo::load_game_level(ctx, false, ENTRY_WARPLVL);
            inc_progress(ctx);
        }
        WM_DIABTOWNWARP => {
            save_level_left(ctx);
            crate::diablo::free_game_mem(ctx);
            ctx.gendung.setlevel = false;
            ctx.gendung.currlevel = ctx.players.Players[me].plrlevel;
            ctx.gendung.leveltype = get_level_type(ctx.gendung.currlevel as i32);
            inc_progress(ctx);
            crate::diablo::load_game_level(ctx, false, ENTRY_TWARPDN);
            inc_progress(ctx);
        }
        WM_DIABTWARPUP => {
            save_level_left(ctx);
            crate::diablo::free_game_mem(ctx);
            ctx.gendung.currlevel = ctx.players.Players[me].plrlevel;
            ctx.gendung.leveltype = get_level_type(ctx.gendung.currlevel as i32);
            inc_progress(ctx);
            crate::diablo::load_game_level(ctx, false, ENTRY_TWARPUP);
            inc_progress(ctx);
        }
        WM_DIABRETOWN => {
            save_level_left(ctx);
            crate::diablo::free_game_mem(ctx);
            ctx.gendung.setlevel = false;
            ctx.gendung.currlevel = ctx.players.Players[me].plrlevel;
            ctx.gendung.leveltype = get_level_type(ctx.gendung.currlevel as i32);
            inc_progress(ctx);
            crate::diablo::load_game_level(ctx, false, ENTRY_MAIN);
            inc_progress(ctx);
        }
        _ => {}
    }

    if !ctx.diablo.headless_mode {
        crate::engine::palette::palette_fade_out(ctx, 8);
    }

    previous_handler = crate::engine::events::set_event_handler(ctx, previous_handler);
    assert!(previous_handler == Some(crate::diablo::disable_input_event_handler as crate::engine::events::EventHandler));
    ctx.interfac.is_progress = false;

    let p = &ctx.players.Players[me];
    let (tile, lvl, set) = (p.position.tile, p.plrlevel, p.plrIsOnSetLevel);
    crate::msg::net_send_cmd_loc_param2(ctx, true, CMD_PLAYER_JOINLEVEL, tile, lvl as u16, if set { 1 } else { 0 });
    crate::plrmsg::plrmsg_delay(ctx, false);

    if ctx.multi.gbSomebodyWonGameKludge && ctx.players.Players[me].is_on_level(16) {
        crate::monster::prep_do_ending(ctx);
    }
    ctx.multi.gbSomebodyWonGameKludge = false;
}
