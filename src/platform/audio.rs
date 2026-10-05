//! Audio output: the role SDL_audiolib (Aulib) plays for DevilutionX. A cpal output stream
//! mixes every playing `Stream` (decoded PCM with volume, stereo position, mute and looping).
//!
//! Differences from Aulib (see port/NOTES.md): sounds are decoded fully when loaded (Aulib streams
//! music from the archive), resampling is linear (Aulib uses Speex by default), and muting is
//! immediate (Aulib fades).

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock, Weak};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Decoded PCM, interleaved, normalised to [-1, 1].
pub struct Pcm {
    pub samples: Vec<f32>,
    pub channels: u16,
    pub rate: u32,
}

impl Pcm {
    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels.max(1) as usize
    }

    /// Duration in milliseconds.
    pub fn duration_ms(&self) -> i32 {
        (self.frames() as u64 * 1000 / self.rate.max(1) as u64) as i32
    }
}

/// Decodes a RIFF/WAVE file (PCM 8/16/24/32-bit, IEEE float), as dr_wav does for the game's files.
pub fn decode_wav(data: &[u8]) -> Result<Pcm, String> {
    if data.len() < 12 || &data[0..4] != b"RIFF" || &data[8..12] != b"WAVE" {
        return Err("not a RIFF/WAVE file".into());
    }
    let mut fmt: Option<(u16, u16, u32, u16)> = None;
    let mut p = 12;
    while p + 8 <= data.len() {
        let id = &data[p..p + 4];
        let len = u32::from_le_bytes(data[p + 4..p + 8].try_into().unwrap()) as usize;
        let body = &data[p + 8..(p + 8 + len).min(data.len())];
        if id == b"fmt " {
            if body.len() < 16 {
                return Err("short fmt chunk".into());
            }
            let mut tag = u16::from_le_bytes([body[0], body[1]]);
            let ch = u16::from_le_bytes([body[2], body[3]]);
            let rate = u32::from_le_bytes([body[4], body[5], body[6], body[7]]);
            let bits = u16::from_le_bytes([body[14], body[15]]);
            if tag == 0xfffe && body.len() >= 26 {
                tag = u16::from_le_bytes([body[24], body[25]]); // WAVE_FORMAT_EXTENSIBLE sub-format
            }
            fmt = Some((tag, ch, rate, bits));
        } else if id == b"data" {
            let (tag, ch, rate, bits) = fmt.ok_or("data before fmt")?;
            if ch == 0 {
                return Err("no channels".into());
            }
            let samples: Vec<f32> = match (tag, bits) {
                (1, 8) => body.iter().map(|&b| (b as f32 - 128.0) / 128.0).collect(),
                (1, 16) => body.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0).collect(),
                (1, 24) => body.chunks_exact(3).map(|c| (i32::from_le_bytes([0, c[0], c[1], c[2]]) >> 8) as f32 / 8_388_608.0).collect(),
                (1, 32) => body.chunks_exact(4).map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]) as f32 / 2_147_483_648.0).collect(),
                (3, 32) => body.chunks_exact(4).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect(),
                _ => return Err(format!("unsupported WAV format {tag:#x} with {bits} bits")),
            };
            return Ok(Pcm { samples, channels: ch, rate });
        }
        p += 8 + len + (len & 1);
    }
    Err("no data chunk".into())
}

/// Playback state of one `Aulib::Stream`.
pub struct StreamState {
    pcm: Arc<Pcm>,
    /// Position in source frames.
    pos: f64,
    playing: bool,
    /// Remaining iterations; 0 = loop forever.
    iterations: i32,
    volume: f32,
    /// -1 (left) .. 1 (right)
    stereo_position: f32,
    muted: bool,
    /// `PushAulibDecoder`'s queue (`AudioQueueItem`s, as interleaved samples) for a push stream;
    /// `pcm` then only carries the channel count and rate.
    push: Option<VecDeque<f32>>,
}

/// `Aulib::Stream`: owned by a `SoundSample` on the game thread, mixed on the audio thread.
pub struct Stream(Arc<Mutex<StreamState>>);

impl Stream {
    pub fn new(pcm: Arc<Pcm>) -> Stream {
        Self::with_state(StreamState { pcm, pos: 0.0, playing: false, iterations: 1, volume: 1.0, stereo_position: 0.0, muted: false, push: None })
    }

    fn with_state(state: StreamState) -> Stream {
        let s = Arc::new(Mutex::new(state));
        if let Some(m) = MIXER.get() {
            m.lock().unwrap().streams.push(Arc::downgrade(&s));
        }
        Stream(s)
    }

