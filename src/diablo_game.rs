//! `Source/diablo.cpp` (continued): the in-game loop, input handling and level loading.

use crate::ctx::Ctx;
use crate::cursor::*;
use crate::diablo::MouseActionType;
use crate::engine::geometry::{Displacement, Point};
use crate::enums::*;
use crate::levels::gendung::{DungeonType, DMAXX, DMAXY, MAXDUNX, MAXDUNY};
use crate::enums::DungeonFlag;
use crate::platform::events::{keys::*, Event, BUTTON_LEFT, BUTTON_RIGHT, BUTTON_X1, KMOD_ALT, KMOD_CTRL, KMOD_SHIFT};

fn mouse(ctx: &Ctx) -> Point {
    Point::new(ctx.diablo.mouse_position.0, ctx.diablo.mouse_position.1)
}

fn me(ctx: &Ctx) -> usize {
    ctx.players.MyPlayer.expect("MyPlayer")
}

fn main_panel_y(ctx: &Ctx) -> i32 {
    crate::control::get_main_panel(ctx).y
}

/// Original: `StartGame(interface_mode uMsg)` (diablo.cpp).
// @port diablo.cpp|devilution::StartGame(interface_mode uMsg) sha=b129839b3bd5
fn start_game_ui(ctx: &mut Ctx, u_msg: interface_mode) {
    crate::engine::render::scrollrt::calc_viewport_geometry(ctx);
    ctx.diablo.cineflag = false;
    crate::cursor::init_cursor(ctx);
    crate::engine::sound::music_stop(ctx);
    crate::qol::monhealthbar::init_monster_health_bar(ctx);
    crate::qol::xpbar::init_xp_bar(ctx);
    crate::interfac::show_progress(ctx, u_msg);
    crate::gmenu::gmenu_init_menu(ctx);
    crate::cursor::init_level_cursor(ctx);
    ctx.diablo.sgn_timeout_curs = CURSOR_NONE;
    ctx.diablo.sgb_mouse_down = CLICK_NONE;
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
}

/// Original: `FreeGame` (diablo.cpp).
// @port diablo.cpp|devilution::FreeGame() sha=948d83071741
fn free_game(ctx: &mut Ctx) {
    crate::qol::monhealthbar::free_monster_health_bar(ctx);
    crate::qol::xpbar::free_xp_bar(ctx);
    crate::control::free_control_pan(ctx);
    crate::inv::free_inv_gfx(ctx);
    crate::gmenu::free_gmenu(ctx);
    crate::minitext::free_quest_text(ctx);
    crate::panels::info_box::free_info_box_gfx(ctx);
    crate::stores::free_store_mem(ctx);
    for pnum in 0..ctx.players.Players.len() {
        crate::player::reset_player_gfx(ctx, pnum);
    }
    crate::cursor::free_cursor(ctx);
    crate::diablo::free_game_mem(ctx);
    crate::effects::stream_stop(ctx);
    crate::engine::sound::music_stop(ctx);
}

/// Original: `ProcessInput` (diablo.cpp).
// @port diablo.cpp|devilution::ProcessInput() sha=bc03d2b2a282
fn process_input(ctx: &mut Ctx) -> bool {
    if ctx.diablo.pause_mode == 2 {
        return false;
    }
    crate::controls::plrctrls::plrctrls_every_frame(ctx);
    if !ctx.init.gb_is_multiplayer && crate::gmenu::gmenu_is_active(ctx) {
        crate::engine::backbuffer_state::redraw_viewport(ctx);
        return false;
    }
    if !crate::gmenu::gmenu_is_active(ctx) && ctx.diablo.sgn_timeout_curs == CURSOR_NONE {
        crate::cursor::check_curs_move(ctx);
        crate::controls::plrctrls::plrctrls_after_check_curs_move(ctx);
        crate::track::repeat_mouse_action(ctx);
    }
    true
}

/// Original: `LeftMouseCmd` (diablo.cpp).
// @port diablo.cpp|devilution::LeftMouseCmd(bool bShift) sha=916e87e5914e
fn left_mouse_cmd(ctx: &mut Ctx, b_shift: bool) {
    use crate::msg::{net_send_cmd_loc, net_send_cmd_loc_param1, net_send_cmd_param1};
    let my_id = ctx.players.MyPlayerId;
    let curs = ctx.cursor.cursPosition;
    let (pcursitem, pcursmonst, pcursplr, pcurs) = (ctx.cursor.pcursitem, ctx.cursor.pcursmonst, ctx.cursor.pcursplr, ctx.cursor.pcurs);
    if ctx.gendung.leveltype == DungeonType::Town {
        crate::qol::stash::close_gold_withdraw(ctx);
        crate::inv::close_stash(ctx);
        if pcursitem != -1 && pcurs == CURSOR_HAND {
            let cmd = if ctx.inv.invflag { CMD_GOTOGETITEM } else { CMD_GOTOAGETITEM };
            net_send_cmd_loc_param1(ctx, true, cmd, curs, pcursitem as u16);
        }
        if pcursmonst != -1 {
            net_send_cmd_loc_param1(ctx, true, CMD_TALKXY, curs, pcursmonst as u16);
        }
        if pcursitem == -1 && pcursmonst == -1 && pcursplr == -1 {
            ctx.diablo.last_mouse_button_action = MouseActionType::Walk;
            net_send_cmd_loc(ctx, my_id, true, CMD_WALKXY, curs);
        }
        return;
    }

    let me = me(ctx);
    let b_near = ctx.players.Players[me].position.tile.walking_distance(curs) < 2;
    let object = ctx.cursor.ObjectUnderCursor;
    let disable = ctx.options.gameplay.disable_crippling_shrines.get();
    let object_ok = object.map(|oi| {
        let o = &ctx.objects.Objects[oi];
        !o.is_disabled_opt(disable) && (!b_shift || (b_near && o._oBreak == 1))
    });
    let friendly = ctx.players.Players[me].friendlyMode;
    if pcursitem != -1 && pcurs == CURSOR_HAND && !b_shift {
        let cmd = if ctx.inv.invflag { CMD_GOTOGETITEM } else { CMD_GOTOAGETITEM };
        net_send_cmd_loc_param1(ctx, true, cmd, curs, pcursitem as u16);
    } else if object_ok == Some(true) {
        ctx.diablo.last_mouse_button_action = MouseActionType::OperateObject;
        let cmd = if pcurs == CURSOR_DISARM { CMD_DISARMXY } else { CMD_OPOBJXY };
        net_send_cmd_loc(ctx, my_id, true, cmd, curs);
    } else if ctx.players.Players[me].uses_ranged_weapon() {
        if b_shift {
            ctx.diablo.last_mouse_button_action = MouseActionType::Attack;
            net_send_cmd_loc(ctx, my_id, true, CMD_RATTACKXY, curs);
        } else if pcursmonst != -1 {
            if crate::monster::can_talk_to_monst(ctx, pcursmonst as usize) {
                net_send_cmd_param1(ctx, true, CMD_ATTACKID, pcursmonst as u16);
            } else {
                ctx.diablo.last_mouse_button_action = MouseActionType::AttackMonsterTarget;
                net_send_cmd_param1(ctx, true, CMD_RATTACKID, pcursmonst as u16);
            }
        } else if pcursplr != -1 && !friendly {
            ctx.diablo.last_mouse_button_action = MouseActionType::AttackPlayerTarget;
            net_send_cmd_param1(ctx, true, CMD_RATTACKPID, pcursplr as u16);
        }
    } else if b_shift {
        if pcursmonst != -1 && crate::monster::can_talk_to_monst(ctx, pcursmonst as usize) {
            net_send_cmd_param1(ctx, true, CMD_ATTACKID, pcursmonst as u16);
        } else {
            ctx.diablo.last_mouse_button_action = MouseActionType::Attack;
            net_send_cmd_loc(ctx, my_id, true, CMD_SATTACKXY, curs);
        }
    } else if pcursmonst != -1 {
        ctx.diablo.last_mouse_button_action = MouseActionType::AttackMonsterTarget;
        net_send_cmd_param1(ctx, true, CMD_ATTACKID, pcursmonst as u16);
    } else if pcursplr != -1 && !friendly {
        ctx.diablo.last_mouse_button_action = MouseActionType::AttackPlayerTarget;
        net_send_cmd_param1(ctx, true, CMD_ATTACKPID, pcursplr as u16);
    }
    if !b_shift && pcursitem == -1 && object.is_none() && pcursmonst == -1 && pcursplr == -1 {
        ctx.diablo.last_mouse_button_action = MouseActionType::Walk;
        net_send_cmd_loc(ctx, my_id, true, CMD_WALKXY, curs);
    }
}

/// Original: `TryOpenDungeonWithMouse` (diablo.cpp).
// @port diablo.cpp|devilution::TryOpenDungeonWithMouse() sha=a8f78e6f33a0
fn try_open_dungeon_with_mouse(ctx: &mut Ctx) -> bool {
    if ctx.gendung.leveltype != DungeonType::Town {
        return false;
    }
    let idx = ctx.players.Players[me(ctx)].HoldItem.IDidx;
    let curs = ctx.cursor.cursPosition;
    if idx == IDI_RUNEBOMB && crate::levels::town::opens_hive(curs) {
        crate::levels::town::open_hive(ctx);
    } else if idx == IDI_MAPOFDOOM && crate::levels::town::opens_grave(curs) {
        crate::levels::town::open_grave(ctx);
    } else {
        return false;
    }
    new_cursor(ctx, CURSOR_HAND);
    true
}

