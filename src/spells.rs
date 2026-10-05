//! `Source/spells.cpp`: casting player spells.

use crate::ctx::Ctx;
use crate::engine::backbuffer_state::{redraw_component, redraw_everything, PanelDrawComponent};
use crate::engine::geometry::{Direction, Point};
use crate::enums::*;
use crate::items::get_spell_data;
use crate::missiles::{add_missile, TARGET_MONSTERS};
use crate::player::Player;

/// `NumHotkeys` (player.h)
pub const NUM_HOTKEYS: usize = 12;

/// Original: `GetSpellBitmask` (spells.h).
// @port spells.h|devilution::GetSpellBitmask(SpellID spellId) sha=229f5edc0bf5
pub const fn get_spell_bitmask(spell_id: SpellID) -> u64 {
    1u64 << ((spell_id as i8 as i32 - 1) as u32 & 63)
}

/// Original: `IsReadiedSpellValid` (spells.cpp).
// @port spells.cpp|devilution::IsReadiedSpellValid(const Player &player) sha=604c6d55e28e
fn is_readied_spell_valid(player: &Player) -> bool {
    match player._pRSplType {
        SpellType::Skill | SpellType::Spell | SpellType::Invalid => true,
        SpellType::Charges => (player._pISpells & get_spell_bitmask(player._pRSpell)) != 0,
        SpellType::Scroll => (player._pScrlSpells & get_spell_bitmask(player._pRSpell)) != 0,
        #[allow(unreachable_patterns)]
        _ => false,
    }
}

/// Original: `ClearReadiedSpell` (spells.cpp).
// @port spells.cpp|devilution::ClearReadiedSpell(Player &player) sha=3d67116e4ccf
fn clear_readied_spell(ctx: &mut Ctx, pnum: usize) {
    if ctx.players.Players[pnum]._pRSpell != SpellID::Invalid {
        ctx.players.Players[pnum]._pRSpell = SpellID::Invalid;
        redraw_everything(ctx);
    }
    if ctx.players.Players[pnum]._pRSplType != SpellType::Invalid {
        ctx.players.Players[pnum]._pRSplType = SpellType::Invalid;
        redraw_everything(ctx);
    }
}

/// Original: `devilution::IsValidSpell` (spells.cpp).
// @port spells.cpp|devilution::IsValidSpell(SpellID spl) sha=1458e65f34ab
pub fn is_valid_spell_hf(spl: SpellID, gb_is_hellfire: bool) -> bool {
    let s = spl as i8;
    s > SpellID::Null as i8 && s <= SpellID::LAST as i8 && (s <= SpellID::LastDiablo as i8 || gb_is_hellfire)
}

/// `IsValidSpell` reading `gbIsHellfire` from the context.
pub fn is_valid_spell(ctx: &Ctx, spl: SpellID) -> bool {
    is_valid_spell_hf(spl, ctx.init.gb_is_hellfire)
}

/// Original: `devilution::IsValidSpellFrom` (spells.cpp).
// @port spells.cpp|devilution::IsValidSpellFrom(int spellFrom) sha=c208d7a01351
pub fn is_valid_spell_from(spell_from: i32) -> bool {
    if spell_from == 0 {
        return true;
    }
    if spell_from >= INVITEM_INV_FIRST as i32 && spell_from <= INVITEM_INV_LAST as i32 {
        return true;
    }
    if spell_from >= INVITEM_BELT_FIRST as i32 && spell_from <= INVITEM_BELT_LAST as i32 {
        return true;
    }
    false
}

/// Original: `devilution::IsWallSpell` (spells.cpp).
// @port spells.cpp|devilution::IsWallSpell(SpellID spl) sha=4218959df6cb
pub fn is_wall_spell(spl: SpellID) -> bool {
    spl == SpellID::FireWall || spl == SpellID::LightningWall
}