    /// `Aulib::Stream(nullptr, std::make_unique<PushAulibDecoder>(numChannels, sampleRate), ...)`
    // @port utils/push_aulib_decoder.h|devilution::PushAulibDecoder::PushAulibDecoder(int numChannels, int sampleRate) sha=7e6c3bd370f7
    // @port utils/push_aulib_decoder.h|devilution::PushAulibDecoder::getChannels() sha=65e6e0a472ae
    // @port utils/push_aulib_decoder.h|devilution::PushAulibDecoder::getRate() sha=a5aafc69672a
    // @port utils/push_aulib_decoder.cpp|devilution::PushAulibDecoder::open([[maybe_unused]] SDL_RWops *rwops) sha=c48e92da7025
    pub fn new_push(num_channels: u16, sample_rate: u32) -> Stream {
        let pcm = Arc::new(Pcm { samples: Vec::new(), channels: num_channels, rate: sample_rate });
        Self::with_state(StreamState { pcm, pos: 0.0, playing: false, iterations: 0, volume: 1.0, stereo_position: 0.0, muted: false, push: Some(VecDeque::new()) })
    }

    /// Original: `PushAulibDecoder::PushSamples(const std::int16_t *, unsigned)`.
    // @port utils/push_aulib_decoder.cpp|devilution::PushAulibDecoder::PushSamples(const std::int16_t *data, unsigned size) sha=ba89baddb60a
    pub fn push_samples_i16(&self, data: &[i16]) {
        const SCALE: f32 = i16::MAX as f32 + 1.0;
        if let Some(q) = self.0.lock().unwrap().push.as_mut() {
            q.extend(data.iter().map(|&v| v as f32 / SCALE));
        }
    }

    /// Original: `PushAulibDecoder::PushSamples(const std::uint8_t *, unsigned)`.
    // @port utils/push_aulib_decoder.cpp|devilution::PushAulibDecoder::PushSamples(const std::uint8_t *data, unsigned size) sha=07abfaa35bfb
    pub fn push_samples_u8(&self, data: &[u8]) {
        let samples: Vec<i16> = data.iter().map(|&v| ((v as i16) - 128) * 256).collect();
        self.push_samples_i16(&samples);
    }

    /// Original: `PushAulibDecoder::DiscardPendingSamples`.
    // @port utils/push_aulib_decoder.cpp|devilution::PushAulibDecoder::DiscardPendingSamples() sha=ee25cd0cfaff
    pub fn discard_pending_samples(&self) {
        if let Some(q) = self.0.lock().unwrap().push.as_mut() {
            q.clear();
        }
    }

    pub fn pcm(&self) -> Arc<Pcm> {
        self.0.lock().unwrap().pcm.clone()
    }

    /// `play(iterations)`: restarts from the beginning; 0 iterations loops forever.
    pub fn play(&self, iterations: i32) -> bool {
        let mut s = self.0.lock().unwrap();
        s.pos = 0.0;
        s.iterations = iterations;
        s.playing = true;
        true
    }

    pub fn stop(&self) {
        let mut s = self.0.lock().unwrap();
        s.playing = false;
        s.pos = 0.0;
    }

    pub fn is_playing(&self) -> bool {
        self.0.lock().unwrap().playing
    }

    pub fn set_volume(&self, v: f32) {
        self.0.lock().unwrap().volume = v;
    }

    pub fn set_stereo_position(&self, p: f32) {
        self.0.lock().unwrap().stereo_position = p.clamp(-1.0, 1.0);
    }

    pub fn mute(&self) {
        self.0.lock().unwrap().muted = true;
    }

    pub fn unmute(&self) {
        self.0.lock().unwrap().muted = false;
    }
}

struct MixerState {
    streams: Vec<Weak<Mutex<StreamState>>>,
    out_rate: u32,
    out_channels: u16,
}

static MIXER: OnceLock<Arc<Mutex<MixerState>>> = OnceLock::new();

thread_local! {
    /// The cpal stream must stay alive (and on the thread that made it).
    static OUTPUT: RefCell<Option<cpal::Stream>> = const { RefCell::new(None) };
}

