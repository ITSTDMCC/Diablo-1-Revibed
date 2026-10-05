//! `Source/effects.cpp`: loading and playing sound effects (monster sounds and music excluded).

#![allow(non_upper_case_globals)]

use crate::ctx::Ctx;
use crate::effects_data::{SfxId, NUM_SFX, SFX_DATA};
use crate::engine::sound::{snd_play_snd, sound_file_load, sound_get_or_set_sound_volume, TSnd, VOLUME_MAX, VOLUME_MIN};

pub use crate::effects_data::*;
use crate::engine::geometry::Point;
use crate::enums::HeroClass;

/// `sfx_flag`
pub const sfx_STREAM: u8 = 1 << 0;
pub const sfx_MISC: u8 = 1 << 1;
pub const sfx_UI: u8 = 1 << 2;
pub const sfx_MONK: u8 = 1 << 3;
pub const sfx_ROGUE: u8 = 1 << 4;
pub const sfx_WARRIOR: u8 = 1 << 5;
pub const sfx_SORCERER: u8 = 1 << 6;
pub const sfx_HELLFIRE: u8 = 1 << 7;

/// `AllowStreaming` (DISABLE_STREAMING_SOUNDS is not defined in the Windows build)
const ALLOW_STREAMING: bool = true;

/// Globals of effects.cpp. `sgSFX[i].pSnd` is `p_snd[i]`; flags and names are `SFX_DATA`.
pub struct EffectsState {
    pub sfxdelay: i32,
    pub sfxdnum: SfxId,
    p_snd: Vec<Option<Box<TSnd>>>,
    /// `sgpStreamSFX` (index into sgSFX)
    sgp_stream_sfx: Option<usize>,
}

impl Default for EffectsState {
    fn default() -> Self {
        EffectsState { sfxdelay: 0, sfxdnum: crate::effects_data::SFX_NONE, p_snd: (0..NUM_SFX).map(|_| None).collect(), sgp_stream_sfx: None }
    }
}

/// Original: `StreamPlay` (effects.cpp).
// @port effects.cpp|devilution::StreamPlay(TSFX *pSFX, int lVolume, int lPan) sha=01ba8726e150
fn stream_play(ctx: &mut Ctx, sfx: usize, mut l_volume: i32, l_pan: i32) {
    assert!(SFX_DATA[sfx].0 & sfx_STREAM != 0);
    stream_stop(ctx);
    if l_volume >= VOLUME_MIN {
        if l_volume > VOLUME_MAX {
            l_volume = VOLUME_MAX;
        }
        if ctx.effects.p_snd[sfx].is_none() {
            let s = sound_file_load(ctx, SFX_DATA[sfx].1, ALLOW_STREAMING);
            ctx.effects.p_snd[sfx] = Some(s);
        }
        let user = sound_get_or_set_sound_volume(ctx, 1);
        let snd = ctx.effects.p_snd[sfx].as_mut().unwrap();
        if snd.dsb.is_loaded() {
            snd.dsb.play_with_volume_and_pan(l_volume, user, l_pan);
        }
        ctx.effects.sgp_stream_sfx = Some(sfx);
    }
}

/// Original: `StreamUpdate` (effects.cpp).
// @port effects.cpp|devilution::StreamUpdate() sha=4bc780595f93
fn stream_update(ctx: &mut Ctx) {
    if let Some(s) = ctx.effects.sgp_stream_sfx {
        if !ctx.effects.p_snd[s].as_ref().is_some_and(|p| p.is_playing()) {
            stream_stop(ctx);
        }
    }
}

/// Original: `PrivSoundInit` (effects.cpp).
// @port effects.cpp|devilution::PrivSoundInit(uint8_t bLoadMask) sha=30ccc7e92bef
fn priv_sound_init(ctx: &mut Ctx, b_load_mask: u8) {
    if !ctx.sound.gb_snd_inited {
        return;
    }
    for (i, &(flags, name)) in SFX_DATA.iter().enumerate() {
        if flags == 0 || ctx.effects.p_snd[i].is_some() {
            continue;
        }
        if flags & sfx_STREAM != 0 {
            continue;
        }
        if flags & b_load_mask == 0 {
            continue;
        }
        if !ctx.init.gb_is_hellfire && flags & sfx_HELLFIRE != 0 {
            continue;
        }
        let s = sound_file_load(ctx, name, false);
        ctx.effects.p_snd[i] = Some(s);
    }
}

