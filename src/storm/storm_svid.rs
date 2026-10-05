//! `Source/storm/storm_svid.cpp`: video (SMK) playback.
//!
//! Video playback is not ported yet, so `SVidAudioStream` never exists and mute/unmute have
//! nothing to act on (as in the original whenever no video is playing).

use crate::ctx::Ctx;

/// `SVidMute`: no video audio stream exists until SVid playback is ported.
pub fn svid_mute(_ctx: &mut Ctx) {}

/// `SVidUnmute`: see `svid_mute`.
pub fn svid_unmute(_ctx: &mut Ctx) {}