/// Original: `LeftMouseDown` (diablo.cpp).
// @port diablo.cpp|devilution::LeftMouseDown(uint16_t modState) sha=073813f6a9fc
pub(crate) fn left_mouse_down(ctx: &mut Ctx, mod_state: u16) {
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
    if crate::gmenu::gmenu_left_mouse(ctx, true) {
        return;
    }
    if crate::control::control_check_talk_btn(ctx) {
        return;
    }
    if ctx.diablo.sgn_timeout_curs != CURSOR_NONE {
        return;
    }
    if ctx.players.MyPlayerIsDead {
        crate::control::control_check_btn_press(ctx);
        return;
    }
    if ctx.diablo.pause_mode == 2 {
        return;
    }
    if ctx.doom.DoomFlag {
        crate::doom::doom_close(ctx);
        return;
    }
    if ctx.control.spselflag {
        crate::panels::spell_list::set_spell(ctx);
        return;
    }
    if !crate::stores::stextflag_is_none(ctx) {
        crate::stores::check_store_btn(ctx);
        return;
    }
    let is_shift_held = (mod_state & KMOD_SHIFT) != 0;
    let is_ctrl_held = (mod_state & KMOD_CTRL) != 0;
    let m = mouse(ctx);
    if !crate::control::get_main_panel(ctx).contains(m) {
        if !crate::gmenu::gmenu_is_active(ctx) && !try_icon_curs(ctx) {
            let lp = crate::control::get_left_panel(ctx).contains(m);
            let rp = crate::control::get_right_panel(ctx).contains(m);
            if ctx.quests.QuestLogIsOpen && lp {
                crate::quests::questlog_esc(ctx);
            } else if crate::minitext::qtextflag(ctx) {
                crate::minitext::set_qtextflag(ctx, false);
                crate::effects::stream_stop(ctx);
            } else if ctx.control.chrflag && lp {
                crate::control::check_chr_btns(ctx);
            } else if ctx.inv.invflag && rp {
                if !ctx.control.drop_gold_flag {
                    crate::inv::check_inv_item(ctx, is_shift_held, is_ctrl_held);
                }
            } else if ctx.stash.IsStashOpen && lp {
                if !ctx.stash.IsWithdrawGoldOpen {
                    crate::qol::stash::check_stash_item(ctx, m, is_shift_held, is_ctrl_held);
                }
                crate::qol::stash::check_stash_button_press(ctx, m);
            } else if ctx.control.sbookflag && rp {
                crate::panels::spell_book::check_s_book(ctx);
            } else if !ctx.players.Players[me(ctx)].HoldItem.is_empty() {
                if !try_open_dungeon_with_mouse(ctx) {
                    let current_position = ctx.players.Players[me(ctx)].position.tile;
                    let dir = crate::engine::get_direction(current_position, ctx.cursor.cursPosition);
                    if let Some(item_tile) = crate::inv::find_adjacent_position_for_item(ctx, current_position, dir) {
                        let hold = ctx.players.Players[me(ctx)].HoldItem.clone();
                        crate::msg::net_send_cmd_p_item(ctx, true, CMD_PUTITEM, item_tile, &hold);
                        new_cursor(ctx, CURSOR_HAND);
                    }
                }
            } else {
                crate::control::check_lvl_btn(ctx);
                if !ctx.control.lvlbtndown {
                    left_mouse_cmd(ctx, is_shift_held);
                }
            }
        }
    } else {
        if !ctx.control.talkflag && !ctx.control.drop_gold_flag && !ctx.stash.IsWithdrawGoldOpen && !crate::gmenu::gmenu_is_active(ctx) {
            crate::inv::check_inv_scrn(ctx, is_shift_held, is_ctrl_held);
        }
        crate::control::do_pan_btn(ctx);
        crate::qol::stash::check_stash_button_press(ctx, m);
        if ctx.cursor.pcurs > CURSOR_HAND && ctx.cursor.pcurs < CURSOR_FIRSTITEM {
            new_cursor(ctx, CURSOR_HAND);
        }
    }
}

/// Original: `LeftMouseUp` (diablo.cpp).
// @port diablo.cpp|devilution::LeftMouseUp(uint16_t modState) sha=6e184a4487e3
pub(crate) fn left_mouse_up(ctx: &mut Ctx, mod_state: u16) {
    crate::gmenu::gmenu_left_mouse(ctx, false);
    crate::control::control_release_talk_btn(ctx);
    if ctx.control.panbtndown {
        crate::control::check_btn_up(ctx);
    }
    let m = mouse(ctx);
    crate::qol::stash::check_stash_button_release(ctx, m);
    if ctx.control.chrbtnactive {
        let is_shift_held = (mod_state & KMOD_SHIFT) != 0;
        crate::control::release_chr_btns(ctx, is_shift_held);
    }
    if ctx.control.lvlbtndown {
        crate::control::release_lvl_btn(ctx);
    }
    if !crate::stores::stextflag_is_none(ctx) {
        crate::stores::release_store_btn(ctx);
    }
}

/// Original: `RightMouseDown` (diablo.cpp).
// @port diablo.cpp|devilution::RightMouseDown(bool isShiftHeld) sha=a60f8df05488
pub(crate) fn right_mouse_down(ctx: &mut Ctx, is_shift_held: bool) {
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
    if crate::gmenu::gmenu_is_active(ctx) || ctx.diablo.sgn_timeout_curs != CURSOR_NONE || ctx.diablo.pause_mode == 2 || ctx.players.Players[me(ctx)]._pInvincible {
        return;
    }
    if crate::minitext::qtextflag(ctx) {
        crate::minitext::set_qtextflag(ctx, false);
        crate::effects::stream_stop(ctx);
        return;
    }
    if ctx.doom.DoomFlag {
        crate::doom::doom_close(ctx);
        return;
    }
    if !crate::stores::stextflag_is_none(ctx) {
        return;
    }
    if ctx.control.spselflag {
        crate::panels::spell_list::set_spell(ctx);
        return;
    }
    if ctx.control.sbookflag && crate::control::get_right_panel(ctx).contains(mouse(ctx)) {
        return;
    }
    if try_icon_curs(ctx) {
        return;
    }
    if ctx.cursor.pcursinvitem != -1 && crate::inv::use_inv_item(ctx, ctx.cursor.pcursinvitem as i32) {
        return;
    }
    if ctx.cursor.pcursstashitem != crate::qol::stash::StashStruct::EmptyCell && crate::qol::stash::use_stash_item(ctx, ctx.cursor.pcursstashitem) {
        return;
    }
    if ctx.cursor.pcurs == CURSOR_HAND {
        let p = &ctx.players.Players[me(ctx)];
        let (spell, spell_type) = (p._pRSpell, p._pRSplType);
        crate::player::check_plr_spell(ctx, is_shift_held, spell, spell_type);
    } else if ctx.cursor.pcurs > CURSOR_HAND && ctx.cursor.pcurs < CURSOR_FIRSTITEM {
        new_cursor(ctx, CURSOR_HAND);
    }
}

/// Original: `ReleaseKey` (diablo.cpp).
// @port diablo.cpp|devilution::ReleaseKey(SDL_Keycode vkey) sha=0f33048f9e3b
fn release_key(ctx: &mut Ctx, vkey: i32) {
    let vkey = crate::controls::remap_keyboard_key(vkey);
    if ctx.diablo.sgn_timeout_curs != CURSOR_NONE {
        return;
    }
    crate::options::keymapper_key_released(ctx, vkey);
}

/// Original: `ClosePanels` (diablo.cpp).
// @port diablo.cpp|devilution::ClosePanels() sha=e11d6f9594fb
pub fn close_panels(ctx: &mut Ctx) {
    if crate::control::can_panels_cover_view(ctx) {
        let m = mouse(ctx);
        let my = main_panel_y(ctx);
        if !crate::control::is_left_panel_open(ctx) && crate::control::is_right_panel_open(ctx) && m.x < 480 && m.y < my {
            crate::controls::set_cursor_pos(ctx, (m.x + 160, m.y));
        } else if !crate::control::is_right_panel_open(ctx) && crate::control::is_left_panel_open(ctx) && m.x > 160 && m.y < my {
            crate::controls::set_cursor_pos(ctx, (m.x - 160, m.y));
        }
    }
    crate::inv::close_inventory(ctx);
    crate::control::close_char_panel(ctx);
    ctx.control.sbookflag = false;
    ctx.quests.QuestLogIsOpen = false;
}

fn toggle_fullscreen(ctx: &mut Ctx) {
    let v = !crate::utils::display::is_full_screen(ctx);
    if let Some(cb) = ctx.options.graphics.fullscreen.set_value(v) {
        crate::options::run_option_callback(ctx, cb);
    }
    crate::options::save_options(ctx);
}

