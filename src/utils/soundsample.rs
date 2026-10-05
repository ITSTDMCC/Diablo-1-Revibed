//! `Source/utils/soundsample.cpp`: one playable sound (a decoded chunk or a stream).
//!
//! The audio output backend (SDL_audiolib in the original) is not wired up yet: `audio_init`
//! reports failure, so `gbSndInited` stays false and every sample fails to load, exactly the
//! path the original takes when `Aulib::init` fails. See "Known differences" in port/NOTES.md.

/// `SoundSample`
#[derive(Default)]
pub struct SoundSample {
    loaded: bool,
}

impl SoundSample {
    /// `Aulib::init`: no backend yet.
    pub fn audio_init(_sample_rate: i32, _channels: i32, _buffer_size: i32, _device: &str) -> Result<(), String> {
        Err("no audio output backend".to_string())
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn is_playing(&self) -> bool {
        false
    }

    /// `SetChunkStream`: 0 on success.
    pub fn set_chunk_stream(&mut self, _path: &str, _is_mp3: bool, _log_errors: bool) -> i32 {
        -1
    }

    /// `SetChunk`: 0 on success.
    pub fn set_chunk(&mut self, _data: Vec<u8>, _is_mp3: bool) -> i32 {
        -1
    }

    pub fn release(&mut self) {
        self.loaded = false;
    }

    pub fn stop(&mut self) {}

    pub fn play(&mut self, _num_iterations: i32) -> bool {
        false
    }

    pub fn play_with_volume_and_pan(&mut self, _log_sound_volume: i32, _log_user_volume: i32, _log_pan: i32) {}

    pub fn set_volume(&mut self, _volume: i32, _min: i32, _max: i32) {}

    pub fn mute(&mut self) {}

    pub fn unmute(&mut self) {}

    /// Length in milliseconds.
    pub fn get_length(&self) -> i32 {
        0
    }
}
