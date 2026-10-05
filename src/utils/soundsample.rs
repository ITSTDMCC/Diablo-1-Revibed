//! `Source/utils/soundsample.cpp`: one playable sound (a decoded chunk or a "stream").

use std::sync::Arc;

use crate::ctx::Ctx;
use crate::platform::audio::{decode_wav, Pcm, Stream};
use crate::platform::log;

const LOG_BASE: f32 = 10.0;
/// Picked so that a volume change of -10 dB results in half perceived loudness.
const VOLUME_SCALE: f32 = 3321.9281;
/// -100 dB (muted) to 0 dB (max. loudness), in millibel.
const MILLIBEL_MIN: f32 = -10000.0;
const MILLIBEL_MAX: f32 = 0.0;
/// Stereo separation factor for left/right speaker panning.
const STEREO_SEPARATION: f32 = 6000.0;

/// `ATTENUATION_MIN` (engine/sound_defs.hpp)
pub const ATTENUATION_MIN: i32 = -6400;

/// Original: `PanLogToLinear` (utils/soundsample.cpp).
// @port utils/soundsample.cpp|devilution::PanLogToLinear(int logPan) sha=08695714def1
fn pan_log_to_linear(log_pan: i32) -> f32 {
    if log_pan == 0 {
        return 0.0;
    }
    let factor = LOG_BASE.powf(-(log_pan.abs() as f32) / STEREO_SEPARATION);
    (1.0 - factor).copysign(log_pan as f32)
}

/// `math::Remap`
fn remap(in_min: f32, in_max: f32, out_min: f32, out_max: f32, v: f32) -> f32 {
    out_min + (v - in_min) * (out_max - out_min) / (in_max - in_min)
}

/// Original: `VolumeLogToLinear` (utils/soundsample.cpp).
// @port utils/soundsample.cpp|devilution::VolumeLogToLinear(int logVolume, int logMin, int logMax) sha=146f3e98ac70
fn volume_log_to_linear(log_volume: i32, log_min: i32, log_max: i32) -> f32 {
    let log_scaled = remap(log_min as f32, log_max as f32, MILLIBEL_MIN, MILLIBEL_MAX, log_volume as f32);
    LOG_BASE.powf(log_scaled / VOLUME_SCALE)
}

/// Original: `CreateStream` (utils/soundsample.cpp): MP3 needs a decoder the port does not have.
// @port utils/soundsample.cpp|devilution::CreateStream(SDL_RWops *handle, bool isMp3) sha=b2aabffda3e1
fn create_stream(data: &[u8], is_mp3: bool) -> Result<Stream, String> {
    if is_mp3 {
        return Err("MP3 audio is not supported".into());
    }
    let pcm: Pcm = decode_wav(data)?;
    Ok(Stream::new(Arc::new(pcm)))
}

/// `SoundSample`
#[derive(Default)]
pub struct SoundSample {
    /// Set for streaming audio to allow for duplicating it.
    file_path: String,
    is_mp3: bool,
    streaming: bool,
    stream: Option<Stream>,
}

impl SoundSample {
    pub fn is_loaded(&self) -> bool {
        self.stream.is_some()
    }

    /// Original: `SoundSample::Release` (utils/soundsample.cpp).
    // @port utils/soundsample.cpp|devilution::SoundSample::Release() sha=41eb7c621dc8
    pub fn release(&mut self) {
        self.stream = None;
    }

    /// Original: `SoundSample::IsPlaying` (utils/soundsample.cpp).
    // @port utils/soundsample.cpp|devilution::SoundSample::IsPlaying() sha=573032431a8c
    pub fn is_playing(&self) -> bool {
        self.stream.as_ref().is_some_and(|s| s.is_playing())
    }

    pub fn is_streaming(&self) -> bool {
        self.streaming
    }

    /// Original: `SoundSample::Play` (utils/soundsample.cpp). 0 iterations loops.
    // @port utils/soundsample.cpp|devilution::SoundSample::Play(int numIterations) sha=9348b139e6e2
    pub fn play(&mut self, num_iterations: i32) -> bool {
        self.stream.as_ref().expect("stream").play(num_iterations)
    }