/// Original: `PressKey` (diablo.cpp).
// @port diablo.cpp|devilution::PressKey(SDL_Keycode vkey, uint16_t modState) sha=b9f4ca843c8f
fn press_key(ctx: &mut Ctx, vkey: i32, mod_state: u16) {
    let vkey = crate::controls::remap_keyboard_key(vkey);
    if vkey == SDLK_UNKNOWN {
        return;
    }
    if vkey == SDLK_PAUSE {
        diablo_pause_game(ctx);
        return;
    }
    if crate::gmenu::gmenu_presskeys(ctx, vkey) || crate::control::control_presskeys(ctx, vkey) {
        return;
    }
    let enter = vkey == SDLK_RETURN || vkey == SDLK_KP_ENTER;
    if ctx.players.MyPlayerIsDead {
        if ctx.diablo.sgn_timeout_curs != CURSOR_NONE {
            return;
        }
        crate::options::keymapper_key_pressed(ctx, vkey as u32);
        if enter {
            if (mod_state & KMOD_ALT) != 0 {
                toggle_fullscreen(ctx);
            } else {
                crate::control::control_type_message(ctx);
            }
        }
        if vkey != SDLK_ESCAPE {
            return;
        }
    }
    if vkey == SDLK_ESCAPE {
        if !press_esc_key(ctx) {
            ctx.diablo.last_mouse_button_action = MouseActionType::None;
            crate::gamemenu::gamemenu_on(ctx);
        }
        return;
    }
    if ctx.control.drop_gold_flag {
        crate::control::control_drop_gold(ctx, vkey);
        return;
    }
    if ctx.stash.IsWithdrawGoldOpen {
        crate::qol::stash::withdraw_gold_key_press(ctx, vkey);
        return;
    }
    if ctx.diablo.sgn_timeout_curs != CURSOR_NONE {
        return;
    }
    crate::options::keymapper_key_pressed(ctx, vkey as u32);
    if ctx.diablo.pause_mode == 2 {
        if enter && (mod_state & KMOD_ALT) != 0 {
            toggle_fullscreen(ctx);
        }
        return;
    }
    if ctx.doom.DoomFlag {
        crate::doom::doom_close(ctx);
        return;
    }
    let automap = crate::automap::automap_active(ctx);
    let store = !crate::stores::stextflag_is_none(ctx);
    match vkey {
        SDLK_PLUS | SDLK_KP_PLUS | SDLK_EQUALS | SDLK_KP_EQUALS => {
            if automap {
                crate::automap::automap_zoom_in(ctx);
            }
        }
        SDLK_MINUS | SDLK_KP_MINUS | SDLK_UNDERSCORE => {
            if automap {
                crate::automap::automap_zoom_out(ctx);
            }
        }
        SDLK_RETURN | SDLK_KP_ENTER => {
            if (mod_state & KMOD_ALT) != 0 {
                toggle_fullscreen(ctx);
            } else if store {
                crate::stores::store_enter(ctx);
            } else if ctx.quests.QuestLogIsOpen {
                crate::quests::questlog_enter(ctx);
            } else {
                crate::control::control_type_message(ctx);
            }
        }
        SDLK_UP => {
            if store {
                crate::stores::store_up(ctx);
            } else if ctx.quests.QuestLogIsOpen {
                crate::quests::questlog_up(ctx);
            } else if ctx.help.HelpFlag {
                crate::help::help_scroll_up(ctx);
            } else if ctx.chatlog.ChatLogFlag {
                crate::qol::chatlog::chat_log_scroll_up(ctx);
            } else if automap {
                crate::automap::automap_up(ctx);
            } else if ctx.stash.IsStashOpen {
                ctx.stash.Stash.previous_page(1);
            }
        }
        SDLK_DOWN => {
            if store {
                crate::stores::store_down(ctx);
            } else if ctx.quests.QuestLogIsOpen {
                crate::quests::questlog_down(ctx);
            } else if ctx.help.HelpFlag {
                crate::help::help_scroll_down(ctx);
            } else if ctx.chatlog.ChatLogFlag {
                crate::qol::chatlog::chat_log_scroll_down(ctx);
            } else if automap {
                crate::automap::automap_down(ctx);
            } else if ctx.stash.IsStashOpen {
                ctx.stash.Stash.next_page(1);
            }
        }
        SDLK_PAGEUP => {
            if store {
                crate::stores::store_prior(ctx);
            } else if ctx.chatlog.ChatLogFlag {
                crate::qol::chatlog::chat_log_scroll_top(ctx);
            }
        }
        SDLK_PAGEDOWN => {
            if store {
                crate::stores::store_next(ctx);
            } else if ctx.chatlog.ChatLogFlag {
                crate::qol::chatlog::chat_log_scroll_bottom(ctx);
            }
        }
        SDLK_LEFT => {
            if automap && !ctx.control.talkflag {
                crate::automap::automap_left(ctx);
            }
        }
        SDLK_RIGHT => {
            if automap && !ctx.control.talkflag {
                crate::automap::automap_right(ctx);
            }
        }
        _ => {}
    }
}

/// Original: `HandleMouseButtonDown` (diablo.cpp).
// @port diablo.cpp|devilution::HandleMouseButtonDown(Uint8 button, uint16_t modState) sha=a4d71ebd1352
fn handle_mouse_button_down(ctx: &mut Ctx, button: u8, mod_state: u16) {
    if !crate::stores::stextflag_is_none(ctx) && button == BUTTON_X1 {
        crate::stores::store_esc(ctx);
        return;
    }
    if ctx.diablo.sgb_mouse_down == CLICK_NONE {
        match button {
            BUTTON_LEFT => {
                ctx.diablo.sgb_mouse_down = CLICK_LEFT;
                left_mouse_down(ctx, mod_state);
            }
            BUTTON_RIGHT => {
                ctx.diablo.sgb_mouse_down = CLICK_RIGHT;
                right_mouse_down(ctx, (mod_state & KMOD_SHIFT) != 0);
            }
            _ => crate::options::keymapper_key_pressed(ctx, button as u32 | crate::options::KEYMAPPER_MOUSE_BUTTON_MASK),
        }
    }
}

/// Original: `HandleMouseButtonUp` (diablo.cpp).
// @port diablo.cpp|devilution::HandleMouseButtonUp(Uint8 button, uint16_t modState) sha=5db87de1ba36
fn handle_mouse_button_up(ctx: &mut Ctx, button: u8, mod_state: u16) {
    if ctx.diablo.sgb_mouse_down == CLICK_LEFT && button == BUTTON_LEFT {
        ctx.diablo.last_mouse_button_action = MouseActionType::None;
        ctx.diablo.sgb_mouse_down = CLICK_NONE;
        left_mouse_up(ctx, mod_state);
    } else if ctx.diablo.sgb_mouse_down == CLICK_RIGHT && button == BUTTON_RIGHT {
        ctx.diablo.last_mouse_button_action = MouseActionType::None;
        ctx.diablo.sgb_mouse_down = CLICK_NONE;
    } else {
        crate::options::keymapper_key_released(ctx, (button as u32 | crate::options::KEYMAPPER_MOUSE_BUTTON_MASK) as i32);
    }
}

/// Original: `HandleTextInput` (diablo.cpp).
// @port diablo.cpp|devilution::HandleTextInput(string_view text) sha=edf3c2487924
fn handle_text_input(ctx: &mut Ctx, text: &str) -> bool {
    if crate::control::is_talk_active(ctx) {
        crate::control::control_new_text(ctx, text);
        return true;
    }
    if ctx.control.drop_gold_flag {
        crate::control::gold_drop_new_text(ctx, text);
        return true;
    }
    if ctx.stash.IsWithdrawGoldOpen {
        crate::qol::stash::gold_withdraw_new_text(ctx, text);
        return true;
    }
    false
}

/// Original: `GameEventHandler` (diablo.cpp).
// @port diablo.cpp|devilution::GameEventHandler(const SDL_Event &event, uint16_t modState) sha=098dbfaeb4aa
pub fn game_event_handler(ctx: &mut Ctx, event: &Event, mod_state: u16) {
    let ctrl_events = crate::controls::controller::to_controller_button_events(ctx, event);
    for &ctrl_event in &ctrl_events {
        let mut action = crate::controls::game_controls::GameAction::default();
        if crate::controls::game_controls::handle_controller_button_event(ctx, event, ctrl_event, &mut action)
            && action.type_ == crate::controls::game_controls::GameActionType::SendKey
        {
            if (action.vk_code & crate::options::KEYMAPPER_MOUSE_BUTTON_MASK) != 0 {
                let button = (action.vk_code & !crate::options::KEYMAPPER_MOUSE_BUTTON_MASK) as u8;
                if !action.up {
                    handle_mouse_button_down(ctx, button, mod_state);
                } else {
                    handle_mouse_button_up(ctx, button, mod_state);
                }
            } else if !action.up {
                press_key(ctx, action.vk_code as i32, mod_state);
            } else {
                release_key(ctx, action.vk_code as i32);
            }
        }
    }
    if ctrl_events.first().is_some_and(|e| e.button != crate::controls::controller_buttons::ControllerButton::None) {
        return;
    }
    match event {
        Event::KeyDown { key, .. } => press_key(ctx, *key, mod_state),
        Event::KeyUp { key, .. } => release_key(ctx, *key),
        Event::TextInput(text) => {
            if !handle_text_input(ctx, text) {
                crate::platform::log::verbose!("Unhandled SDL event: SDL_TEXTINPUT 0");
            }
        }
        Event::MouseMotion { x, y } => {
            if ctx.controls.control_mode == crate::controls::ControlTypes::KeyboardAndMouse && ctx.inv.invflag {
                crate::controls::plrctrls::invalidate_inventory_slot(ctx);
            }
            ctx.diablo.mouse_position = (*x, *y);
            crate::gmenu::gmenu_on_mouse_move(ctx);
        }
        Event::MouseButtonDown { button, x, y, .. } => {
            ctx.diablo.mouse_position = (*x, *y);
            handle_mouse_button_down(ctx, *button, mod_state);
        }
        Event::MouseButtonUp { button, x, y, .. } => {
            ctx.diablo.mouse_position = (*x, *y);
            handle_mouse_button_up(ctx, *button, mod_state);
        }
        Event::TestWarp(lvl) => {
            // Test hook: behave like taking the stairs down into `lvl`.
            if let Some(me) = ctx.players.MyPlayer {
                crate::player::start_new_lvl(ctx, me, WM_DIABNEXTLVL, *lvl);
            }
        }
        Event::TestSetWarp(lvl, ltype) => {
            // Test hook: behave like entering quest level `lvl` (of dungeon type `ltype`) from its trigger.
            if let Some(me) = ctx.players.MyPlayer {
                ctx.gendung.setlvltype = crate::levels::gendung::DungeonType::from_i8(*ltype as i8);
                crate::player::start_new_lvl(ctx, me, WM_DIABSETLVL, *lvl);
            }
        }
        Event::TestStore(id) => {
            // Test hook: open a store page directly.
            crate::stores::start_store(ctx, TalkID::from_raw(*id as _));
        }
        Event::Custom(mode) => {
            if ctx.init.gb_is_multiplayer {
                crate::pfile::pfile_write_hero(ctx, true);
            }
            crate::nthread::nthread_ignore_mutex(ctx, true);
            crate::engine::palette::palette_fade_out(ctx, 8);
            crate::effects::sound_stop(ctx);
            crate::interfac::show_progress(ctx, *mode);
            crate::engine::backbuffer_state::redraw_everything(ctx);
            if !ctx.diablo.headless_mode {
                while crate::engine::backbuffer_state::is_redraw_everything(ctx) {
                    // In direct rendering mode with double/triple buffering, we need
                    // to prepare all buffers before fading in.
                    crate::engine::render::scrollrt::draw_and_blit(ctx);
                }
            }
            crate::quests::load_p_water_palette(ctx);
            if ctx.diablo.gb_run_game {
                crate::engine::palette::palette_fade_in(ctx, 8);
            }
            crate::nthread::nthread_ignore_mutex(ctx, false);
            ctx.diablo.gb_game_loop_startup = true;
        }
        _ => crate::init::main_wnd_proc(ctx, event),
    }
}

