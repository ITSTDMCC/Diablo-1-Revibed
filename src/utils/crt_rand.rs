//! The C runtime's `rand()`/`srand()`. The shipped exe imports them from MSVCRT.DLL (see the
//! `rand`/`srand` thunks in port_bin_functions), whose generator is the LCG below with a
//! 15-bit result. The game uses it only where the outcome is cosmetic (hero name suggestions,
//! floating number jitter after `srand(time(nullptr))`), but the sequence is kept exact.

/// MSVCRT's per-thread `holdrand` (the original only calls `rand` from one thread).
#[derive(Clone, Copy, Debug)]
pub struct CrtRand {
    holdrand: u32,
}

impl Default for CrtRand {
    fn default() -> Self {
        CrtRand { holdrand: 1 }
    }
}

impl CrtRand {
    /// `srand`
    pub fn srand(&mut self, seed: u32) {
        self.holdrand = seed;
    }

    /// `rand`: 0..=0x7FFF
    pub fn rand(&mut self) -> i32 {
        self.holdrand = self.holdrand.wrapping_mul(214013).wrapping_add(2531011);
        ((self.holdrand >> 16) & 0x7FFF) as i32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_msvcrt_sequence() {
        // First values of MSVCRT rand() with the default seed of 1.
        let mut r = CrtRand::default();
        let v: Vec<i32> = (0..5).map(|_| r.rand()).collect();
        assert_eq!(v, vec![41, 18467, 6334, 26500, 19169]);
    }
}
