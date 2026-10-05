//! `Source/engine/animationinfo.cpp`: animation state with the "animation distribution logic"
//! that spreads skipped frames over the shown game ticks.
//!
//! `ProgressToNextGameTick` (nthread.cpp) is passed in by the caller.

#![allow(non_snake_case)]

use crate::engine::clx_sprite::{ClxSprite, ClxSpriteList};
use crate::enums::AnimationDistributionFlags;
use crate::platform::log;

/// `AnimationInfo`
#[derive(Clone, Debug, Default)]
pub struct AnimationInfo {
    pub sprites: Option<ClxSpriteList>,
    pub ticksPerFrame: i8,
    pub tickCounterOfCurrentFrame: i8,
    pub numberOfFrames: i8,
    pub currentFrame: i8,
    pub isPetrified: bool,
    relevantFramesForDistributing_: i8,
    skippedFramesFromPreviousAnimation_: i8,
    tickModifier_: u16,
    ticksSinceSequenceStarted_: i16,
}

impl AnimationInfo {
    pub const BASE_VALUE_FRACTION: i32 = 128;

    /// `currentSprite`
    pub fn current_sprite(&self, progress_to_next_game_tick: u8) -> ClxSprite {
        self.sprites.as_ref().expect("sprites").get(self.get_frame_to_use_for_rendering(progress_to_next_game_tick) as usize)
    }

    /// `isLastFrame`
    pub fn is_last_frame(&self) -> bool {
        self.currentFrame >= self.numberOfFrames - 1
    }

    /// Original: `AnimationInfo::getFrameToUseForRendering` (engine/animationinfo.cpp).
    // @port engine/animationinfo.cpp|devilution::AnimationInfo::getFrameToUseForRendering() sha=e2c0ffb538c1
    pub fn get_frame_to_use_for_rendering(&self, progress_to_next_game_tick: u8) -> i8 {
        if self.relevantFramesForDistributing_ <= 0 {
            return self.currentFrame.max(0);
        }
        if self.currentFrame >= self.relevantFramesForDistributing_ {
            return self.currentFrame;
        }
        let mut ticks_since_sequence_started = self.ticksSinceSequenceStarted_;
        if self.ticksSinceSequenceStarted_ < 0 {
            ticks_since_sequence_started = 0;
            log::verbose!("getFrameToUseForRendering: Invalid ticksSinceSequenceStarted_ {}", self.ticksSinceSequenceStarted_);
        }
        let total = self.get_progress_to_next_game_tick(progress_to_next_game_tick) as i32 + ticks_since_sequence_started as i32;
        let mut absolute = (total * self.tickModifier_ as i32 / Self::BASE_VALUE_FRACTION / Self::BASE_VALUE_FRACTION) as i8;
        if self.skippedFramesFromPreviousAnimation_ > 0 {
            absolute = absolute.wrapping_sub(self.skippedFramesFromPreviousAnimation_);
            if absolute < 0 {
                absolute = self.numberOfFrames.wrapping_add(absolute);
            }
        } else if absolute >= self.relevantFramesForDistributing_ {
            if absolute as i32 >= self.relevantFramesForDistributing_ as i32 + 1 {
                log::verbose!(
                    "getFrameToUseForRendering: Calculated an invalid Animation Frame (Calculated {} MaxFrame {})",
                    absolute,
                    self.relevantFramesForDistributing_
                );
            }
            return self.relevantFramesForDistributing_ - 1;
        }
        if absolute < 0 {
            log::verbose!("getFrameToUseForRendering: Calculated an invalid Animation Frame (Calculated {})", absolute);
            return 0;
        }
        absolute
    }