fn mix(state: &mut MixerState, out: &mut [f32]) {
    out.fill(0.0);
    let och = state.out_channels.max(1) as usize;
    let out_rate = state.out_rate as f64;
    state.streams.retain(|w| w.strong_count() > 0);
    for w in &state.streams {
        let Some(s) = w.upgrade() else { continue };
        let mut s = s.lock().unwrap();
        if !s.playing {
            continue;
        }
        if s.push.is_some() {
            mix_push(&mut s, out, och, out_rate);
            continue;
        }
        let pcm = s.pcm.clone();
        let ich = pcm.channels.max(1) as usize;
        let frames = pcm.frames();
        if frames == 0 {
            s.playing = false;
            continue;
        }
        let step = pcm.rate as f64 / out_rate;
        let gain = if s.muted { 0.0 } else { s.volume };
        let (gl, gr) = (gain * if s.stereo_position > 0.0 { 1.0 - s.stereo_position } else { 1.0 }, gain * if s.stereo_position < 0.0 { 1.0 + s.stereo_position } else { 1.0 });
        for frame in out.chunks_exact_mut(och) {
            if !s.playing {
                break;
            }
            let i = s.pos as usize;
            let t = (s.pos - i as f64) as f32;
            let j = if i + 1 < frames { i + 1 } else { i };
            let sample = |c: usize| {
                let c = c.min(ich - 1);
                let a = pcm.samples[i * ich + c];
                let b = pcm.samples[j * ich + c];
                a + (b - a) * t
            };
            let (l, r) = if ich == 1 {
                let v = sample(0);
                (v, v)
            } else {
                (sample(0), sample(1))
            };
            if och == 1 {
                frame[0] += (l * gl + r * gr) * 0.5;
            } else {
                frame[0] += l * gl;
                frame[1] += r * gr;
            }
            s.pos += step;
            if s.pos >= frames as f64 {
                if s.iterations == 1 {
                    s.playing = false;
                    s.pos = 0.0;
                } else {
                    if s.iterations > 1 {
                        s.iterations -= 1;
                    }
                    s.pos -= frames as f64;
                }
            }
        }
    }
    for v in out.iter_mut() {
        *v = v.clamp(-1.0, 1.0);
    }
}

/// Original: `PushAulibDecoder::doDecoding` (with `Next`): the queued samples, then silence
/// when the queue runs dry (the stream keeps playing). Resampled linearly like the other streams.
// @port utils/push_aulib_decoder.cpp|devilution::PushAulibDecoder::doDecoding(float buf[], int len, bool &callAgain) sha=70e1d8c802cc
// @port utils/push_aulib_decoder.cpp|devilution::PushAulibDecoder::Next() sha=db57695e025b
fn mix_push(s: &mut StreamState, out: &mut [f32], och: usize, out_rate: f64) {
    let ich = s.pcm.channels.max(1) as usize;
    let step = s.pcm.rate as f64 / out_rate;
    let gain = if s.muted { 0.0 } else { s.volume };
    let mut pos = s.pos;
    let q = s.push.as_mut().unwrap();
    for frame in out.chunks_exact_mut(och) {
        if q.len() < ich {
            break;
        }
        let t = pos as f32;
        let next = if q.len() >= 2 * ich { ich } else { 0 };
        let sample = |c: usize| {
            let c = c.min(ich - 1);
            q[c] + (q[next + c] - q[c]) * t
        };
        let (l, r) = if ich == 1 { (sample(0), sample(0)) } else { (sample(0), sample(1)) };
        if och == 1 {
            frame[0] += (l + r) * 0.5 * gain;
        } else {
            frame[0] += l * gain;
            frame[1] += r * gain;
        }
        pos += step;
        while pos >= 1.0 && q.len() >= ich {
            q.drain(..ich);
            pos -= 1.0;
        }
    }
    s.pos = if q.len() < ich { 0.0 } else { pos };
}

/// `Aulib::init`: opens the output device. `device` is a device name ("" = system default).
pub fn init(sample_rate: i32, channels: i32, buffer_size: i32, device: &str) -> Result<(), String> {
    if std::env::var_os("DIABLO_NO_AUDIO").is_some_and(|v| !v.is_empty()) {
        return Err("disabled by DIABLO_NO_AUDIO".into());
    }
    let host = cpal::default_host();
    let dev = if device.is_empty() {
        host.default_output_device()
    } else {
        host.output_devices().map_err(|e| e.to_string())?.find(|d| d.description().is_ok_and(|n| n.name() == device)).or_else(|| host.default_output_device())
    }
    .ok_or("no audio output device")?;
    let supported = dev.default_output_config().map_err(|e| e.to_string())?;
    // Use the requested rate/channels when the device supports them, else its defaults.
    let wanted_rate = sample_rate.max(1) as u32;
    let wanted_channels = channels.clamp(1, 2) as u16;
    let mut config: cpal::StreamConfig = supported.config();
    if let Ok(ranges) = dev.supported_output_configs() {
        for r in ranges {
            if r.channels() == wanted_channels
                && r.sample_format() == cpal::SampleFormat::F32
                && r.min_sample_rate() <= wanted_rate
                && wanted_rate <= r.max_sample_rate()
            {
                config = r.with_sample_rate(wanted_rate).config();
                break;
            }
        }
    }
    if buffer_size > 0 {
        config.buffer_size = cpal::BufferSize::Fixed(buffer_size as u32);
    }
    let mixer = MIXER.get_or_init(|| Arc::new(Mutex::new(MixerState { streams: Vec::new(), out_rate: 0, out_channels: 0 }))).clone();
    {
        let mut m = mixer.lock().unwrap();
        m.out_rate = config.sample_rate;
        m.out_channels = config.channels;
    }
    let m2 = mixer.clone();
    let build = |config: &cpal::StreamConfig| {
        let m3 = m2.clone();
        dev.build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| mix(&mut m3.lock().unwrap(), data),
            |e| crate::platform::log::error!("audio stream error: {}", e),
            None,
        )
    };
    let stream = match build(&config) {
        Ok(s) => s,
        Err(_) if config.buffer_size != cpal::BufferSize::Default => {
            config.buffer_size = cpal::BufferSize::Default;
            build(&config).map_err(|e| e.to_string())?
        }
        Err(e) => return Err(e.to_string()),
    };
    stream.play().map_err(|e| e.to_string())?;
    OUTPUT.with(|o| *o.borrow_mut() = Some(stream));
    crate::platform::log::info!("audio: {} Hz, {} channels", config.sample_rate, config.channels);
    Ok(())
}

