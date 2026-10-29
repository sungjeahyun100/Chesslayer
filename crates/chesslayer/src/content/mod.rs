//! Replaceable prototype content and provisional rules, not final game design.
pub mod cards;
pub mod encounters;
pub mod pieces;
pub mod relics;
pub mod squads;
pub use cards::{GUN_ADJACENT_FORMATION, GUN_HALF_WIDTH, GUN_HITS_ALLIES};
pub use encounters::{encounter, ENCOUNTERS};
pub use relics::{Relic, INITIATIVE_TURNS, RELIC_REWARD_POOL};
pub use squads::{squads, Squad};

pub const BASE_MANA: u32 = 3;
pub const DRAW_PER_TURN: usize = 1;
pub const HAND_LIMIT: usize = 10;
pub const HASTE_ACTIONS: u8 = 2;
/// Small deterministic generator: the same seed and commands yield the same run, including WASM.
#[derive(Clone)]
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(u64::from(seed) + 0x9e3779b97f4a7c15)
    }
    pub fn index(&mut self, len: usize) -> usize {
        let len = len.max(1) as u64;
        // Xorshift visits 1..=u64::MAX. Reject the tail for equally sized buckets.
        let limit = u64::MAX - u64::MAX % len;
        loop {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            if self.0 <= limit {
                return (self.0 % len) as usize;
            }
        }
    }
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.index(i + 1);
            items.swap(i, j);
        }
    }
}
