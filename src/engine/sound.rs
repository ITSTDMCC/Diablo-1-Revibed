//! `Source/engine/sound.cpp`: the audio pipeline (sound effects and music).

use crate::ctx::Ctx;
use crate::platform::log;
use crate::utils::soundsample::SoundSample;

/// `_music_id`
pub const TMUSIC_TOWN: u8 = 0;
pub const TMUSIC_CATHEDRAL: u8 = 1;
pub const TMUSIC_CATACOMBS: u8 = 2;
pub const TMUSIC_CAVES: u8 = 3;
pub const TMUSIC_HELL: u8 = 4;
pub const TMUSIC_NEST: u8 = 5;
pub const TMUSIC_CRYPT: u8 = 6;
pub const TMUSIC_INTRO: u8 = 7;
pub const NUM_MUSIC: u8 = 8;

pub const VOLUME_MIN: i32 = -1600;
pub const VOLUME_MAX: i32 = 0;

/// `TSnd`
pub struct TSnd {
    pub dsb: SoundSample,
    pub start_tc: u32,
}

impl TSnd {
    // @port engine/sound.h|devilution::TSnd::isPlaying() sha=bcd891dec0ee
    pub fn is_playing(&self) -> bool {
        self.dsb.is_playing()
    }
}

impl Drop for TSnd {
    /// Original: `TSnd::~TSnd` (engine/sound.cpp).
    // @port engine/sound.cpp|devilution::TSnd::~TSnd() sha=f9d05af49c93
    fn drop(&mut self) {
        if self.dsb.is_loaded() {
            self.dsb.stop();
        }
        self.dsb.release();
    }
}

/// Globals of sound.cpp.
pub struct SoundState {
    pub gb_snd_inited: bool,
    /// `sgnMusicTrack`
    pub sgn_music_track: u8,
    pub gb_music_on: bool,
    pub gb_sound_on: bool,
    /// `sgbSaveSoundOn` (monster.cpp)
    pub sgb_save_sound_on: bool,
    music: SoundSample,
    /// `duplicateSounds`: copies of sounds that were already playing when triggered again.
    duplicate_sounds: Vec<SoundSample>,
}

impl Default for SoundState {
    fn default() -> Self {
        SoundState { gb_snd_inited: false, sgn_music_track: NUM_MUSIC, gb_music_on: true, gb_sound_on: true, sgb_save_sound_on: false, music: SoundSample::default(), duplicate_sounds: Vec::new() }
    }
}

/// Original: `GetMp3Path` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::GetMp3Path(const char *path) sha=aab3e25f0178
fn get_mp3_path(path: &str) -> String {
    let dot = path.rfind('.').map_or(0, |d| d + 1);
    format!("{}mp3", &path[..dot])
}

/// Original: `LoadAudioFile` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::LoadAudioFile(const char *path, bool stream, bool errorDialog, SoundSample &result) sha=fa35a4c36827
fn load_audio_file(ctx: &mut Ctx, path: &str, stream: bool, error_dialog: bool, result: &mut SoundSample) -> bool {
    use crate::engine::assets::{find_asset, open_asset_ref};
    let mut is_mp3 = true;
    let mut found_path = get_mp3_path(path);
    let mut r = find_asset(ctx, &found_path);
    if !r.ok() {
        r = find_asset(ctx, path);
        found_path = path.to_string();
        is_mp3 = false;
    }
    if !r.ok() {
        crate::appfat::err_dlg(ctx, "Audio file not found", &format!("{path}\n\n"), file!(), line!() as i32);
    }
    if stream {
        if result.set_chunk_stream(ctx, &found_path, is_mp3, true) != 0 {
            if error_dialog {
                crate::appfat::err_dlg(ctx, "Failed to load audio file", &format!("{found_path}\n\n"), file!(), line!() as i32);
            }
            return false;
        }
    } else {
        let size = crate::engine::assets::asset_size(ctx, &r);
        let mut handle = open_asset_ref(ctx, r);
        if !handle.ok() {
            if error_dialog {
                crate::appfat::err_dlg(ctx, "Failed to load audio file", &format!("{found_path}\n\n"), file!(), line!() as i32);
            }
            return false;
        }
        let mut wave_file = vec![0u8; size];
        if !handle.read(&mut wave_file) {
            if error_dialog {
                crate::appfat::err_dlg(ctx, "Failed to read file", &format!("{found_path}: "), file!(), line!() as i32);
            }
            return false;
        }
        if result.set_chunk(wave_file, is_mp3) != 0 {
            if error_dialog {
                crate::appfat::err_dlg(ctx, "SDL Error", "", file!(), line!() as i32);
            }
            return false;
        }
    }
    true
}