/// Original: `devilution::effect_is_playing` (effects.cpp).
// @port effects.cpp|devilution::effect_is_playing(int nSFX) sha=900401587ee9
pub fn effect_is_playing(ctx: &Ctx, n_sfx: i32) -> bool {
    let i = n_sfx as usize;
    if let Some(s) = &ctx.effects.p_snd[i] {
        return s.is_playing();
    }
    if SFX_DATA[i].0 & sfx_STREAM != 0 {
        return ctx.effects.sgp_stream_sfx == Some(i);
    }
    false
}

/// Original: `devilution::stream_stop` (effects.cpp).
// @port effects.cpp|devilution::stream_stop() sha=c6f841838a98
pub fn stream_stop(ctx: &mut Ctx) {
    if let Some(s) = ctx.effects.sgp_stream_sfx.take() {
        ctx.effects.p_snd[s] = None;
    }
}

/// Original: `devilution::sound_stop` (effects.cpp).
// @port effects.cpp|devilution::sound_stop() sha=59b3b678d3ff
pub fn sound_stop(ctx: &mut Ctx) {
    if !ctx.sound.gb_snd_inited {
        return;
    }
    crate::engine::sound::clear_duplicate_sounds(ctx);
    for s in ctx.effects.p_snd.iter_mut().flatten() {
        if s.dsb.is_loaded() {
            s.dsb.stop();
        }
    }
}

/// Original: `devilution::sound_update` (effects.cpp).
// @port effects.cpp|devilution::sound_update() sha=f60dd290a92b
pub fn sound_update(ctx: &mut Ctx) {
    if !ctx.sound.gb_snd_inited {
        return;
    }
    stream_update(ctx);
}

/// Original: `devilution::effects_cleanup_sfx` (effects.cpp).
// @port effects.cpp|devilution::effects_cleanup_sfx() sha=9281178192e4
pub fn effects_cleanup_sfx(ctx: &mut Ctx) {
    sound_stop(ctx);
    for s in ctx.effects.p_snd.iter_mut() {
        *s = None;
    }
}

/// Original: `devilution::sound_init` (effects.cpp).
// @port effects.cpp|devilution::sound_init() sha=c38ab99f3981
pub fn sound_init(ctx: &mut Ctx) {
    let mut mask = sfx_MISC;
    if ctx.init.gb_is_multiplayer {
        mask |= sfx_WARRIOR;
        if !ctx.init.gb_is_spawn {
            mask |= sfx_ROGUE | sfx_SORCERER;
        }
        if ctx.init.gb_is_hellfire {
            mask |= sfx_MONK;
        }
    } else {
        let me = ctx.players.MyPlayer.expect("MyPlayer");
        match ctx.players.Players[me]._pClass {
            HeroClass::Warrior | HeroClass::Barbarian => mask |= sfx_WARRIOR,
            HeroClass::Rogue | HeroClass::Bard => mask |= sfx_ROGUE,
            HeroClass::Sorcerer => mask |= sfx_SORCERER,
            HeroClass::Monk => mask |= sfx_MONK,
            #[allow(unreachable_patterns)]
            _ => crate::appfat::app_fatal(ctx, "effects:1"),
        }
    }
    priv_sound_init(ctx, mask);
}

/// Original: `devilution::ui_sound_init` (effects.cpp).
// @port effects.cpp|devilution::ui_sound_init() sha=27b8890011b3
pub fn ui_sound_init(ctx: &mut Ctx) {
    priv_sound_init(ctx, sfx_UI);
}

/// Original: `devilution::effects_play_sound` (effects.cpp).
// @port effects.cpp|devilution::effects_play_sound(_sfx_id id) sha=4f5e583c6763
pub fn effects_play_sound(ctx: &mut Ctx, id: SfxId) {
    if !ctx.sound.gb_snd_inited || !ctx.sound.gb_sound_on {
        return;
    }
    let i = id as usize;
    if ctx.effects.p_snd[i].as_ref().is_some_and(|s| !s.is_playing()) {
        let mut snd = ctx.effects.p_snd[i].take();
        snd_play_snd(ctx, snd.as_deref_mut(), 0, 0);
        ctx.effects.p_snd[i] = snd;
    }
}