/// Original: `RunGameLoop` (diablo.cpp).
// @port diablo.cpp|devilution::RunGameLoop(interface_mode uMsg) sha=581b00f63f9b
fn run_game_loop(ctx: &mut Ctx, u_msg: interface_mode) {
    crate::engine::demomode::notify_game_loop_start(ctx);
    crate::nthread::nthread_ignore_mutex(ctx, true);
    start_game_ui(ctx, u_msg);
    let mut previous_handler = crate::engine::events::set_event_handler(ctx, Some(game_event_handler));
    crate::msg::run_delta_info(ctx);
    ctx.diablo.gb_run_game = true;
    ctx.diablo.gb_process_players = is_diablo_alive(ctx, true);
    ctx.diablo.gb_run_game_result = true;

    crate::engine::backbuffer_state::redraw_everything(ctx);
    if !ctx.diablo.headless_mode {
        while crate::engine::backbuffer_state::is_redraw_everything(ctx) {
            crate::engine::render::scrollrt::draw_and_blit(ctx);
        }
    }
    crate::quests::load_p_water_palette(ctx);
    crate::engine::palette::palette_fade_in(ctx, 8);
    crate::engine::backbuffer_state::init_backbuffer_state(ctx);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    ctx.diablo.gb_game_loop_startup = true;
    crate::nthread::nthread_ignore_mutex(ctx, false);
    // discord_manager::StartGame: Discord integration is off.

    while ctx.diablo.gb_run_game {
        while let Some((event, mod_state)) = crate::engine::events::fetch_message(ctx) {
            let Some(event) = event else { continue };
            if matches!(event, Event::Quit) {
                ctx.diablo.gb_run_game_result = false;
                ctx.diablo.gb_run_game = false;
                break;
            }
            crate::engine::events::handle_message(ctx, &event, mod_state);
        }
        if !ctx.diablo.gb_run_game {
            break;
        }
        let mut draw_game = true;
        let mut do_process_input = true;
        let run_game_loop = if crate::engine::demomode::is_running(ctx) {
            crate::engine::demomode::get_run_game_loop(ctx, &mut draw_game, &mut do_process_input)
        } else {
            crate::nthread::nthread_has_500ms_passed(ctx, Some(&mut draw_game))
        };
        if crate::engine::demomode::is_recording(ctx) {
            crate::engine::demomode::record_game_loop_result(ctx, run_game_loop);
        }
        if !run_game_loop {
            if do_process_input {
                process_input(ctx);
            }
            if !draw_game {
                continue;
            }
            crate::engine::backbuffer_state::redraw_viewport(ctx);
            crate::engine::render::scrollrt::draw_and_blit(ctx);
            continue;
        }
        crate::multi::multi_process_network_packets(ctx);
        let startup = ctx.diablo.gb_game_loop_startup;
        if game_loop(ctx, startup) {
            diablo_color_cyc_logic(ctx);
        }
        ctx.diablo.gb_game_loop_startup = false;
        if draw_game {
            crate::engine::render::scrollrt::draw_and_blit(ctx);
        }
    }

    crate::engine::demomode::notify_game_loop_end(ctx);
    if ctx.init.gb_is_multiplayer {
        crate::pfile::pfile_write_hero(ctx, false);
        crate::pfile::sfile_write_stash(ctx);
    }
    crate::engine::palette::palette_fade_out(ctx, 8);
    new_cursor(ctx, CURSOR_NONE);
    crate::engine::render::scrollrt::clear_screen_buffer(ctx);
    crate::engine::backbuffer_state::redraw_everything(ctx);
    crate::engine::render::scrollrt::scrollrt_draw_game_screen(ctx);
    previous_handler = crate::engine::events::set_event_handler(ctx, previous_handler);
    let _ = previous_handler;
    free_game(ctx);
    if ctx.diablo.cineflag {
        ctx.diablo.cineflag = false;
        crate::monster::do_ending(ctx);
    }
}

/// Original: `LoadLvlGFX` (diablo.cpp).
// @port diablo.cpp|devilution::LoadLvlGFX() sha=aade002c89cc
fn load_lvl_gfx(ctx: &mut Ctx) {
    assert!(ctx.gendung.pDungeonCels.is_none());
    const SPECIAL_CEL_WIDTH: u16 = 64;
    let (cel, til, special) = match ctx.gendung.leveltype {
        DungeonType::Town => {
            if ctx.init.gb_is_hellfire {
                ("nlevels\\towndata\\town.cel", "nlevels\\towndata\\town.til", "levels\\towndata\\towns")
            } else {
                ("levels\\towndata\\town.cel", "levels\\towndata\\town.til", "levels\\towndata\\towns")
            }
        }
        DungeonType::Cathedral => ("levels\\l1data\\l1.cel", "levels\\l1data\\l1.til", "levels\\l1data\\l1s"),
        DungeonType::Catacombs => ("levels\\l2data\\l2.cel", "levels\\l2data\\l2.til", "levels\\l2data\\l2s"),
        DungeonType::Caves => ("levels\\l3data\\l3.cel", "levels\\l3data\\l3.til", "levels\\l1data\\l1s"),
        DungeonType::Hell => ("levels\\l4data\\l4.cel", "levels\\l4data\\l4.til", "levels\\l2data\\l2s"),
        DungeonType::Nest => ("nlevels\\l6data\\l6.cel", "nlevels\\l6data\\l6.til", "levels\\l1data\\l1s"),
        DungeonType::Crypt => ("nlevels\\l5data\\l5.cel", "nlevels\\l5data\\l5.til", "nlevels\\l5data\\l5s"),
        _ => crate::appfat::app_fatal(ctx, "LoadLvlGFX"),
    };
    let cels = crate::engine::load_file::load_file_in_mem(ctx, cel).unwrap_or_else(|| crate::appfat::app_fatal(ctx, &format!("Failed to open file:\n{cel}")));
    ctx.gendung.pDungeonCels = Some(cels);
    let tils = crate::engine::load_file::load_file_in_mem(ctx, til).unwrap_or_else(|| crate::appfat::app_fatal(ctx, &format!("Failed to open file:\n{til}")));
    ctx.gendung.pMegaTiles = Some(
        tils.chunks_exact(8)
            .map(|c| crate::levels::gendung::MegaTile {
                micro1: u16::from_le_bytes([c[0], c[1]]),
                micro2: u16::from_le_bytes([c[2], c[3]]),
                micro3: u16::from_le_bytes([c[4], c[5]]),
                micro4: u16::from_le_bytes([c[6], c[7]]),
            })
            .collect(),
    );
    ctx.gendung.pSpecialCels = Some(crate::engine::load_sprites::load_cel(ctx, special, SPECIAL_CEL_WIDTH));
}

/// Original: `LoadAllGFX` (diablo.cpp).
// @port diablo.cpp|devilution::LoadAllGFX() sha=5c50a55c1700
fn load_all_gfx(ctx: &mut Ctx) {
    crate::interfac::inc_progress(ctx);
    // InitVirtualGamepadGFX: touch controls are not a target.
    crate::interfac::inc_progress(ctx);
    crate::objects::init_object_gfx(ctx);
    crate::interfac::inc_progress(ctx);
    let hf = ctx.init.gb_is_hellfire;
    crate::missiles::init_missile_gfx(ctx, hf);
    crate::interfac::inc_progress(ctx);
}

/// Original: `CreateLevel` (diablo.cpp).
// @port diablo.cpp|devilution::CreateLevel(lvl_entry entry) sha=59e0f355eca0
fn create_level(ctx: &mut Ctx, entry: lvl_entry) {
    let seed = ctx.diablo.glSeedTbl[ctx.gendung.currlevel as usize];
    crate::levels::gendung::create_dungeon(ctx, seed, entry);
    use crate::levels::trigs::*;
    match ctx.gendung.leveltype {
        DungeonType::Town => init_town_triggers(ctx),
        DungeonType::Cathedral => init_l1_triggers(ctx),
        DungeonType::Catacombs => init_l2_triggers(ctx),
        DungeonType::Caves => init_l3_triggers(ctx),
        DungeonType::Hell => init_l4_triggers(ctx),
        DungeonType::Nest => init_hive_triggers(ctx),
        DungeonType::Crypt => init_crypt_triggers(ctx),
        _ => crate::appfat::app_fatal(ctx, "CreateLevel"),
    }
    if ctx.gendung.leveltype != DungeonType::Town {
        freeupstairs(ctx);
    }
    let lt = ctx.gendung.leveltype;
    crate::engine::palette::load_rnd_lvl_pal(ctx, lt);
}