/// `Aulib::quit`
pub fn quit() {
    OUTPUT.with(|o| *o.borrow_mut() = None);
    if let Some(m) = MIXER.get() {
        m.lock().unwrap().streams.clear();
    }
}

/// Output device names (`SDL_GetAudioDeviceName`).
pub fn output_device_names() -> Vec<String> {
    let host = cpal::default_host();
    host.output_devices().map(|ds| ds.filter_map(|d| d.description().ok().map(|n| n.name().to_string())).collect()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wav(bits: u16, channels: u16, rate: u32, data: &[u8]) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(b"RIFF");
        v.extend_from_slice(&(36 + data.len() as u32).to_le_bytes());
        v.extend_from_slice(b"WAVEfmt ");
        v.extend_from_slice(&16u32.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&channels.to_le_bytes());
        v.extend_from_slice(&rate.to_le_bytes());
        v.extend_from_slice(&(rate * channels as u32 * bits as u32 / 8).to_le_bytes());
        v.extend_from_slice(&(channels * bits / 8).to_le_bytes());
        v.extend_from_slice(&bits.to_le_bytes());
        v.extend_from_slice(b"data");
        v.extend_from_slice(&(data.len() as u32).to_le_bytes());
        v.extend_from_slice(data);
        v
    }

    #[test]
    fn decodes_pcm8_and_pcm16() {
        let p = decode_wav(&wav(8, 1, 22050, &[128, 255, 0])).unwrap();
        assert_eq!(p.samples, vec![0.0, 127.0 / 128.0, -1.0]);
        let p = decode_wav(&wav(16, 2, 44100, &[0, 0x40, 0, 0xc0])).unwrap();
        assert_eq!((p.channels, p.rate, p.frames()), (2, 44100, 1));
        assert_eq!(p.samples, vec![0.5, -0.5]);
    }

    #[test]
    fn mixes_with_pan_volume_and_stops_at_end() {
        let pcm = Arc::new(Pcm { samples: vec![1.0, 1.0], channels: 1, rate: 100 });
        let s = Arc::new(Mutex::new(StreamState { pcm, pos: 0.0, playing: true, iterations: 1, volume: 0.5, stereo_position: 0.5, muted: false, push: None }));
        let mut m = MixerState { streams: vec![Arc::downgrade(&s)], out_rate: 100, out_channels: 2 };
        let mut out = [0f32; 6];
        mix(&mut m, &mut out);
        assert_eq!(out, [0.25, 0.5, 0.25, 0.5, 0.0, 0.0]);
        assert!(!s.lock().unwrap().playing);
    }

    #[test]
    fn push_stream_plays_queued_samples_then_silence() {
        let pcm = Arc::new(Pcm { samples: Vec::new(), channels: 1, rate: 100 });
        let s = Arc::new(Mutex::new(StreamState { pcm, pos: 0.0, playing: true, iterations: 0, volume: 1.0, stereo_position: 0.0, muted: false, push: Some(VecDeque::new()) }));
        Stream(s.clone()).push_samples_i16(&[16384, -16384]);
        let mut m = MixerState { streams: vec![Arc::downgrade(&s)], out_rate: 100, out_channels: 2 };
        let mut out = [0f32; 6];
        mix(&mut m, &mut out);
        assert_eq!(out, [0.5, 0.5, -0.5, -0.5, 0.0, 0.0]);
        assert!(s.lock().unwrap().playing);
    }
}
