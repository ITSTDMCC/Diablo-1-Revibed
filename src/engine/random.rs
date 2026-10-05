//! The vanilla random number generator (`Source/engine/random.cpp`, `random.hpp`).
//!
//! The original keeps `sglGameSeed` and a `std::linear_congruential_engine<uint32_t, 0x015A4E35, 1, 0>`
//! (Borland's LCG, modulus 2^32) as globals. Their values are always equal (the engine is seeded with
//! the seed and every output is the new state), so one `u32` holds both. The port keeps it in a
//! value owned by the game state instead of a global.

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiabloRng {
    /// `sglGameSeed` (= the engine state)
    seed: u32,
}

impl DiabloRng {
    /// Original: `devilution::SetRndSeed` (engine/random.cpp:17).
    // @port engine/random.cpp|devilution::SetRndSeed(uint32_t seed) sha=48feb7817caf
    pub fn set_rnd_seed(&mut self, seed: u32) {
        self.seed = seed;
    }

    /// Original: `devilution::GetLCGEngineState` (engine/random.cpp:23).
    // @port engine/random.cpp|devilution::GetLCGEngineState() sha=076c78e15b36
    pub fn lcg_engine_state(&self) -> u32 {
        self.seed
    }

    /// Original: `devilution::DiscardRandomValues` (engine/random.cpp:28).
    // @port engine/random.cpp|devilution::DiscardRandomValues(unsigned count) sha=cabb0c5707c8
    pub fn discard_random_values(&mut self, count: u32) {
        for _ in 0..count {
            self.generate_seed();
        }
    }

    /// Original: `devilution::GenerateSeed` (engine/random.cpp:36).
    // @port engine/random.cpp|devilution::GenerateSeed() sha=a42ac6140f0f
    pub fn generate_seed(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(0x015A_4E35).wrapping_add(1);
        self.seed
    }

    /// Original: `devilution::AdvanceRndSeed` (engine/random.cpp:42). Returns `i32::MIN` for
    /// `i32::MIN`, otherwise the absolute value of the new state read as signed.
    // @port engine/random.cpp|devilution::AdvanceRndSeed() sha=8d1c2bb9b00a
    pub fn advance_rnd_seed(&mut self) -> i32 {
        let seed = self.generate_seed() as i32;
        if seed == i32::MIN { i32::MIN } else { seed.abs() }
    }

    /// Original: `devilution::GenerateRnd` (engine/random.cpp:49). Uses the high bits for limits up
    /// to 0x7FFF; can return a negative value in (-v, -1] when `advance_rnd_seed` returns `i32::MIN`.
    // @port engine/random.cpp|devilution::GenerateRnd(int32_t v) sha=742ae1250e17
    pub fn generate_rnd(&mut self, v: i32) -> i32 {
        if v <= 0 {
            return 0;
        }
        if v <= 0x7FFF {
            return (self.advance_rnd_seed() >> 16) % v;
        }
        self.advance_rnd_seed() % v
    }

    /// Original: `devilution::FlipCoin` (engine/random.cpp:58). Default frequency in the original is 2.
    // @port engine/random.cpp|devilution::FlipCoin(unsigned frequency) sha=1cbdec0708ff
    pub fn flip_coin(&mut self, frequency: u32) -> bool {
        self.generate_rnd(frequency as i32) == 0
    }

    /// Original: `devilution::PickRandomlyAmong` (engine/random.hpp:92).
    // @port engine/random.hpp|devilution::PickRandomlyAmong(const std::initializer_list<T> &values) sha=89a45c22bd1e
    pub fn pick_randomly_among<T: Copy>(&mut self, values: &[T]) -> T {
        let index = self.generate_rnd(values.len() as i32).max(0);
        values[index as usize]
    }

    /// Original: `devilution::RandomIntLessThan` (engine/random.hpp:107).
    // @port engine/random.hpp|devilution::RandomIntLessThan(int32_t v) sha=012aa2e5e5a3
    pub fn random_int_less_than(&mut self, v: i32) -> i32 {
        self.generate_rnd(v).max(0)
    }

    /// Original: `devilution::RandomIntBetween` (engine/random.hpp:119). `half_open` defaults to false
    /// in the original.
    // @port engine/random.hpp|devilution::RandomIntBetween(int32_t min, int32_t max, bool halfOpen = false) sha=376ccdfc7358
    pub fn random_int_between(&mut self, min: i32, max: i32, half_open: bool) -> i32 {
        self.random_int_less_than(max - min + if half_open { 0 } else { 1 }) + min
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borland_lcg_sequence() {
        let mut r = DiabloRng::default();
        r.set_rnd_seed(0);
        assert_eq!(r.generate_seed(), 1);
        assert_eq!(r.generate_seed(), 0x015A_4E36);
        assert_eq!(r.lcg_engine_state(), 0x015A_4E36);
    }

    #[test]
    fn generate_rnd_uses_high_bits_for_small_limits() {
        let mut a = DiabloRng::default();
        a.set_rnd_seed(123_456);
        let mut b = a.clone();
        let expected = ((b.generate_seed() as i32).abs() >> 16) % 100;
        assert_eq!(a.generate_rnd(100), expected);
        assert_eq!(a.generate_rnd(0), 0);
        assert_eq!(a.generate_rnd(-5), 0);
    }

    #[test]
    fn int_min_state_is_preserved() {
        // find the state whose next output is 0x80000000: s * a + 1 == 0x80000000 (mod 2^32)
        let inv_a = (0..32).fold(1u32, |x, _| x.wrapping_mul(2u32.wrapping_sub(0x015A_4E35u32.wrapping_mul(x))));
        let s = 0x7FFF_FFFFu32.wrapping_mul(inv_a);
        let mut r = DiabloRng::default();
        r.set_rnd_seed(s);
        assert_eq!(r.advance_rnd_seed(), i32::MIN);
        r.set_rnd_seed(s);
        assert!(r.generate_rnd(0x10001) < 0, "vanilla bug: negative result is kept");
    }
}