/// Original: `UnstuckChargers` (diablo.cpp).
// @port diablo.cpp|devilution::UnstuckChargers() sha=3ed2627f2a4a
fn unstuck_chargers(ctx: &mut Ctx) {
    if ctx.init.gb_is_multiplayer {
        for pnum in 0..ctx.players.Players.len() {
            let p = &ctx.players.Players[pnum];
            if !p.plractive || p._pLvlChanging || !crate::player::is_on_active_level(ctx, pnum) || Some(pnum) == ctx.players.MyPlayer {
                continue;
            }
            return;
        }
    }
    for i in 0..ctx.monster.ActiveMonsterCount {
        let m = ctx.monster.ActiveMonsters[i] as usize;
        let monster = &mut ctx.monster.Monsters[m];
        if monster.mode == crate::enums::MonsterMode::Charge {
            monster.mode = crate::enums::MonsterMode::Stand;
        }
    }
}

/// Original: `UpdateMonsterLights` (diablo.cpp).
// @port diablo.cpp|devilution::UpdateMonsterLights() sha=2a51f311645f
fn update_monster_lights(ctx: &mut Ctx) {
    for i in 0..ctx.monster.ActiveMonsterCount {
        let m = ctx.monster.ActiveMonsters[i] as usize;
        if (ctx.monster.Monsters[m].flags & MFLAG_BERSERK as u32) != 0 {
            let light_radius = if ctx.gendung.leveltype == DungeonType::Nest { 9 } else { 3 };
            let tile = ctx.monster.Monsters[m].position.tile;
            ctx.monster.Monsters[m].lightId = crate::lighting::add_light(ctx, tile, light_radius) as _;
        }
        let light_id = ctx.monster.Monsters[m].lightId as i32;
        if light_id != crate::lighting::NO_LIGHT {
            if light_id == ctx.players.Players[me(ctx)].lightId {
                // Fix old saves where some monsters had 0 instead of NO_LIGHT
                ctx.monster.Monsters[m].lightId = crate::lighting::NO_LIGHT as _;
                continue;
            }
            let tile = ctx.monster.Monsters[m].position.tile;
            if tile != ctx.lighting.Lights[light_id as usize].position.tile {
                crate::lighting::change_light_xy(ctx, light_id, tile);
            }
        }
    }
}

/// Original: `GameLogic` (diablo.cpp).
// @port diablo.cpp|devilution::GameLogic() sha=0a637808947b
fn game_logic(ctx: &mut Ctx) {
    if !process_input(ctx) {
        return;
    }
    if ctx.diablo.gb_process_players {
        ctx.diablo.g_game_logic_step = GameLogicStep::ProcessPlayers;
        crate::player::process_players(ctx);
    }
    if ctx.gendung.leveltype != DungeonType::Town {
        ctx.diablo.g_game_logic_step = GameLogicStep::ProcessMonsters;
        crate::monster::process_monsters(ctx);
        ctx.diablo.g_game_logic_step = GameLogicStep::ProcessObjects;
        crate::objects::process_objects(ctx);
        ctx.diablo.g_game_logic_step = GameLogicStep::ProcessMissiles;
        crate::missiles::process_missiles(ctx);
        ctx.diablo.g_game_logic_step = GameLogicStep::ProcessItems;
        crate::items::process_items(ctx);
        crate::lighting::process_light_list(ctx);
        crate::lighting::process_vision_list(ctx);
    } else {
        ctx.diablo.g_game_logic_step = GameLogicStep::ProcessTowners;
        crate::towners::process_towners(ctx);
        ctx.diablo.g_game_logic_step = GameLogicStep::ProcessItemsTown;
        crate::items::process_items(ctx);
        ctx.diablo.g_game_logic_step = GameLogicStep::ProcessMissilesTown;
        crate::missiles::process_missiles(ctx);
    }
    ctx.diablo.g_game_logic_step = GameLogicStep::None;
    crate::effects::sound_update(ctx);
    crate::levels::trigs::check_triggers(ctx);
    crate::quests::check_quests(ctx);
    crate::engine::backbuffer_state::redraw_viewport(ctx);
    crate::pfile::pfile_update(ctx, false);
    crate::controls::plrctrls::plrctrls_after_game_logic(ctx);
}

/// Original: `TimeoutCursor` (diablo.cpp).
// @port diablo.cpp|devilution::TimeoutCursor(bool bTimeout) sha=deea90ee0b8c
fn timeout_cursor(ctx: &mut Ctx, b_timeout: bool) {
    if b_timeout {
        if ctx.diablo.sgn_timeout_curs == CURSOR_NONE && ctx.diablo.sgb_mouse_down == CLICK_NONE {
            ctx.diablo.sgn_timeout_curs = ctx.cursor.pcurs;
            crate::multi::multi_net_ping(ctx);
            ctx.control.info_string.clear();
            crate::control::add_panel_string(ctx, &crate::utils::language::tr("-- Network timeout --"));
            crate::control::add_panel_string(ctx, &crate::utils::language::tr("-- Waiting for players --"));
            new_cursor(ctx, CURSOR_HOURGLASS);
            crate::engine::backbuffer_state::redraw_everything(ctx);
        }
        crate::engine::render::scrollrt::scrollrt_draw_game_screen(ctx);
    } else if ctx.diablo.sgn_timeout_curs != CURSOR_NONE {
        // Timeout is gone, we should restore the previous cursor.
        if ctx.cursor.pcurs == CURSOR_HOURGLASS {
            let c = ctx.diablo.sgn_timeout_curs;
            new_cursor(ctx, c);
        }
        ctx.diablo.sgn_timeout_curs = CURSOR_NONE;
        ctx.control.info_string.clear();
        crate::engine::backbuffer_state::redraw_everything(ctx);
    }
}

/// Original: `HelpKeyPressed` (diablo.cpp).
// @port diablo.cpp|devilution::HelpKeyPressed() sha=879ee70217fe
pub fn help_key_pressed(ctx: &mut Ctx) {
    if ctx.help.HelpFlag {
        ctx.help.HelpFlag = false;
    } else if !crate::stores::stextflag_is_none(ctx) {
        ctx.control.info_string.clear();
        crate::control::add_panel_string(ctx, &crate::utils::language::tr("No help available")); // BUGFIX: message isn't displayed
        crate::control::add_panel_string(ctx, &crate::utils::language::tr("while in stores"));
        ctx.diablo.last_mouse_button_action = MouseActionType::None;
    } else {
        crate::inv::close_inventory(ctx);
        crate::control::close_char_panel(ctx);
        ctx.control.sbookflag = false;
        ctx.control.spselflag = false;
        if crate::minitext::qtextflag(ctx) && ctx.gendung.leveltype == DungeonType::Town {
            crate::minitext::set_qtextflag(ctx, false);
            crate::effects::stream_stop(ctx);
        }
        ctx.quests.QuestLogIsOpen = false;
        crate::error::cancel_current_diablo_msg(ctx);
        crate::gamemenu::gamemenu_off(ctx);
        crate::help::display_help(ctx);
        crate::doom::doom_close(ctx);
    }
}

/// Original: `InventoryKeyPressed` (diablo.cpp).
// @port diablo.cpp|devilution::InventoryKeyPressed() sha=ef4d668efda1
pub fn inventory_key_pressed(ctx: &mut Ctx) {
    if !crate::stores::stextflag_is_none(ctx) {
        return;
    }
    ctx.inv.invflag = !ctx.inv.invflag;
    if !crate::control::is_left_panel_open(ctx) && crate::control::can_panels_cover_view(ctx) {
        let m = mouse(ctx);
        let my = main_panel_y(ctx);
        if !ctx.inv.invflag {
            // We closed the invetory
            if m.x < 480 && m.y < my {
                crate::controls::set_cursor_pos(ctx, (m.x + 160, m.y));
            }
        } else if !ctx.control.sbookflag && m.x > 160 && m.y < my {
            // We opened the invetory
            crate::controls::set_cursor_pos(ctx, (m.x - 160, m.y));
        }
    }
    ctx.control.sbookflag = false;
    crate::qol::stash::close_gold_withdraw(ctx);
    crate::inv::close_stash(ctx);
}

/// Original: `CharacterSheetKeyPressed` (diablo.cpp).
// @port diablo.cpp|devilution::CharacterSheetKeyPressed() sha=4dfc62e34cbf
pub fn character_sheet_key_pressed(ctx: &mut Ctx) {
    if !crate::stores::stextflag_is_none(ctx) {
        return;
    }
    if !crate::control::is_right_panel_open(ctx) && crate::control::can_panels_cover_view(ctx) {
        let m = mouse(ctx);
        let my = main_panel_y(ctx);
        if ctx.control.chrflag {
            // We are closing the character sheet
            if m.x > 160 && m.y < my {
                crate::controls::set_cursor_pos(ctx, (m.x - 160, m.y));
            }
        } else if !ctx.quests.QuestLogIsOpen && m.x < 480 && m.y < my {
            // We opened the character sheet
            crate::controls::set_cursor_pos(ctx, (m.x + 160, m.y));
        }
    }
    crate::control::toggle_char_panel(ctx);
}