    /// Original: `SoundSample::SetChunkStream` (utils/soundsample.cpp). The file is decoded up
    /// front instead of streamed.
    // @port utils/soundsample.cpp|devilution::SoundSample::SetChunkStream(std::string filePath, bool isMp3, bool logErrors) sha=9459943e599c
    pub fn set_chunk_stream(&mut self, ctx: &mut Ctx, file_path: &str, is_mp3: bool, log_errors: bool) -> i32 {
        let handle = crate::engine::assets::open_asset(ctx, file_path);
        if !handle.ok() {
            if log_errors {
                log::error!("OpenAsset failed (from SoundSample::SetChunkStream) for {}", file_path);
            }
            return -1;
        }
        let data = handle.into_bytes();
        self.file_path = file_path.to_string();
        self.is_mp3 = is_mp3;
        self.streaming = true;
        match create_stream(&data, is_mp3) {
            Ok(s) => {
                self.stream = Some(s);
                0
            }
            Err(e) => {
                self.stream = None;
                if log_errors {
                    log::error!("Aulib::Stream::open (from SoundSample::SetChunkStream) for {}: {}", file_path, e);
                }
                -1
            }
        }
    }

    /// Original: `SoundSample::SetChunk` (utils/soundsample.cpp).
    // @port utils/soundsample.cpp|devilution::SoundSample::SetChunk(ArraySharedPtr<std::uint8_t> fileData, std::size_t dwBytes, bool isMp3) sha=5dd6b13b73f2
    pub fn set_chunk(&mut self, file_data: Vec<u8>, is_mp3: bool) -> i32 {
        self.is_mp3 = is_mp3;
        self.streaming = false;
        match create_stream(&file_data, is_mp3) {
            Ok(s) => {
                self.stream = Some(s);
                0
            }
            Err(e) => {
                self.stream = None;
                log::error!("Aulib::Stream::open (from SoundSample::SetChunk): {}", e);
                -1
            }
        }
    }

    /// `SoundSample::DuplicateFrom`: shares the decoded samples.
    pub fn duplicate_from(&mut self, other: &SoundSample) -> i32 {
        match &other.stream {
            Some(s) => {
                self.file_path = other.file_path.clone();
                self.is_mp3 = other.is_mp3;
                self.streaming = other.streaming;
                self.stream = Some(Stream::new(s.pcm()));
                0
            }
            None => -1,
        }
    }

    /// `SoundSample::Stop`
    pub fn stop(&mut self) {
        self.stream.as_ref().expect("stream").stop();
    }

    /// `SoundSample::PlayWithVolumeAndPan`
    pub fn play_with_volume_and_pan(&mut self, log_sound_volume: i32, log_user_volume: i32, log_pan: i32) -> bool {
        self.set_volume(log_sound_volume + log_user_volume * (ATTENUATION_MIN / crate::engine::sound::VOLUME_MIN), ATTENUATION_MIN, 0);
        self.set_stereo_position(log_pan);
        self.play(1)
    }

    /// Original: `SoundSample::SetVolume` (utils/soundsample.cpp).
    // @port utils/soundsample.cpp|devilution::SoundSample::SetVolume(int logVolume, int logMin, int logMax) sha=2f95bafc3231
    pub fn set_volume(&mut self, log_volume: i32, log_min: i32, log_max: i32) {
        self.stream.as_ref().expect("stream").set_volume(volume_log_to_linear(log_volume, log_min, log_max));
    }

    /// Original: `SoundSample::SetStereoPosition` (utils/soundsample.cpp).
    // @port utils/soundsample.cpp|devilution::SoundSample::SetStereoPosition(int logPan) sha=340bc5278fcf
    pub fn set_stereo_position(&mut self, log_pan: i32) {
        self.stream.as_ref().expect("stream").set_stereo_position(pan_log_to_linear(log_pan));
    }

    pub fn mute(&mut self) {
        self.stream.as_ref().expect("stream").mute();
    }

    pub fn unmute(&mut self) {
        self.stream.as_ref().expect("stream").unmute();
    }

    /// Original: `SoundSample::GetLength` (utils/soundsample.cpp): duration in ms.
    // @port utils/soundsample.cpp|devilution::SoundSample::GetLength() sha=c7cac83ea70d
    pub fn get_length(&self) -> i32 {
        match &self.stream {
            None => 0,
            Some(s) => s.pcm().duration_ms(),
        }
    }
}