/// Original: `devilution::TargetsMonster` (spells.cpp).
// @port spells.cpp|devilution::TargetsMonster(SpellID id) sha=51841afe72ca
pub fn targets_monster(id: SpellID) -> bool {
    matches!(id, SpellID::Fireball | SpellID::FireWall | SpellID::Inferno | SpellID::Lightning | SpellID::StoneCurse | SpellID::FlameWave)
}

/// Original: `devilution::GetManaAmount` (spells.cpp).
// @port spells.cpp|devilution::GetManaAmount(const Player &player, SpellID sn) sha=0004383ab5de
pub fn get_mana_amount(ctx: &Ctx, player: &Player, sn: SpellID) -> i32 {
    let mut ma: i32;
    let mut adj = 0;
    let sl = (player.get_spell_level(sn) - 1).max(0);
    if sl > 0 {
        adj = sl * get_spell_data(sn).sManaAdj as i32;
    }
    if sn == SpellID::Firebolt {
        adj /= 2;
    }
    if sn == SpellID::Resurrect && sl > 0 {
        adj = sl * (get_spell_data(SpellID::Resurrect).sManaCost as i32 / 8);
    }
    if sn == SpellID::Healing || sn == SpellID::HealOther {
        ma = get_spell_data(SpellID::Healing).sManaCost as i32 + 2 * player._pLevel as i32 - adj;
    } else if get_spell_data(sn).sManaCost == 255 {
        ma = (player._pMaxManaBase >> 6) - adj;
    } else {
        ma = get_spell_data(sn).sManaCost as i32 - adj;
    }
    ma = ma.max(0);
    ma <<= 6;
    if ctx.init.gb_is_hellfire && player._pClass == HeroClass::Sorcerer {
        ma /= 2;
    } else if matches!(player._pClass, HeroClass::Rogue | HeroClass::Monk | HeroClass::Bard) {
        ma -= ma / 4;
    }
    if get_spell_data(sn).sMinMana as i32 > ma >> 6 {
        ma = (get_spell_data(sn).sMinMana as i32) << 6;
    }
    ma
}

/// Original: `devilution::ConsumeSpell` (spells.cpp).
// @port spells.cpp|devilution::ConsumeSpell(Player &player, SpellID sn) sha=5268c8071394
pub fn consume_spell(ctx: &mut Ctx, pnum: usize, sn: SpellID) {
    match ctx.players.Players[pnum].executedSpell.spellType {
        SpellType::Skill | SpellType::Invalid => {}
        SpellType::Scroll => crate::inv::consume_scroll(ctx, pnum),
        SpellType::Charges => crate::inv::consume_staff_charge(ctx, pnum),
        SpellType::Spell => {
            let ma = get_mana_amount(ctx, &ctx.players.Players[pnum], sn);
            let player = &mut ctx.players.Players[pnum];
            player._pMana -= ma;
            player._pManaBase -= ma;
            redraw_component(ctx, PanelDrawComponent::Mana);
        }
    }
    if sn == SpellID::BloodStar {
        crate::player::apply_plr_damage(ctx, DamageType::Physical, pnum, 5, 0, 0, DeathReason::MonsterOrTrap);
    }
    if sn == SpellID::BoneSpirit {
        crate::player::apply_plr_damage(ctx, DamageType::Physical, pnum, 6, 0, 0, DeathReason::MonsterOrTrap);
    }
}

/// Original: `devilution::EnsureValidReadiedSpell` (spells.cpp).
// @port spells.cpp|devilution::EnsureValidReadiedSpell(Player &player) sha=60428be51a40
pub fn ensure_valid_readied_spell(ctx: &mut Ctx, pnum: usize) {
    if !is_readied_spell_valid(&ctx.players.Players[pnum]) {
        clear_readied_spell(ctx, pnum);
    }
}