/// Original: `QuestLogKeyPressed` (diablo.cpp).
// @port diablo.cpp|devilution::QuestLogKeyPressed() sha=ffa357efad83
pub fn quest_log_key_pressed(ctx: &mut Ctx) {
    if !crate::stores::stextflag_is_none(ctx) {
        return;
    }
    if !ctx.quests.QuestLogIsOpen {
        crate::quests::start_questlog(ctx);
    } else {
        ctx.quests.QuestLogIsOpen = false;
    }
    if !crate::control::is_right_panel_open(ctx) && crate::control::can_panels_cover_view(ctx) {
        let m = mouse(ctx);
        let my = main_panel_y(ctx);
        if !ctx.quests.QuestLogIsOpen {
            // We closed the quest log
            if m.x > 160 && m.y < my {
                crate::controls::set_cursor_pos(ctx, (m.x - 160, m.y));
            }
        } else if !ctx.control.chrflag && m.x < 480 && m.y < my {
            // We opened the character quest log
            crate::controls::set_cursor_pos(ctx, (m.x + 160, m.y));
        }
    }
    crate::control::close_char_panel(ctx);
    crate::qol::stash::close_gold_withdraw(ctx);
    crate::inv::close_stash(ctx);
}

/// Original: `DisplaySpellsKeyPressed` (diablo.cpp).
// @port diablo.cpp|devilution::DisplaySpellsKeyPressed() sha=7171b75243b8
pub fn display_spells_key_pressed(ctx: &mut Ctx) {
    if !crate::stores::stextflag_is_none(ctx) {
        return;
    }
    crate::control::close_char_panel(ctx);
    ctx.quests.QuestLogIsOpen = false;
    crate::inv::close_inventory(ctx);
    ctx.control.sbookflag = false;
    if !ctx.control.spselflag {
        crate::panels::spell_list::do_speed_book(ctx);
    } else {
        ctx.control.spselflag = false;
    }
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
}

/// Original: `SpellBookKeyPressed` (diablo.cpp).
// @port diablo.cpp|devilution::SpellBookKeyPressed() sha=12d564fe2569
pub fn spell_book_key_pressed(ctx: &mut Ctx) {
    if !crate::stores::stextflag_is_none(ctx) {
        return;
    }
    ctx.control.sbookflag = !ctx.control.sbookflag;
    if !crate::control::is_left_panel_open(ctx) && crate::control::can_panels_cover_view(ctx) {
        let m = mouse(ctx);
        let my = main_panel_y(ctx);
        if !ctx.control.sbookflag {
            if m.x < 480 && m.y < my {
                crate::controls::set_cursor_pos(ctx, (m.x + 160, m.y));
            }
        } else if !ctx.inv.invflag && m.x > 160 && m.y < my {
            crate::controls::set_cursor_pos(ctx, (m.x - 160, m.y));
        }
    }
    crate::inv::close_inventory(ctx);
}

/// Original: `IsPlayerDead` (diablo.cpp).
// @port diablo.cpp|devilution::IsPlayerDead() sha=cc1ba684dcb8
fn is_player_dead(ctx: &Ctx) -> bool {
    ctx.players.Players[me(ctx)]._pmode == PM_DEATH || ctx.players.MyPlayerIsDead
}

/// Original: `IsGameRunning` (diablo.cpp).
// @port diablo.cpp|devilution::IsGameRunning() sha=cd104421b7f1
pub fn is_game_running(ctx: &Ctx) -> bool {
    ctx.diablo.pause_mode != 2
}

/// Original: `CanPlayerTakeAction` (diablo.cpp).
// @port diablo.cpp|devilution::CanPlayerTakeAction() sha=4740c3cb0d22
pub fn can_player_take_action(ctx: &Ctx) -> bool {
    !is_player_dead(ctx) && is_game_running(ctx)
}

/// Original: `devilution::StartGame(bool bNewGame, bool bSinglePlayer)` (diablo.cpp).
// @port diablo.cpp|devilution::StartGame(bool bNewGame, bool bSinglePlayer) sha=f3a929de4206
pub fn start_game(ctx: &mut Ctx, b_new_game: bool, b_single_player: bool) -> bool {
    ctx.multi.gbSelectProvider = true;
    ctx.diablo.return_to_main_menu = false;
    loop {
        ctx.diablo.gb_load_game = false;
        if !crate::multi::net_init(ctx, b_single_player) {
            ctx.diablo.gb_run_game_result = true;
            break;
        }
        // Save 2.8 MiB of RAM by freeing all main menu resources before starting the game.
        crate::diablo_ui::diabloui::ui_destroy(ctx);
        ctx.multi.gbSelectProvider = false;
        if b_new_game || !crate::pfile::gb_valid_save_file(ctx) {
            crate::levels::gendung::init_levels(ctx);
            crate::quests::init_quests(ctx);
            crate::portal::init_portals(ctx);
            let me = me(ctx);
            crate::player::init_dung_msgs(ctx, me);
            crate::msg::delta_sync_junk(ctx);
        }
        ctx.loadsave.giNumberOfLevels = if ctx.init.gb_is_hellfire { 25 } else { 17 };
        let mut u_msg = WM_DIABNEWGAME;
        if crate::pfile::gb_valid_save_file(ctx) && ctx.diablo.gb_load_game {
            u_msg = WM_DIABLOADGAME;
        }
        run_game_loop(ctx, u_msg);
        crate::multi::net_close(ctx);
        crate::engine::render::text_render::unload_fonts(ctx);
        // If the player left the game into the main menu, initialize main menu resources.
        if ctx.diablo.gb_run_game_result {
            crate::diablo_ui::diabloui::ui_initialize(ctx);
        }
        if ctx.diablo.return_to_main_menu {
            return true;
        }
        if !ctx.diablo.gb_run_game_result {
            break;
        }
    }
    crate::storm::storm_net::snet_destroy(ctx);
    ctx.diablo.gb_run_game_result
}

/// Original: `devilution::TryIconCurs` (diablo.cpp).
// @port diablo.cpp|devilution::TryIconCurs() sha=d865fb8ebafc
pub fn try_icon_curs(ctx: &mut Ctx) -> bool {
    use crate::msg::*;
    let pcurs = ctx.cursor.pcurs;
    let pcursplr = ctx.cursor.pcursplr;
    if pcurs == CURSOR_RESURRECT {
        if pcursplr != -1 {
            net_send_cmd_param1(ctx, true, CMD_RESURRECT, pcursplr as u16);
            new_cursor(ctx, CURSOR_HAND);
            return true;
        }
        return false;
    }
    if pcurs == CURSOR_HEALOTHER {
        if pcursplr != -1 {
            net_send_cmd_param1(ctx, true, CMD_HEALOTHER, pcursplr as u16);
            new_cursor(ctx, CURSOR_HAND);
            return true;
        }
        return false;
    }
    if pcurs == CURSOR_TELEKINESIS {
        crate::inv::do_telekinesis(ctx);
        return true;
    }
    let me = me(ctx);
    let pcursinvitem = ctx.cursor.pcursinvitem;
    let pcursstashitem = ctx.cursor.pcursstashitem;
    let inspecting = crate::player::is_inspecting_player(ctx);
    let empty = crate::qol::stash::StashStruct::EmptyCell;
    if pcurs == CURSOR_IDENTIFY {
        if pcursinvitem != -1 && !inspecting {
            crate::items::check_identify(ctx, me, pcursinvitem as i32);
        } else if pcursstashitem != empty {
            ctx.stash.Stash.stashList[pcursstashitem as usize]._iIdentified = true;
        }
        new_cursor(ctx, CURSOR_HAND);
        return true;
    }
    if pcurs == CURSOR_REPAIR {
        if pcursinvitem != -1 && !inspecting {
            crate::items::do_repair(ctx, me, pcursinvitem as i32);
        } else if pcursstashitem != empty {
            let lvl = ctx.players.Players[me]._pLevel as i32;
            let mut item = std::mem::take(&mut ctx.stash.Stash.stashList[pcursstashitem as usize]);
            crate::items::repair_item(ctx, &mut item, lvl);
            ctx.stash.Stash.stashList[pcursstashitem as usize] = item;
        }
        new_cursor(ctx, CURSOR_HAND);
        return true;
    }
    if pcurs == CURSOR_RECHARGE {
        if pcursinvitem != -1 && !inspecting {
            crate::items::do_recharge(ctx, me, pcursinvitem as i32);
        } else if pcursstashitem != empty {
            let lvl = ctx.players.Players[me]._pLevel as i32;
            let mut item = std::mem::take(&mut ctx.stash.Stash.stashList[pcursstashitem as usize]);
            crate::items::recharge_loose_item(ctx, &mut item, lvl);
            ctx.stash.Stash.stashList[pcursstashitem as usize] = item;
        }
        new_cursor(ctx, CURSOR_HAND);
        return true;
    }
    if pcurs == CURSOR_OIL {
        let mut change_cursor = true;
        if pcursinvitem != -1 && !inspecting {
            change_cursor = crate::items::do_oil(ctx, me, pcursinvitem as i32);
        } else if pcursstashitem != empty {
            let mut item = std::mem::take(&mut ctx.stash.Stash.stashList[pcursstashitem as usize]);
            let oil = ctx.players.Players[me]._pOilType;
            change_cursor = crate::items::apply_oil_to_item(ctx, &mut item, oil);
            ctx.stash.Stash.stashList[pcursstashitem as usize] = item;
        }
        if change_cursor {
            new_cursor(ctx, CURSOR_HAND);
        }
        return true;
    }
    if pcurs == CURSOR_TELEPORT {
        let p = &ctx.players.Players[me];
        let spell_id = p.inventorySpell;
        let spell_type = SpellType::Scroll;
        let spell_level = p.get_spell_level(spell_id);
        let spell_from = p.spellFrom as i32;
        let tile = p.position.tile;
        let friendly = p.friendlyMode;
        let curs = ctx.cursor.cursPosition;
        if crate::spells::is_wall_spell(spell_id) {
            let sd = crate::engine::get_direction(tile, curs);
            net_send_cmd_loc_param5(ctx, true, CMD_SPELLXYD, curs, spell_id as i8 as u16, spell_type as u8 as u16, sd as u16, spell_level as u16, spell_from as u16);
        } else if ctx.cursor.pcursmonst != -1 {
            let m = ctx.cursor.pcursmonst as u16;
            net_send_cmd_param5(ctx, true, CMD_SPELLID, m, spell_id as i8 as u16, spell_type as u8 as u16, spell_level as u16, spell_from as u16);
        } else if pcursplr != -1 && !friendly {
            net_send_cmd_param5(ctx, true, CMD_SPELLPID, pcursplr as u16, spell_id as i8 as u16, spell_type as u8 as u16, spell_level as u16, spell_from as u16);
        } else {
            net_send_cmd_loc_param4(ctx, true, CMD_SPELLXY, curs, spell_id as i8 as u16, spell_type as u8 as u16, spell_level as u16, spell_from as u16);
        }
        new_cursor(ctx, CURSOR_HAND);
        return true;
    }
    if pcurs == CURSOR_DISARM && ctx.cursor.ObjectUnderCursor.is_none() {
        new_cursor(ctx, CURSOR_HAND);
        return true;
    }
    false
}

