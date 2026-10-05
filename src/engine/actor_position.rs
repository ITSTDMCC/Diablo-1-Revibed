//! `Source/engine/actor_position.cpp`: tile positions of a walking actor and the sub-tile walk
//! offsets derived from its animation progress.

use crate::engine::animationinfo::AnimationInfo;
use crate::engine::geometry::{Direction, Displacement, Point};

/// `ActorPosition`
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ActorPosition {
    pub tile: Point,
    pub future: Point,
    pub last: Point,
    pub old: Point,
    pub temp: Point,
}

/// `VelocityToUse`
#[derive(Clone, Copy, PartialEq, Eq)]
enum VelocityToUse {
    None,
    Full,
    NegativeFull,
    Half,
    NegativeHalf,
    Quarter,
    NegativeQuarter,
}

/// `RoundedWalkVelocity`
struct RoundedWalkVelocity {
    quarter: i16,
    half: i16,
    full: i16,
}

impl RoundedWalkVelocity {
    /// Original: `RoundedWalkVelocity::getVelocity` (engine/actor_position.cpp).
    // @port engine/actor_position.cpp|devilution::RoundedWalkVelocity::getVelocity(VelocityToUse velocityToUse) sha=71920827bb59
    const fn get_velocity(&self, v: VelocityToUse) -> i16 {
        match v {
            VelocityToUse::Quarter => self.quarter,
            VelocityToUse::NegativeQuarter => -self.quarter,
            VelocityToUse::Half => self.half,
            VelocityToUse::NegativeHalf => -self.half,
            VelocityToUse::Full => self.full,
            VelocityToUse::NegativeFull => -self.full,
            VelocityToUse::None => 0,
        }
    }
}

const fn rwv(quarter: i16, half: i16, full: i16) -> RoundedWalkVelocity {
    RoundedWalkVelocity { quarter, half, full }
}

/// `WalkVelocityForFrames`
const WALK_VELOCITY_FOR_FRAMES: [RoundedWalkVelocity; 24] = [
    rwv(256, 512, 1024),
    rwv(128, 256, 512),
    rwv(85, 170, 341),
    rwv(64, 128, 256),
    rwv(51, 102, 204),
    rwv(42, 85, 170),
    rwv(36, 73, 146),
    rwv(32, 64, 128),
    rwv(28, 56, 113),
    rwv(26, 51, 102),
    rwv(23, 46, 93),
    rwv(21, 42, 85),
    rwv(19, 39, 78),
    rwv(18, 36, 73),
    rwv(17, 34, 68),
    rwv(16, 32, 64),
    rwv(15, 30, 60),
    rwv(14, 28, 57),
    rwv(13, 26, 54),
    rwv(12, 25, 51),
    rwv(12, 24, 48),
    rwv(11, 23, 46),
    rwv(11, 22, 44),
    rwv(10, 21, 42),
];

/// `WalkParameter`
struct WalkParameter {
    starting_offset: (i16, i16),
    velocity_x: VelocityToUse,
    velocity_y: VelocityToUse,
}

impl WalkParameter {
    /// Original: `WalkParameter::getVelocity` (engine/actor_position.cpp).
    // @port engine/actor_position.cpp|devilution::WalkParameter::getVelocity(int8_t numberOfFrames) sha=89e8ee155426
    fn get_velocity(&self, number_of_frames: i8) -> (i16, i16) {
        let w = &WALK_VELOCITY_FOR_FRAMES[(number_of_frames - 1) as usize];
        (w.get_velocity(self.velocity_x), w.get_velocity(self.velocity_y))
    }
}

const WALK_PARAMETERS: [WalkParameter; 8] = {
    use VelocityToUse::*;
    [
        WalkParameter { starting_offset: (0, -512), velocity_x: None, velocity_y: Half },
        WalkParameter { starting_offset: (512, -256), velocity_x: NegativeHalf, velocity_y: Quarter },
        WalkParameter { starting_offset: (512, -256), velocity_x: NegativeFull, velocity_y: None },
        WalkParameter { starting_offset: (0, 0), velocity_x: NegativeHalf, velocity_y: NegativeQuarter },
        WalkParameter { starting_offset: (0, 0), velocity_x: None, velocity_y: NegativeHalf },
        WalkParameter { starting_offset: (0, 0), velocity_x: Half, velocity_y: NegativeQuarter },
        WalkParameter { starting_offset: (-512, -256), velocity_x: Full, velocity_y: None },
        WalkParameter { starting_offset: (-512, -256), velocity_x: Half, velocity_y: Quarter },
    ]
};