/// Original: `devilution::GetSFXLength` (effects.cpp).
// @port effects.cpp|devilution::GetSFXLength(int nSFX) sha=beff78bd790d
pub fn get_sfx_length(ctx: &mut Ctx, n_sfx: i32) -> i32 {
    let i = n_sfx as usize;
    if ctx.effects.p_snd[i].is_none() {
        let s = sound_file_load(ctx, SFX_DATA[i].1, ALLOW_STREAMING && SFX_DATA[i].0 & sfx_STREAM != 0);
        ctx.effects.p_snd[i] = Some(s);
    }
    ctx.effects.p_snd[i].as_ref().unwrap().dsb.get_length()
}


/// Original: `PlaySfxPriv` (effects.cpp).
// @port effects.cpp|devilution::PlaySfxPriv(TSFX *pSFX, bool loc, Point position) sha=e7476641f774
fn play_sfx_priv(ctx: &mut Ctx, sfx: usize, loc: bool, position: Point) {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    if ctx.players.Players[me].pLvlLoad != 0 && ctx.init.gb_is_multiplayer {
        return;
    }
    if !ctx.sound.gb_snd_inited || !ctx.sound.gb_sound_on || ctx.msg.gbBufferMsgs != 0 {
        return;
    }
    let flags = SFX_DATA[sfx].0;
    if flags & (sfx_STREAM | sfx_MISC) == 0 && ctx.effects.p_snd[sfx].as_ref().is_some_and(|s| s.is_playing()) {
        return;
    }
    let mut l_volume = 0;
    let mut l_pan = 0;
    if loc && !crate::engine::sound::calculate_sound_position(ctx, position, &mut l_volume, &mut l_pan) {
        return;
    }
    if flags & sfx_STREAM != 0 {
        stream_play(ctx, sfx, l_volume, l_pan);
        return;
    }
    if ctx.effects.p_snd[sfx].is_none() {
        let s = sound_file_load(ctx, SFX_DATA[sfx].1, false);
        ctx.effects.p_snd[sfx] = Some(s);
    }
    if ctx.effects.p_snd[sfx].as_ref().is_some_and(|s| s.dsb.is_loaded()) {
        let mut snd = ctx.effects.p_snd[sfx].take();
        snd_play_snd(ctx, snd.as_deref_mut(), l_volume, l_pan);
        ctx.effects.p_snd[sfx] = snd;
    }
}

/// Original: `devilution::RndSFX` (effects.cpp).
// @port effects.cpp|devilution::RndSFX(_sfx_id psfx) sha=9edd70037f02
pub fn rnd_sfx(ctx: &mut Ctx, psfx: SfxId) -> SfxId {
    let n_rand = match psfx {
        PS_WARR69 | PS_MAGE69 | PS_ROGUE69 | PS_MONK69 | PS_SWING | LS_ACID | IS_MAGIC | IS_BHIT => 2,
        PS_WARR14 | PS_WARR15 | PS_WARR16 | PS_WARR2 | PS_ROGUE14 | PS_MAGE14 | PS_MONK14 => 3,
        _ => return psfx,
    };
    psfx + ctx.rng.generate_rnd(n_rand) as SfxId
}

/// Original: `devilution::PlaySFX` (effects.cpp).
// @port effects.cpp|devilution::PlaySFX(_sfx_id psfx) sha=282da8d375f0
pub fn play_sfx(ctx: &mut Ctx, psfx: SfxId) {
    let psfx = rnd_sfx(ctx, psfx);
    play_sfx_priv(ctx, psfx as usize, false, Point::new(0, 0));
}

/// Original: `devilution::PlaySfxLoc` (effects.cpp).
// @port effects.cpp|devilution::PlaySfxLoc(_sfx_id psfx, Point position, bool randomizeByCategory) sha=cd8ff34926e6
pub fn play_sfx_loc(ctx: &mut Ctx, mut psfx: SfxId, position: Point, randomize_by_category: bool) {
    if randomize_by_category {
        psfx = rnd_sfx(ctx, psfx);
    }
    if (0..=3).contains(&psfx) {
        if let Some(snd) = ctx.effects.p_snd[psfx as usize].as_mut() {
            snd.start_tc = 0;
        }
    }
    play_sfx_priv(ctx, psfx as usize, true, position);
}