    /// Original: `AnimationInfo::getAnimationProgress` (engine/animationinfo.cpp).
    // @port engine/animationinfo.cpp|devilution::AnimationInfo::getAnimationProgress() sha=f826b92db04e
    pub fn get_animation_progress(&self, progress_to_next_game_tick: u8) -> u8 {
        let mut ticks_since_sequence_started = self.ticksSinceSequenceStarted_.max(0) as i32;
        let mut tick_modifier = self.tickModifier_ as i32;
        if self.relevantFramesForDistributing_ <= 0 {
            ticks_since_sequence_started =
                ((self.currentFrame as i32 * self.ticksPerFrame as i32) + self.tickCounterOfCurrentFrame as i32) * Self::BASE_VALUE_FRACTION;
            tick_modifier = Self::BASE_VALUE_FRACTION / self.ticksPerFrame as i32;
        }
        let total = self.get_progress_to_next_game_tick(progress_to_next_game_tick) as i32 + ticks_since_sequence_started;
        let progress_in_animation_frames = total * tick_modifier;
        let animation_fraction = progress_in_animation_frames / self.numberOfFrames as i32 / Self::BASE_VALUE_FRACTION;
        debug_assert!(animation_fraction <= Self::BASE_VALUE_FRACTION);
        animation_fraction as u8
    }

    /// Original: `AnimationInfo::setNewAnimation` (engine/animationinfo.cpp). Defaults in the
    /// original: flags None, numSkippedFrames 0, distributeFramesBeforeFrame 0,
    /// previewShownGameTickFragments 0.
    // @port engine/animationinfo.cpp|devilution::AnimationInfo::setNewAnimation(OptionalClxSpriteList celSprite, int8_t numberOfFrames, int8_t ticksPerFrame, AnimationDistributionFlags flags , int8_t numSkippedFrames , int8_t distributeFramesBeforeFrame , uint8_t previewShownGameTickFragments) sha=625ca63001d7
    #[allow(clippy::too_many_arguments)]
    pub fn set_new_animation(
        &mut self,
        cel_sprite: Option<ClxSpriteList>,
        number_of_frames: i8,
        mut ticks_per_frame: i8,
        flags: AnimationDistributionFlags,
        num_skipped_frames: i8,
        distribute_frames_before_frame: i8,
        preview_shown_game_tick_fragments: u8,
    ) {
        if flags.has_all_of(AnimationDistributionFlags::RepeatedAction)
            && distribute_frames_before_frame != 0
            && self.numberOfFrames == number_of_frames
            && self.currentFrame as i32 + 1 >= distribute_frames_before_frame as i32
            && self.currentFrame != self.numberOfFrames - 1
        {
            self.skippedFramesFromPreviousAnimation_ = self.numberOfFrames - self.currentFrame - 1;
        } else {
            self.skippedFramesFromPreviousAnimation_ = 0;
        }
        if ticks_per_frame <= 0 {
            log::verbose!("setNewAnimation: Invalid ticksPerFrame {}", ticks_per_frame);
            ticks_per_frame = 1;
        }
        self.sprites = cel_sprite;
        self.numberOfFrames = number_of_frames;
        self.currentFrame = num_skipped_frames;
        self.tickCounterOfCurrentFrame = 0;
        self.ticksPerFrame = ticks_per_frame;
        self.ticksSinceSequenceStarted_ = 0;
        self.relevantFramesForDistributing_ = 0;
        self.tickModifier_ = 0;
        self.isPetrified = false;

        if num_skipped_frames != 0 || flags != AnimationDistributionFlags::None {
            let mut relevant_frames = number_of_frames;
            if distribute_frames_before_frame != 0 {
                relevant_frames = distribute_frames_before_frame - 1;
            }
            let tpf = ticks_per_frame as i32;
            let mut ticks_for_distribution = relevant_frames as i32 * tpf;
            let mut ticks_with_skipping = ticks_for_distribution - num_skipped_frames as i32 * tpf;
            if flags.has_all_of(AnimationDistributionFlags::ProcessAnimationPending) {
                ticks_with_skipping -= 1;
                self.ticksSinceSequenceStarted_ = -(Self::BASE_VALUE_FRACTION as i16);
            }
            if flags.has_all_of(AnimationDistributionFlags::SkipsDelayOfLastFrame) {
                ticks_with_skipping -= tpf - 1;
            }
            ticks_for_distribution += self.skippedFramesFromPreviousAnimation_ as i32 * tpf;
            ticks_for_distribution *= Self::BASE_VALUE_FRACTION;
            ticks_with_skipping *= Self::BASE_VALUE_FRACTION;
            self.ticksSinceSequenceStarted_ += preview_shown_game_tick_fragments as i16;
            ticks_with_skipping += preview_shown_game_tick_fragments as i32;
            let mut tick_modifier = 0;
            if ticks_with_skipping != 0 {
                tick_modifier = Self::BASE_VALUE_FRACTION * ticks_for_distribution / ticks_with_skipping;
            }
            tick_modifier /= tpf;
            self.relevantFramesForDistributing_ = relevant_frames;
            self.tickModifier_ = tick_modifier as u16;
        }
    }