/// Original: `devilution::diablo_pause_game` (diablo.cpp).
// @port diablo.cpp|devilution::diablo_pause_game() sha=157ff940a55e
pub fn diablo_pause_game(ctx: &mut Ctx) {
    if !ctx.init.gb_is_multiplayer {
        if ctx.diablo.pause_mode != 0 {
            ctx.diablo.pause_mode = 0;
        } else {
            ctx.diablo.pause_mode = 2;
            crate::effects::sound_stop(ctx);
            crate::minitext::set_qtextflag(ctx, false);
            ctx.diablo.last_mouse_button_action = MouseActionType::None;
        }
        crate::engine::backbuffer_state::redraw_everything(ctx);
    }
}

/// Original: `devilution::PressEscKey` (diablo.cpp).
// @port diablo.cpp|devilution::PressEscKey() sha=440671252e5f
pub fn press_esc_key(ctx: &mut Ctx) -> bool {
    let mut rv = false;
    if ctx.doom.DoomFlag {
        crate::doom::doom_close(ctx);
        rv = true;
    }
    if ctx.help.HelpFlag {
        ctx.help.HelpFlag = false;
        rv = true;
    }
    if ctx.chatlog.ChatLogFlag {
        ctx.chatlog.ChatLogFlag = false;
        rv = true;
    }
    if crate::minitext::qtextflag(ctx) {
        crate::minitext::set_qtextflag(ctx, false);
        crate::effects::stream_stop(ctx);
        rv = true;
    }
    if !crate::stores::stextflag_is_none(ctx) {
        crate::stores::store_esc(ctx);
        rv = true;
    }
    if crate::error::is_diablo_msg_available(ctx) {
        crate::error::cancel_current_diablo_msg(ctx);
        rv = true;
    }
    if ctx.control.talkflag {
        crate::control::control_reset_talk(ctx);
        rv = true;
    }
    if ctx.control.drop_gold_flag {
        crate::control::control_drop_gold(ctx, SDLK_ESCAPE);
        rv = true;
    }
    if ctx.stash.IsWithdrawGoldOpen {
        crate::qol::stash::withdraw_gold_key_press(ctx, SDLK_ESCAPE);
        rv = true;
    }
    if ctx.control.spselflag {
        ctx.control.spselflag = false;
        rv = true;
    }
    if crate::control::is_left_panel_open(ctx) || crate::control::is_right_panel_open(ctx) {
        close_panels(ctx);
        rv = true;
    }
    rv
}

/// Original: `devilution::DisableInputEventHandler` (diablo.cpp).
// @port diablo.cpp|devilution::DisableInputEventHandler(const SDL_Event &event, uint16_t modState) sha=ed51dfe47902
pub fn disable_input_event_handler(ctx: &mut Ctx, event: &Event, _mod_state: u16) {
    match event {
        Event::MouseMotion { x, y } => {
            ctx.diablo.mouse_position = (*x, *y);
        }
        Event::MouseButtonDown { button, .. } => {
            if ctx.diablo.sgb_mouse_down != CLICK_NONE {
                return;
            }
            match *button {
                BUTTON_LEFT => ctx.diablo.sgb_mouse_down = CLICK_LEFT,
                BUTTON_RIGHT => ctx.diablo.sgb_mouse_down = CLICK_RIGHT,
                _ => {}
            }
        }
        Event::MouseButtonUp { .. } => {
            ctx.diablo.sgb_mouse_down = CLICK_NONE;
        }
        _ => crate::init::main_wnd_proc(ctx, event),
    }
}

fn init_players_on_level(ctx: &mut Ctx, lvldir: lvl_entry, firstflag: bool) {
    for pnum in 0..ctx.players.Players.len() {
        if ctx.players.Players[pnum].plractive && crate::player::is_on_active_level(ctx, pnum) {
            crate::player::init_player_gfx(ctx, pnum);
            if lvldir != ENTRY_LOAD {
                crate::player::init_player(ctx, pnum, firstflag);
            }
        }
    }
}