/// `OptionEntryInt::SetValue` runs the value-changed callback immediately.
fn set_audio_option(ctx: &mut Ctx, f: impl FnOnce(&mut crate::options::AudioOptions) -> Option<crate::options::OptionCallback>) {
    if let Some(cb) = f(&mut ctx.options.audio) {
        crate::options::run_option_callback(ctx, cb);
    }
}

/// Original: `CapVolume` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::CapVolume(int volume) sha=40ca9933b0a8
fn cap_volume(volume: i32) -> i32 {
    volume.clamp(VOLUME_MIN, VOLUME_MAX)
}

/// Original: `DuplicateSound` (engine/sound.cpp). Finished duplicates are dropped here and in
/// `ClearDuplicateSounds` (the original erases them from a finish callback).
// @port engine/sound.cpp|devilution::DuplicateSound(const SoundSample &sound) sha=fb9c4623227f
fn duplicate_sound(ctx: &mut Ctx, sound: &SoundSample) -> Option<usize> {
    ctx.sound.duplicate_sounds.retain(|d| d.is_playing());
    let mut duplicate = SoundSample::default();
    if duplicate.duplicate_from(sound) != 0 {
        return None;
    }
    ctx.sound.duplicate_sounds.push(duplicate);
    Some(ctx.sound.duplicate_sounds.len() - 1)
}

/// Original: `devilution::snd_play_snd` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::snd_play_snd(TSnd *pSnd, int lVolume, int lPan) sha=46b6227f8447
pub fn snd_play_snd(ctx: &mut Ctx, p_snd: Option<&mut TSnd>, l_volume: i32, l_pan: i32) {
    let Some(p_snd) = p_snd else { return };
    if !ctx.sound.gb_sound_on {
        return;
    }
    let tc = ctx.platform.ticks();
    if tc.wrapping_sub(p_snd.start_tc) < 80 {
        return;
    }
    let user_volume = ctx.options.audio.sound_volume.get();
    if p_snd.dsb.is_playing() {
        let Some(i) = duplicate_sound(ctx, &p_snd.dsb) else { return };
        ctx.sound.duplicate_sounds[i].play_with_volume_and_pan(l_volume, user_volume, l_pan);
    } else {
        p_snd.dsb.play_with_volume_and_pan(l_volume, user_volume, l_pan);
    }
    p_snd.start_tc = tc;
}

/// Original: `devilution::sound_file_load` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::sound_file_load(const char *path, bool stream) sha=9d40015ed0b3
pub fn sound_file_load(ctx: &mut Ctx, path: &str, stream: bool) -> Box<TSnd> {
    let mut snd = Box::new(TSnd { dsb: SoundSample::default(), start_tc: ctx.platform.ticks().wrapping_sub(80 + 1) });
    load_audio_file(ctx, path, stream, true, &mut snd.dsb);
    snd
}

