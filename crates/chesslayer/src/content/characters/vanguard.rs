use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "vanguard",
    name: "선봉대",
    title: "한 발 더 빠르게",
    description: "기동 보병과 지휘관, 민첩과 희생으로 전선을 돌파합니다. 한 턴에 부대 전체가 두 번 행동합니다.",
    relic: Relic::Haste,
    unlock_floor: 1,
};