/// Original: `devilution::LoadGameLevel` (diablo.cpp).
// @port diablo.cpp|devilution::LoadGameLevel(bool firstflag, lvl_entry lvldir) sha=1ede209d4fc3
pub fn load_game_level(ctx: &mut Ctx, firstflag: bool, lvldir: lvl_entry) {
    use crate::interfac::inc_progress;
    let lt = ctx.gendung.leveltype;
    let needed_track = crate::engine::sound::get_level_music(lt);
    crate::qol::floatingnumbers::clear_floating_numbers(ctx);
    if needed_track != ctx.sound.sgn_music_track {
        crate::engine::sound::music_stop(ctx);
    }
    if ctx.cursor.pcurs > CURSOR_HAND && ctx.cursor.pcurs < CURSOR_FIRSTITEM {
        new_cursor(ctx, CURSOR_HAND);
    }
    let seed = |ctx: &Ctx| ctx.diablo.glSeedTbl[ctx.gendung.currlevel as usize];
    let s = seed(ctx);
    ctx.rng.set_rnd_seed(s);
    inc_progress(ctx);
    crate::lighting::make_light_table(ctx);
    crate::levels::gendung::set_dungeon_micros(ctx);
    load_lvl_gfx(ctx);
    inc_progress(ctx);

    if firstflag {
        crate::inv::close_inventory(ctx);
        crate::minitext::set_qtextflag(ctx, false);
        if !ctx.diablo.headless_mode {
            crate::inv::init_inv(ctx);
            crate::minitext::init_quest_text(ctx);
            crate::panels::info_box::init_info_box_gfx(ctx);
            crate::help::init_help(ctx);
        }
        crate::stores::init_stores(ctx);
        crate::automap::init_automap_once(ctx);
    }
    if !ctx.gendung.setlevel {
        let s = seed(ctx);
        ctx.rng.set_rnd_seed(s);
    } else {
        // Maps are not randomly generated, but the monsters max hitpoints are.
        // So we need to ensure that we have a stable seed when generating quest/set-maps.
        // For this purpose we reuse the normal dungeon seeds.
        let s = ctx.diablo.glSeedTbl[ctx.gendung.setlvlnum as usize];
        ctx.rng.set_rnd_seed(s);
    }
    if ctx.gendung.leveltype == DungeonType::Town {
        crate::stores::setup_town_stores(ctx);
    } else {
        crate::stores::free_store_mem(ctx);
    }
    if firstflag || lvldir == ENTRY_LOAD {
        let is_hellfire_save_game = ctx.init.gb_is_hellfire_save_game;
        ctx.init.gb_is_hellfire_save_game = ctx.init.gb_is_hellfire;
        crate::loadsave::load_stash(ctx);
        ctx.init.gb_is_hellfire_save_game = is_hellfire_save_game;
    }
    inc_progress(ctx);
    crate::automap::init_automap(ctx);
    if ctx.gendung.leveltype != DungeonType::Town && lvldir != ENTRY_LOAD {
        crate::lighting::init_lighting(ctx);
    }
    crate::monster::init_level_monsters(ctx);
    inc_progress(ctx);

    let me = me(ctx);
    if !ctx.gendung.setlevel {
        create_level(ctx, lvldir);
        inc_progress(ctx);
        crate::levels::gendung::load_level_sol_data(ctx);
        let s = seed(ctx);
        ctx.rng.set_rnd_seed(s);

        if ctx.gendung.leveltype != DungeonType::Town {
            crate::monster::get_level_m_types(ctx);
            crate::levels::themes::init_themes(ctx);
            if !ctx.diablo.headless_mode {
                load_all_gfx(ctx);
            }
        } else if !ctx.diablo.headless_mode {
            inc_progress(ctx);
            // InitVirtualGamepadGFX: touch controls are not a target.
            inc_progress(ctx);
            let hf = ctx.init.gb_is_hellfire;
            crate::missiles::init_missile_gfx(ctx, hf);
            inc_progress(ctx);
            inc_progress(ctx);
        }
        inc_progress(ctx);

        if lvldir == ENTRY_RTNLVL {
            ctx.gendung.ViewPosition = crate::quests::get_map_return_position(ctx);
            if ctx.quests.Quests[Q_BETRAYER as usize]._qactive == QUEST_DONE {
                ctx.quests.Quests[Q_BETRAYER as usize]._qvar2 = 2;
            }
        }
        if lvldir == ENTRY_WARPLVL {
            crate::portal::get_portal_lvl_pos(ctx);
        }
        inc_progress(ctx);

        init_players_on_level(ctx, lvldir, firstflag);

        crate::player::play_dung_msgs(ctx);
        crate::player::init_multi_view(ctx);
        inc_progress(ctx);

        let s = seed(ctx);
        ctx.rng.set_rnd_seed(s);

        let cur = ctx.gendung.currlevel as usize;
        if ctx.gendung.leveltype != DungeonType::Town {
            if firstflag || lvldir == ENTRY_LOAD || !ctx.players.Players[me]._pLvlVisited[cur] || ctx.init.gb_is_multiplayer {
                crate::levels::themes::hold_theme_rooms(ctx);
                crate::monster::init_golems(ctx);
                crate::objects::init_objects(ctx);
                inc_progress(ctx);
                crate::monster::init_monsters(ctx);
                crate::items::init_items(ctx);
                crate::levels::themes::create_theme_rooms(ctx);
                inc_progress(ctx);
                crate::missiles::init_missiles(ctx);
                crate::dead::init_corpses(ctx);
                crate::lighting::save_pre_lighting(ctx);
                inc_progress(ctx);
                if ctx.init.gb_is_multiplayer {
                    crate::msg::delta_load_level(ctx);
                }
            } else {
                crate::levels::themes::hold_theme_rooms(ctx);
                crate::monster::init_golems(ctx);
                crate::monster::init_monsters(ctx);
                crate::missiles::init_missiles(ctx);
                crate::dead::init_corpses(ctx);
                inc_progress(ctx);
                crate::loadsave::load_level(ctx);
                inc_progress(ctx);
            }
        } else {
            for i in 0..MAXDUNX {
                for j in 0..MAXDUNY {
                    ctx.gendung.dFlags[i][j] |= DungeonFlag::Lit;
                }
            }
            crate::towners::init_towners(ctx);
            crate::qol::stash::init_stash(ctx);
            crate::items::init_items(ctx);
            crate::missiles::init_missiles(ctx);
            inc_progress(ctx);
            if !firstflag && lvldir != ENTRY_LOAD && ctx.players.Players[me]._pLvlVisited[cur] && !ctx.init.gb_is_multiplayer {
                crate::loadsave::load_level(ctx);
            }
            if ctx.init.gb_is_multiplayer {
                crate::msg::delta_load_level(ctx);
            }
            inc_progress(ctx);
            for x in 0..DMAXX {
                for y in 0..DMAXY {
                    crate::automap::update_automap_explorer(ctx, Point::new(x as i32, y as i32), crate::lighting::MAP_EXP_SELF);
                }
            }
        }
        if crate::quests::use_multiplayer_quests(ctx) {
            crate::quests::resync_mp_quests(ctx);
        } else {
            crate::quests::resync_quests(ctx);
        }
    } else {
        crate::levels::setmaps::load_set_map(ctx);
        inc_progress(ctx);
        crate::monster::get_level_m_types(ctx);
        inc_progress(ctx);
        crate::monster::init_golems(ctx);
        crate::monster::init_monsters(ctx);
        inc_progress(ctx);
        if !ctx.diablo.headless_mode {
            let hf = ctx.init.gb_is_hellfire;
            crate::missiles::init_missile_gfx(ctx, hf);
            inc_progress(ctx);
        }
        crate::dead::init_corpses(ctx);
        inc_progress(ctx);
        crate::levels::gendung::load_level_sol_data(ctx);
        inc_progress(ctx);
        if lvldir == ENTRY_WARPLVL {
            crate::portal::get_portal_lvl_pos(ctx);
        }
        inc_progress(ctx);
        init_players_on_level(ctx, lvldir, firstflag);
        inc_progress(ctx);
        crate::player::play_dung_msgs(ctx);
        crate::player::init_multi_view(ctx);
        inc_progress(ctx);
        let slvl = ctx.gendung.setlvlnum as usize;
        if firstflag || lvldir == ENTRY_LOAD || !ctx.players.Players[me]._pSLvlVisited[slvl] || ctx.init.gb_is_multiplayer {
            crate::items::init_items(ctx);
            crate::lighting::save_pre_lighting(ctx);
        } else {
            crate::loadsave::load_level(ctx);
        }
        if ctx.init.gb_is_multiplayer {
            crate::msg::delta_load_level(ctx);
            if !crate::quests::use_multiplayer_quests(ctx) {
                crate::quests::resync_quests(ctx);
            }
        }
        crate::missiles::init_missiles(ctx);
        inc_progress(ctx);
    }

    crate::portal::sync_portals(ctx);

    for pnum in 0..ctx.players.Players.len() {
        let p = &ctx.players.Players[pnum];
        if p.plractive && crate::player::is_on_active_level(ctx, pnum) && (!p._pLvlChanging || Some(pnum) == ctx.players.MyPlayer) {
            if p._pHitPoints > 0 {
                if lvldir != ENTRY_LOAD {
                    crate::player::sync_init_plr_pos(ctx, pnum);
                }
            } else {
                let t = p.position.tile;
                ctx.gendung.dFlags[t.x as usize][t.y as usize] |= DungeonFlag::DeadPlayer;
            }
        }
    }

    inc_progress(ctx);
    inc_progress(ctx);
    if firstflag {
        crate::control::init_control_pan(ctx);
    }
    inc_progress(ctx);
    update_monster_lights(ctx);
    unstuck_chargers(ctx);
    if ctx.gendung.leveltype != DungeonType::Town {
        // resets the light on entering a level to get rid of incorrect light
        *ctx.gendung.dLight = *ctx.gendung.dPreLight;
        let my_id = ctx.players.MyPlayerId;
        let (light_id, tile) = (ctx.players.Players[my_id].lightId, ctx.players.Players[my_id].position.tile);
        crate::lighting::change_light_xy(ctx, light_id, tile); // forces player light refresh
        crate::lighting::process_light_list(ctx);
        crate::lighting::process_vision_list(ctx);
    }
    if ctx.gendung.leveltype == DungeonType::Crypt {
        if crate::items::CornerStoneStruct::is_available(ctx) {
            let pos = ctx.items.CornerStone.position;
            crate::items::cornerstone_load(ctx, pos);
        }
        if ctx.quests.Quests[Q_NAKRUL as usize]._qactive == QUEST_DONE && ctx.gendung.currlevel == 24 {
            crate::objects::sync_nakrul_room(ctx);
        }
    }
    // ActivateVirtualGamepad: touch controls are not a target.
    if ctx.sound.sgn_music_track != needed_track {
        crate::engine::sound::music_start(ctx, needed_track);
    }
    if ctx.diablo.minimize_paused {
        crate::engine::sound::music_mute(ctx);
    }
    crate::interfac::complete_progress(ctx);

    // Recalculate mouse selection of entities after level change/load
    ctx.diablo.last_mouse_button_action = MouseActionType::None;
    ctx.diablo.sgb_mouse_down = CLICK_NONE;
    crate::qol::itemlabels::reset_itemlabel_highlighted(ctx);
    ctx.cursor.pcursmonst = -1; // ensure pcurstemp is set to a valid value
    crate::cursor::check_curs_move(ctx);
}

/// Original: `devilution::game_loop` (diablo.cpp).
// @port diablo.cpp|devilution::game_loop(bool bStartup) sha=b5c6a184412c
pub fn game_loop(ctx: &mut Ctx, b_startup: bool) -> bool {
    let wait: u16 = if b_startup { ctx.multi.sgGameInitInfo.nTickRate as u16 * 3 } else { 3 };
    for _ in 0..wait {
        if !crate::multi::multi_handle_delta(ctx) {
            timeout_cursor(ctx, true);
            return false;
        }
        timeout_cursor(ctx, false);
        game_logic(ctx);
        crate::msg::clear_last_sent_player_cmd(ctx);
        if !ctx.diablo.gb_run_game
            || !ctx.init.gb_is_multiplayer
            || crate::engine::demomode::is_running(ctx)
            || crate::engine::demomode::is_recording(ctx)
            || !crate::nthread::nthread_has_500ms_passed(ctx, None)
        {
            break;
        }
    }
    true
}

/// Original: `devilution::diablo_color_cyc_logic` (diablo.cpp).
// @port diablo.cpp|devilution::diablo_color_cyc_logic() sha=7fc7b3d43117
pub fn diablo_color_cyc_logic(ctx: &mut Ctx) {
    if !ctx.options.graphics.color_cycling.get() {
        return;
    }
    if ctx.diablo.pause_mode != 0 {
        return;
    }
    match ctx.gendung.leveltype {
        DungeonType::Caves => {
            if ctx.gendung.setlevel && ctx.gendung.setlvlnum == ctx.quests.Quests[Q_PWATER as usize]._qslvl {
                crate::quests::update_p_water_palette(ctx);
            } else {
                crate::engine::palette::palette_update_caves(ctx);
            }
        }
        DungeonType::Hell => crate::lighting::lighting_color_cycling(ctx),
        DungeonType::Nest => crate::engine::palette::palette_update_hive(ctx),
        DungeonType::Crypt => crate::engine::palette::palette_update_crypt(ctx),
        _ => {}
    }
}

/// Original: `devilution::IsDiabloAlive` (diablo.cpp).
// @port diablo.cpp|devilution::IsDiabloAlive(bool playSFX) sha=324013fb2960
pub fn is_diablo_alive(ctx: &mut Ctx, play_sfx: bool) -> bool {
    if ctx.quests.Quests[Q_DIABLO as usize]._qactive == QUEST_DONE && !ctx.init.gb_is_multiplayer {
        if play_sfx {
            crate::effects::play_sfx(ctx, crate::effects_data::USFX_DIABLOD);
        }
        return false;
    }
    true
}

/// Original: `devilution::PrintScreen` (diablo.cpp).
// @port diablo.cpp|devilution::PrintScreen(SDL_Keycode vkey) sha=1d90bbf0580a
pub fn print_screen(ctx: &mut Ctx, vkey: i32) {
    release_key(ctx, vkey);
}
