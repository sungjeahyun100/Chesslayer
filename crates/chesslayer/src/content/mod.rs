//! Replaceable prototype content and provisional rules, not final game design.
pub mod cards;
pub mod characters;
pub mod pieces;
pub use cards::{GUN_ADJACENT_FORMATION, GUN_HALF_WIDTH, GUN_HITS_ALLIES};
pub use characters::{characters, Character};

use crate::*;
use serde::Serialize;

pub const BASE_MANA: u8 = 3;
pub const DRAW_PER_TURN: usize = 5;
pub const HAND_LIMIT: usize = 10;
pub const HASTE_ACTIONS: u8 = 2;
pub const ENCOUNTER_NAMES: [&str; 6] = [
    "홀로 선 왕",
    "왕관을 쓴 기사",
    "쌍둥이 주교",
    "폰의 성벽",
    "마하라자",
    "왕관의 근위대",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Relic {
    Haste,
    Whirlwind,
    SelfDefense,
}
impl Relic {
    pub fn name(self) -> &'static str {
        match self {
            Self::Haste => "가속",
            Self::Whirlwind => "소용돌이",
            Self::SelfDefense => "호신용품",
        }
    }
    pub fn description(self) -> &'static str {
        match self {
            Self::Haste => "양 진영이 턴마다 최대 2회 이동합니다.",
            Self::Whirlwind => "전투 시작 시 적들끼리 시작 위치를 무작위로 교환합니다.",
            Self::SelfDefense => {
                "역습으로 아군이 죽으면 겹친 적도 제거합니다. 적 로얄은 제외됩니다."
            }
        }
    }
}
pub fn encounter(index: usize) -> Encounter {
    let royal_kind = match index {
        1 | 5 => KNIGHT,
        4 => AMAZON,
        _ => KING,
    };
    let mut enemies = vec![EnemyPiece {
        id: 100,
        kind: royal_kind,
        position: Position::new(4, 0),
    }];
    if matches!(index, 2 | 5) {
        for (i, file) in [2, 5].into_iter().enumerate() {
            enemies.push(EnemyPiece {
                id: 101 + i as u32,
                kind: BISHOP,
                position: Position::new(file, 0),
            });
        }
    }
    if index == 5 {
        for (id, file) in [(103, 3), (104, 5)] {
            enemies.push(EnemyPiece {
                id,
                kind: PAWN,
                position: Position::new(file, 1),
            });
        }
    }
    if index == 3 {
        for file in 0..8 {
            enemies.push(EnemyPiece {
                id: 101 + file as u32,
                kind: PAWN,
                position: Position::new(file, 1),
            });
        }
    }
    Encounter {
        enemies,
        royal_id: 100,
    }
}
/// Small deterministic generator: the same seed and commands yield the same run, including WASM.
#[derive(Clone)]
pub struct Rng(u64);
impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(u64::from(seed) + 0x9e3779b97f4a7c15)
    }
    pub fn index(&mut self, len: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % len.max(1) as u64) as usize
    }
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.index(i + 1);
            items.swap(i, j);
        }
    }
}
