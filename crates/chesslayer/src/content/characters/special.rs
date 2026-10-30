use super::{Character, Relic};

pub const DEFINITION: Character = Character {
    id: "special",
    name: "특수부대",
    title: "보이지 않는 전선",
    description:
        "폰 없는 여덟 정예대원. 정찰병의 도약, 저격과 침투·철수 카드로 정밀하게 로얄을 노립니다.",
    relic: Relic::ReconScope,
    unlock_floor: 4,
};
