//! `Source/storm/storm_svid.cpp`: video (SMK) playback. Decoding is done by
//! `storm::smacker` (the role of libsmackerdec); frames are presented at the video's own size,
//! as the original does by setting the renderer's logical size to the video size.

use crate::ctx::Ctx;
use crate::platform::audio::Stream;
use crate::platform::log;
use crate::storm::smacker::Smacker;

/// Globals of storm_svid.cpp.
#[derive(Default)]
pub struct SvidState {
    /// `SVidAudioStream` (with its `PushAulibDecoder`)
    audio_stream: Option<Stream>,
    /// `SVidAudioDepth`
    audio_depth: u32,
    /// `SVidWidth`, `SVidHeight`
    width: u32,
    height: u32,
    /// `SVidFrameEnd` (microseconds)
    frame_end: f64,
    /// `SVidFrameLength` (microseconds)
    frame_length: f64,
    /// `SVidLoop`
    looping: bool,
    /// `SVidHandle`
    handle: Option<Smacker>,
    /// `SVidFrameBuffer`
    frame_buffer: Vec<u8>,
    /// `SVidPalette`
    palette: Vec<[u8; 3]>,
    /// The RGB frame handed to the renderer.
    output: Vec<u8>,
}

fn now_us(ctx: &Ctx) -> f64 {
    ctx.platform.ticks() as f64 * 1000.0
}

/// Original: `HasAudio` (storm/storm_svid.cpp).
// @port storm/storm_svid.cpp|devilution::HasAudio() sha=ce5eee738f0f
fn has_audio(ctx: &Ctx) -> bool {
    ctx.svid.audio_stream.as_ref().is_some_and(|s| s.is_playing())
}

/// Original: `SVidLoadNextFrame` (storm/storm_svid.cpp).
// @port storm/storm_svid.cpp|devilution::SVidLoadNextFrame() sha=15d884504102
fn svid_load_next_frame(ctx: &mut Ctx) -> bool {
    let s = &mut ctx.svid;
    let Some(h) = s.handle.as_mut() else { return false };
    if h.current_frame_num() >= h.num_frames() {
        if !s.looping {
            return false;
        }

        h.rewind();
    }

    s.frame_end += s.frame_length;

    if let Err(e) = h.get_next_frame() {
        log::error!("{}", e);
        return false;
    }
    h.get_frame(&mut s.frame_buffer);

    true
}

/// Original: `UpdatePalette` (storm/storm_svid.cpp).
// @port storm/storm_svid.cpp|devilution::UpdatePalette() sha=9b7a5d2adf56
fn update_palette(ctx: &mut Ctx) {
    let s = &mut ctx.svid;
    let Some(h) = s.handle.as_ref() else { return };
    let palette_data = h.palette();
    s.palette = palette_data.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect();
}

/// Original: `BlitFrame` (storm/storm_svid.cpp), renderer path: the 8-bit frame converted to
/// RGB at the video size, then `RenderPresent`.
// @port storm/storm_svid.cpp|devilution::BlitFrame() sha=7b35d0b2623d
fn blit_frame(ctx: &mut Ctx) -> bool {
    if ctx.diablo.headless_mode {
        return true;
    }
    let s = &mut ctx.svid;
    s.output.clear();
    for &p in &s.frame_buffer {
        s.output.extend_from_slice(&s.palette[p as usize]);
    }
    let (w, h) = (s.width as usize, s.height as usize);
    let out = std::mem::take(&mut s.output);
    ctx.platform.present(w, h, &out);
    ctx.svid.output = out;
    true
}