    /// `setNewAnimation` with the default trailing arguments.
    pub fn set_new_animation_simple(&mut self, cel_sprite: Option<ClxSpriteList>, number_of_frames: i8, ticks_per_frame: i8) {
        self.set_new_animation(cel_sprite, number_of_frames, ticks_per_frame, AnimationDistributionFlags::None, 0, 0, 0);
    }

    /// Original: `AnimationInfo::changeAnimationData` (engine/animationinfo.cpp).
    // @port engine/animationinfo.cpp|devilution::AnimationInfo::changeAnimationData(OptionalClxSpriteList celSprite, int8_t numberOfFrames, int8_t ticksPerFrame) sha=149c86fc9621
    pub fn change_animation_data(&mut self, cel_sprite: Option<ClxSpriteList>, number_of_frames: i8, ticks_per_frame: i8) {
        if number_of_frames != self.numberOfFrames || ticks_per_frame != self.ticksPerFrame {
            if number_of_frames >= 1 {
                self.currentFrame = self.currentFrame.clamp(0, number_of_frames - 1);
            } else {
                self.currentFrame = -1;
            }
            self.numberOfFrames = number_of_frames;
            self.ticksPerFrame = ticks_per_frame;
            self.ticksSinceSequenceStarted_ = 0;
            self.relevantFramesForDistributing_ = 0;
            self.tickModifier_ = 0;
        }
        self.sprites = cel_sprite;
    }

    /// Original: `AnimationInfo::processAnimation` (engine/animationinfo.cpp). `reverseAnimation`
    /// defaults to false.
    // @port engine/animationinfo.cpp|devilution::AnimationInfo::processAnimation(bool reverseAnimation) sha=98d5acab0c0f
    pub fn process_animation(&mut self, reverse_animation: bool) {
        self.tickCounterOfCurrentFrame = self.tickCounterOfCurrentFrame.wrapping_add(1);
        self.ticksSinceSequenceStarted_ = self.ticksSinceSequenceStarted_.wrapping_add(Self::BASE_VALUE_FRACTION as i16);
        if self.tickCounterOfCurrentFrame >= self.ticksPerFrame {
            self.tickCounterOfCurrentFrame = 0;
            if reverse_animation {
                self.currentFrame -= 1;
                if self.currentFrame == -1 {
                    self.currentFrame = self.numberOfFrames - 1;
                    self.ticksSinceSequenceStarted_ = 0;
                }
            } else {
                self.currentFrame = self.currentFrame.wrapping_add(1);
                if self.currentFrame >= self.numberOfFrames {
                    self.currentFrame = 0;
                    self.ticksSinceSequenceStarted_ = 0;
                }
            }
        }
    }

    /// Original: `AnimationInfo::getProgressToNextGameTick` (engine/animationinfo.cpp).
    // @port engine/animationinfo.cpp|devilution::AnimationInfo::getProgressToNextGameTick() sha=095a1c356a58
    fn get_progress_to_next_game_tick(&self, progress_to_next_game_tick: u8) -> u8 {
        if self.isPetrified {
            return 0;
        }
        progress_to_next_game_tick
    }

    /// Fields saved by loadsave.cpp that are otherwise private.
    pub fn distribution_state(&self) -> (i8, i8, u16, i16) {
        (self.relevantFramesForDistributing_, self.skippedFramesFromPreviousAnimation_, self.tickModifier_, self.ticksSinceSequenceStarted_)
    }
}
