//! `Source/nthread.cpp`: game tick timing and the network turn thread (pending).

/// Globals of nthread.cpp.
#[derive(Default)]
pub struct NthreadState {
    /// `ProgressToNextGameTick`: fraction (of `AnimationInfo::baseValueFraction`) of the next tick.
    pub ProgressToNextGameTick: u8,
}
