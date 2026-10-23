use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "vanguard",
    name: "선봉대",
    title: "한 수 더, 한 발 먼저",
    description: "가속으로 연속 두 수를 둡니다. 적도 두 번 움직이므로 킹을 지키세요.",
    relic: Relic::Haste,
};