/// Original: `devilution::snd_init` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::snd_init() sha=b1cf5c3d6a0a
pub fn snd_init(ctx: &mut Ctx) {
    let v = cap_volume(ctx.options.audio.sound_volume.get());
    set_audio_option(ctx, |a| a.sound_volume.set_value(v));
    ctx.sound.gb_sound_on = ctx.options.audio.sound_volume.get() > VOLUME_MIN;
    ctx.sound.sgb_save_sound_on = ctx.sound.gb_sound_on;

    let v = cap_volume(ctx.options.audio.music_volume.get());
    set_audio_option(ctx, |a| a.music_volume.set_value(v));
    ctx.sound.gb_music_on = ctx.options.audio.music_volume.get() > VOLUME_MIN;

    let a = &ctx.options.audio;
    if let Err(e) = crate::platform::audio::init(a.sample_rate.get(), a.channels.get(), a.buffer_size.get(), &a.device.get()) {
        log::error!("Failed to initialize audio (Aulib::init): {}", e);
        return;
    }
    ctx.sound.gb_snd_inited = true;
}

/// Original: `devilution::snd_deinit` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::snd_deinit() sha=44476d98898a
pub fn snd_deinit(ctx: &mut Ctx) {
    if ctx.sound.gb_snd_inited {
        crate::platform::audio::quit();
        ctx.sound.duplicate_sounds.clear();
    }
    ctx.sound.gb_snd_inited = false;
}

pub fn gb_snd_inited(ctx: &Ctx) -> bool {
    ctx.sound.gb_snd_inited
}

/// Original: `devilution::GetLevelMusic` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::GetLevelMusic(dungeon_type dungeonType) sha=4a69ce93eb81
pub fn get_level_music(dungeon_type: crate::levels::gendung::DungeonType) -> u8 {
    use crate::levels::gendung::DungeonType::*;
    match dungeon_type {
        Town => TMUSIC_TOWN,
        Cathedral => TMUSIC_CATHEDRAL,
        Catacombs => TMUSIC_CATACOMBS,
        Caves => TMUSIC_CAVES,
        Hell => TMUSIC_HELL,
        Nest => TMUSIC_NEST,
        Crypt => TMUSIC_CRYPT,
        #[allow(unreachable_patterns)]
        _ => TMUSIC_INTRO,
    }
}

/// Original: `devilution::music_stop` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::music_stop() sha=a7e7bd636680
pub fn music_stop(ctx: &mut Ctx) {
    ctx.sound.music.release();
    ctx.sound.sgn_music_track = NUM_MUSIC;
}

/// `SpawnMusicTracks`
const SPAWN_MUSIC_TRACKS: [&str; NUM_MUSIC as usize] = [
    "music\\stowne.wav",
    "music\\slvla.wav",
    "music\\slvla.wav",
    "music\\slvla.wav",
    "music\\slvla.wav",
    "music\\dlvlf.wav",
    "music\\dlvle.wav",
    "music\\sintro.wav",
];

/// `MusicTracks`
const MUSIC_TRACKS: [&str; NUM_MUSIC as usize] = [
    "music\\dtowne.wav",
    "music\\dlvla.wav",
    "music\\dlvlb.wav",
    "music\\dlvlc.wav",
    "music\\dlvld.wav",
    "music\\dlvlf.wav",
    "music\\dlvle.wav",
    "music\\dintro.wav",
];

/// Original: `devilution::music_start` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::music_start(_music_id nTrack) sha=cacb53bedc69
pub fn music_start(ctx: &mut Ctx, n_track: u8) {
    assert!(n_track < NUM_MUSIC);
    music_stop(ctx);
    if !ctx.sound.gb_music_on {
        return;
    }
    let track_path = if ctx.init.archives.spawn_mpq.is_some() { SPAWN_MUSIC_TRACKS[n_track as usize] } else { MUSIC_TRACKS[n_track as usize] };
    let mut music = std::mem::take(&mut ctx.sound.music);
    let loaded = load_audio_file(ctx, track_path, true, false, &mut music);
    ctx.sound.music = music;
    if !loaded {
        music_stop(ctx);
        return;
    }
    let v = ctx.options.audio.music_volume.get();
    ctx.sound.music.set_volume(v, VOLUME_MIN, VOLUME_MAX);
    if !crate::diablo::diablo_is_focused(ctx) {
        music_mute(ctx);
    }
    if !ctx.sound.music.play(0) {
        log::error!("Aulib::Stream::play (from music_start)");
        music_stop(ctx);
        return;
    }
    ctx.sound.sgn_music_track = n_track;
}