/// Original: `devilution::SVidPlayBegin` (storm/storm_svid.cpp).
// @port storm/storm_svid.cpp|devilution::SVidPlayBegin(const char *filename, int flags) sha=55db1545d0a1
pub fn svid_play_begin(ctx: &mut Ctx, filename: &str, flags: i32) -> bool {
    if (flags & 0x10000) != 0 || (flags & 0x2000_0000) != 0 {
        return false;
    }

    ctx.svid.looping = false;
    if (flags & 0x40000) != 0 {
        ctx.svid.looping = true;
    }
    // 0x8 // Non-interlaced
    // 0x200, 0x800 // Upscale video
    // 0x80000 // Center horizontally
    // 0x100000 // Disable video
    // 0x800000 // Edge detection
    // 0x200800 // Clear FB

    let handle = crate::engine::assets::open_asset(ctx, filename);
    if !handle.ok() {
        return false;
    }
    let mut smk = match Smacker::open(handle.into_bytes()) {
        Ok(h) => h,
        Err(e) => {
            log::error!("{}: {}", filename, e);
            return false;
        }
    };

    let enable_audio = (flags & 0x0100_0000) == 0;

    let audio_info = smk.audio_track_details(0);
    log::verbose!("SVid audio depth={} channels={} rate={}", audio_info.bits_per_sample, audio_info.n_channels, audio_info.sample_rate);

    // Aulib::Stream::open fails without an initialised audio device; the stream is then dropped.
    if enable_audio && audio_info.bits_per_sample != 0 && ctx.sound.gb_snd_inited {
        crate::effects::sound_stop(ctx); // Stop in-progress music and sound effects

        ctx.svid.audio_depth = audio_info.bits_per_sample;
        let stream = Stream::new_push(audio_info.n_channels as u16, audio_info.sample_rate);
        let volume = (ctx.options.audio.sound_volume.get() - crate::engine::sound::VOLUME_MIN) as f32 / -crate::engine::sound::VOLUME_MIN as f32;
        stream.set_volume(volume);
        if !crate::diablo::diablo_is_focused(ctx) {
            stream.mute();
        }
        stream.play(0);
        ctx.svid.audio_stream = Some(stream);
    }

    ctx.svid.frame_length = 1_000_000.0 / smk.frame_rate() as f64;
    let (w, h) = smk.frame_size();
    ctx.svid.width = w;
    ctx.svid.height = h;

    // The buffer for the frame.
    ctx.svid.frame_buffer = vec![0; (w * h) as usize];

    // Decode first frame.
    if let Err(e) = smk.get_next_frame() {
        log::error!("{}: {}", filename, e);
    }
    smk.get_frame(&mut ctx.svid.frame_buffer);
    ctx.svid.handle = Some(smk);

    update_palette(ctx);

    ctx.svid.frame_end = now_us(ctx) + ctx.svid.frame_length;

    true
}

/// Original: `devilution::SVidPlayContinue` (storm/storm_svid.cpp).
// @port storm/storm_svid.cpp|devilution::SVidPlayContinue() sha=e4ca845f9416
pub fn svid_play_continue(ctx: &mut Ctx) -> bool {
    if ctx.svid.handle.as_ref().is_some_and(|h| h.did_palette_change()) {
        update_palette(ctx);
    }

    if now_us(ctx) >= ctx.svid.frame_end {
        return svid_load_next_frame(ctx); // Skip video and audio if the system is to slow
    }

    if has_audio(ctx) {
        let s = &ctx.svid;
        let data = s.handle.as_ref().map(|h| h.audio_data(0)).unwrap_or(&[]);
        let stream = s.audio_stream.as_ref().unwrap();
        if s.audio_depth == 16 {
            let samples: Vec<i16> = data.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();
            stream.push_samples_i16(&samples);
        } else {
            stream.push_samples_u8(data);
        }
    }

    if now_us(ctx) >= ctx.svid.frame_end {
        return svid_load_next_frame(ctx); // Skip video if the system is to slow
    }

    if !blit_frame(ctx) {
        return false;
    }

    let now = now_us(ctx);
    if now < ctx.svid.frame_end {
        let ms = ((ctx.svid.frame_end - now) / 1000.0) as u32;
        ctx.platform.delay(ms); // wait with next frame if the system is too fast
    }

    svid_load_next_frame(ctx)
}

/// Original: `devilution::SVidPlayEnd` (storm/storm_svid.cpp).
// @port storm/storm_svid.cpp|devilution::SVidPlayEnd() sha=8a650569ae29
pub fn svid_play_end(ctx: &mut Ctx) {
    if has_audio(ctx) {
        ctx.svid.audio_stream = None;
    }

    ctx.svid.handle = None;

    ctx.svid.palette.clear();
    ctx.svid.frame_buffer = Vec::new();
    // The renderer's logical size returns to the screen size with the next RenderPresent.
}

/// Original: `devilution::SVidMute` (storm/storm_svid.cpp).
// @port storm/storm_svid.cpp|devilution::SVidMute() sha=bf2d9a1d81b7
pub fn svid_mute(ctx: &mut Ctx) {
    if let Some(s) = &ctx.svid.audio_stream {
        s.mute();
    }
}

/// Original: `devilution::SVidUnmute` (storm/storm_svid.cpp).
// @port storm/storm_svid.cpp|devilution::SVidUnmute() sha=cc4666659f51
pub fn svid_unmute(ctx: &mut Ctx) {
    if let Some(s) = &ctx.svid.audio_stream {
        s.unmute();
    }
}