impl ActorPosition {
    /// Original: `ActorPosition::CalculateWalkingOffset` (engine/actor_position.cpp): an int8 displacement.
    // @port engine/actor_position.cpp|devilution::ActorPosition::CalculateWalkingOffset(Direction dir, const AnimationInfo &animInfo) sha=f52ba3df4c4a
    pub fn calculate_walking_offset(&self, dir: Direction, anim_info: &AnimationInfo, progress_to_next_game_tick: u8) -> Displacement {
        let (x, y) = self.calculate_walking_offset_shifted4_raw(dir, anim_info, progress_to_next_game_tick);
        Displacement::new((x >> 4) as i8 as i32, (y >> 4) as i8 as i32)
    }

    fn calculate_walking_offset_shifted4_raw(&self, dir: Direction, anim_info: &AnimationInfo, progress: u8) -> (i16, i16) {
        let velocity_progress = (anim_info.get_animation_progress(progress) as i32 * anim_info.numberOfFrames as i32 / AnimationInfo::BASE_VALUE_FRACTION) as i16;
        let wp = &WALK_PARAMETERS[dir as usize];
        let (vx, vy) = wp.get_velocity(anim_info.numberOfFrames);
        let x = wp.starting_offset.0.wrapping_add((vx as i32 * velocity_progress as i32) as i16);
        let y = wp.starting_offset.1.wrapping_add((vy as i32 * velocity_progress as i32) as i16);
        (x, y)
    }

    /// Original: `ActorPosition::CalculateWalkingOffsetShifted4` (engine/actor_position.cpp).
    // @port engine/actor_position.cpp|devilution::ActorPosition::CalculateWalkingOffsetShifted4(Direction dir, const AnimationInfo &animInfo) sha=2af811ba0f74
    pub fn calculate_walking_offset_shifted4(&self, dir: Direction, anim_info: &AnimationInfo, progress: u8) -> Displacement {
        let (x, y) = self.calculate_walking_offset_shifted4_raw(dir, anim_info, progress);
        Displacement::new(x as i32, y as i32)
    }

    /// Original: `ActorPosition::CalculateWalkingOffsetShifted8` (engine/actor_position.cpp).
    // @port engine/actor_position.cpp|devilution::ActorPosition::CalculateWalkingOffsetShifted8(Direction dir, const AnimationInfo &animInfo) sha=b39f3ee770f1
    pub fn calculate_walking_offset_shifted8(&self, dir: Direction, anim_info: &AnimationInfo, progress: u8) -> Displacement {
        let (x, y) = self.calculate_walking_offset_shifted4_raw(dir, anim_info, progress);
        Displacement::new(((x as i32) << 4) as i16 as i32, ((y as i32) << 4) as i16 as i32)
    }

    /// Original: `ActorPosition::GetWalkingVelocityShifted4` (engine/actor_position.cpp).
    // @port engine/actor_position.cpp|devilution::ActorPosition::GetWalkingVelocityShifted4(Direction dir, const AnimationInfo &animInfo) sha=289136a09078
    pub fn get_walking_velocity_shifted4(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement {
        let (x, y) = WALK_PARAMETERS[dir as usize].get_velocity(anim_info.numberOfFrames);
        Displacement::new(x as i32, y as i32)
    }

    /// Original: `ActorPosition::GetWalkingVelocityShifted8` (engine/actor_position.cpp).
    // @port engine/actor_position.cpp|devilution::ActorPosition::GetWalkingVelocityShifted8(Direction dir, const AnimationInfo &animInfo) sha=b5a1683b82d1
    pub fn get_walking_velocity_shifted8(&self, dir: Direction, anim_info: &AnimationInfo) -> Displacement {
        let (x, y) = WALK_PARAMETERS[dir as usize].get_velocity(anim_info.numberOfFrames);
        Displacement::new(((x as i32) << 4) as i16 as i32, ((y as i32) << 4) as i16 as i32)
    }
}