/// Original: `devilution::CheckSpell` (spells.cpp).
// @port spells.cpp|devilution::CheckSpell(const Player &player, SpellID sn, SpellType st, bool manaonly) sha=c8e00503068d
pub fn check_spell(ctx: &Ctx, pnum: usize, sn: SpellID, st: SpellType, manaonly: bool) -> SpellCheckResult {
    if !manaonly && ctx.cursor.pcurs != crate::cursor::CURSOR_HAND {
        return SpellCheckResult::Fail_Busy;
    }
    if st == SpellType::Skill {
        return SpellCheckResult::Success;
    }
    let player = &ctx.players.Players[pnum];
    if player.get_spell_level(sn) <= 0 {
        return SpellCheckResult::Fail_Level0;
    }
    if player._pMana < get_mana_amount(ctx, player, sn) {
        return SpellCheckResult::Fail_NoMana;
    }
    SpellCheckResult::Success
}

/// Original: `devilution::CastSpell` (spells.cpp).
// @port spells.cpp|devilution::CastSpell(int id, SpellID spl, int sx, int sy, int dx, int dy, int spllvl) sha=b5780420feb5
pub fn cast_spell(ctx: &mut Ctx, id: i32, spl: SpellID, sx: i32, sy: i32, dx: i32, dy: i32, spllvl: i32) {
    let pnum = id as usize;
    let mut dir = ctx.players.Players[pnum]._pdir;
    if is_wall_spell(spl) {
        dir = ctx.players.Players[pnum].tempDirection;
    }
    let mut fizzled = false;
    let spell_data = get_spell_data(spl);
    for &mis in spell_data.sMissiles.iter() {
        if mis == MissileID::Null {
            break;
        }
        let missile = add_missile(ctx, Point::new(sx, sy), Point::new(dx, dy), dir, mis, TARGET_MONSTERS, id, 0, spllvl, None);
        fizzled |= missile.is_none();
    }
    if spl == SpellID::ChargedBolt {
        let mut i = (spllvl / 2) + 3;
        while i > 0 {
            let missile = add_missile(ctx, Point::new(sx, sy), Point::new(dx, dy), dir, MissileID::ChargedBolt, TARGET_MONSTERS, id, 0, spllvl, None);
            fizzled |= missile.is_none();
            i -= 1;
        }
    }
    if !fizzled {
        consume_spell(ctx, pnum, spl);
    }
}

/// Original: `devilution::DoResurrect` (spells.cpp).
// @port spells.cpp|devilution::DoResurrect(size_t pnum, Player &target) sha=5625d4cbd1c1
pub fn do_resurrect(ctx: &mut Ctx, pnum: usize, target: usize) {
    if pnum >= ctx.players.Players.len() {
        return;
    }
    let tile = ctx.players.Players[target].position.tile;
    add_missile(ctx, tile, tile, Direction::South, MissileID::ResurrectBeam, TARGET_MONSTERS, pnum as i32, 0, 0, None);

    if ctx.players.Players[target]._pHitPoints != 0 {
        return;
    }
    if ctx.players.MyPlayer == Some(target) {
        ctx.players.MyPlayerIsDead = false;
        crate::gamemenu::gamemenu_off(ctx);
        redraw_component(ctx, PanelDrawComponent::Health);
        redraw_component(ctx, PanelDrawComponent::Mana);
    }
    crate::player::clr_plr_path(ctx, target);
    ctx.players.Players[target].destAction = ACTION_NONE;
    ctx.players.Players[target]._pInvincible = false;
    crate::player::sync_init_plr_pos(ctx, target);

    let mut hp = 10 << 6;
    if ctx.players.Players[target]._pMaxHPBase < (10 << 6) {
        hp = ctx.players.Players[target]._pMaxHPBase;
    }
    crate::player::set_player_hit_points(ctx, target, hp);

    let t = &mut ctx.players.Players[target];
    t._pHPBase = t._pHitPoints + (t._pMaxHPBase - t._pMaxHP);
    t._pMana = 0;
    t._pManaBase = t._pMana + (t._pMaxManaBase - t._pMaxMana);
    t._pmode = PM_STAND;

    crate::items::calc_plr_inv(ctx, target, true);

    if crate::player::is_on_active_level(ctx, target) {
        let d = ctx.players.Players[target]._pdir;
        crate::player::start_stand(ctx, target, d);
    }
}

