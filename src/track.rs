//! `Source/track.cpp`: tracking what the mouse cursor is pointing at.

use crate::ctx::Ctx;
use crate::diablo::MouseActionType;
use crate::enums::*;
use crate::levels::gendung::{in_dungeon_bounds, is_tile_lit};

/// Original: `RepeatWalk` (track.cpp).
// @port track.cpp|devilution::RepeatWalk(Player &player) sha=7ebdb3c48cf8
fn repeat_walk(ctx: &mut Ctx, pnum: usize) {
    let curs = ctx.cursor.cursPosition;
    if !in_dungeon_bounds(curs) {
        return;
    }
    let player = &ctx.players.Players[pnum];
    if player._pmode != PM_STAND && !(player.is_walking() && player.AnimInfo.get_frame_to_use_for_rendering(ctx.nthread.ProgressToNextGameTick) > 6) {
        return;
    }
    let target = player.get_target_position();
    if curs == target {
        return;
    }
    let id = ctx.players.MyPlayerId;
    crate::msg::net_send_cmd_loc(ctx, id, true, CMD_WALKXY, curs);
}

/// Original: `devilution::InvalidateTargets` (track.cpp).
// @port track.cpp|devilution::InvalidateTargets() sha=edaa702b210d
pub fn invalidate_targets(ctx: &mut Ctx) {
    if ctx.cursor.pcursmonst != -1 {
        let monster = &ctx.monster.Monsters[ctx.cursor.pcursmonst as usize];
        if monster.isInvalid || monster.hitPoints >> 6 <= 0 || (monster.flags & MFLAG_HIDDEN as u32) != 0 || !is_tile_lit(ctx, monster.position.tile) {
            ctx.cursor.pcursmonst = -1;
        }
    }
    if let Some(o) = ctx.cursor.ObjectUnderCursor {
        if ctx.objects.Objects[o]._oSelFlag < 1 {
            ctx.cursor.ObjectUnderCursor = None;
        }
    }
    if ctx.cursor.pcursplr != -1 {
        let p = ctx.cursor.pcursplr as usize;
        let target_player = &ctx.players.Players[p];
        if target_player._pmode == PM_DEATH
            || target_player._pmode == PM_QUIT
            || !target_player.plractive
            || !crate::player::is_on_active_level(ctx, p)
            || target_player._pHitPoints >> 6 <= 0
            || !is_tile_lit(ctx, target_player.position.tile)
        {
            ctx.cursor.pcursplr = -1;
        }
    }
}

/// Original: `devilution::RepeatMouseAction` (track.cpp).
// @port track.cpp|devilution::RepeatMouseAction() sha=fb0a905da031
pub fn repeat_mouse_action(ctx: &mut Ctx) {
    if ctx.cursor.pcurs != crate::cursor::CURSOR_HAND {
        return;
    }
    if ctx.diablo.sgb_mouse_down == CLICK_NONE && !crate::controls::plrctrls::controller_action_held(ctx) {
        return;
    }
    if !crate::stores::stextflag_is_none(ctx) {
        return;
    }
    if ctx.diablo.last_mouse_button_action == MouseActionType::None {
        return;
    }
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let my_player = &ctx.players.Players[me];
    if my_player.destAction != ACTION_NONE {
        return;
    }
    if my_player._pInvincible {
        return;
    }
    if !my_player.can_change_action() {
        return;
    }
    let ranged_attack = my_player.uses_ranged_weapon();
    let my_id = ctx.players.MyPlayerId;
    let curs = ctx.cursor.cursPosition;
    let kbm = ctx.controls.control_mode == crate::controls::ControlTypes::KeyboardAndMouse;
    match ctx.diablo.last_mouse_button_action {
        MouseActionType::Attack => {
            if in_dungeon_bounds(curs) {
                crate::msg::net_send_cmd_loc(ctx, my_id, true, if ranged_attack { CMD_RATTACKXY } else { CMD_SATTACKXY }, curs);
            }
        }
        MouseActionType::AttackMonsterTarget => {
            if ctx.cursor.pcursmonst != -1 {
                let m = ctx.cursor.pcursmonst as u16;
                crate::msg::net_send_cmd_param1(ctx, true, if ranged_attack { CMD_RATTACKID } else { CMD_ATTACKID }, m);
            }
        }
        MouseActionType::AttackPlayerTarget => {
            if ctx.cursor.pcursplr != -1 && !ctx.players.Players[me].friendlyMode {
                let p = ctx.cursor.pcursplr as u16;
                crate::msg::net_send_cmd_param1(ctx, true, if ranged_attack { CMD_RATTACKPID } else { CMD_ATTACKPID }, p);
            }
        }
        MouseActionType::Spell => {
            if !kbm {
                let spell = ctx.players.Players[me]._pRSpell;
                crate::controls::plrctrls::update_spell_target(ctx, spell);
            }
            let (s, t) = (ctx.players.Players[me]._pRSpell, ctx.players.Players[me]._pRSplType);
            crate::player::check_plr_spell(ctx, kbm, s, t);
        }
        MouseActionType::SpellMonsterTarget => {
            if ctx.cursor.pcursmonst != -1 {
                let (s, t) = (ctx.players.Players[me]._pRSpell, ctx.players.Players[me]._pRSplType);
                crate::player::check_plr_spell(ctx, false, s, t);
            }
        }
        MouseActionType::SpellPlayerTarget => {
            if ctx.cursor.pcursplr != -1 && !ctx.players.Players[me].friendlyMode {
                let (s, t) = (ctx.players.Players[me]._pRSpell, ctx.players.Players[me]._pRSplType);
                crate::player::check_plr_spell(ctx, false, s, t);
            }
        }
        MouseActionType::OperateObject => {
            if let Some(o) = ctx.cursor.ObjectUnderCursor {
                if !ctx.objects.Objects[o].is_door() {
                    crate::msg::net_send_cmd_loc(ctx, my_id, true, CMD_OPOBJXY, curs);
                }
            }
        }
        MouseActionType::Walk => repeat_walk(ctx, me),
        MouseActionType::None => {}
    }
}

/// Original: `devilution::track_isscrolling` (track.cpp).
// @port track.cpp|devilution::track_isscrolling() sha=28118f10a0c9
pub fn track_isscrolling(ctx: &Ctx) -> bool {
    ctx.diablo.last_mouse_button_action == MouseActionType::Walk
}