/// Original: `devilution::sound_disable_music` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::sound_disable_music(bool disable) sha=f1a3c758e0a8
pub fn sound_disable_music(ctx: &mut Ctx, disable: bool) {
    if disable {
        music_stop(ctx);
    } else if ctx.sound.sgn_music_track != NUM_MUSIC {
        let t = ctx.sound.sgn_music_track;
        music_start(ctx, t);
    }
}

/// Original: `devilution::sound_get_or_set_music_volume` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::sound_get_or_set_music_volume(int volume) sha=99f8d367ff16
pub fn sound_get_or_set_music_volume(ctx: &mut Ctx, volume: i32) -> i32 {
    if volume == 1 {
        return ctx.options.audio.music_volume.get();
    }
    set_audio_option(ctx, |a| a.music_volume.set_value(volume));
    if ctx.sound.music.is_loaded() {
        let v = ctx.options.audio.music_volume.get();
        ctx.sound.music.set_volume(v, VOLUME_MIN, VOLUME_MAX);
    }
    ctx.options.audio.music_volume.get()
}

/// Original: `devilution::sound_get_or_set_sound_volume` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::sound_get_or_set_sound_volume(int volume) sha=7448d3517a4d
pub fn sound_get_or_set_sound_volume(ctx: &mut Ctx, volume: i32) -> i32 {
    if volume == 1 {
        return ctx.options.audio.sound_volume.get();
    }
    set_audio_option(ctx, |a| a.sound_volume.set_value(volume));
    ctx.options.audio.sound_volume.get()
}

/// Original: `devilution::music_mute` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::music_mute() sha=8bead310bd8a
pub fn music_mute(ctx: &mut Ctx) {
    if ctx.sound.music.is_loaded() {
        ctx.sound.music.mute();
    }
}

/// Original: `devilution::music_unmute` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::music_unmute() sha=24f73a80e391
pub fn music_unmute(ctx: &mut Ctx) {
    if ctx.sound.music.is_loaded() {
        ctx.sound.music.unmute();
    }
}

/// Original: `devilution::ClearDuplicateSounds` (engine/sound.cpp).
// @port engine/sound.cpp|devilution::ClearDuplicateSounds() sha=6f6d279ee71c
pub fn clear_duplicate_sounds(ctx: &mut Ctx) {
    ctx.sound.duplicate_sounds.clear();
}

pub const ATTENUATION_MIN: i32 = -6400;
pub const PAN_MIN: i32 = -6400;
pub const PAN_MAX: i32 = 6400;

/// Original: `devilution::CalculateSoundPosition` (engine/sound_position.cpp).
// @port engine/sound_position.cpp|devilution::CalculateSoundPosition(Point soundPosition, int *plVolume, int *plPan) sha=cf1bb18ba2f4
pub fn calculate_sound_position(ctx: &Ctx, sound_position: crate::engine::geometry::Point, pl_volume: &mut i32, pl_pan: &mut i32) -> bool {
    let me = ctx.players.MyPlayer.expect("MyPlayer");
    let player_position = ctx.players.Players[me].position.tile;
    let delta = sound_position - player_position;
    let pan = (delta.delta_x - delta.delta_y) * 256;
    *pl_pan = pan.clamp(PAN_MIN, PAN_MAX);
    let volume = player_position.approx_distance(sound_position) * -64;
    if volume <= ATTENUATION_MIN {
        return false;
    }
    *pl_volume = volume;
    true
}
