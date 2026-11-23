//! The simulation's only source of randomness: a seeded PCG32 generator.
//!
//! `ARCHITECTURE.md > Determinism`: no OS randomness. Every random decision draws from
//! the [`Rng`] stored in the [`crate::Sim`], which is seeded from `Scenario::seed`.
//!
//! The algorithm is PCG32 (M. E. O'Neill, "PCG: A Family of Simple Fast Space-Efficient
//! Statistically Good Algorithms for Random Number Generation", 2014): a 64-bit LCG with
//! the XSH RR output function. It matches the reference `pcg32_srandom_r`,
//! `pcg32_random_r` and `pcg32_boundedrand_r` of `pcg-c-basic`, and the tests check it
//! against that library's published demo output.

use serde::{Deserialize, Serialize};

use crate::hash::{StateHash, StateHasher};

/// LCG multiplier of the reference implementation.
const MULTIPLIER: u64 = 6_364_136_223_846_793_005;

/// Stream of the sim's generator. It is the reference demo's stream, so
/// `Rng::from_seed(42)` reproduces the reference demo (`pcg32_srandom_r(&rng, 42, 54)`).
const SIM_STREAM: u64 = 54;

/// PCG32 generator state. Serialized and hashed with the rest of the sim state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rng {
    state: u64,
    /// LCG increment, always odd. It selects one of 2^63 independent streams.
    inc: u64,
}

impl Rng {
    /// Seeds like the reference `pcg32_srandom_r(&rng, seed, stream)`.
    pub fn new(seed: u64, stream: u64) -> Self {
        let mut rng = Self {
            state: 0,
            inc: (stream << 1) | 1,
        };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    /// The sim's generator for a scenario seed.
    pub fn from_seed(seed: u64) -> Self {
        Self::new(seed, SIM_STREAM)
    }

    /// Uniform over all `u32` values (reference `pcg32_random_r`).
    pub fn next_u32(&mut self) -> u32 {
        let old = self.state;
        self.state = old.wrapping_mul(MULTIPLIER).wrapping_add(self.inc);
        let xorshifted = (((old >> 18) ^ old) >> 27) as u32;
        let rot = (old >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Uniform over all `u64` values: two draws, the first one in the high half.
    pub fn next_u64(&mut self) -> u64 {
        let hi = u64::from(self.next_u32());
        let lo = u64::from(self.next_u32());
        (hi << 32) | lo
    }

    /// Uniform in `0..bound`, without modulo bias (reference `pcg32_boundedrand_r`).
    ///
    /// Panics if `bound` is 0, which is a programmer error: check for an empty range first.
    pub fn below(&mut self, bound: u32) -> u32 {
        assert!(bound > 0, "Rng::below(0): empty range");
        // Rejecting values below 2^32 mod bound leaves a multiple of `bound` equally likely values.
        let threshold = bound.wrapping_neg() % bound;
        loop {
            let r = self.next_u32();
            if r >= threshold {
                return r % bound;
            }
        }
    }

    /// Uniform in `[0, 1)`, in steps of 2^-24, so every result is exact in `f32`.
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 * (1.0 / 16_777_216.0)
    }
}

impl StateHash for Rng {
    fn hash(&self, h: &mut StateHasher) {
        let Rng { state, inc } = self;
        state.hash(h);
        inc.hash(h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_hash(rng: &Rng) -> u64 {
        let mut h = StateHasher::new();
        rng.hash(&mut h);
        h.finish()
    }

    /// Published output of the reference `pcg32-demo` (`pcg-c-basic`), round 1, seeded with
    /// `pcg32_srandom_r(&rng, 42u, 54u)`: six 32-bit values, then 65 coin flips
    /// (`boundedrand(2)`, 1 = H) and 33 dice rolls (`boundedrand(6) + 1`).
    #[test]
    fn matches_the_reference_demo() {
        let mut rng = Rng::new(42, 54);
        let values: Vec<u32> = (0..6).map(|_| rng.next_u32()).collect();
        assert_eq!(
            values,
            [0xa15c02b7, 0x7b47f409, 0xba1d3330, 0x83d2f293, 0xbfa4784b, 0xcbed606e]
        );
        let coins: String = (0..65).map(|_| if rng.below(2) == 1 { 'H' } else { 'T' }).collect();
        assert_eq!(
            coins,
            "HHTTTHTHHHTHTTTHHHHHTTTHHHTHTHTHTTHTTTHHHHHHTTTTHHTTTTTHTTTTTTTHT"
        );
        let rolls: Vec<u32> = (0..33).map(|_| rng.below(6) + 1).collect();
        assert_eq!(
            rolls,
            [
                3, 4, 1, 1, 2, 2, 3, 2, 4, 3, 2, 4, 3, 3, 5, 2, 3, 1, 3, 1, 5, 1, 4, 1, 5, 6, 4, 6, 6, 2, 6, 3, 3
            ]
        );
    }

    #[test]
    fn sim_seeding_uses_the_reference_stream() {
        assert_eq!(Rng::from_seed(42), Rng::new(42, 54));
        assert_ne!(Rng::from_seed(1), Rng::from_seed(2));
    }

    /// Golden values for the seed the built-in scenarios use. If these change, every
    /// recorded replay and state hash changes too.
    #[test]
    fn golden_values_for_scenario_seed_1() {
        let mut rng = Rng::from_seed(1);
        let values: Vec<u32> = (0..4).map(|_| rng.next_u32()).collect();
        assert_eq!(values, [2607537577, 204602440, 3466066159, 2183922644]);
        assert_eq!(rng.next_u64(), 10682130337640078819);
        assert_eq!(rng.below(1000), 256);
        assert_eq!(rng.next_f32().to_bits(), 1061778571); // 0.78693455
    }

    #[test]
    fn below_stays_in_range_and_covers_it() {
        let mut rng = Rng::from_seed(7);
        let mut seen = [false; 10];
        for _ in 0..1000 {
            let v = rng.below(10);
            assert!(v < 10);
            seen[v as usize] = true;
        }
        assert!(seen.iter().all(|&s| s));
        assert_eq!(rng.below(1), 0);
        assert!(rng.below(u32::MAX) < u32::MAX);
    }

    #[test]
    #[should_panic(expected = "empty range")]
    fn below_zero_panics() {
        Rng::from_seed(1).below(0);
    }

    #[test]
    fn next_f32_is_in_unit_interval() {
        let mut rng = Rng::from_seed(3);
        for _ in 0..10_000 {
            let v = rng.next_f32();
            assert!((0.0..1.0).contains(&v));
        }
    }

    #[test]
    fn hash_covers_state_and_stream() {
        let base = Rng::from_seed(5);
        let mut advanced = base.clone();
        advanced.next_u32();
        assert_ne!(state_hash(&base), state_hash(&advanced));
        assert_ne!(state_hash(&Rng::new(5, 1)), state_hash(&Rng::new(5, 2)));
        assert_eq!(state_hash(&base), state_hash(&Rng::from_seed(5)));
    }
}