/// Original: `devilution::DoHealOther` (spells.cpp).
// @port spells.cpp|devilution::DoHealOther(const Player &caster, Player &target) sha=b676bce87374
pub fn do_heal_other(ctx: &mut Ctx, caster: usize, target: usize) {
    if (ctx.players.Players[target]._pHitPoints >> 6) <= 0 {
        return;
    }
    let mut hp = (ctx.rng.generate_rnd(10) + 1) << 6;
    for _ in 0..ctx.players.Players[caster]._pLevel as i32 {
        hp += (ctx.rng.generate_rnd(4) + 1) << 6;
    }
    for _ in 0..ctx.players.Players[caster].get_spell_level(SpellID::HealOther) {
        hp += (ctx.rng.generate_rnd(6) + 1) << 6;
    }
    match ctx.players.Players[caster]._pClass {
        HeroClass::Warrior | HeroClass::Barbarian => hp *= 2,
        HeroClass::Rogue | HeroClass::Bard => hp += hp / 2,
        HeroClass::Monk => hp *= 3,
        _ => {}
    }
    let t = &mut ctx.players.Players[target];
    t._pHitPoints = (t._pHitPoints + hp).min(t._pMaxHP);
    t._pHPBase = (t._pHPBase + hp).min(t._pMaxHPBase);
    if ctx.players.MyPlayer == Some(target) {
        redraw_component(ctx, PanelDrawComponent::Health);
    }
}

/// Original: `devilution::GetSpellBookLevel` (spells.cpp).
// @port spells.cpp|devilution::GetSpellBookLevel(SpellID s) sha=5d78f7120b76
pub fn get_spell_book_level(ctx: &Ctx, s: SpellID) -> i32 {
    if ctx.init.gb_is_spawn
        && matches!(s, SpellID::StoneCurse | SpellID::Guardian | SpellID::Golem | SpellID::Elemental | SpellID::BloodStar | SpellID::BoneSpirit)
    {
        return -1;
    }
    if !ctx.init.gb_is_hellfire {
        match s {
            SpellID::Nova | SpellID::Apocalypse => return -1,
            _ => {
                if s as i8 > SpellID::LastDiablo as i8 {
                    return -1;
                }
            }
        }
    }
    get_spell_data(s).sBookLvl as i32
}

/// Original: `devilution::GetSpellStaffLevel` (spells.cpp).
// @port spells.cpp|devilution::GetSpellStaffLevel(SpellID s) sha=23914a7c1276
pub fn get_spell_staff_level(ctx: &Ctx, s: SpellID) -> i32 {
    if ctx.init.gb_is_spawn
        && matches!(
            s,
            SpellID::StoneCurse | SpellID::Guardian | SpellID::Golem | SpellID::Apocalypse | SpellID::Elemental | SpellID::BloodStar | SpellID::BoneSpirit
        )
    {
        return -1;
    }
    if !ctx.init.gb_is_hellfire && s as i8 > SpellID::LastDiablo as i8 {
        return -1;
    }
    get_spell_data(s).sStaffLvl as i32
}

/// `CanUseScroll` (inv.cpp)
pub fn can_use_scroll(ctx: &Ctx, pnum: usize, spell: SpellID) -> bool {
    crate::inv::can_use_scroll(ctx, pnum, spell)
}

/// `CanUseStaff` (inv.cpp)
pub fn can_use_staff(ctx: &Ctx, pnum: usize, spell: SpellID) -> bool {
    crate::inv::can_use_staff(ctx, pnum, spell)
}

impl crate::tables::spelldat::SpellData {
    /// `SpellData::isTargeted`
    pub fn is_targeted(&self) -> bool {
        (self.flags.0 & SpellDataFlags::Targeted.0) == SpellDataFlags::Targeted.0
    }

    /// `SpellData::isAllowedInTown`
    pub fn is_allowed_in_town(&self) -> bool {
        (self.flags.0 & SpellDataFlags::AllowedInTown.0) == SpellDataFlags::AllowedInTown.0
    }
}
