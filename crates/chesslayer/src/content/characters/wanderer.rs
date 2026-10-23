use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "wanderer",
    name: "방랑자",
    title: "흐트러진 진형 속으로",
    description: "소용돌이로 적의 배치를 뒤섞습니다. 다음 전투의 시작 위치를 읽어 보세요.",
    relic: Relic::Whirlwind,
};
